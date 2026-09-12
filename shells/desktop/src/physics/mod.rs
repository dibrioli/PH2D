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

// ─────────────────────────────────────────────────────────────────────────
// **Os gates que ATRAVESSAM a fronteira.** O sujeito destes é uma cena da
// `ph2d-app-physics` e o que eles exercitam é um gesto DESTA shell — e um
// `#[cfg(test)]` não é visível do outro lado de uma crate (HOWTO §2). Por isso
// eles são declarados aqui, e não pela cena: *o teste mora com o que EXERCITA.*
// ⚠️ Os outros gates das mesmas cenas ficaram com o sujeito, na crate.
// ─────────────────────────────────────────────────────────────────────────
#[cfg(test)]
#[path = "physics_smoke_part_tests.rs"]
mod physics_smoke_part_tests;
#[cfg(test)]
#[path = "physics_smoke_pulley_comp_tests.rs"]
mod physics_smoke_pulley_comp_tests;
#[cfg(test)]
#[path = "physics_smoke_pulley_diff_tests.rs"]
mod physics_smoke_pulley_diff_tests;
#[cfg(test)]
#[path = "physics_smoke_pulley_tackle_tests.rs"]
mod physics_smoke_pulley_tackle_tests;
#[cfg(test)]
#[path = "physics_smoke_rig_tests.rs"]
mod physics_smoke_rig_tests;

// Os gates da §14: o painel e' da crate, a PORTA que eles atravessam e' desta shell.
#[cfg(test)]
#[path = "inspector_player_tests.rs"]
pub mod inspector_player_tests;
#[cfg(test)]
#[path = "overlay_joint_world_gizmo_tests.rs"]
mod overlay_joint_world_gizmo_tests;

// ─────────────────────────────────────────────────────────────────────────
// **As declarações que o CORTE levou consigo.** Estes treze eram `mod X;` planos
// no `render_loop/mod.rs`; o ficheiro mudou-se para cá e a LINHA que o declarava
// ficou lá, e foi apagada com os vizinhos que de facto saíram.
// ⛔ Um ficheiro de teste órfão não dá erro nenhum: ele simplesmente **deixa de
// ser compilado**, e a suíte fica verde com menos gates do que tinha. É a perda
// que o `nextest-list-diff` existe para apanhar — e o audit de órfãos, para
// apanhar ANTES.
// ─────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod bridge_tests;
#[cfg(test)]
mod joint_break_tests;
#[cfg(test)]
mod joint_kind_tests;
#[cfg(test)]
mod joint_motor_tests;
#[cfg(test)]
mod joint_pair_tests;
#[cfg(test)]
mod joint_paste_tests;
#[cfg(test)]
mod joint_tests;
#[cfg(test)]
mod joint_wheel_tests;
mod joint_world_tests;
#[cfg(test)]
mod physics_gesture_surface_tests;
#[cfg(test)]
mod physics_gesture_tests;
#[cfg(test)]
mod physics_gesture_zone_tests;
pub(crate) mod physics_tests;
