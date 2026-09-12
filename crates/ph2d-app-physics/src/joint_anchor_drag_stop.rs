//! **A metade do LIMITADOR do gesto de alça** (W-RopeStop) — módulo FILHO de
//! [`super`] pelo teto de 600 LOC, irmão exato do
//! [`super::wheel`](../joint_anchor_drag_wheel.rs).
//!
//! O corte é o ASSUNTO, como no irmão: lá o gesto autora uma **roldana**, aqui o
//! [`RopeStops`] da **corda** — quanta corda tem de sobrar em cada ponta antes de
//! ela encostar na roldana.
//!
//! [`RopeStops`]: ph2d_physics_ecs::RopeStops

use ph2d_ecs::SimWorld;

/// **Os dois limitadores autorados de uma corda** (W-RopeStop) — `[A, B]`.
///
/// Porta única de LEITURA: o `open_drag` a usa para o agarre relativo, o apply
/// para preservar a outra ponta, e o publicador de alças para saber onde pôr a
/// marca. Ausente é `[0, 0]`, que é a trava no próprio aro — o estado de toda
/// corda que ninguém limitou.
pub(crate) fn stops_of(sim: &SimWorld, rope: ph2d_ecs::Entity) -> [f32; 2] {
    sim.world()
        .get::<ph2d_physics_ecs::RopeStops>(rope)
        .map_or([0.0, 0.0], |s| s.pair())
}

/// **Escrever um limitador**, preservando o outro.
///
/// ⚠️ **Insere o componente quando ele não existe**, e é isso que faz o gesto
/// funcionar numa corda que nunca foi limitada: `RopeStops` é opcional (ausente ==
/// zero), então o primeiro arrasto é também o que o cria. O undo global por-diff
/// captura as duas coisas como captura qualquer edição de objeto.
pub(super) fn write_stop(sim: &mut SimWorld, rope: ph2d_ecs::Entity, side: usize, value: f32) {
    let mut pair = stops_of(sim, rope);
    pair[side] = value.max(0.0);
    sim.world_mut()
        .entity_mut(rope)
        .insert(ph2d_physics_ecs::RopeStops {
            a: pair[0],
            b: pair[1],
        });
}
