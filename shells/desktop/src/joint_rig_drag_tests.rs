//! **O grupo carrega o rig** — os gates da porta única de semeadura (W-JG).
//!
//! Tudo aqui roda **headless** sobre um `SimWorld` de verdade: a porta é uma
//! função sobre o mundo AUTORADO, sem despacho, sem janela e sem solver — o que
//! a torna o lugar onde a lei se prova. O que ela NÃO alcança (que o Down de
//! fato a chama, com as três condições) é o arch-gate irmão em
//! `tests/the_drag_carries_the_jointed_rig.rs`.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};

use super::seed_group_drag_starts;

fn body(sim: &mut SimWorld, name: &str, kind: BodyKind, x: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ))
        .id()
}

fn pin(sim: &mut SimWorld, name: &str, a: &str, b: &str) {
    sim.world_mut().spawn((
        Name::new(name),
        PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind: JointKind::Pin,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::ZERO),
    ));
}

/// Os bits que a semeadura produziu, ordenados — o oráculo é o CONJUNTO, não a
/// ordem em que `jointed_group` os devolveu.
fn seeded(out: &[crate::app_state::GroupDragSnapshot]) -> Vec<u64> {
    let mut v: Vec<u64> = out.iter().map(|s| s.entity_bits).collect();
    v.sort_unstable();
    v
}

fn sorted(mut v: Vec<u64>) -> Vec<u64> {
    v.sort_unstable();
    v
}

/// Gancho ESTÁTICO + três elos dinâmicos em fila, pinados vizinho a vizinho.
fn chain() -> (SimWorld, Entity, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let hook = body(&mut sim, "Hook", BodyKind::Static, 0.0);
    let l1 = body(&mut sim, "L1", BodyKind::Dynamic, 1.0);
    let l2 = body(&mut sim, "L2", BodyKind::Dynamic, 2.0);
    let l3 = body(&mut sim, "L3", BodyKind::Dynamic, 3.0);
    pin(&mut sim, "J0", "Hook", "L1");
    pin(&mut sim, "J1", "L1", "L2");
    pin(&mut sim, "J2", "L2", "L3");
    (sim, hook, l1, l2, l3)
}

/// **Pegar UM elo carrega a corrente inteira.** A entrega da wave.
///
/// Mutação-testado: passar `&[]` como seed do `jointed_group` (ou pular a
/// expansão inteira) devolve lista vazia — o elo andaria sozinho e o joint
/// nasceria esticado, que é o defeito que a W-AnchorFollow abriu.
#[test]
fn dragging_one_link_carries_the_whole_chain() {
    let (mut sim, hook, l1, l2, l3) = chain();
    let mut out = Vec::new();
    seed_group_drag_starts(&mut out, &mut sim, l2.to_bits(), &[l2.to_bits()], true);
    assert_eq!(
        seeded(&out),
        sorted(vec![l1.to_bits(), l3.to_bits()]),
        "a corrente inteira menos o primário; o gancho ESTÁTICO fica"
    );
    assert!(
        !out.iter().any(|s| s.entity_bits == hook.to_bits()),
        "um gancho estático é uma parede: ele não anda com o rig"
    );
}

/// **Com o modificador, anda só o corpo que se pegou.** A outra metade do
/// pedido do plano — e o gate que prova que `carry_rig` é o interruptor, e não
/// decoração.
#[test]
fn the_modifier_moves_only_the_body_you_grabbed() {
    let (mut sim, _hook, _l1, l2, _l3) = chain();
    let mut out = Vec::new();
    seed_group_drag_starts(&mut out, &mut sim, l2.to_bits(), &[l2.to_bits()], false);
    assert!(
        out.is_empty(),
        "sem o rig, uma seleção de um só corpo não semeia extra nenhum: {:?}",
        seeded(&out)
    );
}

