//! Code memory of ours near the title's image, for the stubs `write_sites` jumps to.
//!
//! Written for VIctory Unified (GPL-3.0-or-later) in place of the helpers the 7900 XTX fork kept
//! in its XCL-derived `console_insn.rs` (GPL-2.0-only, left out of this program).

use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAlloc};

/// Bytes of one block of stubs.
pub(crate) const STUBS_LEN: usize = 64 << 10;

/// A `jmp rel32` placed at `at` that lands on `to`, if `to` is within its reach.
pub(crate) fn jump(at: usize, to: usize) -> Option<[u8; 5]> {
    let from = at.checked_add(5)? as i64;
    let distance = i32::try_from(to as i64 - from).ok()?;
    let d = distance.to_le_bytes();
    Some([0xE9, d[0], d[1], d[2], d[3]])
}

/// A fresh block of [`STUBS_LEN`] executable bytes that a `jmp rel32` at `at` reaches both ways,
/// or `None` if no free place was found within reach.
pub(crate) fn near(at: usize) -> Option<usize> {
    const GRAIN: usize = 64 << 10;
    // Within a little under 2 GiB either side, so that jumps back from the block reach too.
    const REACH: usize = (1 << 31) - (16 << 20);
    let base = at & !(GRAIN - 1);
    let mut step = GRAIN;
    while step < REACH {
        for place in [base.checked_sub(step), base.checked_add(step)].into_iter().flatten() {
            // SAFETY: asks for fresh memory at a free place; nothing is overwritten.
            let got = unsafe {
                VirtualAlloc(place as *const _, STUBS_LEN, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)
            } as usize;
            if got != 0 {
                return Some(got);
            }
        }
        step += GRAIN;
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_jump_reaches_forward_and_back() {
        assert_eq!(super::jump(0x1000, 0x1005), Some([0xE9, 0, 0, 0, 0]));
        assert_eq!(super::jump(0x1000, 0x1000), Some([0xE9, 0xFB, 0xFF, 0xFF, 0xFF]));
        assert_eq!(super::jump(0, 1 << 32), None);
    }
}
