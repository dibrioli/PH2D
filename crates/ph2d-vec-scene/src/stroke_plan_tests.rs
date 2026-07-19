//! Testes de [`crate::stroke_plan`] — arquivo irmão.
//!
//! O que se prova aqui é a RECEITA. Ela morava dentro do renderer (`ph2d-vec-render`, módulo
//! `markers`) enquanto desenhar era a única coisa que se fazia com um traço; mudou-se para cá
//! quando o **Outline Stroke** passou a ser o segundo consumidor, e estes gates vieram junto —
//! um gate que fica onde a lei NÃO está mais é um gate sobre nada.
//!
//! O de baixo que mais morde continua sendo o mesmo: **a linha para exatamente nas costas da
//! cabeça**. Recuo e cabeça são a mesma medida vista dos dois lados, e errar não quebra a
//! compilação — quebra o desenho.

use super::*;
use crate::{Marker, Rgba8, VecVertex};

/// Uma linha horizontal, da esquerda para a direita: a ponta do FIM olha para `+x`.
fn line(len: f64) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([len, 0.0])],
        closed: false,
        ..VecPath::default()
    }
}

fn spec(width: f64) -> StrokeSpec {
    StrokeSpec::new(Rgba8::new(0, 0, 0, 255), width)
}

fn head_spec(end: Marker, scale: f64, round: f64) -> StrokeSpec {
    let mut s = spec(2.0);
    s.marker_end = end;
    s.marker_scale = scale;
    s.marker_round = round;
    s
}

