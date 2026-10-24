//! **As frases da biblioteca de macros** (W18, ADR-0190): o que guardar, exportar, importar e aplicar
//! dizem ao artista (o `aviso` do painel MACROS e a recusa do «Aplicar por cima»).

/// A tradução de uma chave `app.macros.lib.*` / `app.macros.apply.*` / `app.macros.asset.*`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "app.macros.lib.family_cannot_transpose" => {
            "This macro records something that cannot travel to another object yet."
        }
        "app.macros.lib.file_kind" => "PH2D macro",
        "app.macros.lib.saved" => "Saved \u{201c}{detail}\u{201d} to your macros.",
        "app.macros.lib.nothing_to_save" => {
            "Record something first \u{2014} this object has no macro."
        }
        "app.macros.lib.disk" => "The disk refused: {detail}",
        "app.macros.lib.name_taken" => "There is already a macro called \u{201c}{detail}\u{201d}.",
        "app.macros.lib.empty_name" => "A macro needs a name.",
        "app.macros.lib.unreadable" => "That file is not a macro this version can play.",
        "app.macros.lib.no_folder" => "There is no folder for your macros (HOME is not set).",
        "app.macros.lib.missing" => "\u{201c}{detail}\u{201d} is no longer in your macros.",
        "app.macros.lib.exported" => "Exported to {detail}",
        "app.macros.lib.imported" => "Imported as \u{201c}{detail}\u{201d}.",
        "app.macros.lib.renamed" => "Renamed to \u{201c}{detail}\u{201d}.",
        "app.macros.lib.deleted" => "Deleted \u{201c}{detail}\u{201d}.",
        "app.macros.lib.does_not_play" => {
            "This macro does not play to the end, so it was not written: {detail}"
        }
        "app.macros.apply.applied" => "Applied \u{201c}{detail}\u{201d}.",
        "app.macros.apply.over_mixed" => {
            "This macro builds an image and a Motion graph \u{2014} apply it as new."
        }
        "app.macros.apply.over_incompatible" => {
            "Select an image (for a painting macro) or a Motion object (for a Motion macro)."
        }
        "app.macros.apply.no_target" => "The object to apply over no longer exists.",
        "app.macros.apply.not_born" => "The macro could not create its object: {detail}",
        "app.macros.apply.eight_bit_copy" => {
            "the macro paints over an 8-bit copy of this 16-bit image (the Painter works in 8 bits)"
        }
        "app.macros.asset.saved" => "\u{201c}{detail}\u{201d} is now an asset of this project.",
        // A cena `PH2D_MACRO_SMOKE=7`: o que cada passo É, no ecrã.
        "app.macros.smoke7.step.1" => {
            "1 of 10 \u{2014} Add \u{25b8} Macro: an empty Macro object appears (the ring is its handle)."
        }
        "app.macros.smoke7.step.2" => {
            "2 of 10 \u{2014} Record: the Macro object records what is made inside it."
        }
        "app.macros.smoke7.step.3" => {
            "3 of 10 \u{2014} A new white image (256\u{d7}256) is born inside it; two strokes: red, then blue."
        }
        "app.macros.smoke7.step.4" => {
            "4 of 10 \u{2014} Stop: the Macro object keeps the recording (Macros panel, on the right)."
        }
        "app.macros.smoke7.step.5" => {
            "5 of 10 \u{2014} Save to library: the macro joins your list (a test folder: {detail})."
        }
        "app.macros.smoke7.step.6" => {
            "6 of 10 \u{2014} Export\u{2026}: the macro is written to a file: {detail}"
        }
        "app.macros.smoke7.step.7" => {
            "7 of 10 \u{2014} The Macro object and its image are deleted: the scene is empty again."
        }
        "app.macros.smoke7.step.8" => {
            "8 of 10 \u{2014} Import\u{2026}: the file comes back into the list as a new macro."
        }
        "app.macros.smoke7.step.9" => {
            "9 of 10 \u{2014} Apply \u{25b8} Create new: the Macro object and its image are born, and the strokes replay."
        }
        "app.macros.smoke7.step.10" => {
            "10 of 10 \u{2014} A bigger image (512\u{d7}256); Apply \u{25b8} over it, the Macro object turned 30\u{b0} and moved: the strokes paint it at their recorded size, turned."
        }
        "app.macros.smoke7.bigger_image" => "Bigger image",
        "app.macros.smoke7.done" => "The scene is over \u{2014} the app is yours.",
        _ => return None,
    })
}
