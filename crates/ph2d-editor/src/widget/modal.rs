//! [`Modal`] — centered card on top of a `BgScrim` overlay.
//!
//! Layout: header (title + optional close icon), body slot (consumer
//! paints into [`Modal::body_rect`]), footer with cancel + confirm
//! buttons. Focus trap is the shell's job — we only mark the dialog
//! a11y node so VoiceOver narrates correctly.

use crate::icons::IconId;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
use crate::widget::{Button, ButtonState, paint_button};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const HEADER_H: f32 = 44.0;
const FOOTER_H: f32 = 56.0;

#[derive(Clone, Debug)]
pub struct Modal {
    pub id: NodeId,
    pub title: String,
    pub cancel: Button,
    pub confirm: Button,
    /// Visual state of the inline close icon. Set by the consumer
    /// from `WidgetStore::button_state(modal.close_id)` so the X
    /// icon highlights on hover, matching every other interactive
    /// chip in the editor.
    pub close_state: ButtonState,
}

impl Modal {
    pub fn new(id: NodeId, title: impl Into<String>, cancel: Button, confirm: Button) -> Self {
        Self {
            id,
            title: title.into(),
            cancel,
            confirm,
            close_state: ButtonState::Normal,
        }
    }

    pub fn close_state(mut self, state: ButtonState) -> Self {
        self.close_state = state;
        self
    }

    /// True iff `(x, y)` lies outside the dialog (i.e. on the scrim).
    /// Use this to implement click-on-scrim-to-dismiss behavior.
    pub fn is_on_scrim(&self, dialog_rect: Rect, x: f32, y: f32) -> bool {
        !dialog_rect.contains(x, y)
    }

    pub fn body_rect(&self, host: Rect) -> Rect {
        let pad = Spacing::Lg.px();
        Rect::new(
            host.x + pad,
            host.y + HEADER_H + pad,
            (host.w - pad * 2.0).max(0.0),
            (host.h - HEADER_H - FOOTER_H - pad * 2.0).max(0.0),
        )
    }

    pub fn header_rect(&self, host: Rect) -> Rect {
        Rect::new(host.x, host.y, host.w, HEADER_H)
    }

    pub fn footer_rect(&self, host: Rect) -> Rect {
        Rect::new(host.x, host.y + host.h - FOOTER_H, host.w, FOOTER_H)
    }

    pub fn close_rect(&self, host: Rect) -> Rect {
        let icon = 24.0;
        let pad = Spacing::Lg.px();
        Rect::new(
            host.x + host.w - pad - icon,
            host.y + (HEADER_H - icon) * 0.5,
            icon,
            icon,
        )
    }

    /// Build the dialog node. AccessKit `Role::Dialog` signals
    /// "modal" semantics to assistive tech.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Dialog)
            .label(&self.title)
            .bounds(x, y, w, h)
            .build()
    }
}

