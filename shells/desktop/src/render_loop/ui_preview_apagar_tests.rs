//! Irmão de `ui_preview_tests.rs` pelo tecto de 600 (HR-18): o apagar na Hierarquia durante o Preview.

use super::*;

/// ⭐ **F22 (plano 33 §6.6) — apagar um hospedeiro na Hierarquia DURANTE o Preview e sair: ele não
/// volta.** A Hierarquia fica aberta no Preview (F19: o que ela muda persiste ao sair); o `leave`
/// reinstala as poses capturadas, e uma pose de um objeto que já não existe não o pode ressuscitar.
#[test]
fn a_host_deleted_during_the_preview_stays_deleted_after_leaving() {
    let (mut sim, mut scene, mut map, mut states) = world();
    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    assert!(pv.enter(&mut machines, &states, &mut sim, &mut scene, &map));
    pv.point(&mut machines, &states, &[HOST], false);

    // O que o apagar da Hierarquia deixa: o caminho fora da cena, a entidade fora do mundo.
    assert!(scene.remove_path(HOST));
    let e = ph2d_ecs::Entity::from_bits(map.remove(&HOST).unwrap());
    sim.world_mut().despawn(e);
    crate::render_loop::ui_state_bridge::dispatch(
        &mut machines,
        &mut states,
        (&mut sim, &mut scene, &map),
        0.05,
        &mut crate::render_loop::ui_state_bridge::Cooked::default(),
        &|_| false,
    );

    assert!(pv.leave(&mut machines, &mut sim, &mut scene, &map));
    assert!(
        scene.paths().iter().all(|p| p.id != HOST),
        "sair do Preview ressuscitou o hospedeiro apagado"
    );
    assert!(states.get(HOST).is_empty(), "os estados do apagado ficaram");
    assert!(
        (x_of(&sim, &map, CHILD) - 5.0).abs() < 1e-9,
        "o vizinho mexeu: x = {}",
        x_of(&sim, &map, CHILD)
    );
}
