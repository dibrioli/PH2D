//! [`Scrollbar`] — thin vertical drag-affordance for scrollable
//! panels.
//!
//! Single source of truth for the track/thumb geometry: callers
//! ask [`Scrollbar::thumb_rect`] for the hit zone the dispatch
//! should register, and the painter uses [`paint_scrollbar`] to
//! draw the track + thumb at the same coordinates.
//!
//! No state on the widget itself — `panel_scroll`, `panel_content_h`
//! and `panel_visible_h` all live on the [`crate::interaction::WidgetStore`].
//! The dispatcher reads / writes those side-tables; this module
//! only computes geometry + paints.

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// Default scrollbar track width. Wide enough to be a comfortable
/// drag target on iPad/tablet (the user's "Fundamental para
/// ipad/tablet" requirement); narrow enough to not encroach on
/// the panel content.
pub const SCROLLBAR_W: f32 = 10.0; // LITERAL-PX-OK: scrollbar track width (chrome-specific, between Spacing::Md and Lg)
/// Minimum thumb height. When the visible viewport is a large
/// fraction of the content, the proportional thumb height could
/// shrink to 0 px; we clamp so the user can always grab it.
pub const SCROLLBAR_THUMB_MIN_H: f32 = ROW_H_PX;

/// **Os tres estados de um polegar de scrollbar.**
///
/// ⚠️ **Enum proprio, e nao o `SliderState` reciclado.** A interaccao e parecida — agarra-se e
/// arrasta-se — mas o vocabulario de um widget e o que decide a sua tinta, e o dia em que o
/// slider ganhar um quarto estado a barra herda-o sem ninguem ter escolhido isso. E o precedente
/// da casa: `CheckboxState`, `ToggleState`, `TagState` e `DropdownState` sao todos irmaos deste.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ScrollbarState {
    #[default]
    Normal,
    Hovered,
    Dragging,
}

/// Compute the track rect inside the panel body. The caller is
/// responsible for clipping the track to the panel's body region —
/// the rect returned here is just the geometry.
pub fn track_rect(body_rect: Rect) -> Rect {
    Rect::new(
        body_rect.x + body_rect.w - SCROLLBAR_W - 2.0,
        body_rect.y,
        SCROLLBAR_W,
        body_rect.h,
    )
}

/// Compute the thumb rect inside `track`, given the panel's scroll
/// position and the content / visible heights. Returns the full
/// track when `content_h <= visible_h` (no scroll needed — caller
/// should hide).
pub fn thumb_rect(track: Rect, scroll_y: f32, content_h: f32, visible_h: f32) -> Rect {
    if content_h <= visible_h || track.h <= 0.0 {
        return track;
    }
    let raw_h = visible_h / content_h * track.h;
    let thumb_h = raw_h.max(SCROLLBAR_THUMB_MIN_H).min(track.h);
    let max_scroll = (content_h - visible_h).max(1e-3);
    let t = (scroll_y / max_scroll).clamp(0.0, 1.0);
    let thumb_y = track.y + t * (track.h - thumb_h);
    Rect::new(track.x, thumb_y, track.w, thumb_h)
}

/// Returns true iff the scrollbar should be visible at all (some
/// content overflows the viewport). When false, callers can skip
/// paint AND hit registration entirely.
pub fn is_needed(content_h: f32, visible_h: f32) -> bool {
    content_h > visible_h + 0.5
}

