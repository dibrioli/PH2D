//! **O EDITOR DE ANIMAÇÕES** (GameEngineFuncional plano 07, `ph2d-app-sprite-anim::editor`): a aba que
//! abre do «Editar animações…» do cartão Animation.

/// A tradução de uma chave `anim_editor.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "anim_editor.tab" => "Animations \u{2014} {name}",
        "anim_editor.unnamed" => "Sprite",
        "anim_editor.play" => "\u{25b6}",
        "anim_editor.pause" => "\u{23f8}",
        "anim_editor.fps" => "{fps} fps",
        "anim_editor.repeat" => "Repeat",
        "anim_editor.once" => "Once",
        "anim_editor.ping_pong" => "Ping-pong",
        "anim_editor.cols" => "{n} cols",
        "anim_editor.rows" => "{n} rows",
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
        "anim_editor.strip_hint" => {
            "Click cells on the sheet, or drag across them, to add frames in that order."
        }
        "anim_editor.remove_frame" => "\u{d7}",
        "anim_editor.frame_factor" => "{x}x",
        "anim_editor.commit_failed" => "The animation edit did not apply: {e}",
        // O editor de Region (plano GEF 16 D2): a área que o objecto mostra.
        "region_editor.tab" => "Region \u{2014} {name}",
        "region_editor.columns" => "Columns",
        "region_editor.rows" => "Rows",
        "region_editor.hint" => {
            "Drag to move \u{00b7} handles resize \u{00b7} drag outside or click a cell for a new area \u{00b7} Alt: no snap"
        }
        "region_editor.hint_alt" => "Alt: snapping to pixels",
        "region_editor.ok" => "OK",
        "region_editor.cancel" => "Cancel",
        "region_editor.no_pixels" => "This sprite's image is not in memory, so it can't be shown.",
        _ => return None,
    })
}
