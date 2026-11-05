//! **As palavras do Export e Import do PIXEL ART, P9b** — os nomes do Aseprite, em Plain English.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel_export.title" => "Pixel Export",
        "panel.pixel.export.title_as" => "Export As",
        "panel.pixel.export.title_sheet" => "Export Sprite Sheet",
        "panel.pixel.export.title_import" => "Import Sprite Sheet",
        "panel.pixel.export.reset" => "Reset",
        "panel.pixel.export.cancel" => "Cancel",
        "panel.pixel.export.export" => "Export",
        "panel.pixel.export.import" => "Import",
        // Os cartões.
        "panel.pixel.export.sec_file" => "File",
        "panel.pixel.export.sec_content" => "What goes out",
        "panel.pixel.export.sec_preview" => "Preview",
        "panel.pixel.export.sec_layout" => "Layout",
        "panel.pixel.export.sec_sprite" => "Sprite",
        "panel.pixel.export.sec_borders" => "Borders",
        "panel.pixel.export.sec_output" => "Output",
        "panel.pixel.export.sec_image" => "Image",
        "panel.pixel.export.sec_grid" => "Grid",
        // Export As.
        "panel.pixel.export.format" => "Format",
        "panel.pixel.export.fmt_png" => "PNG",
        "panel.pixel.export.fmt_gif" => "GIF",
        "panel.pixel.export.fmt_apng" => "APNG",
        "panel.pixel.export.fmt_webp" => "WebP",
        "panel.pixel.export.fmt_ase" => "ASE",
        "panel.pixel.export.hint_png" => {
            "One frame saves one file. Several frames save one numbered file each."
        }
        "panel.pixel.export.hint_gif" => {
            "GIF holds 256 colors and no partial transparency. If needed, colors are reduced and you are told."
        }
        "panel.pixel.export.hint_apng" => "An animated PNG, with all colors and transparency.",
        "panel.pixel.export.hint_webp" => "An animated WebP, with all colors and transparency.",
        "panel.pixel.export.hint_ase" => {
            "Saves the whole sprite: layers, frames, tags and palette. It is a copy: your drawing stays in the project."
        }
        "panel.pixel.export.layers" => "Layers",
        "panel.pixel.export.layers_visible" => "Visible",
        "panel.pixel.export.layers_all" => "All",
        "panel.pixel.export.frames" => "Frames",
        "panel.pixel.export.frames_all" => "All",
        "panel.pixel.export.frames_range" => "Range",
        "panel.pixel.export.from" => "From frame",
        "panel.pixel.export.to" => "To frame",
        "panel.pixel.export.direction" => "Direction",
        "panel.pixel.export.dir_forward" => "Forward",
        "panel.pixel.export.dir_reverse" => "Reverse",
        "panel.pixel.export.dir_pingpong" => "Ping-pong",
        "panel.pixel.export.dir_pingpong_reverse" => "Ping-pong reverse",
        "panel.pixel.export.scale" => "Scale (%)",
        "panel.pixel.export.loops" => "Loops (0 = forever)",
        // A folha.
        "panel.pixel.export.preview_size" => "{w} x {h} px, {n} {n?frame|frames}",
        "panel.pixel.export.preview_wait" => "The preview appears here.",
        "panel.pixel.export.sheet_type" => "Sheet type",
        "panel.pixel.export.type_horizontal" => "Horizontal strip",
        "panel.pixel.export.type_vertical" => "Vertical strip",
        "panel.pixel.export.type_rows" => "By rows",
        "panel.pixel.export.type_columns" => "By columns",
        "panel.pixel.export.type_packed" => "Packed",
        "panel.pixel.export.limit" => "Limit",
        "panel.pixel.export.limit_none" => "No limit",
        "panel.pixel.export.limit_columns" => "Fixed columns",
        "panel.pixel.export.limit_rows" => "Fixed rows",
        "panel.pixel.export.limit_width" => "Fixed width",
        "panel.pixel.export.limit_height" => "Fixed height",
        "panel.pixel.export.limit_n" => "Limit value",
        "panel.pixel.export.merge" => "Merge duplicates",
        "panel.pixel.export.ignore_empty" => "Ignore empty",
        "panel.pixel.export.pow2" => "Power of 2",
        "panel.pixel.export.split_layers" => "Split layers",
        "panel.pixel.export.split_tags" => "Split tags",
        "panel.pixel.export.border" => "Border (px)",
        "panel.pixel.export.shape" => "Spacing (px)",
        "panel.pixel.export.inner" => "Inner (px)",
        "panel.pixel.export.extrude" => "Extrude (px)",
        "panel.pixel.export.trim_cels" => "Trim frames",
        "panel.pixel.export.trim_sprite" => "Trim sprite",
        "panel.pixel.export.json" => "Save JSON data",
        "panel.pixel.export.json_format" => "JSON format",
        "panel.pixel.export.json_hash" => "Hash",
        "panel.pixel.export.json_array" => "Array",
        "panel.pixel.export.json_tags" => "Tags",
        "panel.pixel.export.json_layers" => "Layers",
        "panel.pixel.export.json_slices" => "Slices",
        "panel.pixel.export.item_name" => "Frame name in JSON",
        "panel.pixel.export.item_name_hint" => "{title} {frame}.{extension}",
        // O Import.
        "panel.pixel.export.grid_x" => "Left (px)",
        "panel.pixel.export.grid_y" => "Top (px)",
        "panel.pixel.export.grid_w" => "Frame W (px)",
        "panel.pixel.export.grid_h" => "Frame H (px)",
        "panel.pixel.export.gap_x" => "Gap X (px)",
        "panel.pixel.export.gap_y" => "Gap Y (px)",
        "panel.pixel.export.order" => "Frame order",
        "panel.pixel.export.by_rows" => "By rows",
        "panel.pixel.export.by_columns" => "By columns",
        "panel.pixel.export.import_count" => "{c} x {r} cells, {n} {n?frame|frames}",
        "panel.pixel.export.import_none" => {
            "No whole cell fits this image. Make the frame smaller."
        }
        // Os avisos e as recusas.
        "pixel.io.default_name" => "sprite",
        "pixel.io.exporting" => "Exporting {name}",
        "pixel.io.err.stopped" => {
            "The export stopped because of an internal error. Nothing else changed."
        }
        "pixel.io.saved" => "Saved {path}",
        "pixel.io.saved_many" => "Saved {n} {n?file|files} next to {path}",
        "pixel.io.opened" => "Opened {name} as a new Pixel Sprite",
        "pixel.io.kind_images" => "Images and Aseprite files",
        "pixel.io.kind_png" => "PNG image",
        "pixel.io.kind_gif" => "GIF image",
        "pixel.io.kind_apng" => "Animated PNG",
        "pixel.io.kind_webp" => "WebP image",
        "pixel.io.kind_ase" => "Aseprite file",
        "pixel.io.warn.slices" => "This file has {n} {n?slice|slices}. Slices are not kept yet.",
        "pixel.io.warn.tilemap" => "A tilemap layer became a normal image layer.",
        "pixel.io.warn.palette_per_frame" => {
            "This file changes its palette between frames. The first palette was kept."
        }
        "pixel.io.warn.unknown_blend" => "A layer uses an unknown blend mode. It became Normal.",
        "pixel.io.warn.duration_cut" => {
            "A frame was too long for this format. It became 65,535 ms."
        }
        "pixel.io.warn.gif_colors" => {
            "The drawing has {n} colors. The GIF was reduced to the 256 closest, without dithering."
        }
        "pixel.io.warn.gif_alpha" => {
            "GIF has no partial transparency: each cell became solid or empty."
        }
        "pixel.io.err.detail" => "Could not do that: {e}",
        "pixel.io.err.disk" => "Could not save {path}: {e}",
        "pixel.io.err.type" => "This file type cannot be opened: {e}",
        "pixel.io.err.empty" => "This file has no image.",
        "pixel.io.err.sizes" => {
            "These images have different sizes, so they cannot be frames of one sprite."
        }
        "pixel.io.err.invalid" => "This file does not make a valid sprite.",
        "pixel.io.err.gif_size" => "This image is too large for a GIF (65,535 px at most).",
        "pixel.io.err.too_large" => "This scale is too large (65,535 px per side at most).",
        "pixel.io.err.ase_frames" => "{n} frames do not fit an .ase file (65,535 at most).",
        "pixel.io.err.ase_cel" => "A frame has pixels too far away for an .ase file.",
        "pixel.io.err.target_gone" => {
            "The layer or tag of the last export no longer exists. Use Export As again."
        }
        "pixel.io.err.preset" => "The last export cannot be repeated. Use Export As again.",
        _ => return None,
    })
}
