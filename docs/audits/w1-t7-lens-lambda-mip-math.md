# W1.T7 — Lens Λ (lambda) — Mip Math Correctness + Edge Cases

**Auditor:** Lens Λ adversarial (mip math + edge cases focus)
**Commit:** `7ff552c`
**Files audited:**
- `tools/asset-cooker/src/texture/mip_gen.rs` (~261 LOC + 13 tests)
- `tools/asset-cooker/src/texture/mod.rs` (re-export)
- `tools/asset-cooker/src/texture/cook.rs` (consumer — *not yet wired*)
- `~/.cargo/registry/src/.../ctt-0.4.0/src/surface.rs` (multi-mip Image input)
- `~/.cargo/registry/src/.../image-0.25.10/src/imageops/sample.rs` (resize semantics)

**Score: 8.5/10 — APPROVE WITH ONE HIGH FIX**

Math is impeccable. All test asserts verified against an independent Python re-implementation: 11 cases (256², 1², 100×80, 2×1, 3×2, 16², 0, 1024×1, 7×11, 1023²) — every produced count matches Rust line-by-line. Tests cover the key invariants (count, dims-halve, cap, filter dispatch, intra-machine determinism, NPOT, 1×1 source, mapping table). One semantic edge case (`max_levels = Some(0)`) contradicts the docstring and merits a HIGH-priority fix; the rest is polish.

---

## CRITICAL (math / semantic violation)

**None.** The two non-trivial claims were re-verified by exhaustive simulation:

- `mip_levels_for_dimensions(100, 80) == 7` — **CORRECT**
  Trace: 100×80 → 50×40 → 25×20 → 12×10 → 6×5 → 3×2 → 1×1. Loop condition `w > 1 || h > 1` keeps shrinking until **both** = 1. Last iter: `w=3,h=2` → `(3/2).max(1)=1`, `(2/2).max(1)=1`; +1 → 7. (`mip_gen.rs:67-71`)
- `mip_levels_for_dimensions(2, 1) == 2` — **CORRECT** (2×1 → 1×1 in one shrink; +1 for level 0). (`mip_gen.rs:163`)
- `mip_levels_for_dimensions(3, 2) == 2` — implicit case; verified independently (3×2 → 1×1 single step).
- Early-stop in `generate_mip_chain` (`mip_gen.rs:106-108`): triggers iff `nw == pw && nh == ph`, which only happens at `pw=1, ph=1` (since `(x/2).max(1) == x` ⟺ `x ≤ 1`). Correct and equivalent to the `mip_levels_for_dimensions` termination.

---

## HIGH (untested edge case + doc-vs-behavior mismatch)

### H1. `max_levels = Some(0)` returns 1 level, not 0
**Cite:** `mip_gen.rs:78` (docstring) vs `mip_gen.rs:97-99` (impl).
**Severity:** HIGH (semantic surprise; silent off-by-one for caller passing `0` to mean "no mips").

Docstring says `Some(N)` = "level 0 + N-1 downsampled" — by that contract `Some(0)` would mean **zero levels total**. But `chain.push(source.clone())` at line 98 runs **unconditionally** before the `while chain.len() < capped` guard, so `Some(0)` yields a 1-element chain `[source]` (the loop never executes since `1 >= 0` is false… actually `1 < 0` is false so loop skips). Caller asking for `Some(0)` silently gets `Some(1)` behavior.

**Fix options (pick one):**
1. Treat `Some(0)` as "no chain": guard `if capped == 0 { return Vec::new(); }` before the unconditional push (cheap, matches doc literally).
2. Document that `Some(0)` is normalized to `Some(1)` (level 0 is always emitted).
3. Change signature to `max_levels: Option<NonZeroUsize>` (compile-time guarantee).

Add test either way (positive or negative assertion).

