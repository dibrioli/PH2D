//! **O seletor de *tip* pontilhado** (Draw mode, 03 §8) — a ponta do pincel ao longo do
//! traço: a linha cheia de sempre, ou contas (redondas/quadradas) espaçadas por arco.
//!
//! Módulo irmão de `paint_sections.rs` (que bateu o teto de LOC do painel) — a mesma
//! `BodyCtx`, um pedaço da COMPOSIÇÃO do modo Draw.

use crate::ids;
use crate::paint_sections::BodyCtx;
use ph2d_tool_flip::{DOT_SPACING_MAX, FlipMode, FlipStyleSnapshot, StrokeTip};

impl BodyCtx<'_> {
    /// A linha **Tip** [Line | Dots | Squares] + o slider **Spacing** (só com contas).
    /// Só no Draw — o *tip* é atributo do traço desenhado. Fora do Draw, no-op.
    pub(crate) fn tip_section(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        if snap.mode != FlipMode::Draw {
            return y;
        }
        let is = |t| snap.tip == t;
        y = self.segmented(
            "Tip",
            [
                (ids::FLIP_TIP_LINE, "Line", is(StrokeTip::Continuous)),
                (ids::FLIP_TIP_DOTS, "Dots", is(StrokeTip::Dots)),
                (ids::FLIP_TIP_SQUARES, "Squares", is(StrokeTip::Squares)),
            ],
            y,
        );
        // Spacing — só quando há contas: o Continuous não tem vão, e um controle que não faz
        // nada é a doutrina modal deste painel proibindo (some fora dos pontos).
        if snap.tip != StrokeTip::Continuous {
            let track = self
                .store
                .slider(ids::FLIP_DOT_SPACING)
                .map(|(_, v)| v)
                .unwrap_or((snap.dot_spacing / DOT_SPACING_MAX) as f32);
            let world = self
                .store
                .number_value(ids::FLIP_DOT_SPACING_NUM)
                .unwrap_or(snap.dot_spacing);
            y = self.slider_row(
                "Spacing",
                ids::FLIP_DOT_SPACING,
                ids::FLIP_DOT_SPACING_NUM,
                track,
                world,
                &format!("{world:.2}"),
                y,
            );
        }
        // **Self Overlap** (03 §8) — o toggle de auto-sobreposição com acúmulo. Um chip
        // largura-cheia (segmented de 1 opção, sem caption): destaca quando ligado, e o
        // clique ALTERNA (o `PanelEvent::Click` cai no arm de toggle da tool). Só no Draw.
        y = self.segmented(
            "",
            [(ids::FLIP_SELF_OVERLAP, "Self Overlap", snap.self_overlap)],
            y,
        );
        y
    }
}