/// Paint the scrollbar track + thumb at `body_rect`. No-op when
/// `!is_needed(content_h, visible_h)`.
///
/// **O `visual` e o PAR** ([`crate::interaction::WidgetStore::scrollbar_visual`]): o estado do
/// polegar e quanto do hover esta presente.
///
/// ⚠️ **O par esta na ASSINATURA, e nao num builder, de proposito** — e o precedente do
/// `paint_icon_button`: um `bool` solto **nao compila**, entao o sitio N+1 nao consegue nascer na
/// estrada antiga e nao e preciso um scanner para o vigiar. *Quando o compilador consegue ser o
/// gate, o gate e ele.*
///
/// **A lei da cor, tres degraus e zero token novo:** `BorderEmph` em repouso (L 0.550) ->
/// `Text3` sob o ponteiro (L 0.600, o passo neutro seguinte da mesma familia de matiz) ->
/// `Accent` enquanto se arrasta. O `Accent` fica **reservado ao arrasto**, que e a lei que ja
/// shipava: se o hover ja o usasse, agarrar deixaria de ter resposta.
pub fn paint_scrollbar(
    body_rect: Rect,
    scroll_y: f32,
    content_h: f32,
    visible_h: f32,
    visual: (ScrollbarState, f32),
    scene: &mut VectorScene,
    theme: Theme,
) {
    if !is_needed(content_h, visible_h) {
        return;
    }
    let track = track_rect(body_rect);
    // Subtle track tint so the rail itself reads against the panel
    // surface; uses `Bg3` (the same token the collapsed-section
    // plate uses, for a coherent "interactive surface" palette).
    fill_rounded_rect(
        scene,
        track,
        SCROLLBAR_W * 0.5,
        resolve(ColorToken::Bg3, theme),
    );
    let thumb = thumb_rect(track, scroll_y, content_h, visible_h);
    fill_rounded_rect(
        scene,
        thumb,
        (SCROLLBAR_W * 0.5).min(thumb.h * 0.5),
        thumb_color(visual, theme),
    );
}

/// **A cor de um polegar, e a unica resposta a essa pergunta no app.**
///
/// Tres degraus e ZERO token novo: `BorderEmph` (L 0.550) em repouso -> `Text3` (L 0.600, o passo
/// neutro seguinte da mesma familia de matiz) sob o ponteiro -> `Accent` enquanto se arrasta.
///
/// ⚠️ **O `Accent` fica RESERVADO ao arrasto**, e isso e a lei que ja shipava: se o hover tambem o
/// usasse, agarrar deixaria de ter resposta -- o polegar ja estaria na cor final antes de o dedo
/// fechar. Por isso o eixo macio e so `BorderEmph -> Text3`, e o `Dragging` e estado DURO que nao
/// o percorre.
///
/// ⚠️ **Ela e uma funcao publica porque tem DOIS consumidores** — o [`paint_scrollbar`] e a barra
/// que o painel da timeline pinta a mao, com geometria propria. Duas copias divergiriam no dia em
/// que um degrau se movesse, e o modo de falha seria *uma* barra do app com outra cor, que e
/// exactamente o tipo de coisa que ninguem reporta e todos sentem.
#[must_use]
pub fn thumb_color(visual: (ScrollbarState, f32), theme: Theme) -> ph2d_vector::Color {
    let (state, t) = visual;
    let discrete = match state {
        ScrollbarState::Dragging => ColorToken::Accent,
        ScrollbarState::Hovered => ColorToken::Text3,
        ScrollbarState::Normal => ColorToken::BorderEmph,
    };
    // O neutro `SETTLED` devolve o token discreto, ao bit.
    let soft = matches!(state, ScrollbarState::Normal | ScrollbarState::Hovered);
    crate::motion::hover_axis(
        soft,
        t,
        Some(ColorToken::BorderEmph.resolve(theme)),
        Some(ColorToken::Text3.resolve(theme)),
    )
    .map_or_else(|| resolve(discrete, theme), crate::paint::token_to_vello)
}

/// Convert a cursor y delta to a scroll-y delta given the track
/// dimensions. Used by the drag dispatcher to translate
/// `cursor_y - cursor_y_at_down` into `scroll_y - scroll_y_at_down`.
pub fn delta_for_drag(cursor_delta_y: f32, track_h: f32, content_h: f32, visible_h: f32) -> f32 {
    let raw_h = visible_h / content_h * track_h;
    let thumb_h = raw_h.max(SCROLLBAR_THUMB_MIN_H).min(track_h);
    let drag_range = (track_h - thumb_h).max(1.0);
    let max_scroll = (content_h - visible_h).max(0.0);
    cursor_delta_y * (max_scroll / drag_range)
}

// ⚠️ **A escada dos ids mudou-se** para o irmão `scrollbar_ids` (com os dois gates que a
// guardam), quando este ficheiro bateu no tecto de 500 LOC dos primitivos. Ver o cabeçalho de lá
// para o porquê de um id se CONTAR e não se escolher.

