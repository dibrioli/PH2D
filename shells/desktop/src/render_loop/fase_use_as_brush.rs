//! **Fase do quadro: USAR COMO FORMA / GRÃO DO PINCEL** — o menu da Hierarquia lê os pixels da linha e
//! instala-os como a Shape (silhueta) ou o Grain (textura) do pincel (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! PRECISION-READONLY: os pixels lidos pelo `read_sprite_source` viram a Shape (silhueta) ou o Grain
//! (textura) do pincel do Painter; a sprite da linha nunca é escrita de volta, então os seus 16 bits
//! não se perdem aqui.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_use_as_brush(
        &mut self,
        use_as_brush_texture_row: Option<NodeId>,
        use_as_brush_shape_row: Option<NodeId>,
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
        // Hierarchy "Use as Brush Shape / Grain" → read the right-clicked sprite's pixels, install
        // them as the brush Shape (silhouette) or Grain (texture) image (Rec.601 luminance, mirror of
        // the file-load path), and activate the brush tool so the user can paint immediately. A
        // non-image row toasts + no-ops. Shape wins if both fired in one frame.
        let use_as_brush_intent = use_as_brush_shape_row
            .map(|r| (r, true))
            .or(use_as_brush_texture_row.map(|r| (r, false)));
        if let Some((row, as_shape)) = use_as_brush_intent
            && let Some(live) = hero_live.as_ref()
            && let Some(bits) = live.bridge.entity_for(row)
        {
            // Active painter document: read the LIVE layers NON-DESTRUCTIVELY — Shape captures the
            // layer stack (so the per-layer-colour feature works), Grain composites to luminance.
            // Crucially this does NOT bake/re-push the sprite: a re-push runs `set_source`, which
            // resets the LayerStack and would DESTROY the user's layers — the flatten bug Enio hit
            // (replaces the old auto-commit path; Enio 2026-06-26).
            let on_active_doc = self.last_painter_pushed_entity == Some(bits);
            let mut handled = false;
            if on_active_doc {
                tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
                if let Some(painter) = tools.active_mut().and_then(|t| {
                    t.as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()
                }) {
                    if as_shape {
                        painter.capture_layers_as_brush_shape();
                        toasts.push(ph2d_editor_core::Toast::success(
                            "Brush shape set from layers",
                        ));
                        handled = true;
                    } else if let Some((lum, w, h)) = painter.composite_to_lum() {
                        painter.set_brush_texture_image(lum, w, h);
                        toasts.push(ph2d_editor_core::Toast::success(
                            "Brush grain set from sprite",
                        ));
                        handled = true;
                    }
                }
            }
            if !handled {
                // A different (flat) sprite in the hierarchy — read its baked texture (no layers to
                // lose), mirror of the file-load path.
                let entity = ph2d_ecs::Entity::from_bits(bits);
                match crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                ) {
                    Some(src) => {
                        let (w, h) = (src.image.width, src.image.height);
                        // Rec.601 luminance: weights 77/150/29 sum to 256, `>> 8` keeps `[0,255]`.
                        let lum: Vec<u8> = src
                            .image
                            .pixels
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .map(|p| {
                                ((u32::from(p[0]) * 77
                                    + u32::from(p[1]) * 150
                                    + u32::from(p[2]) * 29)
                                    >> 8) as u8
                            })
                            .collect();
                        // Reach the painter only via the active tool → activate it first.
                        tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
                        if let Some(painter) = tools.active_mut().and_then(|t| {
                            t.as_any_mut()
                                .downcast_mut::<ph2d_tool_painter::PainterTool>()
                        }) {
                            if as_shape {
                                // ⚠️ A COR do sprite viaja junto (Enio, 2026-08-09): a silhueta
                                // continua sendo a mesma luminância, byte a byte, mas o slot passa a
                                // guardar uma camada com o RGB, e é isso que dá ao checkbox "Use
                                // Texture Colors" o que ligar. Antes daqui saía só a máscara — a cor
                                // morria na conversão para cinza, e pintar com as cores da textura
                                // era possível apenas para o documento ABERTO no Painter.
                                painter.set_brush_shape_image_rgba(
                                    &src.image.pixels,
                                    w,
                                    h,
                                    Some(bits),
                                );
                                toasts.push(ph2d_editor_core::Toast::success(
                                    "Brush shape set from sprite",
                                ));
                            } else {
                                painter.set_brush_texture_image(lum, w, h);
                                toasts.push(ph2d_editor_core::Toast::success(
                                    "Brush grain set from sprite",
                                ));
                            }
                        }
                    }
                    None => {
                        let what = if as_shape {
                            "Brush Shape"
                        } else {
                            "Brush Grain"
                        };
                        toasts.push(ph2d_editor_core::Toast::warning(format!(
                            "Use as {what}: select an image sprite"
                        )));
                    }
                }
            }
            self.title_dirty = true;
        }
    }
}
