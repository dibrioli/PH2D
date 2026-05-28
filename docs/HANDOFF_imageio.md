# HANDOFF — Image I/O pipeline (ADR-0054)

**Status:** **W0..W3 closed** with W3.T4 AVIF *deshipped* and W3.T5 SVG
parse-only stub.
**Last commit (this session):** `eb4be4f` (docs §5.17 + plan).
**Date:** 2026-05-28.
**Owner of this handoff:** LLM (any session) — Enio reviews on demand.

> **Read order before touching imageio code:**
> 1. This file.
> 2. [`docs/architecture/decisions/0054-imageio-pipeline.md`](architecture/decisions/0054-imageio-pipeline.md) — §1 (context), §2 (caps + variant policy + golden-hash scope amendments), §5 (full execution history 5.1–5.17).
> 3. [`docs/plans/2026-05-imageio-waves.md`](plans/2026-05-imageio-waves.md) — current wave map.
> 4. Memory entries: [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md), [`feedback-no-industrial-claims-without-verification`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md), [`feedback-audit-internal-state-grep`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_internal_state_grep.md), [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md), [`feedback-parallel-agent-collision`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_parallel_agent_collision.md).

---

## §0 — One-paragraph state

The contract crate [`ph2d-imageio`](../crates/ph2d-imageio/) defines a
**FROZEN** trait surface (`ImageImporter`, `ImageExporter`,
`DecodedImage` = 5 variants, `Error` = 11 variants, `ExportFormat` = 14
variants, `ColorProfile` = 8 variants) plus a hoisted-caps surface in
[`limits.rs`](../crates/ph2d-imageio/src/limits.rs)
(`MAX_RASTER_DIMENSION`, `MAX_ANIMATION_FRAMES`, `MAX_DOCUMENT_PAGES`,
`MAX_LAYER_DEPTH`, `MAX_LAYER_COUNT`, `MAX_ARCHIVE_ENTRIES`,
`MAX_ARCHIVE_TEXT_BYTES`, `MAX_PH2D_PAYLOAD_LEN`, `MAX_ICC_PROFILE_LEN`).
14 format crates implement the trait; codegen tool
[`tools/ph2d-imageio-sync`](../tools/ph2d-imageio-sync/) scans
`crates/ph2d-imageio-*/` and regenerates `register_all_importers` /
`register_all_exporters` plus alphabetical-order + staleness arch-gates
in [`ph2d-imageio-registry-init`](../crates/ph2d-imageio-registry-init/).
The shell registry → asset bridge wire-up lives in
[`ph2d-asset/src/loader.rs::decode_via_imageio_registry`](../crates/ph2d-asset/src/loader.rs)
(W3.T1.0); user drag-drop AVIF/TIFF/PSD/ORA/APNG/GIF/.ph2d-native now
flows through the audited defences (OOM caps, NaN reject in ORA, path-
traversal reject in ORA, recursion bomb caps, EOF semantics centralised
via [`Error::from_decoder_message`](../crates/ph2d-imageio/src/error.rs)).

## §1 — Format matrix

Per [ADR-0054 §5](architecture/decisions/0054-imageio-pipeline.md). Real
= production-ready decode + (optional) encode wired through the
registry. Stub = magic recognition + `Error::Unsupported` with
actionable defer message.

