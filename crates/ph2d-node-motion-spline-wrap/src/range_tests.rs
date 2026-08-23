//! **O TRECHO DA CURVA — `from`/`to`** (doc 89 folha 04 — o último P0 da família).
//!
//! A célula: *"**From / To** — C4D Spline Wrap: 'Define the spline region where
//! deformations occur, expressed as percentages'. Temos `offset` (desliza) e
//! **nenhuma extensão**"*, com o veredito **NÃO** — *"nada no catálogo re-escala o
//! mapeamento `x → u ∈ [0,1]`; `field.*` mascara a mistura, não a
//! parametrização"*.
//!
//! ⚠️ A refutação foi RECONFERIDA contra o código antes de uma linha ser escrita, e
//! ela é mais forte do que a folha diz: o `u` é `(x − xmin) / w` com `xmin`/`w`
//! derivados do PRÓPRIO stream, então a normalização é **invariante a qualquer
//! afim em x** — um `motion.transform` a montante escala a geometria E os
//! extremos, e o `u` não se move. O trecho não era só *não-exposto*, era
//! **inalcançável de fora**.
//!
//! O oráculo destes gates é uma RETA de comprimento 3: numa reta parametrizada por
//! arco a posição `s` pousa em `x = 3s`, por aritmética — sem chamar o `frame_at`,
//! que é a função que os gates da curva já cobrem.

use super::*;

/// Uma reta de (0,0) a (3,0) com controles uniformes: `|B'(t)|` é constante, logo
/// arco ∝ t e a posição de arco `s` cai em `x = 3s`.
const RULER: [P2; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];

/// Três elementos em `y = 0`, cobrindo `u = 0, 0.5, 1`.
fn row() -> Vec<P2> {
    vec![[-1.0, 0.0], [0.0, 0.0], [1.0, 0.0]]
}

/// Os `x` sobre a régua, que com `height_scale = 0` e `y = 0` são exactamente
/// `3 · s_at(u)`.
fn xs(from: f32, to: f32, offset: f32) -> Vec<f32> {
    let map = ArcMap { from, to, offset };
    wrap(&row(), &Curve::cubic(&RULER), 0.0, map, false, &[1.0], &[])
        .iter()
        .map(|p| p[0])
        .collect()
}

/// **O default é o nó que shipava, AO BIT** — e a identidade é medida, não afirmada.
///
/// ⚠️ Este é o gate que impede alguém de reescrever o `s_at` numa forma
/// algebricamente igual e numericamente diferente (um lerp centrado, um
/// `from*(1−u) + to*u` com arredondamento próprio). O desenho todo se apoia em
/// `0.0 + u * 1.0` ser `u` **ao bit**, e isso vale porque `u` nunca é negativo:
/// ele é `(x − xmin) / w` com `xmin` o mínimo e `w > EPS`.
#[test]
fn the_whole_curve_reduces_to_the_expression_that_shipped_bit_for_bit() {
    for i in 0..=200u32 {
        let u = f32::from(u16::try_from(i).expect("small")) / 200.0;
        for offset in [-1.0f32, -0.37, 0.0, 0.13, 1.0] {
            // A expressão que shipava, escrita aqui para o gate dizer o que guarda.
            let shipped = (u + offset).clamp(0.0, 1.0);
            let now = ArcMap {
                from: 0.0,
                to: 1.0,
                offset,
            }
            .s_at(u);
            assert_eq!(
                now.to_bits(),
                shipped.to_bits(),
                "u={u} offset={offset}: {now} contra {shipped}"
            );
        }
    }
}

/// **O layout ocupa o TRECHO pedido da curva** — o começo do layout pousa em
/// `from`, o fim em `to`, e é isso que o C4D chama de spline region.
#[test]
fn the_layout_lands_on_the_slice_of_curve_it_was_given() {
    // Curva inteira: os três elementos varrem 0 -> 3.
    let full = xs(0.0, 1.0, 0.0);
    assert!((full[0] - 0.0).abs() < 2e-2, "{full:?}");
    assert!((full[2] - 3.0).abs() < 2e-2, "{full:?}");

    // O quarto do meio: 0.25 -> 0.75 do arco, ou seja x de 0.75 a 2.25.
    let mid = xs(0.25, 0.75, 0.0);
    assert!((mid[0] - 0.75).abs() < 2e-2, "comeco em from: {mid:?}");
    assert!((mid[1] - 1.50).abs() < 2e-2, "meio no meio: {mid:?}");
    assert!((mid[2] - 2.25).abs() < 2e-2, "fim em to: {mid:?}");
}

