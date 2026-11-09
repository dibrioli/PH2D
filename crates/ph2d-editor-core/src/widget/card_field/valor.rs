//! O que um campo de cartão mostra em repouso: o texto do número e a amostra da cor.

use super::{CardInk, ColorToken, Rect, VectorScene, decimals, fill_rounded_rect, m};

/// A amostra de uma cor, à Figma: com alfa, a metade esquerda é a cor sem alfa e a direita a cor como ela
/// é, sobre o tom do campo. Sem isto, uma cor de alfa 0 («FF0000») não tinha amostra nenhuma.
pub fn paint_amostra(scene: &mut VectorScene, ink: CardInk, r: Rect, c: [u8; 4]) {
    let cor = |a| ph2d_vector::Color::from_rgba8(c[0], c[1], c[2], a); // LITERAL-COLOR-OK: a cor do documento
    if c[3] == u8::MAX {
        fill_rounded_rect(scene, r, m::SEG_RADIUS, cor(u8::MAX));
        return;
    }
    fill_rounded_rect(
        scene,
        r,
        m::SEG_RADIUS,
        ink.color(ColorToken::CardFieldHover),
    );
    fill_rounded_rect(scene, r, m::SEG_RADIUS, cor(c[3]));
    // A metade opaca: os cantos de fora redondos, o corte a direito.
    let metade = r.w * 0.5;
    fill_rounded_rect(
        scene,
        Rect::new(r.x, r.y, metade, r.h),
        m::SEG_RADIUS,
        cor(u8::MAX),
    );
    fill_rounded_rect(
        scene,
        Rect::new(r.x + m::SEG_RADIUS, r.y, metade - m::SEG_RADIUS, r.h),
        0.0,
        cor(u8::MAX),
    );
}

/// O texto do valor em repouso. Um infinito (a Dur sem duração autorada) lê-se «∞», nunca «inf».
pub(super) fn texto_do_valor(value: f64, step: f64) -> String {
    // T29: um passo inteiro não esconde a fração do valor (Friction 0.5 não se lê «0»).
    let s = if value.is_infinite() || (decimals(step) == 0 && value.fract().abs() > 1e-6) {
        crate::widget::number_input::format_number(value)
    } else {
        crate::widget::number_input::format_number_com_casas(value, Some(decimals(step)))
    };
    // `30.000` ao lado de `40` lê-se como duas grandezas: um número só mostra as casas que tem, mesmo com o
    // ruído de um `f32` escalado (`-980.00001`).
    match s.strip_suffix(".000") {
        Some(inteiro) => inteiro.to_owned(),
        None => s,
    }
}

#[cfg(test)]
#[test]
fn um_valor_infinito_le_se_infinito() {
    assert!(!texto_do_valor(f64::INFINITY, 0.2).contains("inf"));
    assert!(texto_do_valor(f64::INFINITY, 0.2).contains('\u{221e}'));
    assert_eq!(texto_do_valor(4.0, 0.2), "4.00");
    // Passo fino: só as casas que o valor tem, mesmo com o ruído de um `f32` escalado (0,3 m em px).
    assert_eq!(texto_do_valor(30.0, 0.001), "30");
    assert_eq!(texto_do_valor(-980.000_01, 1.0), "-980");
    assert_eq!(texto_do_valor(f64::from(0.3_f32 * 100.0), 0.001), "30");
    assert_eq!(texto_do_valor(0.2, 0.001), "0.200");
    assert!(texto_do_valor(0.5, 1.0).starts_with("0.5"));
    // A foto do dono: a Scale X a `-0.00`.
    assert_eq!(texto_do_valor(-0.0, 0.1), "0.00");
    assert_eq!(texto_do_valor(-0.004, 0.1), "0.00");
    assert_eq!(texto_do_valor(-0.04, 0.01), "-0.04");
}

thread_local! {
    static PARADOS: std::cell::RefCell<Vec<super::NodeId>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Os campos PARADOS daqui em diante (substitui os de antes): apagados, sem alvo (nem arrasto, nem escrita,
/// nem foco). Não é um escopo: o fluxo compacto pinta um campo curto só quando pede o vizinho. Quem trava
/// regista a razão por cima (o Detail do Outline enquanto os pontos se editam no canvas, dono 09/11).
pub fn parados(ids: &[super::NodeId]) {
    PARADOS.with(|s| *s.borrow_mut() = ids.to_vec());
}

pub(crate) fn parado(id: super::NodeId) -> bool {
    PARADOS.with(|s| s.borrow().contains(&id))
}
