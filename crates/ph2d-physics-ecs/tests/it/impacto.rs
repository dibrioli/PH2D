//! ⭐⭐⭐ **O IMPACTO na ponte** (plano 28, W5) — o empurrão e o piscar, pela API pública.
//!
//! Cada gate tem o CONTROLO ao lado: a mesma cena com a peça que ele afirma retirada. ⚠️ Sem ele,
//! uma ponte que empurrasse TUDO (ou nada) passaria: um corpo dinâmico sobreposto a um sólido
//! também se mexe sozinho — o solver desfaz a sobreposição —, e é o controlo que separa o
//! empurrão do golpe da despenetração.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::health::HealthEventKind;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, GravityScale, Health, PhysicsBridge, RigidBody,
    TopDownPlayer,
};

fn caixa(h: f32, sensor: bool) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: h,
            half_y: h,
        },
        density: 1.0,
        is_sensor: sensor,
        ..Collider::default()
    }
}

/// Um alvo DINÂMICO sem gravidade — é o solver que o move, logo só um empurrão o tira do sítio.
fn alvo_dinamico(sim: &mut SimWorld, vida: Health) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Alvo"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            caixa(0.4, false),
            GravityScale(0.0),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            vida,
        ))
        .id()
}

/// Uma fonte de dano estática em `(x, y)`, sensor ou sólida.
fn fonte(sim: &mut SimWorld, x: f32, y: f32, sensor: bool, dano: Damage) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Fonte"),
            RigidBody {
                kind: BodyKind::Static,
            },
            caixa(0.4, sensor),
            Transform::from_translation(Vec2::new(x, y)),
            dano,
        ))
        .id()
}

fn empurra(knockback: f32, lift: f32) -> Damage {
    Damage {
        amount: 10.0,
        knockback,
        knockback_lift: lift,
        ..Damage::default()
    }
}

fn pose(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("o alvo tem pose");
    [t.translation.x, t.translation.y]
}

/// Corre `0..=ticks` e devolve onde o alvo ficou e os factos de vida.
fn corre(sim: &mut SimWorld, alvo: Entity, ticks: u64) -> ([f32; 2], Vec<HealthEventKind>) {
    let mut ponte = PhysicsBridge::new();
    let mut factos = Vec::new();
    for t in 0..=ticks {
        ponte.dispatch(sim, true, t);
        factos.extend(ponte.health_events().iter().map(|e| e.kind));
    }
    (pose(sim, alvo), factos)
}

/// A cena do sensor: a fonte à esquerda, sobreposta, e o empurrão a sair do centro dela para o do
/// alvo (um sensor não tem normal).
fn cena_do_sensor(dano: Damage, vida: Health) -> [f32; 2] {
    let mut sim = SimWorld::new();
    let a = alvo_dinamico(&mut sim, vida);
    let _f = fonte(&mut sim, -0.3, 0.0, true, dano);
    corre(&mut sim, a, 60).0
}

/// ⭐⭐⭐ **Um golpe EMPURRA quem leva** — e o CONTROLO (a mesma cena com o empurrão a zero) não
/// sai do sítio, porque um sensor não empurra fisicamente.
///
/// **Mutação que deve sangrar:** apagar o `self.empurra(corpo, dv)` do fim do `drive_health`.
#[test]
fn um_golpe_empurra_quem_leva() {
    let com = cena_do_sensor(empurra(4.0, 0.0), Health::default());
    let sem = cena_do_sensor(empurra(0.0, 0.0), Health::default());
    assert!(sem[0].abs() < 1e-4, "o CONTROLO mexeu-se sozinho: {sem:?}");
    // Um segundo a 4 m/s, menos o tique do golpe.
    assert!(com[0] > 3.0, "o golpe não empurrou: {com:?}");
    assert!(com[1].abs() < 1e-3, "o empurrão saiu torto: {com:?}");
}

