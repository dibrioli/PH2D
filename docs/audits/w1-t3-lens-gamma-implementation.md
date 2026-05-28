# W1.T3 — Lens Γ (Implementation) Audit

**Date:** 2026-05-27
**Scope:** First Rust implementation of KTX2 Phase 2 — `tools/asset-cooker/src/texture/`.
**Auditor lens:** correctness of cooking decisions + honesty of tests + edge-case discipline.
**Rule:** Anti-Goodhart — if cook decisions are sound, tests prove what they claim, and edge cases are either guarded or cited, score 9–10 APPROVE. No invented findings to pad the count.

---

## Verdict

**Score: 9.2 / 10 — APPROVE with 2 cited follow-ups + 1 fix recommended pre-T4.**

All four CRITICAL classes from the briefing were probed and either disproved (BC7 test is not vacuous; `unreachable!()` invariant holds on ctt 0.4.0 source inspection) or downgraded to MEDIUM/cited. Two MEDIUM findings are worth surfacing because they will sting once `NormalMap` or `SingleChannel` real assets enter the pipeline. One HIGH (type-level falsity of `Option<TargetFormat>`) is a 5-line cleanup that costs nothing now and pays off when `target_for` consumers grow.

The implementation is small (≈230 LOC across 3 files + 1 CLI subcommand), idiomatic, well-commented with W1.T2 audit lineage citations, and honest about what it doesn't yet test (cross-machine determinism is explicitly deferred to W1.T10 in code comments, not just docs — `cook.rs:206-207`). That on-the-line honesty is exactly what the briefing asked for and what we asked for in the meta-audit handoff.

---

## CRITICAL (none)

All four CRITICAL candidates from the briefing were probed against ctt 0.4.0 source and downgraded:

1. **`unreachable!()` at `cook.rs:151`** — VERIFIED safe. `passthrough.rs:36-49` and the encoder paths in `convert.rs` all route `Container::Ktx2(_)` to `PipelineOutput::Encoded(_)`. `Container::Raw` is the only `PipelineOutput::Raw(_)` source, and `cook()` hardcodes `Container::ktx2()` (`cook.rs:141`). Invariant is structurally enforced by ctt 0.4.0; comment on `cook.rs:146-148` documents the reasoning. No action.
2. **`bc7_paths_never_dispatch_via_auto` vacuity** (`target_matrix.rs:191-208`) — NOT vacuous. Matrix has BC7 at `(Desktop, SpriteColor)` and `(Desktop, CriticalUi)`; both pairs are present in the test's nested loops. Test asserts the `Bc7encoder` kind on exactly the 2 BC7 entries the matrix produces. Real D3 guard.
3. **`stride = width * 4` overflow** (`cook.rs:113`) — bounded by `image` crate's PNG decoder, which rejects dimensions `> i32::MAX / 4` long before reaching `to_rgba8()`. PNG itself caps width at `2^31 - 1`. Theoretical overflow is unreachable via the actual API. No action.
4. **`decode()` panic** — `image 0.25` with only `png` feature returns `ImageError` on all malformed input; no panic path on dimension `u32::MAX` (rejected as `LimitError::DimensionError` before allocation). For offline cooker, even a panic would only abort one cook job — acceptable per HR-3-not-applicable lineage. No action.

---

## HIGH

### H1. `target_for` returns `Option<TargetFormat>` but the body is total

**Site:** `target_matrix.rs:61-104`
**Severity:** HIGH (type-level falsity, not a runtime bug)

The function signature promises `None` for "NOT_SUPPORTED in this wave (e.g., HDR formats W4+)" (`target_matrix.rs:59-60`) but every match arm returns `Some(...)`. The `(Constrained, _)` wildcard catches all 4 `AssetClass` × `Constrained` combinations, so there is no path to `None`. The `cook.rs:131-136` caller pattern-matches `ok_or_else` against a never-fired branch — dead code reachable only by future refactor.

