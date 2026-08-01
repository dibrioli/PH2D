//! O gesto de desenhar um pino de MUNDO no canvas (W-JointWorld).

use super::inspector_joint_world::create_world_pin_at;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointWorldAnchor, PhysicsBridge, PhysicsJoint,
    RigidBody,
};

fn one_body() -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Lamp"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    (sim, e)
}

/// **Soltar no vazio cria um pino de MUNDO, e ele SEGURA.**
///
/// O relato do smoke: *"não aceita desenhar a junta a partir do canvas vazio,
/// apenas de um objeto para outro"*. Este gate é esse gesto.
#[test]
fn releasing_on_empty_canvas_pins_the_body_to_the_world() {
    let (mut sim, body) = one_body();
    let joint = create_world_pin_at(
        &mut sim,
        body.to_bits(),
        JointKind::Pin,
        [0.0, 5.0],
        [0.0, 7.0],
    )
    .expect("o gesto tinha de criar o pino");
    assert!(
        sim.world().get::<JointWorldAnchor>(joint).is_some(),
        "sem o marcador este joint é meio-autorado, e o corpo cai"
    );
    let mut bridge = PhysicsBridge::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = {
        let mut q = sim.world_mut().query::<(&Name, &Transform)>();
        q.iter(sim.world())
            .find(|(n, _)| n.as_str() == "Lamp")
            .map(|(_, t)| t.translation.y)
            .expect("corpo vivo")
    };
    assert!(
        y > 4.0,
        "a lâmpada tinha de ficar pendurada; ela caiu para {y:.4}"
    );
}

/// **Um Pin nasce SEM TRANCO** — o pivô é onde o artista soltou, e o corpo não
/// é arrancado para lá.
///
/// ⚠️ É a razão de a política de âncora ser copiada do irmão de dois corpos em
/// vez de simplificada: num tipo que COMPARTILHA um ponto os dois lados são o
/// MESMO lugar, então ancorar A no ponto do *press* faria o solver puxar o corpo
/// até o ponto do *release* no primeiro passo.
#[test]
fn a_world_pin_does_not_yank_the_body_on_creation() {
    let (mut sim, body) = one_body();
    create_world_pin_at(
        &mut sim,
        body.to_bits(),
        JointKind::Pin,
        [0.1, 5.1],
        [0.0, 7.0],
    )
    .expect("pino");
    let mut bridge = PhysicsBridge::new();
    // Dois tiques só: o tranco, se houver, acontece no primeiro.
    for t in 1..=2 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p = {
        let mut q = sim.world_mut().query::<(&Name, &Transform)>();
        q.iter(sim.world())
            .find(|(n, _)| n.as_str() == "Lamp")
            .map(|(_, t)| [t.translation.x, t.translation.y])
            .expect("corpo vivo")
    };
    assert!(
        (p[0] - 0.0).abs() < 0.1 && (p[1] - 5.0).abs() < 0.1,
        "o corpo foi ARRANCADO para o ponto do release: está em {p:?}, e nasceu em [0, 5]"
    );
}

/// **A POLIA é recusada no GESTO**, não só no reconcile — senão o artista cria
/// um objeto que nasce dormente e não diz por quê.
#[test]
fn a_pulley_cannot_be_pinned_to_the_world() {
    let (mut sim, body) = one_body();
    assert!(
        create_world_pin_at(
            &mut sim,
            body.to_bits(),
            JointKind::Pulley,
            [0.0, 5.0],
            [0.0, 7.0]
        )
        .is_none(),
        "uma corda puxa as DUAS pontas — uma delas no cenário é outra máquina"
    );
    let mut q = sim.world_mut().query::<&PhysicsJoint>();
    assert_eq!(
        q.iter(sim.world()).count(),
        0,
        "a recusa não pode deixar um joint para trás"
    );
}

