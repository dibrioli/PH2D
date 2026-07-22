//! Gates do overlay da correção de pares. O que se pina: a cor da linha DIZ a confiança
//! (verde<->vermelho por custo, âmbar para o manual) e a geometria sai em px de TELA (senão o
//! Vello multiplicaria a espessura pelo zoom — a mesma cicatriz do `flip_selection_overlay`).

use super::*;
use ph2d_core::Vec2;
use ph2d_vector::{Affine, Shape};

/// 🔴 **A cor da linha diz a confiança.** Custo 0 = verde (casou certo); no teto de recusa =
/// vermelho (duvidoso); manual (sem custo) = âmbar (escolha do artista, não pontuação).
///
/// Mutação que sangra: devolver sempre `LINK_GOOD` (ignorar o custo) ⇒ o par duvidoso deixa
/// de se anunciar em vermelho e o overlay para de guiar a correção.
#[test]
fn the_link_colour_reads_the_confidence() {
    assert_eq!(link_color(None), LINK_MANUAL, "manual = âmbar");
    assert_eq!(link_color(Some(0.0)), LINK_GOOD, "custo 0 = verde");
    assert_eq!(
        link_color(Some(ph2d_flip::PAIR_REJECT_COST)),
        LINK_BAD,
        "no teto = vermelho"
    );
    // No meio, cada canal está ENTRE os dois extremos (interpolação monotônica).
    let mid = link_color(Some(ph2d_flip::PAIR_REJECT_COST * 0.5));
    for k in 0..3 {
        let (lo, hi) = (LINK_GOOD[k].min(LINK_BAD[k]), LINK_GOOD[k].max(LINK_BAD[k]));
        assert!(
            mid[k] >= lo - 1e-6 && mid[k] <= hi + 1e-6,
            "o canal {k} do meio saiu fora do intervalo verde<->vermelho: {}",
            mid[k]
        );
    }
    // Um custo ACIMA do teto satura em vermelho (clamp), não estoura a cor.
    assert_eq!(
        link_color(Some(ph2d_flip::PAIR_REJECT_COST * 4.0)),
        LINK_BAD
    );
}

/// 🔴 **A geometria do overlay sai em px de TELA** — o afim é aplicado aos PONTOS, e o
/// `stroke` desenha sob `IDENTITY`. Se ela saísse em espaço local, o Vello multiplicaria a
/// espessura pelo zoom e o overlay viraria um borrão (a cicatriz do W7).
///
/// Mutação que sangra: tirar o `aff *` (devolver os pontos crus) ⇒ a bbox deixa de
/// acompanhar a câmera.
#[test]
fn the_overlay_geometry_comes_out_in_screen_pixels() {
    let mut s = ph2d_flip::FlipStroke::new();
    s.push_default(Vec2::new(0.0, 0.0));
    s.push_default(Vec2::new(10.0, 0.0));
    // 20 px de tela por unidade de mundo, origem em (100, 50).
    let aff = Affine::new([20.0, 0.0, 0.0, 20.0, 100.0, 50.0]);
    let bbox = stroke_path(&s, aff).bounding_box();
    assert!(
        (bbox.x0 - 100.0).abs() < 1e-6 && (bbox.x1 - 300.0).abs() < 1e-6,
        "a geometria não saiu em px de tela: {bbox:?}"
    );
}

/// **Um traço FECHADO fecha o contorno; um ABERTO não** — desenhar o segmento de fecho de um
/// traço aberto mostraria uma linha que o artista não fez.
#[test]
fn a_closed_stroke_closes_the_outline_and_an_open_one_does_not() {
    let poly = |closed: bool| {
        let mut s = ph2d_flip::FlipStroke::new();
        for (x, y) in [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)] {
            s.push_default(Vec2::new(x, y));
        }
        s.closed = closed;
        s
    };
    let open = stroke_path(&poly(false), Affine::IDENTITY);
    let closed = stroke_path(&poly(true), Affine::IDENTITY);
    assert!(
        closed.elements().len() > open.elements().len(),
        "o fechado devia ter o segmento de fecho a mais"
    );
}
