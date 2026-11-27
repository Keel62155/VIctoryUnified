//! Diagnostics: how long the card itself works on our submissions, by its own
//! timestamps (`VICTORY_CARD_SPANS=1`). The clock of the frame line
//! (`card_clock_line`) counts from the call that sends a submission to the
//! moment a thread of ours wakes behind its last signal: the time the system
//! takes to hand a submission to the card and to report it done is in it, 40
//! times a frame. Here two lists made once, of nothing but a timestamp, stand
//! around each submission.

use std::sync::Mutex;

use windows::Win32::Graphics::Direct3D12::*;
use windows::core::Interface;

use super::Host;

/// Submissions stamped before the first stamps are read and used again.
const SPANS: usize = 512;

/// A ring of stamp lists, `per` to a slot: each list holds one timestamp
/// and its way into memory the PC reads.
pub(super) struct Spans {
    _allocator: ID3D12CommandAllocator,
    _heap: ID3D12QueryHeap,
    _back: ID3D12Resource,
    /// The mapped results, `per` a slot.
    data: usize,
    lists: Vec<ID3D12CommandList>,
    per: usize,
    ticks_per_ms: f64,
    next: usize,
    sums: Sums,
    /// Ticks between the stamp in front of our compare list and the end
    /// (submissions), and how many submissions had one.
    compare: u64,
    compares: u64,
}

// SAFETY: free-threaded objects and a mapping only read behind a lock.
unsafe impl Send for Spans {}

/// What the stamps read so far add up to.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Sums {
    /// Submissions, ticks the card worked on them, ticks between them.
    count: u64,
    busy: u64,
    between: u64,
    /// Ticks of the longest one, and the last one's end.
    longest: u64,
    last: u64,
}

impl Sums {
    /// A submission that ran from `start` to `end` (card ticks), after those
    /// before it. The start's stamp may be of before the last one's end: the
    /// part before that end is the last one's.
    fn add(&mut self, start: u64, end: u64) {
        if start == 0 || end < start {
            return;
        }
        let start = start.max(self.last);
        let took = end.saturating_sub(start);
        if self.last != 0 {
            self.between += start - self.last;
        }
        self.count += 1;
        self.busy += took;
        self.longest = self.longest.max(took);
        self.last = end.max(self.last);
    }
}

fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_CARD_SPANS").is_some_and(|v| v == "1"))
}

/// Submissions: three stamps each (start, in front of our compare list, end).
static SPANS_MADE: Mutex<Option<Option<Spans>>> = Mutex::new(None);
/// Our rehash lists, sent on their own ahead of a submission: two stamps each.
static SYNC_MADE: Mutex<Option<Option<Spans>>> = Mutex::new(None);

/// `ExecuteCommandLists` calls on the drawing queue since the last frame-rate
/// line (submissions' parts and our rehash lists).
pub(crate) static CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The stamp lists of a submission: in front of it, in front of our compare
/// list (or of the end, without one), behind it.
pub(super) struct SpanSet {
    pub(super) front: ID3D12CommandList,
    pub(super) mid: ID3D12CommandList,
    pub(super) behind: ID3D12CommandList,
}

impl Spans {
    /// The next slot's lists, after adding up what the card wrote into the
    /// slot the last time round (long done: [`SPANS`] slots ago).
    fn take(&mut self) -> &[ID3D12CommandList] {
        let i = self.next;
        self.next = (i + 1) % SPANS;
        let mut read = [0u64; 3];
        // SAFETY: inside the mapping; written by the card at whole words.
        unsafe {
            let at = (self.data as *mut u64).add(self.per * i);
            for (k, r) in read.iter_mut().enumerate().take(self.per) {
                *r = at.add(k).read_volatile();
                at.add(k).write_volatile(0);
            }
        }
        let (start, end) = (read[0], read[self.per - 1]);
        self.sums.add(start, end);
        if self.per == 3 && start != 0 && read[1] >= start && end >= read[1] {
            self.compare += end - read[1];
            self.compares += u64::from(end > read[1]);
        }
        &self.lists[self.per * i..self.per * (i + 1)]
    }
}

impl Host {
    /// The stamp lists for a submission about to be sent to the drawing
    /// queue, if stamps are asked for.
    pub(super) fn span_lists(&self) -> Option<SpanSet> {
        if !on() {
            return None;
        }
        let mut made = SPANS_MADE.lock().unwrap_or_else(|e| e.into_inner());
        let spans = made
            .get_or_insert_with(|| {
                self.make_spans(3)
                    .inspect_err(|e| tracing::warn!(error = %e, "no stamps around submissions"))
                    .ok()
            })
            .as_mut()?;
        let l = spans.take();
        Some(SpanSet { front: l[0].clone(), mid: l[1].clone(), behind: l[2].clone() })
    }

    /// The stamp lists in front of and behind one of our rehash lists.
    pub(super) fn sync_span_lists(&self) -> Option<[ID3D12CommandList; 2]> {
        if !on() {
            return None;
        }
        let mut made = SYNC_MADE.lock().unwrap_or_else(|e| e.into_inner());
        let spans = made.get_or_insert_with(|| self.make_spans(2).ok()).as_mut()?;
        let l = spans.take();
        Some([l[0].clone(), l[1].clone()])
    }

