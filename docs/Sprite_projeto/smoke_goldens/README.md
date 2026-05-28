# Smoke goldens — Sprite Inspector v2

Pixel-identifiable PNG goldens for the smoke fixtures at [`assets/smoke_fixtures/sprite_inspector_v2/`](../../../assets/smoke_fixtures/sprite_inspector_v2/). Spec §15.8.2 protocol.

W0 creates the directory only — the PNG goldens land wave-by-wave alongside their `.scene` fixtures (W2 / W3 / W4 / W5). The `smoke_fixture_renderable` gate ([`crates/ph2d-render/tests/smoke_fixture_renderable.rs`](../../../crates/ph2d-render/tests/smoke_fixture_renderable.rs)) asserts directory presence today; per-fixture load + golden-diff lands when the matching wave wires the feature.

| Wave | Goldens                                                                                                          |
|------|------------------------------------------------------------------------------------------------------------------|
| W2   | `w2_tint_cascade.png`, `w2_self_tint_local.png`, `w2_per_corner_gradient.png`, `w2_opacity_independent.png`, `w2_tint_fill_silhouette.png` |
| W3   | `w3_y_sort_topdown.png`, `w3_z_index_relative.png`, `w3_show_behind_parent.png`, `w3_sorting_group_block.png`, `w3_clip_children_3_modes.png` |
| W4   | `w4_use_parent_material_batching.png`, `w4_instance_params_variation.png`, `w4_anim_pingpong_60fps.png`           |
| W5   | `w5_socket_attach.png`, `w5_slice_bounds_drag.png`, `w5_9slice_region_drag.png`, `w5_per_frame_anchor_override.png` |
