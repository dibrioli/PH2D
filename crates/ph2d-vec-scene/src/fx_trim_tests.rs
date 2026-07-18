//! Gates do Trim. **O oráculo é o ARCO MEDIDO**, não a regra da implementação: pedir 30%
//! do caminho tem de devolver 30% do *comprimento*, e é exatamente aí que a versão ingênua
//! (fatiar por `t`, ou pela poligonal das âncoras) falha — e falha *parecendo certa* numa
//! reta.

use super::*;
use crate::arclen::arclen;
use crate::corner_live::segment;

/// Números do produto.
const R: f64 = 60.0;

/// Um contorno FECHADO e genuinamente curvo: uma elipse em quatro cúbicas. É o fixture que
/// importa — um quadrado esconde o bug (numa reta o `t` É proporcional ao arco).
/// [[reference_topic_fixture_discipline]]
fn circle() -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4; // a constante do círculo em cúbicas
    let p = [[R, 0.0], [0.0, R], [-R, 0.0], [0.0, -R]];
    let tang = [[0.0, K * R], [-K * R, 0.0], [0.0, -K * R], [K * R, 0.0]];
    (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - tang[i][0], p[i][1] - tang[i][1]],
            out_handle: [p[i][0] + tang[i][0], p[i][1] + tang[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect()
}

/// O comprimento de um contorno.
fn len(verts: &[VecVertex], closed: bool) -> f64 {
    let n = verts.len();
    if n < 2 {
        return 0.0;
    }
    let segs = if closed { n } else { n - 1 };
    (0..segs).map(|i| arclen(&segment(verts, i, n))).sum()
}

/// **O ponto neutro é um no-op byte-idêntico.** Todo efeito da pilha deve isto — sem ele,
/// a pilha não pode saltar o efeito e o `Cow::Borrowed` morre.
#[test]
fn the_neutral_point_is_a_byte_identical_no_op() {
    let v = circle();
    let (out, closed) = trim_contour(&v, true, &TrimSpec::default());
    assert_eq!(out, v, "neutro tem de devolver os MESMOS vértices");
    assert!(closed, "e não pode abrir um contorno fechado");
}

/// **Pedir 30% devolve 30% do COMPRIMENTO** — o gate que separa este Trim do ingênuo.
///
/// Numa elipse os quatro segmentos têm o mesmo comprimento, mas dentro de cada um o `t` não
/// é proporcional ao arco. Uma implementação que fatiasse por `t` erraria aqui.
#[test]
fn asking_for_a_fraction_returns_that_fraction_of_the_length() {
    let v = circle();
    let whole = len(&v, true);
    for frac in [0.1, 0.3, 0.5, 0.75] {
        let (out, closed) = trim_contour(
            &v,
            true,
            &TrimSpec {
                start: 0.0,
                end: frac,
                offset: 0.0,
            },
        );
        let got = len(&out, closed);
        let want = whole * frac;
        assert!(
            ((got - want) / whole).abs() < 1e-6,
            "pedi {frac} do caminho ({want}), veio {got}"
        );
    }
}

/// **Um trecho parcial de um contorno fechado sai ABERTO.** Revelar um pedaço de círculo é
/// um arco, não um círculo mais curto.
#[test]
fn a_partial_trim_of_a_closed_contour_opens_it() {
    let (_, closed) = trim_contour(
        &circle(),
        true,
        &TrimSpec {
            start: 0.0,
            end: 0.4,
            offset: 0.0,
        },
    );
    assert!(!closed);
}

/// **A volta inteira mantém o fechado intacto** — o outro extremo do gate acima.
#[test]
fn the_full_range_keeps_the_loop_closed() {
    let v = circle();
    let (out, closed) = trim_contour(
        &v,
        true,
        &TrimSpec {
            start: 0.0,
            end: 1.0,
            offset: 0.25,
        },
    );
    assert!(closed, "cobrir a volta toda não abre a forma");
    assert_eq!(out, v);
}

/// **Span zero é VAZIO, e vazio é uma resposta legítima**: é o primeiro quadro do draw-on.
/// [[feedback_absence_gate_needs_a_presence_sibling]] — o gate de presença é o de cima.
#[test]
fn a_zero_span_reveals_nothing() {
    let (out, _) = trim_contour(
        &circle(),
        true,
        &TrimSpec {
            start: 0.4,
            end: 0.4,
            offset: 0.0,
        },
    );
    assert!(out.is_empty());
}

/// **O `offset` gira o trecho sem mudar o comprimento dele** — num fechado ele dá a volta
/// pela emenda, que é a razão de existir do parâmetro.
#[test]
fn the_offset_rotates_the_window_around_the_seam_without_resizing_it() {
    let v = circle();
    let spec = |offset| TrimSpec {
        start: 0.0,
        end: 0.25,
        offset,
    };
    let (a, ca) = trim_contour(&v, true, &spec(0.0));
    let (b, cb) = trim_contour(&v, true, &spec(0.9)); // atravessa a emenda
    assert!(
        ((len(&a, ca) - len(&b, cb)) / len(&a, ca)).abs() < 1e-6,
        "girar a janela não pode mudar o tamanho dela"
    );
    let apart = (a[0].anchor[0] - b[0].anchor[0]).hypot(a[0].anchor[1] - b[0].anchor[1]);
    assert!(apart > 1.0, "mas tem de mudar ONDE ela começa (moveu {apart})");
}

/// **Num contorno ABERTO o offset desliza e recorta** — não dá a volta, porque não há volta.
#[test]
fn an_open_contour_clips_instead_of_wrapping() {
    let v: Vec<VecVertex> = circle().into_iter().take(3).collect();
    let whole = len(&v, false);
    let (out, closed) = trim_contour(
        &v,
        false,
        &TrimSpec {
            start: 0.0,
            end: 0.5,
            offset: 0.75, // metade da janela cai fora do domínio
        },
    );
    assert!(!closed);
    let got = len(&out, closed);
    assert!(
        got < whole * 0.3,
        "a janela deslizada para fora tem de ser RECORTADA, não enrolada: sobrou {got} de \
         {whole}"
    );
}

/// **O trecho fica SOBRE o caminho original** — o corte é uma sub-cúbica exata, então todo
/// vértice de saída está na curva de entrada. É o gate que pega uma remontagem de handles
/// trocados, que encurta certo e desenha errado.
#[test]
fn every_output_anchor_lies_on_the_original_curve() {
    let v = circle();
    let (out, _) = trim_contour(
        &v,
        true,
        &TrimSpec {
            start: 0.15,
            end: 0.65,
            offset: 0.0,
        },
    );
    assert!(!out.is_empty());
    for w in &out {
        let r = w.anchor[0].hypot(w.anchor[1]);
        assert!(
            (r - R).abs() < 0.05,
            "âncora em {:?} está a {r} do centro, e o círculo tem raio {R}",
            w.anchor
        );
    }
}
