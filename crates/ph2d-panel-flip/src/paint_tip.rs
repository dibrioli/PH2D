//! **O seletor de *tip* pontilhado** (Draw mode, 03 §8) — a ponta do pincel ao longo do
//! traço: a linha cheia de sempre, ou contas (redondas/quadradas) espaçadas por arco.
//!
//! Módulo irmão de `paint_sections.rs` (que bateu o teto de LOC do painel) — a mesma
//! `BodyCtx`, um pedaço da COMPOSIÇÃO do modo Draw.

use crate::ids;
use crate::paint_sections::BodyCtx;
use ph2d_i18n::tr;
use ph2d_tool_flip::{Cap, DOT_SPACING_MAX, FlipMode, FlipStyleSnapshot, StrokeTip};

impl BodyCtx<'_> {
    /// A linha **Tip** [Line | Dots | Squares] + o slider **Spacing** (só com contas).
    /// Só no Draw — o *tip* é atributo do traço desenhado. Fora do Draw, no-op.
    pub(crate) fn tip_section(&mut self, snap: &FlipStyleSnapshot, mut y: f32) -> f32 {
        if snap.mode != FlipMode::Draw {
            return y;
        }
        let is = |t| snap.tip == t;
        y = self.segmented(
            tr("panel.flip.tip.tip"),
            [
                (
                    ph2d_tool_flip::ids::FLIP_TIP_LINE,
                    tr("panel.flip.tip.line"),
                    is(StrokeTip::Continuous),
                ),
                (
                    ph2d_tool_flip::ids::FLIP_TIP_DOTS,
                    tr("panel.flip.tip.dots"),
                    is(StrokeTip::Dots),
                ),
                (
                    ph2d_tool_flip::ids::FLIP_TIP_SQUARES,
                    tr("panel.flip.tip.squares"),
                    is(StrokeTip::Squares),
                ),
            ],
            y,
        );
        // Spacing — só quando há contas: o Continuous não tem vão, e um controle que não faz
        // nada é a doutrina modal deste painel proibindo (some fora dos pontos).
        if snap.tip != StrokeTip::Continuous {
            let track = self
                .store
                .slider(ph2d_tool_flip::ids::FLIP_DOT_SPACING)
                .map(|(_, v)| v)
                .unwrap_or((snap.dot_spacing / DOT_SPACING_MAX) as f32);
            let world = self
                .store
                .number_value(ids::FLIP_DOT_SPACING_NUM)
                .unwrap_or(snap.dot_spacing);
            y = self.slider_row(
                tr("panel.flip.tip.spacing"),
                ph2d_tool_flip::ids::FLIP_DOT_SPACING,
                ids::FLIP_DOT_SPACING_NUM,
                track,
                world,
                &format!("{world:.2}"),
                y,
            );
        }
        // **Cap** — a PONTA do traço: o disco do pincel (Round) ou o corte reto (Flat, o `butt` do
        // SVG). Fica logo abaixo do Tip porque as duas respondem *"que forma tem a ponta?"* — uma
        // ao LONGO do traço, outra na EXTREMIDADE dele.
        //
        // ⚠️ **Oferecido com QUALQUER tip, e é medido, não suposto:** com contas o `Flat` corta a
        // fita antes da última conta, então ele não é inerte ali; escondê-lo seria decidir por um
        // artista que talvez queira exatamente isso.
        y = self.segmented(
            tr("panel.flip.tip.cap"),
            [
                (
                    ph2d_tool_flip::ids::FLIP_CAP_ROUND,
                    tr("panel.flip.tip.round"),
                    snap.cap == Cap::Round,
                ),
                (
                    ph2d_tool_flip::ids::FLIP_CAP_FLAT,
                    tr("panel.flip.tip.flat"),
                    snap.cap == Cap::Flat,
                ),
                (
                    ph2d_tool_flip::ids::FLIP_CAP_SQUARE,
                    tr("panel.flip.tip.square"),
                    snap.cap == Cap::Square,
                ),
            ],
            y,
        );
        // **Self Overlap** (03 §8) — o toggle de auto-sobreposição com acúmulo. Um chip
        // largura-cheia (segmented de 1 opção, sem caption): destaca quando ligado, e o
        // clique ALTERNA (o `PanelEvent::Click` cai no arm de toggle da tool). Só no Draw.

        // **Airbrush** (03 §8) — o toggle do pincel airbrush analítico (falloff físico de dab
        // esférico; o slider Hardness vira a densidade). Mesmo idioma do chip acima. Só no Draw.
        // ⭐ Os dois são UM corpo (wave 20): duas fileiras de uma peça, encostadas. Eles são a
        //    mesma pergunta — *como é que o pincel se comporta?* —, e o Cap acima é outra (*que
        //    forma tem a ponta?*), por isso fica separado.
        y = self.segmented_block(
            "",
            &[
                (
                    ph2d_tool_flip::ids::FLIP_SELF_OVERLAP,
                    tr("panel.flip.tip.self_overlap"),
                    snap.self_overlap,
                ),
                (
                    ph2d_tool_flip::ids::FLIP_AIRBRUSH,
                    tr("panel.flip.tip.airbrush"),
                    snap.airbrush,
                ),
            ],
            &[1, 1],
            y,
        );
        y
    }
}
