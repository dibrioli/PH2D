//! Onda 3, rodada 3 (foto `smoke_f1_2_rato_na_linha.png`): o hover da linha LÊ-SE sobre o painel (≥ 1,3:1 contra o
//! `Bg1`) e o nome da linha continua ≥ 4,5:1 sobre ele — nos 12 temas, medido sobre a cor COMPOSTA (o token tem
//! alfa e pinta-se por cima do painel). E nunca se confunde com a linha escolhida, pela COR: o pairado é neutro
//! (croma ≈ 0) e a escolhida é a do acento.

use ph2d_tokens::{Color, ColorToken, Theme, srgb_to_oklch};

use crate::paint::{HOVER_INK, ROW_INK};

const PANEL: ColorToken = ColorToken::Bg1;
const LEGIVEL: f64 = 1.3;
const TEXTO: f64 = 4.5;

/// `top` por cima de `under` (alfa de `top`), opaco.
fn over(top: Color, under: Color) -> Color {
    let a = f64::from(top.a) / 255.0;
    let mix = |t: u8, u: u8| (f64::from(t) * a + f64::from(u) * (1.0 - a)).round() as u8;
    Color {
        r: mix(top.r, under.r),
        g: mix(top.g, under.g),
        b: mix(top.b, under.b),
        a: 255,
    }
}

/// A cor composta de `ink` sobre o painel do tema `t`.
fn on_panel(ink: ColorToken, t: Theme) -> Color {
    over(ink.resolve(t), PANEL.resolve(t))
}

fn chroma(c: Color) -> f64 {
    srgb_to_oklch(c.r, c.g, c.b).1
}

#[test]
fn o_hover_da_linha_le_se_e_o_texto_le_se_nele() {
    let mut falhas = Vec::new();
    for t in Theme::ALL {
        let hover = on_panel(HOVER_INK, t);
        let h = hover.contrast_ratio(&PANEL.resolve(t));
        let x = over(ColorToken::Text1.resolve(t), hover).contrast_ratio(&hover);
        if h < LEGIVEL || x < TEXTO {
            falhas.push(format!("{t:?}: hover {h:.2} · texto {x:.2}"));
        }
    }
    assert!(
        falhas.is_empty(),
        "o hover da linha ({HOVER_INK:?}) tem de ler-se (≥ {LEGIVEL}:1) e deixar o texto ≥ {TEXTO}:1:\n{}",
        falhas.join("\n")
    );
}

#[test]
fn o_hover_e_neutro_e_a_escolhida_e_a_do_acento() {
    for t in Theme::ALL {
        let (hover, sel) = (on_panel(HOVER_INK, t), on_panel(ROW_INK, t));
        assert!(
            chroma(hover) < chroma(sel),
            "{t:?}: o pairado (croma {:.3}) tem de ser mais neutro que a escolhida ({:.3})",
            chroma(hover),
            chroma(sel)
        );
        assert_ne!(hover, sel, "{t:?}: pairado e escolhida distintos");
    }
}
