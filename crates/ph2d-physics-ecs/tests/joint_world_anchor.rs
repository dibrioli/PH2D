//! **Um joint cujo lado B é o MUNDO** (W-JointWorld, plano 02 §9).
//!
//! Sem isto, prender algo ao cenário obriga o artista a **inventar um corpo
//! estático** só para servir de âncora — um objeto a mais para nomear, achar na
//! Hierarquia e mover por acidente. Estes gates pinam as quatro metades:
//!
//! 1. o pino **SEGURA** (e o controle sem marcador CAI);
//! 2. um **scrub para um tique do MEIO** continua segurando — a lição do Weston,
//!    e ela é **invisível num Reset**;
//! 3. mover o `Transform` do joint **MOVE** o pino;
//! 4. o `sync_joint_pivots` **não reescreve** esse `Transform` — senão arrastar
//!    o dot seria desfeito no frame seguinte, e a feature seria inautorável.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointWorldAnchor, PhysicsBridge, PhysicsJoint,
    RigidBody,
};

/// Um único corpo dinâmico, pendurado num PONTO — sem nenhum segundo corpo na
/// cena. É a fixture inteira, e é o ponto da wave: se ela precisasse de um corpo
/// estático para funcionar, não haveria feature nenhuma.
///
/// A âncora fica 1 m acima do centro do corpo, então *"onde no corpo o pino
/// está"* é uma afirmação com dentes.
fn hanging(marked: bool) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Lamp"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    let joint = sim
        .world_mut()
        .spawn((
            Name::new("Wall Pin"),
            PhysicsJoint {
                body_a: stable_name_id("Lamp"),
                // ⚠️ **`body_b` fica ZERO de propósito** — é o mundo, e não há
                // nome a apontar. Este é exatamente o estado que sem o marcador
                // significa *meio-autorado*, e é por isso que o marcador existe
                // em vez de um overload deste campo.
                body_b: 0,
                kind: JointKind::Pin,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(0.0, 6.0)),
        ))
        .id();
    if marked {
        sim.world_mut().entity_mut(joint).insert(JointWorldAnchor);
    }
    (sim, joint)
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ticks: u64) {
    for t in 1..=ticks {
        bridge.dispatch(sim, true, t);
    }
}

/// **O pino SEGURA, e o controle CAI.**
///
/// As duas metades no mesmo gate porque uma sozinha não prova nada: *"o corpo
/// está a 5 m"* é verdade tanto para um pino que segura quanto para uma cena que
/// nunca simulou.
#[test]
fn a_body_pinned_to_the_world_hangs_and_an_unmarked_one_falls() {
    let (mut sim, _) = hanging(true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 120);
    let held = y_of(&mut sim, "Lamp");

    let (mut sim2, _) = hanging(false);
    let mut bridge2 = PhysicsBridge::new();
    run(&mut sim2, &mut bridge2, 120);
    let fell = y_of(&mut sim2, "Lamp");

    assert!(
        (held - 5.0).abs() < 0.05,
        "o pino de mundo tinha de segurar a lâmpada em y=5; ela está em {held:.4}"
    );
    assert!(
        fell < 3.0,
        "o CONTROLE (sem marcador) tinha de cair — `body_b == 0` é meio-autorado, \
         não um pino; ele parou em {fell:.4}"
    );
}

