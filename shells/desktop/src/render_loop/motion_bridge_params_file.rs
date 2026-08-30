//! **O lado da shell do [`ParamWidget::File`]** — quem abre o diálogo, e quem sabe que
//! extensões este build lê.
//!
//! ## Por que o filtro mora AQUI e não no nó
//!
//! O `audio.bands` não pode nomear `wav`/`flac`/`ogg`: a cerca dele é **estrutural** —
//! ele não depende de crate de áudio nenhuma, e é isso que impede a FFT de entrar no cook
//! (gate `the_fft_never_reaches_the_cook`). Uma lista de extensões copiada para dentro dele
//! seria uma **segunda resposta** à pergunta *o que é um ficheiro de som*, e envelheceria no
//! dia em que um codec fosse acrescentado — o defeito que a §5 do `CLAUDE.md` regista pelo
//! nome no importador de imagens (*«uma lista escrita à mão ao lado de um predicado é duas
//! respostas à mesma pergunta, e a que o artista vê é a que envelhece»*).
//!
//! ⇒ O nó declara uma [`FileKind`]; a shell — que possui os descodificadores — resolve-a
//! para a **constante canónica** ([`AUDIO_IMPORT_EXTS`](crate::audio::decode_any)), a mesma
//! que o importador de áudio do menu já usa.
//!
//! ## E o diálogo passa pela PORTA
//!
//! Um `rfd::FileDialog` aberto à mão congela o loop sem declarar, e a mensagem escrita a
//! seguir vive um quadro ([`crate::modal`]). Aqui ele passa por `modal::pick_file`, que
//! cronometra. Gate: `every_field3d_modal_goes_through_the_door` varre a árvore.

use crate::motion_state::MotionState;
use ph2d_node_registry::{FileKind, ParamWidget};

/// **O filtro de um [`FileKind`]** — o rótulo que o diálogo mostra e as extensões que aceita.
///
/// ⚠️ **Total por construção** (um `match` exaustivo, sem braço `_`): uma espécie nova não
/// compila até alguém dizer que ficheiros ela é. O modo de falha que isto proíbe é o caro —
/// um filtro vazio abre um diálogo que **não mostra nenhum ficheiro**, e da cadeira isso lê-se
/// como *"este programa não abre o meu ficheiro"*.
#[must_use]
pub(crate) fn file_filter(kind: FileKind) -> (&'static str, &'static [&'static str]) {
    match kind {
        FileKind::Audio => ("Audio", crate::audio::decode_any::AUDIO_IMPORT_EXTS),
        // ⚠️ **As extensões são da SHELL, e o leitor é um só** (`ph2d_table::parse`): ele deteta
        // o separador, então `.csv` e `.tsv` são o MESMO caminho de código — a lista aqui diz o
        // que o diálogo oferece, nunca o que o leitor sabe.
        FileKind::Table => ("Table", crate::render_loop::motion_table_gen::TABLE_EXTS),
    }
}

/// A [`FileKind`] que um `(nó, param)` declara — `None` se aquele param não é um ficheiro.
///
/// É a resolução que o `MotionParamIntent::PickFile` deixou de fora de propósito: o painel
/// não depende do registry, então quem publicou o `ParamUiHint` é quem o lê de volta.
#[must_use]
pub(crate) fn kind_of(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> Option<FileKind> {
    let tid = motion.doc.graph.node(nid)?.type_id();
    motion
        .registry
        .param_ui(tid)?
        .iter()
        .find(|h| h.param == param)
        .and_then(|h| match h.widget {
            ParamWidget::File { kind } => Some(kind),
            _ => None,
        })
}

/// **Abre o diálogo e devolve o caminho escolhido** (`None` = cancelado, ou o param não é um
/// ficheiro).
///
/// ⚠️ **O diálogo abre na pasta do caminho ACTUAL** quando há um. É o que torna *«este
/// ficheiro mudou de sítio»* reparável com dois cliques em vez de uma navegação inteira — e é
/// gratuito, porque o caminho velho já está no documento.
#[must_use]
pub(super) fn pick(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> Option<String> {
    let kind = kind_of(motion, nid, param)?;
    let (label, exts) = file_filter(kind);
    let mut dialog = rfd::FileDialog::new().add_filter(label, exts);
    if let Some(dir) = motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(param))
        .map(std::path::Path::new)
        .and_then(std::path::Path::parent)
        .filter(|p| p.is_dir())
    {
        dialog = dialog.set_directory(dir);
    }
    crate::modal::pick_file(dialog).map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "motion_bridge_params_file_tests.rs"]
mod tests;
