//! Os gates do TETO DE DESCIDA (`W-Glide` + `W-Fall`).
//!
//! ⚠️ **O oráculo destes gates é o `Δv` que a lei impõe**, e não um estado: o
//! que o jogador vê é o personagem parar de acelerar para baixo, e um gate sobre
//! um booleano `gliding` ficaria verde com o corpo a cair como sempre.
//!
//! ⚠️ **Os sete primeiros nasceram no `glide_tests.rs` e foram RE-APONTADOS, não
//! reescritos.** Eles sempre mediram a lei do TETO, que agora é partilhada — e é
//! por isso que servem de rede: o teto de queda entra por baixo de gates que já
//! provavam o comportamento do planeio, em vez de estrear numa suíte própria que
//! nada obriga a concordar com a antiga.

use super::*;

const UP: Vec2 = [0.0, 1.0];

fn glide_at(speed: f32) -> GlideConfig {
    GlideConfig { fall_speed: speed }
}

fn cap_at(speed: f32) -> FallConfig {
    FallConfig { max_speed: speed }
}

/// O planeio armado a 2 m/s e NENHUM teto de queda — o mundo do `W-Glide`.
fn glide_only(held: bool) -> Option<f32> {
    descent_ceiling(&glide_at(2.0), &FallConfig::STARTING_POINT, held)
}

/// **O mundo de ANTES das duas waves** — desligadas, elas não escrevem um bit.
///
/// ⚠️ **É este o gate que torna os dois degraus de schema baratos:** os defaults
/// são `0.0`, então todo projeto salvo antes reabre a cair exatamente como caía.
#[test]
fn with_both_laws_disabled_nothing_is_written() {
    let ceiling = descent_ceiling(
        &GlideConfig::STARTING_POINT,
        &FallConfig::STARTING_POINT,
        true,
    );
    assert_eq!(ceiling, None, "sem lei autorada nao ha' teto");
    for v in [-200.0_f32, -20.0, -5.0, 0.0, 5.0] {
        assert_eq!(
            descent_motor(ceiling, v, UP),
            Motor::default(),
            "desligadas, a lei nao pode escrever nada (rel_up = {v})"
        );
    }
}

/// **Sem o dedo não há planeio** — é um regime, e ele dura enquanto o dedo dura.
#[test]
fn without_the_finger_there_is_no_glide() {
    assert_eq!(glide_only(false), None);
    assert_eq!(descent_motor(glide_only(false), -12.0, UP), Motor::default());
}

/// **⚠️ E o TETO DE QUEDA não pergunta nada ao jogador** — é o discriminante
/// entre as duas leis, e o que faz dele uma velocidade terminal em vez de uma
/// segunda assistência.
#[test]
fn the_fall_cap_needs_no_finger() {
    let ceiling = descent_ceiling(&GlideConfig::STARTING_POINT, &cap_at(20.0), false);
    assert_eq!(ceiling, Some(20.0));
    let m = descent_motor(ceiling, -50.0, UP);
    assert!(
        (m.boost[1] - 30.0).abs() < 1.0e-4,
        "o teto tem de travar uma queda de 50 ate' 20 com o dedo SOLTO: {:?}",
        m.boost
    );
}

/// **O FREIO: uma queda mais rápida que o teto é travada até ele.**
#[test]
fn a_fall_faster_than_the_ceiling_is_braked_to_it() {
    let m = descent_motor(glide_only(true), -12.0, UP);
    // O `boost` leva `rel_up` de −12 para −2 ⇒ Δv = +10.
    assert!(
        (m.boost[1] - 10.0).abs() < 1.0e-4,
        "a lei tem de levar a descida ate' o teto: {:?}",
        m.boost
    );
    assert_eq!(m.accel, [0.0, 0.0], "o freio e' VELOCIDADE, nao aceleracao");
}