/// **O WRITE-ON: crescer o `to` abre o layout ao longo da curva.**
///
/// É a razão de a folha marcar isto P0 — *"a animação de REVELAÇÃO pela qual este
/// nó existe"*. Com `to` colado no `from` os elementos saem empilhados no começo
/// do trecho; conforme ele cresce, eles se abrem, monotonicamente.
#[test]
fn growing_the_end_of_the_slice_unfolds_the_layout_along_the_curve() {
    let mut prev = -1.0f32;
    for step in 0..=10u32 {
        let to = f32::from(u16::try_from(step).expect("small")) / 10.0;
        let x = xs(0.0, to, 0.0);
        let span = x[2] - x[0];
        assert!(
            span >= prev - 1e-4,
            "o vao so cresce: to={to} span={span} contra {prev}"
        );
        prev = span;
    }
    // E as duas pontas do intervalo dizem o que a animação faz.
    let closed = xs(0.0, 0.0, 0.0);
    assert!(
        (closed[2] - closed[0]).abs() < 1e-4,
        "to == from empilha tudo no comeco: {closed:?}"
    );
    let open = xs(0.0, 1.0, 0.0);
    assert!(
        (open[2] - open[0]) > 2.9,
        "e a curva inteira abre o layout todo: {open:?}"
    );
}

/// **`from > to` percorre a curva ao CONTRÁRIO** — sai de graça da mesma
/// aritmética, e é uma direção, não um caso degenerado a recusar.
#[test]
fn a_backwards_slice_runs_the_layout_the_other_way() {
    let fwd = xs(0.0, 1.0, 0.0);
    let back = xs(1.0, 0.0, 0.0);
    assert!(fwd[0] < fwd[2], "para a frente: {fwd:?}");
    assert!(back[0] > back[2], "e para tras: {back:?}");
    // É o espelho exacto, não um percurso qualquer.
    for (f, b) in fwd.iter().zip(back.iter().rev()) {
        assert!((f - b).abs() < 2e-2, "{fwd:?} contra {back:?}");
    }
}

/// **O `offset` continua deslizando, e desliza o TRECHO inteiro** — ele age no
/// arco (absoluto), não dentro da janela, que é o que o mantém byte-idêntico ao
/// que shipava quando o trecho é a curva inteira.
#[test]
fn the_offset_slides_the_slice_along_the_arc() {
    let at_rest = xs(0.25, 0.5, 0.0);
    let slid = xs(0.25, 0.5, 0.25);
    for (a, b) in at_rest.iter().zip(&slid) {
        // 0.25 de arco sobre uma regua de comprimento 3.
        assert!(
            (b - a - 0.75).abs() < 2e-2,
            "o trecho inteiro anda 0.75: {at_rest:?} -> {slid:?}"
        );
    }
}

/// **Fora de `[0, 1]` o mapeamento SATURA em vez de sair da curva** — é por isso
/// que a faixa `0..1` do slider é a faixa honrada, e por isso a caixa de texto não
/// ganha um `ParamHardMax` que a alargue para um número que o `clamp` desmente.
#[test]
fn a_slice_outside_the_curve_saturates_at_its_ends() {
    let over = xs(0.5, 3.0, 0.0);
    assert!(
        (over[2] - 3.0).abs() < 2e-2,
        "um `to` de 3.0 para no fim da curva: {over:?}"
    );
    let under = xs(-2.0, 0.5, 0.0);
    assert!(
        (under[0] - 0.0).abs() < 2e-2,
        "e um `from` negativo para no comeco: {under:?}"
    );
}

/// **E o trecho vale na curva de verdade, não só na régua** — o oráculo reto é
/// exacto e por isso é o principal, mas um gate que só o usasse ficaria verde
/// sobre um mapeamento que a curvatura quebrasse.
#[test]
fn the_slice_holds_on_a_real_curve_too() {
    const S: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
    let map = |from: f32, to: f32| {
        wrap(
            &row(),
            &Curve::cubic(&S),
            0.0,
            ArcMap {
                from,
                to,
                offset: 0.0,
            },
            false,
            &[1.0],
            &[],
        )
    };
    let full = map(0.0, 1.0);
    let half = map(0.0, 0.5);
    // O comeco é o MESMO ponto (os dois trechos partem de `from = 0`)...
    assert!(
        (full[0][0] - half[0][0]).abs() < 1e-3 && (full[0][1] - half[0][1]).abs() < 1e-3,
        "{full:?} contra {half:?}"
    );
    // ...e o fim do meio-trecho é o MEIO do trecho inteiro.
    assert!(
        (half[2][0] - full[1][0]).abs() < 2e-2 && (half[2][1] - full[1][1]).abs() < 2e-2,
        "o fim de [0, 0.5] é o meio de [0, 1]: {half:?} contra {full:?}"
    );
}