### H2. ASTC 6×6 block format awareness deferred to caller — no signal
**Cite:** `mip_gen.rs:55-58` (doc) — no mention; `cook.rs:1-360` doesn't call `mip_gen` yet.
**Severity:** HIGH (latent; will bite W2 mobile cooking).

For 256² + ASTC 6×6 (Tier::Mobile per ADR-0053), mip levels 5-8 (dims 8, 4, 2, 1) are **smaller than one ASTC 6×6 block**. `mip_gen` correctly emits all levels — but cook() must either (a) pad lower levels to 6×6, (b) skip them and write `levelCount = first_level_below_block_size`, or (c) fail loud. No API or doc mentions this. Mitigation: when wiring `cook` to `mip_gen` (T7.1 future), add a `min_dimension_for_format(Format)` helper and a `cook_with_mips` policy parameter (`Skip / Pad / Fail`). Capture as W2 follow-up; not blocking T7.

### H3. No `u32::MAX` overflow guard
**Cite:** `mip_gen.rs:60-73`, `mip_gen.rs:84-92`.
**Severity:** HIGH-LOW (offline cooker, but silent OOM ≠ OK).

`mip_levels_for_dimensions(u32::MAX, u32::MAX)` terminates (32 iters, log2(u32::MAX)), no overflow. **Safe.** But `generate_mip_chain` on `RgbaImage::new(u32::MAX, u32::MAX)` — `image::imageops::resize` would allocate `u32::MAX² × 4` bytes → OOM. Since cooker is offline + fed by fixtures, low practical risk, but consider an upfront `if w * h > MAX_PIXELS { return Err(...) }`. Note: would require changing return type to `Result<Vec<RgbaImage>, Error>`. Defer to T7.1 unless cook() spec already caps source dims (it does not, per `cook.rs:1-360`).

---

## MEDIUM (coverage gap)

### M1. Point filter never tested for byte-distinct output
**Cite:** `mip_gen.rs:201-217` (`generate_chain_lanczos3_distinct_from_box`).
**Severity:** MEDIUM.

Test confirms Box ≠ Lanczos. But Point uses Nearest sampling (totally different math — no interpolation, sample-only) — should be visibly distinct from both. Add a third assert `box_level1.as_raw() != point_level1.as_raw()` (3 lines). Cheap coverage gain; protects against silent filter collapse if `to_image_filter` mapping regresses.

### M2. No 16×16 fixture coverage
**Cite:** `mip_gen.rs:168-188` (256² coverage), `mip_gen.rs:236-245` (100×80 NPOT).
**Severity:** LOW-MEDIUM.

Only 256² is exercised end-to-end (`brush_atlas_256_r8`). A 16² case (5-level chain) would catch any future regression where the implementation special-cases >32 or <32. The `fixtures::critical_ui_16` fixture is mentioned in the audit prompt as a natural candidate. Defer or add now (3 lines).

### M3. Cross-machine determinism left to W1.T10 — currently untested
**Cite:** `mip_gen.rs:9-10` (docstring claim), `mip_gen.rs:219-233` (intra-machine test).
**Severity:** MEDIUM (documented as deferred, but worth flagging for HR-6 traceability).

The "no `f32::sin/cos`" claim depends on `image::imageops`' Lanczos using a polynomial sinc — true today (verified in `image-0.25.10/src/imageops/sample.rs:996+` Filter kernels are arithmetic), but no test pins this. Wave-T10 canonical runner will catch drift cross-arch; until then, the determinism claim is **plausible but unproven**. Doc honestly says so; no fix needed.

---

## LOW (code quality)

### L1. Redundant `ImageBuffer::from_raw` round-trip
**Cite:** `mip_gen.rs:110-115`.
**Severity:** LOW.

