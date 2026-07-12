//! **Vocabulário de fluxograma** (ANSI/ISO 5807) — módulo irmão de [`crate::shapes`].
//!
//! É o alfabeto de qualquer app de diagrama: cada símbolo tem um SIGNIFICADO fixo
//! (losango = decisão, pílula = início/fim, paralelogramo = dados, cilindro = base de
//! dados…), e é por isso que eles valem como formas próprias em vez de "um polígono que
//! você deforma": o leitor do diagrama reconhece a silhueta.
//!
//! **Autorado no espaço unitário** ([`crate::space`]): `v = 0` é o TOPO, como na
//! referência. A inversão para o mundo (Y-para-cima) acontece uma vez, em [`Unit::p`] — é
//! o que impede o paralelogramo de nascer inclinado para o lado errado e o cilindro com a
//! barriga em cima, que foi o que aconteceu quando estes símbolos eram escritos direto em
//! coordenadas de mundo.
//!
//! Todos cabem na caixa do gesto e usam **frações** dela nos parâmetros, para guardarem a
//! proporção ao redimensionar.

use crate::VecPath;
use crate::space::{Unit, Uv, add_sub, closed, open, poly, straighten};

/// **Decisão** — o losango. Quatro pontos nos meios das bordas.
#[must_use]
pub fn diamond(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let u = Unit::of(a, b);
    poly(&u, &[(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)])
}

/// **Início / fim** — a pílula (stadium): um retângulo cujas pontas são semicírculos.
/// Não é um round-rect com raio grande: o raio é SEMPRE metade do lado menor, então a
/// silhueta é estável em qualquer proporção (é o que a torna reconhecível).
#[must_use]
pub fn pill(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let (hw, hh) = ((b[0] - a[0]).abs() * 0.5, (b[1] - a[1]).abs() * 0.5);
    crate::rounded_rect(a, b, hw.min(hh))
}

/// **Dados / entrada-saída** — o paralelogramo, com o topo deslocado para a DIREITA
/// (a inclinação canônica do símbolo de dados). `slant` = quanto a base desliza (fração
/// da largura).
#[must_use]
pub fn parallelogram(a: [f64; 2], b: [f64; 2], slant: f64) -> VecPath {
    let u = Unit::of(a, b);
    let s = slant.clamp(0.0, 0.9);
    poly(&u, &[(s, 0.0), (1.0, 0.0), (1.0 - s, 1.0), (0.0, 1.0)])
}

/// **Trapézio** — topo mais estreito (o símbolo de "operação manual"). `flip` troca qual
/// base é a curta, o que no vocabulário ANSI vira "entrada manual".
#[must_use]
pub fn trapezoid(a: [f64; 2], b: [f64; 2], slant: f64, flip: bool) -> VecPath {
    let u = Unit::of(a, b);
    let s = slant.clamp(0.0, 0.49);
    let (top, bot) = if flip { (0.0, s) } else { (s, 0.0) };
    poly(
        &u,
        &[(top, 0.0), (1.0 - top, 0.0), (1.0 - bot, 1.0), (bot, 1.0)],
    )
}

/// **Hexágono achatado** — "preparação" no fluxograma. `cut` = o recuo das pontas.
#[must_use]
pub fn hexagon_flat(a: [f64; 2], b: [f64; 2], cut: f64) -> VecPath {
    let u = Unit::of(a, b);
    let c = cut.clamp(0.02, 0.49);
    poly(
        &u,
        &[
            (c, 0.0),
            (1.0 - c, 0.0),
            (1.0, 0.5),
            (1.0 - c, 1.0),
            (c, 1.0),
            (0.0, 0.5),
        ],
    )
}

/// **Base de dados** — o cilindro. A silhueta é: meia-elipse de TRÁS no topo → lado
/// direito → meia-elipse da FRENTE na base → lado esquerdo; e a tampa aparece porque a
/// meia-elipse da FRENTE do topo entra como contorno próprio.
///
/// `lip` = a altura CHEIA da elipse (fração da altura da caixa). Clampada em `0.45`: com
/// meia elipse maior que isso a tampa encostaria na barriga e o cilindro deixaria de
/// existir.
#[must_use]
pub fn cylinder(a: [f64; 2], b: [f64; 2], lip: f64) -> VecPath {
    let u = Unit::of(a, b);
    let e = lip.clamp(0.05, 0.45) * 0.5; // semi-altura da elipse, em unidades de v
    let (top, bot): (Uv, Uv) = ((0.5, e), (0.5, 1.0 - e));

    // De trás no topo: da esquerda (180°) até a direita (360°), passando pelo topo (270°).
    let mut verts = u.arc(top, 0.5, e, 180.0, 180.0);
    let seam = verts.len() - 1;
    // Da frente na base: da direita (0°) até a esquerda (180°), passando pela base (90°).
    verts.extend(u.arc(bot, 0.5, e, 0.0, 180.0));
    straighten(&mut verts, seam); // o lado direito é RETO, não a continuação do arco

    let mut p = closed(verts);
    // A tampa: a meia-elipse da FRENTE do topo. É ela que faz o cilindro parecer um
    // cilindro (sem ela, a silhueta é um retângulo de cantos curvos).
    add_sub(&mut p, u.arc(top, 0.5, e, 0.0, 180.0), false);
    p
}

