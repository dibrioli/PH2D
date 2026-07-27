//! **A cinemática DIRETA** (W-FK) — o elo gira na própria junta e os
//! descendentes seguem.
//!
//! O oráculo aqui é geométrico e não "algo se moveu": um movimento rígido
//! preserva TODA distância dentro da peça que se move, e é exatamente isso que
//! distingue a FK de qualquer outra coisa que também move corpos. Um gate que
//! só olhasse a ponta ficaria verde sobre uma implementação que esticasse a
//! cadeia.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

fn body(sim: &mut SimWorld, name: &str, x: f32, y: f32, kind: BodyKind) -> Entity {
    let _ = sim.world_mut().spawn((
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, y)),
    ));
    named(sim, name)
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

fn joint(
    sim: &mut SimWorld,
    a: &str,
    b: &str,
    kind: JointKind,
    at: [f32; 2],
    limits: Option<[f32; 2]>,
) -> Entity {
    let n = format!("J-{a}-{b}");
    let mut j = PhysicsJoint {
        body_a: stable_name_id(a),
        body_b: stable_name_id(b),
        kind,
        ..PhysicsJoint::of_kind(kind)
    };
    if let Some([lo, hi]) = limits {
        j.limits_enabled = true;
        j.limit_min = lo;
        j.limit_max = hi;
    }
    let _ = sim.world_mut().spawn((
        Name::new(&n),
        j,
        Transform::from_translation(Vec2::new(at[0], at[1])),
    ));
    named(sim, &n)
}

/// Gancho estático + três elos de 1 m deitados em +X, pinados ponta a ponta.
/// Âncoras em `x = 0, 1, 2`; centros em `0.5, 1.5, 2.5`.
fn arm(limits: Option<[f32; 2]>) -> (SimWorld, PhysicsBridge, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let hook = body(&mut sim, "Hook", 0.0, 0.0, BodyKind::Static);
    let l1 = body(&mut sim, "L1", 0.5, 0.0, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, 0.0, BodyKind::Dynamic);
    let l3 = body(&mut sim, "L3", 2.5, 0.0, BodyKind::Dynamic);
    joint(&mut sim, "Hook", "L1", JointKind::Pin, [0.0, 0.0], None);
    joint(&mut sim, "L1", "L2", JointKind::Pin, [1.0, 0.0], limits);
    joint(&mut sim, "L2", "L3", JointKind::Pin, [2.0, 0.0], None);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge, vec![hook, l1, l2, l3])
}

