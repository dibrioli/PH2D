//! Gates da [`super::welds`] — **um por regra**, porque as defesas em camadas precisam de
//! gate por camada ([[feedback_layered_defenses_need_per_layer_gates]]).

use super::welds;
use ph2d_core::Vec2;

/// Dois traços em L cujas pontas ficam a `gap` uma da outra, com meia-largura `r`.
fn corner(gap: f32, r: f32) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    vec![
        (
            vec![Vec2::new(-1.0, 0.0), Vec2::new(0.0, 0.0)],
            vec![r; 2],
            false,
        ),
        (
            vec![Vec2::new(gap, 0.0), Vec2::new(gap, 1.0)],
            vec![r; 2],
            false,
        ),
    ]
}

/// 🔴 **A junta que a tinta cobre é soldada.** O corpo pintado (2 × 0,13) engole o vão de
/// 0,04 ⇒ na tela a quina está fechada, e a parede tem de estar também.
#[test]
fn a_joint_the_paint_covers_is_welded() {
    let w = welds(&corner(0.04, 0.13));
    assert!(!w.is_empty(), "a quina coberta pela tinta tem de soldar");
    // A solda liga a ponta ao ponto mais próximo do vizinho — comprimento = o próprio vão.
    let len = (w[0].1 - w[0].0).length();
    assert!(
        (len - 0.04).abs() < 1e-4,
        "a solda tem o comprimento do vão ({len})"
    );
}

/// 🔴 **Um vão DELIBERADO não é tocado.** O do smoke tem 1,2 doc contra 0,26 de tinta — o
/// artista o desenhou aberto, e fechá-lo seria a ferramenta desfazendo a intenção.
#[test]
fn a_deliberate_gap_is_left_open() {
    assert!(
        welds(&corner(1.2, 0.13)).is_empty(),
        "vão maior que a tinta que o cobriria é DELIBERADO"
    );
}

/// 🔴 **A regra é a soma das meias-larguras, e ela é MONOTÔNICA na largura**: o mesmo vão que
/// um traço fino deixa aberto, um traço grosso fecha — porque o grosso de fato o cobre.
#[test]
fn the_rule_is_the_painted_width_not_a_constant() {
    assert!(welds(&corner(0.5, 0.05)).is_empty(), "fino: vão aberto");
    assert!(!welds(&corner(0.5, 0.30)).is_empty(), "grosso: vão coberto");
}

/// 🔴 **Um traço de largura ZERO nunca solda.** Sem corpo pintado não há nada cobrindo o vão,
/// e soldar seria inventar parede — as fixtures antigas (largura 0) ficam byte-idênticas.
#[test]
fn a_zero_width_stroke_never_welds() {
    assert!(welds(&corner(0.001, 0.0)).is_empty());
}

/// 🔴 **O círculo que quase fecha solda em si mesmo.** É UM traço aberto cujas duas pontas se
/// encontram — a exclusão da vizinhança da ponta é por comprimento de ARCO, então a volta
/// inteira é candidata e o índice adjacente não.
#[test]
fn a_stroke_that_almost_closes_on_itself_is_welded() {
    // Um quadrado percorrido como polilinha única, faltando 0,05 para fechar.
    let pts = vec![
        Vec2::new(0.05, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0),
    ];
    let n = pts.len();
    let w = welds(&[(pts, vec![0.13; n], false)]);
    assert_eq!(
        w.len(),
        2,
        "as duas pontas se enxergam (uma solda por ponta)"
    );
    for (a, b) in w {
        assert!(
            (a - b).length() < 0.06,
            "a solda fecha a volta, não corta o quadrado"
        );
    }
}

/// 🔴 **Um traço FECHADO não tem ponta** — ele só participa como ALVO. O índice 0 de um anel
/// é artefato de PARAMETRIZAÇÃO (onde o traçador começou a listar), não um lugar da arte:
/// soldar dali penduraria uma parede num ponto que o artista não desenhou, e duas cópias da
/// mesma forma listadas de pontos diferentes soldariam em lugares diferentes.
///
/// ⚠️ A 1ª fixture deste gate era um anel SOZINHO, e ficava verde com a regra removida — as
/// duas pontas de um anel são vizinhas pelo fechamento e a exclusão por arco já as descarta.
/// Um gate que não consegue exprimir a própria regra não a testa: aqui há um traço vizinho
/// bem embaixo do vértice 0, então sem a regra nasce uma solda.
#[test]
fn a_closed_stroke_has_no_loose_end() {
    let r = 0.13;
    let ring = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];
    // Um traço aberto passando POR BAIXO do vértice 0 do anel, a 0,05 dele — bem dentro do
    // alcance dos dois corpos (0,26). Sem a regra, o vértice 0 vira ponta e solda nele.
    let bar = vec![Vec2::new(-1.0, -0.05), Vec2::new(0.4, -0.05)];
    let w = welds(&[(ring, vec![r; 4], true), (bar, vec![r; 2], false)]);
    assert!(
        !w.is_empty(),
        "controle positivo: a ponta do BAR alcança o anel"
    );
    assert!(
        w.iter().all(|(a, _)| (a.y + 0.05).abs() < 1e-6),
        "toda solda tem de sair de uma PONTA do traço aberto (y = -0,05), nunca de um \
         vértice do anel fechado: {w:?}"
    );
}

/// 🔴 **Uma junta em T tripla solda nos DOIS vizinhos.** Uma solda por traço-alvo (a mais
/// próxima), nunca duas no mesmo — a tela mostra os três unidos, e a parede tem de mostrar.
#[test]
fn a_triple_junction_welds_to_both_neighbours() {
    let r = 0.13;
    let strokes = vec![
        // A ponta solta, no meio, tocando os dois braços pelo corpo.
        (
            vec![Vec2::new(0.0, -1.0), Vec2::new(0.0, -0.05)],
            vec![r; 2],
            false,
        ),
        (
            vec![Vec2::new(-1.0, 0.05), Vec2::new(-0.05, 0.05)],
            vec![r; 2],
            false,
        ),
        (
            vec![Vec2::new(0.05, 0.05), Vec2::new(1.0, 0.05)],
            vec![r; 2],
            false,
        ),
    ];
    let w = welds(&strokes);
    let from_tip = w
        .iter()
        .filter(|(a, _)| (*a - Vec2::new(0.0, -0.05)).length() < 1e-4)
        .count();
    assert_eq!(from_tip, 2, "a ponta do meio solda nos dois braços");
}