/// **Documento** — retângulo com a base ONDULADA (a folha de papel). A onda é **uma
/// cúbica só**, com a forma do preset `flowChartDocument` do ECMA-376 (a especificação
/// pública do OOXML) renormalizada para a caixa unitária: a direita sai reta, a curva
/// mergulha e a esquerda termina mais baixa — a assimetria é o que faz parecer papel, e
/// não um "til".
///
/// `wave` = a altura da faixa que a onda ocupa (fração da altura). O pico da cúbica
/// encosta EXATAMENTE na base da caixa (a janela `WAVE_PEAK − WAVE_R` é o máximo real da
/// curva, não o do handle) — é o que mantém a bbox honesta.
#[must_use]
pub fn document(a: [f64; 2], b: [f64; 2], wave: f64) -> VecPath {
    /// Onde a onda começa (direita), no espaço do preset.
    const WAVE_R: f64 = 0.802;
    /// Onde ela termina (esquerda) — mais baixa que a direita.
    const WAVE_L: f64 = 0.9339;
    /// O 2º controle da cúbica; passa da base do preset (é handle, não ponto da curva).
    const WAVE_C2: f64 = 1.1075;
    /// O MÁXIMO real da cúbica (avaliado, não o do handle) — a janela de normalização.
    const WAVE_PEAK: f64 = 0.986_85;

    let u = Unit::of(a, b);
    let w = wave.clamp(0.05, 0.4);
    let span = WAVE_PEAK - WAVE_R;
    let f = |y: f64| 1.0 - w + w * (y - WAVE_R) / span;

    let mut verts = vec![
        u.corner((0.0, 0.0)),
        u.corner((1.0, 0.0)),
        // A onda: sai da direita com tangente horizontal e mergulha.
        u.smooth((1.0, f(WAVE_R)), (1.0, f(WAVE_R)), (0.5, f(WAVE_R))),
        u.smooth((0.0, f(WAVE_L)), (0.5, f(WAVE_C2)), (0.0, f(WAVE_L))),
    ];
    straighten(&mut verts, 0); // topo reto
    closed(verts)
}

/// **Delay** — o "D": retângulo com o lado direito em meia-elipse. A tampa é CIRCULAR no
/// mundo (raio = metade da altura), não elíptica — por isso o raio em `u` carrega o
/// `aspect` da caixa; e é clampada em meia caixa, senão numa caixa alta a tampa engoliria
/// o retângulo inteiro.
#[must_use]
pub fn delay(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let u = Unit::of(a, b);
    let ru = (0.5 * u.aspect()).min(0.5);
    let x = 1.0 - ru;

    // Do topo (270°) até a base (90°), passando pela direita (0°).
    let mut verts = vec![u.corner((0.0, 0.0))];
    let seam = verts.len() - 1;
    verts.extend(u.arc((x, 0.5), ru, 0.5, -90.0, 180.0));
    verts.push(u.corner((0.0, 1.0)));
    straighten(&mut verts, seam); // o topo é reto até onde a tampa começa
    let last = verts.len() - 2;
    straighten(&mut verts, last); // e a base também
    closed(verts)
}

/// **Display** — a "tela": bico à esquerda, lado direito arredondado.
#[must_use]
pub fn display(a: [f64; 2], b: [f64; 2], nose: f64) -> VecPath {
    let u = Unit::of(a, b);
    let n = nose.clamp(0.02, 0.45);
    let x = 1.0 - n;

    let mut verts = vec![u.corner((n, 0.0))];
    let seam = verts.len() - 1;
    verts.extend(u.arc((x, 0.5), n, 0.5, -90.0, 180.0));
    verts.push(u.corner((n, 1.0)));
    verts.push(u.corner((0.0, 0.5))); // o bico
    straighten(&mut verts, seam);
    let last = verts.len() - 3;
    straighten(&mut verts, last);
    closed(verts)
}