/// **O release no VAZIO chega à porta do pino de mundo** — arch-gate.
///
/// ⚠️ `joint_draw_release` exige `gfx` (janela + GPU), então **nenhum teste de
/// unidade o alcança**: a decisão de rotear o vazio mora lá dentro, e sem este
/// gate os três gates acima ficariam verdes sobre um gesto que continua
/// recusando — que é exatamente o defeito que o smoke reportou.
#[test]
fn the_canvas_release_on_empty_routes_to_the_world_pin() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/joint_draw.rs"))
        .expect("joint_draw.rs");
    assert!(
        src.contains("inspector_joint_world::create_world_pin_at"),
        "o release do canvas não chama a porta do pino de mundo — soltar no vazio \
         continua sendo uma recusa"
    );
    // E a recusa que ELE substituiu não pode voltar a existir.
    assert!(
        !src.contains("world pins are not a thing yet"),
        "a recusa antiga ainda está na tela, e ela agora é FALSA"
    );
    // ⚠️ Controle positivo: se este arquivo deixar de ser o do gesto, as duas
    // asserções acima viram vácuo — elas só sabem dizer o que NÃO está lá.
    assert!(
        src.contains("fn joint_draw_release"),
        "este gate está lendo o arquivo errado"
    );
}

/// **A ÂNCORA DE MUNDO SE MOVE** — o 2º relato de smoke (*"ainda não posso mover
/// a âncora colocada no mundo"*) — **pelo ANEL** (W-WorldPinLocal).
///
/// ⚠️ O mecanismo original era exato: o arrasto escrevia `local_a` — *onde no
/// CORPO o pino prende* — e o prego ficava onde estava, então arrastar não fazia
/// nada visível. Mas a cura de então deu ao lado **A** a segunda
/// responsabilidade, e com ela a porta passou a **ler um número e escrever
/// outro**: `joint_anchor_world(A)` devolve `world_from_local(corpo, local_a)`
/// sempre que o joint está `anchored`, e um pino de mundo está — medido, pedir
/// `[0,5; 2,0]` deixava a leitura em `[0,0; 2,0]` com o `Transform` já em 0,5, ou
/// seja **o dot não seguia o mouse**.
///
/// ⚠️ **Agora cada lado significa o que significa em todo joint:** o **dot (A)** é
/// onde no corpo A, o **anel (B)** é a outra ponta — que num pino de mundo é o
/// PREGO. Os dois lêem e escrevem o MESMO número, então os dois seguem o cursor,
/// e a pergunta que não tinha porta nenhuma (*onde no corpo isto prende?*) passa a
/// ter a mesma do resto do app.
///
/// ⚠️ **É MUDANÇA DE COMPORTAMENTO:** o gesto aprovado no smoke da `=65` era o
/// dot. Ele mudou de alça, e a infra de par coincidente é o que o torna
/// alcançável — *A pega o quadrado interno e B a banda de fora*.
///
/// ⚠️ **E o CONTROLE é a outra metade:** num joint corpo-a-corpo o MESMO gesto tem
/// de continuar escrevendo `local_a`. Sem ele este gate ficaria verde sobre uma
/// porta que passou a mover o pivô de TODO joint do app.
#[test]
fn dragging_the_ring_moves_a_world_pins_nail_and_the_dot_moves_where_it_hangs() {
    let (mut sim, body) = one_body();
    let joint = create_world_pin_at(
        &mut sim,
        body.to_bits(),
        JointKind::Pin,
        [0.0, 5.0],
        [0.0, 7.0],
    )
    .expect("pino");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    // Uma closure sobre `sim` prenderia o empréstimo pela função inteira; a
    // pergunta é feita na hora, ao lado de cada escrita.
    let follows = |got: Option<[f32; 2]>, want: [f32; 2]| {
        let g = got.expect("a alça não existe");
        (g[0] - want[0]).abs() < 1e-3 && (g[1] - want[1]).abs() < 1e-3
    };
    // Em repouso as duas alças COINCIDEM: é o que um pino satisfeito é, e o
    // quadrado interno / banda de fora é como o par continua sendo duas.
    assert_eq!(
        bridge.joint_anchor_world(&sim, joint, ph2d_physics_ecs::JointSide::A),
        bridge.joint_anchor_world(&sim, joint, ph2d_physics_ecs::JointSide::B)
    );

    // --- o ANEL: o prego anda, e onde no corpo prende NÃO muda ---
    let before = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert!(
        bridge.set_joint_anchor_world(&mut sim, joint, ph2d_physics_ecs::JointSide::B, [2.0, 9.0]),
        "a porta recusou o arrasto do anel"
    );
    let t = sim.world().get::<Transform>(joint).expect("joint");
    assert!(
        (t.translation.x - 2.0).abs() < 1e-4 && (t.translation.y - 9.0).abs() < 1e-4,
        "a âncora de mundo não andou: está em ({}, {})",
        t.translation.x,
        t.translation.y
    );
    assert!(
        follows(
            bridge.joint_anchor_world(&sim, joint, ph2d_physics_ecs::JointSide::B),
            [2.0, 9.0]
        ),
        "o anel não seguiu o próprio arrasto -- a porta le' um numero e escreve outro"
    );
    let after = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_eq!(
        before.local_a, after.local_a,
        "mover o prego NÃO pode mexer em onde no corpo o pino prende — é o \
         `local_a` intacto que faz o corpo ir junto"
    );

    // --- o DOT: onde no corpo prende, e o prego NÃO se move ---
    assert!(
        bridge.set_joint_anchor_world(&mut sim, joint, ph2d_physics_ecs::JointSide::A, [1.0, 6.0]),
        "a porta recusou o arrasto do dot"
    );
    assert!(
        follows(
            bridge.joint_anchor_world(&sim, joint, ph2d_physics_ecs::JointSide::A),
            [1.0, 6.0]
        ),
        "o dot não seguiu o próprio arrasto"
    );
    let after = *sim.world().get::<PhysicsJoint>(joint).expect("joint");
    assert_ne!(
        before.local_a, after.local_a,
        "o dot nao autorou onde no corpo o pino prende -- a pergunta segue sem porta"
    );
    let t = sim.world().get::<Transform>(joint).expect("joint");
    assert!(
        (t.translation.x - 2.0).abs() < 1e-4,
        "arrastar o dot moveu o PREGO: {}",
        t.translation.x
    );

    // --- o CONTROLE: um joint corpo-a-corpo segue escrevendo o local ---
    let mut sim2 = SimWorld::new();
    let a = sim2
        .world_mut()
        .spawn((
            Name::new("A"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider::default(),
            Transform::from_translation(Vec2::new(0.0, 6.0)),
        ))
        .id();
    let b = sim2
        .world_mut()
        .spawn((
            Name::new("B"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider::default(),
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    let j2 = crate::render_loop::inspector_joint::create_joint(
        &mut sim2,
        a.to_bits(),
        b.to_bits(),
        JointKind::Pin,
    )
    .expect("joint");
    let mut bridge2 = PhysicsBridge::new();
    bridge2.dispatch(&mut sim2, false, 0);
    let before2 = *sim2.world().get::<PhysicsJoint>(j2).expect("joint");
    bridge2.set_joint_anchor_world(&mut sim2, j2, ph2d_physics_ecs::JointSide::A, [1.0, 6.0]);
    let after2 = *sim2.world().get::<PhysicsJoint>(j2).expect("joint");
    assert_ne!(
        before2.local_a, after2.local_a,
        "CONTROLE: num joint corpo-a-corpo o arrasto tem de continuar escrevendo \
         o `local_a` — se ele passou a mover o pivô, a porta quebrou para TODO \
         joint do app"
    );
}

/// **O gesto vale nas DUAS direções, e elas produzem o MESMO pino** — o 2º
/// relato (*"arrastar do canvas vazio para o objeto também deveria funcionar"*).
///
/// ⚠️ **A primeira versão deste gate NÃO PODIA FALHAR:** eu chamava a porta de
/// criação com os MESMOS argumentos duas vezes e comparava os resultados —
/// verde por construção, sobre nada. O que a direção de fato muda é o PAPEL dos
/// dois pontos, e é isso que se pergunta aqui, à porta pura que os decide.
#[test]
fn the_gesture_reads_the_same_from_either_direction() {
    let press = [0.0, 5.0];
    let release = [0.0, 7.0];
    // Saindo do CORPO: o press é o ponto no corpo, o release é a âncora.
    assert_eq!(
        super::inspector_joint_world::gesture_points(true, press, release),
        (press, release)
    );
    // Saindo do CENÁRIO: os papéis TROCAM — o press é a âncora.
    assert_eq!(
        super::inspector_joint_world::gesture_points(false, release, press),
        (press, release),
        "vazio->corpo tinha de dar o MESMO par que corpo->vazio; sem a troca o \
         pino nasce com a âncora onde o artista SOLTOU, o oposto do que apontou"
    );
}
