//! ⭐ **A barra de rolagem da tira de quadros** (dono 08/11: «um belo scroll horizontal»): o trilho fica logo abaixo
//! da tira, o polegar mede a parte visível e agarra-se, e clicar no trilho salta para lá. A cor é a de toda barra do
//! app ([`scrollbar_thumb_color`]); a roda continua a rolar pela tira.

use super::doc::AnimDoc;
use ph2d_editor_core::paint::{
    fill_rounded_rect, label_column_width, paint_text_rotated_ccw, resolve,
    stroke_rounded_rect_dashed,
};
use ph2d_editor_core::widget::{SCROLLBAR_W, ScrollbarState, scrollbar_thumb_color};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O polegar no trilho `track` para `content` de largura com `visible` à vista, rolado `scroll`. `None` = cabe tudo.
#[must_use]
pub fn thumb(track: Rect, scroll: f32, content: f32, visible: f32) -> Option<Rect> {
    if track.w <= 0.0 || content <= visible + f32::EPSILON {
        return None;
    }
    let w = (track.w * visible / content).clamp(ROW_H_PX.min(track.w), track.w);
    let max = content - visible;
    let x = track.x + (track.w - w) * (scroll / max).clamp(0.0, 1.0);
    Some(Rect::new(x, track.y, w, track.h))
}

fn hit(r: Rect, x: f32, y: f32) -> bool {
    r.w > 0.0 && x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

impl AnimDoc {
    fn bar_thumb(&self) -> Option<Rect> {
        let l = &self.layout;
        thumb(l.strip_bar, self.strip_scroll, l.strip_content, l.strip.w)
    }

    fn bar_scroll_for(&self, left: f32) -> f32 {
        let l = &self.layout;
        let Some(t) = self.bar_thumb() else {
            return 0.0;
        };
        let room = (l.strip_bar.w - t.w).max(f32::EPSILON);
        ((left - l.strip_bar.x) / room).clamp(0.0, 1.0) * (l.strip_content - l.strip.w)
    }

    /// O Down: no polegar agarra-o onde se tocou; no trilho, o polegar centra-se no ponteiro e fica agarrado.
    pub(super) fn bar_press(&mut self, x: f32, y: f32) -> bool {
        let Some(t) = self
            .bar_thumb()
            .filter(|_| hit(self.layout.strip_bar, x, y))
        else {
            return false;
        };
        let grab = if hit(t, x, y) { x - t.x } else { t.w / 2.0 };
        self.strip_scroll = self.bar_scroll_for(x - grab);
        self.bar_grab = Some(grab);
        true
    }

    /// O arrasto do polegar; `false` = não se arrastava a barra.
    pub(super) fn bar_motion(&mut self, x: f32) -> bool {
        let Some(grab) = self.bar_grab else {
            return false;
        };
        self.strip_scroll = self.bar_scroll_for(x - grab);
        true
    }

    /// Pinta o trilho e o polegar, só quando a tira não cabe.
    pub(super) fn paint_bar(&self, scene: &mut VectorScene, theme: Theme) {
        let Some(t) = self.bar_thumb() else {
            return;
        };
        let r = SCROLLBAR_W / 2.0;
        let track = self.layout.strip_bar;
        fill_rounded_rect(scene, track, r, resolve(ColorToken::Bg3, theme));
        let state = match (self.bar_grab, self.hover) {
            (Some(_), _) => ScrollbarState::Dragging,
            (None, Some((x, y))) if hit(t, x, y) => ScrollbarState::Hovered,
            _ => ScrollbarState::Normal,
        };
        fill_rounded_rect(
            scene,
            t,
            r,
            scrollbar_thumb_color((state, ph2d_editor_core::motion::SETTLED), theme),
        );
    }
}

impl AnimDoc {
    /// ⭐ **A marca de inserção** (dono 08/11): o vão quadriculado entre dois quadros, com borda tracejada de acento e
    /// «INSERT HERE» na vertical — onde as próximas células da folha entram.
    pub(super) fn paint_playhead(
        &self,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
    ) {
        let (r, strip) = (self.layout.playhead, self.layout.strip);
        if r.w <= 0.0 || r.h <= 0.0 {
            return;
        }
        scene.push_clip(&ph2d_vector::Rect::new(
            f64::from(strip.x),
            f64::from(strip.y),
            f64::from(strip.x + strip.w),
            f64::from(strip.y + strip.h),
        ));
        let tile = Spacing::Sm.px();
        let tons = [ColorToken::Bg2, ColorToken::Bg3].map(|c| resolve(c, theme));
        let (cols, rows) = ((r.w / tile).ceil() as usize, (r.h / tile).ceil() as usize);
        for j in 0..rows {
            for i in 0..cols {
                let x = r.x + i as f32 * tile;
                let y = r.y + j as f32 * tile;
                let w = tile.min(r.x + r.w - x);
                let h = tile.min(r.y + r.h - y);
                fill_rounded_rect(scene, Rect::new(x, y, w, h), 0.0, tons[(i + j) % 2]);
            }
        }
        let accent = resolve(ColorToken::Accent, theme);
        let dash = (f64::from(Spacing::Xs.px()), f64::from(Spacing::Xxs.px()));
        let radius = Spacing::Xxs.px();
        stroke_rounded_rect_dashed(scene, r, radius, StrokeToken::Thick.px(), dash, accent);
        let font = TypeToken::Sm.px();
        let label = ph2d_i18n::tr("anim_editor.insert_here");
        let tw = label_column_width(text, font, [label]).min(r.h);
        let (ax, ay) = (r.x + r.w / 2.0, r.y + (r.h + tw) / 2.0);
        paint_text_rotated_ccw(text, scene, label, ax, ay, font, r.h, accent);
        scene.pop_layer();
    }
}

impl AnimDoc {
    /// ⭐ **Os cartões em voo**: cada célula escolhida desce da folha até o seu lugar na tira, por cima de tudo, com a
    /// moldura de acento da escolha.
    pub(super) fn paint_flight(
        &self,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
        frames: &[ph2d_editor_core::AnimFrameRef],
    ) {
        let Some(f) = &self.flight else {
            return;
        };
        let radius = Spacing::Xs.px();
        let accent = resolve(ColorToken::Accent, theme);
        for (j, from) in f.from.iter().enumerate() {
            let Some((slot, _)) = self.layout.frames.get(f.at + j) else {
                continue;
            };
            let full = Rect::new(slot.x, slot.y, self.layout.strip_thumb_w(), slot.h);
            let card = super::doc::flight::between(*from, full, f.t());
            fill_rounded_rect(scene, card, radius, resolve(ColorToken::Bg2, theme));
            let sheets = &self.snap.sheets;
            self.sheet_ui
                .paint_frame(scene, text, theme, sheets, frames, f.at + j, card);
            let w = StrokeToken::Thick.px();
            ph2d_editor_core::paint::stroke_rounded_rect(scene, card, radius, w, accent);
        }
    }
}

#[cfg(test)]
#[path = "strip_bar_tests.rs"]
mod tests;
