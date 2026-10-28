//! **Importar do Trello e exportar** (TrelloClone W10) — a porta `File → Import/Export` do Kanban.
//!
//! - [`import_trello`] / [`import_trello_into`]: o JSON de «Menu → Print, export and share → Export
//!   as JSON» de um quadro → um [`Kanban`] e o [`ImportReport`] do que ficou de fora. Entrada
//!   defeituosa nunca entra em pânico.
//! - [`export_trello_json`]: o inverso (importar o que ela escreve dá o MESMO Kanban).
//! - [`export_csv`] (as colunas do CSV do Trello Premium, RFC 4180) e [`export_markdown`].
//!
//! Membros: a pessoa do Kanban é o `username` do Trello — o mesmo nome das `@menções` dos
//! comentários, que entram sem tradução.

mod export_json;
mod export_text;
mod files;
mod import;
mod import_actions;
mod import_files;
mod report;
mod trello;

use ph2d_kanban_model::{Kanban, KanbanId, KanbanSet};

pub use export_json::export_trello_json;
pub use export_text::{CSV_COLUMNS, export_csv, export_markdown};
pub use files::{DirFiles, FileSource};
pub use import::{Imported, import_trello};
pub use report::{Counts, ImportError, ImportReport, Loss, LossKind};
pub use trello::{label_color_name, label_color_of};

/// Importa um quadro do Trello para um Kanban NOVO do projecto (os ficheiros vão para o conjunto)
/// e devolve a chave dele e o relatório. Em erro, o conjunto fica como estava.
///
/// # Errors
/// Os de [`import_trello`].
pub fn import_trello_into(
    set: &mut KanbanSet,
    json: &[u8],
    files: Option<&dyn FileSource>,
) -> Result<(KanbanId, ImportReport), ImportError> {
    let Imported {
        mut kanban,
        files,
        report,
    } = import_trello(json, KanbanId(0), files)?;
    kanban.id = set.create(&kanban.header.name);
    for bytes in files {
        set.put_file(bytes, None);
    }
    let id = kanban.id;
    set.insert(kanban);
    Ok((id, report))
}

/// O Kanban `id` do conjunto como Trello-JSON, CSV ou markdown (`None` = não existe).
#[must_use]
pub fn export(set: &KanbanSet, id: KanbanId, format: Format) -> Option<String> {
    let k: &Kanban = set.get(id)?;
    Some(match format {
        Format::TrelloJson => export_trello_json(k),
        Format::Csv => export_csv(k),
        Format::Markdown => export_markdown(k),
    })
}

/// Os formatos de exportação.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Format {
    TrelloJson,
    Csv,
    Markdown,
}

impl Format {
    pub const ALL: [Format; 3] = [Format::TrelloJson, Format::Csv, Format::Markdown];

    /// A extensão do ficheiro.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Format::TrelloJson => "json",
            Format::Csv => "csv",
            Format::Markdown => "md",
        }
    }
}