/// ⭐⭐⭐ **O empurrão sai da NORMAL REAL do contacto** — a vantagem que o plano §5 declara sobre
/// o Godot e o Unity. A fonte sólida está em cima E à esquerda: a direcção de centro a centro
/// desce `~34°`, e a normal da face que toca é HORIZONTAL. ⇒ o alvo tem de sair a direito.
///
/// **Mutação que deve sangrar:** trocar o `Some(amostra.normal)` dos contactos por `None` (a
/// direcção passa a ser a do centro, e o alvo desce).
#[test]
fn o_empurrao_sai_da_normal_real_do_contacto() {
    let mut sim = SimWorld::new();
    let a = alvo_dinamico(&mut sim, Health::default());
    // Sobreposição de `0,05` em x e de `0,3` em y: a face que toca é a vertical.
    let _f = fonte(&mut sim, -0.75, 0.5, false, empurra(4.0, 0.0));
    let (onde, factos) = corre(&mut sim, a, 60);
    assert!(
        factos
            .iter()
            .any(|k| matches!(k, HealthEventKind::Damaged { .. })),
        "a fonte sólida nunca bateu: {factos:?}"
    );
    assert!(onde[0] > 3.0, "o golpe não empurrou: {onde:?}");
    // A direcção de centro a centro poria o alvo `~0,66 × x` abaixo; a normal põe-no a direito.
    assert!(
        onde[1].abs() < 0.1 * onde[0],
        "o alvo saiu na direcção do CENTRO e não da normal do contacto: {onde:?}"
    );

    // O CONTROLO: sem empurrão, só a despenetração o mexe — pouco.
    let mut sim = SimWorld::new();
    let a = alvo_dinamico(&mut sim, Health::default());
    let _f = fonte(&mut sim, -0.75, 0.5, false, empurra(0.0, 0.0));
    let (sem, _) = corre(&mut sim, a, 60);
    assert!(
        sem[0] < 0.5,
        "a despenetração sozinha levou-o longe: {sem:?}"
    );
}

/// ⭐⭐ **Quem LEVA decide** — o `knockback_taken` da vida escala o empurrão: `0` não se mexe (igual
/// ao controlo), `2` voa o dobro de `1`.
///
/// **Mutação que deve sangrar:** o `empurrao` ignorar o `knockback_taken`.
#[test]
fn quem_leva_decide_quanto_voa() {
    let leva = |k: f32| {
        cena_do_sensor(
            empurra(3.0, 0.0),
            Health {
                knockback_taken: k,
                ..Health::default()
            },
        )
    };
    let (zero, um, dois) = (leva(0.0), leva(1.0), leva(2.0));
    assert!(zero[0].abs() < 1e-4, "com `0` ele voou: {zero:?}");
    let razao = dois[0] / um[0];
    assert!(
        (1.9..=2.1).contains(&razao),
        "`2` tinha de voar o dobro de `1`: {um:?} contra {dois:?}"
    );
}

/// ⭐⭐ **Só um golpe que ENTRA empurra** — uma esquiva certa não o tira do sítio (um herói que
/// esquiva e é arrastado leria-se como um golpe que entrou sem tirar vida).
///
/// **Mutação que deve sangrar:** o `empurrao` deixar de exigir um facto de dano.
#[test]
fn uma_esquiva_nao_empurra() {
    let esquiva = cena_do_sensor(
        empurra(4.0, 0.0),
        Health {
            dodge: 1.0,
            ..Health::default()
        },
    );
    assert!(esquiva[0].abs() < 1e-4, "a esquiva empurrou: {esquiva:?}");
}

/// ⭐ **O empurrão PARA CIMA não precisa de direcção** — um golpe num chão plano levanta o herói.
#[test]
fn o_empurrao_para_cima_levanta_a_direito() {
    let onde = cena_do_sensor(empurra(0.0, 3.0), Health::default());
    assert!(onde[1] > 2.0, "o empurrão para cima não levantou: {onde:?}");
    assert!(onde[0].abs() < 1e-3, "ele saiu de lado: {onde:?}");
}

/// ⭐⭐⭐ **O mover de VISTA DE CIMA é empurrado pelo ESTADO dele** — o corpo dele é cinemático e o
/// solver recusaria o empurrão; ele entra pela velocidade que a lei do mover já guarda e trava.
///
/// **Mutação que deve sangrar:** apagar o ramo do `topdown_state` do `empurra`.
#[test]
fn o_mover_de_vista_de_cima_e_empurrado_pelo_estado_dele() {
    let corrida = |k: f32| {
        let mut sim = SimWorld::new();
        let a = sim
            .world_mut()
            .spawn((
                Name::new("Heroi"),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                caixa(0.4, false),
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                TopDownPlayer::default(),
                Health::default(),
            ))
            .id();
        let _f = fonte(&mut sim, -0.3, 0.0, true, empurra(k, 0.0));
        corre(&mut sim, a, 60).0
    };
    let (com, sem) = (corrida(8.0), corrida(0.0));
    assert!(sem[0].abs() < 1e-4, "o CONTROLO andou sozinho: {sem:?}");
    assert!(com[0] > 0.3, "o mover não foi empurrado: {com:?}");
}

