//! Os gates do PLANEIO (`W-Glide`).
//!
//! ⚠️ **O oráculo destes gates é o `Δv` que a lei impõe**, e não um estado: o
//! que o jogador vê é o personagem parar de acelerar para baixo, e um gate sobre
//! um booleano `gliding` ficaria verde com o corpo a cair como sempre.

use super::*;

const UP: Vec2 = [0.0, 1.0];

fn armed() -> GlideConfig {
    GlideConfig { fall_speed: 2.0 }
}

/// **O mundo de ANTES desta wave** — desligada, ela não escreve um bit.
///
/// ⚠️ **É este o gate que torna o degrau de schema barato:** o default é `0.0`,
/// então todo projeto salvo antes desta wave reabre a cair exatamente como caía.
#[test]
fn a_disabled_glide_is_the_world_before_this_wave() {
    let off = GlideConfig::STARTING_POINT;
    assert!(!off.armed());
    for v in [-20.0_f32, -5.0, 0.0, 5.0] {
        assert_eq!(
            glide_motor(&off, true, v, UP),
            Motor::default(),
            "desligada, a lei nao pode escrever nada (rel_up = {v})"
        );
    }
}

/// **Sem o dedo não há planeio** — é um regime, e ele dura enquanto o dedo dura.
#[test]
fn without_the_finger_there_is_no_glide() {
    assert_eq!(glide_motor(&armed(), false, -12.0, UP), Motor::default());
}

/// **O FREIO: uma queda mais rápida que o teto é travada até ele.**
#[test]
fn a_fall_faster_than_the_ceiling_is_braked_to_it() {
    let c = armed();
    let m = glide_motor(&c, true, -12.0, UP);
    // O `boost` leva `rel_up` de −12 para −2 ⇒ Δv = +10.
    assert!(
        (m.boost[1] - 10.0).abs() < 1.0e-4,
        "a lei tem de levar a descida ate' o teto: {:?}",
        m.boost
    );
    assert_eq!(
        m.accel,
        [0.0, 0.0],
        "o planeio e' VELOCIDADE, nao aceleracao"
    );
}

/// **⚠️ O GATE DA WAVE: a lei NUNCA empurra para baixo.**
///
/// É esta propriedade que separa o **teto** do **alvo**, e é o motivo de o alvo
/// ter sido descartado: apertado a subir a 8 m/s, um alvo imporia `−10 m/s`.
/// Aqui a resposta tem de ser silêncio em **todo** momento que não seja uma
/// queda rápida — subindo, no ápice, e caindo devagar.
#[test]
fn the_law_can_never_push_downward() {
    let c = armed();
    for v in [8.0_f32, 2.0, 0.5, 0.0, -0.5, -1.0, -1.99] {
        let m = glide_motor(&c, true, v, UP);
        assert!(
            m.boost[1] >= 0.0,
            "a rel_up {v} produziu um empurrao para BAIXO: {:?}",
            m.boost
        );
        assert_eq!(
            m,
            Motor::default(),
            "e acima do teto a lei tem de ficar CALADA (rel_up = {v})"
        );
    }
}

/// **A fronteira exata é o teto**, e ela é fechada do lado de cima.
///
/// ⚠️ **`>=` e não `>`**: em `rel_up == −fall_speed` o `delta` seria exatamente
/// zero, e um `boost` de zero não é o mesmo que silêncio — ele conta como *"esta
/// lei escreveu este eixo"* para quem somar motores, e o módulo tem laws que se
/// calam justamente para não serem dois donos do mesmo número.
#[test]
fn the_boundary_is_the_ceiling_and_it_is_closed_from_above() {
    let c = armed();
    assert_eq!(
        glide_motor(&c, true, -2.0, UP),
        Motor::default(),
        "exatamente NO teto a lei nao age"
    );
    assert_ne!(
        glide_motor(&c, true, -2.001, UP),
        Motor::default(),
        "e um fio abaixo dele, age"
    );
}

/// **O teto é o número autorado** — dois valores, duas descidas.
#[test]
fn the_ceiling_is_the_number_the_artist_wrote() {
    for speed in [1.0_f32, 2.0, 4.0] {
        let c = GlideConfig { fall_speed: speed };
        let m = glide_motor(&c, true, -20.0, UP);
        let landed = -20.0 + m.boost[1];
        assert!(
            (landed + speed).abs() < 1.0e-4,
            "com fall_speed {speed} a descida tem de ficar em {}: ficou {landed}",
            -speed
        );
    }
}

/// **O eixo de CIMA é o do suporte, não o `Y` do mundo.**
///
/// ⚠️ Sem isto o planeio numa gravidade lateral travaria o eixo errado — e o
/// módulo inteiro carrega o `up` por essa razão.
#[test]
fn the_brake_runs_along_the_support_axis() {
    let c = armed();
    let side: Vec2 = [1.0, 0.0];
    let m = glide_motor(&c, true, -12.0, side);
    assert!((m.boost[0] - 10.0).abs() < 1.0e-4, "{:?}", m.boost);
    assert!(m.boost[1].abs() < 1.0e-4, "{:?}", m.boost);
}
