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
        "app.sculpt3d.export.exported_piece_s_kb" => "Exported {n} piece(s), {size} KB -- {name}",
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
        // ⬇️ **Escritas à MÃO, fora dos marcadores** (o cabeçalho manda): a caixa de saída da
        //    escultura — a queixa do passe de topologia, as três recusas do pen-down e as duas do
        //    corte. ⚠️ Cada uma é *a frase que o artista lê*, e não diagnóstico de consola: quem
        //    só vê o terminal é o `sonda_undo.rs`, isento com o mecanismo no gate da crate.
        //
        // ⛔⛔⛔ **AS OITO NASCERAM EM PORTUGUÊS, E ISSO PASSOU POR TODA A MIGRAÇÃO DO HR-15.**
        //    Elas estão FORA dos marcadores do script, logo nenhuma das trinta catracas do texto as
        //    tocou — e a régua que as apanhou foi outra: *a tabela INGLESA do app tem de falar
        //    inglês*, medida em 2026-09-19 sobre as `5 226` entradas. ⚠️ **Elas são exactamente a
        //    metade que o comentário acima declara artista-facing** — a metade de terminal daquele
        //    mesmo ficheiro está certa em português, e é essa diferença que torna estas um defeito.
        //    Traduzidas por ORDEM DO DONO (*«tudo em inglês»*, 2026-09-19).
        "app.sculpt3d.dyntopo.pilha_montada_j_reverte" => {
            "a multiresolution stack is mounted -- J reverts it"
        }
        "app.sculpt3d.dyntopo.ja_no_ponto_que_o_detail_pede" => {
            "the mesh here is already at the density Detail asks for -- move the slider (or \
             press U) to ask for another one, or grow the brush with ] to reach more of the piece"
        }
        "app.sculpt3d.recusa.precisa_de_uma_pilha" => {
            "{nome} needs a multiresolution stack -- with no level BELOW there is no \
             displacement at all (K subdivides, ',' steps down)"
        }
        "app.sculpt3d.recusa.trabalha_a_beira_de_uma_peca_aberta" => {
            "{nome} works the RIM of an open piece -- this piece is closed, and its region \
             starts at the border (try a bowl, or delete faces to open a mouth)"
        }
        "app.sculpt3d.recusa.precisa_de_outra_peca_a_vista" => {
            "{nome} needs ANOTHER piece IN SIGHT -- the one{plural} there is are hidden (open \
             its eye in the Hierarchy, or leave isolation)"
        }
        "app.sculpt3d.recusa.precisa_de_outra_peca_na_cena" => {
            "{nome} needs ANOTHER piece in the scene -- it pushes the clay until it meets \
             that piece, and here there is only one"
        }
        "app.sculpt3d.recusa.a_tinta_fina_perde_detalhe_com_topologia" => {
            "{nome} changes the topology, and Paint Detail is on for this piece -- the fine \
             paint is re-seeded from the per-vertex colour, so detail finer than the mesh is \
             lost (turn Dynamic Topology off to keep it)"
        }
        "app.sculpt3d.recusa.a_tinta_fina_dispensa_a_topologia" => {
            "Dynamic Topology is on, and {nome} will NOT densify the mesh -- Paint Detail is \
             armed for this piece, so the paint has its own resolution and refining would only \
             throw the fine plane away (a shape brush still densifies)"
        }
        "app.sculpt3d.trim_aplica.sem_peca_para_cortar" => "there is no piece to cut",
        "app.sculpt3d.trim_aplica.pilha_montada_j_reverte" => {
            "a multiresolution stack is mounted -- J reverts it and the cut comes back"
        }
        _ => return None,
    })
}
