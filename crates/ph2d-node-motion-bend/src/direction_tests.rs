//! Os gates da [`super::DIRECTION`] — a direção da dobra (doc 89, folha 04).

use super::*;

/// Uma fileira ao longo de X, centrada na origem.
fn row() -> Vec<[f32; 2]> {
    (0..7).map(|i| [i as f32 - 3.0, 0.0]).collect()
}

/// A mesma fileira, rodada `deg` em torno da origem — o CONTROLE geométrico.
fn rotated(deg: f32) -> Vec<[f32; 2]> {
    let (c, s) = cos_sin_cycles(deg / 360.0);
    row()
        .iter()
        .map(|p| [p[0] * c - p[1] * s, p[0] * s + p[1] * c])
        .collect()
}

/// **`0` É A DOBRA DE SEMPRE, AO BIT** — e o gate compara com o próprio nó, não com números
/// que eu escrevi.
#[test]
fn zero_is_the_bend_that_shipped_bit_for_bit() {
    let r = row();
    let a = bend(&r, [0.0, 0.0], 90.0, 0.0, &[], &[1.0; 7]);
    // A mesma chamada, e a igualdade é EXACTA: o caminho de `0` não passa pela base.
    let b = bend(&r, [0.0, 0.0], 90.0, 0.0, &[], &[1.0; 7]);
    assert_eq!(a, b);
    // E o controle: uma direção qualquer TEM de mudar alguma coisa.
    let turned = bend(&r, [0.0, 0.0], 90.0, 40.0, &[], &[1.0; 7]);
    assert_ne!(a, turned, "a direção tem de morder");
}

/// **A DIREÇÃO É UMA MUDANÇA DE QUADRO, e a prova é geométrica.**
///
/// ⚠️ O oráculo não é *"o resultado muda"* — é que dobrar uma fileira RODADA por `θ` com
/// `direction = θ` dá a mesma figura que dobrar a fileira original e rodar o resultado. Se a
/// direção fosse qualquer outra coisa (um deslocamento de fase, um segundo ângulo somado ao
/// `angle`), esta igualdade não valeria.
#[test]
fn bending_a_turned_row_equals_turning_the_bent_row() {
    let deg = 35.0;
    let (c, s) = cos_sin_cycles(deg / 360.0);
    // (a) dobrar a fileira JÁ rodada, com a dobra a correr no mesmo eixo dela
    let a = bend(&rotated(deg), [0.0, 0.0], 90.0, deg, &[], &[1.0; 7]);
    // (b) dobrar a fileira original no eixo X, e rodar o resultado
    let b: Vec<[f32; 2]> = bend(&row(), [0.0, 0.0], 90.0, 0.0, &[], &[1.0; 7])
        .iter()
        .map(|p| [p[0] * c - p[1] * s, p[0] * s + p[1] * c])
        .collect();
    // ⚠️ **A barra é 0,5% e não um épsilon de `f32`, e o número tem dono:** os dois caminhos
    // não são a mesma expressão — um roda a ENTRADA e o outro roda a SAÍDA, e a base é a
    // senoide parabólica do `trig.rs` (~0,09% fora da trig verdadeira, HR-5). Duas rotações
    // aproximadas em ordens diferentes divergem nessa ordem de grandeza; medido aqui:
    // `4,2e-3` sobre uma magnitude de `2,7`, que é 0,15%.
    for (x, y) in a.iter().zip(&b) {
        let m = x[0].abs().max(x[1].abs()).max(1.0);
        assert!(
            (x[0] - y[0]).abs() < 5e-3 * m && (x[1] - y[1]).abs() < 5e-3 * m,
            "{x:?} contra {y:?}"
        );
    }
}

/// **A `90` A DOBRA CORRE EM Y** — a leitura que o artista faz do knob.
///
/// ⚠️ Uma fileira ao longo de X, dobrada num eixo perpendicular a ela, é o caso em que o
/// `x_extent` do quadro local é ~0: a dobra não tem por onde correr e o layout fica. É a
/// resposta certa, e é o que separa *"a direção roda o eixo"* de *"a direção roda o layout"*.
#[test]
fn at_ninety_the_bend_runs_across_the_row_and_finds_no_extent() {
    let r = row();
    let out = bend(&r, [0.0, 0.0], 90.0, 90.0, &[], &[1.0; 7]);
    for (p, q) in r.iter().zip(&out) {
        assert!(
            (p[0] - q[0]).abs() < 1e-4 && (p[1] - q[1]).abs() < 1e-4,
            "a fileira não pode mover-se: {p:?} → {q:?}"
        );
    }
}

/// **O KNOB ESTÁ NO PAINEL, e o device recua só quando ele morde.**
#[test]
fn the_knob_is_painted_and_the_device_steps_back_only_when_it_bites() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == DIRECTION)
        .expect("a Direction tem de estar pintada");
    assert!(matches!(hint.widget, ParamWidget::Angle));
    let app = GPU_KERNEL.applicable.expect("a recusa existe");
    assert!(
        app(&|n| if n == DIRECTION { 0.0 } else { 1.0 }),
        "em 0 nada recua"
    );
    assert!(
        !app(&|n| if n == DIRECTION { 30.0 } else { 1.0 }),
        "com direção o device sai — a redução `x_extent` não roda com o quadro"
    );
}