/// ⭐⭐⭐ **Um scrub devolve a pose EXACTA do tique** — o empurrão é estado de SIMULAÇÃO, e o replay
/// refá-lo igual (ao bit).
///
/// **Mutação que deve sangrar:** empurrar só quando `publicar` (o replay deixa de o refazer).
#[test]
fn um_scrub_devolve_a_pose_empurrada_exacta() {
    let mut sim = SimWorld::new();
    let a = alvo_dinamico(&mut sim, Health::default());
    let _f = fonte(&mut sim, -0.3, 0.0, true, empurra(4.0, 1.0));
    let mut ponte = PhysicsBridge::new();
    let mut corrida = Vec::new();
    for t in 0..=90 {
        ponte.dispatch(&mut sim, true, t);
        corrida.push(pose(&sim, a).map(f32::to_bits));
    }
    assert!(f32::from_bits(corrida[90][0]) > 3.0, "a cena não empurrou");
    for t in [30_u64, 75, 5, 90] {
        ponte.dispatch(&mut sim, true, t);
        assert_eq!(
            pose(&sim, a).map(f32::to_bits),
            corrida[t as usize],
            "o scrub para o tique {t} devolveu outra pose"
        );
    }
}

/// Um alvo que não se mexe (cinemático) com invencibilidade e piscar.
fn alvo_que_pisca(sim: &mut SimWorld, blink_s: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Alvo"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            caixa(0.4, false),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            Health {
                invincible_s: 0.5,
                blink_s,
                ..Health::default()
            },
        ))
        .id()
}

/// ⭐⭐⭐ **A invencibilidade PISCA sozinha** — a vantagem que o plano §5 declara: começa VISÍVEL
/// (o clarão vê-se), alterna a cada `blink_s`, e acaba com a janela. ⭐ O CONTROLO (`blink_s = 0`)
/// nunca esconde.
///
/// **Mutações que devem sangrar:** o `publica_vidas` não pôr a marca · pô-la fora da janela.
#[test]
fn a_invencibilidade_pisca_sozinha() {
    let mut sim = SimWorld::new();
    let a = alvo_que_pisca(&mut sim, 0.1);
    let _f = fonte(&mut sim, -0.3, 0.0, true, empurra(0.0, 0.0));
    let mut ponte = PhysicsBridge::new();
    let mut escondido = Vec::new();
    let mut golpe = None;
    for t in 0..=60 {
        ponte.dispatch(&mut sim, true, t);
        if golpe.is_none() && !ponte.health_events().is_empty() {
            golpe = Some(t as usize);
        }
        escondido.push(sim.world().get::<ph2d_ecs::BlinkOff>(a).is_some());
    }
    let g = golpe.expect("o sensor nunca bateu");
    // `k` tiques depois do golpe o relógio da vida lê `k/60` s; metades de `0,1 s`.
    assert!(
        !escondido[g],
        "no tique do golpe ele tem de se VER (o clarão)"
    );
    assert!(!escondido[g + 1]);
    assert!(escondido[g + 7], "na 2.ª metade (0,117 s) ele esconde-se");
    assert!(!escondido[g + 13], "na 3.ª metade (0,217 s) ele volta");
    assert!(
        escondido[g + 19],
        "na 4.ª metade (0,317 s) esconde-se outra vez"
    );
    assert!(
        !escondido[g + 35],
        "depois da janela (0,583 s > 0,5 s) ele fica visível para sempre"
    );

    // O CONTROLO: sem piscar, nunca se esconde.
    let mut sim = SimWorld::new();
    let a = alvo_que_pisca(&mut sim, 0.0);
    let _f = fonte(&mut sim, -0.3, 0.0, true, empurra(0.0, 0.0));
    let mut ponte = PhysicsBridge::new();
    for t in 0..=60 {
        ponte.dispatch(&mut sim, true, t);
        assert!(sim.world().get::<ph2d_ecs::BlinkOff>(a).is_none());
    }
}

