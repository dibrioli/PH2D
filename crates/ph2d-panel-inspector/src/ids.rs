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
mod inspector_camera;
mod inspector_factory;
mod inspector_projectile;
/// ⭐ Os ids da secção STATE MACHINE (TOP-20 #15) — ver o cabeçalho.
mod inspector_statemachine;
mod inspector_topdown;
pub use inspector_camera::*;
pub use inspector_factory::*;
pub use inspector_projectile::*;
pub use inspector_statemachine::*;
pub use inspector_topdown::*;
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
mod menus;
pub use menus::*;
mod inspector_physics_body;
pub use inspector_physics_body::*;