#[cfg(test)]
mod tests {
    use super::*;
    /// **Os tres degraus sao TRES cores, e o `Accent` fica reservado ao arrasto.**
    ///
    /// **Mutacao que deve sangrar:** colapsar `Hovered` em `BorderEmph` (a lei que shipava — o
    /// polegar so tinha DOIS estados e nenhuma barra do app acendia sob o ponteiro).
    #[test]
    fn a_thumb_has_three_steps_and_the_accent_belongs_to_the_drag() {
        let t = Theme::Forge;
        let rest = thumb_color((ScrollbarState::Normal, crate::motion::SETTLED), t);
        let hot = thumb_color((ScrollbarState::Hovered, crate::motion::SETTLED), t);
        let drag = thumb_color((ScrollbarState::Dragging, crate::motion::SETTLED), t);
        assert_ne!(rest, hot, "o polegar nao acende sob o ponteiro");
        assert_ne!(
            hot, drag,
            "hover e arrasto pintam o mesmo — agarrar deixa de ter resposta"
        );
        assert_ne!(rest, drag);
        // O `Accent` e do arrasto: e o unico dos tres com croma alto.
        assert_eq!(
            drag,
            crate::paint::resolve(ColorToken::Accent, t),
            "o arrasto deixou de ser `Accent` — a lei que ja shipava"
        );
    }
    /// **O CONTROLO: o neutro devolve o token discreto, ao bit.**
    ///
    /// Sem esta metade o gate acima seria satisfeito por uma lei que interpolasse SEMPRE — e toda
    /// barra parada do app ficaria numa cor intermedia que ninguem escolheu.
    #[test]
    fn the_settled_neutral_is_the_discrete_token_to_the_bit() {
        let t = Theme::Forge;
        for (state, token) in [
            (ScrollbarState::Normal, ColorToken::BorderEmph),
            (ScrollbarState::Hovered, ColorToken::Text3),
            (ScrollbarState::Dragging, ColorToken::Accent),
        ] {
            assert_eq!(
                thumb_color((state, crate::motion::SETTLED), t),
                crate::paint::resolve(token, t),
                "{state:?} em repouso deixou de ser o token discreto"
            );
        }
    }
    /// **E a meio caminho ele esta ENTRE os dois** — a prova de que o `t` e consumido.
    ///
    /// **Mutacao que deve sangrar:** ignorar o `t` (devolver sempre o token discreto) — a barra
    /// passa a saltar de cor em vez de deslizar, que e o defeito que a wave inteira ataca.
    #[test]
    fn halfway_the_thumb_is_between_the_two_ends() {
        let t = Theme::Forge;
        let rest = thumb_color((ScrollbarState::Normal, crate::motion::SETTLED), t);
        let hot = thumb_color((ScrollbarState::Hovered, crate::motion::SETTLED), t);
        let mid = thumb_color((ScrollbarState::Hovered, 0.5), t);
        assert_ne!(mid, rest);
        assert_ne!(
            mid, hot,
            "o `t` nao e consumido — a cor salta em vez de deslizar"
        );
    }
    #[test]
    fn no_scrollbar_when_content_fits() {
        assert!(!is_needed(100.0, 200.0));
    }
    #[test]
    fn thumb_at_top_when_scroll_zero() {
        let body = Rect::new(0.0, 0.0, 100.0, 200.0);
        let track = track_rect(body);
        let thumb = thumb_rect(track, 0.0, 400.0, 200.0);
        assert!((thumb.y - track.y).abs() < 0.001);
    }
    #[test]
    fn thumb_at_bottom_when_scroll_max() {
        let body = Rect::new(0.0, 0.0, 100.0, 200.0);
        let track = track_rect(body);
        let max = 400.0 - 200.0;
        let thumb = thumb_rect(track, max, 400.0, 200.0);
        let expected_bottom = track.y + track.h;
        assert!((thumb.y + thumb.h - expected_bottom).abs() < 0.001);
    }
    #[test]
    fn delta_for_drag_proportional() {
        // track 100 px, content 400, visible 200 → thumb 50 px,
        // drag_range 50 px, max_scroll 200 → delta×4.
        let d = delta_for_drag(10.0, 100.0, 400.0, 200.0);
        // raw_h = 50, drag_range = 50, ratio = 200/50 = 4 → 10*4 = 40.
        assert!((d - 40.0).abs() < 0.001);
    }
}
