//! As peças com que o verso se pinta (botão, chip, avatar, caixa, título de secção) e as medidas da
//! maquete `Back.dc.html`. Cada peça que se clica regista o alvo em [`Ui::hits`].

use super::BackHit;
use crate::metrics as m;
use crate::paint::{c, label, text_top};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_icon, paint_text_block, paint_text_elided,
    paint_text_title_elided, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{LabelColor, Member, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::{Color, VectorScene};

// ── As medidas do verso (a maquete aprovada) ──
pub(crate) const MODAL_W: f32 = 768.0;
pub(crate) const MODAL_TOP: f32 = 32.0;
pub(crate) const MODAL_RADIUS: f32 = 12.0;
pub(crate) const MODAL_GUTTER: f32 = 16.0;
pub(crate) const COVER_H: f32 = 112.0;
pub(crate) const PAD_L: f32 = 24.0;
pub(crate) const PAD_R: f32 = 20.0;
pub(crate) const PAD_TOP: f32 = 16.0;
pub(crate) const PAD_BOTTOM: f32 = 24.0;
pub(crate) const TITLE_PX: f32 = 20.0;
pub(crate) const TITLE_LINE: f32 = 28.0;
pub(crate) const HEADING_PX: f32 = 16.0;
pub(crate) const BODY_PX: f32 = 14.0;
pub(crate) const SMALL_PX: f32 = 12.0;
pub(crate) const META_LINE: f32 = 20.0;
pub(crate) const SIDE_W: f32 = 176.0;
pub(crate) const COL_GAP: f32 = 20.0;
/// A coluna principal entra `32` (alinha com o título, depois do ícone).
pub(crate) const MAIN_INDENT: f32 = 32.0;
pub(crate) const SECTION_GAP: f32 = 24.0;
pub(crate) const GAP: f32 = 8.0;
pub(crate) const SMALL_GAP: f32 = 4.0;
pub(crate) const CHIP_GAP_X: f32 = 24.0;
pub(crate) const CHIP_GAP_Y: f32 = 16.0;
pub(crate) const AVATAR: f32 = 32.0;
pub(crate) const AVATAR_SMALL: f32 = 24.0;
pub(crate) const CLOSE: f32 = 36.0;
pub(crate) const CHECK: f32 = 16.0;
pub(crate) const ITEM_H: f32 = 36.0;
pub(crate) const BAR_H: f32 = 8.0;
pub(crate) const PERCENT_W: f32 = 32.0;
pub(crate) const THUMB_W: f32 = 64.0;
pub(crate) const THUMB_H: f32 = 48.0;
pub(crate) const FIELD_COLS: usize = 3;
pub(crate) const POP_W: f32 = 304.0;
pub(crate) const POP_PAD: f32 = 12.0;
/// Um ajuste de um traço (as pílulas e os afastamentos finos da maquete).
pub(crate) const HAIR: f32 = 2.0;
/// O botão «Cover» fica a `56` da borda direita da capa (deixa o fechar à vista).
pub(crate) const COVER_BTN_RIGHT: f32 = 56.0;
/// A faixa de uma linha de texto: o canto `(x, y)`, a largura `w` e a altura `row` (o texto
/// centra-se nela).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Band {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub row: f32,
}

impl Band {
    pub const fn new(x: f32, y: f32, w: f32, row: f32) -> Self {
        Self { x, y, w, row }
    }
}

/// A lista do `@`.
pub(crate) const MENTION_W: f32 = 228.0;

/// Quem pinta o verso: a cena, o texto, o tema e os alvos.
pub(crate) struct Ui<'a> {
    pub scene: &'a mut VectorScene,
    pub ts: &'a mut TextSystem,
    pub theme: Theme,
    pub hits: &'a mut Vec<(Rect, BackHit)>,
    pub hover: Option<BackHit>,
}

