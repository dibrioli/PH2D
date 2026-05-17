//! [`Card`] — surface container with optional header, body and
//! footer rects. Pure layout primitive; the consumer paints children
//! into the slot rects we expose.

use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ICON_BTN_SIZE_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const HEADER_H: f32 = Spacing::Xl3.px();
const FOOTER_H: f32 = ICON_BTN_SIZE_PX;

#[derive(Clone, Debug)]
pub struct Card {
    pub id: NodeId,
    pub title: Option<String>,
    pub footer: bool,
}

impl Card {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            title: None,
            footer: false,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn footer(mut self, yes: bool) -> Self {
        self.footer = yes;
        self
    }

    pub fn header_rect(&self, host: Rect) -> Option<Rect> {
        if self.title.is_some() {
            Some(Rect::new(host.x, host.y, host.w, HEADER_H))
        } else {
            None
        }
    }

    pub fn footer_rect(&self, host: Rect) -> Option<Rect> {
        if self.footer {
            Some(Rect::new(
                host.x,
                host.y + host.h - FOOTER_H,
                host.w,
                FOOTER_H,
            ))
        } else {
            None
        }
    }

    /// Body slot (between header and footer if either is present).
    pub fn body_rect(&self, host: Rect) -> Rect {
        let mut top = host.y;
        let mut bot = host.y + host.h;
        if self.title.is_some() {
            top += HEADER_H;
        }
        if self.footer {
            bot -= FOOTER_H;
        }
        let pad = Spacing::Lg.px();
        Rect::new(
            host.x + pad,
            top + pad,
            (host.w - pad * 2.0).max(0.0),
            (bot - top - pad * 2.0).max(0.0),
        )
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::Group).bounds(x, y, w, h);
        if let Some(title) = &self.title {
            builder = builder.label(title);
        } else {
            builder = builder.label("card");
        }
        builder.build()
    }
}

pub fn paint_card(
    card: &Card,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Lg.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    if let (Some(header), Some(title)) = (card.header_rect(rect), &card.title) {
        let pad = Spacing::Lg.px();
        let font = TypeToken::Md.px();
        let y = header.y + (header.h - font) * 0.5;
        paint_text(
            text_system,
            scene,
            title,
            header.x + pad,
            y,
            font,
            (header.w - pad * 2.0).max(0.0),
            resolve(ColorToken::Text1, theme),
        );
        // 1-px divider between header and body, inset so the line
        // doesn't run into the rounded corners and create the same
        // ear artifact tracked in `docs/UI_Bugs/README.md` §3.1.
        let pad_x = Spacing::Md.px();
        let div_rect = Rect::new(
            rect.x + pad_x,
            header.y + header.h - 1.0,
            rect.w - pad_x * 2.0,
            1.0,
        );
        fill_rounded_rect(scene, div_rect, 0.5, resolve(ColorToken::Border, theme));
    }
    if let Some(footer) = card.footer_rect(rect) {
        let pad_x = Spacing::Md.px();
        let div_rect = Rect::new(rect.x + pad_x, footer.y, rect.w - pad_x * 2.0, 1.0);
        fill_rounded_rect(scene, div_rect, 0.5, resolve(ColorToken::Border, theme));
    }
}

/// Push a clip layer matching `card.body_rect(host)` so consumer
/// content painted between this call and [`pop_card_body_clip`] is
/// kept inside the body slot and can't leak past the rounded chrome.
pub fn push_card_body_clip(card: &Card, host: Rect, scene: &mut VectorScene) {
    let body = card.body_rect(host);
    let clip = ph2d_vector::Rect::new(
        body.x as f64,
        body.y as f64,
        (body.x + body.w) as f64,
        (body.y + body.h) as f64,
    );
    scene.push_clip(&clip);
}

/// Pop the clip layer pushed by [`push_card_body_clip`].
pub fn pop_card_body_clip(scene: &mut VectorScene) {
    scene.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_rect_only_with_title() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        assert!(Card::new(NodeId(1)).header_rect(host).is_none());
        assert!(Card::new(NodeId(1)).title("x").header_rect(host).is_some());
    }

    #[test]
    fn footer_rect_only_when_enabled() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        assert!(Card::new(NodeId(1)).footer_rect(host).is_none());
        assert!(
            Card::new(NodeId(1))
                .footer(true)
                .footer_rect(host)
                .is_some()
        );
    }

    #[test]
    fn body_rect_inset_for_padding() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        let body = Card::new(NodeId(1)).body_rect(host);
        assert!(body.w < host.w);
        assert!(body.h < host.h);
    }

    #[test]
    fn body_rect_excludes_header_and_footer() {
        let host = Rect::new(0.0, 0.0, 200.0, 200.0);
        let card = Card::new(NodeId(1)).title("x").footer(true);
        let body = card.body_rect(host);
        assert!(body.y > host.y + HEADER_H * 0.5);
        assert!(body.y + body.h < host.y + host.h - FOOTER_H * 0.5);
    }

    #[test]
    fn a11y_role_is_group() {
        let node = Card::new(NodeId(1))
            .title("Inspector")
            .build_a11y(0.0, 0.0, 200.0, 200.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(card: Card, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_card(
            &card,
            Rect::new(0.0, 0.0, 240.0, 200.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_plain() {
        smoke(Card::new(NodeId(1)), Theme::Forge);
    }

    #[test]
    fn paint_smoke_with_header() {
        smoke(Card::new(NodeId(1)).title("Inspector"), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_with_footer() {
        smoke(
            Card::new(NodeId(1)).title("Project").footer(true),
            Theme::Blueprint,
        );
    }
}
