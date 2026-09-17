//! **O QUE A FAMÍLIA app.sculpt3d DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-sculpt3d` mostra (da ESCULTURA (as recusas da retopologia, importar/exportar malhas)), na forma `app.sculpt3d.<ficheiro>.<frase>`.
//!
//! ⚠️ **Frases com peças do código usam [`crate::tr_with`] com marcadores NOMEADOS** — uma língua
//! pode reordenar os marcadores; não pode mudar o que eles valem.
//!
//! ⛔ **O que NÃO está aqui, de propósito** (cada um com excepção NOMEADA no gate da crate): o
//! diagnóstico de CONSOLA, as cenas de smoke e os **nomes por omissão de objecto** — um nome que
//! entra no `Name` é identidade durável (`stable_name_id` fecha um hash sobre ele), e traduzi-lo
//! é decisão do dono, não desta migração.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave `app.sculpt3d.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.sculpt3d.export.export_failed" => "Export failed: {e}",
        "app.sculpt3d.export.exported_piece_s_kb" => {
            "Exported {n} piece(s), {size} KB -- {name} ({fmt})"
        }
        "app.sculpt3d.export.unknown_extension_use" => "Unknown extension: use {join}",
        "app.sculpt3d.export.nothing_to_export_no_sculpture_open" => {
            "Nothing to export: no sculpture open"
        }
        "app.sculpt3d.import.mesh" => "Mesh",
        "app.sculpt3d.import.imported_mesh_piece_s" => "Imported {n} mesh piece(s)",
        "app.sculpt3d.import.mesh_refused" => "Mesh refused: {name} ({e})",
        "app.sculpt3d.import.unknown_extension" => "unknown extension",
        "app.sculpt3d.remesh_refusal.shattered" => {
            "The retopology broke the piece into {pieces} loose pieces (it came in as {was}), and the \
         sculpture stays as it is \u{2014} undo (Ctrl+Z) back to the original sculpture before \
         running it again, or lower the Detail"
        }
        "app.sculpt3d.remesh_refusal.extraction_refused" => {
            "The integer-grid extraction refused, and the sculpture stays as it is: {e} \u{2014} set \
         PH2D_RETOPO_EXTRACT=0 to go back to the usual path"
        }
        "app.sculpt3d.remesh_refusal.assembly_refused" => {
            "The assembly refused (a defect upstream), and the sculpture stays as it is: {e_}"
        }
        "app.sculpt3d.remesh_refusal.quantization_failed" => {
            "The quantization did not close, and the sculpture stays as it is: {e_} \u{2014} try another \
         Detail, or PH2D_RETOPO_LEGACY=1"
        }
        "app.sculpt3d.remesh_refusal.tracing_failed" => {
            "The tracing did not close a layout, and the sculpture stays as it is: {e_} \u{2014} try \
         another Detail, or PH2D_RETOPO_LEGACY=1"
        }
        "app.sculpt3d.remesh_refusal.handle_not_traced" => {
            "This piece has a hole (or a handle) the tracing could not go around, and the sculpture stays \
         as it is: the decomposition closed as {complex} and the piece is {surface}. Changing the \
         Detail will not fix it \u{2014} use PH2D_RETOPO_LEGACY=1, or run the Remesh on the piece \
         first"
        }
        "app.sculpt3d.remesh_refusal.too_coarse_for_quads" => {
            "The mesh is too coarse for a quad grid, and the sculpture stays as it is: subdivide it (or run \
         the Remesh) first"
        }
        "app.sculpt3d.remesh_refusal.mesh_not_closed" => {
            "The retopology did not close a mesh, and the sculpture stays as it is: {e}"
        }
        "app.sculpt3d.remesh_refusal.rebuild_refused" => {
            "It will not rebuild, and the sculpture stays as it is: {e} \u{2014} try another resolution"
        }
        "app.sculpt3d.remesh_refusal.our_defect" => {
            "The retopology failed on a defect of OURS and the sculpture stays as it is: try another Detail \
         \u{2014} and if you can, save the piece and tell us"
        }
        "app.sculpt3d.remesh_refusal.no_piece_in_the_scene" => {
            "It will not rebuild: there is no piece in the scene"
        }
        "app.sculpt3d.remesh_refusal.multires_stack" => {
            "It will not rebuild with the multires stack up: the remesh swaps the TOPOLOGY, and every level \
         above it is a subdivision of that one \u{2014} FLATTEN the stack first"
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
