//! **As PRIMITIVAS de linha** do painel Flip — como uma linha se PARECE, separado de
//! quais linhas cada modo mostra (isso é o [`crate::paint_sections`]).
//!
//! O split nasceu do teto de LOC do `paint_sections.rs`, mas a costura é a que já
//! existia no arquivo: aqui moram os 4 blocos reusáveis (`section_label`, `slider_row`,
//! `slider_row_linked`, `segmented`) que TODA seção compõe; lá, a composição. Um
//! `impl BodyCtx` a mais no mesmo crate — o tipo e os campos seguem sendo do irmão.

use crate::paint_sections::BodyCtx;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{paint_icon, paint_text, resolve};
use ph2d_editor_core::widget::paint_slider_with_chip_layout_adaptive;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button_in_group;
use ph2d_editor_core::widget::{block_cells, grid_height};
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
        y += label_font + ph2d_tokens::control_gap_px();
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
            ph2d_editor_core::widget::property_label_col_w(self.inner_x, self.inner_w),
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
            ph2d_editor_core::widget::property_label_col_w(self.inner_x, body_w),
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
        y: f32,
    ) -> f32 {
        self.segmented_block(label, &opts, &[N], y)
    }

    /// ⭐⭐⭐ **N opções em várias fileiras, e o conjunto é UM CORPO** (wave 20, report do dono:
    /// *«tudo o que puder ser ajuntado, ajunte»*).
    ///
    /// ⚠️ **Este painel reimplementava a disposição de um grupo** — largura dividida por `N`, um
    /// `Spacing::Sm` de vão entre chips e o `paint_segmented_button` de quatro quinas — enquanto a
    /// casa já tinha a porta (`segment_rects` / `block_cells` + `paint_segmented_button_in_group`)
    /// desde a wave 10. *Uma cópia da disposição não se lê como cópia: lê-se como um painel que
    /// «ainda não foi convertido», e a diferença só aparece a olho.*
    ///
    /// ⛔ **As fileiras são DADAS, não refluídas.** O `paint_segmented_group_adaptive` decide a
    /// quebra pela largura dos rótulos, e aqui o autor já a escolheu (`Mode` é `3·3·2` porque
    /// «Sculpt» não cabe num sexto de painel docado) — reflectir isso é preservar uma medição que
    /// já foi feita, não teimosia.
    pub(crate) fn segmented_block(
        &mut self,
        label: &str,
        opts: &[(ph2d_a11y::NodeId, &str, bool)],
        cols_per_row: &[usize],
        mut y: f32,
    ) -> f32 {
        debug_assert_eq!(
            opts.len(),
            cols_per_row.iter().sum::<usize>(),
            "a grelha declarada nao cobre as opcoes"
        );
        if !label.is_empty() {
            let sd_font = TypeToken::Sm.px();
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
            y += sd_font + ph2d_tokens::control_gap_px();
        }
        let block = block_cells(
            Rect::new(self.inner_x, y, self.inner_w, 0.0),
            cols_per_row,
            self.row_h,
        );
        let mut i = 0usize;
        for (r, count) in cols_per_row.iter().enumerate() {
            for k in 0..*count {
                let (id, lbl, active) = opts[i + k];
                let (rect, cell) = block[r][k];
                let st = self.store.button_visual(id);
                paint_segmented_button_in_group(
                    rect,
                    lbl,
                    active,
                    st,
                    self.scene,
                    self.text_system,
                    self.theme,
                    cell,
                );
                self.hit_index.register(id, rect);
            }
            i += count;
        }
        y + grid_height(cols_per_row.len(), self.row_h) + self.row_gap
    }
}
