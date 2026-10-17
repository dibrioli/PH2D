//! Gates da **F18 — o desenho é o corpo** (`vetor_nova_ui/29_plano_F18_…md` §6, gates 2–4).
//!
//! O censo é `20 tipos × 12 temas × 5 corpos`, lido pela GRAVAÇÃO da cena (`VectorScene::record`)
//! e pelas tintas das corridas de glifos (o texto não passa pela gravação).

use super::*;
use ph2d_tokens::skin_body::{Body, INK_MIN, readable_on};
use ph2d_tokens::{Color as TokenColor, ColorToken};
use ph2d_vector::{Brush, DrawOp, Shape};

/// Larga e baixa como o botão da cena 25: a caixa do checkbox (`h × h`) não cobre a moldura.
fn frame() -> Rect {
    Rect::new(10.0, 20.0, 180.0, 70.0)
}

fn rgb(r: u8, g: u8, b: u8) -> TokenColor {
    TokenColor { r, g, b, a: 255 }
}

/// O azul do dono e o laranja do Hover (cena 25), os dois extremos, e o próprio acento do tema.
fn bodies(theme: Theme) -> [TokenColor; 5] {
    [
        rgb(40, 120, 240),
        rgb(240, 140, 40),
        rgb(250, 250, 250),
        rgb(8, 8, 8),
        TokenColor {
            a: 255,
            ..ColorToken::Accent.resolve(theme)
        },
    ]
}

struct Painted {
    ops: Vec<DrawOp>,
    /// A tinta de cada corrida de glifos, `[r, g, b, a]` pré-multiplicada.
    inks: Vec<[u8; 4]>,
}

fn paint(kind: WidgetKind, theme: Theme, body: Option<Body>) -> Painted {
    let options = vec!["One".to_owned(), "Two".to_owned()];
    let mut scene = ph2d_vector::VectorScene::new();
    scene.record();
    paint_widget_skin(
        kind,
        "Play",
        SkinParam {
            options: &options,
            body,
            ..SkinParam::default()
        },
        frame(),
        &mut scene,
        &mut TextSystem::without_system_fonts(),
        theme,
    );
    let ops = scene.take_record();
    let e = scene.inner().encoding();
    let inks = e
        .resources
        .glyph_runs
        .iter()
        .map(|r| e.draw_data[r.stream_offsets.draw_data].to_ne_bytes())
        .collect();
    Painted { ops, inks }
}

/// O corpo que a ponte do canvas dá a este tipo sobre `under` (a porta `WidgetKind::drawing`).
fn worn(kind: WidgetKind, under: TokenColor) -> Option<Body> {
    match kind.drawing() {
        Drawing::Body => Some(Body::Drawing { under: Some(under) }),
        Drawing::Ground => Some(Body::Ground { under: Some(under) }),
        Drawing::Consumed => None,
    }
}

/// Uma operação OPACA de conteúdo (preenchimento, traço ou glifo).
fn opaque_parts(p: &Painted) -> (usize, usize) {
    let shapes = p
        .ops
        .iter()
        .filter(|op| match op {
            DrawOp::Fill { brush, .. } | DrawOp::Stroke { brush, .. } => {
                solid(brush).is_some_and(|c| c[3] == 255)
            }
            _ => false,
        })
        .count();
    (shapes, p.inks.iter().filter(|c| c[3] == 255).count())
}

fn solid(brush: &Brush) -> Option<[u8; 4]> {
    match brush {
        Brush::Solid(c) => Some(c.to_rgba8().to_u8_array()),
        _ => None,
    }
}

/// Uma operação OPACA que cobre a moldura inteira (folga de 1 px): o corpo de um widget.
fn covers_the_frame(op: &DrawOp) -> bool {
    let DrawOp::Fill {
        brush,
        path,
        transform,
        ..
    } = op
    else {
        return false;
    };
    if solid(brush).is_none_or(|c| c[3] == 0) {
        return false;
    }
    let b = (*transform * path.clone()).bounding_box();
    let f = frame();
    b.x0 <= f64::from(f.x) + 1.0
        && b.y0 <= f64::from(f.y) + 1.0
        && b.x1 >= f64::from(f.x + f.w) - 1.0
        && b.y1 >= f64::from(f.y + f.h) - 1.0
}

fn token(c: [u8; 4]) -> TokenColor {
    TokenColor {
        r: c[0],
        g: c[1],
        b: c[2],
        a: c[3],
    }
}

