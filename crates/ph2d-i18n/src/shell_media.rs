//! **O QUE A SHELL DIZ SOBRE MÍDIA** — as ferramentas de imagem, as folhas de sprites, importar e
//! exportar (imagem, SVG, tokens, projecto), o áudio e os diálogos de ficheiro.
//!
//! ⚠️ Irmã do `shell.rs`, cortada pelo tecto de LOC (796/700) e por assunto; o prefixo é o mesmo
//! (`shell.<ficheiro>.<frase>`) e a cadeia do `tr` pergunta às duas.

/// A tradução de uma chave `shell.*` de mídia, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "shell.ase_import.frames_of_x_need_a_x" => {
            "{frames} frames of {width}x{height} need a {sheet_w}x{sheet_h} sheet, and the limit is {MAX_SHEET_EDGE_PX}"
        }
        "shell.ase_import.read" => "read: {e}",
        "shell.ase_import.tag_was_not_imported" => "tag \"{name}\" was not imported ({e_})",
        "shell.ase_import.the_file_has_no_tags" => {
            "the file has no tags — one animation named \"{stem}\" covers all {frames} frames"
        }
        "shell.ase_import.has_per_frame_timing" => {
            "\"{t}\" has per-frame timing ({per_frame_ms} frames, {into}..{into2} ms) — the Frame ms field shows the \
             most common one"
        }
        "shell.forwarding.palette" => "Palette",
        "shell.forwarding.adobe_color" => "Adobe Color",
        "shell.forwarding.adobe_swatch_exchange" => "Adobe Swatch Exchange",
        "shell.forwarding.hex_list" => "Hex list",
        "shell.forwarding.gimp_palette" => "GIMP palette",
        "shell.forwarding.colour_palette" => "Colour palette",
        "shell.bgremoval.bg_removal_island_s_2" => {
            "Bg Removal · {islands} island(s), {spawned} spawned, {spawn_failed} failed · Cmd+Z restores"
        }
        "shell.bgremoval.bg_removal_island_s" => {
            "Bg Removal · {islands} island(s), {spawned} spawned · Cmd+Z restores"
        }
        "shell.bgremoval.island_acquire_failed" => "Island {n} acquire failed: {e}",
        "shell.bgremoval.bg_removal_separate" => "Bg Removal · Separate Islands",
        "shell.bgremoval.bg_removal_failed" => "Bg Removal failed: {err}",
        "shell.bgremoval.bg_removal_applied_cmd" => "Bg Removal applied · Cmd+Z to undo",
        "shell.bgremoval.bg_removal" => "Bg Removal",
        "shell.bgremoval.bg_removal_source" => {
            "Bg Removal: source unavailable (Atlas key missing or readback failed)"
        }
        "shell.color_equalization.color_eq_applied_cmd_z" => "Color EQ applied · Cmd+Z to undo",
        "shell.color_equalization.color_eq" => "Color EQ",
        "shell.color_equalization.color_eq_failed" => "Color EQ failed: {err}",
        "shell.color_equalization.color_eq_source" => {
            "Color EQ: source unavailable (Atlas key missing or readback failed)"
        }
        "shell.equalize_sizes.equalize_sizes" => "Equalize Sizes",
        "shell.equalize_sizes.equalize_sizes_failed" => "Equalize Sizes failed: {err}",
        "shell.equalize_sizes.equalize_sizes_target" => {
            "Equalize Sizes: target exceeds GPU texture limit ({max_dim} px max, would need {width} × {height} px)"
        }
        "shell.equalize_sizes.equalize_sizes_sprites" => {
            "Equalize Sizes · {applied} sprites · Cmd+Z to undo"
        }
        "shell.equalize_sizes.equalize_sizes_applied" => "Equalize Sizes applied · Cmd+Z to undo",
        "shell.equalize_sizes.equalize_sizes_sprite" => {
            "Equalize Sizes · {applied} sprite(s), {skipped} skipped · Cmd+Z to undo"
        }
        "shell.equalize_sizes.equalize_sizes_nothing" => "Equalize Sizes: nothing to change",
        "shell.equalize_sizes.equalize_sizes_no" => {
            "Equalize Sizes: no eligible sprites in selection"
        }
        "shell.make_square.made_square_px_cmd_z" => {
            "Made square · {size} × {size2} px · Cmd+Z to undo"
        }
        "shell.make_square.make_square" => "Make square",
        "shell.make_square.make_square_failed" => "Make Square failed: {err}",
        "shell.make_square.make_square_would" => {
            "Make Square would exceed GPU texture limit ({max_texture_dimension_2d} px max, would need {size} px)"
        }
        "shell.padding.padded_px_cmd_z_to" => "Padded · {width} × {height} px · Cmd+Z to undo",
        "shell.padding.padding" => "Padding",
        "shell.padding.padding_failed" => "Padding failed: {err}",
        "shell.padding.padding_would_exceed" => {
            "Padding would exceed GPU texture limit ({max_dim} px max, would need {width} × {height} px)"
        }
        "shell.padding.padding_nothing" => "Padding: nothing changed",
        "shell.padding.padding_unavailable" => "Padding unavailable for this sprite",
        "shell.padding.padding_nothing_to" => "Padding: nothing to apply (all edges 0)",
        "shell.painter.painter_applied_cmd_z" => "Painter applied · Cmd+Z to undo",
        "shell.painter.painter" => "Painter",
        "shell.painter.painter_failed" => "Painter failed: {err}",
        "shell.painter.painter_empty_canvas" => "Painter: empty canvas (no source pushed)",
        "shell.painter.painter_source" => {
            "Painter: source unavailable (Atlas key missing or readback failed)"
        }
        "shell.rasterize.rasterized_px_cmd_z_to" => {
            "Rasterized · {width} × {height} px · Cmd+Z to undo"
        }
        "shell.rasterize.rasterize" => "Rasterize",
        "shell.rasterize.rasterize_failed" => "Rasterize failed: {err}",
        "shell.rasterize.rasterize_would_exceed" => {
            "Rasterize would exceed GPU texture limit ({max_dim} px max, would need {width} × {height} px)"
        }
        "shell.rasterize.rasterize_already_at" => "Rasterize: already at identity Transform",
        "shell.rasterize.rasterize_source" => "Rasterize: source unavailable",
        "shell.rasterize.rasterize_unavailable" => "Rasterize unavailable for this sprite",
        "shell.trim_transparency.trimmed_px_cmd_z_to" => {
            "Trimmed · {width} × {height} px · Cmd+Z to undo"
        }
        "shell.trim_transparency.trim" => "Trim",
        "shell.trim_transparency.trim_failed" => "Trim failed: {err}",
        "shell.undo.sprites" => "{toast_done} · {label} ({n} sprites)",
        "shell.upscale.upscaled_px_cmd_z_to" => "Upscaled · {out_w} × {out_h} px · Cmd+Z to undo",
        "shell.upscale.upscale" => "Upscale",
        "shell.upscale.upscale_failed" => "Upscale failed: {err}",
        "shell.upscale.upscale_would_exceed" => {
            "Upscale would exceed GPU texture limit ({max_dim} px max, would need {out_w} × {out_h} px). Try a smaller scale factor."
        }
        "shell.upscale.upscale_source" => "Upscale: source unavailable",
        "shell.sprite_merge.converted_sprite_s_to" => {
            "Converted {downgraded} sprite(s) to RGBA8 — merging composites in 8-bit"
        }
        "shell.sprite_merge.merged_sprites" => "Merged {n_sources} sprites",
        "shell.sprite_merge.merged_sprites_skipped" => {
            "Merged {n_sources} sprites · skipped {skipped} non-sprite entries"
        }
        "shell.sprite_merge.merged" => "Merged",
        "shell.sprite_merge.layer" => "Layer",
        "shell.sprite_merge.merge_sprites_gpu" => "Merge Sprites: GPU upload failed: {e}",
        "shell.sprite_merge.merge_sprites_could" => {
            "Merge Sprites: could not read 2 source images (atlas miss or readback failed)"
        }
        "shell.sprite_merge.merge_sprites_select_2" => {
            "Merge Sprites: select 2 or more sprites first"
        }
        "shell.sprite_merge_warp.merge_sprites_output" => {
            "Merge Sprites: output {out_w}×{out_h} px exceeds device limit {max_dim} px"
        }
        "shell.sprite_merge_warp.merge_sprites" => "Merge Sprites: degenerate union bounding box",
        "shell.texture_edit.image_tools_work_in_8" => "image tools work in 8-bit",
        "shell.texture_edit.converted_to_rgba8" => "Converted to RGBA8 — {because}",
        "shell.image_export.export_failed" => "Export failed: {e}",
        "shell.image_export.exported_bytes" => "Exported: {picked} ({n} bytes)",
        "shell.image_export.could_not_write" => "could not write {path}: {e}",
        "shell.image_export.exported_as_rgba8" => {
            "Exported as RGBA8 — .{extension} cannot carry 16-bit; try .exr or .hdr"
        }
        "shell.image_export.no_exporter_for" => "no exporter for .{extension}",
        "shell.image_export.this_sprite_s_pixels" => "this sprite's pixels are unreadable",
        "shell.image_export.unknown_extension_try" => "unknown extension — try one of: {join}",
        "shell.import_router.shapes" => "{name} ({shapes} shapes)",
        "shell.import_router.frames_animations" => {
            "{name} ({frames} frames, {animations} animations)"
        }
        "shell.import_router.unnamed" => "(unnamed)",
        "shell.import_router.images" => "Images",
        "shell.import_router.vector_svg" => "Vector (SVG)",
        "shell.import_router.aseprite" => "Aseprite",
        "shell.import_router.all_supported" => "All supported",
        "shell.input_drop.drop_failed" => "Drop failed: {error}",
        "shell.input_drop.imported" => "Imported {label}",
        "shell.input_drop.skipped_not_an_image" => {
            "Skipped {name}: not an image, an SVG drawing or an Aseprite file"
        }
        "shell.input_drop.sheet" => "Sheet {name}: {error}",
        "shell.input_drop.sheet_sprites" => "Sheet {name}: {regions} sprites",
        "shell.merge_layers.merged_into_layers" => {
            "Merged into {made} layers — open the Painter on it to separate them again"
        }
        "shell.merge_layers.merged_into_of_layers" => {
            "Merged into {made} of {layers} layers — the rest did not fit"
        }
        "shell.merge_layers.merged_but_the_layered" => {
            "Merged, but the layered document could not be created (painter unavailable)"
        }
        "shell.precision_convert.format" => "Format · {label}",
        "shell.precision_convert.format_conversion" => "Format conversion failed: {e}",
        "shell.precision_convert.cannot_convert_source" => {
            "Cannot convert — source is not an image"
        }
        "shell.precision_convert.cannot_convert_source_2" => {
            "Cannot convert — source pixels missing"
        }
        "shell.precision_convert.cooked_textures_come" => {
            "Cooked textures come from the asset pipeline — format is read-only"
        }
        "shell.project_io.ph2d_project" => "PH2D project",
        "shell.project_io.ph2d_project_2" => "PH2D project (.{ext})",
        "shell.project_load.project_loaded_2" => "Project loaded · {tracks} animation track(s)",
        "shell.project_load.project_loaded" => "Project loaded",
        "shell.project_load.project_refused_its_4" => {
            "Project refused: its sprite images are from another version ({e})"
        }
        "shell.project_load.project_refused_its_3" => {
            "Project refused: its sculpture is from another version ({e})"
        }
        "shell.project_load.project_refused_its_2" => {
            "Project refused: its pattern artwork is from another version ({e})"
        }
        "shell.project_load.project_refused_its" => {
            "Project refused: its animation is from another version ({e})"
        }
        "shell.project_load.project_refused_file" => {
            "Project refused: file format {ver}, this build reads {PROJECT_SCHEMA}"
        }
        "shell.project_load.project_refused_format" => {
            "Project refused: format 95 file is unreadable ({e})"
        }
        "shell.project_load.project_migrated_from" => {
            "Project migrated from format 95 to {PROJECT_SCHEMA}"
        }
        "shell.project_load.project_refused_format_128" => {
            "Project refused: format 128 file is unreadable ({e})"
        }
        "shell.project_load.project_migrated_from_128" => {
            "Project migrated from format 128 to {PROJECT_SCHEMA}"
        }
        "shell.project_save.project_save_failed" => "Project save FAILED: {e}",
        "shell.project_save.project_saved_kb" => {
            "Project saved · {bytes} KB · {n} animation track(s)"
        }
        "shell.audio_overlay.audio_editor_waveform" => "Audio Editor \u{00b7} Waveform",
        "shell.audio_overlay.audio_editor" => "Audio Editor \u{00b7} Spectrogram",
        "shell.bgremoval_preview_gpu.preview_upload_failed" => {
            "Bg Removal: the preview upload to the GPU failed ({e}). Retrying next frame."
        }
        "shell.bgremoval_preview_gpu.mask_tint_upload_failed" => {
            "Bg Removal: the mask tint upload to the GPU failed ({e}). Retrying next frame."
        }
        "shell.color_equalization_bridge.color_equalization_gpu_2" => {
            "Color Equalization: GPU texture upload failed during deselect revert ({e})"
        }
        "shell.color_equalization_bridge.color_equalization_gpu_3" => {
            "Color Equalization: GPU texture upload failed during revert ({e})"
        }
        "shell.color_equalization_bridge.color_equalization_gpu" => {
            "Color Equalization: GPU texture upload failed ({e})"
        }
        "shell.fase_audio_editor.ph2d_variation_set" => "PH2D variation set",
        "shell.fase_audio_editor.ph2d_audio_preset" => "PH2D audio preset",
        "shell.fase_audio_editor.impulse_response" => "impulse response",
        "shell.fase_image_tool_activation.tool" => "Tool · {label}",
        "shell.fase_image_tool_activation.painter_kept_the_last" => {
            "Painter: kept the last selected sprite ({dropped} deselected)"
        }
        "shell.fase_new_image_modal.new_canvas_failed" => "New canvas failed: {e}",
        "shell.fase_new_image_modal.new_canvas" => "New canvas · {label} ({size}²)",
        "shell.fase_use_as_brush.use_as_select_an_image" => "Use as {what}: select an image sprite",
        "shell.fase_use_as_brush.brush_grain" => "Brush Grain",
        "shell.fase_use_as_brush.brush_shape" => "Brush Shape",
        "shell.fase_use_as_brush.brush_grain_set_from" => "Brush grain set from sprite",
        "shell.fase_use_as_brush.brush_shape_set_from_2" => "Brush shape set from sprite",
        "shell.fase_use_as_brush.brush_shape_set_from" => "Brush shape set from layers",
        "shell.fase_use_as_paper.use_as_select_an_image" => "Use as {what}: select an image sprite",
        "shell.fase_use_as_paper.watercolor_paper" => "Watercolor Paper",
        "shell.fase_use_as_paper.granulation" => "Granulation",
        "shell.fase_use_as_paper.watercolor_paper_set" => "Watercolor paper set from layer",
        "shell.fase_use_as_paper.watercolor_granulation" => "Watercolor granulation set from layer",
        "shell.image_edit_import.import_failed" => "Import failed: {error}",
        "shell.image_edit_import.imported" => "Imported {label}",
        "shell.image_edit_import.skipped_not_an_image" => {
            "Skipped {name}: not an image, an SVG drawing or an Aseprite file"
        }
        "shell.sheet_overlay.doesn_t_fit" => "  \u{00b7} DOESN'T FIT",
        "shell.sheet_overlay.overlap" => "  \u{00b7} OVERLAP",
        "shell.sheet_overlay.overlap_doesn_t_fit" => "  \u{00b7} OVERLAP \u{00b7} DOESN'T FIT",
        "shell.sheet_overlay.sprite_sheet" => "Sprite Sheet",
        "shell.tokens_bridge.is_not_a_length_a_px" => {
            "{v} is not a length: a px token needs a finite value >= 0"
        }
        "shell.tokens_bridge.can_t_make_follow_that" => {
            "Can't make {token} follow {target}: that closes a loop at {at}"
        }
        "shell.sheet_bake.sheet_baked_pieces" => {
            "Sheet baked: {regions} pieces share one {width}\u{00d7}{height} texture"
        }
        "shell.sheet_bake.bake_sheet_gpu_upload" => "Bake Sheet: GPU upload: {e}",
        "shell.sheet_bake.bake_sheet" => "Bake Sheet: {e}",
        "shell.sheet_bake.bake_sheet_this_sheet" => "Bake Sheet: this sheet has no pieces",
        "shell.sheet_bake.converted_piece_s_to" => {
            "Converted {downgraded} piece(s) to RGBA8 — a sheet is one texture, and it is 8-bit"
        }
        "shell.sheet_bake.bake_sheet_select_a" => "Bake Sheet: select a sheet",
        "shell.sheet_bake.bake_sheet_fix_it" => {
            "Bake Sheet: {what} - fix it first (Auto-Arrange Pieces)"
        }
        "shell.sheet_bake.some_pieces_fall" => "some pieces fall outside the sheet",
        "shell.sheet_bake.pieces_overlap" => "pieces overlap",
        "shell.sheet_bake.pieces_overlap_and" => "pieces overlap and some fall outside",
        "shell.sheet_export.export_sheet_image" => {
            "Export Sheet: image written, but metadata failed ({json}): {e}"
        }
        "shell.sheet_export.export_sheet_could_not" => "Export Sheet: could not write {png}: {e}",
        "shell.sheet_export.png" => "PNG",
        "shell.sheet_frame.the_sheet_rectangle" => "the sheet rectangle could not be created",
        "shell.sheet_frame.select_a_sprite_sheet" => "select a sprite sheet, or sprites to pack",
        "shell.sheet_frame.select_at_least_one" => "select at least one sprite first",
        "shell.sheet_frame.sprite_sheet" => "Sprite Sheet",
        "shell.sheet_frame.sheet" => "Sheet: {e}",
        "shell.sheet_frame.sheet_pieces_packed" => {
            "Sheet: {pieces} pieces packed into {size_px} \u{00d7} {size_px}"
        }
        "shell.sheet_frame.sheet_re_packed_pieces" => "Sheet re-packed: {moved} pieces",
        "shell.sheet_import.gpu_upload" => "GPU upload: {e}",
        "shell.sheet_import.the_metadata_declares" => "the metadata declares no frames",
        "shell.sheet_import.metadata_says_x_but_is" => {
            "metadata says {image_size}x{image_size2} but {image_filename} is {width}x{height} — re-export both"
        }
        "shell.sheet_import.is_not_an_image" => "{image_filename} is not an image",
        "shell.sheet_import.vanished_after_decode" => "{image_filename} vanished after decode",
        "shell.sheet_import.decode" => "decode {image_filename}: {e}",
        "shell.sheet_import.not_found_next_to_the" => {
            "{image_filename} not found next to the metadata ({e})"
        }
        "shell.sheet_import.the_metadata_file_has" => "the metadata file has no directory",
        "shell.sheet_import.read" => "read: {e}",
        "shell.vec_svg_export.export_svg_failed" => "Export SVG FAILED: {e}",
        "shell.vec_svg_export.exported_shape_s_to" => {
            "Exported {formas} shape(s) to {to_string_lossy}{extra}"
        }
        "shell.vec_svg_export.approximated" => " ({aproximadas} approximated)",
        "shell.vec_svg_export.export_svg_the_drawing" => {
            "Export SVG: the drawing has no visible shape"
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