impl Ui<'_> {
    pub fn c(&self, t: T) -> Color {
        c(self.theme, t)
    }

    pub fn hit(&mut self, r: Rect, h: BackHit) {
        self.hits.push((r, h));
    }

    /// O azul do Trello (a etiqueta forte azul; a tinta dela é `kanban-label-ink-bold`, medida).
    pub fn accent(&self) -> Color {
        self.c(T::KanbanBlueBold)
    }

    pub fn fg(&self) -> Color {
        self.c(T::KanbanCardFg)
    }

    pub fn meta(&self) -> Color {
        self.c(T::KanbanCardMeta)
    }

    /// A altura de `s` em bloco na largura `w`.
    pub fn measure(&mut self, s: &str, px: f32, w: f32) -> f32 {
        if s.is_empty() {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let lines = self.ts.layout(s, px, w).len().max(1) as f32;
        lines * line_of(px)
    }

    /// Um bloco de texto que quebra; devolve a altura.
    pub fn text(&mut self, s: &str, x: f32, y: f32, px: f32, w: f32, color: Color) -> f32 {
        let h = self.measure(s, px, w);
        paint_text_block(self.ts, self.scene, s, x, y, px, w, color);
        h
    }

    /// Uma linha na faixa `b` (centrada na altura dela), cortada com reticências.
    pub fn line(&mut self, s: &str, b: Band, px: f32, color: Color) {
        let ty = text_top(self.ts, b.y, b.row, px);
        paint_text_elided(self.ts, self.scene, s, b.x, ty, px, b.w, color);
    }

    /// [`Self::line`] em SemiBold.
    pub fn strong(&mut self, s: &str, b: Band, px: f32, color: Color) {
        let ty = text_top(self.ts, b.y, b.row, px);
        paint_text_title_elided(self.ts, self.scene, s, b.x, ty, px, b.w, color);
    }

    pub fn width(&mut self, s: &str, px: f32) -> f32 {
        self.ts.prefix_width(s, px).ceil()
    }

    /// O título de uma secção («Description», «Activity»).
    pub fn heading(&mut self, s: &str, x: f32, y: f32, w: f32) {
        let fg = self.fg();
        self.strong(s, Band::new(x, y, w, super::ROW), HEADING_PX, fg);
    }

    /// O rótulo pequeno por cima de um grupo («Members», «Labels»).
    pub fn caption(&mut self, s: &str, x: f32, y: f32, w: f32) {
        let meta = self.meta();
        self.strong(s, Band::new(x, y, w, META_LINE), SMALL_PX, meta);
    }

    /// A largura de um botão com este texto (e ícone).
    pub fn button_w(&mut self, s: &str, icon: bool) -> f32 {
        let icon_w = if icon { m::ICON_PX + GAP } else { 0.0 };
        self.width(s, BODY_PX) + icon_w + 2.0 * m::BUTTON_PAD_X
    }

    /// Um botão cinzento do verso (o `btn` da maquete): texto ao centro, ou à esquerda com ícone.
    pub fn button(&mut self, r: Rect, s: &str, icon: Option<IconId>, h: BackHit, on: bool) {
        let hover = self.hover == Some(h);
        let (fill, ink) = if on {
            (self.c(T::KanbanHeaderBtnOn), self.c(T::KanbanBtnOnFg))
        } else if hover {
            (self.c(T::KanbanHeaderBtn), self.fg())
        } else {
            (self.c(T::KanbanSlot), self.fg())
        };
        fill_rounded_rect(self.scene, r, m::LABEL_RADIUS, fill);
        if hover && !on {
            fill_rounded_rect(self.scene, r, m::LABEL_RADIUS, self.c(T::KanbanSlot));
        }
        let mut x = r.x + m::BUTTON_PAD_X;
        if let Some(i) = icon {
            let ir = Rect::new(x, r.y + (r.h - m::ICON_PX) / 2.0, m::ICON_PX, m::ICON_PX);
            paint_icon(self.scene, i, ir, ink, StrokeToken::Default.px());
            x += m::ICON_PX + GAP;
        } else {
            let w = self.width(s, BODY_PX);
            x = r.x + ((r.w - w) / 2.0).max(m::BUTTON_PAD_X);
        }
        self.line(
            s,
            Band::new(x, r.y, r.x + r.w - x - m::BUTTON_PAD_X / 2.0, r.h),
            BODY_PX,
            ink,
        );
        self.hit(r, h);
    }

    /// O botão azul («Save», «Add»).
    pub fn primary(&mut self, r: Rect, s: &str, h: BackHit) {
        fill_rounded_rect(self.scene, r, m::LABEL_RADIUS, self.accent());
        let w = self.width(s, BODY_PX);
        let ink = self.c(T::KanbanLabelInkBold);
        self.strong(
            s,
            Band::new(r.x + (r.w - w) / 2.0, r.y, r.w, r.h),
            BODY_PX,
            ink,
        );
        self.hit(r, h);
    }

    /// Um ícone sozinho num botão quadrado (o fechar, o olho).
    pub fn icon_button(&mut self, r: Rect, icon: IconId, h: BackHit, round: bool, fill: Color) {
        let radius = if round { r.w / 2.0 } else { m::LABEL_RADIUS };
        fill_rounded_rect(self.scene, r, radius, fill);
        if self.hover == Some(h) {
            fill_rounded_rect(self.scene, r, radius, self.c(T::KanbanSlot));
        }
        let ir = Rect::new(
            r.x + (r.w - m::ICON_PX) / 2.0,
            r.y + (r.h - m::ICON_PX) / 2.0,
            m::ICON_PX,
            m::ICON_PX,
        );
        let fg = self.fg();
        paint_icon(self.scene, icon, ir, fg, StrokeToken::Default.px());
        self.hit(r, h);
    }

    /// Uma etiqueta-chip de 32 (nome sobre a cor). Devolve a largura.
    pub fn label_chip(&mut self, x: f32, y: f32, name: &str, color: Option<LabelColor>) -> f32 {
        let (fill, ink) = match color {
            Some(col) => label(self.theme, col),
            None => (self.c(T::KanbanSlot), self.fg()),
        };
        let w = (self.width(name, BODY_PX) + 2.0 * m::CARD_PAD_X).max(m::LABEL_MIN_W);
        let r = Rect::new(x, y, w, super::ROW);
        fill_rounded_rect(self.scene, r, m::LABEL_RADIUS, fill);
        self.strong(
            name,
            Band::new(x + m::CARD_PAD_X, y, w, super::ROW),
            BODY_PX,
            ink,
        );
        w
    }

    /// O avatar redondo de um membro (as iniciais sobre a cor dele).
    pub fn avatar(&mut self, x: f32, y: f32, d: f32, who: &Member) {
        let hue = crate::front::avatar_hue(who);
        let (fill, _) = label(
            self.theme,
            LabelColor {
                hue,
                shade: Shade::Normal,
            },
        );
        fill_circle(self.scene, x + d / 2.0, y + d / 2.0, d / 2.0, fill);
        let ini = crate::front::initials(who);
        let px = if d < AVATAR { m::AVATAR_PX } else { SMALL_PX };
        let w = self.width(&ini, px);
        let ink = self.c(T::KanbanAvatarInk);
        self.strong(&ini, Band::new(x + (d - w) / 2.0, y, d, d), px, ink);
    }

    /// A caixa de marcar (16) numa faixa.
    pub fn checkbox(&mut self, r: Rect, on: bool) {
        if on {
            fill_rounded_rect(self.scene, r, m::LABEL_RADIUS / 2.0, self.accent());
            let ink = self.c(T::KanbanLabelInkBold);
            paint_icon(self.scene, IconId::Check, r, ink, StrokeToken::Default.px());
        } else {
            let meta = self.meta();
            stroke_rounded_rect(
                self.scene,
                r,
                m::LABEL_RADIUS / 2.0,
                StrokeToken::Default.px(),
                meta,
            );
        }
    }

    /// Um campo de texto em edição: o fundo, o texto e o cursor.
    pub fn field(&mut self, r: Rect, f: &ph2d_editor_core::widget::LineField, placeholder: &str) {
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.c(T::KanbanCardRaised));
        stroke_rounded_rect(
            self.scene,
            r,
            m::FIELD_RADIUS,
            StrokeToken::Default.px() * 2.0,
            self.accent(),
        );
        let x = r.x + m::FIELD_PAD_X;
        let w = r.w - 2.0 * m::FIELD_PAD_X;
        if f.text.is_empty() {
            let meta = self.meta();
            self.line(placeholder, Band::new(x, r.y, w, r.h), BODY_PX, meta);
        } else {
            let fg = self.fg();
            self.line(&f.text, Band::new(x, r.y, w, r.h), BODY_PX, fg);
        }
        let cx = x + self
            .ts
            .prefix_width(&f.text[..f.caret.min(f.text.len())], BODY_PX);
        let caret = Rect::new(
            cx.min(r.x + r.w - m::FIELD_PAD_X),
            r.y + m::FIELD_PAD_Y,
            m::CARET_W,
            r.h - 2.0 * m::FIELD_PAD_Y,
        );
        fill_rounded_rect(self.scene, caret, 0.0, self.accent());
    }
}

/// A altura de uma linha de texto de `px` (a da maquete: 14 → 20, 12 → 16, 16 → 22, 20 → 28).
pub(crate) fn line_of(px: f32) -> f32 {
    (px * TITLE_LINE / TITLE_PX).round()
}
