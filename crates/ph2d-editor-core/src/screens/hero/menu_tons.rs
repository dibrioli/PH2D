//! ⭐ **Os tons dos menus da barra** (onda 8): o rótulo e o atalho ≥ 4,5:1 contra o fundo do menu, o
//! ícone ≥ 3:1 (WCAG 1.4.11), a linha acesa (o caminho da cascata e a linha sob o rato) ≥ 3:1 com o
//! texto dela ≥ 4,5:1, e o risco entre grupos ≥ 1,5:1 — decorativo, fora da 1.4.11; medido 13/11: VS
//! Code 1,7, Godot 1,2, maquete C ~1,3. Os tons saem do tema e andam o mínimo para o texto (molde de
//! [`super::faixa_tons`]); nunca um hex novo. Portão: `menu_tons_tests.rs`, nos 12 temas.

use super::faixa_tons::{CONTROLO_MIN, PLACA_MIN, TEXTO_MIN, ate_ler, pilula, pousa, razao};
use crate::widget::lista_flutuante::Superficie;
use ph2d_tokens::{ColorToken, Radius, Theme};
use ph2d_vector::Color;

/// Os tons de um menu da barra.
#[derive(Clone, Copy, Debug)]
pub struct TonsDoMenu {
    pub fundo: Color,
    pub rotulo: Color,
    /// A coluna do atalho («Ctrl+S»): mais apagada que o rótulo, ainda texto.
    pub atalho: Color,
    pub icone: Color,
    /// O risco entre dois grupos.
    pub risco: Color,
    /// A linha acesa: a que abriu o submenu e a que está sob o rato (maquete `.c15it.on`).
    pub aceso: Color,
    pub aceso_texto: Color,
    /// A linha cinzenta (indisponível): apagada de propósito, nunca some.
    pub cinza: Color,
    /// A quina da linha acesa, pela porta do tema.
    pub raio: f32,
}

/// Os tons do menu no tema, na UI de cartões ou na de antes (a mesma [`Superficie`] que o pinta).
#[must_use]
pub fn tons_do_menu(theme: Theme) -> TonsDoMenu {
    let s = Superficie::de(theme);
    // O fundo é o da superfície; num tema cujo texto não lê nele (Plumber Red/Green na UI de antes: 4,1
    // e 3,8 sobre o BgElev, e nenhum tom do tema passa de 4,1), o primeiro fundo do tema em que lê.
    let tintas = [
        ColorToken::Text1,
        ColorToken::AccentFg,
        ColorToken::Bg0,
        ColorToken::Bg1,
    ];
    let extremo_em = |f: Color| {
        tintas
            .into_iter()
            .map(|tk| pousa(s.cor(tk), f))
            .max_by(|a, b| razao(*a, f).total_cmp(&razao(*b, f)))
            .unwrap_or(f)
    };
    let fundo = std::iter::once(s.fundo())
        .chain(
            [
                ColorToken::Bg1,
                ColorToken::Bg0,
                ColorToken::Bg2,
                ColorToken::Bg3,
            ]
            .map(|tk| pousa(s.cor(tk), s.fundo())),
        )
        .find(|f| razao(extremo_em(*f), *f) >= TEXTO_MIN)
        .unwrap_or_else(|| s.fundo());
    let extremo = extremo_em(fundo);
    let rotulo = ate_ler(s.cor(ColorToken::Text1), extremo, fundo, TEXTO_MIN);
    let atalho = ate_ler(s.cor(ColorToken::Text3), rotulo, fundo, TEXTO_MIN);
    let icone = ate_ler(s.cor(ColorToken::Text2), rotulo, fundo, CONTROLO_MIN);
    let risco = pousa(
        ate_ler(s.cor(ColorToken::Border), rotulo, fundo, PLACA_MIN),
        fundo,
    );
    let escuros = [fundo, rotulo, s.cor(ColorToken::AccentFg)];
    let (aceso, aceso_texto) = pilula(s.cor(ColorToken::Accent), fundo, rotulo, &escuros);
    TonsDoMenu {
        fundo,
        rotulo,
        atalho,
        icone,
        risco,
        aceso,
        aceso_texto,
        cinza: s.cor(ColorToken::Text3),
        raio: crate::paint::frame_radius(theme, Radius::Sm.px()),
    }
}

/// O atalho dentro da linha acesa: a tinta dela, um pouco recuada, ainda ≥ 4,5:1.
#[must_use]
pub fn atalho_aceso(t: &TonsDoMenu) -> Color {
    let recuado = t.aceso_texto.lerp_rect(t.aceso, 0.28); // LITERAL-PX-OK: fração da mistura de duas cores do tema
    if razao(recuado, t.aceso) >= TEXTO_MIN {
        recuado
    } else {
        t.aceso_texto
    }
}

#[cfg(test)]
#[path = "menu_tons_tests.rs"]
mod tests;
