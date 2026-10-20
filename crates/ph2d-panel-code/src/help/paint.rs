//! A pintura da página: o fundo, o conteúdo (só a faixa à vista: as caixas dos exemplos, as marcas
//! da seleção e da busca, o texto, os sublinhados dos links, os botões de copiar), o índice e o
//! cabeçalho. A disposição refaz-se só quando a página ou a largura mudam.

use super::layout::{self, Dims, Font, Measure, Run, Shape, Tone};
use super::model::HelpDoc;
use super::view::{HelpFrame, HelpHit, HelpView, face, pad, x_in_run};
use crate::chrome::{self, NO_RECT, Ui};
use crate::find_bar::FindBarView;
use crate::glyphs::paint_mono;
use crate::metrics::Metrics;
use crate::palette::Palette;
use crate::style::Ink;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::motion::SETTLED;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_text, paint_text_elided, paint_text_title,
    paint_text_title_elided, rect_to_vello, resolve,
};
use ph2d_editor_core::ui_scale::ao_pixel;
use ph2d_editor_core::widget::{
    ButtonState, IconButtonStyle, IconGlyph, paint_icon_button, paint_popover_row_focus,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ICON_BTN_SIZE_PX, LineHeight, PANEL_MIN_W_PX, ROW_H_PX, Radius, Spacing,
    StrokeToken, Theme, TypeToken,
};
use ph2d_vector::{Color, VectorScene};
use std::ops::Range;

/// As larguras na fonte da pintura; a mono é colunas × avanço, como no editor.
struct Meas<'a> {
    ts: &'a mut TextSystem,
    advance: f32,
}

impl Measure for Meas<'_> {
    fn prose(&mut self, s: &str, font: Font) -> f32 {
        let (px, w) = face(font);
        self.ts.prefix_width_weighted(s, px, w)
    }

    fn mono(&self, s: &str) -> f32 {
        s.chars().count() as f32 * self.advance
    }
}

/// As cores do conteúdo, resolvidas uma vez pelo tema.
struct Tones {
    text: Color,
    muted: Color,
    heading: Color,
    /// O fundo de um exemplo: o degrau da escala ao lado do `CodeBg`.
    code_box: Color,
}

impl Tones {
    fn new(theme: Theme) -> Self {
        let c = |t: ColorToken| resolve(t, theme);
        Self {
            text: c(ColorToken::Text1),
            muted: c(ColorToken::Text2),
            heading: c(ColorToken::Accent),
            code_box: c(ColorToken::Bg3),
        }
    }
}

impl HelpView {
    pub fn paint(
        &mut self,
        frame: &HelpFrame<'_>,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let (pal, ui) = (Palette::new(*theme), Ui::new(*theme));
        let want = ph2d_text::displayed_font_px(TypeToken::Base.px());
        if self.m.font_px != want {
            self.m = Metrics::measure(&mut self.cache, ts, 1).0;
        }
        self.cache.begin_frame();
        self.place(frame, area);
        self.lay(frame.doc, ts);
        self.apply_goal();
        self.goal = None;
        self.clamp();
        self.title.clone_from(&frame.doc.title);
        self.labels.clone_from(&frame.doc.sections);
        self.focus = frame.focus_link;
        let top = self.top_line();
        self.section = self
            .laid
            .as_ref()
            .and_then(|(_, l)| l.sections.iter().rposition(|&y| y <= top));
        scene.push_clip(&rect_to_vello(area));
        scene.fill_rect(rect_to_vello(area), pal.bg);
        self.paint_content(frame, scene, ts, &pal, theme);
        self.paint_index(frame, scene, ts, &ui);
        self.paint_header(frame, scene, ts, &ui);
        if let (Some(bar), Some(r)) = (&frame.find, self.find_rect) {
            self.find.paint(bar, r, scene, ts, theme);
        }
        scene.pop_layer();
        self.cache.end_frame();
    }

    /// Os retângulos do quadro: cabeçalho, índice (se cabe), conteúdo e barra de busca.
    fn place(&mut self, frame: &HelpFrame<'_>, area: Rect) {
        self.area = area;
        let gap = Spacing::Sm.px();
        let head_h = (ROW_H_PX + 2.0 * gap).min(area.h);
        self.header = Rect::new(area.x, area.y, area.w, head_h);
        let body = Rect::new(area.x, area.y + head_h, area.w, (area.h - head_h).max(0.0));
        // O índice só quando o conteúdo fica com pelo menos duas colunas do painel mais estreito.
        let side = PANEL_MIN_W_PX;
        let room = side * 3.0; // LITERAL-PX-OK: o índice e duas larguras de conteúdo
        self.index = if area.w >= room {
            Rect::new(body.x, body.y, side, body.h)
        } else {
            NO_RECT
        };
        let content = Rect::new(
            body.x + self.index.w,
            body.y,
            (body.w - self.index.w).max(0.0),
            body.h,
        );
        self.find_rect = frame.find.as_ref().map(|b| {
            let h = FindBarView::height(b).min(content.h);
            Rect::new(content.x, content.y + content.h - h, content.w, h)
        });
        let bar_h = self.find_rect.map_or(0.0, |r| r.h);
        self.viewport = Rect::new(
            content.x,
            content.y,
            content.w,
            (content.h - bar_h).max(0.0),
        );
        let side = ICON_BTN_SIZE_PX.min(ROW_H_PX);
        let back = Rect::new(area.x + gap, area.y + gap, side, ROW_H_PX);
        let fwd = Rect::new(back.x + side + Spacing::Xs.px(), back.y, side, ROW_H_PX);
        self.back = (back, frame.back);
        self.forward = (fwd, frame.forward);
    }

