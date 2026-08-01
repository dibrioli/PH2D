//! **O que o Inspector diz sobre uma PEÇA** (W-PartFace).
//!
//! A W-Compound deu ao artista o gesto de CRIAR uma peça, o contorno para vê-la
//! e a ponte para simulá-la. O que ela não deu foi a volta: selecionar a peça e
//! **editá-la**. Medido antes da wave, com uma peça autorada como barra
//! `0,17 × 0,91`, offset `[0,13, −0,07]`, densidade `3,5`, atrito `0,9`, quique
//! `0,4` e camada `2`, o §11 respondia:
//!
//! ```text
//!   has_body ............ false              → face VAZIA, texto "Not simulated"
//!   shape_tag/half ...... 1 / [0.50, 0.50]   ← as SEMENTES, não a barra
//!   offset .............. [0.0, 0.0]
//!   density ............. 1.00
//!   friction/restitution  0.50 / 0.00
//!   layer ............... 0
//! ```
//!
//! Ou seja: **nada do que o artista autorou**, sob um texto que afirma o oposto
//! da verdade (uma peça É simulada, como forma do corpo ancestral).

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::Sprite;

use super::inspector_physics::build_physics_info;

/// Um "L": braço (corpo) e perna (peça) pendurada nele. A perna carrega um
/// collider **autorado** — uma barra fina e deslocada, nada parecido com o
/// default — para que qualquer coisa que a reponha apareça nos números.
fn ell() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let arm = sim
        .world_mut()
        .spawn((
            Name::new("Arm"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    let leg = sim
        .world_mut()
        .spawn((
            Name::new("Leg"),
            Sprite::atlas(0, [0.4, 2.0], [1.0, 1.0, 1.0, 1.0]),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.17,
                    half_y: 0.91,
                },
                offset: [0.13, -0.07],
                density: 3.5,
                friction: 0.9,
                restitution: 0.4,
                layer: 2,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.8, -1.0)),
            ChildOf(arm),
        ))
        .id();
    (sim, arm, leg)
}

/// O snapshot que o painel leria, com `part_count` resolvido como a shell o
/// resolve: contando os candidatos (collider sem corpo) cujo dono é `e`.
fn info(sim: &mut SimWorld, e: Entity) -> ph2d_editor::InspectorPhysicsInfo {
    let mut q = sim.world_mut().query_filtered::<Entity, (
        bevy_ecs::query::With<Collider>,
        bevy_ecs::query::Without<RigidBody>,
    )>();
    let candidates: Vec<Entity> = q.iter(sim.world()).collect();
    let parts =
        u8::try_from(ph2d_physics_ecs::count_parts(sim.world(), e, candidates)).unwrap_or(u8::MAX);
    build_physics_info(
        sim.world(),
        e.to_bits(),
        0,
        0,
        parts,
        false,
        0,
        (0.0, 5.0),
        0,
    )
    .expect("info")
}

/// **A sonda.** `cargo test -p ph2d-host-desktop measure_what_the_inspector_says_about_a_part
/// -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_inspector_says_about_a_part() {
    let (mut sim, _arm, leg) = ell();
    let i = info(&mut sim, leg);
    println!("\n=== o §11 sobre uma PEÇA autorada (barra 0,17 x 0,91, offset [0,13, -0,07]) ===");
    println!("  has_body ............ {}", i.has_body);
    println!("  has_collider ........ {}", i.has_collider);
    println!("  part_owner .......... {:?}", i.part_owner);
    println!(
        "  shape_tag/half ...... {} / [{:.2}, {:.2}]",
        i.shape_tag, i.half_x, i.half_y
    );
    println!("  offset .............. {:?}", i.offset);
    println!("  density ............. {:.2}", i.density);
    println!(
        "  friction/restitution  {:.2} / {:.2}",
        i.friction, i.restitution
    );
    println!("  layer ............... {}", i.layer);
    println!();
}

/// **A entrega da wave:** a peça mostra a forma que o artista autorou, e não as
/// sementes da face vazia.
///
/// Cada `assert` aqui nasceu VERMELHO com o número da doc do módulo — este gate
/// É o repro do defeito, virado do avesso.
#[test]
fn the_part_face_shows_the_authored_collider() {
    let (mut sim, _arm, leg) = ell();
    let i = info(&mut sim, leg);
    assert!(!i.has_body, "uma peça não tem corpo próprio, por definição");
    assert!(
        i.has_collider,
        "…mas TEM collider, e é isso que a manda para a face de peça em vez da vazia"
    );
    assert_eq!((i.half_x, i.half_y), (0.17, 0.91), "a forma autorada");
    assert_eq!(i.offset, [0.13, -0.07], "o offset autorado");
    assert!((i.density - 3.5).abs() < 1e-6, "a densidade autorada");
    assert!((i.friction - 0.9).abs() < 1e-6, "o atrito autorado");
    assert!((i.restitution - 0.4).abs() < 1e-6, "o quique autorado");
    assert_eq!(i.layer, 2, "a camada autorada");
}