/// **⚠️ O GATE DA WAVE: a lei NUNCA empurra para baixo.**
///
/// É esta propriedade que separa o **teto** do **alvo**, e é o motivo de o alvo
/// ter sido descartado: apertado a subir a 8 m/s, um alvo imporia `−10 m/s`.
/// Aqui a resposta tem de ser silêncio em **todo** momento que não seja uma
/// queda rápida — subindo, no ápice, e caindo devagar.
#[test]
fn the_law_can_never_push_downward() {
    for v in [8.0_f32, 2.0, 0.5, 0.0, -0.5, -1.0, -1.99] {
        let m = descent_motor(glide_only(true), v, UP);
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
/// ⚠️ **`>=` e não `>`**: em `rel_up == −ceiling` o `delta` seria exatamente
/// zero, e um `boost` de zero não é o mesmo que silêncio — ele conta como *"esta
/// lei escreveu este eixo"* para quem somar motores, e o módulo tem laws que se
/// calam justamente para não serem dois donos do mesmo número.
#[test]
fn the_boundary_is_the_ceiling_and_it_is_closed_from_above() {
    assert_eq!(
        descent_motor(glide_only(true), -2.0, UP),
        Motor::default(),
        "exatamente NO teto a lei nao age"
    );
    assert_ne!(
        descent_motor(glide_only(true), -2.001, UP),
        Motor::default(),
        "e um fio abaixo dele, age"
    );
}

/// **O teto é o número autorado** — dois valores, duas descidas.
#[test]
fn the_ceiling_is_the_number_the_artist_wrote() {
    for speed in [1.0_f32, 2.0, 4.0] {
        let m = descent_motor(Some(speed), -20.0, UP);
        let landed = -20.0 + m.boost[1];
        assert!(
            (landed + speed).abs() < 1.0e-4,
            "com o teto {speed} a descida tem de ficar em {}: ficou {landed}",
            -speed
        );
    }
}

/// **O eixo de CIMA é o do suporte, não o `Y` do mundo.**
///
/// ⚠️ Sem isto o freio numa gravidade lateral travaria o eixo errado — e o
/// módulo inteiro carrega o `up` por essa razão.
#[test]
fn the_brake_runs_along_the_support_axis() {
    let side: Vec2 = [1.0, 0.0];
    let m = descent_motor(glide_only(true), -12.0, side);
    assert!((m.boost[0] - 10.0).abs() < 1.0e-4, "{:?}", m.boost);
    assert!(m.boost[1].abs() < 1.0e-4, "{:?}", m.boost);
}

/// **⚠️ O GATE DA WAVE D: com as duas leis vivas, vence a MENOR.**
///
/// E ele afirma o número nos DOIS sentidos, porque um `max` acidental passaria
/// por metade de um gate que só medisse um deles. A consequência de errar não é
/// cosmética: o planeio ganharia o poder de **acelerar** uma queda que o teto já
/// tinha limitado, que é exactamente o que a porta única existe para impedir.
#[test]
fn when_both_are_live_the_smallest_ceiling_wins() {
    // O planeio é mais forte que o teto ⇒ manda o planeio.
    assert_eq!(
        descent_ceiling(&glide_at(2.0), &cap_at(20.0), true),
        Some(2.0),
        "segurar o botao tem de travar MAIS que a velocidade terminal"
    );
    // O teto é mais forte que o planeio ⇒ manda o teto.
    assert_eq!(
        descent_ceiling(&glide_at(30.0), &cap_at(8.0), true),
        Some(8.0),
        "um planeio mais frouxo que o teto NUNCA pode soltar a queda"
    );
    // E soltar o dedo devolve a queda ao teto, nunca ao ilimitado.
    assert_eq!(
        descent_ceiling(&glide_at(2.0), &cap_at(20.0), false),
        Some(20.0),
        "sem o dedo sobra o teto de queda, e ele continua vivo"
    );
}

/// **O teto de queda é uma velocidade terminal de verdade: a descida ASSENTA.**
///
/// ⚠️ **O oráculo é a FORMA da sequência, não um valor** — ver o topo do
/// [`super`]: o freio é aplicado no topo do tique e a gravidade soma dentro
/// dele, então a descida assenta uns 6% acima do número autorado. Um gate de
/// igualdade exata nasceria vermelho sobre produto correto; o que o jogador vê,
/// e o que esta wave promete, é a descida **parar de crescer**.
#[test]
fn a_capped_fall_stops_growing_and_an_uncapped_one_does_not() {
    const DT: f32 = 1.0 / 60.0;
    const G: f32 = 9.81;
    let mut with_cap = 0.0_f32;
    let mut without = 0.0_f32;
    let ceiling = descent_ceiling(&GlideConfig::STARTING_POINT, &cap_at(10.0), false);
    for _ in 0..600 {
        with_cap += descent_motor(ceiling, with_cap, UP).boost[1] - G * DT;
        without += descent_motor(None, without, UP).boost[1] - G * DT;
    }
    assert!(
        with_cap < -10.0 && with_cap > -11.0,
        "a queda capada tem de assentar perto do teto de 10: {with_cap}"
    );
    assert!(
        without < -90.0,
        "o CONTROLE sem teto tem de continuar a acelerar: {without}"
    );
}
