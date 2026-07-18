//! **As PRIMITIVAS de linha** do painel Flip — como uma linha se PARECE, separado de
//! quais linhas cada modo mostra (isso é o [`crate::paint_sections`]).
//!
//! O split nasceu do teto de LOC do `paint_sections.rs`, mas a costura é a que já
//! existia no arquivo: aqui moram os 4 blocos reusáveis (`section_label`, `slider_row`,
//! `slider_row_linked`, `segmented`) que TODA seção compõe; lá, a composição. Um
//! `impl BodyCtx` a mais no mesmo crate — o tipo e os campos seguem sendo do irmão.

use crate::paint_sections::{BodyCtx, LABEL_COL_W};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{paint_icon, paint_text, resolve};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::paint_slider_with_chip_layout_adaptive;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, TypeToken};

impl BodyCtx<'_> {
    /// A `Section` label line (Sm, Text2) + its advance.
    pub(crate) fn section_label(&mut self, label: &str, mut y: f32) -> f32 {
        let label_font = TypeToken::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y,
            label_font,
            self.inner_w,
            resolve(ColorToken::Text2, self.theme),
        );
        y += label_font + Spacing::Xs.px();
        y
    }

    /// A full-width slider + linked value chip row; returns the advanced `y`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn slider_row(
        &mut self,
        label: &str,
        slider_id: ph2d_a11y::NodeId,
        chip_id: ph2d_a11y::NodeId,
        track: f32,
        val: f64,
        display: &str,
        y: f32,
    ) -> f32 {
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(self.inner_x, y, self.inner_w, self.row_h),
            label,
            track,
            val,
            Some(display),
            slider_id,
            chip_id,
            LABEL_COL_W,
            self.chip_w,
            self.store,
            self.hit_index,
            self.scene,
            self.text_system,
            self.theme,
        );
        y + used + self.row_gap
    }

    /// A slider row that carries a **link toggle** at its right end (ADR-0114 §4.C).
    ///
    /// The Blender *Unified Paint Settings* idiom: the toggle sits ON THE PROPERTY ROW
    /// and says whether the eraser's copy of that property FOLLOWS the paint brush.
    /// Lit (accent) = linked, dim = the eraser owns its own number.
    ///
    /// The toggle eats a square column at the end and the slider shrinks to fit —
    /// floating the icon over the row would park it on top of the value chip.
    #[allow(clippy::too_many_arguments)] // slider + chip + o toggle: uma linha só
    pub(crate) fn slider_row_linked(
        &mut self,
        label: &str,
        slider_id: ph2d_a11y::NodeId,
        chip_id: ph2d_a11y::NodeId,
        track: f32,
        val: f64,
        display: &str,
        link_id: ph2d_a11y::NodeId,
        linked: bool,
        y: f32,
    ) -> f32 {
        let gap = Spacing::Xs.px();
        let link_w = self.row_h;
        let body_w = (self.inner_w - link_w - gap).max(1.0);
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(self.inner_x, y, body_w, self.row_h),
            label,
            track,
            val,
            Some(display),
            slider_id,
            chip_id,
            LABEL_COL_W,
            self.chip_w,
            self.store,
            self.hit_index,
            self.scene,
            self.text_system,
            self.theme,
        );
        let rect = Rect::new(self.inner_x + self.inner_w - link_w, y, link_w, self.row_h);
        let color = resolve(
            if linked {
                ColorToken::Accent
            } else {
                ColorToken::Text3
            },
            self.theme,
        );
        paint_icon(
            self.scene,
            IconId::Link,
            rect,
            color,
            StrokeToken::Default.px(),
        );
        self.hit_index.register(link_id, rect);
        y + used + self.row_gap
    }

    /// A labelled N-across segmented button row; returns the advanced `y`.
    ///
    /// An EMPTY label paints no caption and reserves no space for one — that is how
    /// a control with more options than fit one row (the eight Reshape brushes)
    /// becomes two rows under a single caption, instead of two captioned rows.
    pub(crate) fn segmented<const N: usize>(
        &mut self,
        label: &str,
        opts: [(ph2d_a11y::NodeId, &str, bool); N],
        mut y: f32,
    ) -> f32 {
        let sd_font = TypeToken::Sm.px();
        let sd_gap = Spacing::Sm.px();
        let cols = N as f32;
        let sd_w = ((self.inner_w - sd_gap * (cols - 1.0)) / cols).max(1.0);
        if !label.is_empty() {
            paint_text(
                self.text_system,
                self.scene,
                label,
                self.inner_x,
                y,
                sd_font,
                self.inner_w,
                resolve(ColorToken::Text2, self.theme),
            );
            y += sd_font + Spacing::Xs.px();
        }
        for (i, (id, lbl, active)) in opts.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (sd_w + sd_gap);
            let rect = Rect::new(rx, y, sd_w, self.row_h);
            let st = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                lbl,
                *active,
                st,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        y + self.row_h + self.row_gap
    }}
