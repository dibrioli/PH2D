//! B25.6: a **porta única** da rota de cor de um lote. A decisão é uma função pura do spec do LOTE
//! (`stroke_spec`, com o Erase já posto) e do estado da ferramenta; o despachante só executa o `match`.
//! Quem precisa de saber «por onde este lote pinta» (a mistura opaca, uma sonda, um gate) pergunta aqui,
//! e as duas respostas não podem divergir.

use super::super::PaintMode;
use super::super::ramp_lut::RampLutOwner;
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushSpec, Dab};

#[cfg(test)]
thread_local! {
    /// A porta do último lote despachado neste fio: (rota, `eligible` do spec do lote). Só sob teste, para a
    /// ponte espiã do B25.8 provar «subiu ⇒ passou a porta» lote a lote.
    pub(in crate::tool::paint) static ULTIMA_PORTA: std::cell::Cell<Option<(RotaDeCor, bool)>> =
        const { std::cell::Cell::new(None) };
}

/// Por onde a cor de um lote é carimbada (a ordem dos ramos é a precedência).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tool::paint) enum RotaDeCor {
    /// `stamp_dabs_mistura_opaca`: a tela sob o dab entra na cor.
    MisturaOpaca,
    /// `stamp_dabs_pixel`: a ponta de píxel (L18).
    Pixel,
    /// `stamp_dabs_per_layer_dynamic`: Shape de várias camadas com dinâmica por dab.
    PorCamadaDinamica,
    /// `stamp_dabs_cached_color_rgba`: Shape de várias camadas com a cor da textura, orientação constante.
    PorCamadaRgba,
    /// `stamp_dabs_cached_color`: Shape de várias camadas, carimbo colorido em cache.
    PorCamadaCor,
    /// `stamp_dabs_cached_ramped`: rampa indexada pela cobertura em cache.
    RampaEmCache,
    /// `stamp_dabs_cached_ramp_color`: rampa do Grain (ou do padrão do Shape) num carimbo colorido.
    RampaCorEmCache,
    /// `stamp_dabs_ramped`: rampa por píxel.
    RampaPorPixel,
    /// `stamp_dabs_cached`: silhueta × Grain assados uma vez (pode subir à placa).
    EmCache,
    /// `stamp_dabs_canvas_cached`: Grain preso à tela, uma vez por traço.
    TelaEmCache,
    /// `stamp_dabs_per_pixel`: nenhum cache serve (ou o cap de Accumulate).
    PorPixel,
}

impl PainterTool {
    /// A rota de cor do lote `dabs` com o spec do lote `brush`. Só leitura: nada muda até o `match`.
    pub(in crate::tool::paint) fn rota_de_cor(&self, dabs: &[Dab], brush: &BrushSpec) -> RotaDeCor {
        // A MISTURA OPACA (só ligada, só a pintar): a tela sob o dab entra na cor (`stamp_route::mistura`).
        // ⛔ Pergunta ao BLEND, não à ferramenta: a camada Erase e a Blur do Composite medem a cobertura apagando
        // (`composite_acumulado::acumula_camada`), e a mistura não apaga — pintava a camada Erase e enchia a
        // película da Blur, que travava a Strength da Brush (Bugs #73, #75). A cor por texel (#77) é da rota.
        if brush.mistura_opaca.ligada()
            && matches!(self.paint.paint_mode, PaintMode::Paint)
            && !matches!(brush.blend, ph2d_painter_brush::BrushBlend::EraseAlpha)
        {
            return RotaDeCor::MisturaOpaca;
        }
        self.rota_da_pegada(dabs, brush)
    }

