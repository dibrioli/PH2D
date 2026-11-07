//! ⭐ A fila «Collides with» do Inspector, da ação do bus ao motor (ficha f1, onda 6): a edição chega ao
//! mundo, a bola passa ou bate, e o losango e o Auto Key são os da célula. O clique real está em
//! `ph2d-panel-inspector` (`os_quadros_da_colisao`) e o passo de desfazer na shell (`undo_drag_tests`).

use crate::canais_mundo::{self, ConducaoMundo, kc};
use crate::marcas_mundo;
use crate::panel_bridge::settings_do_painel;
use ph2d_core::Vec2;
use ph2d_ecs::canal_mundo as m;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, SimWorld, Transform};
use ph2d_panel_inspector::ids::INSP_PHYS_COLLIDES;
use ph2d_panel_physics::ids::{PHYSICS_LAYER_CELL, physics_layer_cell_index};
use ph2d_panel_physics::{PhysicsIntent, PhysicsSnapshot, alterna_par, drain_intents};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LayerMatrix, LayerNames, PhysicsBridge, PhysicsSettings,
    RigidBody,
};

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

/// Bola parada na camada `a`, bola que cai em cima dela na `b`; devolve a altura da que cai.
fn duas_bolas(settings: PhysicsSettings, a: u8, b: u8) -> f32 {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 1.0 },
            layer: a,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let bola = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                layer: b,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 2.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(settings);
    for tick in 1..=120 {
        bridge.dispatch(&mut sim, true, tick);
    }
    sim.world()
        .get::<Transform>(bola)
        .expect("bola")
        .translation
        .y
}

#[test]
fn o_quadro_decide_no_motor_se_as_bolas_de_camadas_diferentes_batem() {
    let mut ponte = PhysicsBridge::new();
    let mut mundo = ConducaoMundo::default();
    assert!(duas_bolas(ponte.settings(), 1, 3) > 0.5, "de fábrica batem");
    clique_no_quadro(&mut ponte, &mut mundo, 3, 1);
    let m = LayerMatrix::from_rows(ponte.settings().layer_matrix);
    assert!(!m.collides(3, 1) && !m.collides(1, 3), "os dois sentidos");
    assert!(duas_bolas(ponte.settings(), 1, 3) < -2.0, "apagado, passa");
    assert!(duas_bolas(ponte.settings(), 3, 1) < -2.0, "e ao contrário");
    assert!(duas_bolas(ponte.settings(), 1, 2) > 0.5, "o controlo bate");
    clique_no_quadro(&mut ponte, &mut mundo, 1, 3);
    assert_eq!(
        ponte.settings().layer_matrix,
        PhysicsSettings::default().layer_matrix,
        "o quadro da outra camada apaga o mesmo par: a célula e os dois quadros são um facto"
    );
    assert!(duas_bolas(ponte.settings(), 1, 3) > 0.5, "aceso, bate");
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

/// Sonda (`--ignored --nocapture`, ficha f1): o documento depois do losango da matriz, com 1, 36 e 528 curvas,
/// intercaladas, o mínimo de 200: bytes do blob, salvar, abrir, clonar (o passo de desfazer) e um quadro.
#[test]
#[ignore = "mede: corre à mão"]
fn mede_o_losango_da_matriz() {
    use ph2d_timeline::{Instante, PropKind, TimelineIntent, TimelineState, apply_intent};
    use std::time::Instant as T;
    let monta = |n: u16| {
        let mut sim = SimWorld::new();
        ph2d_ecs::garante_cena(sim.world_mut());
        let c = ph2d_ecs::bits_da_cena(sim.world()).expect("cena");
        let mut tl = TimelineState::default();
        let mut relogio = ph2d_core::Playhead::default();
        for i in 0..n {
            apply_intent(
                &mut tl,
                &mut relogio,
                TimelineIntent::AddKey {
                    entity: c,
                    prop: PropKind::Field(kc(i)),
                    t: ph2d_anim::RationalTime::ZERO,
                    value: ph2d_anim::AnimValue::Float(1.0),
                    interp: ph2d_anim::Interp::Eased(ph2d_anim::Easing::new(
                        ph2d_anim::EasingFamily::Cubic,
                        ph2d_anim::EasingMode::InOut,
                    )),
                },
            );
        }
        (sim, tl)
    };
    let casos = [1u16, 36, m::CELULAS];
    let mut clique = [f64::MAX; 3];
    for _ in 0..20 {
        for (v, &n) in casos.iter().enumerate() {
            let t = T::now();
            std::hint::black_box(monta(n));
            clique[v] = clique[v].min(t.elapsed().as_secs_f64() * 1e6);
        }
    }
    println!("o clique (monta o doc), µs: {clique:.0?}");
    let mut mundos: Vec<_> = casos.iter().map(|&n| monta(n)).collect();
    let mut ponte = PhysicsBridge::new();
    let mut cond = ConducaoMundo::default();
    let mut min = [[f64::MAX; 5]; 3];
    let us = |t: T| t.elapsed().as_secs_f64() * 1e6;
    for _ in 0..200 {
        for (v, (sim, tl)) in mundos.iter_mut().enumerate() {
            let t = T::now();
            let bytes = tl.doc.to_bytes().expect("salva");
            min[v][0] = min[v][0].min(us(t));
            let t = T::now();
            let lido = ph2d_timeline::TimelineDoc::from_bytes(&bytes).expect("abre");
            min[v][1] = min[v][1].min(us(t));
            assert_eq!(lido.bindings().len(), tl.doc.bindings().len());
            let t = T::now();
            let copia = tl.doc.clone();
            min[v][2] = min[v][2].min(us(t));
            std::hint::black_box(copia);
            let t = T::now();
            ph2d_timeline::apply_active_clip(sim.world_mut(), &mut tl.doc, 0.5, |_| false);
            canais_mundo::publicar(sim, &ponte);
            canais_mundo::aplicar(sim, &mut ponte, &mut cond);
            min[v][3] = min[v][3].min(us(t));
            let t = T::now();
            let inst = Instante {
                t_cut: 0.5,
                solo: false,
            };
            let marcas = marcas_mundo::marcas(&tl.doc, sim.world(), inst, |_, _| Some(1.0));
            min[v][4] = min[v][4].min(us(t));
            std::hint::black_box(marcas);
        }
    }
    for (v, (_, tl)) in mundos.iter().enumerate() {
        println!(
            "curvas {:>3}: blob {:>6} B · salvar {:>7.1} µs · abrir {:>7.1} µs · clonar {:>7.1} µs · quadro {:>7.1} µs · marcas {:>7.1} µs",
            casos[v],
            tl.doc.to_bytes().expect("salva").len(),
            min[v][0],
            min[v][1],
            min[v][2],
            min[v][3],
            min[v][4]
        );
    }
}
