
### quadro lento 96 ms, acaba -9369 ms depois do mousedown: 96 ms · 810 amostras · 3972 M ciclos
fios: 42 com amostras · principal ph2d-host-deskt/366809 3 % · ph2d-host-deskt/366977 6 % · ph2d-host-deskt/366992 6 % · ph2d-host-deskt/366978 6 % · ph2d-host-deskt/366985 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 51.3 % · ph2d_painter_brush 36.1 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 3.8 % · ph2d_blend_mode 2.5 % · [[unknown]] 2.2 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 1.5 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 1.4 % · [libc.so.6] 0.2 % · ph2d_ecs 0.1 % · ph2d_a11y 0.1 % · wgpu_core 0.1 % · crossbeam_epoch 0.1 %
| % | função folha |
|---|---|
| 16.3 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 14.8 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 13.5 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 9.5 | `ph2d_painter_brush::texture::sample` |
| 8.7 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 6.6 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 5.0 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 3.8 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 2.8 | `std::sys::backtrace::__rust_begin_short_backtrace::<ph2d_tool_painter::compositor::compose::encode::{closure#1}::{closure#0}::{closure#0}, (` |
| 2.4 | `ph2d_blend_mode::apply` |
| 2.2 | `[[unknown]]` |
| 2.0 | `ph2d_painter_brush::texture::contrasta` |
| 2.0 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 1.8 | `ph2d_painter_brush::texture::patterns::sample_kind_px` |
| 1.6 | `<ph2d_tool_painter::tool::PainterTool>::pilha_aplicada` |

### quadro lento 97 ms, acaba +446 ms depois do mousedown: 97 ms · 838 amostras · 3937 M ciclos
fios: 44 com amostras · principal ph2d-host-deskt/366809 2 % · ph2d-host-deskt/366980 6 % · ph2d-host-deskt/366985 6 % · ph2d-host-deskt/366992 6 % · ph2d-host-deskt/366982 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 49.8 % · ph2d_painter_brush 35.8 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 5.1 % · ph2d_blend_mode 2.9 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 2.6 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 1.2 % · [[unknown]] 1.2 % · [libc.so.6] 0.7 % · ph2d_a11y 0.1 % · ph2d_physics 0.1 % · ph2d_timeline 0.1 % · wgpu_core 0.1 %
| % | função folha |
|---|---|
| 17.6 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 13.6 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 11.2 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 8.4 | `ph2d_painter_brush::texture::sample` |
| 6.5 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 6.3 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 5.9 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 5.1 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 3.6 | `std::sys::backtrace::__rust_begin_short_backtrace::<ph2d_tool_painter::compositor::compose::encode::{closure#1}::{closure#0}::{closure#0}, (` |
| 3.2 | `ph2d_painter_brush::texture::patterns::sample_kind_px` |
| 2.6 | `ph2d_blend_mode::apply` |
| 2.6 | `<[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32>` |
| 2.2 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 1.5 | `<ph2d_tool_painter::tool::PainterTool>::pilha_aplicada` |
| 1.4 | `ph2d_tool_painter::compositor::compose::composite_into::<ph2d_tool_painter::tool::internal::ToolPixelSource>` |

### quadro lento 99 ms, acaba +473 ms depois do mouse-up: 99 ms · 812 amostras · 4000 M ciclos
fios: 43 com amostras · principal ph2d-host-deskt/366809 2 % · ph2d-host-deskt/366978 6 % · ph2d-host-deskt/366990 6 % · ph2d-host-deskt/366977 6 % · ph2d-host-deskt/366989 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 53.1 % · ph2d_painter_brush 35.2 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 3.4 % · ph2d_blend_mode 2.9 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 1.9 % · [[unknown]] 1.3 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 1.0 % · core 0.3 % · [libc.so.6] 0.2 % · ph2d_board_model 0.1 % · __rustc 0.1 % · parley 0.1 %
| % | função folha |
|---|---|
| 15.8 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 14.5 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 11.5 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 9.6 | `ph2d_painter_brush::texture::sample` |
| 8.0 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 7.6 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 5.7 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 4.1 | `std::sys::backtrace::__rust_begin_short_backtrace::<ph2d_tool_painter::compositor::compose::encode::{closure#1}::{closure#0}::{closure#0}, (` |
| 3.4 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 2.7 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 2.7 | `ph2d_blend_mode::apply` |
| 1.9 | `<f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length` |
| 1.9 | `ph2d_painter_brush::texture::patterns::sample_kind_px` |
| 1.6 | `<ph2d_tool_painter::tool::PainterTool>::pilha_aplicada` |
| 1.5 | `ph2d_tool_painter::compositor::compose::composite_into::<ph2d_tool_painter::tool::internal::ToolPixelSource>` |

