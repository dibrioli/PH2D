//! ⭐ A fila «Collides with» do Inspector (ficha f1, onda 6): o losango e o Auto Key são os da célula. O clique
//! real está em `ph2d-panel-inspector` (`os_quadros_da_colisao`), o motor em `ph2d-panel-physics` (`os_cartoes`)
//! e o passo de desfazer na shell (`fase_bus_inspector_preso_tests`).

use crate::canais_mundo::{self, ConducaoMundo, kc};
use crate::marcas_mundo;
use crate::panel_bridge::settings_do_painel;
use ph2d_ecs::canal_mundo as m;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, SimWorld};
use ph2d_panel_inspector::ids::INSP_PHYS_COLLIDES;
use ph2d_panel_physics::ids::{PHYSICS_LAYER_CELL, physics_layer_cell_index};
use ph2d_panel_physics::{PhysicsIntent, PhysicsSnapshot, alterna_par, drain_intents};
use ph2d_physics_ecs::{LayerMatrix, LayerNames, PhysicsBridge};

/// Publica a ponte como a shell, alterna o par pela porta do quadro e aplica o intent.
fn clique_no_quadro(ponte: &mut PhysicsBridge, mundo: &mut ConducaoMundo, a: usize, b: usize) {
    ph2d_panel_physics::set_current_physics(Some(PhysicsSnapshot {
        settings: ponte.settings(),
        vivo: ponte.settings_vivos(),
        ..PhysicsSnapshot::default()
    }));
    let _ = drain_intents();
    alterna_par(a, b);
    let mut pedidos = drain_intents();
    assert_eq!(pedidos.len(), 1, "um clique, um pedido");
    let Some(PhysicsIntent::SetSettings(s, t)) = pedidos.pop() else {
        panic!("o quadro não pediu settings");
    };
    settings_do_painel(ponte, mundo, *s, *t);
}

/// A curva da Cena conduz o par e o clique no quadro ganha-lhe, como na célula: o Auto Key lê a mão.
#[test]
fn o_auto_key_le_o_quadro_como_le_a_celula() {
    let mut sim = SimWorld::new();
    ph2d_ecs::garante_cena(sim.world_mut());
    let c = ph2d_ecs::bits_da_cena(sim.world()).expect("cena");
    let i = physics_layer_cell_index(3, 1);
    let chave = kc(u16::try_from(i).expect("célula"));
    let mut ponte = PhysicsBridge::new();
    let mut mundo = ConducaoMundo::default();
    // Dois quadros com a curva a dizer «apagado»: o vivo mostra apagado e o autorado fica aceso.
    for _ in 0..2 {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(c, chave, 0.0);
        canais_mundo::aplicar(&mut sim, &mut ponte, &mut mundo);
    }
    assert!(!LayerMatrix::from_rows(ponte.settings_vivos().layer_matrix).collides(3, 1));
    // O quadro, apagado à vista, acende: o autorado já valia isso, e a mão ganha mesmo assim.
    clique_no_quadro(&mut ponte, &mut mundo, 3, 1);
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(c, chave, 0.0);
    canais_mundo::aplicar(&mut sim, &mut ponte, &mut mundo);
    assert!(
        LayerMatrix::from_rows(ponte.settings_vivos().layer_matrix).collides(3, 1),
        "a mão ganha à curva no quadro do clique"
    );
    canais_mundo::publicar(&mut sim, &ponte);
    assert_eq!(
        sim.world().resource::<CanaisLidos>().valor(c, chave),
        Some(1.0),
        "o Auto Key lê o que a mão pediu"
    );
}

/// O losango da fila é o da matriz: o mesmo grupo de canais, quadro a quadro.
#[test]
fn o_losango_da_fila_e_o_grupo_da_matriz() {
    let da_celula = marcas_mundo::canal_do_campo(PHYSICS_LAYER_CELL[0]);
    assert_eq!(da_celula.len(), usize::from(m::CELULAS));
    for &q in &INSP_PHYS_COLLIDES {
        assert_eq!(marcas_mundo::canal_do_campo(q), da_celula);
    }
}

/// O «+» da fila e o «+ Layer» da matriz escolhem a mesma camada.
#[test]
fn o_mais_da_fila_escolhe_a_camada_do_mais_da_matriz() {
    use ph2d_panel_inspector::camadas_fisica::{proxima_livre, set_current};
    for bits in [1u32, 0b1011, 0xFF, u32::MAX >> 1, u32::MAX] {
        set_current(Default::default(), [u32::MAX; 32], bits);
        assert_eq!(proxima_livre(), LayerNames::next_free(bits), "{bits:#x}");
    }
    set_current(Default::default(), [u32::MAX; 32], 0xFF);
}
