//! **O que da família `physics` ficou na shell** (W2/L2 Fase B).
//!
//! ⚠️⚠️ **Nenhum destes ficou por causa da `App`** — e essa é a resposta que esta linha deve às
//! outras quatro. As 5 portas do [`ph2d_app_host::AppHost`] cobriram todos os gestos; o que prende
//! estes ficheiros são **três módulos da shell que são eles próprios FOLHAS**, partilhados por
//! várias famílias:
//!
//! | módulo | usos daqui | quem mais o consome |
//! |---|---:|---:|
//! | `render_loop::inspector_ordering` (595 LOC) | 7 | 11 ficheiros do Inspector |
//! | `preview_drive` (530 LOC) | 11 | 38 ficheiros (undo, timeline, morph, skeleton…) |
//! | `name_unique` (181 LOC) | 4 | 14 ficheiros (flip, sculpt3d, vec, sheet…) |
//! | `app_state::GroupDragSnapshot` · `render_loop::point_gizmo` | 3 | a shell |
//!
//! Os três primeiros dependem **só de crates de módulo** (`ph2d_core`, `ph2d_ecs`, `ph2d_editor`,
//! `ph2d_render`) e **não tocam a `App`** — são folhas a viver na shell por inércia, exactamente o
//! caso do HOWTO §1.2 (*«duas famílias que partilham código partilham uma FOLHA»*). ⛔ Movê-las é
//! decisão do **integrador**, não desta linha: elas são consumidas por ficheiros de quatro famílias
//! que ainda não abriram, e a regra 5 da Fase B proíbe editar a árvore de outra linha.
//!
//! ⇒ **Quando essas três forem folhas, estes 19 ficheiros seguem sem mais nada.**

pub(crate) mod bake;
pub(crate) mod bridge;
pub(crate) mod joint;
pub(crate) mod joint_anchor_drag;
pub(crate) mod joint_anchor_drag_stop;
pub(crate) mod joint_anchor_drag_wheel;
pub(crate) mod joint_create;
pub(crate) mod joint_draw;
pub(crate) mod joint_rig;
pub(crate) mod joint_rig_drag;
pub(crate) mod joint_wheel;
pub(crate) mod joint_world;
pub(crate) mod measure_player_tape;
pub(crate) mod panel_bridge;
pub(crate) mod physics;
pub(crate) mod physics_apply;
pub(crate) mod physics_area;
pub(crate) mod physics_markers;
pub(crate) mod physics_seed;
pub(crate) mod physics_smoke;
pub(crate) mod physics_smoke_base;
pub(crate) mod physics_smoke_joint_anim;
pub(crate) mod physics_smoke_out;
pub(crate) mod physics_smoke_player;
pub(crate) mod physics_smoke_rigs;
pub(crate) mod physics_state;
pub(crate) mod physics_surface;
pub(crate) mod player_input;
