//! **O EDITOR DE ANIMAÇÕES** (GameEngineFuncional plano 07, `ph2d-app-sprite-anim::editor`): a aba que
//! abre do «Editar animações…» do cartão Animation.

/// A tradução de uma chave `anim_editor.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "anim_editor.tab" => "Animations \u{2014} {name}",
        "anim_editor.unnamed" => "Sprite",
        "anim_editor.animations" => "Animations",
        "anim_editor.sheet" => "SHEET",
        // As folhas no editor (plano GEF 26 §3.4).
        "anim_editor.sheet_n" => "Sheet {n}",
        "anim_editor.sheet_pick" => "{name} \u{25BE}",
        "anim_editor.sheet_item" => "{name} \u{b7} {cols} \u{00d7} {rows}",
        "anim_editor.sheet_item_on" => "{name} \u{b7} {cols} \u{00d7} {rows} \u{2713}",
        "anim_editor.empty_frame" => "\u{26A0}",
        "anim_editor.insert_here" => "INSERT HERE",
        "anim_editor.sheet_hint" => "\u{b7} drag across cells to add frames",
        "anim_editor.frames_title" => "{name} \u{b7} {n} {n?frame|frames}",
        "anim_editor.total_ms" => "{ms} ms total",
        "anim_editor.this_frame" => "This frame (x)",
        "anim_editor.no_frame" => "\u{2014}",
        "anim_editor.glyph_factor" => "F",
        "anim_editor.more_bands" => "+{n}",
        "anim_editor.repeat" => "Repeat",
        "anim_editor.once" => "Once",
        "anim_editor.ping_pong" => "Ping-pong",
        "anim_editor.cols_title" => "Cols",
        "anim_editor.rows_title" => "Rows",
        "anim_editor.glyph_cols" => "C",
        "anim_editor.glyph_rows" => "R",
        // A grelha que a transparência da folha propõe, e o botão que a aplica.
        "anim_editor.looks_like" => "This sheet looks like {cols} \u{d7} {rows} cells.",
        "anim_editor.use_grid" => "Use",
        // Um campo aberto: o texto e o cursor no fim.
        "anim_editor.caret" => "{text}|",
        // Entre as posições de uma célula que aparece mais de uma vez («1, 3»).
        "anim_editor.order_sep" => ", ",
        "anim_editor.no_pixels" => {
            "This sprite's image is not in memory, so the sheet can't be shown."
        }
        "anim_editor.no_animations" => "No animations yet. Press + Add.",
        "anim_editor.row_playing" => "\u{25b6} {name}",
        "anim_editor.add" => "+ Add",
        "anim_editor.remove" => "Remove",
        "anim_editor.split_by_rows" => "Split by rows",
        "anim_editor.split_nothing" => "Every row with a drawing already has its animation.",
        "anim_editor.row_name" => "row {n}",
        "anim_editor.marker_add" => "Add marker",
        "anim_editor.marker_remove" => "Remove marker",
        "anim_editor.marker_hint" => "Click a frame to mark it",
        "anim_editor.strip_hint" => {
            "Click cells on the sheet, or drag across them, to add frames in that order."
        }
        "anim_editor.remove_frame" => "\u{d7}",
        "anim_editor.frame_factor" => "{x}x",
        // A prévia Align (GEF 28, onda B).
        "anim_editor.align" => "Align",
        "anim_editor.align_previous" => "Previous",
        "anim_editor.align_reference" => "Reference",
        "anim_editor.align_ref_is" => "Reference: {sheet} \u{b7} cell {n}",
        "anim_editor.align_hint" => {
            "Drag: move the sheet \u{b7} corners: sheet size \u{b7} Alt+drag or arrows: this frame only (Shift: 10 px)"
        }
        "anim_editor.use_as_reference" => "Use as reference",
        "anim_editor.align_pick" => "Pick",
        "anim_editor.align_picking" => "Pick a frame below",
        "anim_editor.align_by_feet" => "By feet",
        "anim_editor.align_by_centre" => "By centre",
        "anim_editor.commit_failed" => "The animation edit did not apply: {e}",
        // O editor de Region (plano GEF 16 D2): a área que o objecto mostra.
        "region_editor.tab" => "Region \u{2014} {name}",
        "region_editor.columns" => "Columns",
        "region_editor.rows" => "Rows",
        "region_editor.hint" => {
            "Drag to move \u{00b7} handles resize \u{00b7} drag outside or click a cell for a new area \u{00b7} Alt: no snap"
        }
        "region_editor.hint_alt" => "Alt: snapping to pixels",
        "region_editor.hint_free" => {
            "Snap off: the area moves pixel by pixel \u{00b7} Alt: snap to the grid"
        }
        "region_editor.hint_alt_snap" => "Alt: snapping to the grid",
        "region_editor.snap" => "Snap",
        "region_editor.ok" => "OK",
        "region_editor.cancel" => "Cancel",
        "region_editor.no_pixels" => "This sprite's image is not in memory, so it can't be shown.",
        _ => return None,
    })
}
