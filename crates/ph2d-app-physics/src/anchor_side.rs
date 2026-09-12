//! **Que LADO de uma junta uma alça autora** — a lei que vivia no pintor do gizmo.
//!
//! ⚠️⚠️ **Ela mudou-se de `render_loop::point_gizmo` na W2/L2 Fase B, e a razão é a régua do
//! HOWTO §1.2:** *«isto pergunta alguma coisa ao ESTADO da família dona do ficheiro?»* — e não
//! pergunta. É um `match` de seis linhas entre dois tipos de **crate de módulo**
//! ([`ph2d_editor_core::gizmo::PointHandleKind`] e [`ph2d_physics_ecs::JointSide`]), sem uma linha de
//! gizmo dentro: ela estava ali porque o pintor foi quem a escreveu primeiro.
//!
//! ⭐ **E ela era o ÚNICO fio que prendia o `joint_anchor_drag` à shell** — que por sua vez prendia
//! o `PhysicsState`, que prendia o roteador. *Um `use` de seis linhas segurava a cadeia inteira.*

use ph2d_editor_core::gizmo::PointHandleKind;
use ph2d_physics_ecs::JointSide;

/// O [`JointSide`] que uma alça autora, ou `None` para as pegas de parâmetro
/// (elas escrevem o COMPONENTE da junta, não uma âncora).
#[must_use]
pub fn anchor_side(kind: PointHandleKind) -> Option<JointSide> {
    match kind {
        PointHandleKind::AnchorA => Some(JointSide::A),
        PointHandleKind::AnchorB => Some(JointSide::B),
        _ => None,
    }
}
