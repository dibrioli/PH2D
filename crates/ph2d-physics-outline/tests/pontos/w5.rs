//! **W5 — os pontos do «Outline» no canvas** pelas portas reais do ponteiro (`ph2d_shape_handles::Porta`), em
//! ecrã, sobre a argola traçada (`followed_ring`: 4 m × 4 m, buraco de 2 m no meio).

use super::*;
use ph2d_editor_core::action_bus::ActionBus;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_shape_handles::pontos::{PontosNoCanvas, do_alvo, locais};
use ph2d_shape_handles::{Porta, Teclas, alterna};

const WIN: WindowSize = WindowSize {
    width: 1400,
    height: 900,
};
fn cam() -> Camera2d {
    Camera2d::new([0.0, 0.0], 10.0)
}

fn porta<'a>(sim: &'a mut SimWorld, bus: &'a mut ActionBus) -> Porta<'a> {
    Porta {
        world: sim.world_mut(),
        camera: cam(),
        window: WIN,
        bus,
    }
}

/// Onde o ponto de mundo `w` está em ecrã.
fn ecra(w: [f32; 2]) -> (f32, f32) {
    cam().world_to_screen(w, WIN)
}

/// O ponto do anel `anel` mais perto de `w` (mundo; o corpo está na origem, sem rodar).
fn perto(sim: &SimWorld, shape: ph2d_ecs::Entity, anel: usize, w: [f32; 2]) -> [f32; 2] {
    let (pose, m, o) = do_alvo(sim.world(), shape.to_bits()).expect("pontos");
    locais(&pose, &m, &o)[anel]
        .iter()
        .copied()
        .min_by(|a, b| {
            let d = |p: &[f32; 2]| (p[0] - w[0]).hypot(p[1] - w[1]);
            d(a).total_cmp(&d(b))
        })
        .expect("anel")
}

fn arrasta(sim: &mut SimWorld, de: [f32; 2], para: [f32; 2]) {
    let mut bus = ActionBus::default();
    let mut p = porta(sim, &mut bus);
    assert!(p.desce_com(ecra(de), Teclas::default(), false), "agarrou");
    assert!(p.move_(ecra(para), Teclas::default()));
    assert!(p.sobe());
}

fn outline(sim: &SimWorld, shape: ph2d_ecs::Entity) -> ColliderOutline {
    sim.world()
        .get::<ColliderOutline>(shape)
        .cloned()
        .expect("contorno")
}

/// Uma bola largada em `(0, 4)` pousa no topo (raio 0,3).
fn pousa(sim: &mut SimWorld) -> f32 {
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 4.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240 {
        bridge.dispatch(sim, true, tick);
    }
    let y = bridge.body_pose(ball).expect("corpo").1;
    sim.world_mut().despawn(ball);
    y
}

/// ⭐ (1) **Arrastar um ponto move-o e o motor segue**: os dois cantos de cima sobem 1 m (dois gestos) e a bola
/// pousa 1 m acima; o `Esc` a meio devolve os anéis do `Down` (o gesto é um valor só).
/// *Mutação: o `move_` dos pontos não escrever ⇒ a bola pousa onde pousava.*
#[test]
fn arrastar_um_ponto_move_o_e_a_bola_pousa_na_nova_altura() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let antes = pousa(&mut sim);
    assert!((antes - 2.3).abs() < 0.05, "no topo da argola: {antes}");
    let inicio = outline(&sim, shape);
    // Esc a meio: nada fica.
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    {
        let mut bus = ActionBus::default();
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(de), Teclas::default(), false));
        assert!(p.move_(ecra([de[0], de[1] + 1.0]), Teclas::default()));
        assert!(p.esc());
    }
    assert_eq!(outline(&sim, shape), inicio, "o Esc devolve o Down");
    for x in [-2.0, 2.0] {
        let de = perto(&sim, shape, 0, [x, 2.0]);
        arrasta(&mut sim, de, [de[0], de[1] + 1.0]);
        let agora = perto(&sim, shape, 0, [x, 3.0]);
        assert!(
            (agora[1] - (de[1] + 1.0)).abs() < 0.01,
            "o ponto foi ao ponteiro: {agora:?}"
        );
    }
    let depois = pousa(&mut sim);
    assert!(
        (depois - 3.3).abs() < 0.05,
        "a bola pousa 1 m acima: {depois}"
    );
}

/// ⭐ (2) **Duplo clique numa aresta acrescenta o ponto ONDE se clicou; Alt+clique tira-o**; tirar um canto de
/// fora é recusado (a diagonal cruzaria o buraco); um clique simples na aresta não acrescenta.
/// *Mutações: `acrescentado` no índice errado ⇒ o anel cruza-se e nada entra; `tirado` sem `validos` ⇒ o canto
/// sai.*
#[test]
fn duplo_clique_acrescenta_e_alt_clique_tira() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let n = |sim: &SimWorld| outline(sim, shape).rings[0].len();
    let n0 = n(&sim);
    let clique = [0.5, 2.0];
    let mut bus = ActionBus::default();
    {
        let mut p = porta(&mut sim, &mut bus);
        let _ = p.desce_com(ecra(clique), Teclas::default(), false);
        let _ = p.sobe();
    }
    assert_eq!(n(&sim), n0, "um clique simples não acrescenta");
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(clique), Teclas::default(), true));
        let _ = p.sobe();
    }
    assert_eq!(n(&sim), n0 + 1, "o duplo clique acrescentou");
    let novo = perto(&sim, shape, 0, clique);
    assert!(
        (novo[0] - clique[0]).hypot(novo[1] - clique[1]) < 0.01,
        "o ponto nasce onde se clicou: {novo:?}"
    );
    assert!(outline(&sim, shape).custom, "editar pontos passa a Custom");
    let alt = Teclas {
        alt: true,
        ..Teclas::default()
    };
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(novo), alt, false));
        let _ = p.sobe();
    }
    assert_eq!(n(&sim), n0, "Alt+clique tirou-o");
    let canto = perto(&sim, shape, 0, [-2.0, 2.0]);
    {
        let mut p = porta(&mut sim, &mut bus);
        assert!(p.desce_com(ecra(canto), alt, false));
        let _ = p.sobe();
    }
    assert_eq!(n(&sim), n0, "o canto não sai: a diagonal cruzaria o buraco");
}