/// **Nenhuma superfície do tema cobre o desenho** — e a régua vê corpos (controlo no modo tema).
#[test]
fn nenhuma_superficie_do_tema_cobre_o_desenho() {
    let mut seen_bodies = 0;
    for theme in Theme::ALL {
        for kind in WidgetKind::ALL {
            if !kind.wears_the_drawing() {
                continue;
            }
            if paint(kind, theme, None).ops.iter().any(covers_the_frame) {
                seen_bodies += 1;
            }
            for under in bodies(theme) {
                let p = paint(kind, theme, worn(kind, under));
                assert!(
                    !p.ops.iter().any(covers_the_frame),
                    "{kind:?} {theme:?} {under:?}: um corpo do tema cobre o desenho"
                );
            }
        }
    }
    assert!(
        seen_bodies >= 4 * Theme::ALL.len(),
        "a régua não vê corpos: {seen_bodies}"
    );
}

/// **O conteúdo lê-se sobre o desenho**: toda tinta opaca (glifo ou gráfico) a `≥ 3:1` do corpo,
/// e o RÓTULO dos tipos que o mostram é exactamente a tinta legível (`≥ 4,5:1`).
#[test]
fn o_conteudo_le_se_sobre_o_desenho() {
    const LABELLED: [WidgetKind; 4] = [
        WidgetKind::Button,
        WidgetKind::Card,
        WidgetKind::Tag,
        WidgetKind::TextInput,
    ];
    for theme in Theme::ALL {
        for kind in WidgetKind::ALL {
            if !kind.wears_the_drawing() {
                continue;
            }
            for under in bodies(theme) {
                let p = paint(kind, theme, worn(kind, under));
                let ink = readable_on(under, theme);
                for c in p.inks.iter().filter(|c| c[3] == 255) {
                    let r = token(*c).contrast_ratio(&under);
                    assert!(
                        r >= 3.0,
                        "{kind:?} {theme:?} {under:?}: glifo {c:?} a {r:.2}:1"
                    );
                }
                if LABELLED.contains(&kind) {
                    assert!(
                        p.inks.iter().any(|c| token(*c) == ink),
                        "{kind:?} {theme:?} {under:?}: o rótulo não é a tinta legível {ink:?} — {:?}",
                        p.inks
                    );
                    assert!(ink.contrast_ratio(&under) >= INK_MIN);
                }
            }
        }
    }
}

/// **Cada pele vestida guarda o seu conteúdo** — medido na foto F18: o interruptor sem polegar e o
/// checkbox sem caixa, porque as partes deles eram apagadas como se fossem o corpo.
///
/// Por tipo, por extenso: quantas formas opacas e quantos glifos opacos, no MÍNIMO, sobrevivem. O
/// `Divider` declara zero — o desenho É a linha.
#[test]
fn cada_pele_vestida_guarda_o_seu_conteudo() {
    let want = |kind: WidgetKind| -> (usize, usize) {
        match kind {
            WidgetKind::Divider => (0, 0),
            // O `paint_toggle` não recebe texto: o interruptor é só as peças.
            WidgetKind::Toggle => (1, 0),
            WidgetKind::Checkbox => (1, 1),
            WidgetKind::Slider | WidgetKind::ProgressBar | WidgetKind::Spinner => (1, 0),
            WidgetKind::LevelMeter => (1, 0),
            WidgetKind::Dropdown => (1, 1),
            WidgetKind::Button
            | WidgetKind::Tag
            | WidgetKind::TextInput
            | WidgetKind::Card
            | WidgetKind::SectionHeader
            | WidgetKind::ListItem
            | WidgetKind::NumberInput
            | WidgetKind::Tabs
            | WidgetKind::RadioGroup
            | WidgetKind::SegmentedAdaptive => (0, 1),
            WidgetKind::IconButton | WidgetKind::ColorSwatch => (0, 0),
        }
    };
    for theme in Theme::ALL {
        for kind in WidgetKind::ALL {
            for under in bodies(theme) {
                let got = opaque_parts(&paint(kind, theme, worn(kind, under)));
                let (shapes, glyphs) = want(kind);
                assert!(
                    got.0 >= shapes && got.1 >= glyphs,
                    "{kind:?} {theme:?} {under:?}: sobrou {got:?}, o mínimo é {:?}",
                    (shapes, glyphs)
                );
            }
        }
    }
}
