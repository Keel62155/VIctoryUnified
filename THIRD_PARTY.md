# Third-party code and notices

This program is GPL-3.0-or-later ([LICENSE](LICENSE)), as VIctory, the project it is a fork of.
Everything below is either part of that lineage (same licence, authors credited) or under a
licence that GPL-3.0 allows to be combined with it. Nothing of the game is included
([docs/asset-policy.md](docs/asset-policy.md)).

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
  repository's licence does not cover ([docs/asset-policy.md](docs/asset-policy.md)).

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

## Not included, and needed at run time

- Your own dump of the game.
- `amd_fidelityfx_upscaler_dx12.dll` (AMD, proprietary licence) from a PC game you own, for
  Series S mode on NVIDIA. It is loaded, never shipped.
- Microsoft's DXIL compiler and the graphics driver, as installed with Windows and the
  Windows SDK.
