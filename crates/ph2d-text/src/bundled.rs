//! ⭐⭐ **As fontes EMBUTIDAS e o ESTILO do texto da interface** — o que o artista escolhe em
//! `Settings ▸ Interface font / Font weight / Font size` (2026-10-01).
//!
//! Irmão do [`super::system`] por RESPONSABILIDADE: lá mora a moldagem (o layout e a cache), aqui
//! o que entra nela — que bytes de fonte existem, com que nome são registados, e o estilo que a
//! porta única de layout aplica a todo texto.

use parley::FontContext;
use parley::fontique::{Blob, FontInfoOverride};
use std::sync::Arc;

/// Inter Variable (v4.0, SIL OFL) — bundled so chrome text rasterizes
/// to the same glyphs everywhere, independent of installed system fonts.
/// Inter was designed for screen rendering without LCD subpixel AA,
/// which matches Vello's glyph pipeline (vs. system fonts like SF that
/// are tuned for CoreText's subpixel rendering and look soft here).
/// Source: <https://github.com/rsms/inter/releases/tag/v4.0> (LICENSE.txt
/// in this directory).
pub(crate) const INTER_VARIABLE_TTF: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

/// Os bytes da fonte embutida (InterVariable, OFL). Exposto para quem precisa dos
/// contornos crus dos glyphs — ex. texto VETORIAL (skrifa → `VecPath`), que não passa
/// pelo pipeline parley/vello de UI.
#[must_use]
pub fn inter_variable_ttf() -> &'static [u8] {
    INTER_VARIABLE_TTF
}

/// Family name registered when bundled Inter loads successfully.
/// Falls back to `sans-serif` if registration fails (corrupted bytes,
/// future fontique breaking change, etc.) so we never panic at startup.
pub(crate) const INTER_FAMILY: &str = "InterVariable";

/// ⭐ **Noto Sans Variable (SIL OFL 1.1)** — a 2.ª fonte da interface ([`ph2d_tokens::UiFont`]).
/// Fonte: <https://github.com/google/fonts/tree/main/ofl/notosans> (licença ao lado).
const NOTO_SANS_TTF: &[u8] = include_bytes!("../fonts/NotoSansVariable.ttf");
/// ⭐ **Atkinson Hyperlegible Next (SIL OFL 1.1)** — a 3.ª, desenhada para leitura fácil.
/// Fonte: <https://github.com/google/fonts/tree/main/ofl/atkinsonhyperlegiblenext>.
const ATKINSON_TTF: &[u8] = include_bytes!("../fonts/AtkinsonHyperlegibleNextVariable.ttf");

/// ⭐⭐⭐ **As fontes EMBUTIDAS, pela ordem de [`ph2d_tokens::UiFont::ALL`]** — os bytes e o nome
/// de família com que cada uma é REGISTADA.
///
/// ⛔⛔ **O nome é SEMPRE forçado, nos dois construtores** (2026-10-01, medido): o nome que a Inter
/// traz na tabela dela é *«Inter Variable»*, com espaço, e a pilha pedia `InterVariable` ⇒ com as
/// fontes do sistema carregadas a pilha **não a achava** e caía no `sans-serif` — nesta máquina a
/// `NotoSans-Medium.ttf` ESTÁTICA de `/usr/share/fonts` (`619 976` bytes, contra os `862 936` da
/// Inter). *O app desenhava há meses uma fonte que não era a dele*, e os testes (que correm sem as
/// fontes do sistema, onde o nome era forçado) mediam a Inter: o produto e a régua liam fontes
/// diferentes. ⚠️ E forçar um nome PRÓPRIO evita a colisão inversa: a Noto embutida e a Noto
/// instalada não podem responder pelo mesmo nome.
const BUNDLED: [(&[u8], &str); 3] = [
    (INTER_VARIABLE_TTF, INTER_FAMILY),
    (NOTO_SANS_TTF, "PH2D Noto Sans"),
    (ATKINSON_TTF, "PH2D Atkinson Hyperlegible"),
];

thread_local! {
    /// ⭐⭐ **O estilo do texto da interface** ([`ph2d_tokens::UiTextStyle`]) — mora ao lado da
    /// estratégia de nitidez e pela mesma razão: quem MEDE (o `prefix_width`, a elisão) e quem
    /// PINTA leem o mesmo valor, senão o cursor e as reticências caem no sítio errado.
    static ACTIVE_TEXT_STYLE: std::cell::Cell<ph2d_tokens::UiTextStyle> =
        std::cell::Cell::new(ph2d_tokens::UiTextStyle::default());
}

/// Publica o estilo do texto da interface para esta thread (a shell chama-o uma vez por quadro).
pub fn set_active_text_style(style: ph2d_tokens::UiTextStyle) {
    ACTIVE_TEXT_STYLE.with(|c| c.set(style));
}

/// O estilo do texto da interface desta thread.
#[must_use]
pub fn active_text_style() -> ph2d_tokens::UiTextStyle {
    ACTIVE_TEXT_STYLE.with(std::cell::Cell::get)
}

/// ⭐⭐ **O tamanho em que um texto pedido a `nominal` px é DESENHADO** — a porta de quem conta
/// alturas de linha à mão (a caixa de várias linhas), para elas crescerem com a letra.
#[must_use]
pub fn displayed_font_px(nominal: f32) -> f32 {
    nominal * active_text_style().size.scale()
}

/// Regista as fontes EMBUTIDAS ([`BUNDLED`]) com o nome forçado e devolve a pilha de cada uma —
/// `"<nome><cauda>"`, ou `"sans-serif"` quando o registo não produziu família (bytes rejeitados):
/// o arranque nunca entra em pânico por causa de uma fonte.
pub(crate) fn register_bundled(font_context: &mut FontContext, tail: &str) -> [String; 3] {
    BUNDLED.map(|(bytes, family)| {
        let blob = Blob::new(Arc::new(bytes));
        let override_info = FontInfoOverride {
            family_name: Some(family),
            ..Default::default()
        };
        let registered = font_context
            .collection
            .register_fonts(blob, Some(override_info));
        if registered.is_empty() {
            "sans-serif".to_string()
        } else {
            format!("{family}{tail}")
        }
    })
}