**Why HIGH:** the dead `NoTargetFormat` error variant (`cook.rs:35`) is the kind of latent surface that grows lies during fan-out. A future implementer adding `AssetClass::HdrSpriteColor` will see the existing matrix has `(Constrained, _)` wildcard and either (a) ship without realizing the wildcard catches HDR-as-RGBA8 (silent quality loss) or (b) refactor the wildcard, breaking the only existing `None` defense.

**Recommended fix:** either (a) change return to `TargetFormat` directly and delete `NoTargetFormat` from `TextureCookError`, or (b) keep `Option` and add an explicit pinned `None` case (e.g., add `AssetClass::Hdr` placeholder that returns `None` until W4). Option (a) is simpler given the scope freeze. Cite this in `mod.rs` if you keep (b).

### H2. `CookOptions::default()` produces wrong color-space for `NormalMap` / `SingleChannel`

**Site:** `cook.rs:87-96`
**Severity:** HIGH (semantic mis-tagging in container header)

`Default` for `CookOptions` returns `color_space: ColorSpace::Srgb`. The CLI in `main.rs:166-170` uses `..Default::default()` to fill in `alpha` and `color_space` regardless of `--asset_class`. Concretely: `cooker texture cook foo.png out.ktx2 --asset-class normal-map` produces a KTX2 file whose surface metadata claims `ColorSpace::Srgb`. The renderer will then sample normal-map data with sRGB→linear decoding applied — wrong gamma, wrong normals.

**Verified against ctt source:** `vk_format.rs:533-580` `denormalize` returns the format as-is for BC4/BC5/BC6 (no `*_SRGB_BLOCK` variants exist), so `color_space: Srgb` does NOT produce an invalid vkFormat for those families — but it DOES tag the input surface as sRGB, which ctt threads into `format.denormalize(first.color_space)` at `output/ktx2.rs:19`. For BC7/ASTC/ETC2 the denormalization actually flips to SRGB_BLOCK variant — so a NormalMap routed through ASTC (`(Mobile, NormalMap)` at `target_matrix.rs:82`) will get `ASTC_6x6_SRGB_BLOCK` in the KTX2 container, a real shader-visible bug.

**Recommended fix:** either (a) derive `color_space` from `asset_class` inside `cook()` so caller can't get it wrong (NormalMap + SingleChannel → `Linear`, color → `Srgb`), or (b) validate the (asset_class, color_space) combination at `cook()` entry and return a `TextureCookError::InvalidColorSpaceForClass`. Option (a) removes a footgun entirely. Option (b) preserves caller intent (rare cases like color data stored in NormalMap slots) at the cost of one extra error path.

### H3. Determinism test is single-machine only, but test name does not advertise this

**Site:** `cook.rs:204-215`
**Severity:** HIGH (test honesty / PR review trap)

The test name `cook_is_deterministic_for_same_input_same_cpu` is honest in name. The inline comment is honest (`cook.rs:206-207`). But the test reads like a HR-6 determinism guarantee to a casual reviewer doing line-by-line PR review, and the assertion message ("must produce byte-identical KTX2") repeats the absolute claim without the qualifier. A future contributor who deletes the `NB:` comment to "clean up dead docs" will leave a test whose name implies more than it checks.

**Recommended fix (5 min):** rename to `cook_is_byte_stable_on_single_runner` AND change the assertion message to `"local cook → cook round-trip stable; cross-machine guarantee deferred to W1.T10 canonical runner"`. Cost: 3 lines. Benefit: removes the only avenue by which this test could degrade into a false guarantee under code-rot.

---

## MEDIUM

### M1. `cook_64x64_mobile_critical_ui_uses_astc_4x4` (`cook.rs:217-232`) does not verify ASTC 4×4

The test name claims the cook used ASTC 4×4. The body only checks (a) byte length > 100 and (b) KTX2 magic header. If the matrix entry at `(Mobile, CriticalUi)` ever switched silently to ASTC 6×6 or BC7, this test would still pass. Two cheap fixes: (a) parse `vkFormat` field from the KTX2 header (24 bytes in, little-endian u32 = ktx2 vkFormat code; ASTC_4x4_UNORM_BLOCK = 157, ASTC_4x4_SRGB_BLOCK = 158) and assert; or (b) rename to `cook_mobile_critical_ui_emits_valid_ktx2` and let `target_matrix.rs::mobile_critical_ui_uses_astc_4x4` carry the format assertion. Option (b) is honest about division of labor — matrix tests check matrix; cook tests check pipeline glue. Recommend (b).

