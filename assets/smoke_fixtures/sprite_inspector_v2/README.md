# Smoke fixtures — Sprite Inspector v2

Canonical scene fixtures for the Enio smoke loop (`Sprite_projeto §15.8.2`). Each wave drops one `.scene` here; goldens live at [`docs/Sprite_projeto/smoke_goldens/`](../../../docs/Sprite_projeto/smoke_goldens/).

W0 creates the directory contract (gate `smoke_fixture_renderable` is a directory-presence stub today; populated as features land). The scenes themselves depend on schema/features that arrive wave-by-wave — listing the empty paths here keeps the contract visible.

| Wave | Fixture                              | Depends on                                                                 |
|------|--------------------------------------|----------------------------------------------------------------------------|
| W2   | `smoke_w2_color_tint.scene`          | per_corner_tint + self_tint + tint_fill + opacity (W1 schema + W2 widgets) |
| W3   | `smoke_w3_sorting.scene`             | SortingLayer + ZIndexOverride + YSort + SortingGroup + ShowBehindParent     |
| W4   | `smoke_w4_material_animation.scene`  | Material + UseParentMaterial + InstanceShaderParams + SpriteAnimator        |
| W5   | `smoke_w5_named_anchors.scene`       | NamedAnchorList (socket / slice / 9slice) + per-frame override               |

Smoke protocol (spec §15.8.2 lines 299-309):
```
./play.command --load smoke_fixtures/sprite_inspector_v2/<wave>.scene
```
Each checklist item references a pixel-identifiable golden in `smoke_goldens/`.
