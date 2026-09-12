//! A metade `overlay` da família `physics`, vinda de `shells/desktop/src/render_loop/`
//! (W2/L2 Fase B). ⚠️ O prefixo saiu dos nomes: dentro desta crate tudo é a física.

pub mod annotations;
pub mod contacts;
pub mod gesture;
pub mod joint_ghost;
pub mod joint_glyphs;
pub mod joint_readout;
pub mod joints;
pub mod outline;
/// ⭐ **Quais âncoras ganham alça de canvas** — vindo de
/// `shells/desktop/src/render_loop/point_gizmo.rs` (W2/L2 Fase C).
///
/// ⚠️ **Ele tinha nome genérico e era 100% física:** os `use` dele são
/// `ph2d_physics_ecs::{JointSide, PhysicsBridge}` e — decisivo — o
/// [`joint_glyphs`] desta mesma pasta. As seis funções são junta, corda,
/// roldana e âncora. Ficou no `render_loop/` por inércia, não por desenho.
pub mod point_gizmo;
pub mod probes;
pub mod pulley;
pub mod shapes;
