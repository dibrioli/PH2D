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
        "app.macros.asset.saved" => "\u{201c}{detail}\u{201d} is now an asset of this project.",
        _ => return None,
    })
}
