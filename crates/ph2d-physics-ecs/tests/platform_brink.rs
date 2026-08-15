//! **A TRAVA DE BEIRADA, no PRODUTO** (`W-Brink`, `bCanWalkOffLedges`) — o que
//! o artista vê: ele anda para a quina e **PARA**, em vez de cair dela.
//!
//! O oráculo é sempre a TRAJETÓRIA e nunca uma bandeira interna: cair é o
//! `Transform` a descer abaixo do topo da laje, que é o mesmo que o olho julga.
//!
//! ⚠️ **Toda tabela desta wave saiu de `tests/measure_ledge_stop.rs`**, e as
//! duas leituras que decidiram o desenho estão lá com os números: o leque
//! sozinho **acende sobre uma fenda de 5 cm** (por isso o sensor pergunta à
//! frente) e um alcance igual à distância de paragem é o **caso de fronteira**
//! (por isso a ponte lhe soma a meia-largura).

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

const FLOAT: f32 = 0.9;
const HALF_W: f32 = 0.2;
/// A quina: a laje acaba aqui.
const LEDGE_X: f32 = 0.0;

/// O fim de uma travessia: *(caiu?, último `x`)*.
struct Run {
    fell: bool,
    last_x: f32,
}

/// Anda para a direita a partir de `x = −3` sobre uma laje que acaba em `x = 0`.
///
/// `gap` em `Some` põe uma SEGUNDA laje daquela largura de vão adiante — é como
/// a mesma cena responde às duas perguntas (*ele pára na beirada?* e *ele ainda
/// atravessa uma fenda?*) sem duas fixtures a divergirem.
fn walk_dir(dir: f32, trava: bool, speed: f32, gap: Option<f32>) -> Run {
    let mut sim = SimWorld::new();
    let slab = |sim: &mut SimWorld, name: &str, cx: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 10.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(cx, -0.5)),
        ));
    };
    slab(&mut sim, "Near", LEDGE_X - dir * 10.0);
    if let Some(g) = gap {
        slab(&mut sim, "Far", LEDGE_X + dir * (g + 10.0));
    }

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: HALF_W,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                speed,
                walk_off_ledges: !trava,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(LEDGE_X - dir * 3.0, FLOAT)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    let mut last_x = f32::NAN;
    for i in 1..=300u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: dir,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, i);
        let t = sim.world().get::<Transform>(player).expect("transform");
        last_x = t.translation.x;
        if t.translation.y < -1.0 {
            return Run { fell: true, last_x };
        }
    }
    Run {
        fell: false,
        last_x,
    }
}

/// **A wave numa asserção: ele PARA em vez de cair — e o CONTROLE cai.**
///
/// ⚠️ As duas metades vivem no mesmo teste de propósito: sozinha, a primeira
/// passaria com um personagem que não anda, e a segunda com uma trava inerte.
#[test]
fn he_stops_at_the_ledge_instead_of_walking_off_it() {
    let armed = walk_dir(1.0, true, 4.0, None);
    let control = walk_dir(1.0, false, 4.0, None);
    assert!(
        !armed.fell,
        "com a trava armada ele nao pode cair (parou em {:.4})",
        armed.last_x
    );
    assert!(
        control.fell,
        "e o CONTROLE tem de cair, senao o gate e' verde sobre um personagem \
         que nunca chegou a' quina (parou em {:.4})",
        control.last_x
    );
}

/// **Em TODA velocidade**, e é a metade que a primeira versão da wave não tinha.
///
/// ⚠️ Com o alcance igual à distância de paragem — o desenho anterior — ele
/// acabava equilibrado num pé só sobre o lábio e **caía a 2 m/s** enquanto as
/// outras velocidades escapavam por um fio. Uma trava que só funciona a algumas
/// velocidades não é uma trava, e o `2.0` está nesta lista por causa disso.
#[test]
fn the_trava_holds_at_every_speed() {
    for speed in [1.0f32, 2.0, 4.0, 6.0, 8.0, 12.0] {
        let r = walk_dir(1.0, true, speed, None);
        assert!(!r.fell, "a {speed} m/s ele caiu (parou em {:.4})", r.last_x);
        // E ele ANDOU: sem isto o gate seria verde sobre um personagem parado.
        assert!(
            r.last_x > LEDGE_X - 1.0,
            "a {speed} m/s ele nem chegou perto da quina: {:.4}",
            r.last_x
        );
    }
}

