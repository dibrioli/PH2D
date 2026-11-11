//! A coluna do lado da Sheet: Call stack e Waiting here numa rolagem só (smoke do dono 10/11: com
//! muitos à espera passava por cima do pé). Linhas inteiras, a barra da casa só se transborda; só as
//! linhas inteiras à vista pintam e recebem o clique.

use super::sheet::{line, line_in};
use super::{CardBox, DebugCardView};
use ph2d_code_debugcard::DebugCard;
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{fill_rounded_rect, frame_radius, resolve};
use ph2d_editor_core::widget::scroll_area::OwnBar;
use ph2d_editor_core::widget::{ScrollbarState, paint_scrollbar, scrollbar_track_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A paragem: outra (linha, função, objecto) volta a coluna ao topo.
fn stop_key(c: &DebugCard) -> String {
    format!("{}:{}:{}", c.line, c.function, c.who.name)
}

/// A altura que a coluna pede: o respiro de cima, a pilha e, se há, quem espera (com o seu respiro).
pub(super) fn natural_height(c: &DebugCard) -> f32 {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let stack = rh * (1 + c.stack.len()) as f32 + pad;
    if c.waiting.is_empty() {
        stack
    } else {
        stack + pad + rh * (1 + c.waiting.len()) as f32
    }
}

impl DebugCardView {
    /// A roda sobre a coluna do lado: linhas inteiras, presa ao fim. `false` = fora dela.
    pub(super) fn side_wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        let Some(b) = self.painted.side.filter(|b| b.rect.contains(x, y)) else {
            return false;
        };
        let rh = ph2d_tokens::row_pitch_px();
        let step = (dy.abs() / rh).ceil().max(1.0) * rh * dy.signum();
        self.side_scroll = safe_clamp(self.side_scroll - step, 0.0, b.max);
        true
    }

    /// Carregar na barra da coluna do lado começa o arrasto. `true` = tomado.
    pub(super) fn side_press(&mut self, x: f32, y: f32) -> bool {
        let Some((d, s)) = self
            .painted
            .side
            .and_then(|b| b.track.press(x, y, self.side_scroll))
        else {
            return false;
        };
        self.side_scroll = s;
        self.side_drag = Some(d);
        true
    }
}

/// ⭐ A coluna em `r` (de cima até ao pé): a pilha (a do topo acesa; clicar vai à linha) e quem espera.
pub(super) fn paint(
    st: &mut DebugCardView,
    c: &DebugCard,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let key = stop_key(c);
    if st.side_stop.as_deref() != Some(key.as_str()) {
        st.side_scroll = 0.0;
        st.side_stop = Some(key);
    }
    let view = Rect::new(r.x, r.y + pad, r.w, (r.h - pad).max(0.0));
    let wait_top = rh * (1 + c.stack.len()) as f32 + pad;
    let content = natural_height(c) - pad;
    let max = ((content - view.h).max(0.0) / rh).ceil() * rh;
    let s = (safe_clamp(st.side_scroll, 0.0, max) / rh).round() * rh;
    st.side_scroll = s;
    let state = if st.side_drag.is_some() {
        ScrollbarState::Dragging
    } else {
        ScrollbarState::Normal
    };
    let total = view.h + max;
    let bar = paint_scrollbar(view, s, total, view.h, (state, 0.0), scene, theme);
    let track = OwnBar {
        body: view,
        content_h: total,
        visible_h: view.h,
    };
    st.painted.side = Some(CardBox {
        rect: r,
        max,
        bar,
        track,
    });
    let room = bar.map_or(r.w, |_| scrollbar_track_rect(view).x - r.x);
    let x = r.x + super::inset();
    let w = (room - super::inset() - pad).max(0.0);
    // O y no ecrã de uma linha que começa em `top` do conteúdo, só se cabe inteira.
    let eps = 0.5; // LITERAL-PX-OK: meio píxel de tolerância ao arredondar
    let seen = |top: f32| {
        let y = view.y + top - s;
        (y >= view.y - eps && y + rh <= view.y + view.h + eps).then_some(y)
    };
    let h4 = (ColorToken::Text3, crate::peso::BOTAO);
    if let Some(y) = seen(0.0) {
        line_in(ts, scene, tr("code.debug.stack"), (x, y, w), h4, theme);
    }
    for (i, f) in c.stack.iter().enumerate() {
        let Some(y) = seen(rh * (1 + i) as f32) else {
            continue;
        };
        let rr = Rect::new(x - pad, y, w + pad, rh);
        if i == 0 {
            let radius = frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, rr, radius, resolve(ColorToken::AccentSoft, theme));
        }
        let name = if f.function.is_empty() {
            tr("code.debug.top_level").to_owned()
        } else {
            f.function.clone()
        };
        let ln = tr_with("code.debug.stack_line", &[("line", &f.line)]);
        let lw = crate::peso::largura(ts, &ln, TypeToken::Xs.px());
        let at = (x, y, w - lw - pad);
        line(ts, scene, &name, at, ColorToken::Text1, theme);
        let at = (x + w - lw, y, lw + pad);
        line(ts, scene, &ln, at, ColorToken::Text3, theme);
        st.painted.stack.push((f.line, rr));
    }
    if c.waiting.is_empty() {
        return;
    }
    if let Some(y) = seen(wait_top) {
        let title = tr_with("code.debug.waiting_here", &[("count", &c.waiting.len())]);
        line_in(ts, scene, &title, (x, y, w), h4, theme);
    }
    for (k, who) in c.waiting.iter().enumerate() {
        let Some(y) = seen(wait_top + rh * (1 + k) as f32) else {
            continue;
        };
        let at = Rect::new(x, y, w, rh);
        super::head::who_plain(who, at, ColorToken::Text2, scene, ts, theme);
        st.painted.waiting.push((who.name.clone(), at));
    }
}