`image::imageops::resize(prev, nw, nh, image_filter)` already returns `ImageBuffer<Rgba<u8>, Vec<u8>>` = `RgbaImage` (verified: `image-0.25.10/src/imageops/sample.rs:964-989` + `images/buffer.rs:1798` `pub type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>`). The `into_raw()` + `from_raw()` ceremony is functionally a no-op (the `Vec<u8>` is moved both ways, no realloc — but the `.expect()` is dead-code and the explicit `RgbaImage` type annotation could just be:

```rust
let resized: RgbaImage = image::imageops::resize(prev, nw, nh, image_filter);
chain.push(resized);
```

Cleaner, removes a `panic!` branch, easier to read. Pure simplification.

### L2. `MipFilter` derives `Hash` without obvious use site
**Cite:** `mip_gen.rs:27`.
**Severity:** LOW.

`Hash` is defensible (3-variant enum, future `HashMap<MipFilter, _>` is plausible), but YAGNI-flag. Keep; trivial.

### L3. `Box` filter is technically bilinear (`Triangle`), not true box
**Cite:** `mip_gen.rs:29-31` (doc), `mip_gen.rs:46` (impl).
**Severity:** LOW (doc clarifies; defensible).

Doc honestly states "= Triangle/bilinear, equivalent à Box para 2:1 downsample exato". For **exact 2:1** downsample (always the case in mip generation), Triangle kernel reduces to a 2×2 box average → bytewise identical to a true box filter. Mathematically defensible. Documented. No fix.

### L4. Doc uses unicode `×` and `→`
**Cite:** `mip_gen.rs:13-58` (sparse), `mip_gen.rs:139,157,163,237` (tests).
**Severity:** LOW-COSMETIC.

`cargo doc` handles UTF-8 fine on all supported platforms (rustdoc is UTF-8 native since 1.0). Consistent with existing PH2D doc style. Keep.

### L5. `.expect("chain has level 0")` on internal invariant
**Cite:** `mip_gen.rs:101`.
**Severity:** LOW.

Defensible — line 98 unconditionally pushes `source`, so `chain.last()` is provably `Some`. The `expect` documents the invariant. If H1 is fixed via option 1 (early-return on `capped == 0`), this stays valid.

---

## ctt 0.4.0 multi-mip Image input — confirmed compatible

`surface.rs:71-75`: `Image { surfaces: Vec<Vec<Surface>> }` where `surfaces[i][j]` = slice `i`, mip level `j`. For 2D texture: `surfaces.len() == 1` (one slice), inner `Vec<Surface>` = the mip chain. Per `surface.rs:88-100` `validate()`, all layers must have the same mip count, and the head surface metadata propagates. So `mip_gen`'s output `Vec<RgbaImage>` maps 1:1 to `ctt::Image { surfaces: vec![chain_as_surfaces] }`. **No API impedance mismatch.** Wiring is mechanical (T7.1).

---

## Recommendation

**APPROVE T7 with one HIGH fix (H1: `Some(0)` semantics) + the cosmetic cleanup (L1).** All other findings are deferrable to T7.1/W2.

Math: 10/10. Code quality: 8/10 (L1, H1 nick it). Coverage: 8/10 (M1+M2 leave small gaps). Documentation: 9/10 (honest, accurate, only L4-style trim possible).

**Composite: 8.5/10.**

### Quick fixes (≤15 LOC total)

1. **H1 fix** — at `mip_gen.rs:97`, before `chain.push(source.clone())`:
   ```rust
   if capped == 0 { return Vec::new(); }
   ```
   Plus a `generate_chain_max_levels_zero_returns_empty` test.

2. **L1 fix** — replace `mip_gen.rs:110-115` with:
   ```rust
   let resized: RgbaImage = image::imageops::resize(prev, nw, nh, image_filter);
   chain.push(resized);
   ```

3. **M1 add** — in `generate_chain_lanczos3_distinct_from_box`, also assert `box_level1.as_raw() != point_level1.as_raw()` (3 lines).

4. **M2 add** — `generate_chain_16x16_has_5_levels` test (4 lines, mirrors `_256_box_has_9_levels`).

These four changes close every actionable finding short of H2/H3 (which are correctly W2/cook-wiring scope).