/// **A LIÇÃO DO WESTON: um scrub para um tique do MEIO continua segurando.**
///
/// ⚠️ Este gate **tem de scrubbar para o meio**, e não é preferência: o
/// `rebuild_from_rest` troca o `PhysicsWorld` inteiro — a âncora do mundo velho
/// morre com ele — e o replay roda no MESMO chamado. Num **Reset**
/// (`target == 0`) o replay dá **zero passos**, então o dispatch seguinte
/// reconstrói tudo e o defeito **não aparece**. Foi exatamente assim que a
/// tabela de polias sumiu e *um rewind replayava sem as cordas*.
/// ⚠️ **E "um tique do meio" NÃO BASTA — a primeira versão deste gate também
/// sobreviveu à mutação.** Um scrub para trás normalmente ACERTA o ring de
/// checkpoints (W1.5), e aí o `rebuild_from_rest` **nunca roda**: o gate media um
/// caminho que a mutação não toca.
///
/// O que produz o miss é o gesto ordinário que **limpa o ring** — uma edição de
/// parâmetro — seguida do scrub. É o par exato que um artista faz ao afinar uma
/// mola e voltar para rever o movimento.
#[test]
fn a_scrub_that_MISSES_the_ring_replays_with_the_world_pin_still_holding() {
    let (mut sim, joint) = hanging(true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 200);

    // Uma edição de parâmetro: ela re-descreve o joint e **limpa o ring**.
    if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(joint) {
        j.stiffness = 123.0;
    }
    bridge.dispatch(&mut sim, true, 200);

    // Agora o scrub para trás não tem checkpoint para semear: é o
    // `rebuild_from_rest` + replay de 63 passos que corre.
    bridge.dispatch(&mut sim, true, 63);
    let y = y_of(&mut sim, "Lamp");
    assert!(
        (y - 5.0).abs() < 0.05,
        "depois do scrub a lâmpada tinha de continuar pendurada em y=5; \
         ela está em {y:.4} — o replay correu SEM a âncora"
    );
}

/// **Mover o `Transform` do joint MOVE o pino.**
///
/// ⚠️ É o gate que o `desc` sozinho não pode dar: o `local_b` de um pino de mundo
/// é `[0, 0]` **onde quer que a âncora esteja**, então o descritor entregue ao
/// solver é IDÊNTICO antes e depois do arrasto. Quem detecta a mudança é a
/// comparação do PONTO (`JointRef::world_anchor`), e sem ela arrastar o dot não
/// moveria nada.
#[test]
fn moving_the_joints_transform_moves_the_pin() {
    let (mut sim, joint) = hanging(true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 60);
    let before = y_of(&mut sim, "Lamp");

    // O gesto do dot âmbar: escrever o `Transform` do joint.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(joint) {
        t.translation.y = 9.0;
    }
    run(&mut sim, &mut bridge, 120);
    let after = y_of(&mut sim, "Lamp");

    assert!(
        (before - 5.0).abs() < 0.05,
        "controle: antes do arrasto a lâmpada pende de y=6, logo fica em 5; está em {before:.4}"
    );
    assert!(
        (after - 8.0).abs() < 0.1,
        "a âncora subiu para y=9, então a lâmpada tinha de subir para ~8; está em {after:.4}"
    );
}

/// **O `sync_joint_pivots` NÃO reescreve o `Transform` de um pino de mundo.**
///
/// Num joint corpo↔corpo aquele `Transform` é DERIVADO (`bodyA · local_a`) e ser
/// reescrito é o correto. Num pino de mundo ele é a **FONTE** — quem segue a
/// âncora é o corpo — e reescrevê-lo desfaria o arrasto do dot no frame
/// seguinte, que é a diferença entre uma feature e uma alça que volta sozinha.
/// ⚠️ **A FIXTURE TEM DE CONTER O FENÔMENO, e a primeira versão deste gate não
/// continha** — ela media o `Transform` logo depois do seed, quando o valor
/// DERIVADO (`bodyA · local_a`) é **exatamente igual** ao autorado. Escrever ou
/// não escrever produzia o mesmo número, então o gate passava com a proteção
/// removida: verde sobre o defeito que ele alega pegar.
///
/// O fenômeno só existe depois de o artista **MOVER a âncora**: aí o autorado é
/// o ponto novo, o derivado ainda é o velho (o `local_a` não foi re-semeado), e
/// os dois discordam. É o instante exato do arrasto do dot.
#[test]
fn the_pivot_sync_leaves_a_world_pins_transform_alone_after_a_drag() {
    let (mut sim, joint) = hanging(true);
    let mut bridge = PhysicsBridge::new();
    // Pausado: é quando o sync roda (rest-only).
    for t in 0..4 {
        bridge.dispatch(&mut sim, false, t);
    }
    // O gesto: arrastar o dot âmbar para um ponto NOVO.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(joint) {
        t.translation.y = 9.0;
    }
    // E o frame seguinte, ainda pausado — é aqui que o sync reescreveria.
    for t in 4..8 {
        bridge.dispatch(&mut sim, false, t);
    }
    let authored = sim
        .world()
        .get::<Transform>(joint)
        .map(|t| [t.translation.x, t.translation.y])
        .expect("joint vivo");
    assert_eq!(
        authored,
        [0.0, 9.0],
        "o arrasto foi DESFEITO: o sync reescreveu o ponto do artista a partir de \
         `bodyA · local_a`, e a alça volta sozinha"
    );
}