| Crate | Wave | Decode | Encode | Notes |
|---|---|---|---|---|
| `ph2d-imageio-png` | W1.T1 | ✅ Real | ✅ Real | `image` 0.25; goldens |
| `ph2d-imageio-jpeg` | W1.T2 | ✅ Real | ✅ Real (quality knob) | `image` 0.25 |
| `ph2d-imageio-webp` | W1.T3 | ✅ Real | ✅ Real | `image` 0.25 |
| `ph2d-imageio-gif` | W1.T4 | ✅ Real | ✅ Real | `image` 0.25; MAX_ANIMATION_FRAMES |
| `ph2d-imageio-ph2d-native` | W1.T5 | ✅ Real | ✅ Real | postcard; HR-14 versioning; MAX_RASTER walker (audit-7 J-3) |
| `ph2d-imageio-tiff` | W2.T1 | ✅ Real | ✅ Real | `tiff` 0.11; multi-page; CMYK naive; MAX_DOCUMENT_PAGES |
| `ph2d-imageio-ora` | W2.T2 | ✅ Real | ✅ Real | OpenRaster 0.0.5; depth + count + opacity NaN + path-traversal caps |
| `ph2d-imageio-apng` | W2.T3 | ✅ Real | ✅ Real (single frame) | acTL walker; MAX_ANIMATION_FRAMES |
| `ph2d-imageio-psd` | W2.T4 | ✅ Real | ❌ Unsupported (defer) | `psd =0.3.5` pinned; `catch_unwind` wrapper; multi-layer cap |
| `ph2d-imageio-hdr-radiance` | W3.T3 | ✅ Real | ✅ Real | `image` 0.25 hdr feature; Inf/NaN/neg sanitise (audit-14 LL) |
| `ph2d-imageio-exr` | W3.T2 | ✅ Real | ❌ Unsupported (defer) | `exr` 1.x closure-state walker |
| `ph2d-imageio-jxl` | W3.T1 | ✅ Real (LDR) | ❌ Unsupported (permanent) | `jxl-oxide` 0.10 decode-only; HDR/Animated/CMYK reject |
| `ph2d-imageio-avif` | W3.T4 | ❌ Stub | ❌ Stub | **DESHIPPED** in `f034e9a` — vide §5.17 |
| `ph2d-imageio-svg` | W3.T5 | ⚠️ Parse-only | ❌ Unsupported (defer) | `usvg` 0.43 parse → `VectorDoc::default()`; awaits ADR-0056 |

## §2 — Audit timeline (15 rounds)

Every round used 5–7 adversarial lenses rotated per
`feedback-audit-lens-diversity`. Findings remediated inline per
`feedback-perfection-no-deferrals`.

| Round | Commit | Findings | Wave |
|---|---|---|---|
| Audit-3 | `f71f16a` | hex-baked fixtures + golden blake3 pins | W3 pre-gates |
| Audit-5..6 | `35cc149`..`a5edbf1` | HR-9 vs single-platform amendment §2.6.1 | W3.T0 |
| Audit-7 | `108a623` | OOM caps (APNG/TIFF/native walker) | W3.T0.2 |
| Audit-8 | `2a41a0b`..`64f54d9` | ColorProfile::Custom doc honesty + GIF semantics + EOF helper adoption | W3.T0.3 |
| Audit-9 | `cde3e44` | **CRITICAL** ORA recursion stack-overflow (`MAX_LAYER_DEPTH=64`) + path traversal reject | W3.T0.4 |
| Audit-10 | `4d7dfdd` | **CRITICAL** ORA `opacity="NaN"` reject + ZIP central directory cap + take(N) | W3.T0.5 |
| Audit-11 | `7ce34da` | convergence ratification | W3.T0.6 |
| Audit-12 | `cc97cd4` + `084b914` | W3 fan-out 5 stubs landed | W3.T1..T5 |
| Audit-13 | `54a8a12` | EXR doc-honesty + APNG cargo `numer` ignore | W3.T1.5 |
| Audit-14 | `5f9582b` | **3 CRITICAL** HDR Inf panic + JXL ColorProfile/CMYK auto-srgb | W3 wave-2 |
| Audit-15 | `f034e9a` | **DESHIP AVIF** — RUSTSEC + upstream bugs | W3.T4 deship |

## §3 — Hard rules + defences (live)

The 11 W3.T0.* audits + W3 wave-2 audits installed defences across all
14 crates. **Do not regress these** without ADR amendment:

- **HR-1 platform-agnostic**: only 4 tests `#[cfg(target_os="macos", target_arch="aarch64")]` (golden blake3 drift pins). All other tests cross-platform.
- **HR-5 byte-exact**: 4 golden-hash tests (PNG/TIFF/ORA/APNG) pinned for Mac aarch64 local drift; cross-OS not pinned (see §2.6.1).
- **HR-6 blake3**: AssetDb content-addressed via blake3.
- **HR-9 cross-platform**: see §2.6.1 — single-platform pin admitted; multi-platform deferred until first divergence.
- **HR-13 OOM/DoS**: 9 caps in `limits.rs` (all hoisted from per-crate constants).
- **HR-14 save-versioning**: `.ph2d-native` `LayerStackV1.version: u32` + walker validates nested `version == 1`.
- **HR-15 i18n**: `Error::fluent_key()` surface; user-facing strings via Fluent keys.

