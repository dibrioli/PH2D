//! **Fase do quadro: USAR COMO PAPEL / GRANULAÇÃO** — o menu da Hierarquia lê os pixels da linha como
//! luminância e instala-os como o papel da aquarela (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! PRECISION-READONLY: os pixels lidos pelo `read_sprite_source` viram a luminância do papel (o slot
//! Grain do Painter, ancorado ao canvas); a sprite da linha nunca é escrita de volta, então os seus
//! 16 bits não se perdem aqui.

use super::*;
use ph2d_i18n::{tr, tr_with};

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_use_as_paper(
        &mut self,
        use_as_paper_row: Option<NodeId>,
        use_as_granulation_row: Option<NodeId>,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            tools,
            hero_live,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);
        // Hierarchy "Use as Watercolor Paper / Granulation" → read the row's pixels as luminance and
        // install them as the watercolor paper (Grain slot, canvas-anchored), turning the render-path
        // on so the wash granulates against the layer. Granulation wins if both fired in one frame.
        // Mirror of the "Use as Brush Grain" path above (`docs/Painter/10…` §5).
        let use_as_paper_intent = use_as_granulation_row
            .map(|r| (r, true))
            .or(use_as_paper_row.map(|r| (r, false)));
        if let Some((row, as_granulation)) = use_as_paper_intent
            && let Some(live) = hero_live.as_ref()
            && let Some(bits) = live.bridge.entity_for(row)
        {
            let on_active_doc = self.last_painter_pushed_entity == Some(bits);
            // Luminance: the active painter doc composites its layers (a Group of textures folds in);
            // a different flat sprite reads its baked texture (Rec.601, mirror of the file-load path).
            let lum_wh: Option<(Vec<u8>, u32, u32)> = if on_active_doc {
                tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
                tools
                    .active_mut()
                    .and_then(|t| {
                        t.as_any_mut()
                            .downcast_mut::<ph2d_tool_painter::PainterTool>()
                    })
                    .and_then(|p| p.composite_to_lum())
            } else {
                let entity = ph2d_ecs::Entity::from_bits(bits);
                crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                )
                .map(|src| {
                    let (w, h) = (src.image.width, src.image.height);
                    let lum: Vec<u8> = src
                        .image
                        .pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|p| {
                            ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29)
                                >> 8) as u8
                        })
                        .collect();
                    (lum, w, h)
                })
            };
            match lum_wh {
                Some((lum, w, h)) => {
                    tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
                    if let Some(painter) = tools.active_mut().and_then(|t| {
                        t.as_any_mut()
                            .downcast_mut::<ph2d_tool_painter::PainterTool>()
                    }) {
                        if as_granulation {
                            painter.use_layers_as_granulation(lum, w, h);
                            toasts.push(ph2d_editor_core::Toast::success(tr(
                                "shell.fase_use_as_paper.watercolor_granulation",
                            )));
                        } else {
                            painter.use_layers_as_watercolor_paper(lum, w, h);
                            toasts.push(ph2d_editor_core::Toast::success(tr(
                                "shell.fase_use_as_paper.watercolor_paper_set",
                            )));
                        }
                    }
                }
                None => {
                    let what = if as_granulation {
                        tr("shell.fase_use_as_paper.granulation")
                    } else {
                        tr("shell.fase_use_as_paper.watercolor_paper")
                    };
                    toasts.push(ph2d_editor_core::Toast::warning(tr_with(
                        "shell.fase_use_as_paper.use_as_select_an_image",
                        &[("what", &what)],
                    )));
                }
            }
            self.title_dirty = true;
        }
    }
}