/// A pose de um corpo na saída de um `fk_move`.
fn pose_of(poses: &[(Entity, [f32; 2], f32)], e: Entity) -> ([f32; 2], f32) {
    poses
        .iter()
        .find(|(x, _, _)| *x == e)
        .map(|&(_, t, r)| (t, r))
        .expect("body is in the moved set")
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// **O elo pego gira em torno da PRÓPRIA junta, e os descendentes vão junto.**
///
/// Três afirmações que só valem juntas:
/// 1. o elo girou o ângulo que o cursor pediu (90°);
/// 2. o **pai** não se moveu — não está sequer no conjunto;
/// 3. a distância entre o elo e o filho dele é a MESMA (movimento rígido).
#[test]
fn a_link_swings_about_its_own_joint_and_carries_its_children() {
    let (sim, mut bridge, e) = arm(None);
    // Pega o elo do MEIO pelo centro dele. A junta acima é a de `x = 1`.
    assert!(
        bridge.fk_begin(&sim, e[2], [1.5, 0.0]),
        "o elo tem uma junta com grau de liberdade acima dele"
    );
    assert_eq!(
        bridge.fk_bodies(),
        &[e[2], e[3]],
        "a FK move o elo pego e os descendentes — e só eles"
    );

    // Cursor a 90° em torno da âncora (1, 0): de (1.5, 0) para (1, 0.5).
    let poses = bridge.fk_move([1.0, 0.5]);
    let (p2, r2) = pose_of(&poses, e[2]);
    let (p3, _) = pose_of(&poses, e[3]);

    assert!(
        (r2 - std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "o elo tinha de girar 90 graus, girou {r2}"
    );
    // (1.5, 0) girado 90° em torno de (1, 0) = (1, 0.5).
    assert!(
        dist(p2, [1.0, 0.5]) < 1e-3,
        "o centro do elo caiu em {p2:?}, não sobre o arco da junta"
    );
    // E o filho, que estava em (2.5, 0), vai para (1, 1.5).
    assert!(
        dist(p3, [1.0, 1.5]) < 1e-3,
        "o descendente caiu em {p3:?} — ele não seguiu rigidamente"
    );
    // O invariante que nomeia o gesto: a peça que se move é RÍGIDA.
    assert!(
        (dist(p2, p3) - 1.0).abs() < 1e-4,
        "a peça esticou: {} contra 1.0",
        dist(p2, p3)
    );
}

/// **O resultado é função do CURSOR, nunca da sequência de Moves.**
///
/// A doença que esta linha e a do Painter pagaram várias vezes: um gesto que
/// compõe sobre o resultado anterior faz o desenho depender da taxa de polling
/// do mouse. Aqui a fonte é congelada no press, então mil Moves intermediários
/// não podem mudar o destino.
#[test]
fn the_pose_is_a_fact_of_the_cursor_not_of_the_move_count() {
    let target = [1.0, 0.5];
    let direct = {
        let (sim, mut bridge, e) = arm(None);
        assert!(bridge.fk_begin(&sim, e[2], [1.5, 0.0]));
        bridge.fk_move(target)
    };
    let stepped = {
        let (sim, mut bridge, e) = arm(None);
        assert!(bridge.fk_begin(&sim, e[2], [1.5, 0.0]));
        // Vinte passos pelo arco, terminando no MESMO ponto.
        for i in 1..=20 {
            let t = i as f32 / 20.0;
            let a = t * std::f32::consts::FRAC_PI_2;
            let (s, c) = (a.sin(), a.cos());
            bridge.fk_move([1.0 + 0.5 * c, 0.5 * s]);
        }
        bridge.fk_move(target)
    };
    for (a, b) in direct.iter().zip(stepped.iter()) {
        assert_eq!(a.0, b.0, "a ordem do conjunto movido mudou");
        assert!(
            dist(a.1, b.1) < 1e-4 && (a.2 - b.2).abs() < 1e-4,
            "vinte Moves deram outra pose que um: {:?} contra {:?}",
            a,
            b
        );
    }
}

/// **Um limite autorado é honrado ao posar.**
///
/// O mesmo padrão-ouro que a IK teve de cravar: uma pose que o Play desfaz no
/// primeiro tick não é uma pose. Aqui a coordenada é exatamente o delta do
/// gesto, então o clamp é fechado — sem iteração, sem resíduo.
#[test]
fn a_hinge_limit_is_honoured_while_posing() {
    let (sim, mut bridge, e) = arm(Some([0.0, 0.3]));
    assert!(bridge.fk_begin(&sim, e[2], [1.5, 0.0]));
    // Puxa MUITO além do limite, para o lado permitido.
    let poses = bridge.fk_move([1.0, 0.5]);
    let (_, r) = pose_of(&poses, e[2]);
    assert!(
        (r - 0.3).abs() < 1e-3,
        "a junta parou em {r} rad, fora da faixa autorada [0, 0.3]"
    );
    // E para o lado proibido ela não passa de zero.
    let poses = bridge.fk_move([1.0, -0.5]);
    let (_, r) = pose_of(&poses, e[2]);
    assert!(r >= -1e-3, "a junta dobrou para o lado proibido: {r} rad");
}

/// **Sair da faixa e voltar devolve o elo ao CURSOR, na hora.**
///
/// ⚠️ Este gate nasceu VERMELHO e derrubou a primeira versão do `fk_move`, que
/// clampava o ACUMULADOR: o excedente era jogado fora, e voltar para dentro da
/// faixa movia a junta em relação à parede em vez de em relação à mão — o elo
/// ficava 0,00 rad onde o cursor pedia 0,15 e não voltava mais a sincronizar.
///
/// A lei certa é a de um slider: isto é manipulação DIRETA, o ângulo do cursor
/// em torno do pivô É o ângulo da junta, então quem se clampa é o MAPEAMENTO.
#[test]
fn coming_back_from_a_limit_is_immediate() {
    let (sim, mut bridge, e) = arm(Some([0.0, 0.3]));
    assert!(bridge.fk_begin(&sim, e[2], [1.5, 0.0]));
    let at = |a: f32| {
        let (s, c) = (a.sin(), a.cos());
        [1.0 + 0.5 * c, 0.5 * s]
    };
    // Dentro da faixa: o elo segue o cursor.
    let poses = bridge.fk_move(at(0.2));
    assert!((pose_of(&poses, e[2]).1 - 0.2).abs() < 1e-3);
    // Muito fora: ele para na parede.
    let poses = bridge.fk_move(at(1.2));
    assert!((pose_of(&poses, e[2]).1 - 0.3).abs() < 1e-3);
    // E de volta para dentro: ele volta AO CURSOR, não à parede menos um passo.
    let poses = bridge.fk_move(at(0.15));
    let (_, r) = pose_of(&poses, e[2]);
    assert!(
        (r - 0.15).abs() < 1e-3,
        "voltar do limite deu {r} rad em vez de 0.15 — o gesto guardou o excesso"
    );
}

/// **Um Weld é UMA peça: o gesto sobe até a junta seguinte e leva a peça
/// inteira.**
///
/// Sem isso, pegar um elo soldado não faria nada — e "não faz nada" é
/// indistinguível de quebrado.
#[test]
fn a_welded_link_swings_from_the_joint_above_it() {
    let mut sim = SimWorld::new();
    let hook = body(&mut sim, "Hook", 0.0, 0.0, BodyKind::Static);
    let l1 = body(&mut sim, "L1", 0.5, 0.0, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, 0.0, BodyKind::Dynamic);
    joint(&mut sim, "Hook", "L1", JointKind::Pin, [0.0, 0.0], None);
    // L2 é SOLDADO em L1: os dois são uma peça.
    joint(&mut sim, "L1", "L2", JointKind::Weld, [1.0, 0.0], None);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let _ = hook;

    assert!(
        bridge.fk_begin(&sim, l2, [1.5, 0.0]),
        "pegar o elo soldado tem de subir para a junta do gancho"
    );
    let mut moved = bridge.fk_bodies().to_vec();
    moved.sort_by_key(|e| e.to_bits());
    let mut want = vec![l1, l2];
    want.sort_by_key(|e| e.to_bits());
    assert_eq!(moved, want, "a peça soldada tem de viajar inteira");

    // Gira 90° em torno da âncora do GANCHO, que é (0, 0) — não a do Weld.
    let poses = bridge.fk_move([0.0, 1.5]);
    let (p1, _) = pose_of(&poses, l1);
    assert!(
        dist(p1, [0.0, 0.5]) < 1e-3,
        "o pivô não foi o do gancho: L1 caiu em {p1:?}"
    );
}

/// **Um trilho DESLIZA, e o limite dele é em metros.**
///
/// O Slider é a outra metade de [`ph2d_physics::fk_dof`], e é onde um gesto que
/// só soubesse girar produziria uma pose que o solver desfaz — o joint proíbe
/// rotação relativa por construção.
#[test]
fn a_slider_link_slides_along_its_axis_within_the_stroke() {
    let mut sim = SimWorld::new();
    let rail = body(&mut sim, "Rail", 0.0, 0.0, BodyKind::Static);
    let car = body(&mut sim, "Car", 0.0, 0.0, BodyKind::Dynamic);
    joint(
        &mut sim,
        "Rail",
        "Car",
        JointKind::Slider,
        [0.0, 0.0],
        Some([-1.0, 1.0]),
    );
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let _ = rail;

    assert!(bridge.fk_begin(&sim, car, [0.0, 0.0]));
    // O eixo default do Slider é +X. Puxa 0.4 m para a direita.
    let poses = bridge.fk_move([0.4, 0.9]);
    let (p, r) = pose_of(&poses, car);
    assert!(
        (p[0] - 0.4).abs() < 1e-3 && p[1].abs() < 1e-3,
        "o carro saiu do trilho: {p:?}"
    );
    assert!(r.abs() < 1e-6, "um trilho não gira nada, e girou {r}");
    // E o curso é honrado.
    let poses = bridge.fk_move([9.0, 0.0]);
    let (p, _) = pose_of(&poses, car);
    assert!(
        (p[0] - 1.0).abs() < 1e-3,
        "o carro passou do fim do curso: {p:?}"
    );
}

/// **Sem junta acima, não há FK** — e a recusa deixa o arrasto normal acontecer.
#[test]
fn a_lone_body_has_no_joint_to_swing_about() {
    let mut sim = SimWorld::new();
    let solo = body(&mut sim, "Solo", 0.0, 0.0, BodyKind::Dynamic);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert!(!bridge.fk_begin(&sim, solo, [0.0, 0.0]));
    assert!(!bridge.is_posing_fk());
    assert!(bridge.fk_move([1.0, 1.0]).is_empty());
}

/// **Um joint autorado com o FILHO como corpo A ainda abre, e pivota no PAI.**
///
/// A metade que este gate de fato prova é a resolução do lado B:
/// `joint_anchor_world` devolve `None` para o lado B de um joint não semeado, e
/// nesse caso o gesto inteiro recusaria.
///
/// ⚠️ **O que ele NÃO prova, e a medição é que disse:** numa dobradiça em
/// repouso as duas âncoras são o **mesmo ponto de mundo** e os frames locais do
/// Pin carregam só translação — então nem a troca de âncoras nem a escolha do
/// lado mudam um número aqui. A mutação que apaga o `swap_anchors` SOBREVIVE a
/// esta fixture, e é o gate seguinte que a mata.
#[test]
fn a_joint_authored_child_first_still_opens_and_pivots_on_the_parent() {
    let mut sim = SimWorld::new();
    let hook = body(&mut sim, "Hook", 0.0, 0.0, BodyKind::Static);
    let link = body(&mut sim, "Link", 0.5, 0.0, BodyKind::Dynamic);
    // ⚠️ Ordem INVERTIDA: o corpo A do joint é o elo, o B é o gancho.
    joint(&mut sim, "Link", "Hook", JointKind::Pin, [0.0, 0.0], None);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let _ = hook;

    assert!(bridge.fk_begin(&sim, link, [0.5, 0.0]));
    let poses = bridge.fk_move([0.0, 0.5]);
    let (p, _) = pose_of(&poses, link);
    // Girando 90° em torno de (0,0) o centro (0.5, 0) vai para (0, 0.5).
    assert!(
        dist(p, [0.0, 0.5]) < 1e-3,
        "o pivô saiu no corpo errado: o elo caiu em {p:?}"
    );
}

/// **A coordenada de um trilho é medida do lado do PAI** — e é aqui que a troca
/// de âncoras deixa de ser inerte.
///
/// ⚠️ Este gate existe porque o irmão acima **não podia** pegar a mutação
/// (`swap_anchors` removido): num Pin em repouso a troca não move número nenhum.
/// Num trilho ela move dois — a coordenada troca de SINAL, porque medir "quanto
/// o carro andou" da ponta do carro em vez da ponta do trilho é a mesma distância
/// para o outro lado. Com um limite autorado isso vira curso errado, visível.
///
/// A fixture tem as três coisas de que o fenômeno precisa: um **Slider**, o carro
/// **DESLOCADO** da origem do trilho (na origem os dois lados coincidem), e o
/// joint autorado **filho primeiro**.
///
/// ⚠️ **O número que este gate afirma corrigiu a minha premissa.** Eu esperava
/// que o carro parasse em `x = 1.0`, lendo o limite como *"posição no trilho"*;
/// o par de âncoras é semeado **em repouso** (W-AnchorFollow), então a
/// coordenada de QUALQUER junta é **0** onde o artista a criou, e um curso
/// `[0, 1]` significa *"daqui até um metro adiante"* — que é exatamente o que o
/// solver impõe no Play. `0.6 + 1.0 = 1.6`. Medido: com a troca, 1,6; sem ela,
/// **0,4**.
#[test]
fn the_slider_coordinate_is_measured_from_the_parents_side() {
    let mut sim = SimWorld::new();
    let rail = body(&mut sim, "Rail", 0.0, 0.0, BodyKind::Static);
    let car = body(&mut sim, "Car", 0.6, 0.0, BodyKind::Dynamic);
    joint(
        &mut sim,
        "Car",
        "Rail",
        JointKind::Slider,
        [0.0, 0.0],
        Some([0.0, 1.0]),
    );
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let _ = rail;

    assert!(bridge.fk_begin(&sim, car, [0.6, 0.0]));
    // Puxa muito além do fim do curso: o carro tem de parar em x = 1.0.
    let poses = bridge.fk_move([9.0, 0.0]);
    let (p, _) = pose_of(&poses, car);
    assert!(
        (p[0] - 1.6).abs() < 1e-3,
        "o carro parou em {p:?}: a coordenada foi medida do lado errado, então o \
         curso autorado [0, 1] a partir do repouso virou outro"
    );
}