/// **Dois pêndulos no MESMO gancho estático ficam independentes.**
///
/// A lei do `jointed_group` pinada NESTA camada: o gancho é alcançado pela
/// aresta e nunca atravessado, senão arrastar um pêndulo arrastaria o outro —
/// e, num chão estático compartilhado, a cena inteira.
#[test]
fn two_pendulums_on_the_same_hook_stay_independent() {
    let mut sim = SimWorld::new();
    let _hook = body(&mut sim, "Hook", BodyKind::Static, 0.0);
    let a = body(&mut sim, "BobA", BodyKind::Dynamic, -1.0);
    let b = body(&mut sim, "BobB", BodyKind::Dynamic, 1.0);
    pin(&mut sim, "JA", "Hook", "BobA");
    pin(&mut sim, "JB", "Hook", "BobB");
    let mut out = Vec::new();
    seed_group_drag_starts(&mut out, &mut sim, a.to_bits(), &[a.to_bits()], true);
    assert!(
        out.is_empty(),
        "o gancho é uma parede entre os dois pêndulos, não um fio: {:?} (B = {})",
        seeded(&out),
        b.to_bits()
    );
}

/// **A multi-seleção simples continua semeada exatamente como antes.**
///
/// A regressão que a extração de porta poderia ter comido: dois sprites sem
/// física nenhuma, selecionados juntos, arrastados — o extra tem de aparecer
/// com a pose de início dele.
#[test]
fn a_plain_multi_selection_is_still_seeded_as_before() {
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(1.0, 2.0)),))
        .id();
    let b = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(3.0, 4.0)),))
        .id();
    let mut out = Vec::new();
    seed_group_drag_starts(
        &mut out,
        &mut sim,
        a.to_bits(),
        &[a.to_bits(), b.to_bits()],
        false,
    );
    assert_eq!(seeded(&out), vec![b.to_bits()]);
    assert_eq!(out[0].start_transform.translation, [3.0, 4.0]);
}

/// **Um corpo sem joint nenhum não carrega nada** — e o rig ligado não é um
/// caminho diferente para a cena comum.
#[test]
fn a_body_with_no_joints_carries_nothing() {
    let mut sim = SimWorld::new();
    let solo = body(&mut sim, "Solo", BodyKind::Dynamic, 0.0);
    let _other = body(&mut sim, "Other", BodyKind::Dynamic, 5.0);
    let mut out = Vec::new();
    seed_group_drag_starts(&mut out, &mut sim, solo.to_bits(), &[solo.to_bits()], true);
    assert!(out.is_empty(), "sem aresta não há rig: {:?}", seeded(&out));
}

/// **O rig não acrescenta um corpo que o parentesco já carrega** — nas DUAS
/// direções.
///
/// O translate de grupo soma o delta de MUNDO ao `Transform` **local** de cada
/// extra, então um descendente de outro membro andaria duas vezes: o rig
/// explodiria em vez de andar. Mutação-testada nas duas metades — perguntar só
/// para cima deixa passar o caso em que o corpo arrastado é o de baixo.
#[test]
fn the_rig_does_not_add_a_body_that_parenthood_already_carries() {
    let build = || {
        let mut sim = SimWorld::new();
        let p = body(&mut sim, "Parent", BodyKind::Dynamic, 0.0);
        let c = body(&mut sim, "Child", BodyKind::Dynamic, 1.0);
        sim.world_mut().entity_mut(c).insert(ChildOf(p));
        pin(&mut sim, "J", "Parent", "Child");
        (sim, p, c)
    };
    // Arrastando o PAI: o filho já vem junto por herança.
    let (mut sim, p, c) = build();
    let mut out = Vec::new();
    seed_group_drag_starts(&mut out, &mut sim, p.to_bits(), &[p.to_bits()], true);
    assert!(
        out.is_empty(),
        "o filho anda com o pai por herança; somá-lo o moveria em dobro: {:?} (filho = {})",
        seeded(&out),
        c.to_bits()
    );
    // E arrastando o FILHO: mover o pai empurraria o filho de novo.
    let (mut sim, p, c) = build();
    let mut out = Vec::new();
    seed_group_drag_starts(&mut out, &mut sim, c.to_bits(), &[c.to_bits()], true);
    assert!(
        out.is_empty(),
        "mover o pai carregaria o primário uma segunda vez: {:?} (pai = {})",
        seeded(&out),
        p.to_bits()
    );
}

