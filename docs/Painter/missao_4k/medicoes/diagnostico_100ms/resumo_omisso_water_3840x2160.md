
### quadro lento 100 ms, acaba -8742 ms depois do mousedown: 100 ms · 891 amostras · 3992 M ciclos
fios: 44 com amostras · principal ph2d-host-deskt/454135 3 % · ph2d-host-deskt/454291 6 % · ph2d-host-deskt/454292 6 % · ph2d-host-deskt/454289 6 % · ph2d-host-deskt/454286 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 53.4 % · ph2d_painter_brush 34.0 % · ph2d_blend_mode 4.0 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 3.7 % · [[unknown]] 1.3 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 1.2 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 1.1 % · crossbeam_epoch 0.3 % · [libc.so.6] 0.2 % · ph2d_a11y 0.2 % · core 0.2 % · [libvulkan_radeon.so] 0.2 %
| % | função folha |
|---|---|
| 16.7 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 15.0 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 13.6 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 9.9 | `ph2d_painter_brush::texture::sample` |
| 8.5 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 7.3 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 4.5 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 3.7 | `ph2d_blend_mode::apply` |
| 3.7 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 3.2 | `std::sys::backtrace::__rust_begin_short_backtrace::<ph2d_tool_painter::compositor::compose::encode::{closure#1}::{closure#0}::{closure#0}, (` |
| 1.9 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 1.7 | `<ph2d_tool_painter::tool::PainterTool>::pilha_aplicada` |
| 1.4 | `ph2d_tool_painter::compositor::compose::composite_into::<ph2d_tool_painter::tool::internal::ToolPixelSource>` |
| 1.3 | `[[unknown]]` |
| 1.2 | `<[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32>` |

### quadro lento 161 ms, acaba +177 ms depois do mousedown: 161 ms · 1268 amostras · 5514 M ciclos
fios: 33 com amostras · principal ph2d-host-deskt/454135 3 % · ph2d-host-deskt/454282 6 % · ph2d-host-deskt/454289 6 % · ph2d-host-deskt/454292 6 % · ph2d-host-deskt/454288 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 49.4 % · ph2d_painter_brush 25.3 % · rayon 8.9 % · ph2d_blend_mode 3.2 % · [[unknown]] 3.0 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 2.8 % · crossbeam_epoch 1.8 % · ph2d_render 1.0 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 0.9 % · [libc.so.6] 0.8 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 0.6 % · crossbeam_deque 0.5 %
| % | função folha |
|---|---|
| 12.9 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 9.9 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 8.5 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 8.4 | `ph2d_painter_brush::texture::sample` |
| 6.6 | `<&ph2d_tool_painter::compositor::vidro::composite_region_sobre_o_papel<ph2d_tool_painter::tool::internal::ToolPixelSource>::{closure#0} as c` |
| 5.9 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 5.8 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 5.4 | `<rayon::iter::collect::consumer::CollectResult<[u8; 7]> as rayon::iter::plumbing::Folder<[u8; 7]>>::consume_iter::<core::iter::adapters::cop` |
| 4.3 | `ph2d_tool_painter::compositor::compose::composite_into::<ph2d_tool_painter::tool::internal::ToolPixelSource>` |
| 3.2 | `<rayon::iter::repeat::RepeatNProducer<[f32; 3]> as rayon::iter::plumbing::Producer>::fold_with::<rayon::iter::collect::consumer::CollectResu` |
| 3.0 | `[[unknown]]` |
| 2.9 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 2.9 | `ph2d_blend_mode::apply` |
| 2.8 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 2.8 | `<rayon::iter::collect::consumer::CollectResult<u8> as rayon::iter::plumbing::Folder<u8>>::consume_iter::<core::iter::adapters::map::Map<core` |