### quadro lento 99 ms, acaba +451 ms depois do Ctrl+Z: 99 ms · 842 amostras · 4052 M ciclos
fios: 43 com amostras · principal ph2d-host-deskt/366809 2 % · ph2d-host-deskt/366992 6 % · ph2d-host-deskt/366982 6 % · ph2d-host-deskt/366977 6 % · ph2d-host-deskt/366981 6 %
por crate (a 1.ª nomeada na folha): ph2d_tool_painter 54.4 % · ph2d_painter_brush 33.5 % · <&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::{closure#0}> as core::ops::function::FnMut<(usize,)>>::call_mut 5.1 % · ph2d_blend_mode 3.0 % · [[unknown]] 1.5 % · <f32 as core::slice::cmp::SlicePartialEq<f32>>::equal_same_length 0.8 % · <[u8; 7]>::map::<<f32 as core::convert::From<u8>>::from, f32> 0.8 % · ph2d_editor_core 0.1 % · [libc.so.6] 0.1 % · ph2d_a11y 0.1 % · wgpu_core 0.1 % · ph2d_text 0.1 %
| % | função folha |
|---|---|
| 15.5 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::material_at` |
| 15.3 | `ph2d_painter_brush::texture::sample::{closure#0}` |
| 12.2 | `<&<ph2d_tool_painter::tool::PainterTool>::apply_impasto_light::{closure#0} as core::ops::function::FnMut<((usize, &mut [u8]),)>>::call_mut` |
| 10.1 | `<ph2d_tool_painter::tool::paint::impasto_shade::Rig>::shade` |
| 8.8 | `ph2d_painter_brush::texture::sample` |
| 7.4 | `<ph2d_tool_painter::tool::paint::relief_fields::ReliefFields>::height_at` |
| 5.1 | `<&mut core::array::drain::Drain<u8, <core::ops::try_trait::NeverShortCircuit<f32>>::wrap_mut_1<u8, <f32 as core::convert::From<u8>>::from>::` |
| 4.2 | `std::sys::backtrace::__rust_begin_short_backtrace::<ph2d_tool_painter::compositor::compose::encode::{closure#1}::{closure#0}::{closure#0}, (` |
| 3.9 | `ph2d_painter_brush::texture::patterns::paper_tile::sample` |
| 3.0 | `ph2d_painter_brush::texture::patterns::sample_kind_t` |
| 2.8 | `ph2d_blend_mode::apply` |
| 1.7 | `ph2d_painter_brush::texture::patterns::sample_kind_px` |
| 1.7 | `ph2d_tool_painter::compositor::compose::composite_into::<ph2d_tool_painter::tool::internal::ToolPixelSource>` |
| 1.6 | `<ph2d_tool_painter::tool::PainterTool>::pilha_aplicada` |
| 1.5 | `[[unknown]]` |

parado0: CPU do processo 50 % de um núcleo

### parado0 (sem entrada): 5009 ms · 2410 amostras · 12116 M ciclos · 25.5 M ciclos/quadro em 475 quadros
fios: 3 com amostras · principal ph2d-host-deskt/366809 100 % · ph2d-host-deskt/366809 100 % · WSI swapchain q/367245 0 % · cpal_alsa_out/366815 0 %
por crate (a 1.ª nomeada na folha): ph2d_a11y 24.0 % · ph2d_ecs 10.9 % · [libc.so.6] 8.9 % · ph2d_board_model 7.9 % · alloc 5.0 % · ph2d_editor_core 4.3 % · ph2d_painter_brush 3.6 % · core 2.3 % · [libvulkan_radeon.so] 2.2 % · ph2d_app_vec 1.8 % · read_fonts 1.8 % · std 1.6 %
| % | função folha |
|---|---|
| 8.9 | `[libc.so.6]` |
| 8.6 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, ph2d_editor_core::interaction::state::kinds::InteractiveState> as core::iter:` |
| 6.6 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_board_model::board::BoardId, u32, alloc::coll` |
| 4.4 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, f32> as core::iter::traits::iterator::Iterator>::next` |
| 3.0 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_a11y::node::NodeId, ph2d_editor_core::interac` |
| 2.2 | `[libvulkan_radeon.so]` |
| 2.0 | `<alloc::collections::btree::map::Values<u64, ph2d_ecs::scene::registry::ComponentTypeEntry> as core::iter::traits::iterator::Iterator>::next` |
| 1.9 | `ph2d_painter_brush::texture::patterns::preview::render_texture_preview` |
| 1.8 | `<alloc::collections::btree::map::BTreeMap<ph2d_a11y::node::NodeId, f32>>::insert` |
| 1.3 | `<alloc::collections::btree::map::entry::VacantEntry<ph2d_a11y::node::NodeId, ph2d_editor_core::widget::property_box::key_mark::KeyMark>>::in` |
| 1.2 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, (u64, ph2d_ecs::canal::FieldKey), (f32, f32), allo` |
| 1.1 | `malloc` |
| 0.8 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_text::layout_cache::LayoutCacheKey, parley::l` |
| 0.8 | `<alloc::collections::btree::map::Range<(u64, ph2d_ecs::canal::FieldKey), f32> as core::iter::traits::iterator::Iterator>::next` |
| 0.8 | `<read_fonts::FontRef>::table_data` |

parado1: CPU do processo 51 % de um núcleo

### parado1 (sem entrada): 5009 ms · 2399 amostras · 12328 M ciclos · 26.0 M ciclos/quadro em 475 quadros
fios: 4 com amostras · principal ph2d-host-deskt/366809 100 % · ph2d-host-deskt/366809 100 % · WSI swapchain q/367245 0 % · gilrs/366811 0 % · cpal_alsa_out/366815 0 %
por crate (a 1.ª nomeada na folha): ph2d_a11y 22.4 % · ph2d_ecs 9.3 % · [libc.so.6] 8.8 % · ph2d_board_model 8.5 % · alloc 5.2 % · ph2d_editor_core 4.4 % · [libvulkan_radeon.so] 3.4 % · ph2d_painter_brush 3.1 % · core 2.5 % · wgpu_core 1.8 % · vello_encoding 1.7 % · ph2d_timeline 1.6 %
| % | função folha |
|---|---|
| 8.8 | `[libc.so.6]` |
| 7.8 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, ph2d_editor_core::interaction::state::kinds::InteractiveState> as core::iter:` |
| 7.3 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_board_model::board::BoardId, u32, alloc::coll` |
| 4.3 | `<alloc::collections::btree::map::Iter<ph2d_a11y::node::NodeId, f32> as core::iter::traits::iterator::Iterator>::next` |
| 3.4 | `[libvulkan_radeon.so]` |
| 2.6 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_a11y::node::NodeId, ph2d_editor_core::interac` |
| 2.2 | `<alloc::collections::btree::map::Values<u64, ph2d_ecs::scene::registry::ComponentTypeEntry> as core::iter::traits::iterator::Iterator>::next` |
| 1.8 | `<alloc::collections::btree::map::BTreeMap<ph2d_a11y::node::NodeId, f32>>::insert` |
| 1.3 | `ph2d_painter_brush::texture::patterns::preview::render_texture_preview` |
| 1.2 | `malloc` |
| 1.0 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_text::layout_cache::LayoutCacheKey, parley::l` |
| 0.8 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, (u64, ph2d_ecs::canal::FieldKey), (f32, f32), allo` |
| 0.8 | `<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut, ph2d_a11y::node::NodeId, ph2d_editor_core::widget:` |
| 0.7 | `<parley::layout::layout::Layout<()> as core::clone::Clone>::clone` |
| 0.7 | `<alloc::raw_vec::RawVecInner>::try_allocate_in` |