/// **Processo predefinido** — retângulo com duas barras verticais (a "sub-rotina").
/// As barras entram como contornos ABERTOS próprios: assim o símbolo desenha certo tanto
/// preenchido quanto em traço.
#[must_use]
pub fn predefined_process(a: [f64; 2], b: [f64; 2], bar: f64) -> VecPath {
    let u = Unit::of(a, b);
    let d = bar.clamp(0.02, 0.4);
    let mut p = poly(&u, &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    for x in [d, 1.0 - d] {
        add_sub(&mut p, vec![u.corner((x, 0.0)), u.corner((x, 1.0))], false);
    }
    p
}

/// **Conector fora-de-página** — o pentágono com o bico para BAIXO. `tip` = a profundidade
/// do bico (fração da altura).
#[must_use]
pub fn offpage(a: [f64; 2], b: [f64; 2], tip: f64) -> VecPath {
    let u = Unit::of(a, b);
    let t = tip.clamp(0.05, 0.9);
    poly(
        &u,
        &[
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0 - t),
            (0.5, 1.0),
            (0.0, 1.0 - t),
        ],
    )
}

/// **Junção / somatório** — o círculo com uma cruz. Compound: o círculo + as duas linhas.
#[must_use]
pub fn junction(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let u = Unit::of(a, b);
    let mut p = closed(u.ellipse((0.5, 0.5), 0.5, 0.5));
    add_sub(
        &mut p,
        vec![u.corner((0.0, 0.5)), u.corner((1.0, 0.5))],
        false,
    );
    add_sub(
        &mut p,
        vec![u.corner((0.5, 0.0)), u.corner((0.5, 1.0))],
        false,
    );
    p
}