/// A peça da LINHA do plano, se houver.
fn line_piece<'a>(plan: &'a [StrokePiece<'a>]) -> Option<&'a VecPath> {
    plan.iter().find_map(|p| match p {
        StrokePiece::Line { path } => Some(&**path),
        _ => None,
    })
}

/// A geometria da PONTA — cheia ou vazada, o que importa aqui é a forma.
fn head_piece<'a>(plan: &'a [StrokePiece<'a>]) -> Option<&'a VecPath> {
    plan.iter().find_map(|p| match p {
        StrokePiece::Fill { path } | StrokePiece::Symbol { path } => Some(path),
        StrokePiece::Line { .. } => None,
    })
}

/// O quanto a cabeça avança para TRÁS do bico, no contorno real (as cúbicas, não as âncoras).
fn head_depth(geo: &VecPath, tip_x: f64) -> f64 {
    const STEPS: usize = 32;
    let n = geo.verts.len();
    let segs = if geo.closed { n } else { n - 1 };
    let mut deepest = f64::MIN;
    for i in 0..segs {
        let (a, b) = (&geo.verts[i], &geo.verts[(i + 1) % n]);
        let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
        for k in 0..=STEPS {
            let u = k as f64 / STEPS as f64;
            let v = 1.0 - u;
            let x = v * v * v * p0[0]
                + 3.0 * v * v * u * p1[0]
                + 3.0 * v * u * u * p2[0]
                + u * u * u * p3[0];
            deepest = deepest.max(tip_x - x);
        }
    }
    deepest
}

/// **O caso de 99% dos paths não paga uma cópia.** Sem ponta há UMA peça, e o caminho dela é
/// o próprio path emprestado — não um clone. Isto não é micro-otimização: o renderer chama
/// isto para cada traço de cada frame, e a promessa de custo zero é o que permitiu que a
/// receita saísse de dentro dele. (E a linha chega inteira na extremidade.)
#[test]
fn a_plain_stroke_is_one_borrowed_piece() {
    let p = line(60.0);
    let s = spec(2.0);
    assert!(!s.has_markers());
    let plan = stroke_plan(&p, &s);
    assert_eq!(plan.len(), 1, "sem ponta, só a linha");
    let StrokePiece::Line { path } = &plan[0] else {
        panic!("a linha é traçada, não preenchida");
    };
    assert!(
        matches!(path, Cow::Borrowed(_)),
        "sem ponta nada é reconstruído — o plano empresta o path"
    );
    assert_eq!(
        path.verts.last().expect("tem vertices").anchor,
        [60.0, 0.0],
        "e não foi encurtada"
    );
}

/// **O gate que morde: a linha para EXATAMENTE nas costas da cabeça, em qualquer `scale`.**
///
/// O recuo e a cabeça têm de ler o MESMO `marker_scale`. Se o recuo ignorar o `scale`, uma
/// cabeça 2.5× maior engole o fim do traço e a linha reaparece ATRAVESSANDO a seta; se a
/// cabeça ignorar, sobra um vão entre o fim da linha e a ponta.
#[test]
fn the_line_meets_the_head_at_every_scale() {
    let path = line(60.0);
    for m in [Marker::Triangle, Marker::Diamond, Marker::CircleOpen] {
        for scale in [0.5, 1.0, 2.5] {
            for round in [0.0, 0.5, 1.0] {
                let s = head_spec(m, scale, round);
                let plan = stroke_plan(&path, &s);
                let geo = head_piece(&plan).expect("a ponta existe");
                let trimmed = line_piece(&plan).expect("sobra linha");
                let end_x = trimmed.verts.last().expect("tem vertices").anchor[0];

                let gap = head_depth(geo, 60.0) - (60.0 - end_x);
                assert!(
                    gap.abs() < 1e-3 * s.width * s.marker_scale,
                    "{m:?} (scale {scale}, round {round}): a linha termina em x={end_x} e a \
                     cabeça vai ate x={} — {} de {}",
                    60.0 - head_depth(geo, 60.0),
                    if gap > 0.0 { "VAO" } else { "sobreposicao" },
                    gap.abs()
                );
            }
        }
    }
}

/// O `marker_round` do usuário chega na cabeça: com ele a ponta perde as quinas vivas (mais
/// vértices, handles não-degenerados). Um `0.0` cravado passaria despercebido — a cena
/// renderiza, só que afiada para sempre.
#[test]
fn the_users_round_reaches_the_head() {
    let path = line(60.0);
    let sharp = stroke_plan(&path, &head_spec(Marker::Triangle, 1.0, 0.0));
    let round = stroke_plan(&path, &head_spec(Marker::Triangle, 1.0, 0.6));
    let sharp = head_piece(&sharp).expect("existe");
    let round = head_piece(&round).expect("existe");
    assert_eq!(sharp.verts.len(), 3, "afiada: um vertice por quina");
    assert_eq!(round.verts.len(), 6, "arredondada: dois por quina");
    assert!(
        round
            .verts
            .iter()
            .all(|v| v.in_handle != v.anchor || v.out_handle != v.anchor),
        "sobrou quina viva: o marker_round nao chegou no build"
    );
}

/// Uma ponta CHEIA vira peça **preenchida** e a linha ENCURTA para caber nela; uma ponta
/// VAZADA vira **símbolo** — traçado, mas com caneta própria, porque o tracejado é da LINHA
/// (um losango pontilhado é ruído, não desenho).
#[test]
fn a_filled_head_fills_and_an_open_head_is_a_symbol() {
    let p = line(60.0);
    let mut s = head_spec(Marker::Triangle, 1.0, 0.0);
    s.dash = Some((2.0, 2.0));
    let plan = stroke_plan(&p, &s);
    assert!(
        matches!(plan[1], StrokePiece::Fill { .. }),
        "o triângulo é cheio"
    );

    s.marker_end = Marker::Open;
    let plan = stroke_plan(&p, &s);
    assert!(
        matches!(plan[1], StrokePiece::Symbol { .. }),
        "a ponta aberta é um símbolo, nunca a caneta tracejada da linha"
    );
    assert!(
        matches!(plan[0], StrokePiece::Line { .. }),
        "…e a LINHA continua sendo a linha (é ela que carrega o tracejado)"
    );
}

/// **Uma linha mais curta que os recuos somados não tem linha** — só as pontas. Cair de
/// volta na linha inteira desenharia exatamente o traço que o recuo existe para esconder.
#[test]
fn a_line_shorter_than_its_heads_has_no_line_piece() {
    let p = line(0.5);
    let mut s = head_spec(Marker::Triangle, 1.0, 0.0);
    s.width = 4.0; // caneta gorda: os recuos somam mais que o comprimento
    s.marker_start = Marker::Triangle;
    let plan = stroke_plan(&p, &s);
    assert!(
        !plan.is_empty(),
        "as pontas continuam existindo — some a LINHA, não o desenho"
    );
    assert!(line_piece(&plan).is_none(), "não sobra linha para desenhar");
}

/// Um contorno FECHADO não tem extremo onde pôr uma ponta — nem com o seletor marcado. E
/// por não ter ponta, também não encurta.
#[test]
fn a_closed_contour_has_no_heads() {
    let p = VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([40.0, 0.0]),
            VecVertex::corner([40.0, 40.0]),
        ],
        closed: true,
        ..VecPath::default()
    };
    let mut s = head_spec(Marker::Triangle, 2.0, 0.0);
    s.marker_start = Marker::Triangle;
    let plan = stroke_plan(&p, &s);
    assert_eq!(plan.len(), 1, "só a linha — um anel não tem extremo");
    assert_eq!(
        line_piece(&plan).expect("a linha").verts.len(),
        3,
        "e ela não foi encurtada"
    );
}