/// Paint the scrim + dialog at `dialog_rect`. The scrim spans
/// `viewport`; the dialog spans `dialog_rect` (consumer chooses the
/// position — typically centered).
pub fn paint_modal(
    modal: &Modal,
    viewport: Rect,
    dialog_rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    scene.fill_rect(rect_to_vello(viewport), resolve(ColorToken::BgScrim, theme));

    let radius = Radius::Lg.px();
    fill_rounded_rect(
        scene,
        dialog_rect,
        radius,
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        dialog_rect,
        radius,
        1.0,
        resolve(ColorToken::Border, theme),
    );

    let header = modal.header_rect(dialog_rect);
    let pad = Spacing::Lg.px();
    let title_font = TypeToken::Md.px();
    let title_y = header.y + (header.h - title_font) * 0.5;
    let close_rect = modal.close_rect(dialog_rect);
    // Reserve everything left of the close icon for the title — the
    // previous `header.w - pad * 3 - 24` heuristic underestimated
    // the gap and let long titles slip under the X.
    let title_max_w = ((close_rect.x - (header.x + pad)) - pad).max(0.0);
    paint_text(
        text_system,
        scene,
        &modal.title,
        header.x + pad,
        title_y,
        title_font,
        title_max_w,
        resolve(ColorToken::Text1, theme),
    );

    // Hover/press feedback on the close X — a flat icon with no
    // visible chip felt static next to the rest of the chrome. Uses
    // the same AccentSoft/Accent pair as `HIERARCHY_ADD`.
    let (close_bg, close_fg) = match modal.close_state {
        ButtonState::Pressed => (Some(ColorToken::Accent), ColorToken::AccentFg),
        ButtonState::Hovered => (Some(ColorToken::AccentSoft), ColorToken::Text1),
        _ => (None, ColorToken::Text2),
    };
    if let Some(bg) = close_bg {
        fill_rounded_rect(scene, close_rect, 6.0, resolve(bg, theme));
    }
    paint_icon(
        scene,
        IconId::Close,
        close_rect,
        resolve(close_fg, theme),
        1.5,
    );

    let footer = modal.footer_rect(dialog_rect);
    let btn_w = 96.0_f32;
    let btn_h = 32.0_f32;
    let gap = Spacing::Md.px();
    let confirm_x = footer.x + footer.w - pad - btn_w;
    let cancel_x = confirm_x - gap - btn_w;
    let btn_y = footer.y + (footer.h - btn_h) * 0.5;
    paint_button(
        &modal.cancel,
        Rect::new(cancel_x, btn_y, btn_w, btn_h),
        scene,
        text_system,
        theme,
    );
    paint_button(
        &modal.confirm,
        Rect::new(confirm_x, btn_y, btn_w, btn_h),
        scene,
        text_system,
        theme,
    );
}

/// Push a clip layer matching `modal.body_rect(host)` so consumer
/// content painted between this call and [`pop_modal_body_clip`]
/// can't leak past the dialog chrome. Always pair the calls.
pub fn push_modal_body_clip(modal: &Modal, host: Rect, scene: &mut VectorScene) {
    let body = modal.body_rect(host);
    let clip = ph2d_vector::Rect::new(
        body.x as f64,
        body.y as f64,
        (body.x + body.w) as f64,
        (body.y + body.h) as f64,
    );
    scene.push_clip(&clip);
}

/// Pop the clip layer pushed by [`push_modal_body_clip`].
pub fn pop_modal_body_clip(scene: &mut VectorScene) {
    scene.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::ButtonKind;

    fn fixture() -> Modal {
        Modal::new(
            NodeId(1),
            "Confirm delete",
            Button::new(NodeId(2), "Cancel"),
            Button::new(NodeId(3), "Delete").kind(ButtonKind::Danger),
        )
    }

    #[test]
    fn body_rect_carved_out_of_header_footer() {
        let m = fixture();
        let host = Rect::new(0.0, 0.0, 400.0, 300.0);
        let body = m.body_rect(host);
        assert!(body.y >= host.y + HEADER_H);
        assert!(body.y + body.h <= host.y + host.h - FOOTER_H);
    }

    #[test]
    fn close_rect_lives_in_header() {
        let m = fixture();
        let host = Rect::new(0.0, 0.0, 400.0, 300.0);
        let close = m.close_rect(host);
        assert!(close.y < HEADER_H);
        assert!(close.x > host.x + host.w - 50.0);
    }

    #[test]
    fn a11y_role_is_dialog() {
        let node = fixture().build_a11y(0.0, 0.0, 400.0, 300.0);
        assert_eq!(node.role(), Role::Dialog);
    }

    fn smoke(modal: Modal, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_modal(
            &modal,
            Rect::new(0.0, 0.0, 1024.0, 768.0),
            Rect::new(312.0, 234.0, 400.0, 300.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_with_focused_confirm() {
        let mut m = fixture();
        m.confirm = m.confirm.state(ButtonState::Focused);
        smoke(m, Theme::Sunstone);
    }
}