/// **Nota / anotação** — o colchete: um "[" que abraça o conteúdo (ABERTO, é um traço).
#[must_use]
pub fn note_bracket(a: [f64; 2], b: [f64; 2], lip: f64) -> VecPath {
    let u = Unit::of(a, b);
    let l = lip.clamp(0.05, 1.0);
    open(vec![
        u.corner((l, 0.0)),
        u.corner((0.0, 0.0)),
        u.corner((0.0, 1.0)),
        u.corner((l, 1.0)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [f64; 2] = [-2.0, -1.0];
    const B: [f64; 2] = [2.0, 1.0];

    /// Todas as formas de fluxo cabem na caixa do gesto — se uma extravasar, a bbox
    /// mente e o gizmo desalinha do desenho.
    #[test]
    fn every_flow_symbol_fits_inside_the_gesture_box() {
        let all = [
            ("diamond", diamond(A, B)),
            ("pill", pill(A, B)),
            ("parallelogram", parallelogram(A, B, 0.2)),
            ("trapezoid", trapezoid(A, B, 0.2, false)),
            ("hexagon", hexagon_flat(A, B, 0.2)),
            ("cylinder", cylinder(A, B, 0.3)),
            ("document", document(A, B, 0.18)),
            ("delay", delay(A, B)),
            ("display", display(A, B, 0.2)),
            ("predefined", predefined_process(A, B, 0.12)),
            ("offpage", offpage(A, B, 0.3)),
            ("junction", junction(A, B)),
            ("note", note_bracket(A, B, 0.15)),
        ];
        for (name, p) in all {
            assert!(!p.verts.is_empty(), "{name}: vazio");
            for v in p.verts_all() {
                assert!(
                    v.anchor[0] >= -2.0 - 1e-6
                        && v.anchor[0] <= 2.0 + 1e-6
                        && v.anchor[1] >= -1.0 - 1e-6
                        && v.anchor[1] <= 1.0 + 1e-6,
                    "{name}: ancora {:?} fora da caixa",
                    v.anchor
                );
            }
        }
    }

    /// **A orientação, executável.** Estas quatro asserções são a definição de "de pé" —
    /// cada uma delas caiu de verdade quando os símbolos eram autorados em coordenadas de
    /// mundo (tudo nascia espelhado na vertical). No mundo Y-para-cima, `y` maior é mais
    /// ALTO.
    #[test]
    fn the_asymmetric_flow_symbols_are_right_side_up() {
        // O bico do conector fora-de-página aponta para BAIXO.
        let off = offpage(A, B, 0.3);
        let tip = off
            .verts
            .iter()
            .map(|v| v.anchor[1])
            .fold(f64::MAX, f64::min);
        let tip_x = off
            .verts
            .iter()
            .find(|v| (v.anchor[1] - tip).abs() < 1e-9)
            .expect("o bico existe")
            .anchor[0];
        assert!(tip_x.abs() < 1e-9, "o bico e o ponto mais BAIXO, no meio");

        // O trapézio (operação manual) tem o TOPO mais curto que a base.
        let tz = trapezoid(A, B, 0.25, false);
        let width_at = |up: bool| {
            let ys: Vec<&crate::VecVertex> = tz
                .verts
                .iter()
                .filter(|v| (v.anchor[1] > 0.0) == up)
                .collect();
            let lo = ys.iter().map(|v| v.anchor[0]).fold(f64::MAX, f64::min);
            let hi = ys.iter().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
            hi - lo
        };
        assert!(
            width_at(true) < width_at(false),
            "o topo do trapezio e o lado CURTO: {} vs {}",
            width_at(true),
            width_at(false)
        );

        // O cilindro tem a barriga EMBAIXO: o ponto mais baixo do contorno principal fica
        // no meio (a meia-elipse da frente), não numa quina.
        let cy = cylinder(A, B, 0.3);
        let low = cy
            .verts
            .iter()
            .min_by(|p, q| p.anchor[1].total_cmp(&q.anchor[1]))
            .expect("tem vertices");
        assert!(
            low.anchor[0].abs() < 1e-6,
            "a barriga do cilindro e embaixo, no meio: {:?}",
            low.anchor
        );

        // A onda do documento fica na BASE: o ponto mais baixo não é uma das quinas de
        // cima, e o topo é reto (dois vértices na altura máxima).
        let doc = document(A, B, 0.18);
        let top_count = doc
            .verts
            .iter()
            .filter(|v| (v.anchor[1] - 1.0).abs() < 1e-9)
            .count();
        assert_eq!(top_count, 2, "o topo do documento e RETO (duas quinas)");
    }

    /// O paralelogramo de dados inclina para a DIREITA: o topo é deslocado no sentido
    /// positivo de x em relação à base.
    #[test]
    fn the_data_parallelogram_leans_right() {
        let p = parallelogram(A, B, 0.25);
        let top_left = p
            .verts
            .iter()
            .filter(|v| v.anchor[1] > 0.0)
            .map(|v| v.anchor[0])
            .fold(f64::MAX, f64::min);
        let bot_left = p
            .verts
            .iter()
            .filter(|v| v.anchor[1] < 0.0)
            .map(|v| v.anchor[0])
            .fold(f64::MAX, f64::min);
        assert!(
            top_left > bot_left,
            "o topo tem de estar deslocado para a direita: {top_left} vs {bot_left}"
        );
    }

    /// O losango toca os QUATRO meios das bordas — é o que o torna um losango e não um
    /// quadrado girado (que não caberia na caixa).
    #[test]
    fn the_diamond_touches_the_four_edge_midpoints() {
        let p = diamond(A, B);
        let want = [[0.0, 1.0], [2.0, 0.0], [0.0, -1.0], [-2.0, 0.0]];
        for (v, w) in p.verts.iter().zip(want) {
            assert_eq!(v.anchor, w);
        }
    }

    /// A pílula tem o raio SEMPRE em metade do lado menor — a silhueta não muda com a
    /// proporção (é o que a distingue de um round-rect qualquer).
    #[test]
    fn the_pill_radius_is_always_half_the_short_side() {
        let wide = pill([-4.0, -1.0], [4.0, 1.0]);
        let ys: Vec<f64> = wide.verts.iter().map(|v| v.anchor[1]).collect();
        assert!(ys.iter().any(|y| (*y - 1.0).abs() < 1e-9));
        assert!(ys.iter().any(|y| (*y + 1.0).abs() < 1e-9));
    }

    /// O processo predefinido e a junção são COMPOUND: as barras / a cruz são contornos
    /// próprios. Sem isso o símbolo seria um retângulo liso e perderia o significado. O
    /// cilindro carrega a tampa pelo mesmo mecanismo.
    #[test]
    fn compound_symbols_carry_their_inner_strokes() {
        assert_eq!(
            predefined_process(A, B, 0.12).subpaths.len(),
            2,
            "duas barras"
        );
        assert_eq!(junction(A, B).subpaths.len(), 2, "a cruz");
        assert_eq!(cylinder(A, B, 0.3).subpaths.len(), 1, "a tampa");
    }
}
