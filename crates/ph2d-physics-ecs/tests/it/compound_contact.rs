//! **Um corpo COMPOSTO toca UMA vez** (W-CompoundContact).
//!
//! O terceiro canal que envelheceu quando a W-Compound tornou falso *"um corpo tem
//! exatamente um collider"* — depois do canal de TRIGGER (W-PartSensor) e das ZONAS
//! (W-CompoundZone). Aqui a frase estava escrita no doc do módulo como uma LEI:
//!
//! > *"uma caixa deitada tem duas quinas; relatar cada uma responde **quantas
//! > quinas**, fato sobre tesselação, não sobre a cena — dois objetos se tocando é UM
//! > evento"*
//!
//! Ela valia para **pontos de contato** e quebrou para **FORMAS**: `contact_pairs()`
//! itera pares de COLLIDER, e enquanto um corpo tinha um só, *"uma entrada por par de
//! collider"* e *"uma entrada por par de corpos"* eram a mesma frase.
//!
//! ⚠️ **Todo gate aqui compara contra o CONTROLE** — a jangada de UMA peça, mesma
//! silhueta e mesma massa. Sem ele nenhum número é atribuível: *"a composta relata 2"*
//! não distingue *a fusão está faltando* de *a física é assim mesmo*.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const HALF_X: f32 = 0.6;
const HALF_Y: f32 = 0.25;

fn cuboid(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        density: 1.0,
        ..Collider::default()
    }
}

