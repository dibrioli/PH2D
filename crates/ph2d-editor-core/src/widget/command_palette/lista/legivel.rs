//! O contraste da lista: a tinta translúcida pousada sobre o que tem por baixo e a cor apagada
//! puxada para o texto só o bastante para ler ([`TEXTO_MIN`], WCAG 2.2 AA).

use ph2d_vector::Color;

/// O contraste mínimo de um texto (WCAG AA, corpo normal) — lei do dono 12/11.
pub const TEXTO_MIN: f64 = ph2d_tokens::contrast::TEXT_MIN_RATIO;

/// `fg` pousada sobre `bg` (o translúcido mede-se sobre o que tem por baixo), em 8 bits opacos.
#[must_use]
pub fn pousa(fg: Color, bg: Color) -> ph2d_tokens::Color {
    let (f, b) = (fg.to_rgba8(), bg.to_rgba8());
    let a = f32::from(f.a) / f32::from(u8::MAX);
    let mix = |x: u8, y: u8| (f32::from(x) * a + f32::from(y) * (1.0 - a)).round() as u8;
    ph2d_tokens::Color {
        r: mix(f.r, b.r),
        g: mix(f.g, b.g),
        b: mix(f.b, b.b),
        a: u8::MAX,
    }
}

/// ⭐ **A cor apagada que ainda se LÊ**: `c` puxada para `nome` só o bastante para ter
/// [`TEXTO_MIN`] contra cada um dos `fundos` (pousados sobre o primeiro). O tom do tema fica onde
/// ele já lê; onde não lê, aproxima-se do texto — nunca um hex novo.
pub(super) fn legivel(c: Color, nome: Color, fundos: [Color; 2]) -> Color {
    let base = fundos[0];
    let lê = |x: Color| {
        fundos.iter().all(|f| {
            let bg = pousa(*f, base);
            let bg_v = Color::from_rgba8(bg.r, bg.g, bg.b, u8::MAX); // LITERAL-COLOR-OK: a mistura opaca de duas cores do tema, nenhum valor novo
            pousa(x, bg_v).contrast_ratio(&bg) >= TEXTO_MIN
        })
    };
    const PASSOS: u8 = 32; // LITERAL-PX-OK: CONTAGEM de passos da mistura, não medida
    (0..=PASSOS)
        .map(|k| c.lerp_rect(nome, f32::from(k) / f32::from(PASSOS)))
        .find(|x| lê(*x))
        .unwrap_or(nome)
}