/// ⭐⭐ **Quem PERDE a vida perde a marca** — senão um objecto cuja `Health` saiu a meio de uma
/// metade escondida ficava invisível para sempre.
///
/// **Mutação que deve sangrar:** apagar a varredura dos `apagados` do `publica_vidas`.
#[test]
fn quem_perde_a_vida_perde_a_marca_do_piscar() {
    let mut sim = SimWorld::new();
    let a = alvo_que_pisca(&mut sim, 0.1);
    let _f = fonte(&mut sim, -0.3, 0.0, true, empurra(0.0, 0.0));
    let mut ponte = PhysicsBridge::new();
    let mut t = 0;
    while sim.world().get::<ph2d_ecs::BlinkOff>(a).is_none() {
        ponte.dispatch(&mut sim, true, t);
        t += 1;
        assert!(t < 60, "a marca nunca apareceu — o gate mediria nada");
    }
    sim.world_mut().entity_mut(a).remove::<Health>();
    ponte.dispatch(&mut sim, true, t);
    assert!(
        sim.world().get::<ph2d_ecs::BlinkOff>(a).is_none(),
        "sem vida, a marca do piscar ficou — o objecto ficaria invisível para sempre"
    );
}

/// ⭐⭐ **Um dano CONTÍNUO empurra UMA vez — ao ENTRAR, nunca a cada tique** — a lava que fere a
/// cada tique não é uma mangueira: sem a cerca `comecou` o empurrão somava-se enquanto o toque
/// durasse (e o dano contínuo ENTRA em todo tique, logo a cerca «algo entrou» não o apanha).
///
/// ⚠️ **Esta é a fixtura que a 1.ª prova de mutação NÃO tinha** (a F8 sobreviveu): com dano por
/// golpe o dano só entra no tique do começo, e as duas cercas coincidem.
///
/// **Mutação que deve sangrar:** tirar o `comecou &&` do empurrão.
#[test]
fn um_dano_continuo_empurra_so_ao_entrar() {
    let continuo = Damage {
        per_second: true,
        amount: 30.0,
        ..empurra(4.0, 0.0)
    };
    let um_golpe = cena_do_sensor(empurra(4.0, 0.0), Health::default());
    let mangueira = cena_do_sensor(continuo, Health::default());
    assert!(
        um_golpe[0] > 0.3,
        "o CONTROLO não foi empurrado: {um_golpe:?}"
    );
    assert!(
        (mangueira[0] - um_golpe[0]).abs() < 0.05 * um_golpe[0],
        "um dano contínuo empurrou mais do que um golpe: {mangueira:?} contra {um_golpe:?}"
    );
}

/// ⭐⭐ **O empurrão do mover GASTA-SE e ele PÁRA** — o canal próprio desce a zero à taxa da lei
/// (`knockback_recovery`, fábrica `24 m/s²`: um empurrão de `8 m/s` gasta-se em `⅓ s`).
///
/// **Mutação que deve sangrar:** o `recover` devolver o empurrão sem o gastar (o herói desliza para
/// sempre).
#[test]
fn o_empurrao_do_mover_gasta_se_e_ele_para() {
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((
            Name::new("Heroi"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            caixa(0.4, false),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            TopDownPlayer::default(),
            Health::default(),
        ))
        .id();
    let _f = fonte(&mut sim, -0.3, 0.0, true, empurra(8.0, 0.0));
    let mut ponte = PhysicsBridge::new();
    let mut aos_60 = [0.0; 2];
    for t in 0..=120 {
        ponte.dispatch(&mut sim, true, t);
        if t == 60 {
            aos_60 = pose(&sim, a);
        }
    }
    let aos_120 = pose(&sim, a);
    assert!(aos_60[0] > 0.3, "o mover não foi empurrado: {aos_60:?}");
    assert!(
        (aos_120[0] - aos_60[0]).abs() < 1e-3,
        "o mover ainda desliza um segundo depois do golpe: {aos_60:?} → {aos_120:?}"
    );
}