/// **A âncora não VAZA.** Uma edição de parâmetro passa por remove+spawn, e a
/// âncora nasce no spawn — se ela não morresse no remove, cada nudge de slider
/// deixaria um corpo fixo invisível na arena para sempre.
#[test]
fn re_describing_a_world_pin_does_not_leak_anchor_bodies() {
    let (mut sim, joint) = hanging(true);
    let mut bridge = PhysicsBridge::new();
    // ⚠️ **Relógio MONOTÔNICO, e é a quarta lição de fixture deste arquivo:** o
    // helper `run` recomeça o tique em 1, então chamá-lo em laço é um salto de
    // relógio **para trás** a cada volta — o `rebuild_from_rest` constrói um
    // mundo NOVO e leva junto toda âncora vazada. O gate media a limpeza que o
    // rewind faz, não a que o remove deveria fazer.
    let mut tick = 0u64;
    let mut step = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, n: u64| {
        for _ in 0..n {
            tick += 1;
            bridge.dispatch(sim, true, tick);
        }
    };
    step(&mut sim, &mut bridge, 10);
    let baseline = bridge.arena_body_count();

    // ⚠️ **A edição tem de RE-DESCREVER de verdade**, e a primeira versão deste
    // gate não redescrevia: ela mexia em `stiffness` num **Pin**, e o
    // `joint_desc` recusa todo parâmetro que o tipo ignora ⇒ o `desc` saía
    // idêntico, nenhum remove+spawn acontecia, e o gate media a ausência do
    // fenômeno. Mover a ÂNCORA re-descreve por construção — é o gesto do dot.
    for i in 0..10 {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(joint) {
            t.translation.y = 6.0 + i as f32 * 0.1;
        }
        step(&mut sim, &mut bridge, 2);
    }
    let after = bridge.arena_body_count();
    assert_eq!(
        baseline,
        after,
        "dez re-descrições deixaram {} corpos a mais na arena — a âncora está vazando",
        after as i64 - baseline as i64
    );
}

/// **O pino de mundo se DESENHA — e a ponta B senta no ponto do cenário.**
///
/// Medido em vez de suposto: a view sai de `JointRef`, e um pino de mundo TEM
/// um `JointRef` (a âncora é um corpo de verdade na arena), então a hipótese era
/// que o overlay já o desenhasse de graça. Ela se confirma — e este gate é o que
/// impede a próxima wave de "consertar" um desenho que já funciona.
///
/// ⚠️ O que ele afirma é a GEOMETRIA (`anchor_b` no ponto autorado), não que o
/// glifo seja distinto: hoje um pino de parede e um pino entre dois corpos leem
/// IGUAL na tela, e isso está nomeado como aberto no plano 02 §9.3.
#[test]
fn a_world_pin_produces_a_view_whose_b_end_sits_on_the_authored_point() {
    let (mut sim, _) = hanging(true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 30);

    let v = bridge
        .joint_views()
        .next()
        .expect("um pino de mundo tem de produzir view — sem ela o overlay não o desenha");
    assert!(
        (v.anchor_b[0] - 0.0).abs() < 1e-3 && (v.anchor_b[1] - 6.0).abs() < 1e-3,
        "a ponta B tinha de sentar no ponto autorado (0, 6); está em {:?}",
        v.anchor_b
    );
    assert_eq!(
        v.body_b, None,
        "não há corpo B a apontar — o fantasma do limite não tem silhueta a desenhar"
    );
}
