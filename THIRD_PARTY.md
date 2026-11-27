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
| **cool_fox** (codec_d), and through it DexNom, Belobug, FQY55kai, myokex, Geckomacabre, cxy0072019-sketch | the direct base of this fork: residency, pareto settings, thin lists and more (`docs/README-cool_fox.md`) | GPL-3.0 (VIctory's) |
| [stellarr-studio/kelvaroq](https://github.com/stellarr-studio/kelvaroq) | paired-lane wave64 translation (`emit/bitcode/paired.rs`) | GPL-3.0 (VIctory's) |
| [zm0kiemedia/Neptune_VI](https://github.com/zm0kiemedia/Neptune_VI) | FidelityFX redirect (`external.rs`, `fidelityfx.rs`), thin-list `caught_up` | GPL-3.0 (VIctory's) |
| [trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA) | pipeline wait-once, `VICTORY_BVH_MISS`, ambient cap, in-memory title patches (from victory-perf / [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort)) | GPL-3.0 (VIctory's) |
| [rorodev6/victory-nvidia-megafork](https://github.com/rorodev6/victory-nvidia-megafork) | texture read-backs on their own thread (`VICTORY_TEXBACK_ASYNC`), the pipeline panic fix as it carried it | GPL-3.0 (VIctory's) |
| "willy" cool_fox build | R11G11B10 stand-in for R9G9B9E5 targets (`VICTORY_E5_TARGETS`) | GPL-3.0 (VIctory's) |
| [clankenjoyer/scarlet-fps](https://github.com/clankenjoyer/scarlet-fps) | the 60 fps frame interval and the title's frame-rate read (`frame.rs`, `victory-loader/src/frame_rate.rs`); opt-in: `VICTORY_GPU_BACK_MAX_KB`, `VICTORY_ASK_QUIET` (after Neptune), `VICTORY_PORT_CONSTANTS` (`port.rs`) | GPL-3.0 (VIctory's) |
| [Yaekai/victory-scarlett-3070](https://github.com/Yaekai/victory-scarlett-3070) (after Belobug/VIctory-Belobug-AMD) | opt-in descriptor-heap sync changes (`VICTORY_SYNC_FINE`, `VICTORY_SYNC_IN_PLACE`, the `safecopy` compare) | GPL-3.0 (VIctory's) |
| [voidopcode/vilocity](https://github.com/voidopcode/vilocity) (finding of RyanCase94, CuteBooty, KainanP) | opt-in 2 MiB sections for the title's physical pages (`VICTORY_SECTION_MB`) | GPL-3.0 (VIctory's) |
| [trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA) (fxceless issue #1243, Vice-Tay) | opt-in thread ticket for console lanes in several host waves (`VICTORY_THREAD_TICKET`) | GPL-3.0 (VIctory's) |
| [Marvisak/VIctory](https://github.com/Marvisak/VIctory) PRs #175, #463, #969, issue #753 | loading the DirectX 12 Agility SDK at run time (`agility.rs`) | GPL-3.0 |

Each ported change names its source in a comment and in [PROGRESS.md](PROGRESS.md).

### Left out on purpose

- `console_insn.rs` / `console_insn_decode.rs` of cool_fox, ported from
  [xcl-dev/xcl](https://github.com/xcl-dev/xcl): GPL-2.0-only, which cannot be combined with
  GPL-3.0. VIctory's own emulation of the same instructions (`crates/victory-hle/src/stubs.rs`)
  is used instead.

## Notices inside source files

- `crates/victory-formats/src/texture.rs`: parts follow AMD's addrlib (Copyright Advanced Micro
  Devices, Inc., MIT); its notice is kept in the file.
- `crates/victory-shader/src/gfx10/opcodes.rs`: opcode table after Mesa's ACO (Copyright Valve
  Corporation, MIT); its notice is kept in the file.
- `crates/victory-formats/src/bcpack-tables.bin`: the BCPack format's constant tables, which the
  repository's licence does not cover (docs/asset-policy.md in the source).

## Rust crates

Fetched by Cargo from crates.io at build time (`Cargo.lock`), not stored here. On Windows x64 the
build uses 84 of them:

| Licence | Crates |
|---|---|
| MIT OR Apache-2.0 | 62 (among them `windows`, `windows-sys`, `tracing`, `serde`, `toml`, `clap`, `anyhow`) |
| MIT | 10 |
| Unlicense OR MIT | `aho-corasick`, `memchr`, `same-file`, `walkdir`, `winapi-util` |
| CC0-1.0 OR Apache-2.0 (WITH LLVM-exception) | `blake3` |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | `constant_time_eq` |
| (MIT OR Apache-2.0) AND Unicode-3.0 | `unicode-ident` |
| BSD-3-Clause | `opus-pure` |
| MIT (with mimalloc's C sources, Microsoft, MIT) | `mimalloc`, `libmimalloc-sys`: the global allocator |
| Zlib | `zlib-rs` |

`cargo metadata --filter-platform x86_64-pc-windows-msvc` lists every crate with its licence.

## Shipped in the Windows release (not in the source)

- `bin\victory.exe`: built from this source (`cargo build --profile fast --locked -p victory`,
  `-Ctarget-cpu=x86-64-v3`). The licences of the crates compiled into it are in
  `THIRD_PARTY_LICENSES.txt`.
- `fsr\amd_fidelityfx_upscaler_dx12.dll`: AMD FidelityFX SDK, FSR 4.1.1 (file version 4.1.1.2740),
  AMD-signed, unmodified. Redistributed in binary form under AMD's licence, which permits that and
  is reproduced in `fsr\LICENSE-AMD-FidelityFX.md`. Copyright (C) Advanced Micro Devices, Inc.

## Fetched on the first start (not in this download)

- `bin\D3D12\` (`tools\setup-agility.ps1`): Microsoft's DirectX 12 Agility SDK runtime
  (`D3D12Core.dll`, `d3d12SDKLayers.dll`), package Microsoft.Direct3D.D3D12 1.620.2 from nuget.org,
  checked against the SHA-512 nuget.org publishes, under Microsoft's licence (put beside it as
  `LICENSE-Microsoft-Agility-SDK.txt`). Without it Windows' own runtime is used.
- `dlss\` (`tools\setup-dlss.ps1`, only with `START-DLSS.bat`): OptiScaler 0.9.5 (GPL-3.0,
  https://github.com/optiscaler/OptiScaler) from its GitHub release, checked against its SHA-256,
  with the AMD FidelityFX and the licences it carries (`dlss\Licenses\`); and NVIDIA's
  `nvngx_dlss.dll`, copied unmodified from a game the user has installed, never downloaded.

## Not included

- Your own dump of the game.
- Microsoft's Direct3D 12 and the graphics driver, as installed with Windows and your driver.
