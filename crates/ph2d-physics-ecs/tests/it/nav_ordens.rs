//! ⭐⭐⭐ **As ORDENS de navegação** (plano 30, W6) — os verbos `Start/Stop Navigation` pela porta
//! `PhysicsBridge::pede_navegacao`: valem a partir do tique seguinte, mandam mais que o autorado sem
//! o reescrever, e um scrub refá-las ao bit (a fita por tique, o idioma da vida).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_nav::Status;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, PedidoDeNavegacao,
    PhysicsBridge, RigidBody, TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

fn cena(ativo: bool) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [8.0, 6.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Outro"),
        Transform::from_translation(Vec2::new(-5.0, 5.0)),
    ));
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Guarda"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            TopDownPlayer::from_law(TopDownLaw {
                default_controls: false,
                direction: DirectionMode::Free,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Point([6.0, 0.0]),
                active: ativo,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(-6.0, 0.0)),
        ))
        .id();
    (sim, PhysicsBridge::new(), quem)
}

fn pos(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    (t.translation.x, t.translation.y)
}

fn corre(sim: &mut SimWorld, b: &mut PhysicsBridge, e: Entity, de: u64, ate: u64) {
    for t in de..=ate {
        b.dispatch(sim, true, t);
    }
    let _ = pos(sim, e);
}

#[test]
fn stop_para_o_agente_e_start_retoma_o() {
    let (mut sim, mut b, quem) = cena(true);
    corre(&mut sim, &mut b, quem, 1, 20);
    let andou = pos(&sim, quem);
    assert!(andou.0 > -5.5, "o guarda nem saiu do sítio: {andou:?}");
    b.pede_navegacao(quem, PedidoDeNavegacao::Para);
    // O pedido vale a partir do tique SEGUINTE ao que o grava; o mover trava em poucos tiques.
    corre(&mut sim, &mut b, quem, 21, 40);
    let parado = pos(&sim, quem);
    corre(&mut sim, &mut b, quem, 41, 60);
    assert_eq!(pos(&sim, quem), parado, "o `Stop` não parou o guarda");
    assert_eq!(b.nav_agent(quem).map(|r| r.status), Some(Status::Idle));
    b.pede_navegacao(quem, PedidoDeNavegacao::Anda(0));
    corre(&mut sim, &mut b, quem, 61, 80);
    assert!(
        pos(&sim, quem).0 > parado.0 + 0.5,
        "o `Start` não o pôs a andar outra vez"
    );
    // ⛔ Nada disto é documento.
    let a = sim.world().get::<NavAgent>(quem).expect("o agente");
    assert!(a.active && a.target == NavTarget::Point([6.0, 0.0]));
}

#[test]
fn start_com_um_nome_persegue_esse_objecto() {
    let (mut sim, mut b, quem) = cena(true);
    corre(&mut sim, &mut b, quem, 1, 5);
    b.pede_navegacao(quem, PedidoDeNavegacao::Anda(stable_name_id("Outro")));
    corre(&mut sim, &mut b, quem, 6, 60);
    let p = pos(&sim, quem);
    assert!(
        p.1 > 1.0,
        "o guarda não foi atrás do «Outro» (em −5, 5): {p:?}"
    );
    // E um `Start` vazio devolve-o ao alvo AUTORADO.
    b.pede_navegacao(quem, PedidoDeNavegacao::Anda(0));
    corre(&mut sim, &mut b, quem, 61, 62);
    let fim = b.nav_agent(quem).and_then(|r| r.path.last().copied());
    assert_eq!(
        fim,
        Some([6.0, 0.0]),
        "o `Start` vazio não voltou ao alvo autorado"
    );
}

#[test]
fn start_liga_um_agente_autorado_desligado() {
    let (mut sim, mut b, quem) = cena(false);
    corre(&mut sim, &mut b, quem, 1, 20);
    assert_eq!(
        pos(&sim, quem),
        (-6.0, 0.0),
        "o CONTROLO: desligado não anda"
    );
    b.pede_navegacao(quem, PedidoDeNavegacao::Anda(0));
    corre(&mut sim, &mut b, quem, 21, 40);
    assert!(pos(&sim, quem).0 > -5.5, "o `Start` não ligou o agente");
    assert!(
        !sim.world().get::<NavAgent>(quem).expect("o agente").active,
        "a ordem reescreveu o `Active` autorado"
    );
}

#[test]
fn um_scrub_refaz_as_ordens_ao_bit() {
    let (mut sim, mut b, quem) = cena(true);
    let mut primeira = Vec::new();
    for t in 1..=120u64 {
        if t == 30 {
            b.pede_navegacao(quem, PedidoDeNavegacao::Para);
        }
        if t == 70 {
            b.pede_navegacao(quem, PedidoDeNavegacao::Anda(stable_name_id("Outro")));
        }
        b.dispatch(&mut sim, true, t);
        primeira.push(pos(&sim, quem));
    }
    // O fenómeno está na fixtura: o guarda esteve parado entre as duas ordens.
    assert_eq!(primeira[50], primeira[60], "o `Stop` não segurou o guarda");
    // O scrub relê as ordens da FITA: a 90 passa pelas duas, a 45 só pela primeira — e o 45 sai de
    // um checkpoint cuja ordem em vigor (o `Stop`) é OUTRA que a de agora (o `Start` do tique 70):
    // sem a ordem no anel o guarda replayado iria atrás do «Outro».
    for meio in [90u64, 45] {
        b.dispatch(&mut sim, false, meio);
        assert_eq!(
            pos(&sim, quem),
            primeira[(meio - 1) as usize],
            "o scrub a {meio}"
        );
    }
    // ⚠️ Tocar de novo depois de um scrub é AUTORAR POR CIMA (a regra da fita da vida): um tique
    // vivo sem pedido apaga o que a fita tinha nele — a ordem do tique 70 já não volta sozinha.
    b.dispatch(&mut sim, false, 60);
    for t in 61..=90 {
        b.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        pos(&sim, quem),
        primeira[59],
        "sem a ordem do tique 70 o guarda fica parado"
    );
}
