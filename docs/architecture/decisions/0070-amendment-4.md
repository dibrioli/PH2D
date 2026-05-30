# ADR-0070-amendment-4 — `RenderInstance.rotation: f32` → `basis: [f32; 4]` (render skew as a true parallelogram)

**Status:** Accepted (W2.T2.x skew render step, 2026-05-30)
**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md) §1.7 ABI.
**Implements:** [ADR-0025-amendment-1 — Transform 2D skew](0025-amendment-1.md) §2.6 ("skew aplica via vertex shader extra step (W2.T2.x decide)").
**Spec sections clarified:** `docs/Sprite_projeto/01_anatomia_canonica.md` §1.7 (RenderInstance ABI), `docs/Sprite_projeto/10_schema_versionamento.md` §10.5/§10.11 (size_of).
**Tags:** render, abi, skew, transform, hr-4

---

## 1. Context — skew was authored but never rendered

W2.T2.2/T2.3 added `Transform.skew_x`/`skew_y` (ADR-0025-amendment-1) and the
Inspector Skew X/Y sliders. The data model + propagation are correct: a skewed
`Transform` produces a sheared (non-orthogonal) `GlobalTransform` 2×2 basis.

But the **renderer never applied the shear.** The extract phase
([`shells/desktop/src/render_loop/sim_extract.rs`](../../../shells/desktop/src/render_loop/sim_extract.rs))
decomposed `GlobalTransform.affine()` into:

```
scale_x = |col0|,  scale_y = |col1|,  rotation = atan2(col0.y, col0.x)
```

and shipped only the single `rotation` scalar (`RenderInstance.rotation`,
`@location(6)`) plus a scale-folded `size`. The shader rebuilt the quad as a
**rotated axis-aligned rectangle**. A shear lives in `col1` being non-orthogonal
to `col0` — and that information is destroyed by the decomposition. Visually the
sprite skewed as "rotation + a stretched scale", never a true parallelogram —
i.e. *misturava scale + rotação*, not skew. The decomposition is fundamentally
lossy for any non-orthogonal (sheared) basis.

ADR-0025-amendment-1 §2.6 explicitly deferred this: "extract para RenderInstance
usa rotation apenas. Skew aplica via vertex shader extra step (W2.T2.x decide)."
This amendment is that decision.

## 2. Decision — send the full 2×2 world basis to the shader

Replace the decomposed `rotation: f32` with the exact 2×2 world linear basis:

```rust
// RenderInstance (was: pub rotation: f32, // @location(6) Float32)
pub basis: [f32; 4],   // @location(6) Float32x4 — [col0.x, col0.y, col1.x, col1.y]
```

- **Extract** stops decomposing: `basis = [affine[0], affine[1], affine[2], affine[3]]`.
  `size` and `anchor` revert to LOCAL (`Sprite::size` / `Sprite::anchor`) — the
  basis carries the world scale, so no double-scaling.
- **Shader** (`sprite.wgsl`) maps the local quad through the basis:
  `world = world_pos + mat2x2(col0, col1) · (anchor + quad·size)`. A non-orthogonal
  basis maps the axis-aligned local quad to the correct parallelogram — true skew.
- **Picking** ([`picking.rs`](../../../crates/ph2d-render/src/picking.rs)) inverts the
  basis to bring the world cursor into local space (point tests) and uses the exact
  parallelogram AABB (rect / gizmo box). Strictly more correct than the prior path,
  which ignored rotation for the AABB cases entirely.

### 2.1 Equivalence for the no-skew path (the safety argument)

For any unsheared transform the basis is exactly `R·S` (`col0 = (cos·sx, sin·sx)`,
`col1 = (−sin·sy, cos·sy)`), and `basis · (anchor + quad·size)` equals the old
`rotate(scale·(anchor + quad·size))`. So **every existing scene renders identically**
(the old path even round-tripped through `atan2`→`cos`/`sin`, which the basis path
avoids — strictly more accurate, sub-pixel at most). Picking for the unsheared case
reduces to the prior inverse-rotation. No visual regression.

### 2.2 ABI cost — `size_of` 144 → 156 bytes

`rotation: f32` (4 B) → `basis: [f32; 4]` (16 B): **+12 B**, stride 144 → 156.
Field count stays **12** (a `f32` field became one `[f32; 4]` field), so the
ADR-0070 §1.7 12-field cap still holds; only the byte-size pin moves. Vertex
attribute `@location(6)` changes `Float32` → `Float32x4`. Gates updated:
`architecture_sprite_inspector_surface` (156), `render_instance_pod_size_v4`
(156), `vertex_attr_offsets_match_struct` (basis offset), plus a new
`sprite_wgsl_valid` naga gate that parses + validates the shader and pins
`@location(6)` as `vec4<f32>`.

The +12 B is well within the W1.T1.7b bench headroom (the CPU build+memcpy at 10k
sprites is ~0.x% of frame; 156 B is not a bandwidth bottleneck). The ADR-0070 §2.5
dual-buffer mitigation remains un-fired.

## 3. Alternatives rejected

- **Godot's single-skew scalar** (`col1 = (−sin(r+skew), cos(r+skew))·sy`): one
  extra angle, ABI-preservable by packing `premultiplied` into `flip_uv`. Rejected
  because PH2D's `Transform` carries **two** independent skew axes (skew_x, skew_y,
  ADR-0025-amendment-1 §2.1, the LÖVE `kx`/`ky` model) — a single scalar can't
  represent both. The full basis represents any affine, including the two-axis case.
- **Keep `rotation` + add `skew_x`/`skew_y` floats:** still requires reconstructing
  the basis in the shader from angles, more ALU and more state than passing the
  basis directly, for no benefit.
- **Apply skew on the CPU by pre-shearing quad corners:** the quad geometry is a
  shared unit strip; corners are built in the vertex shader, not per-instance on CPU.

## 4. Consequences

- **Positive:** skew renders correctly (true parallelogram); rotation + non-uniform
  scale render without the lossy `atan2` round-trip; picking honours rotation/scale/
  skew exactly; a naga gate now catches shader/layout drift at `cargo test` time.
- **Negative:** +12 B per instance; one frozen ABI number moved (this amendment is
  the deliberate cost). Any crate constructing `RenderInstance` by hand sets `basis`
  (`IDENTITY_BASIS` for the no-transform case) instead of `rotation`.
- **Neutral:** the gizmo selection-box build in `snapshots.rs` still decomposes for
  its own bbox; under heavy skew the box is a conservative AABB (cosmetic, not the
  sprite render). A follow-up can switch it to the parallelogram AABB helper.