    /// A disposição na largura do conteúdo (refeita só quando a página ou a largura mudam).
    fn lay(&mut self, doc: &HelpDoc, ts: &mut TextSystem) {
        let width = (self.viewport.w - 2.0 * pad()).max(0.0);
        let key = (doc.key, width.to_bits());
        if self.laid.as_ref().is_some_and(|(k, _)| *k == key) {
            return;
        }
        let lh = |t: TypeToken| {
            ao_pixel(ph2d_text::displayed_font_px(t.px()) * LineHeight::Normal.ratio())
        };
        self.dims = Dims {
            width,
            prose_lh: lh(TypeToken::Base),
            mono_lh: self.m.line_h,
            title_lh: lh(TypeToken::Xl),
            head_lh: lh(TypeToken::Lg),
            gap: Spacing::Lg.px(),
            indent: Spacing::Xl2.px(),
            pad: Spacing::Md.px(),
            button: ICON_BTN_SIZE_PX.min(ROW_H_PX),
        };
        let mut meas = Meas {
            ts,
            advance: self.m.advance,
        };
        self.laid = Some((key, layout::lay_out(&doc.blocks, self.dims, &mut meas)));
    }

    fn paint_content(
        &mut self,
        frame: &HelpFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
        theme: &Theme,
    ) {
        let Some((key, l)) = self.laid.take() else {
            return;
        };
        let vp = self.viewport;
        let top = self.scroll - pad();
        let bottom = top + vp.h;
        let a = l.runs.partition_point(|r| r.y + r.h <= top);
        let b = l.runs.partition_point(|r| r.y < bottom).max(a);
        self.shown = a..b;
        let tones = Tones::new(*theme);
        scene.push_clip(&rect_to_vello(vp));
        let mut copies = Vec::new();
        for (s, r) in l
            .shapes
            .iter()
            .filter(|(_, r)| r[1] + r[3] >= top && r[1] <= bottom)
        {
            let sr = self.screen(r[0], r[1], r[2], r[3]);
            match s {
                Shape::CodeBox => fill_rounded_rect(
                    scene,
                    sr,
                    frame_radius(*theme, Radius::Sm.px()),
                    tones.code_box,
                ),
                Shape::Rule => {
                    let line = Rect::new(sr.x, sr.y, sr.w, StrokeToken::Thin.px());
                    scene.fill_rect(rect_to_vello(line), pal.border);
                }
                Shape::Bullet => {
                    let (cx, cy) = (sr.x + sr.w * 0.5, sr.y + sr.h * 0.5);
                    fill_circle(scene, cx, cy, Spacing::Sm.px() * 0.5, tones.muted);
                }
                Shape::Copy(n) => copies.push((*n, sr)),
            }
        }
        let runs = &l.runs[a..b];
        if let Some(sel) = &frame.selection {
            self.paint_marks(runs, sel, pal.selection, scene, ts);
        }
        for (i, m) in frame.matches.iter().enumerate() {
            let c = if frame.current == Some(i) {
                pal.match_current
            } else {
                pal.match_
            };
            self.paint_marks(runs, m, c, scene, ts);
        }
        let inflate = Spacing::Xxs.px();
        for r in runs
            .iter()
            .filter(|r| r.link.is_some() && r.link == frame.focus_link)
        {
            let sr = self.screen(r.x - inflate, r.y, r.w + 2.0 * inflate, r.h);
            paint_popover_row_focus(scene, sr, *theme);
        }
        let right = vp.x + vp.w;
        for r in runs {
            let lit = r.link.is_some()
                && (frame.hover == r.link.map(HelpHit::Link) || frame.focus_link == r.link);
            self.paint_run(r, lit, right, scene, ts, pal, &tones);
        }
        for (n, r) in copies {
            let state = if frame.hover == Some(HelpHit::Copy(n)) {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            };
            let glyph = IconGlyph::Builtin(IconId::Copy);
            paint_icon_button(
                r,
                glyph,
                IconButtonStyle::Plain,
                (state, SETTLED),
                scene,
                *theme,
            );
        }
        scene.pop_layer();
        self.laid = Some((key, l));
    }