### quadro lento 108 ms, acaba +527 ms depois do mouse-up: 108 ms · 1142 amostras · 5174 M ciclos
fios: 35 com amostras · principal ph2d-host-deskt/454135 0 % · ph2d-host-deskt/454288 6 % · ph2d-host-deskt/454283 6 % · ph2d-host-deskt/454281 6 % · ph2d-host-deskt/454289 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 57.7 % · ph2d_painter_brush 26.7 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 4.4 % · ph2d_blend_mode 3.9 % · rayon 3.1 % · [[unknown]] 1.4 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 1.1 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 0.9 % · crossbeam_epoch 0.3 % · [libc.so.6] 0.1 % · ph2d_a11y 0.1 % · ph2d_ecs 0.1 %
| % | função folha |
|---|---|
| 13.0 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 12.5 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 10.6 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 10.0 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 8.8 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 8.2 | `<&ph2d_tool_painter::compositor::vidro::composite_region_sobre_o_papel<ph2d_tool_painter::tool::internal::ToolPixelSource>::{closure#0} as c` |
| 7.2 | `ph2d_painter_brush::texture::sample` |
| 4.4 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 3.7 | `ph2d_blend_mode::apply` |
| 3.4 | `ph2d_tool_painter::compositor::compose::composite_into::<ph2d_tool_painter::tool::internal::ToolPixelSource>` |
| 3.1 | `<rayon::iter::repeat::RepeatNProducer<[f32; 3]> as rayon::iter::plumbing::Producer>::fold_with::<rayon::iter::collect::consumer::CollectResu` |
| 2.4 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 1.6 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 1.4 | `ph2d_painter_brush::texture::contrasta` |
| 1.4 | `ph2d_painter_brush::texture::patterns::sample_kind_px` |

