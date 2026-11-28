# Third-party code and notices

This program is GPL-3.0-or-later (LICENSE), as VIctory, the project it is a fork of.
Everything below is either part of that lineage (same licence, authors credited) or under a
licence that GPL-3.0 allows to be combined with it. Nothing of the game is included.

## Projects this code comes from

| Project | What of it is here | Licence |
|---|---|---|
| [Marvisak/VIctory](https://github.com/Marvisak/VIctory) (Marvisak and contributors) | the base: loader, runtime services, Direct3D 12 back end, shader recompiler | GPL-3.0 |
| [fxceless/victory-fork](https://github.com/fxceless/victory-fork) (formerly victory-gta6) | fork the AMD line grew from; pipeline job completion after a panic (#1353, Filip722) | GPL-3.0 (VIctory's) |
| [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX) | threading, write relay, card fills, copy queue, texture read-back | GPL-3.0 (VIctory's) |
| **cool_fox** (codec_d), and through it DexNom, Belobug, FQY55kai, myokex, Geckomacabre, cxy0072019-sketch | the direct base of this fork: residency, pareto settings, thin lists and more | GPL-3.0 (VIctory's) |
| [stellarr-studio/kelvaroq](https://github.com/stellarr-studio/kelvaroq) | paired-lane wave64 translation (`emit/bitcode/paired.rs`) | GPL-3.0 (VIctory's) |
| [zm0kiemedia/Neptune_VI](https://github.com/zm0kiemedia/Neptune_VI) | FidelityFX redirect (`external.rs`, `fidelityfx.rs`), thin-list `caught_up` | GPL-3.0 (VIctory's) |
| [trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA) | pipeline wait-once, `VICTORY_BVH_MISS`, ambient cap, in-memory title patches (from victory-perf / [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort)) | GPL-3.0 (VIctory's) |
| [rorodev6/victory-nvidia-megafork](https://github.com/rorodev6/victory-nvidia-megafork) | texture read-backs on their own thread (`VICTORY_TEXBACK_ASYNC`), the pipeline panic fix as it carried it | GPL-3.0 (VIctory's) |
| "willy" cool_fox build | R11G11B10 stand-in for R9G9B9E5 targets (`VICTORY_E5_TARGETS`) | GPL-3.0 (VIctory's) |
| [clankenjoyer/scarlet-fps](https://github.com/clankenjoyer/scarlet-fps) | the 60 Hz menu and video interval and the title's frame-rate read (`frame.rs`, `victory-loader/src/frame_rate.rs`); opt-in: `VICTORY_GPU_BACK_MAX_KB`, `VICTORY_ASK_QUIET` (after Neptune), `VICTORY_PORT_CONSTANTS` (`port.rs`) | GPL-3.0 (VIctory's) |
| [Yaekai/victory-scarlett-3070](https://github.com/Yaekai/victory-scarlett-3070) (after Belobug/VIctory-Belobug-AMD) | opt-in descriptor-heap sync changes (`VICTORY_SYNC_FINE`, `VICTORY_SYNC_IN_PLACE`, the `safecopy` compare) | GPL-3.0 (VIctory's) |
| [voidopcode/vilocity](https://github.com/voidopcode/vilocity) (finding of RyanCase94, CuteBooty, KainanP) | opt-in 2 MiB sections for the title's physical pages (`VICTORY_SECTION_MB`) | GPL-3.0 (VIctory's) |
| [trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA) (fxceless issue #1243, Vice-Tay) | opt-in thread ticket for console lanes in several host waves (`VICTORY_THREAD_TICKET`) | GPL-3.0 (VIctory's) |
| [Marvisak/VIctory](https://github.com/Marvisak/VIctory) PRs #175, #463, #969, issue #753 | loading the DirectX 12 Agility SDK at run time (`agility.rs`) | GPL-3.0 |
| [Biskwit/VIctory-reloaded](https://github.com/Biskwit/VIctory-reloaded) | opt-in CPU placement, split submission and offline preparation | GPL-3.0-or-later |

Each ported change names its source in a comment and in [PROGRESS.md](PROGRESS.md).

### Left out on purpose

- `console_insn.rs` / `console_insn_decode.rs` of cool_fox, ported from
  [xcl-dev/xcl](https://github.com/xcl-dev/xcl): GPL-2.0-only, which cannot be combined with
  GPL-3.0. VIctory's own emulation of the same instructions (`crates/victory-loader/src/crash.rs`)
  is used with the v1.1 write-site helpers in `crates/victory-loader/src/stubs.rs`.
  See the [GNU compatibility guidance](https://www.gnu.org/licenses/gpl-faq.html#v2v3Compatibility).

## Notices inside source files

- `crates/victory-formats/src/texture.rs`: parts follow AMD's addrlib (Copyright Advanced Micro
  Devices, Inc., MIT); its notice is kept in the file.
- `crates/victory-shader/src/gfx10/opcodes.rs`: opcode table after Mesa's ACO (Copyright Valve
  Corporation, MIT); its notice is kept in the file.
- `crates/victory-formats/src/bcpack-tables.bin`: the BCPack format's constant tables, which the
  repository's licence does not cover (docs/asset-policy.md in the source).

- `crates/victory-gpu/src/streamline/abi.rs` mirrors the public NVIDIA Streamline 2.14.16 ABI.
  The NVIDIA header attribution and MIT terms are in [LICENSE-Streamline.txt](LICENSE-Streamline.txt).
- `apps/victory-launcher/LICENSE-zip.txt` carries the ZIP crate's copyright and MIT terms.

## Rust crates

Cargo fetches third-party Rust crates using the checked-in lockfiles. The Windows x64
dependency metadata lists 245 third-party package versions for the workspace. The declared
terms are permissive (such as MIT, Apache-2.0, BSD, BSL, ISC, Zlib and Unicode-3.0) or give a
compatible permissive option. In particular, self_cell is used under its Apache-2.0 option.
The egui default font assets carry their own SIL OFL 1.1 and Ubuntu Font Licence terms.
A downstream binary distributor should carry the applicable dependency and asset notices.

Run `cargo metadata --locked --filter-platform x86_64-pc-windows-msvc --format-version 1`
to inspect package versions, their license expressions and the dependency graph.

## Runtime dependencies

- [Microsoft DirectX 12 Agility SDK 1.620.2](https://www.nuget.org/packages/Microsoft.Direct3D.D3D12/1.620.2)
  is fetched by `tools/setup-agility.ps1` and checked against a pinned SHA-512. Its license
  travels with the downloaded installation.
- [OptiScaler 0.9.5](https://github.com/optiscaler/OptiScaler/releases/tag/v0.9.5) is fetched with its
  accompanying notices and checked against the pinned upstream SHA-256. OptiScaler is GPL-3.0.
  The script uses a separately supplied, locally installed NVIDIA DLSS DLL.
- [NVIDIA Streamline 2.14.16](https://github.com/NVIDIA-RTX/Streamline/releases/tag/v2.14.16)
  is fetched from NVIDIA's official release, checked against the published SHA-256, and
  validated with NVIDIA Authenticode signatures. The vendor's SDK and plugin terms apply.
- The AMD FidelityFX upscaler DLL, Microsoft graphics components, GPU drivers and the game
  dump are supplied separately.