/// ⭐ (3) **Arrastar para um cruzamento é recusado**: o canto de cima à esquerda levado para dentro do buraco
/// fica na última posição válida (antes de cruzar).
/// *Mutação: `movido` sem `validos` ⇒ o canto entra no buraco.*
#[test]
fn o_arrasto_que_cruzaria_fica_na_ultima_posicao_valida() {
    let (_reg, mut sim, _body, shape, _db) = followed_ring();
    alterna(sim.world_mut(), shape.to_bits());
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    let mut bus = ActionBus::default();
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.desce_com(ecra(de), Teclas::default(), false));
    assert!(p.move_(ecra([-2.5, 2.5]), Teclas::default()));
    assert!(p.move_(ecra([0.0, 0.0]), Teclas::default()));
    assert!(p.sobe());
    let agora = perto(&sim, shape, 0, [-2.5, 2.5]);
    assert!(
        (agora[0] + 2.5).abs() < 0.01 && (agora[1] - 2.5).abs() < 0.01,
        "fica na última válida: {agora:?}"
    );
}

/// ⭐ (4) **Os pontos só com a FORMA em edição** (resposta 3 do dono): o corpo não os tem; a forma tem.
/// *Mutação: `do_alvo` pela 1.ª forma do corpo ⇒ o corpo agarra um ponto.*
#[test]
fn os_pontos_so_aparecem_com_a_forma_selecionada() {
    let (_reg, mut sim, body, shape, _db) = followed_ring();
    let canto = perto(&sim, shape, 0, [-2.0, 2.0]);
    alterna(sim.world_mut(), body.to_bits());
    assert!(
        do_alvo(sim.world(), body.to_bits()).is_none(),
        "o corpo não tem pontos"
    );
    let mut bus = ActionBus::default();
    {
        let mut p = porta(&mut sim, &mut bus);
        let _ = p.desce_com(ecra(canto), Teclas::default(), false);
    }
    assert!(
        sim.world()
            .get_resource::<PontosNoCanvas>()
            .is_none_or(|p| p.arrasto.is_none()),
        "com o corpo, nenhum ponto se agarra"
    );
    alterna(sim.world_mut(), shape.to_bits());
    assert!(
        do_alvo(sim.world(), shape.to_bits()).is_some(),
        "a forma tem pontos"
    );
    let mut p = porta(&mut sim, &mut bus);
    assert!(p.desce_com(ecra(canto), Teclas::default(), false));
    assert!(
        p.world
            .get_resource::<PontosNoCanvas>()
            .is_some_and(|p| p.arrasto.is_some()),
        "com a forma, o ponto agarra-se"
    );
}

/// ⭐ (5) **Editar pontos passa a «Custom»; a imagem nova não lhes toca; «Fit to picture» retraça** e volta
/// a seguir. *Mutações: `escreve` sem `custom` ⇒ segue; `follows` sem o `custom` ⇒ o cartão diz Follows.*
#[test]
fn editar_pontos_e_custom_e_o_fit_retraca() {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let (reg, mut sim, body, shape, db) = followed_ring();
    let traced = outline(&sim, shape);
    assert_eq!(
        ph2d_physics_outline::follows(sim.world(), shape),
        Some(true)
    );
    alterna(sim.world_mut(), shape.to_bits());
    let de = perto(&sim, shape, 0, [-2.0, 2.0]);
    arrasta(&mut sim, de, [de[0], de[1] + 1.0]);
    assert_eq!(
        ph2d_physics_outline::follows(sim.world(), shape),
        Some(false),
        "Custom"
    );
    let editado = outline(&sim, shape);
    let q = EditorCommandQueue::new();
    let disc = db.insert_image_rgba8(32, 32, img(32, 32, |_, _| true));
    sim.world_mut().entity_mut(body).insert(SpritePixels(disc));
    assert_eq!(
        follow(&mut sim, &db, &q, &reg),
        0,
        "o Custom não segue a imagem"
    );
    assert_eq!(outline(&sim, shape), editado);
    sim.world_mut()
        .entity_mut(body)
        .insert(SpritePixels(db.insert_image_rgba8(32, 32, ring_px())));
    let none = std::collections::BTreeMap::new();
    let fit = [(shape.to_bits(), P::FitToPicture)];
    ph2d_physics_outline::retrace_edited(sim.world(), &fit, &db, &none, PPM, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    assert_eq!(outline(&sim, shape), traced, "o Fit devolve o traço");
    assert_eq!(
        ph2d_physics_outline::follows(sim.world(), shape),
        Some(true)
    );
}
