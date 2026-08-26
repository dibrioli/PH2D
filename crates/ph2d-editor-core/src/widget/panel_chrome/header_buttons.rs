//! **Os BOTÕES do cabeçalho de um painel** — o X de fechar e o `+` de acrescentar.
//!
//! Irmão de [`super`] pelo teto de 500 LOC dos primitivos; o corte é por **responsabilidade**: o
//! pai responde *que forma tem um painel* (superfície, cantos, alças, título, recortes), e isto
//! responde *o que vive na ponta direita da banda do título*.
//!
//! ⚠️ **Os dois rects são PÚBLICOS de propósito**, e não um detalhe do pintor: o `HitIndex` resolve
//! back-to-front, e a alça de arrasto do painel — registada no FIM do quadro — cobre a banda do
//! título. Quem quiser um botão vivo ali tem de **re-registar o hit depois dela**, e para isso
//! precisa do rect sem repetir a geometria. *Duas cópias do mesmo rect divergem no dia em que uma
//! for ajustada, e aí o alvo que o dedo procura fica noutro sítio do que o alvo que o olho vê.*

use super::{PANEL_HEAD_PAD, PANEL_TITLE_BASELINE};
use crate::paint::resolve;
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Rect of the canonical X close button for a panel — same geometry
/// `paint_panel_close_button` paints into. Exposed so panels with a
/// scrollable body can RE-REGISTER the close hit at end-of-frame
/// (after body widgets register their potentially-scrolled rects)
/// so the click-block z-order doesn't accidentally bury the close
/// hit. Without the re-register, a body widget that scrolled into
/// the title-bar band ends up on top of close in the dispatch's
/// back-to-front `HitIndex` walk.
pub fn panel_close_button_rect(panel: Rect) -> Rect {
    let close_size = Spacing::Xl2.px();
    Rect::new(
        panel.x + panel.w - PANEL_HEAD_PAD - close_size,
        panel.y + PANEL_TITLE_BASELINE - 2.0, // LITERAL-PX-OK: baseline-align with title text
        close_size,
        close_size,
    )
}

/// ⭐ **Rect do botão que fica À ESQUERDA do X** — o `+` do Inspector (ADR-0166 / F3).
///
/// ⚠️ **Existe pela MESMA razão do [`panel_close_button_rect`], e a razão mordeu:** o painel
/// regista a **alça de arrasto** no fim do quadro, e ela cobre a banda do título — no passeio
/// back-to-front do `HitIndex` ela ganha a tudo o que foi registado antes. O X sobrevive porque é
/// **re-registado depois dela**; o `+` nasceu sem isso e ficou **morto sob o dedo**, pintado e
/// clicável a nada. *Um controlo nunca pintado e um morto sob o dedo dão o mesmo report.*
///
/// ⛔ **A geometria mora AQUI, num sítio só**, e não repetida no pintor e no re-registo: dois
/// rects para o mesmo botão divergem no dia em que um deles for ajustado, e o resultado é o alvo
/// que o dedo procura noutro sítio do que o alvo que o olho vê.
#[must_use]
pub fn panel_header_add_button_rect(panel: Rect) -> Rect {
    let close = panel_close_button_rect(panel);
    let size = HEADER_ADD_BUTTON_PX;
    Rect::new(
        close.x - size - Spacing::Xl.px(),
        panel.y + PANEL_TITLE_BASELINE - 2.0, // LITERAL-PX-OK: baseline-align with title text
        size,
        size,
    )
}

/// O lado do quadrado do botão de cabeçalho — o mesmo do `Add` da Hierarquia.
pub const HEADER_ADD_BUTTON_PX: f32 = 30.0; // LITERAL-PX-OK: quadrado do botão de cabeçalho

/// Paint the canonical X close button at the top-right of a panel
/// and register its hit rect against `close_id`. Returns the rect so
/// callers can position other header affordances relative to it.
///
/// **UI canon (post-2026-05-24):** every floating panel EXCEPT
/// Hierarchy carries this close button. For image-tool panels
/// (BgRemoval, Padding, Color Equalization, Upscale, Equalize Sizes)
/// the `close_id` is the SAME id as the panel's existing "Cancel"
/// button — clicking the X dispatches the same `WidgetEvent::Click`
/// the bottom Cancel button does, which the panel's `apply_event`
/// translates to `EditorAction::CancelActiveTool`. No new handler
/// needed. For visibility-toggle panels (Inspector, Widget Gallery,
/// Grid Snap) the `close_id` is a panel-specific `*_CLOSE` constant
/// that the panel's apply_event toggles via
/// [`PanelHostInternal::set_panel_visible`].
///
/// Visually: a hit zone de `Spacing::Xl2` (24 px na escala de fábrica) com o X desenhado dentro
/// dela — ⚠️ o doc dizia "32×32 com um glifo de 16×16" e o `panel_close_button_rect` sempre usou o
/// `Spacing::Xl2`; nomear o TOKEN é o que mantém esta frase verdadeira sob uma escala autorada.
/// Color
/// `Text2` (matches subtitle weight — affordance reads as chrome,
/// not a primary action). Sits inside the [`PANEL_HEADER_CLOSE_RESERVE`]
/// slot the [`panel_drag_handle_rect`] leaves clear on the right
/// edge of the title band.
pub fn paint_panel_close_button(
    panel: Rect,
    close_id: ph2d_a11y::NodeId,
    hit_index: &mut crate::interaction::HitIndex,
    scene: &mut VectorScene,
    theme: Theme,
) -> Rect {
    let close_rect = panel_close_button_rect(panel);
    hit_index.register(close_id, close_rect);
    crate::paint::paint_icon(
        scene,
        crate::icons::IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );
    close_rect
}
