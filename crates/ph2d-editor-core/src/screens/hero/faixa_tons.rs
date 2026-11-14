//! ⭐ **Os tons da faixa** (onda 6, dono 12/11: «contraste suficiente entre widgets e fundo»): cada
//! controlo da faixa ≥ 3:1 contra o chão e cada texto ≥ 4,5:1 contra o fundo em que se pinta
//! (WCAG 1.4.11 e 1.4.3). Os tons saem do tema; onde um não chega, puxa-se para o texto só o
//! bastante — nunca um hex novo. Portão: `faixa_tons_tests.rs`, nos 12 temas.

use crate::paint::resolve;
use crate::widget::cromo_barra::Barra;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::Color;

/// O contraste mínimo de um texto.
pub const TEXTO_MIN: f64 = ph2d_tokens::contrast::TEXT_MIN_RATIO;
/// O contraste mínimo de um controlo (a borda ou o fundo que o identifica).
pub const CONTROLO_MIN: f64 = ph2d_tokens::contrast::GRAPHIC_MIN_RATIO;

/// Uma plaquinha (a tecla dentro do campo) tem de se ver contra o campo, sem pesar como um botão.
pub const PLACA_MIN: f64 = 1.5; // LITERAL-PX-OK: razão de contraste (adimensional), abaixo do piso WCAG de propósito

/// Os tons de um quadro da faixa.
#[derive(Clone, Copy, Debug)]
pub struct Tons {
    pub chao: Color,
    /// O nome de um botão («Menu», as abas) e o texto escrito.
    pub texto: Color,
    /// O texto apagado que ainda se lê (o convite do campo).
    pub apagado: Color,
    /// O filete do campo.
    pub contorno: Color,
    pub campo: Color,
    /// A plaquinha da tecla dentro do campo («Ctrl K»).
    pub tecla: Color,
    pub tecla_texto: Color,
    /// A marca «PH2D».
    pub marca: Color,
    pub marca_texto: Color,
    /// A pílula do Play.
    pub play: Color,
    pub play_texto: Color,
}

/// `fg` pousada sobre `bg` (o translúcido mede-se sobre o que tem por baixo), opaca.
#[must_use]
pub fn pousa(fg: Color, bg: Color) -> Color {
    let (f, b) = (fg.to_rgba8(), bg.to_rgba8());
    let a = f32::from(f.a) / f32::from(u8::MAX);
    let mix = |x: u8, y: u8| (f32::from(x) * a + f32::from(y) * (1.0 - a)).round() as u8;
    Color::from_rgba8(mix(f.r, b.r), mix(f.g, b.g), mix(f.b, b.b), u8::MAX) // LITERAL-COLOR-OK: a mistura opaca de duas cores do tema, nenhum valor novo
}

/// O contraste WCAG de `fg` pousada sobre `bg` (opaca).
#[must_use]
pub fn razao(fg: Color, bg: Color) -> f64 {
    let opaca = |c: Color| {
        let c = c.to_rgba8();
        ph2d_tokens::Color {
            r: c.r,
            g: c.g,
            b: c.b,
            a: u8::MAX,
        }
    };
    let bg = pousa(bg, bg);
    opaca(pousa(fg, bg)).contrast_ratio(&opaca(bg))
}

/// `c` puxada para `alvo` só o bastante para ter `min` contra `bg`.
pub(super) fn ate_ler(c: Color, alvo: Color, bg: Color, min: f64) -> Color {
    const PASSOS: u8 = 32; // LITERAL-PX-OK: CONTAGEM de passos da mistura, não medida
    (0..=PASSOS)
        .map(|k| c.lerp_rect(alvo, f32::from(k) / f32::from(PASSOS)))
        .find(|x| razao(*x, bg) >= min)
        .unwrap_or(alvo)
}

/// ⭐ **Uma pílula cheia** (a marca, o Play): o tom do tema e a tinta que mais lê nele; se o par não
/// chega (≥ 3:1 contra o chão, texto ≥ 4,5:1), o fundo anda o mínimo para o texto ou para o chão.
pub(super) fn pilula(base: Color, chao: Color, texto: Color, tintas: &[Color]) -> (Color, Color) {
    const PASSOS: u8 = 32; // LITERAL-PX-OK: CONTAGEM de passos da mistura, não medida
    let serve = |f: Color| {
        tintas
            .iter()
            .copied()
            .max_by(|a, b| razao(*a, f).total_cmp(&razao(*b, f)))
            .filter(|t| razao(*t, f) >= TEXTO_MIN && razao(f, chao) >= CONTROLO_MIN)
            .map(|t| (f, t))
    };
    (0..=PASSOS)
        .flat_map(|k| {
            let u = f32::from(k) / f32::from(PASSOS);
            [base.lerp_rect(texto, u), base.lerp_rect(chao, u)]
        })
        .find_map(serve)
        .unwrap_or((texto, chao))
}

/// Os tons do tema, na UI de cartões (`Barra`) ou na de antes.
#[must_use]
pub fn tons(theme: Theme) -> Tons {
    let barra = Barra::de(theme);
    let cor = |t: ColorToken| barra.map_or_else(|| resolve(t, theme), |b| b.tinta().color(t));
    let chao = barra.map_or_else(|| resolve(ColorToken::Bg1, theme), Barra::chao);
    let texto = cor(ColorToken::Text1);
    let campo = pousa(cor(ColorToken::CardField), chao);
    let contorno = [
        ColorToken::Border,
        ColorToken::BorderStrong,
        ColorToken::Text3,
        ColorToken::Text2,
    ]
    .into_iter()
    .map(cor)
    .find(|c| razao(*c, chao) >= CONTROLO_MIN && razao(*c, campo) >= CONTROLO_MIN)
    .unwrap_or(texto);
    let apagado = ate_ler(cor(ColorToken::Text3), texto, campo, TEXTO_MIN);
    let tecla = [
        ColorToken::CardFieldHover,
        ColorToken::KanbanCardRaised,
        ColorToken::Border,
        ColorToken::BorderStrong,
    ]
    .into_iter()
    .map(|t| pousa(cor(t), campo))
    .find(|c| razao(*c, campo) >= PLACA_MIN)
    .unwrap_or_else(|| ate_ler(campo, texto, campo, PLACA_MIN));
    let tecla_texto = ate_ler(cor(ColorToken::Text2), texto, tecla, TEXTO_MIN);
    let escuros = [chao, texto, cor(ColorToken::AccentFg)];
    let (marca, marca_texto) = pilula(cor(ColorToken::Accent), chao, texto, &escuros);
    let (play, play_texto) = pilula(cor(ColorToken::Success), chao, texto, &escuros);
    Tons {
        chao,
        texto,
        apagado,
        contorno,
        campo,
        tecla,
        tecla_texto,
        marca,
        marca_texto,
        play,
        play_texto,
    }
}

#[cfg(test)]
#[path = "faixa_tons_tests.rs"]
mod tests;