    /// As faixas `range` de [`Self::plain`] por baixo do texto dos trechos `runs`.
    fn paint_marks(
        &self,
        runs: &[Run],
        range: &Range<usize>,
        color: Color,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
    ) {
        for r in runs {
            let (lo, hi) = (range.start.max(r.at), range.end.min(r.at + r.text.len()));
            if lo >= hi {
                continue;
            }
            let (x0, x1) = (
                x_in_run(ts, &self.m, r, lo - r.at),
                x_in_run(ts, &self.m, r, hi - r.at),
            );
            scene.fill_rect(
                rect_to_vello(self.screen(r.x + x0, r.y, x1 - x0, r.h)),
                color,
            );
        }
    }

    /// Um trecho na sua letra; `lit` = o link sob o rato ou com o teclado (sublinhado).
    #[allow(clippy::too_many_arguments)]
    fn paint_run(
        &mut self,
        r: &Run,
        lit: bool,
        right: f32,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        pal: &Palette,
        tones: &Tones,
    ) {
        let ink = match (r.link, r.tone) {
            (Some(_), _) => Ink::Link,
            (None, Tone::Code(i)) => i,
            (None, _) => Ink::Plain,
        };
        let color = match (r.link, r.tone) {
            (Some(_), _) | (None, Tone::Code(_)) => pal.ink(ink),
            (None, Tone::Text) => tones.text,
            (None, Tone::Muted) => tones.muted,
            (None, Tone::Heading) => tones.heading,
        };
        let s = self.screen(r.x, r.y, r.w, r.h);
        let (px, _) = face(r.font);
        let ty = s.y + (s.h - px) * 0.5;
        match r.font {
            Font::Mono => {
                let row_y = s.y + (s.h - self.m.line_h) * 0.5;
                paint_mono(
                    &mut self.cache,
                    &self.m,
                    ts,
                    scene,
                    pal,
                    &r.text,
                    ink,
                    s.x,
                    row_y,
                    right,
                );
            }
            Font::Prose => paint_text(ts, scene, &r.text, s.x, ty, px, f32::INFINITY, color),
            Font::Strong | Font::Heading | Font::Title => {
                paint_text_title(ts, scene, &r.text, s.x, ty, px, f32::INFINITY, color);
            }
        }
        if lit {
            let line = Rect::new(s.x, ty + px, s.w, StrokeToken::Thin.px());
            scene.fill_rect(rect_to_vello(line), color);
        }
    }

    fn paint_index(
        &mut self,
        frame: &HelpFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        ui: &Ui,
    ) {
        self.rows.clear();
        let ix = self.index;
        if ix.w <= 0.0 {
            return;
        }
        scene.fill_rect(rect_to_vello(ix), ui.bg);
        let t = StrokeToken::Thin.px();
        scene.fill_rect(
            rect_to_vello(Rect::new(ix.x + ix.w - t, ix.y, t, ix.h)),
            ui.border,
        );
        scene.push_clip(&rect_to_vello(ix));
        let gap = Spacing::Sm.px();
        let f = chrome::font();
        let radius = frame_radius(ui.theme, Radius::Sm.px());
        let mut y = ix.y + gap;
        for (i, label) in frame.doc.sections.iter().enumerate() {
            let r = Rect::new(ix.x + gap, y, (ix.w - 2.0 * gap).max(0.0), ROW_H_PX);
            let current = self.section == Some(i);
            if current {
                fill_rounded_rect(scene, r, radius, ui.button_on);
            } else if frame.hover == Some(HelpHit::Section(i)) {
                fill_rounded_rect(scene, r, radius, ui.hover);
            }
            let color = if current { ui.text1 } else { ui.text2 };
            let w = (r.w - 2.0 * gap).max(0.0);
            paint_text_elided(
                ts,
                scene,
                label,
                r.x + gap,
                r.y + (r.h - f) * 0.5,
                f,
                w,
                color,
            );
            self.rows.push(r);
            y += ph2d_tokens::row_pitch_px();
        }
        scene.pop_layer();
    }

    fn paint_header(
        &mut self,
        frame: &HelpFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        ui: &Ui,
    ) {
        chrome::panel_bg(scene, self.header, ui, true);
        let buttons = [
            (self.back, HelpHit::Back, IconId::ChevronLeft),
            (self.forward, HelpHit::Forward, IconId::ChevronRight),
        ];
        for ((r, on), hit, icon) in buttons {
            let state = if !on {
                ButtonState::Disabled
            } else if frame.hover == Some(hit) {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            };
            paint_icon_button(
                r,
                IconGlyph::Builtin(icon),
                IconButtonStyle::Plain,
                (state, SETTLED),
                scene,
                ui.theme,
            );
        }
        let h = self.header;
        let x = self.forward.0.x + self.forward.0.w + Spacing::Md.px();
        let w = (h.x + h.w - Spacing::Sm.px() - x).max(0.0);
        let (px, _) = face(Font::Prose);
        paint_text_title_elided(
            ts,
            scene,
            &frame.doc.title,
            x,
            h.y + (h.h - px) * 0.5,
            px,
            w,
            ui.text1,
        );
    }
}