/// `compound = false` ⇒ UMA caixa larga (o CONTROLE).
/// `compound = true`  ⇒ a MESMA silhueta partida em duas, a segunda como PEÇA.
///
/// As duas ocupam `x ∈ [−0,6; 1,8]` e têm massa `1,200000` (medida).
fn raft(sim: &mut SimWorld, compound: bool, drop_from: f32) -> Entity {
    let hull = if compound { HALF_X } else { HALF_X * 2.0 };
    let x = if compound { 0.0 } else { HALF_X };
    let body = sim
        .world_mut()
        .spawn((
            Name::new("Raft"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            cuboid(hull, HALF_Y),
            Transform::from_translation(Vec2::new(x, drop_from)),
        ))
        .id();
    if compound {
        sim.world_mut().spawn((
            Name::new("Raft Deck"),
            cuboid(HALF_X, HALF_Y),
            Transform::from_translation(Vec2::new(HALF_X * 2.0, 0.0)),
            ChildOf(body),
        ));
    }
    body
}

fn ground(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(10.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
}

/// Larga a jangada e simula até assentar, pela PORTA REAL (a ponte).
fn settled(compound: bool) -> (PhysicsBridge, SimWorld, Entity) {
    run(compound, 1.0, 180)
}

fn run(compound: bool, drop_from: f32, ticks: u64) -> (PhysicsBridge, SimWorld, Entity) {
    let mut sim = SimWorld::new();
    ground(&mut sim);
    let raft = raft(&mut sim, compound, drop_from);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    (bridge, sim, raft)
}

fn mine(bridge: &PhysicsBridge, raft: Entity) -> Vec<ph2d_physics_ecs::BodyContact> {
    bridge
        .contacts()
        .iter()
        .filter(|c| c.a == raft || c.b == raft)
        .copied()
        .collect()
}

/// **A lei, no nível em que ela sempre foi.** Duas jangadas de mesma silhueta e mesma
/// massa, no mesmo chão: a de DUAS peças relata **um** toque, como a de uma.
///
/// Mutação (não fundir por par de corpos) ⇒ a composta relata **2**, e o overlay
/// desenha duas cruzes onde houve um toque.
#[test]
fn a_compound_body_touching_the_floor_is_one_contact_like_its_one_piece_control() {
    let (control, _s1, r1) = settled(false);
    let (compound, _s2, r2) = settled(true);
    assert_eq!(mine(&control, r1).len(), 1, "o CONTROLE tem de relatar um");
    assert_eq!(
        mine(&compound, r2).len(),
        1,
        "a composta relatou uma entrada por FORMA -- fato sobre como o artista a \
         decompos, nao sobre a cena"
    );
}

/// **A CARGA é a mesma nas duas** — a soma sobre as formas do par, não a de uma delas.
///
/// O braço da cruz do overlay significa carga, então uma composta com metade da carga
/// desenha metade da marca para o mesmo peso.
///
/// Mutação (relatar por par de collider) ⇒ **0,030677** contra os 0,061313 do controle.
#[test]
fn the_load_a_compound_pair_carries_is_the_sum_over_its_shapes() {
    let (control, _s1, r1) = settled(false);
    let (compound, _s2, r2) = settled(true);
    let (a, b) = (
        mine(&control, r1)[0].impulse,
        mine(&compound, r2)[0].impulse,
    );
    assert!(
        (a - b).abs() < 0.002,
        "controle {a:.6} contra composta {b:.6}"
    );
}

/// **O IMPACTO é o pico da carga SOMADA, nunca o maior pico de uma forma sozinha.**
///
/// Este é o número que um som de batida dimensiona (W-ImpactForce), e é a metade que
/// vive dentro do laço de sub-passos — fundir só na leitura deixaria o `impact` pela
/// metade com o `impulse` já certo.
///
/// Mutação (`max` sobre as formas em vez da soma) ⇒ a composta lê ~metade.
#[test]
fn the_impact_of_a_compound_hit_is_the_peak_of_the_summed_load() {
    let (control, _s1, r1) = run(false, 2.5, 200);
    let (compound, _s2, r2) = run(true, 2.5, 200);
    let (a, b) = (mine(&control, r1)[0].impact, mine(&compound, r2)[0].impact);
    assert!(
        b > a * 0.8,
        "o impacto da composta ({b:.6}) tem de ser o do controle ({a:.6}), \
         nao o da forma mais atingida"
    );
}

/// **`contact_count` conta OBJETOS tocados, não formas.**
///
/// A porta que um consumidor pergunta (*"em quantas coisas estou encostado?"*), e a
/// que o overlay usa para decidir o que desenhar.
#[test]
fn the_contact_count_of_a_compound_body_counts_objects_not_shapes() {
    let (control, _s1, r1) = settled(false);
    let (compound, _s2, r2) = settled(true);
    assert_eq!(control.contact_count(r1), 1);
    assert_eq!(compound.contact_count(r2), 1, "contou as formas");
}

/// **A entrada fundida traz o ponto MAIS PROFUNDO das formas do par**, que é a
/// extensão literal do que um par de collider já responde com `find_deepest_contact`.
///
/// ⚠️ **A fixture é AUTORADA, e a primeira não era.** A tentativa óbvia — largar a
/// composta numa rampa, esperando que ela tombe e uma peça afunde mais — foi medida e
/// **não continha o fenômeno**: a jangada escorrega e descansa numa ponta só
/// (`ponto x = 1,8632`, **um** par de collider ativo), então a fusão recebe um
/// elemento e o desempate nunca roda. O gate passava com o desempate **desligado**.
///
/// Aqui as duas profundidades são POSTAS: gravidade zero e a jangada colocada dentro
/// do chão, com a peça 3 mm mais funda que a forma do corpo. As duas penetram, e qual
/// delas o ponto nomeia deixa de ser acidente da física para virar a pergunta do gate.
#[test]
fn the_merged_point_is_the_deepest_of_the_pairs_shapes() {
    let mut sim = SimWorld::new();
    ground(&mut sim);
    // Topo do chão em `y = 0`. A forma do corpo penetra 5 mm; a peça, 8 mm.
    let body = sim
        .world_mut()
        .spawn((
            Name::new("Raft"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            ph2d_physics_ecs::GravityScale(0.0),
            cuboid(HALF_X, HALF_Y),
            Transform::from_translation(Vec2::new(0.0, HALF_Y - 0.005)),
        ))
        .id();
    sim.world_mut().spawn((
        Name::new("Raft Deck"),
        cuboid(HALF_X, HALF_Y),
        Transform::from_translation(Vec2::new(HALF_X * 2.0, -0.003)),
        ChildOf(body),
    ));
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 0);
    bridge.dispatch(&mut sim, true, 1);
    let c = mine(&bridge, body);
    assert_eq!(c.len(), 1, "um toque, e um ponto");
    // A peça vive em `x ∈ [0,6; 1,8]`; a forma do corpo em `[−0,6; 0,6]`.
    assert!(
        c[0].point[0] > 0.3,
        "o ponto fundido ({:.3}) ficou do lado do CORPO -- a peca esta' 3 mm mais funda",
        c[0].point[0]
    );
}

/// **CONTROLE de regressão: um corpo de UMA forma não mudou.**
///
/// A fusão é uma passada a mais sobre uma lista de um elemento; se ela mexer no
/// resultado do caso comum, a wave regrediu o mundo que já shipava.
#[test]
fn a_single_shape_body_reports_exactly_what_it_always_did() {
    let (bridge, _sim, raft) = settled(false);
    let c = mine(&bridge, raft);
    assert_eq!(c.len(), 1);
    assert!(c[0].impulse > 0.0 && c[0].impact >= c[0].impulse);
    assert!(
        (c[0].point[1] - -0.001).abs() < 0.01,
        "o ponto ficou na superficie do chao: {:.4}",
        c[0].point[1]
    );
}

/// **Um toque de composta é UM evento de início, não um por forma.**
///
/// O canal de transição (W-ContactEvents) já era chaveado por corpo via
/// `tick_contacts`, então este gate é o que impede que uma "correção" futura o
/// re-chaveie por collider junto com o resto.
#[test]
fn a_compound_landing_fires_one_began_not_one_per_shape() {
    let mut sim = SimWorld::new();
    ground(&mut sim);
    let raft = raft(&mut sim, true, 1.2);
    let mut bridge = PhysicsBridge::new();
    let mut began = 0;
    for t in 0..=120 {
        bridge.dispatch(&mut sim, true, t);
        began += bridge
            .contact_events()
            .iter()
            .filter(|e| {
                matches!(e.phase, ph2d_physics_ecs::ContactPhase::Began)
                    && (e.a == raft || e.b == raft)
            })
            .count();
    }
    assert_eq!(began, 1, "um pouso, um evento -- saiu {began}");
}
