//! ⭐ **A seleção, as guias e o laço do vetor pintam com o ACENTO do tema** (F30 §E, F41): o censo
//! `o_overlay_vetorial_nao_tem_cor_crua` prova só a AUSÊNCIA de cor crua; aqui, a tinta que sai — a
//! gravação da cena (`VectorScene::record`) — é a do `ColorToken::Accent` de cada tema, e muda com ele.

use super::*;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_scene::{VecScene, VecViewState, VecXforms, rectangle};
use ph2d_vector::{Brush, Color, DrawOp};

/// As tintas sólidas que `pinta` gravou.
fn tintas(pinta: impl FnOnce(&mut VectorScene)) -> Vec<Color> {
    let mut alvo = VectorScene::default();
    alvo.record();
    pinta(&mut alvo);
    (alvo.take_record().into_iter())
        .filter_map(|op| match op {
            DrawOp::Fill {
                brush: Brush::Solid(c),
                ..
            }
            | DrawOp::Stroke {
                brush: Brush::Solid(c),
                ..
            } => Some(c),
            _ => None,
        })
        .collect()
}

/// Um pintor de overlay sob um tema.
type Pintor = Box<dyn Fn(Theme, &mut VectorScene)>;

fn pintores() -> Vec<(&'static str, Pintor)> {
    vec![
        (
            "caixa",
            Box::new(|t, a| draw_marquee([0.0, 0.0], [9.0, 9.0], t, a)),
        ),
        (
            "laço",
            Box::new(|t, a| draw_lasso(&[(0.0, 0.0), (9.0, 0.0), (4.0, 7.0)], t, a)),
        ),
        (
            "guia",
            Box::new(|t, a| {
                let g = Guide {
                    a: [0.0, 0.0],
                    b: [9.0, 0.0],
                    kind: GuideKind::Align,
                };
                draw_snap_guides(&[g], Affine::IDENTITY, t, a);
            }),
        ),
        (
            "seleção",
            Box::new(|t, a| {
                let mut cena = VecScene::new();
                let p = cena.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
                draw_overlays(
                    &cena,
                    &VecViewState::default(),
                    Some(p),
                    &[p],
                    &[(p, 0)],
                    &VecXforms::new(),
                    Affine::IDENTITY,
                    t,
                    a,
                );
            }),
        ),
    ]
}

#[test]
fn a_selecao_as_guias_e_o_laco_pintam_com_o_acento_do_tema() {
    let (a, b) = (Theme::Forge, Theme::Workshop);
    let (ka, kb) = (
        token_ink(ColorToken::Accent, a),
        token_ink(ColorToken::Accent, b),
    );
    assert_ne!(ka, kb, "controlo: os dois temas têm acentos diferentes");
    for (nome, pinta) in pintores() {
        let (ta, tb) = (tintas(|x| pinta(a, x)), tintas(|x| pinta(b, x)));
        assert!(ta.contains(&ka), "{nome}: sem o acento do {a:?} em {ta:?}");
        assert!(tb.contains(&kb), "{nome}: sem o acento do {b:?} em {tb:?}");
        assert!(!ta.contains(&kb), "{nome}: a tinta não segue o tema");
    }
}