/// **A regra de parentesco é do que o RIG acrescenta, não da seleção.**
///
/// Um pai e um filho escolhidos à mão continuam ambos na lista — é o
/// comportamento anterior à wave, e mexer nele seria mudar a semântica de
/// seleção do editor por baixo de outra wave.
#[test]
fn the_parenthood_rule_does_not_touch_an_explicit_multi_selection() {
    let mut sim = SimWorld::new();
    let p = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(0.0, 0.0)),))
        .id();
    let c = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(1.0, 0.0)),))
        .id();
    sim.world_mut().entity_mut(c).insert(ChildOf(p));
    let mut out = Vec::new();
    seed_group_drag_starts(
        &mut out,
        &mut sim,
        p.to_bits(),
        &[p.to_bits(), c.to_bits()],
        true,
    );
    assert_eq!(seeded(&out), vec![c.to_bits()]);
}

/// **Uma seleção de vários elos não duplica ninguém**, e o rig é o do conjunto:
/// pegar L1 com L3 já selecionado semeia L3 uma vez só e traz L2 junto.
#[test]
fn a_multi_selection_inside_one_rig_seeds_each_body_once() {
    let (mut sim, _hook, l1, l2, l3) = chain();
    let mut out = Vec::new();
    seed_group_drag_starts(
        &mut out,
        &mut sim,
        l1.to_bits(),
        &[l1.to_bits(), l3.to_bits()],
        true,
    );
    assert_eq!(seeded(&out), sorted(vec![l2.to_bits(), l3.to_bits()]));
}

/// **A SONDA da cena 51** — os números que a mensagem do smoke afirma, medidos
/// sobre as MESMAS armações que o artista abre (`physics_smoke_joint_rig::
/// spawn_rigs`), e não sobre umas parecidas.
///
/// `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_smoke_51 -- \
///  --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime os números da cena 51, não afirma nada"]
fn probe_smoke_51() {
    let mut sim = SimWorld::new();
    crate::physics_smoke_joint_rig::spawn_rigs(sim.world_mut());
    let by_name = |sim: &mut SimWorld, want: &str| -> Entity {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == want)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("{want} não está na cena"))
    };
    for grabbed in ["Chain L2", "Twin Left", "Free Left"] {
        let e = by_name(&mut sim, grabbed);
        let mut out = Vec::new();
        seed_group_drag_starts(&mut out, &mut sim, e.to_bits(), &[e.to_bits()], true);
        let mut names: Vec<String> = out
            .iter()
            .map(|s| {
                sim.world()
                    .get::<Name>(Entity::from_bits(s.entity_bits))
                    .map_or_else(|| "?".to_string(), |n| n.as_str().to_string())
            })
            .collect();
        names.sort();
        println!(
            "pegar '{grabbed}' leva {} corpo(s) a mais: {names:?}",
            names.len()
        );
    }
}

/// **O rig é o da SELEÇÃO INTEIRA, não só o do corpo agarrado.**
///
/// Duas correntes independentes, um elo de cada selecionado: arrastar traz as
/// duas inteiras, porque cada corpo selecionado vai andar e cada um quebraria o
/// PRÓPRIO rig se o dele ficasse para trás.
///
/// ⚠️ Este gate existe porque `dragging_one_link_carries_the_whole_chain`
/// **não** o cobre: com uma corrente só, semear a seleção inteira e semear só o
/// primário dão exatamente a mesma resposta (o extra já está no componente
/// conexo do primário) — a mutação que descarta os extras da semente passava
/// por toda a suíte antes disto.
#[test]
fn the_rig_is_the_whole_selections_rig_not_just_the_grabbed_bodys() {
    let mut sim = SimWorld::new();
    let a1 = body(&mut sim, "A1", BodyKind::Dynamic, 0.0);
    let a2 = body(&mut sim, "A2", BodyKind::Dynamic, 1.0);
    let b1 = body(&mut sim, "B1", BodyKind::Dynamic, 10.0);
    let b2 = body(&mut sim, "B2", BodyKind::Dynamic, 11.0);
    pin(&mut sim, "JA", "A1", "A2");
    pin(&mut sim, "JB", "B1", "B2");
    let mut out = Vec::new();
    seed_group_drag_starts(
        &mut out,
        &mut sim,
        a1.to_bits(),
        &[a1.to_bits(), b1.to_bits()],
        true,
    );
    assert_eq!(
        seeded(&out),
        sorted(vec![a2.to_bits(), b1.to_bits(), b2.to_bits()]),
        "o parceiro do elo B selecionado tem de vir junto"
    );
}