### quadro lento 80 ms, acaba +92 ms depois do Ctrl+Z: 80 ms · 584 amostras · 3101 M ciclos
fios: 17 com amostras · principal ph2d-host-deskt/454135 4 % · ph2d-host-deskt/454283 6 % · ph2d-host-deskt/454284 6 % · ph2d-host-deskt/454285 6 % · ph2d-host-deskt/454281 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 44.6 % · ph2d_painter_brush 24.4 % · [libc.so.6] 14.1 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 11.3 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 3.7 % · core 0.3 % · ph2d_a11y 0.2 % · ph2d_board_model 0.2 % · hashbrown 0.2 % · bevy_ecs 0.2 % · std 0.2 % · crossbeam_epoch 0.2 %
| % | função folha |
|---|---|
| 34.5 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 14.1 | `[libc.so.6]` |
| 11.6 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 11.3 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 7.0 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 4.2 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 3.7 | `<[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32>` |
| 3.3 | `ph2d_painter_brush::texture::sample` |
| 2.2 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 1.5 | `ph2d_painter_brush::texture::contrasta` |
| 1.2 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::cover_at` |
| 1.0 | `ph2d_painter_brush::texture::patterns::sample_kind_px` |
| 0.7 | `<&<ph2d_tool_painter::tool::PainterTool>::impasto_gpu_planes_in::{closure#0} as core::ops::function::FnMut<((usize, (((&mut [f32], &mut [u8]` |
| 0.5 | `ph2d_painter_brush::texture::pela_area::largura_no_dab` |
| 0.3 | `<&mut core::array::drain::Drain<f32, <core::ops::try_trait::NeverShortCircuit<u8>>::wrap_mut_1<f32, <ph2d_tool_painter::tool::paint::relief_` |

### quadro lento 104 ms, acaba +537 ms depois do Ctrl+Z: 104 ms · 953 amostras · 4431 M ciclos
fios: 50 com amostras · principal ph2d-host-deskt/454135 2 % · ph2d-host-deskt/454296 6 % · ph2d-host-deskt/454282 6 % · ph2d-host-deskt/454284 6 % · ph2d-host-deskt/454292 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 54.1 % · ph2d_painter_brush 31.4 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 4.8 % · ph2d_blend_mode 4.4 % · [[unknown]] 1.7 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 1.4 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 1.2 % · [libc.so.6] 0.5 % · ph2d_board_model 0.2 % · ph2d_a11y 0.1 % · alloc 0.1 % · read_fonts 0.1 %
| % | função folha |
|---|---|
| 14.4 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 13.5 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 11.7 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 9.6 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 9.5 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 9.2 | `ph2d_painter_brush::texture::sample` |
| 4.9 | `std::sys::backtrace::__rust_begin_short_backtrace::<ph2d_tool_painter::compositor::compose::encode::{closure#1}::{closure#0}::{closure#0}, (` |
| 4.8 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 3.9 | `ph2d_blend_mode::apply` |
| 2.5 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 2.4 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 1.9 | `ph2d_painter_brush::texture::patterns::sample_kind_px` |
| 1.7 | `[[unknown]]` |
| 1.5 | `<ph2d_tool_painter::tool::PainterTool>::pilha_aplicada` |
| 1.4 | `<[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32>` |

parado0: CPU do processo 56 % de um núcleo

### parado0 (sem entrada): 5009 ms · 2551 amostras · 13935 M ciclos · 32.6 M ciclos/quadro em 428 quadros
fios: 3 com amostras · principal ph2d-host-deskt/454135 100 % · ph2d-host-deskt/454135 100 % · WSI swapchain q/454530 0 % · cpal_alsa_out/454141 0 %
por crate (a 1.ª nomeada na folha): ph2d_a11y 20.1 % · ph2d_ecs 8.3 % · [libc.so.6] 8.2 % · alloc 6.4 % · ph2d_editor_core 5.5 % · ph2d_board_model 5.0 % · ph2d_tool_painter 3.9 % · core 2.7 % · [libvulkan_radeon.so] 2.6 % · vello_encoding 2.4 % · ph2d_app_vec 1.8 % · ph2d_painter_brush 1.7 %
| % | função folha |
|---|---|
| 8.2 | `[libc.so.6]` |
| 6.2 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, ph2d_editor_core::interaction::state::kinds::InteractiveState> as core::iter:` |
| 5.3 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, f32> as core::iter::traits::iterator::Iterator>::next` |
| 3.6 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_board_model::board::BoardId, u32, alloc::coll` |
| 2.7 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_a11y::node::NodeId, ph2d_editor_core::interac` |
| 2.6 | `[libvulkan_radeon.so]` |
| 2.4 | `ph2d_tool_painter::tool::paint::watercolor_noise::value_noise_pair` |
| 1.4 | `<alloc::collections::btree::map::Values<u64, ph2d_ecs::scene::registry::ComponentTypeEntry> as core::iter::traits::iterator::Iterator>::next` |
| 1.2 | `malloc` |
| 1.1 | `ph2d_painter_brush::texture::patterns::preview::render_texture_preview` |
| 1.0 | `<&mut <ph2d_editor_core::interaction::state::store_fields::WidgetStore>::hover_targets::{closure#0} as core::ops::function::FnMut<((&ph2d_a1` |
| 1.0 | `ph2d_tool_registry::node_id::hash_node_id_runtime` |
| 0.9 | `<alloc::collections::btree::map::BTreeMap<ph2d_a11y::node::NodeId, f32>>::insert` |
| 0.9 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, (u64, ph2d_ecs::canal::FieldKey), (f32, f32), allo` |
| 0.8 | `<alloc::raw_vec::RawVecInner>::try_allocate_in` |

parado1: CPU do processo 56 % de um núcleo

### parado1 (sem entrada): 5008 ms · 2575 amostras · 13631 M ciclos · 31.8 M ciclos/quadro em 428 quadros
fios: 4 com amostras · principal ph2d-host-deskt/454135 100 % · ph2d-host-deskt/454135 100 % · WSI swapchain q/454530 0 % · cpal_alsa_out/454141 0 % · data-loop.0/454140 0 %
por crate (a 1.ª nomeada na folha): ph2d_a11y 19.9 % · [libc.so.6] 7.8 % · ph2d_ecs 7.5 % · alloc 6.2 % · ph2d_board_model 6.0 % · ph2d_editor_core 5.0 % · ph2d_tool_painter 4.4 % · core 3.6 % · [libvulkan_radeon.so] 3.0 % · ph2d_painter_brush 2.1 % · vello_encoding 1.7 % · ph2d_app_vec 1.6 %
| % | função folha |
|---|---|
| 7.8 | `[libc.so.6]` |
| 7.7 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, ph2d_editor_core::interaction::state::kinds::InteractiveState> as core::iter:` |
| 4.7 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_board_model::board::BoardId, u32, alloc::coll` |
| 3.5 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, f32> as core::iter::traits::iterator::Iterator>::next` |
| 3.0 | `[libvulkan_radeon.so]` |
| 2.8 | `ph2d_tool_painter::tool::paint::watercolor_noise::value_noise_pair` |
| 2.5 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_a11y::node::NodeId, ph2d_editor_core::interac` |
| 1.4 | `<alloc::collections::btree::map::Values<u64, ph2d_ecs::scene::registry::ComponentTypeEntry> as core::iter::traits::iterator::Iterator>::next` |
| 1.3 | `ph2d_painter_brush::texture::patterns::preview::render_texture_preview` |
| 1.1 | `malloc` |
| 1.1 | `ph2d_tool_registry::node_id::hash_node_id_runtime` |
| 1.0 | `<u8 as core::fmt::Display>::fmt` |
| 1.0 | `<alloc::collections::btree::map::BTreeMap<ph2d_a11y::node::NodeId, f32>>::insert` |
| 0.8 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, (u64, ph2d_ecs::canal::FieldKey), (f32, f32), allo` |
| 0.8 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, alloc::boxed::Box<str>, &str, alloc::collections::` |
