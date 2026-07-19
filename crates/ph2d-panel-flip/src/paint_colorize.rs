//! O painter da **Colorize section** (C2) — irmão do [`crate::paint_sections`], separado
//! só pelo teto de LOC (a costura que o arquivo já tinha). Estende o mesmo [`BodyCtx`].

use crate::ids;
use crate::paint_sections::{BodyCtx, LABEL_COL_W};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, paint_color_swatch};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::ColorToken;
use ph2d_tool_flip::{FlipMode, FlipStyleSnapshot};

impl BodyCtx<'_> {
    /// **Colorize section** (C2) — a paleta do rabisco + as ações do gesto.
    ///
    /// O gesto (rabiscar) vive no canvas (modo Colorize); aqui ficam a COR do próximo
    /// rabisco e os botões que agem sobre os rabiscos ACUMULADOS: **Apply** roda o corte
    /// LazyBrush e materializa as regiões, **Clear** descarta os rabiscos. Os dois são ops do
    /// SHELL (mexem no buffer transiente + no documento), não da tool.
    pub(crate) fn colorize_section(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        if snap.mode != FlipMode::Colorize {
            return y;
        }
        y = self.section_label("Colorize", y);

        // A cor do PRÓXIMO rabisco — paleta própria (o picker OKLCH é compartilhado).
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
        let swatch = ColorSwatch::new(
            ids::FLIP_COLORIZE_SWATCH,
            "Colorize color",
            snap.colorize_color,
        )
        .size(SwatchSize::Md);
        paint_color_swatch(&swatch, swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::FLIP_COLORIZE_SWATCH, swatch_rect);
        y += self.row_h + self.row_gap;

        // Apply (roda o corte) · Clear (descarta os rabiscos).
        self.segmented(
            "Scribbles",
            [
                (ids::FLIP_COLORIZE_APPLY, "Apply", false),
                (ids::FLIP_COLORIZE_CLEAR, "Clear", false),
            ],
            y,
        )
    }
}