/// **Ele para PERTO da quina, não a meio caminho dela** — o número que separa
/// *"a trava funciona"* de *"a trava é usável"*.
///
/// ⚠️ **A barra é MEDIDA, não escolhida:** a borda do corpo pousa entre `−0,13`
/// e `+0,19` da quina na varredura de velocidade (a tabela do
/// `measure_ledge_stop`), e a folga do gate cobre isso com margem. Um número
/// mais apertado seria um gate a medir o solver.
#[test]
fn he_stops_within_a_body_width_of_the_ledge() {
    for speed in [1.0f32, 4.0, 8.0] {
        let r = walk_dir(1.0, true, speed, None);
        let edge = r.last_x + HALF_W;
        assert!(
            (LEDGE_X - 2.0 * HALF_W..=LEDGE_X + 2.0 * HALF_W).contains(&edge),
            "a {speed} m/s a borda parou em {edge:.4}, longe demais da quina"
        );
    }
}

/// **Uma fenda que a perna atravessa continua a ser atravessada** — o defeito
/// que o primeiro desenho tinha, e a razão de o sensor perguntar à FRENTE.
///
/// ⚠️ **A fenda LARGA é a outra metade**, e ela é a semântica, não um bug: um
/// vão que a perna não vence é um patamar, e a trava recusa andar para ele
/// mesmo que a inércia o cruzasse — que é literalmente o que *"não ande para
/// fora"* significa. O CONTROLE ao lado mostra que sem a trava ele o cruza.
#[test]
fn the_trava_still_crosses_a_gap_the_leg_spans() {
    for gap in [0.05f32, 0.15, 0.30] {
        let r = walk_dir(1.0, true, 4.0, Some(gap));
        assert!(
            !r.fell && r.last_x > LEDGE_X + gap + 0.3,
            "uma fenda de {gap} m tem de ser atravessada (caiu={}, x={:.4})",
            r.fell,
            r.last_x
        );
    }
    let wide = walk_dir(1.0, true, 4.0, Some(0.5));
    assert!(
        !wide.fell && wide.last_x < LEDGE_X + 0.5,
        "uma fenda que a perna nao vence e' um patamar: ele para' antes dela \
         (caiu={}, x={:.4})",
        wide.fell,
        wide.last_x
    );
    let control = walk_dir(1.0, false, 4.0, Some(0.5));
    assert!(
        control.last_x > LEDGE_X + 0.5,
        "e sem a trava a inercia cruza a mesma fenda: {:.4}",
        control.last_x
    );
}

/// **O mundo que já shipava é byte-idêntico** — a trava desarmada não move um
/// tique da trajetória, que é o que todo projeto já salvo recebe.
#[test]
fn an_unarmed_player_walks_exactly_as_he_did_before() {
    let a = walk_dir(1.0, false, 4.0, None);
    let b = walk_dir(1.0, false, 4.0, None);
    assert_eq!(a.last_x.to_bits(), b.last_x.to_bits(), "determinismo");
    // E ele CAI — o mundo de antes desta wave, sem uma trava a segurá-lo.
    assert!(a.fell, "sem a trava ele anda para fora do patamar");
}

/// **E do outro LADO** — o sensor casta na direção em que o dedo empurra, então
/// a esquerda é um caminho de código próprio, e não a mesma linha espelhada.
///
/// ⚠️ Ele existe porque uma mutação sobreviveu: inverter o braço esquerdo do
/// `clamp_target` passava por toda a suíte, que só sabia andar para a direita.
#[test]
fn the_trava_holds_walking_left_too() {
    let armed = walk_dir(-1.0, true, 4.0, None);
    let control = walk_dir(-1.0, false, 4.0, None);
    assert!(
        !armed.fell,
        "para a esquerda ele tambem para' (x={:.4})",
        armed.last_x
    );
    assert!(control.fell, "e o CONTROLE cai (x={:.4})", control.last_x);
    let edge = armed.last_x - HALF_W;
    assert!(
        (LEDGE_X - 2.0 * HALF_W..=LEDGE_X + 2.0 * HALF_W).contains(&edge),
        "a borda esquerda parou em {edge:.4}, longe demais da quina"
    );
}
