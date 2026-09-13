//! **Fase do quadro: OS INSUMOS DO EXTRACT** — o passo do movimento de demo, a folha aberta no canvas,
//! as três pré-visualizações que trocam a textura de uma sprite, os px/m e a amostragem por omissão do
//! projecto, e o upload das texturas cozidas ANTES de o extract ler o `texture_id` delas (OBRA 2 da
//! `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Preparados aqui e LIDOS muito depois**: o `sim_extract::run` corre depois da timeline, da física
//! e dos sinais, e recebe-os pelo contexto [`ExtractInputs`] com os mesmos nomes.

use super::*;

/// **O CONTEXTO que os insumos do extract entregam ao resto do quadro** — os locais que ATRAVESSAM a
/// fronteira da fase, com os MESMOS nomes que o corpo do quadro já usava. Quem os lê é o
/// `sim_extract::run`, que corre depois da timeline, da física e dos sinais.
pub(super) struct ExtractInputs {
    /// O passo do movimento de demo, do relógio de parede e limitado (ver o corpo).
    pub(super) dt: f32,
    /// As pré-visualizações que trocam a textura de uma sprite no extract (Painter, BgRemoval, forma).
    pub(super) preview_overrides: Vec<sim_extract::PreviewOverride>,
    /// A sprite cuja folha está aberta no canvas, se houver.
    pub(super) sheet_preview: Option<ph2d_ecs::Entity>,
    /// Os px/m do projecto, para o `Sprite::resolve_anchor`.
    pub(super) ppm: f32,
    /// A amostragem por omissão das sprites `Inherit`, do filtro de imagem do projecto.
    pub(super) default_filter: ph2d_ecs::FilterMode,
}

impl crate::App {
    /// Ver o cabeçalho do módulo. O `None` é inalcançável (os guardas correram na `fase_chrome_clock`).
    pub(super) fn fase_extract_inputs(&mut self, wall_dt: f64) -> Option<ExtractInputs> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            hero_screen,
            logical_texture_map,
            ..
        } = FrameGfx::of(gfx);

        // Sim tick + extract — extracted to sibling `sim_extract.rs`
        // (Wave 3.2 stage A). Runs the bouncing-motion sim tick and
        // the ADR-0021 / ADR-0025 propagate-transforms + sprite
        // emit pass.
        // Demo bouncing-motion integrates ONCE per render frame, so it
        // must use the real wall-clock delta — not the fixed timestep —
        // or its speed scales with the frame rate. That was invisible
        // under vsync (~60 fps) but the non-blocking `Immediate` present
        // mode (stutter fix, 2026-05-21) uncaps the loop to hundreds of
        // fps, which made the sprites race + jitter. `wall_dt` makes the
        // motion frame-rate-independent (real-time, smooth at any fps);
        // clamped so a hitch / debugger pause can't teleport a sprite.
        // (The proper fixed-step substep integration lands with the M10
        // gameplay sim; this is the M5 demo's stop-gap.)
        let dt = (wall_dt as f32).min(1.0 / 30.0);
        // Lens F (2026-05-26): the Background-Removal live preview no
        // longer suppresses the sprite + paints a Vello overlay on
        // top; instead it injects a synthetic `PreviewOverride` that
        // swaps the entity's `RenderInstance.texture_id` for a
        // transient `IndividualTextureStore` slot owning the preview
        // pixels. Same `Rgba8UnormSrgb` + sprite shader + premul
        // blend as Apply → byte-for-byte parity. The GPU slot is
        // populated by `bgremoval_preview::dispatch` LATER in the
        // frame, so reading `self.bgremoval_preview_gpu` here picks
        // up last frame's upload (1-frame lag is invisible — the
        // preview is a continuous animation).
        // **A FOLHA ABERTA** (Enio, 2026-08-23: *«você digita 8 quadros e não vê onde eles começam
        // ou terminam»*): com a caixa marcada, a sprite selecionada desdobra a grelha dela em
        // células fantasma no canvas.
        //
        // ⚠️ **Lido direto do `WidgetStore`, sem passar pelo barramento**, e a razão é o que isto
        // É: uma VISTA. Uma `EditorAction` levá-lo-ia ao commit, ao undo e ao save — e o artista
        // reabriria o projeto com a folha aberta sem se lembrar de a ter aberto. O store já é a
        // porta de outros pedidos de vista deste mesmo painel (`take_sheet_size_request`).
        let sheet_preview: Option<ph2d_ecs::Entity> =
            hero_screen.as_ref().and_then(sim_extract_sheet::previewed);
        let bgremoval_preview_override: Option<sim_extract::PreviewOverride> = self
            .bgremoval_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                // Byte-space premul upload + Apply uses the same flag.
                premultiplied: true,
            });
        // W3 Painter sprite-suppression: the active sprite's `preview_override`.
        // The base layer composite REPLACES the source sprite in-place (no
        // overlay duplication) so its opacity/representation affects the whole
        // image.
        let painter_preview_override: Option<sim_extract::PreviewOverride> = self
            .painter_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                premultiplied: true,
            });
        // A sprite used as the brush Shape but NOT currently selected previews its OWN composite too, so
        // brush opacity/blend remote-control edits show on it in real time (its `IndividualTextureStore`
        // slot, driven by the painter bridge). Distinct entity from the active sprite ⇒ a SEPARATE override.
        let painter_shape_source_override: Option<sim_extract::PreviewOverride> = self
            .painter_shape_source_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                premultiplied: true,
            });
        // Several sprites can preview at once now (active + shape-source); `sim_extract` matches each
        // sprite to its own entry. Painter and BgRemoval are never active simultaneously (one active tool).
        let preview_overrides: Vec<sim_extract::PreviewOverride> = [
            painter_preview_override,
            bgremoval_preview_override,
            painter_shape_source_override,
        ]
        .into_iter()
        .flatten()
        .collect();
        // Project px/m for `Sprite::resolve_anchor` (intrinsic-px `offset` →
        // local meters). `None` only under the M5 demo / headless, whose sprites
        // use the centered/offset defaults so the value is inert; fall back to
        // the canonical default.
        let ppm = hero_screen
            .as_ref()
            .map(|h| h.project.pixels_per_meter)
            .unwrap_or(ph2d_editor_core::project::DEFAULT_PIXELS_PER_METER);
        // W3.T3.11: project-default sampling for all-Inherit sprites, from the
        // project image filter (PixelArt → Nearest, Smooth → Linear); repeat
        // defaults to clamp (Disabled).
        let default_filter = match hero_screen
            .as_ref()
            .map(|h| h.project.image_filter)
            .unwrap_or(ph2d_render::ImageFilterMode::Smooth)
        {
            ph2d_render::ImageFilterMode::PixelArt => ph2d_ecs::FilterMode::Nearest,
            ph2d_render::ImageFilterMode::Smooth => ph2d_ecs::FilterMode::Linear,
        };
        // W2.T4 cooked-texture loader: resolve + decode + upload every
        // `SpriteSource::CookedTexture` sprite's KTX2 (for the device tier,
        // descending the fallback ladder) BEFORE extract reads back the cached
        // `texture_id`. Idempotent + cheap after the first upload.
        cooked_texture_bridge::ensure_uploaded(sim, renderer, asset_db, logical_texture_map);
        Some(ExtractInputs {
            dt,
            preview_overrides,
            sheet_preview,
            ppm,
            default_filter,
        })
    }
}