/// **Uma peça NOMEIA o dono; um corpo não é peça de ninguém.**
///
/// O cabeçalho da face diz *"Shape of Arm"*, e esse nome é a única coisa que o
/// artista não consegue inferir da tela — um collider é invisível e a hierarquia
/// pode ter um grupo no meio.
#[test]
fn a_part_names_its_owner_and_a_body_names_nobody() {
    let (mut sim, arm, leg) = ell();
    assert_eq!(info(&mut sim, leg).part_owner, "Arm");
    assert_eq!(
        info(&mut sim, arm).part_owner,
        "",
        "um corpo não vira peça de ninguém"
    );
}

/// **Um corpo diz quantas peças tem** — o outro lado da invisibilidade.
///
/// Com o contorno desligado, nada na tela nem no §11 distinguia um corpo
/// composto de um de forma única.
#[test]
fn a_body_reports_how_many_parts_hang_from_it() {
    let (mut sim, arm, leg) = ell();
    assert_eq!(info(&mut sim, arm).part_count, 1);
    // Uma segunda perna, e — a metade que importa — um GRUPO no meio, que tem de
    // ser transparente: pôr as formas de uma peça dentro de uma pasta não pode
    // desligá-las do corpo.
    let group = sim
        .world_mut()
        .spawn((
            Name::new("Shapes"),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            ChildOf(arm),
        ))
        .id();
    sim.world_mut().spawn((
        Name::new("Leg 2"),
        Collider::default(),
        Transform::from_translation(Vec2::new(-0.8, -1.0)),
        ChildOf(group),
    ));
    assert_eq!(
        info(&mut sim, arm).part_count,
        2,
        "o grupo no meio deixou a peça invisível para a contagem"
    );
    // ⚠️ **Um SEGUNDO corpo com peça própria**, e a fixture não é decorativa: com
    // um corpo só, *"as peças do braço"* e *"todas as peças da cena"* são o MESMO
    // número — uma contagem que ignorasse o dono passaria. Medido: a mutação
    // (`owner_body(...).is_some()` no lugar de `== Some(body)`) SOBREVIVEU até
    // esta metade existir.
    let other = sim
        .world_mut()
        .spawn((
            Name::new("Other"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider::default(),
            Transform::from_translation(Vec2::new(9.0, 5.0)),
        ))
        .id();
    for i in 0..3 {
        sim.world_mut().spawn((
            Name::new(format!("Other Part {i}")),
            Collider::default(),
            Transform::from_translation(Vec2::new(0.2, -0.5)),
            ChildOf(other),
        ));
    }
    assert_eq!(
        info(&mut sim, arm).part_count,
        2,
        "as peças do OUTRO corpo entraram na conta do braço"
    );
    assert_eq!(info(&mut sim, other).part_count, 3);
    // E uma peça não hospeda peças: o walk sobe até o CORPO, então o dono de
    // todas elas é o braço.
    assert_eq!(info(&mut sim, leg).part_count, 0);
}

/// **O apply de `AddShape` é INCONDICIONAL, e isso é de propósito.**
///
/// Ele escreve a caixa do sprite — o que a face vazia quer. A recusa de
/// re-oferecê-lo a algo que JÁ tem forma vive no painel, com todo irmão do §11
/// (`event_physics::click_edit`), porque é lá que a lei *dim não é recusa* é
/// aplicada e é lá que o gate de seam consegue clicar de verdade.
///
/// Este gate existe para PINAR o preço daquele clique — se um dia ele deixar de
/// resetar, a recusa do painel vira cerimônia e alguém a removerá.
#[test]
fn the_add_shape_apply_still_overwrites_and_that_is_why_the_panel_refuses_it() {
    let (mut sim, _arm, leg) = ell();
    super::inspector_physics_tests::apply(&mut sim, leg, ph2d_editor::PhysicsFieldEdit::AddShape);
    let col = sim.world().get::<Collider>(leg).copied().expect("collider");
    let ColliderShape::Cuboid { half_x, half_y } = col.shape else {
        panic!("a forma virou outra coisa");
    };
    // O sprite mede 0,4 x 2,0 ⇒ a caixa dele é 0,2 x 1,0, e é ela que substitui
    // a barra autorada de 0,17 x 0,91.
    assert!(
        (half_x - 0.2).abs() < 1e-6 && (half_y - 1.0).abs() < 1e-6,
        "a forma autorada sobreviveu ({half_x}, {half_y}) — a medição mudou"
    );
    assert_eq!(col.offset, [0.0, 0.0], "o offset autorado foi apagado");
    assert_eq!(col.density, 1.0, "a densidade autorada foi apagada");
    assert_eq!(col.layer, 0, "a camada autorada foi apagada");
}

/// **Um `Remove` numa peça tira a forma e deixa o resto em paz.**
///
/// `queue_remove` de um componente ausente é no-op, então o MESMO edit que
/// remove um corpo serve para *Remove Shape* — uma porta, uma operação, dois
/// rótulos que descrevem a consequência em cada face.
#[test]
fn removing_a_part_leaves_a_plain_drawing() {
    let (mut sim, arm, leg) = ell();
    super::inspector_physics_tests::apply(&mut sim, leg, ph2d_editor::PhysicsFieldEdit::Remove);
    assert!(
        sim.world().get::<Collider>(leg).is_none(),
        "a forma continua lá — a peça era porta de mão única"
    );
    assert!(
        sim.world().get::<Sprite>(leg).is_some(),
        "o desenho tem de sobreviver: remover a FORMA não é apagar o objeto"
    );
    assert!(
        sim.world().get::<Collider>(arm).is_some(),
        "o dono não pode perder a própria forma"
    );
}

/// **O seed do `Mass: Auto → Manual` conhece as PEÇAS** (W-PartMass).
///
/// ⚠️ O comentário daquele gesto diz, com todas as letras, que o seed existe
/// *"para a massa não saltar quando o toggle vira"* — e num corpo composto ele
/// fazia exatamente isso: lia a forma PRÓPRIA e ignorava o resto. Medido numa
/// jangada de duas metades iguais, **`0,600` semeado contra `1,200` reais**.
/// Metade, em silêncio, no gesto cuja razão de existir é não mexer no número.
///
/// ⚠️ **O oráculo é o DOBRO do que o gate irmão mede num corpo simples**, e não
/// uma segunda conta de área: as duas metades são idênticas, então a resposta
/// certa é `2 × 0,600`. Re-derivar a área aqui daria uma expectativa que
/// concordaria com o bug se ele voltasse pela mesma porta.
#[test]
fn the_mass_seed_of_a_compound_body_counts_its_parts() {
    use ph2d_ecs::ChildOf;
    use ph2d_physics_ecs::MassOverride;

    let mut sim = SimWorld::new();
    let body = sim
        .world_mut()
        .spawn((
            Name::new("Raft"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.6,
                    half_y: 0.25,
                },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    sim.world_mut().spawn((
        Name::new("Raft Deck"),
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.6,
                half_y: 0.25,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(1.2, 0.0)),
        ChildOf(body),
    ));

    // ⚠️ **Um SEGUNDO corpo, com peças PRÓPRIAS, e ele é o que dá dentes ao
    // gate.** Sem ele *"as peças deste corpo"* e *"todas as peças da cena"* são
    // o mesmo número, e a mutação que ignora o dono passa — foi exatamente o que
    // aconteceu, pela terceira vez nesta linha (o `count_parts` da W-PartFace
    // teve o mesmo buraco).
    let other = sim
        .world_mut()
        .spawn((
            Name::new("Other"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 2.0,
                    half_y: 2.0,
                },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(20.0, 0.0)),
        ))
        .id();
    for i in 0..3 {
        sim.world_mut().spawn((
            Name::new(format!("Other Part {i}")),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 2.0,
                    half_y: 2.0,
                },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 4.0 * (i + 1) as f32)),
            ChildOf(other),
        ));
    }

    super::inspector_physics_tests::apply(
        &mut sim,
        body,
        ph2d_editor::PhysicsFieldEdit::MassMode(true),
    );
    let seeded = sim
        .world()
        .get::<MassOverride>(body)
        .map(|m| m.0)
        .expect("o toggle Manual tem de semear um override");

    // Uma metade pesa `4 · 0,6 · 0,25 · 1,0 = 0,600` (o gate irmão a mede pela
    // porta do produto); duas metades idênticas pesam o dobro.
    let half = 0.6_f32;
    assert!(
        (seeded - half * 2.0).abs() < 1e-3,
        "seed {seeded:.4} contra os {:.4} das duas metades -- a forma propria \
         sozinha da' {half:.4}, que e' o bug",
        half * 2.0
    );
}

/// **E o seed de um corpo SIMPLES não mudou** — o controle, e o que impede a
/// cura de virar uma regressão para todo corpo que não é composto.
#[test]
fn the_mass_seed_of_a_plain_body_is_its_own_shape() {
    use ph2d_physics_ecs::MassOverride;

    let mut sim = SimWorld::new();
    let body = sim
        .world_mut()
        .spawn((
            Name::new("Box"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.6,
                    half_y: 0.25,
                },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    super::inspector_physics_tests::apply(
        &mut sim,
        body,
        ph2d_editor::PhysicsFieldEdit::MassMode(true),
    );
    let seeded = sim
        .world()
        .get::<MassOverride>(body)
        .map(|m| m.0)
        .expect("override");
    assert!(
        (seeded - 0.6).abs() < 1e-3,
        "um corpo de UMA forma passou a semear {seeded:.4} em vez de 0,600"
    );
}
