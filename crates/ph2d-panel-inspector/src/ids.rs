//! Os `NodeId` de widget que esta crate LÊ — declarados AQUI, e não na `ph2d-editor-core`.
//!
//! ⭐ **Desceram da fundação em 2026-09-12** (auditoria de arquitectura A5b): em 30 dias, 221 dos 437
//! commits à `ph2d-editor-core/src` tocaram `ids/`, e cada um recompilava as 60 crates que dependem
//! dela. O dono de um id é a crate MAIS BAIXA que todo leitor dele vê (`scripts/censo-ids.py`), e
//! um ficheiro de ids — um ASSUNTO — desce inteiro.
//!
//! ⚠️ **UMA definição e ZERO re-exportações**: quem lê de fora nomeia `ph2d_panel_inspector::ids::X`. As colisões
//! de slug vigia-as o censo DERIVADO (`ph2d-editor-core/tests/it/node_id_collisions.rs`), que lê os
//! literais da workspace inteira — o id não precisa de morar ao lado dele.

mod inspector;
pub use inspector::*;
mod inspector_action;
pub use inspector_action::*;
mod inspector_anchor;
pub use inspector_anchor::*;
mod inspector_anim;
pub use inspector_anim::*;
mod inspector_audio;
pub use inspector_audio::*;
/// ⭐ Os ids da secção COUNTER WATCH — a vigia do contador.
mod inspector_action_trigger;
mod inspector_camera;
pub use inspector_action_trigger::*;
mod inspector_counter_watch;
pub use inspector_counter_watch::*;
/// ⭐ Os ids das DUAS secções do ABANÃO (suplente #25) — ver o cabeçalho.
mod inspector_shake;
pub use inspector_shake::*;
mod inspector_factory;
/// ⭐ Os ids da secção SCRIPT (TOP-20 #16) — ver o cabeçalho.
mod inspector_hud;
/// Os ids da secção RAY SENSOR (suplente #21).
mod inspector_parallax;
mod inspector_particles;
mod inspector_projectile;
mod inspector_ray;
mod inspector_script;
/// ⭐ Os ids da secção SEQUENCE (TOP-20 #19) — ver o cabeçalho.
mod inspector_sequence;
/// ⭐ Os ids da secção STATE MACHINE (TOP-20 #15) — ver o cabeçalho.
mod inspector_statemachine;
mod inspector_topdown;
mod inspector_vida;
mod inspector_weapon;
pub use inspector_camera::*;
pub use inspector_factory::*;
pub use inspector_hud::*;
pub use inspector_parallax::*;
pub use inspector_particles::*;
pub use inspector_projectile::*;
pub use inspector_ray::*;
pub use inspector_script::*;
pub use inspector_sequence::*;
pub use inspector_statemachine::*;
pub use inspector_topdown::*;
pub use inspector_vida::*;
pub use inspector_weapon::*;
mod inspector_instance;
pub use inspector_instance::*;
mod inspector_joint;
pub use inspector_joint::*;
mod inspector_player;
pub use inspector_player::*;
mod inspector_sampling;
pub use inspector_sampling::*;
mod inspector_slice;
pub use inspector_slice::*;
mod inspector_tags;
pub use inspector_tags::*;
mod inspector_timer;
pub use inspector_timer::*;
mod inspector_tween;
pub use inspector_tween::*;
/// ⭐⭐⭐ **Os ids do SEGUIDOR DE CAMINHO** (suplente #23) — ver o cabeçalho.
mod inspector_path_follow;
pub use inspector_path_follow::*;
mod menus;
pub use menus::*;
mod inspector_physics_body;
pub use inspector_physics_body::*;