    /// A rota que a PEGADA do lote toma fora da mistura opaca — nunca [`RotaDeCor::MisturaOpaca`]. A mistura
    /// pergunta-lhe de que rota vem a cor por texel da pegada (Bug #77), e a resposta é a do despacho.
    pub(in crate::tool::paint) fn rota_da_pegada(
        &self,
        dabs: &[Dab],
        brush: &BrushSpec,
    ) -> RotaDeCor {
        // A ponta de PÍXEL (L18, só no Digital): o carimbo binário dos dabs que o trilho já ancorou.
        if self.pixel_vivo(brush) {
            return RotaDeCor::Pixel;
        }
        let has_shape_image = self.paint.shape_image.is_some();
        // Per-layer-colour Shape (multi-layer, mode on): the z-ordered tinted layers recomposite onto the
        // canvas. Guarded by an ACTIVE Image silhouette so a stale `per_layer_color` flag (e.g. after the
        // Shape was reset to None) can never route a non-Image Shape into the coloured path (Enio).
        // ⚠️ Esta rama NÃO consulta a porta da cobertura abaixo, então um traço com Per-Layer Color armado
        // ignora o cap de Accumulate. Pré-existente e de propósito (doc 25 §13.10: a máscara pinta pela lei
        // do brush); fica NOMEADO em vez de contrabandeado.
        if self.paint.shape_layers.is_color_mode() && brush.shape_silhouette_active(has_shape_image)
        {
            // Per-dab dynamics the constant-orientation cached path can't express → the per-pixel dynamic
            // path: Shape Rake / Random rotation, Randomize Color (`has_colour_jitter_amount`, the engine's
            // actual gate), Grain Jitter Rotate, or a canvas-fixed / Random Offset Grain (`!is_cacheable()`,
            // the SAME predicate the grayscale routes use: the dab-LOCAL coloured stamp ignores the Stencil
            // rect and gives every dab the same offset — Enio 2026-06-27, sweep 2026-07-12).
            // …e a orla em PÍXEIS (J3): o carimbo colorido em cache é de um raio, ela é de cada dab.
            let per_dab_dynamic = brush.shape_has_per_dab_rotation(has_shape_image)
                || brush.borda_px > 0.0
                // A imagem pela área (J4) mede o píxel da tela: a máscara assada é de outro raio.
                || brush.shape.lei.area
                || ph2d_painter_brush::pen::any_tilted(dabs)
                || brush.has_colour_jitter_amount()
                || brush.has_per_dab_rotation()
                || (brush.texture.is_active() && !brush.texture.is_cacheable())
                // O dente do papel é preso à tela: o carimbo colorido em cache não o leva.
                || brush.dente.is_some();
            return if per_dab_dynamic {
                RotaDeCor::PorCamadaDinamica
            } else if self.paint.shape_layers.any_texture_color() {
                // Texture colour with a CONSTANT orientation: the per-pixel RGB is pre-baked into each
                // layer's premul-RGBA stamp (was the dynamic per-pixel resample = the "FPS 60→10" cliff).
                RotaDeCor::PorCamadaRgba
            } else {
                RotaDeCor::PorCamadaCor
            };
        }
        // Grain Jitter-Rotate OR a per-dab Shape rotation (Rake / Random) → each dab needs its own
        // basis, so the constant-orientation caches are skipped (the per-pixel path resolves per dab).
        let per_dab_rotation = brush.has_per_dab_rotation()
            || brush.shape_has_per_dab_rotation(has_shape_image)
            || ph2d_painter_brush::pen::any_tilted(dabs);
        // A stroke that TRACKS its own coverage ([`Self::stroke_cover_wanted`] — the Accumulate-OFF cap)
        // cannot be served by the constant-orientation caches (no per-stroke buffer to thread). Same door
        // the threading routes ask, so the two cannot disagree.
        // …e a carga que se ESGOTA (o fall-off da L5) é um tecto que desce: a mesma porta (`traco_plano`).
        let accumulate_cap = self.stroke_cover_wanted(brush) || self.cobertura_tapada(brush);
        // A Colour Ramp owns the painted COLOUR (baked LUT): the Shape ramp or the Grain ramp. With NO
        // Grain the COVERAGE indexes the ramp and the StampMask already caches it → blit the cached mask
        // with `ramp[coverage]` (critical for the shape fills, which re-stamp the WHOLE shape every move).
        // Per-pixel otherwise: a Grain to index, per-dab rotation, or the Accumulate cap (Enio 2026-06-26).
        let grain_active = brush.texture.is_active();
        // The dab-capsule (Continuous Spacing) is per dab: no baked or canvas cache carries its step, nor
        // its colour under a Shape ramp (lista 26 A9) — only the per-pixel `stamp_dabs_ramped` honours it.
        let capsulas = brush.lays_capsules();
        let owner = self.color_ramp_owner(grain_active);
        if owner != RampLutOwner::None {
            let static_ok = brush.dab_mask_cacheable(has_shape_image)
                && !per_dab_rotation
                && !accumulate_cap
                && !capsulas;
            // The Shape's Texture Color over a PROCEDURAL pattern reads a value the coverage mask does
            // not hold (prompt 38) → the baked colour stamp, like a Grain.
            let padrao = self.padrao_do_shape_vira_cor(owner, has_shape_image);
            return if static_ok && !grain_active && !padrao {
                RotaDeCor::RampaEmCache
            } else if static_ok {
                RotaDeCor::RampaCorEmCache
            } else {
                RotaDeCor::RampaPorPixel
            };
        }
        // Whether either slot genuinely shapes the dab beyond the falloff: a Shape image (silhouette) or
        // an active Grain. A bare falloff brush (neither) stays on the per-pixel path, as before.
        let want_cache =
            brush.shape_silhouette_active(has_shape_image) || brush.texture.is_active();
        // Both slots static & View ⇒ bake silhouette × Grain once into the scale-invariant stamp (Blender).
        if want_cache
            && brush.dab_mask_cacheable(has_shape_image)
            && !per_dab_rotation
            && !accumulate_cap
            && !capsulas
        {
            return RotaDeCor::EmCache;
        }
        // Grain Tiled / Stencil is canvas-fixed but dab-independent — cache each canvas pixel's Grain
        // once per stroke + look it up per dab; the silhouette (Shape or falloff) stays per-pixel.
        if brush.texture.is_canvas_cacheable() && !per_dab_rotation && !accumulate_cap && !capsulas
        {
            return RotaDeCor::TelaEmCache;
        }
        // Per-pixel path (no cache applies, or the Accumulate cap): resolves the texture frame +
        // Randomize-Color colour per dab and threads the per-stroke coverage mask.
        RotaDeCor::PorPixel
    }
}