Specific defences shipped:
- ORA recursion bomb (audit-9 T-#1) — `MAX_LAYER_DEPTH = 64`, `MAX_LAYER_COUNT = 4096`, path-traversal reject.
- ORA NaN/Inf opacity (audit-10 #1) — `.filter(is_finite).clamp(0,1)`.
- ORA ZIP bomb (audit-10 #2) — `MAX_ARCHIVE_ENTRIES = 8192`, `MAX_ARCHIVE_TEXT_BYTES = 16 MiB`.
- APNG/GIF frame bombs — `MAX_ANIMATION_FRAMES = 1024`.
- TIFF multi-page bomb — `MAX_DOCUMENT_PAGES = 256`.
- HDR encode Inf/NaN panic guard (audit-14 LL) — `.filter(is_finite).max(0.0)` per-channel.
- JXL HDR PQ/HLG + multi-frame + CMYK reject (audit-14 MM) — `request_color_encoding(srgb)` + `hdr_type().is_some()` + `num_loaded_keyframes() > 1`.
- PSD upstream panic — `catch_unwind` wrapper.
- EOF semantics centralised — `Error::from_decoder_message` (10 substrings) used in PNG/JPEG/WEBP/GIF/ORA/TIFF/APNG/PSD/native.

## §4 — Defers + open work (prioritised)

### High-value

1. **W3.T4 AVIF real decode** — `f034e9a` reverted `avif-decode = "1"` due to RUSTSEC-2022-0040 (owning_ref UAF) + upstream `unprem()` bug. **Three candidate paths** (vide §5.17):
   - **(A)** `image = { features = ["avif-native"] }` — verify dep tree avoids `owning_ref` before adding.
   - **(B)** Wait for `avif-decode 2.x` to migrate to `safer_owning_ref` + fix `unprem()` upstream.
   - **(C)** `libavif-sys` direct (C FFI; fastest but unsafe ABI).
   - **Verification protocol** for any candidate: `cargo audit` clean + `cargo tree | grep owning_ref` empty + grep `unsafe` count + bus-factor check + RUSTSEC search.

2. **W3.T5 SVG real vector body** — Currently parses via `usvg` 0.43 and returns `VectorDoc::default()` (empty stub). Real vector body needs amendment to the **FROZEN** contract type `VectorDoc` (currently `_reserved_for_w3: ()`). Outras sessões shipped `ph2d-vector-doc` per [ADR-0056](architecture/decisions/0056-vector-network-data-model.md). When ADR-0054 §2.1.1 amendment lands, SVG importer plugs `Ph2dVectorAsset` into `VectorDoc` body.

3. **W3.T2.1 EXR encode** — currently `Error::Unsupported`. `exr` 1.x `SpecificChannels::Image` construction needs typed callback wires. Wire-up when first real export client (Painter HDR save demo).

4. **W3.T1.6 JXL real-fixture decode test** — current tests are magic-recognition + truncated-error only. Real-decode test needs a `.jxl` fixture binary (cjxl CLI offline) checked in at `crates/ph2d-imageio-jxl/tests/fixtures/`.

### Lower-value (LOW or deferred with entry-points)

- TIFF/PSD `ColorProfile::Custom` populate (currently hard-codes `Srgb`) — W2.0.1 ICC pipeline.
- JPEG/WebP/GIF/BMP golden hashes — lossy encoders may re-optimise bytes without pixel regression; gold hash semantically inappropriate (permanent defer).
- `ImportOpts.color_profile_strictness::Strict` honor — currently 9/9 crates ignore; W2.0.1 ICC pipeline.
- `ExportOpts.tone_map` honor — currently silent; W3+ HDR tone-map.
- Tier-1 fixture expansion (APNG dispose_op, TIFF Gray arms, ORA malformed branches) — W3.T1.6 incremental.

## §5 — Multi-agent coordination

Active parallel sessions touching files **outside** imageio scope (do
not collide):

| Crate / area | Session |
|---|---|
| `crates/ph2d-vector*` (vector-doc, vector-traits, vector) | Vector Module team — ADR-0056/0057 |
| `crates/ph2d-painter-stroke`, `crates/ph2d-painter-brush` | Painter stroke/brush team |
| `crates/ph2d-asset` (W1.T4 KTX2/Tier/LogicalTextureMap; dev-dep HDR end-to-end test) | KTX2 + ph2d-asset session (ADR-0055-v4) |
| `crates/ph2d-asset-cooker` | KTX2 cook sub-command |
| `crates/ph2d-tool-bgremoval` | bgremoval session |
| `crates/ph2d-panel-*` (equalize-sizes/padding/upscale/hierarchy) | panel sessions |
| `shells/desktop/src/render_loop/mod.rs` | hero_intents Merge Sprites |
| `crates/ph2d-editor-core/interaction/dispatch/` | dispatch refactor |
| `docs/Painter_projeto/`, `docs/Vector Module/`, `docs/UI_Fonts/` | painter/vector docs |

**Conflict avoidance rules for imageio sessions:**

1. `git status --short` before EVERY `git add`.
2. `git add <explicit paths only>` — never `git add docs/` wholesale (per [`feedback-destructive-git-outside-pasta`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_destructive_git_outside_pasta.md)).
3. Stage → commit window must be **short** — long windows let parallel sessions absorb your staged files (see `feedback-parallel-agent-collision` lesson: `e54d41a` absorbed our X-HIGH-1 wire-up files into a KTX2 commit).
4. Never modify `shells/desktop/src/render_loop/mod.rs` from an imageio session — Merge Sprites lives there. If you need destructure changes, coordinate via the user.
5. Cargo.lock churn: new deps in `ph2d-imageio-*` crates re-resolve transitives in parallel sessions' next `cargo build`. Acceptable; if parallel session is mid-staging, defer your commit.

## §6 — Test counts (live snapshot 2026-05-28)

Per-crate (Mac aarch64; `#[cfg]`-gated golden tests included locally):

| Crate | Tests |
|---|---|
| `ph2d-imageio` (contract + arch-gates) | 21 (16 lib + 2 + 3 integration) |
| `ph2d-imageio-png` | 14 |
| `ph2d-imageio-jpeg` | 12 |
| `ph2d-imageio-webp` | 8 |
| `ph2d-imageio-gif` | 9 |
| `ph2d-imageio-ph2d-native` | 18 |
| `ph2d-imageio-tiff` | 18 |
| `ph2d-imageio-ora` | 16+ |
| `ph2d-imageio-apng` | 14 |
| `ph2d-imageio-psd` | 8 + 3 fixture |
| `ph2d-imageio-hdr-radiance` | 7 |
| `ph2d-imageio-exr` | 7 |
| `ph2d-imageio-jxl` | 8 |
| `ph2d-imageio-avif` | 9 (stub) |
| `ph2d-imageio-svg` | 11 |
| `ph2d-imageio-registry-init` | 8 (2 lib + 3 alphabetical + 3 staleness) |
| **Total** | **~190 verdes Mac aarch64** |

CI matrix in [`.github/workflows/spike.yml`](../.github/workflows/spike.yml)
lists all 16 imageio crates (14 format + contract + registry-init).

## §7 — Canonical paths

```
crates/
├── ph2d-imageio/                       # contract: trait surface + limits.rs + Error + ColorProfile
├── ph2d-imageio-png/
├── ph2d-imageio-jpeg/
├── ph2d-imageio-webp/
├── ph2d-imageio-gif/
├── ph2d-imageio-ph2d-native/
├── ph2d-imageio-tiff/
├── ph2d-imageio-ora/
├── ph2d-imageio-apng/
├── ph2d-imageio-psd/
├── ph2d-imageio-hdr-radiance/
├── ph2d-imageio-exr/
├── ph2d-imageio-jxl/
├── ph2d-imageio-avif/                  # STUB (deshipped audit-15)
├── ph2d-imageio-svg/
└── ph2d-imageio-registry-init/         # codegen output (do not hand-edit)

tools/ph2d-imageio-sync/                # codegen tool: scans crates/ + regenerates registry-init

docs/
├── architecture/decisions/0054-imageio-pipeline.md   # ADR
└── plans/2026-05-imageio-waves.md                    # wave map (W3 wave-2 latest)

crates/ph2d-asset/src/loader.rs::decode_via_imageio_registry  # asset bridge (X-HIGH-1 wire-up)
```

## §8 — How to add a new format (fan-out drop-crate)

Per [ADR §3.8](architecture/decisions/0054-imageio-pipeline.md#38) +
[Lens U U-9 audit-9](architecture/decisions/0054-imageio-pipeline.md#511).
Recipe proven by W3.T1..T5 fan-out + AVIF/EXR/JXL/HDR wave-2 wires:

1. **Verify the candidate dep tree first**:
   - `cargo search <crate>` — published, version, last update.
   - `cargo tree -p <crate>` (in a scratch dir) — count transitives; grep for `owning_ref`/known-RUSTSEC deps.
   - `grep -c "unsafe" $(find ~/.cargo/registry/src/*<crate>* -name "*.rs")` — unsafe budget.
   - Bus-factor: GitHub commit history + maintainer count.
   - License (workspace `deny.toml` allowlist).
2. **Create skeleton**:
   - `mkdir -p crates/ph2d-imageio-<slug>/src`
   - `crates/ph2d-imageio-<slug>/Cargo.toml` with `default-features = false` deps + `ph2d-imageio` path dep.
   - `crates/ph2d-imageio-<slug>/src/lib.rs` with `#![forbid(unsafe_code)]` + magic recognition + Importer/Exporter structs + register fns.
3. **Run codegen**: `cargo run -p ph2d-imageio-sync` → regenerates registry-init.
4. **Add to CI matrix**: `.github/workflows/spike.yml` `cargo nextest run -p ph2d-imageio-<slug>` block.
5. **Audit before declaring real**: 5-lens audit per `feedback-audit-lens-diversity` covering: closure correctness · dep tree (cargo audit + cargo deny) · spec compliance · ship.sh local · HR coverage. If 1+ CRITICAL persists in dep tree → **revert and document** (see W3.T4 deship in §5.17 for the canonical example).

## §9 — Known traps (lessons learned)

- **`use exr::prelude::*` / similar prelude imports shadow `Error`** — alias `use ph2d_imageio::Error as IoError` inside affected `fn`. Audit-14 wave-2.
- **`f32::INFINITY` panics inside `image-0.25.10::HdrEncoder::encode`** — sanitise non-finite + negative pre-encode. Audit-14 LL.
- **JXL hardcoded `ColorProfile::Srgb` mislabel** — call `request_color_encoding(EnumColourEncoding::srgb(RenderingIntent::Relative))` BEFORE `render_frame`. Audit-14 MM.
- **Single-platform golden blake3 hashes** — gate with `#[cfg(all(target_os="macos", target_arch="aarch64"))]`. Audit-5 §2.6.1.
- **EOF heuristic must be centralised** — use `Error::from_decoder_message`; never reinvent per-crate substring match. Audit-7 I-3.
- **`git add docs/` wholesale** — never. Stage per-file. Audit-7 H1 lesson.
- **`use <crate>::prelude::*` inside `mod tests`** is fine; outside (in `impl` body) shadow Error. Audit-14.
- **PSD upstream panic on malformed input** — wrap `psd::Psd::from_bytes` in `std::panic::catch_unwind`. Audit-7 G-F2.
- **AVIF deps trap** — `avif-decode = "1"` carries RUSTSEC + libaom-sys 26MB C + upstream `unprem()` bug. See §5.17. Verify alternatives before re-shipping.
- **Multi-agent staging collision** — `git add` → `git commit` window must be tight. Parallel session can absorb staged files into their commit. Audit-13 KTX2 absorbed our W3.T1.0 wire-up into `e54d41a`. Recover via `git revert` + `git restore --staged` carefully (never `git reset --hard` outside your pasta — see `feedback-destructive-git-outside-pasta`).
