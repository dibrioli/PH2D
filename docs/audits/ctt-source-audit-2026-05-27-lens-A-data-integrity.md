# ctt 0.4.0 Source Audit — Lens A: Data-Integrity (2026-05-27)

Auditor: Claude Opus 4.7 sub-agent, adversarial lens A.
Target: `ctt 0.4.0` + wrapper sub-crates in `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.
Scope: data-integrity, determinism, encoder dispatch, FFI safety. Time-boxed ~50 min.
Decision context: ADR-0055-v4 Accepted (today) — gate decides whether PH2D pins `ctt = "0.4.0"` in W1.T2 of `docs/plans/2026-05-texture-compression-waves.md`.

## Resumo executivo
- Total LOC auditadas: ~5.8k (processing/* 1664 + encoders/* 1851 + format/convert/vk_format 2030 + output/ktx2 434 + ctt-bc7enc-rdo 197 + ctt-astcenc dispatch ~700). FFI vendored C/C++ NOT audited (out of scope).
- Findings: **0 CRITICAL hardcoded bugs**, **3 HIGH determinism risks**, **2 MEDIUM API/wrapper concerns**, **2 LOW code-quality notes**.
- Recomendação: **APPROVE_WITH_CAVEATS** — adoption viable for offline cooker IF PH2D pins (1) canonical runner CPU class with a fixed feature set, (2) ctt features `encoder-bc7enc + encoder-astcenc` ONLY (avoid compressonator BC7 ultrafast), (3) Quality preset ≥ `Slow` for compressonator BC7, (4) the ctt 0.4.0 exact version (KTX2 KVD embeds `CARGO_PKG_VERSION`).

## Findings detalhados

### CRITICAL: nenhum CRITICAL identificado.
The processing pipeline (load → swizzle/mipmap → store → encode) is structurally sound. Stride math is consistent. `tile_to_blocks` (surface.rs:338-376) correctly clamps edges with `min(max_x/y)`. Surface output buffers are pre-sized to exact `blocks_x * blocks_y * bytes_per_block` (no truncation possible from short writes). Alpha premul on load + unpremul on store (load.rs:77-79, store.rs:50-52) is symmetric. KTX2 output (output/ktx2.rs:172-195) allocates `total_size` then writes section-by-section with no overlapping ranges. mipmap.rs uses `use_alpha(false)` (line 59) which is **correct** because the pipeline already produces premultiplied buffers — `fast_image_resize` would otherwise double-(un)premultiply.

### HIGH-1: Encoder ISA dispatch is runtime CPU-detected → cross-machine non-determinism
- **Onde**: `ctt-astcenc-0.4.0/src/lib.rs:628-655` (`init_dispatch` picks `avx2/sse41/sse2` via `is_x86_feature_detected!`). Same pattern in `ctt-0.4.0/src/processing/load_kernels/srgb.rs:56-72` (AVX-512 / AVX2+FMA / SSE4.1 / NEON fallback ladder, runtime-selected). Comment at `srgb.rs:629-642` test admits "within u8 tolerance" of LUT — i.e., **SIMD paths produce different f32 intermediates** vs scalar, fed into the encoder verbatim.
- **Por que importa para PH2D**: HR-6 (`AssetId = blake3(cooked_bytes)`) requires byte-identical output for a given input + settings + version. Two CI runners with different CPU feature sets (e.g., one Intel Skylake without AVX-512, one Ice Lake with it) will execute different ISA paths in astcenc / sRGB load, producing different float intermediates and likely different encoded blocks. Output bytes diverge → AssetId diverges → cache misses + Git LFS thrash + asset-cooker drift across contributors.
- **Reprodução**: cook the same RGBA8 sRGB asset to `ASTC_8x8` on a Linux x86_64 host with `RUSTFLAGS="-C target-cpu=native"` on (a) AMD Zen 1 (AVX2 max), (b) AMD Zen 4 (AVX-512). Diff the resulting `.ktx2` byte-by-byte. Expect divergence in payload bytes; identical headers.
- **Mitigação**: PH2D must (1) pin canonical runner to one CPU class (e.g., GitHub Actions `ubuntu-22.04` runner is consistently AVX2-capable but NOT AVX-512 — verifiable via `/proc/cpuinfo`), (2) document in ADR-0055-v4 that cooker output is **only** reproducible on the canonical runner, (3) require asset cooking artifacts originate from CI not local dev machines, (4) consider adding a CI gate that prints CPU features before cooking so divergence is detectable. **No upstream fix needed** — this is by design.

### HIGH-2: Compressonator BC7 + UltraFast produces R=0 on non-MSVC toolchains (acknowledged data corruption)
- **Onde**: `ctt-0.4.0/src/encoders/compressonator.rs:296-300` — explicit comment in test code:
  ```
  // Compressonator BC7 at UltraFast produces R=0 output on non-MSVC
  // toolchains (Linux, macOS). Use Slow so the NPOT coverage is
  // independent of that upstream quirk.
  ```
  The test works around by using `Quality::Slow`. There is **no runtime guard** in `compressonator.rs:212-223` rejecting the (BC7 + UltraFast) combination on Linux/macOS — a user can ask for it and get black-red output.
- **Por que importa para PH2D**: PH2D's canonical runner is Linux x86_64. If anyone selects (BC7, UltraFast, Compressonator), the cooked texture is silently corrupted (red channel zeroed). The ASTC-mobile + BC7-desktop tier strategy in ADR-0055-v4 must avoid this combination.
- **Reprodução**: `CompressonatorEncoder::compress(rgba_red_surface, BC7_UNORM_BLOCK, Quality::UltraFast, ...)` on Linux → decoded R channel ≈ 0.
- **Mitigação**:
  - Tactical (PH2D wrapper): in the PH2D cooker, **forbid** `Encoder::Amd + Quality::UltraFast` for BC7 targets, OR force `encoder-amd` feature OFF in ctt features (recommended — the BC7 path goes via `encoder-bc7enc` / `encoder-intel` anyway and the Auto dispatch order `bc7enc → intel → etcpak → amd → astcenc` (mod.rs:67-92) ensures bc7enc wins for BC7 when compiled in).
  - Long-term: file upstream issue against ctt to either guard or fix.

### HIGH-3: Encoder Auto dispatch order is compile-time feature-dependent → output silently changes if features differ
- **Onde**: `ctt-0.4.0/src/encoders/mod.rs:54-92` (`compiled_in_encoders()` priority list), `ctt-0.4.0/src/processing/encode.rs:85-111` (`pick_auto()` returns first supported). Auto-dispatch order = bc7enc → intel → etcpak → amd → astcenc.
- **Por que importa para PH2D**: If two developers build the cooker with different `--features` sets (e.g., one with `encoder-intel`, one without), the Auto-selected encoder for BC7 silently changes → different bytes → different AssetId. This is not a bug per se but a deployment hazard.
- **Reprodução**: build cooker A with `--features encoder-bc7enc,encoder-intel`, cooker B with `--features encoder-bc7enc` only. Cook the same RGBA → BC7. Compare bytes — should be identical because bc7enc wins in both (it's first in priority), but if A had only `encoder-intel`, output would diverge.
- **Mitigação**: in PH2D's `Cargo.toml` for the cooker crate, set `default-features = false` and explicitly list features (e.g., `features = ["encoder-bc7enc", "encoder-etcpak", "encoder-astcenc"]`). NEVER rely on default features. Pin in CI canonical-runner build script.

### MEDIUM-1: astcenc wrapper does a `*const → *mut` cast of caller's read-only data
- **Onde**: `ctt-0.4.0/src/encoders/astcenc.rs:200`:
  ```rust
  let mut data_ptr = surface.data.as_ptr() as *mut std::ffi::c_void;
  ```
  Combined with `&Surface` (immutable borrow) → handing `*mut` to the C API. The astcenc C API takes `astcenc_image::data` as `void**` because for 3D images it expects `dim_z` slice pointers; for 2D (`dim_z = 1`) it reads `data[0]` and uses the pointed buffer as **input** to compression (read-only in practice). astcenc's compress path does not write to the input buffer in the documented API.
- **Por que importa para PH2D**: If a future astcenc upstream version (or a custom build flag) ever writes through `data[0]` (e.g., for in-place swizzle), this would be undefined behavior — mutation through a pointer derived from a shared borrow. PH2D would observe occasional non-determinism or memory corruption depending on layout.
- **Reprodução**: stress test with miri or sanitizer (`MIRIFLAGS=-Zmiri-disable-isolation`) on a workload that calls `AstcencEncoder::compress` from multiple threads on the same `Surface`. Today this is safe because the wrapper above (`Context`) is single-threaded.
- **Mitigação**: PH2D should not stress-test parallel cooks on the same source `Surface` reference. Prefer cooking one asset at a time per worker (already the design intent). Optionally file upstream issue asking ctt to make a defensive copy before the cast, or to take `&mut Surface`.

### MEDIUM-2: encode.rs silent fallbacks on unknown block size / bpp
- **Onde**: `ctt-0.4.0/src/processing/encode.rs:59-60`:
  ```rust
  let bpp_block = step.target_format.bytes_per_block().unwrap_or(16) as u32;
  let (bw, _bh) = step.target_format.block_size().unwrap_or((4, 4));
  ```
  If a format slips past `vk_format::block_size()` / `bytes_per_block()` returning `Some`, the fallback `(4,4)` + 16 bytes is silently used. For real BC1/BC4 (8 bytes/block, 4x4) this would produce a Surface whose `stride` reports double the real bytes-per-row, breaking downstream tight_data packing.
- **Por que importa para PH2D**: All BC/ETC/ASTC formats currently handled by `vk_format.rs` are mapped, so this fallback is unreachable for the formats PH2D will use. But a future Vulkan format addition could regress silently. Spec gap, not active bug.
- **Mitigação**: PH2D's cooker integration test should round-trip every format ctt advertises in `compiled_in_encoders()` to detect any silent stride miscomputation. Optionally PR upstream to make these `.expect("known")` instead of `.unwrap_or(...)`.

### LOW-1: `tight_data()` panics if format is unknown
- **Onde**: `ctt-0.4.0/src/surface.rs:305-328` — `.expect("tight_data requires a known format size")` (lines 311, 314). Comment claims `Image::validate` is the gatekeeper but `validate` does NOT enforce format-known-ness across all paths.
- **Mitigação**: PH2D's cooker wrapper should call `Surface::validate` (which itself rejects unknown formats per surface.rs:83) before invoking ctt. Already best practice.

### LOW-2: `unpremultiply_f32` produces uncapped values for tiny alpha
- **Onde**: `ctt-0.4.0/src/processing/alpha.rs:19-29` — `if a > 0.0 { p[0] /= a; ... }`. No upper clamp. For `a = 0.001` and `p[0] = 0.005` (premul), result = 5.0 which then must be saturated by the store kernel.
- **Por que importa para PH2D**: Most store kernels clamp to [0,1] before quantizing to u8; if any kernel skips the clamp (e.g., F32 storage), the saturated value gets persisted as-is. Not a corruption per se — the original premultiplied content carried that ratio — but the unpremul result is mathematically meaningless. Roundtrip test (mod.rs:296-327) accepts ±1 byte error.
- **Mitigação**: doc-only; PH2D should not produce sources with `a < 1/256` and significant RGB, or should pre-bake premultiplication itself.

## Encoder backend dispatch analysis
- **BC7 default backend (Auto)**: `Bc7encEncoder` (ctt_bc7enc_rdo) wins — first in `compiled_in_encoders()` order (mod.rs:67-71) when `encoder-bc7enc` feature is enabled. Fall-through: Intel ISPC → Compressonator → (Etcpak does NOT support BC7).
- **ASTC default backend (Auto)**: `AstcencEncoder` — only ctt encoder that handles `ASTC_*` formats (others' `supported_formats()` exclude ASTC). Always selected when `encoder-astcenc` enabled.
- **Como ctt escolhe?** `processing/encode.rs:85-111` `pick_auto()` iterates compile-time-gated `if` blocks in fixed order; first `supported_formats().contains(&target)` wins.
- **Determinismo da escolha?** YES given fixed `--features` set + fixed cargo `default-features` policy. Cross-build feature drift = different encoder selection (HIGH-3 above).
- **ASTC quality presets**: `ctt-0.4.0/src/encoders/astcenc.rs:282-291` maps `Quality::{UltraFast..VerySlow}` to `astc::Preset::{Fastest..Exhaustive}`. Stable mapping in 0.4.0; upstream astcenc preset values are publicly defined constants — safe to pin.
- **BC7 quality presets (bc7enc-rdo)**: `ctt-0.4.0/src/encoders/bc7enc.rs:107-114` calls `params_init_ultrafast / veryfast / fast / basic / slow / veryslow`. Each preset baked into the C++ codec — output depends on upstream codec version (pinned via ctt 0.4.0 lock).

## FFI surface summary
- `extern "C"` total declarations (sample, ctt-astcenc only): 9 functions × 3 ISA variants = 27 (x86_64) + 9 (aarch64). Other wrappers similar scale.
- `unsafe` blocks in ctt main src: only SIMD CPU intrinsics in `processing/load_kernels/srgb.rs` (legitimate `target_feature` annotated calls + `read_unaligned`). All other ctt code is safe Rust.
- `unsafe impl Send for Context`: `ctt-astcenc/src/lib.rs:371` — comment justifies (single-threaded codec, `thread_count = 1`). Acceptable.
- **No `unsafe impl Sync` for Context** — good, Context is `!Sync` by default which prevents cross-thread races on the astcenc internal state.
- Vendored encoders: bc7enc-rdo (ISPC, license: unknown — needs license audit, OUT OF SCOPE for this lens), astcenc (BSD-3 from ARM), etcpak (BSD-3 from Bartosz Taudul), Intel ISPC tex compressor (Apache-2/MIT-style from Intel), Compressonator (MIT from AMD).
- No HashMap / rayon / `std::thread::spawn` / `rand` / RNG appearance in ctt main src (verified by grep — empty output).
- No `panic!` / `unreachable!` / `unwrap()` in non-test ctt main src outside the documented-known-format `tight_data` panics (LOW-1).

## Veredito
**APPROVE_WITH_CAVEATS** (score: 7.5/10).

ctt 0.4.0 is **architecturally sound** for an offline texture cooker: single-threaded, deterministic per-machine, no random state, no parallelism in the encoder path, clean Rust abstractions over FFI, comprehensive format coverage with explicit edge-pixel clamping. The risks are real but well-bounded and **mitigable entirely from PH2D's side**:

1. Pin `default-features = false` + explicit feature list to lock the encoder dispatch order (HIGH-3).
2. Pin canonical CI runner CPU class to lock the ISA dispatch (HIGH-1) — verify with a CI step that prints CPU feature flags before cooking.
3. Exclude `encoder-amd` from ctt features OR forbid (Amd + UltraFast + BC7) in the PH2D wrapper (HIGH-2 — known upstream corruption).
4. Add a PH2D integration test that round-trips every (format, encoder, quality) the cooker actually uses and golden-snapshots the first 64 bytes of output, so any silent ctt-version drift gets caught.

The cooker is **not** safe to run on contributor laptops with the expectation that output bytes match CI — this would be a footgun for HR-6. Document explicitly in the ADR-0055-v4 reproducibility section. With these caveats acknowledged, the lib clears the data-integrity bar for the cooker role; rejecting on these grounds would be over-cautious given the alternatives (writing a Rust BC7/ASTC encoder from scratch is a 5+ KLOC effort that would itself need the same audit).

Fallback B (custom encoder) **not** required; this is a green-light with operational discipline.