    fn make_spans(&self, per: usize) -> windows::core::Result<Spans> {
        let device = &self.device;
        let count = per * SPANS;
        // SAFETY: plain object creation; lists of one allocator recorded one
        // after the other.
        unsafe {
            let mut heap: Option<ID3D12QueryHeap> = None;
            device.CreateQueryHeap(
                &D3D12_QUERY_HEAP_DESC { Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP, Count: count as u32, NodeMask: 0 },
                &mut heap,
            )?;
            let heap = heap.ok_or_else(windows::core::Error::empty)?;
            let back = self.read_back_buffer(8 * count as u64).ok_or_else(windows::core::Error::empty)?;
            let mut data = std::ptr::null_mut();
            back.Map(0, None, Some(&mut data))?;
            std::ptr::write_bytes(data.cast::<u8>(), 0, 8 * count);
            let allocator: ID3D12CommandAllocator = device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
            let mut lists = Vec::with_capacity(count);
            for i in 0..count as u32 {
                let list: ID3D12GraphicsCommandList =
                    device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?;
                list.EndQuery(&heap, D3D12_QUERY_TYPE_TIMESTAMP, i);
                list.ResolveQueryData(&heap, D3D12_QUERY_TYPE_TIMESTAMP, i, 1, &back, u64::from(i) * 8);
                list.Close()?;
                lists.push(list.cast()?);
            }
            Ok(Spans {
                _allocator: allocator,
                _heap: heap,
                _back: back,
                data: data as usize,
                lists,
                per,
                ticks_per_ms: self.queue.GetTimestampFrequency()? as f64 / 1000.0,
                next: 0,
                sums: Sums::default(),
                compare: 0,
                compares: 0,
            })
        }
    }
}

/// The GPU timestamps the title was given since the last frame-rate line:
/// how many, how often a later one was 1 ms or more after the one before
/// (written with another submission), and where the first few lie.
static STAMPED: Mutex<(u64, u64, u64, Vec<usize>)> = Mutex::new((0, 0, 0, Vec::new()));

/// The title's GPU timestamp at `at` was given the value `now` (ticks of 10
/// ns).
pub(crate) fn stamped(at: usize, now: u64) {
    if !on() {
        return;
    }
    let mut s = STAMPED.lock().unwrap_or_else(|e| e.into_inner());
    s.0 += 1;
    s.1 += u64::from(now.saturating_sub(s.2) > 100_000);
    s.2 = now;
    if s.3.len() < 64 && !s.3.contains(&at) {
        s.3.push(at);
        tracing::info!(address = format_args!("{at:#x}"), now, "a GPU timestamp of the title");
    }
}

/// The viewports' sizes seen (each said once): whether the title draws its
/// scene at sizes of its choice.
pub(crate) fn viewport(width: u32, height: u32) {
    if !on() || width < 1024 || height < 576 {
        return;
    }
    static SEEN: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());
    let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
    if seen.len() < 200 && !seen.contains(&(width, height)) {
        seen.push((width, height));
        tracing::info!(width, height, "a viewport of the title");
    }
}

/// For the frame-rate line: what the stamps read since the last one say, per
/// frame (they are of some [`SPANS`] submissions ago).
pub(super) fn line(frames: f64) {
    {
        let mut s = STAMPED.lock().unwrap_or_else(|e| e.into_inner());
        if s.0 > 0 {
            tracing::info!(
                "GPU timestamps the title asked for per frame: {:.1}, in {:.1} groups written at one time",
                s.0 as f64 / frames,
                s.1 as f64 / frames
            );
        }
        (s.0, s.1) = (0, 0);
    }
    let calls = CALLS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / frames;
    let mut made = SPANS_MADE.lock().unwrap_or_else(|e| e.into_inner());
    let Some(Some(spans)) = made.as_mut() else {
        return;
    };
    let last = spans.sums.last;
    let s = std::mem::replace(&mut spans.sums, Sums { last, ..Sums::default() });
    let (compare, compares) = (std::mem::take(&mut spans.compare), std::mem::take(&mut spans.compares));
    let per_ms = spans.ticks_per_ms;
    let ms = |ticks: u64| ticks as f64 / per_ms / frames;
    let (sync_n, sync_ms) = {
        let mut sync = SYNC_MADE.lock().unwrap_or_else(|e| e.into_inner());
        match sync.as_mut() {
            Some(Some(y)) => {
                let last = y.sums.last;
                let t = std::mem::replace(&mut y.sums, Sums { last, ..Sums::default() });
                (t.count as f64 / frames, ms(t.busy))
            }
            _ => (0.0, 0.0),
        }
    };
    tracing::info!(
        "the card's own stamps per frame: {:.1} submissions in {calls:.1} ExecuteCommandLists calls, working {:.1} ms \
         (the title's lists {:.1} ms, our compares behind them {:.1} ms in {:.1}), idle between them {:.1} ms, the longest \
         {:.1} ms; our rehash lists sent ahead: {sync_n:.1}, {sync_ms:.2} ms",
        s.count as f64 / frames,
        ms(s.busy),
        ms(s.busy.saturating_sub(compare)),
        ms(compare),
        compares as f64 / frames,
        ms(s.between),
        s.longest as f64 / per_ms
    );
}

#[cfg(test)]
mod tests {
    use super::Sums;

    #[test]
    fn submissions_add_up_without_counting_anything_twice() {
        let mut s = Sums::default();
        s.add(100, 150);
        s.add(140, 200); // began before the last one ended
        s.add(260, 270);
        assert_eq!((s.count, s.busy, s.between, s.longest, s.last), (3, 110, 60, 50, 270));
        // Never written, or nonsense: nothing.
        s.add(0, 0);
        s.add(300, 290);
        assert_eq!(s.count, 3);
        // Whole inside the last one.
        s.add(265, 268);
        assert_eq!((s.count, s.busy, s.between), (4, 110, 60));
    }
}
