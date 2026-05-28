# ADR-0070-amendment-3 — `RenderInstance.flip_uv` is a general flags bitfield (packs `flip_x`, `flip_y`, `tint_fill`)

**Status:** Accepted (W1.T1.10/T1.11 enabler, 2026-05-28)
**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md) §1.7 ABI + [ADR-0071 — Tint channels multiplicative](0071-tint-channels-multiplicative.md) (tint_fill GPU path).
**Slot rationale:** `-1` stays pre-reserved for the dual-buffer perf mitigation (ADR-0070 §2.5, fired only if the W1.T1.7b bench shows a GPU bandwidth bottleneck — a **W2** call per decision 1(a); the CPU bench does not trigger it). `-2` is the empirical-back-compat amendment. This is the next free slot.
**Spec sections clarified:** `docs/Sprite_projeto/01_anatomia_canonica.md` §1.7 (flip_uv comment), `docs/Sprite_projeto/10_schema_versionamento.md` §10.5, `docs/Sprite_projeto/04_color_tint_canais.md` §4.2.

---

## 1. Context — the contradiction

Three normative sources disagree on how `tint_fill` reaches the GPU:

1. **§1.7 / §10.5 (ABI)** freeze `RenderInstance` at **12 fields / 144 bytes** and document `flip_uv: u32` as a bitfield carrying only `bit0=flip_x, bit1=flip_y`. The cap is gated by `architecture_sprite_inspector_surface` (W1.T1.12): `RenderInstance fields == 12`.
2. **§4.2 (canonical tint math)** writes the fragment shader as reading `instance.tint_fill != 0.0` — i.e. `tint_fill` must be visible to the shader.
3. **§15.2 T1.11** ("shader lê per_corner_tint interpolado + **tint_fill** + opacity") confirms the shader must consume `tint_fill`.

`tint_fill` is a `Sprite` field (§1.6, a `bool`), but it is NOT one of the 3 new `RenderInstance` fields the ABI added (`per_corner_tint`, `opacity`, `flip_uv`). With the 12-field cap **frozen**, a 13th field (`tint_fill: f32/u32`) is forbidden without an ABI cap bump. So `tint_fill` has no declared transport to the shader. This blocked T1.11.

## 2. Decision

`flip_uv` is **a general per-instance flags bitfield**, not a flip-only field. Bit layout (FROZEN):

| Bit | Mask | Flag | Source `Sprite` field |
|----|------|------|------|
| 0 | `0x1` | flip_x | `Sprite::flip_x` |
| 1 | `0x2` | flip_y | `Sprite::flip_y` |
| 2 | `0x4` | tint_fill | `Sprite::tint_fill` |
| 3..31 | — | reserved (must be 0) | — |

- **Zero ABI cost:** `RenderInstance` stays at 12 fields / 144 bytes. The `u32` already had 29 unused bits. `architecture_sprite_inspector_surface` (== 12) is unaffected.
- The field KEEPS the name `flip_uv` (it is already shipped in the T1.7a struct + the §1.7/§10.5 spec text + the `vertex_attr_offsets_match_struct` gate; renaming would churn the frozen ABI surface for no semantic gain). Its DOC is widened to "packed flags" — see §4.
- The extract phase (W1.T1.10) encodes:
  `flip_uv = (flip_x as u32) | ((flip_y as u32) << 1) | ((tint_fill as u32) << 2)`.
- The fragment shader (W1.T1.11) decodes:
  `flip_x = (flip_uv & 1u) != 0u`, `flip_y = (flip_uv & 2u) != 0u`, `tint_fill = (flip_uv & 4u) != 0u`.

## 3. Why pack instead of bumping the cap to 13 fields

- The 12-field / 144-byte cap is FROZEN with a gate and a 5-lens W0 ratification behind it. Adding a 13th GPU field for a single bool wastes 4 bytes/instance (× the instance stride budget HR-4 watches) and breaks the frozen surface — exactly the "21º campo = re-think" stance §10.11 takes for `Sprite`.
- `flip_uv` being a `u32` for two booleans was already over-provisioned; the spec author's choice of `u32` (not `u8`/`u16`) only makes sense as a flags word. Packing is the reading that makes the existing ABI self-consistent.
- All three flags are per-instance booleans consumed in the same fragment-shader stage; one flags word is the idiomatic GPU encoding (cf. material flag bitfields in Godot/Bevy).

## 4. Consequences

- **§1.7 / §10.5 flip_uv comment** should read: *"u32 packed flags: bit0=flip_x, bit1=flip_y, bit2=tint_fill (anatomia §4.2); bits 3+ reserved=0."* (Spec is the source of truth — Coord to apply; the live code doc in `crates/ph2d-render/src/sprite.rs` carries this wording from T1.10.)
- **§4.2 shader pseudocode** `instance.tint_fill` resolves to `(instance.flip_uv & 4u) != 0u`. The multiplicative math (§4.2 / §4.5) is otherwise unchanged: `tint_fill` replaces sampled RGB with the combined tint RGB, alpha preserved.
- **Forward-compat:** bits 3..31 are reserved-zero. A future per-instance bool flag (e.g. a sampler-mode toggle) claims the next free bit via an ADR-0070-amendment-N, still at zero ABI cost, until the 32 bits are exhausted.
- **`architecture_sprite_inspector_surface`** (W1.T1.12) is unaffected: still asserts `RenderInstance fields == 12`, `size_of == 144`. A dedicated unit test (`flip_uv_flag_bits_roundtrip`) pins the bit positions so a future edit can't silently re-map them.

## 5. Provenance

- Surfaced during W1.T1.7a→T1.8 by the Sprite implementer: the frozen 12-field ABI vs §4.2/§15.2's `tint_fill` shader read had no reconciliation in any spec section.
- Decision ratified by Enio (2026-05-28): pack into `flip_uv` as a flags bitfield, zero ABI cost, over a 13th field.
