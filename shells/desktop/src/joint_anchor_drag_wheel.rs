//! **A metade da RODLANA do gesto de alça** (W-Pulley W1/W6) — módulo FILHO de
//! [`super`] pelo cap de 600 LOC, e o corte é o assunto: lá o gesto autora um
//! **JOINT** (a âncora, o limite, o comprimento, o trilho); aqui ele autora uma
//! **RODLANA**, que é outra entidade e não é um joint nenhum.
//!
//! O irmão já confessava a costura no próprio comentário (*"`joint` aqui é a
//! RODA, não a corda"*): as três alças de roldana viajam pela máquina de arrasto
//! do joint porque a máquina é boa, não porque a coisa arrastada seja a mesma.
//!
//! Filho e não irmão para alcançar os privados de [`super`] (`Grab`, `distance`)
//! sem os tornar públicos — o mesmo `#[path]` que os `mod tests` deste repo usam.

use super::{Grab, distance};
use ph2d_ecs::SimWorld;
use ph2d_editor::gizmo::PointHandleKind;

/// **Que raio esta alça mede** — a porta única do `open_drag` e do apply.
///
/// Um `WheelRimOut` sobre uma roldana comum não deveria existir (o publicador não
/// o oferece), e se existisse mediria o raio de entrada: o fallback é o valor que
/// a alça está desenhada em cima, nunca zero.
pub(super) fn wheel_radius_of(w: &ph2d_physics_ecs::PulleyWheel, kind: PointHandleKind) -> f32 {
    if matches!(kind, PointHandleKind::WheelRimOut) && w.radius_out > 0.0 {
        w.radius_out
    } else {
        w.radius
    }
}

/// **O agarre de uma alça de roldana** — mover o centro, ou dimensionar um raio.
///
/// `None` quando a entidade não é (mais) uma roldana desenhável: o gesto
/// simplesmente não abre, que é o que se quer de uma alça cujo alvo sumiu.
pub(super) fn open_grab(
    sim: &SimWorld,
    wheel: ph2d_ecs::Entity,
    kind: PointHandleKind,
    cursor: [f32; 2],
) -> Option<Grab> {
    // ⚠️ **A entidade aqui é a RODA, não a corda** — uma roldana é uma entidade
    // própria (W-Pulley W1), e é o `Transform` dela que estas alças autoram.
    let t = sim.world().get::<ph2d_ecs::Transform>(wheel)?;
    let centre = [t.translation.x, t.translation.y];
    if kind.is_wheel_radius() {
        // ⚠️ **Uma pergunta, dois raios.** Qual deles esta alça dimensiona é
        // decidido AQUI e no apply pela MESMA porta, senão agarrar o aro de saída
        // mediria um número e escreveria noutro — e a alça saltaria no clique pela
        // diferença dos dois raios.
        let w = sim
            .world()
            .get::<ph2d_physics_ecs::PulleyWheel>(wheel)?
            .clamped();
        Some(Grab::Radius(
            wheel_radius_of(&w, kind) - distance(centre, cursor),
        ))
    } else {
        Some(Grab::World([centre[0] - cursor[0], centre[1] - cursor[1]]))
    }
}

/// **Mover uma roldana** — escrever o `Transform` dela, e re-colocar o eixo.
///
/// ⚠️ **Numa roldana MONTADA o `Transform` é DERIVADO** (`corpo · local`), então
/// sem desarmar o sentinela o `sync_mounted_wheels` reescreve o número no frame
/// seguinte: a alça anda com o dedo e volta ao soltar, sem erro e sem aviso. A
/// porta é a MESMA que a row Position usa (W6) — dois gestos que dizem a mesma
/// coisa não podem discordar sobre o que ela significa.
///
/// O undo global por-diff captura as duas escritas como captura a de qualquer
/// objeto.
pub(super) fn move_wheel(sim: &mut SimWorld, wheel: ph2d_ecs::Entity, to: [f32; 2]) {
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(wheel) {
        t.translation = ph2d_core::Vec2::new(to[0], to[1]);
    }
    ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), wheel);
}

/// **Dimensionar um raio** — o de entrada ou o de saída, pela porta acima.
///
/// ⚠️ **O piso do raio de SAÍDA é o mesmo do de entrada, e não zero:**
/// `radius_out = 0` é o sentinela de *"roldana comum"* (W4), então deixar o
/// arrasto chegar a zero faria a alça se APAGAR debaixo do dedo — e com ela a
/// engrenagem que o artista estava dimensionando. Quem desliga um diferencial é a
/// row, digitando 0.
pub(super) fn resize_wheel(
    sim: &mut SimWorld,
    wheel: ph2d_ecs::Entity,
    kind: PointHandleKind,
    cursor: [f32; 2],
    off: f32,
) {
    let Some(centre) = sim
        .world()
        .get::<ph2d_ecs::Transform>(wheel)
        .map(|t| [t.translation.x, t.translation.y])
    else {
        return;
    };
    {
        let Some(mut w) = sim
            .world_mut()
            .get_mut::<ph2d_physics_ecs::PulleyWheel>(wheel)
        else {
            return;
        };
        let r = (distance(centre, cursor) + off).max(ph2d_physics_ecs::PulleyWheel::MIN_RADIUS);
        if matches!(kind, PointHandleKind::WheelRimOut) {
            w.radius_out = r;
        } else {
            w.radius = r;
        }
    }
    // ⚠️ **Crescer o raio CRESCE a rota** (o abraço é maior), e o `L0` da corda
    // era semeado UMA vez e congelado ⇒ a restrição `L(rota) <= L0` nascia
    // VIOLADA e o solver comia a diferença num tick: medido **14,12 m de salto**
    // a raio 0,90 e **50,43 m** a 1,50, com a carga arremessada para fora de
    // quadro. A porta re-abre o que depende da geometria desta roldana.
    ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), wheel);
}