### M2. Test fixture only exercises perfect synthetic gradient — no quality regression detection

`fixture_png_64x64()` (`cook.rs:163-177`) generates a smooth gradient. Cook output is asserted only on (length, magic-bytes). The cook could be entirely producing garbage block payloads (all-zero, all-FF, wrong block ordering) and these tests would all still pass. Real PSNR / decode-round-trip / golden-bytes assertions are deferred to W1.T6 (multi-tier emit) + W1.T11.5 (LFS golden fixtures). The deferral is cited at `cook.rs:14-15` and again in `mod.rs:24-25` — that's honest. Recommendation: add a `// TODO(W1.T11.5):` marker inside the test bodies, not just at module-doc level, so future contributors editing these tests see the gap inline.

### M3. `Display for ImageError` may swallow PNG-decoder context

`cook.rs:41` writes `"texture cook: PNG decode failed: {e}"`. `image::ImageError::Display` is decent but truncates the underlying `png::DecodingError` cause chain in some cases. For an offline cooker CLI where the user just wants to know *which* of 10k PNGs corrupted, consider `{e:#}` (alternate Display) or `{e:?}` (Debug, full chain). Current behavior won't lose data but will make support harder once batch cooking lands.

---

## LOW

### L1. `pub use cook::cook` shadow (`mod.rs:30`)

`texture::cook` exists as both module (`pub mod cook`) and function (`pub use cook::cook`). Rust resolves correctly via namespace, but a future contributor reading `texture::cook(...)` cannot immediately tell whether the call is in scope via the re-export or via fully-qualified path. Cosmetic. Acceptable as long as `mod.rs:30` is preserved verbatim. No action unless you touch the file for other reasons.

### L2. `TierArg`/`AssetClassArg` enum duplication in `main.rs:69-107`

Standard clap idiom — keeps lib enum independent of clap derive. Acceptable. Worth a one-line comment in `main.rs:69` pointing at clap's `value_parser` alternative for whoever inherits this and wonders why we duplicated.

### L3. `run_texture_cook` reads entire PNG into memory (`main.rs:165`)

For 16K×16K PNG (~1 GB raw decoded) the cooker tool will allocate the PNG bytes + the decoded RGBA8 buffer simultaneously. Acceptable for offline cooker on dev machines. Worth a `// NB:` comment when batch cooking enters scope so nobody panics when this OOMs.

---

## Strengths (worth calling out)

- **Lineage citations:** every module-level doc-comment cites the W1.T2 audit finding it defends against (D1, D3) and the future task that will close the gap (W1.T10, W1.T11.5). This is the discipline the meta-audit reinforced and it shows up in code, not just docs.
- **`ColorSpace` doc note** at `target_matrix.rs:65-69` — proactively documents the SRGB-vs-UNORM encoder restriction discovered during the W1.T2 source audit. Future implementer adding new entries to the matrix won't have to rediscover this.
- **`Encoder::Auto` for Constrained tier** (`target_matrix.rs:100`) — correct: no encoder dispatch needed because passthrough handles `RGBA8 → KTX2` via `output/ktx2.rs:encode_ktx2_image`. Verified against ctt source.
- **`#[derive(..., PartialEq, Eq, Hash)]` on `Tier` and `AssetClass`** — anticipates W1.T6 multi-tier emit needing to key into a `HashMap<(Tier, AssetClass), …>` or similar. Cheap and right.

---

## Recommended follow-ups (pre-T4)

1. **Fix H2** (color-space derivation from asset_class) — actual silent bug.
2. **Fix H1** (collapse `Option<TargetFormat>` to `TargetFormat`) — 5 lines, removes dead error path.
3. **Fix H3** (rename determinism test + soften assertion message) — 3 lines, removes only avenue for the test to lie under code-rot.

M1 and L1–L3 are notes; defer until you touch those files for other reasons.
