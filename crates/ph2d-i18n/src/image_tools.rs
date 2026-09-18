//! **AS STRINGS DOS PAINÉIS DAS FERRAMENTAS DE IMAGEM** — Color Equalization, Background Removal,
//! Equalize Sizes, Upscale e Padding (`panel.color_eq.*`, `panel.bg_removal.*`,
//! `panel.equalize_sizes.*`, `panel.upscale.*`, `panel.padding.*`).
//!
//! ⚠️ **Um corte por ASSUNTO**: cinco painéis irmãos, o mesmo contrato (`RasterEditTool`), e as
//! mesmas três acções no pé (*Reset to Defaults* · *Cancel* · *Apply*) — cada painel com as suas
//! chaves, porque a secção que o artista vê é a do painel. Migrado em 2026-09-16 por
//! `scripts/migrar-texto-pintado.py` (mapas `docs/UI_New_and_Simple/ferramentas/seccoes_<painel>.tsv`):
//! os cinco escreviam **107** textos no fonte.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave dos painéis das ferramentas de imagem, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── LETRAS SOLTAS, que a régua lexical não conta por CONSTRUÇÃO ────────────────────────
        // ⚠️⚠️ Os dois chips do modo `Fixed` (largura e altura) pintavam `"W"` e `"H"` crus com o
        //    censo desta crate VERDE: o `is_language` exige DUAS letras SEGUIDAS, senão acusaria
        //    todo identificador, e uma letra sozinha não tem forma que o distinga de um. ⇒ o
        //    tipo é que passa a ser a cerca (`TextKey` no pintor), e a régua é o gate desta crate.
        "panel.equalize_sizes.fixed.w" => "W",
        "panel.equalize_sizes.fixed.h" => "H",
        // ph2d-migrar-texto:begin
        // ⛔ Era "Color EQ" e a aba dizia "Color Equalization" — uma fonte só desde que o
        // `Panel::TITLE` é a chave (2026-09-17); ganhou a palavra da ABA. ⚠️ O nome ABREVIADO que
        // esta família de facto precisa é o do CHIP da barra do topo, e esse tem chave própria
        // (`tool.color_equalization.label` = "CEQ"): encurtar o título do painel para caber num
        // chip era responder à pergunta do vizinho.
        "panel.color_eq.title" => "Color Equalization",
        "panel.color_eq.adjust.lut_1" => "LUT 1",
        "panel.color_eq.adjust.lut_2" => "LUT 2",
        "panel.color_eq.adjust.posterize" => "Posterize",
        "panel.color_eq.adjust.quantize" => "Quantize",
        "panel.color_eq.adjust.dither" => "Dither",
        "panel.color_eq.adjust.dither_on" => "Dither: On",
        "panel.color_eq.adjust.dither_strength" => "Dither Strength",
        "panel.color_eq.adjust.dither_grain" => "Dither Grain",
        "panel.color_eq.adjust.auto_levels" => "Auto Levels",
        "panel.color_eq.adjust.auto_levels_on" => "Auto Levels: On",
        "panel.color_eq.adjust.auto_contrast" => "Auto Contrast",
        "panel.color_eq.adjust.auto_contrast_on" => "Auto Contrast: On",
        "panel.color_eq.adjust.auto_colors" => "Auto Colors",
        "panel.color_eq.adjust.auto_colors_on" => "Auto Colors: On",
        "panel.color_eq.adjust.auto_wb" => "Auto WB",
        "panel.color_eq.adjust.auto_wb_on" => "Auto WB: On",
        "panel.color_eq.adjust.reset_to_defaults" => "Reset to Defaults",
        "panel.color_eq.adjust.cancel" => "Cancel",
        "panel.color_eq.adjust.apply" => "Apply",
        "panel.color_eq.adjust.off" => "Off",
        "panel.color_eq.adjust.n_levels" => "{level} Levels",
        "panel.color_eq.adjust.n_colors" => "{colors} Colors",
        "panel.color_eq.adjust.clip" => "Clip",
        "panel.color_eq.adjust.tile_grid" => "Tile Grid",
        "panel.color_eq.adjust.exposure" => "Exposure",
        "panel.color_eq.adjust.temperature" => "Temperature",
        "panel.color_eq.adjust.tint" => "Tint",
        "panel.color_eq.adjust.brightness" => "Brightness",
        "panel.color_eq.adjust.contrast" => "Contrast",
        "panel.color_eq.adjust.vibrance" => "Vibrance",
        "panel.color_eq.adjust.saturation" => "Saturation",
        "panel.color_eq.adjust.sharpen" => "Sharpen",
        "panel.color_eq.adjust.radius" => "Radius",
        "panel.color_eq.adjust.lut_intensity" => "LUT Intensity",
        "panel.color_eq.adjust.lut_mix" => "LUT Mix",
        // ⛔ Era "Bg Removal" e a aba dizia "Background Removal" — ver a nota do `panel.color_eq.title`
        // acima. O chip da barra do topo continua com a abreviatura dele (`tool.bgremoval.label`).
        "panel.bg_removal.title" => "Background Removal",
        "panel.bg_removal.mask.tolerance" => "Tolerance",
        "panel.bg_removal.mask.feather" => "Feather",
        "panel.bg_removal.mask.refine" => "Refine",
        "panel.bg_removal.mask.grow" => "Grow",
        "panel.bg_removal.mask.separate_islands" => "Separate islands",
        "panel.bg_removal.mask.min_px" => "Min px",
        "panel.bg_removal.mask.add_area" => "Add area",
        "panel.bg_removal.mask.clear_added_areas" => "Clear added areas",
        "panel.bg_removal.mask.pick_colors" => "Pick colors",
        "panel.bg_removal.mask.extra_bg_colour" => "Extra bg colour",
        "panel.bg_removal.mask.detect_subject" => "Detect subject",
        "panel.bg_removal.mask.protect" => "Protect",
        "panel.bg_removal.mask.size" => "Size",
        "panel.bg_removal.mask.show_mask" => "Show mask",
        "panel.bg_removal.mask.clear_protection" => "Clear protection",
        "panel.bg_removal.mask.reset_to_defaults" => "Reset to Defaults",
        "panel.bg_removal.mask.cancel" => "Cancel",
        "panel.bg_removal.mask.apply" => "Apply",
        "panel.equalize_sizes.title" => "Equalize Sizes",
        "panel.equalize_sizes.size.max" => "Max",
        "panel.equalize_sizes.size.fixed" => "Fixed",
        "panel.equalize_sizes.size.grid" => "Grid",
        "panel.equalize_sizes.size.upscale_if_smaller" => "Upscale if smaller",
        "panel.equalize_sizes.size.lanczos" => "Lanczos",
        "panel.equalize_sizes.size.nearest" => "Nearest",
        "panel.equalize_sizes.size.epx" => "EPX",
        "panel.equalize_sizes.size.rasterize_after" => "Rasterize after",
        "panel.equalize_sizes.size.cell_from_grid_snap" => "Cell: {cell} px (from Grid Snap)",
        "panel.equalize_sizes.size.offset" => "Offset",
        "panel.equalize_sizes.size.final_size" => "Final size: {size} x {size} px",
        "panel.equalize_sizes.size.arrange_on_grid" => "Arrange on Grid (1 per cell)",
        "panel.equalize_sizes.actions.reset_to_defaults" => "Reset to Defaults",
        "panel.equalize_sizes.actions.cancel" => "Cancel",
        "panel.equalize_sizes.actions.apply" => "Apply",
        "panel.upscale.title" => "Upscale",
        "panel.upscale.scale.lanczos3" => "Lanczos3",
        "panel.upscale.scale.nearest" => "Nearest",
        "panel.upscale.scale.epx" => "EPX",
        "panel.upscale.scale.scale" => "Scale",
        "panel.upscale.scale.output_select_a_sprite" => "Output: select a sprite",
        "panel.upscale.scale.output_size" => {
            "Output: {ow} \u{00d7} {oh} px  \u{00b7}  from {iw} \u{00d7} {ih}"
        }
        "panel.upscale.scale.reset_to_defaults" => "Reset to Defaults",
        "panel.upscale.scale.cancel" => "Cancel",
        "panel.upscale.scale.apply" => "Apply",
        "panel.upscale.scale.lanczos3_smooth_gradients_photos_illustrationsult" => {
            "Lanczos3 \u{00b7} smooth gradients \u{00b7} photos / illustrations \u{00b7} default"
        }
        "panel.upscale.scale.nearest_keeps_hard_pixel_edges_pixel_art_tile_sprites" => {
            "Nearest \u{00b7} keeps hard pixel edges \u{00b7} pixel art / tile sprites"
        }
        "panel.upscale.scale.epxge_directed_pixel_art_upscale_any_whole_factor_1x_16x" => {
            "EPX \u{00b7} edge-directed pixel-art upscale \u{00b7} any whole factor 1x-16x"
        }
        "panel.padding.title" => "Padding",
        "panel.padding.padding.top" => "Top",
        "panel.padding.padding.right" => "Right",
        "panel.padding.padding.bottom" => "Bottom",
        "panel.padding.padding.left" => "Left",
        "panel.padding.padding.pivot_recenter" => "Pivot: Recenter",
        "panel.padding.padding.pivot_keep" => "Pivot: Keep",
        "panel.padding.padding.reset_to_defaults" => "Reset to Defaults",
        "panel.padding.padding.cancel" => "Cancel",
        "panel.padding.padding.apply" => "Apply",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
