//! Body-section painters for the Flip Style panel (Mode / Brush / Color /
//! Erase), extracted from [`crate::paint`] so the file stays under the panel
//! LOC caps. [`BodyCtx`] bundles the per-frame mutables (Vello scene, text
//! shaper, widget store, hit index) + the shared layout metrics; each `paint_*`
//! takes the running `y` and returns the advanced `y`. Mirror of the Vector
//! panel's `BodyCtx`.

use crate::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{
    ColorSwatch, SwatchSize, paint_color_swatch, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_tool_flip::{
    EraseMode, FillMode, FlipMode, FlipStyleSnapshot, GAP_MAX_PX, GROW_MAX, GROW_MIN,
    PRECISION_MAX, PRECISION_MIN, px_to_slider,
};
use ph2d_vector::VectorScene;

/// Label column width for slider rows + the Stroke label.
pub(crate) const LABEL_COL_W: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

/// Per-frame paint context for the Flip Style panel body — mutable render
/// targets + shared layout metrics. Each section method borrows disjoint fields.
pub(crate) struct BodyCtx<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub store: &'a WidgetStore,
    pub hit_index: &'a mut HitIndex,
    pub theme: Theme,
    pub inner_x: f32,
    pub inner_w: f32,
    pub row_h: f32,
    pub row_gap: f32,
    pub chip_w: f32,
    pub font: f32,
}

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
    fn slider_row(
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

    /// A labelled N-across segmented button row; returns the advanced `y`.
    fn segmented<const N: usize>(
        &mut self,
        label: &str,
        opts: [(ph2d_a11y::NodeId, &str, bool); N],
        mut y: f32,
    ) -> f32 {
        let sd_font = TypeToken::Sm.px();
        let sd_gap = Spacing::Sm.px();
        let cols = N as f32;
        let sd_w = ((self.inner_w - sd_gap * (cols - 1.0)) / cols).max(1.0);
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
    }

    /// Mode row (Select / Draw / Erase / Fill) — the gizmo is live only in Select.
    pub(crate) fn mode_row(&mut self, snap: &FlipStyleSnapshot, y: f32) -> f32 {
        self.segmented(
            "Mode",
            [
                (
                    ids::FLIP_MODE_SELECT,
                    "Select",
                    snap.mode == FlipMode::Select,
                ),
                (ids::FLIP_MODE_DRAW, "Draw", snap.mode == FlipMode::Draw),
                (ids::FLIP_MODE_ERASE, "Erase", snap.mode == FlipMode::Erase),
                (ids::FLIP_MODE_FILL, "Fill", snap.mode == FlipMode::Fill),
            ],
            y,
        )
    }

    /// **Cada modo mostra SÓ os seus atributos** (Enio 2026-07-12).
    ///
    /// O painel é o da FERRAMENTA, não o da crate: exibir o Hardness do pincel enquanto o
    /// usuário está no balde é oferecer um controle que não faz nada — e um controle que
    /// não faz nada é pior que a ausência dele (o usuário mexe e conclui que o app está
    /// quebrado).
    ///
    /// - **Draw**: o pincel inteiro (Size / Hardness / Opacity / Smoothing).
    /// - **Erase**: só o que a borracha REALMENTE usa — o raio (`width_px`) e a força
    ///   (`opacity`). Ela não tem dureza, nem alisamento, nem cor.
    /// - **Select / Fill**: nada daqui.
    pub(crate) fn brush(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        let eraser = snap.mode == FlipMode::Erase;
        if !eraser && snap.mode != FlipMode::Draw {
            return y;
        }
        y = self.section_label(if eraser { "Eraser" } else { "Brush" }, y);
        // Size (px) — o raio da borracha, no modo Erase.
        let track = self
            .store
            .slider(ids::FLIP_SIZE)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap.width_px));
        let px = self
            .store
            .number_value(ids::FLIP_SIZE_NUM)
            .unwrap_or(snap.width_px);
        y = self.slider_row(
            "Size",
            ids::FLIP_SIZE,
            ids::FLIP_SIZE_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );
        // Hardness (0..1) — só o pincel tem borda; a borracha não.
        if eraser {
            let track = self
                .store
                .slider(ids::FLIP_OPACITY)
                .map(|(_, v)| v)
                .unwrap_or(snap.opacity);
            let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent chip
            return self.slider_row(
                "Strength", // é o que a opacidade SIGNIFICA para a borracha
                ids::FLIP_OPACITY,
                ids::FLIP_OPACITY_NUM,
                track,
                pct,
                &format!("{}", pct.round() as i64),
                y,
            );
        }
        let track = self
            .store
            .slider(ids::FLIP_HARDNESS)
            .map(|(_, v)| v)
            .unwrap_or(snap.hardness);
        y = self.slider_row(
            "Hardness",
            ids::FLIP_HARDNESS,
            ids::FLIP_HARDNESS_NUM,
            track,
            f64::from(track),
            &format!("{track:.2}"),
            y,
        );
        // Opacity (0..100 %).
        let track = self
            .store
            .slider(ids::FLIP_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or(snap.opacity);
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent chip
        y = self.slider_row(
            "Opacity",
            ids::FLIP_OPACITY,
            ids::FLIP_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );
        // Smoothing (0..1) — the "settle".
        let track = self
            .store
            .slider(ids::FLIP_SMOOTHING)
            .map(|(_, v)| v)
            .unwrap_or(snap.smoothing);
        self.slider_row(
            "Smoothing",
            ids::FLIP_SMOOTHING,
            ids::FLIP_SMOOTHING_NUM,
            track,
            f64::from(track),
            &format!("{track:.2}"),
            y,
        )
    }

    /// Color section — a Stroke colour swatch (opens the shared OKLCH picker).
    ///
    /// **Só no modo Draw**: a cor do TRAÇO não é atributo da borracha nem do balde (que
    /// tem a cor própria dele, na seção Fill).
    pub(crate) fn color(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        if snap.mode != FlipMode::Draw {
            return y;
        }
        y = self.section_label("Color", y);
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            "Stroke",
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let swatch = ColorSwatch::new(ids::FLIP_STROKE_SWATCH, "Stroke color", snap.stroke)
            .size(SwatchSize::Md);
        paint_color_swatch(&swatch, swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::FLIP_STROKE_SWATCH, swatch_rect);
        y + self.row_h + self.row_gap
    }

    /// **Fill section** (W4) — only in Fill mode. The bucket's own colour, the
    /// animation-bucket modes (Paint / Behind / Unpaint) and the three knobs that
    /// decide whether the bucket "works first time": Gap Closure, Grow and Precision.
    pub(crate) fn fill_section(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        if snap.mode != FlipMode::Fill {
            return y;
        }
        y = self.section_label("Fill", y);

        // A cor do BALDE — própria, não a do traço (colorir usa outra paleta).
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            "Color",
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let swatch = ColorSwatch::new(ids::FLIP_FILL_SWATCH, "Fill color", snap.fill_color)
            .size(SwatchSize::Md);
        paint_color_swatch(&swatch, swatch_rect, self.scene, self.theme);
        self.hit_index.register(ids::FLIP_FILL_SWATCH, swatch_rect);
        y += self.row_h + self.row_gap;

        // Paint / Behind / Unpaint — a semântica de balde de ANIMAÇÃO (Toon Boom).
        y = self.segmented(
            "Bucket",
            [
                (
                    ids::FLIP_FILL_PAINT,
                    "Paint",
                    snap.fill_mode == FillMode::Paint,
                ),
                (
                    ids::FLIP_FILL_BEHIND,
                    "Behind",
                    snap.fill_mode == FillMode::PaintBehind,
                ),
                (
                    ids::FLIP_FILL_UNPAINT,
                    "Unpaint",
                    snap.fill_mode == FillMode::Unpaint,
                ),
            ],
            y,
        );

        // Gap Closure (px de tela; 0 = desligado) — o knob que salva o line-art aberto.
        let track = self
            .store
            .slider(ids::FLIP_GAP)
            .map(|(_, v)| v)
            .unwrap_or((snap.gap_px / GAP_MAX_PX) as f32);
        let gap = f64::from(track) * GAP_MAX_PX;
        y = self.slider_row(
            "Gap",
            ids::FLIP_GAP,
            ids::FLIP_GAP_NUM,
            track,
            gap,
            &format!("{}", gap.round() as i64),
            y,
        );
        // Grow/Shrink (px do buffer): positivo enfia a cor por baixo da linha.
        let track = self
            .store
            .slider(ids::FLIP_GROW)
            .map(|(_, v)| v)
            .unwrap_or(((snap.grow - GROW_MIN) / (GROW_MAX - GROW_MIN)) as f32);
        let grow = GROW_MIN + f64::from(track) * (GROW_MAX - GROW_MIN);
        y = self.slider_row(
            "Grow",
            ids::FLIP_GROW,
            ids::FLIP_GROW_NUM,
            track,
            grow,
            &format!("{}", grow.round() as i64),
            y,
        );
        // Precision: resolução do buffer do balde.
        let track = self
            .store
            .slider(ids::FLIP_PRECISION)
            .map(|(_, v)| v)
            .unwrap_or(((snap.precision - PRECISION_MIN) / (PRECISION_MAX - PRECISION_MIN)) as f32);
        let prec = PRECISION_MIN + f64::from(track) * (PRECISION_MAX - PRECISION_MIN);
        self.slider_row(
            "Precision",
            ids::FLIP_PRECISION,
            ids::FLIP_PRECISION_NUM,
            track,
            prec,
            &format!("{prec:.1}"),
            y,
        )
    }

    /// Erase sub-mode row (Soft / Hard / Stroke) — only in Erase mode.
    pub(crate) fn erase_row(&mut self, snap: &FlipStyleSnapshot, y: f32) -> f32 {
        if snap.mode != FlipMode::Erase {
            return y;
        }
        self.segmented(
            "Erase",
            [
                (ids::FLIP_ERASE_SOFT, "Soft", snap.erase == EraseMode::Soft),
                (ids::FLIP_ERASE_HARD, "Hard", snap.erase == EraseMode::Hard),
                (
                    ids::FLIP_ERASE_STROKE,
                    "Stroke",
                    snap.erase == EraseMode::Stroke,
                ),
            ],
            y,
        )
    }
}
