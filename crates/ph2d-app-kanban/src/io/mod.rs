//! W10 — «Import from Trello…» e «Export Kanban…» (menu File). Os diálogos passam por [`Dialogs`]:
//! o nativo (`rfd` pela porta `ph2d_app_host::modal`) e as respostas que um teste dá
//! ([`KanbanDocs::answer_next_dialogs`]).

mod report;

use crate::docs::{HOME, KanbanDocs};
use ph2d_a11y::NodeId;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_import::{DirFiles, FileSource, Format, ImportError, LossKind};
use ph2d_kanban_model::KanbanId;
use ph2d_tool_registry::hash_node_id;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

pub(crate) use report::{key, paint, pointer, wheel};

/// Os três diálogos do importar/exportar.
pub(crate) trait Dialogs {
    fn pick_json(&mut self) -> Option<PathBuf>;
    /// A pasta dos ficheiros dos anexos (`None` = cancelar: entram como links).
    fn pick_folder(&mut self, near: &Path) -> Option<PathBuf>;
    fn save_export(&mut self, file_name: &str) -> Option<PathBuf>;
}

struct Native;

impl Dialogs for Native {
    fn pick_json(&mut self) -> Option<PathBuf> {
        let d = rfd::FileDialog::new()
            .set_title(tr("kanban.import.pick_json"))
            .add_filter(tr("kanban.import.filter_json"), &["json"]);
        ph2d_app_host::modal::pick_file(d)
    }

    fn pick_folder(&mut self, near: &Path) -> Option<PathBuf> {
        let mut d = rfd::FileDialog::new().set_title(tr("kanban.import.pick_folder"));
        if let Some(dir) = near.parent() {
            d = d.set_directory(dir);
        }
        // A porta não tem `pick_folder`: o congelamento declara-se à mão, como ela o faz.
        let t0 = std::time::Instant::now();
        let out = d.pick_folder();
        ph2d_app_host::modal::note_stall(t0.elapsed());
        out
    }

    fn save_export(&mut self, file_name: &str) -> Option<PathBuf> {
        let mut d = rfd::FileDialog::new()
            .set_title(tr("kanban.import.export_title"))
            .set_file_name(file_name);
        for f in EXPORT_ORDER {
            d = d.add_filter(format_name(f), &[f.extension()]);
        }
        ph2d_app_host::modal::save_file(d)
    }
}

/// As respostas dadas aos próximos diálogos, por ordem.
struct Answers;

thread_local! {
    static ANSWERS: RefCell<Option<VecDeque<Option<PathBuf>>>> = const { RefCell::new(None) };
}

fn answered() -> Option<Option<PathBuf>> {
    ANSWERS.with(|a| a.borrow_mut().as_mut().and_then(VecDeque::pop_front))
}

impl Dialogs for Answers {
    fn pick_json(&mut self) -> Option<PathBuf> {
        answered().flatten()
    }
    fn pick_folder(&mut self, _: &Path) -> Option<PathBuf> {
        answered().flatten()
    }
    fn save_export(&mut self, _: &str) -> Option<PathBuf> {
        answered().flatten()
    }
}

fn dialogs() -> Box<dyn Dialogs> {
    if ANSWERS.with(|a| a.borrow().is_some()) {
        Box::new(Answers)
    } else {
        Box::new(Native)
    }
}

/// A ordem dos formatos no diálogo de gravar (o 1.º é o do nome proposto): a folha de cálculo
/// primeiro, como o «Export as CSV» do Trello.
const EXPORT_ORDER: [Format; 3] = [Format::Csv, Format::Markdown, Format::TrelloJson];

fn format_name(f: Format) -> &'static str {
    tr(match f {
        Format::Csv => "kanban.import.format.csv",
        Format::Markdown => "kanban.import.format.md",
        Format::TrelloJson => "kanban.import.format.json",
    })
}

/// O formato pela extensão escolhida; sem extensão conhecida, o 1.º (e a extensão junta-se).
fn format_of(path: &Path) -> (Format, PathBuf) {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("csv") => (Format::Csv, path.to_owned()),
        Some("md" | "markdown") => (Format::Markdown, path.to_owned()),
        Some("json") => (Format::TrelloJson, path.to_owned()),
        _ => {
            let f = EXPORT_ORDER[0];
            let mut p = path.as_os_str().to_owned();
            p.push(".");
            p.push(f.extension());
            (f, PathBuf::from(p))
        }
    }
}

/// O nome do Kanban como nome de ficheiro (sem separadores nem caracteres que os sistemas recusam).
fn file_stem(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if "/\\:*?\"<>|".contains(c) || c.is_control() { '-' } else { c })
        .collect();
    let s = s.trim().trim_matches('.').to_owned();
    if s.is_empty() {
        tr("kanban.import.export_title").to_owned()
    } else {
        s
    }
}

/// ⭐ A chave da frase de cada perda. O `match` é exaustivo: uma variante nova não compila sem frase.
pub(crate) fn loss_key(k: LossKind) -> &'static str {
    match k {
        LossKind::PowerUp => "kanban.import.loss.power_up",
        LossKind::VoteWithoutMember => "kanban.import.loss.vote_without_member",
        LossKind::UnknownMember => "kanban.import.loss.unknown_member",
        LossKind::UnknownColor => "kanban.import.loss.unknown_color",
        LossKind::Orphan => "kanban.import.loss.orphan",
        LossKind::DuplicateId => "kanban.import.loss.duplicate_id",
        LossKind::CoverImage => "kanban.import.loss.cover_image",
        LossKind::BackgroundImage => "kanban.import.loss.background_image",
        LossKind::BoardDescription => "kanban.import.loss.board_description",
        LossKind::UnsupportedAction => "kanban.import.loss.unsupported_action",
        LossKind::ActionWithoutAuthor => "kanban.import.loss.action_without_author",
        LossKind::BadValue => "kanban.import.loss.bad_value",
        LossKind::AttachmentFileMissing => "kanban.import.loss.attachment_file_missing",
        LossKind::CardRole => "kanban.import.loss.card_role",
        LossKind::Subscription => "kanban.import.loss.subscription",
    }
}

impl KanbanDocs {
    /// «Import from Trello…» (menu File).
    pub const CMD_IMPORT: NodeId = hash_node_id("kanban_cmd_import");
    /// «Export Kanban…» (menu File).
    pub const CMD_EXPORT: NodeId = hash_node_id("kanban_cmd_export");

    /// ⚠️ SONDA/TESTE: os próximos diálogos do importar/exportar respondem isto, por ordem, sem
    /// abrir janela (`None` = o artista cancelou). Vazia a fila, os diálogos cancelam.
    pub fn answer_next_dialogs(answers: Vec<Option<PathBuf>>) {
        ANSWERS.with(|a| *a.borrow_mut() = Some(answers.into()));
    }

    /// «Import from Trello…»: o `.json`, a pasta dos anexos (só se o quadro os tem), o Kanban novo
    /// numa aba e o relatório por cima dela.
    pub(crate) fn import_from_trello(&mut self) {
        let mut d = dialogs();
        let Some(path) = d.pick_json() else {
            return;
        };
        let shown = path.display().to_string();
        let json = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.notices.push(tr_with(
                    "kanban.import.read_failed",
                    &[("path", &shown), ("error", &e.to_string())],
                ));
                return;
            }
        };
        // Pergunta-se pela pasta só a quem tem anexos (o quadro sem anexos importa num gesto).
        let has_attachments = ph2d_kanban_import::import_trello(&json, KanbanId(0), None)
            .is_ok_and(|i| i.report.counts.attachments > 0);
        let folder = has_attachments
            .then(|| d.pick_folder(&path))
            .flatten()
            .map(DirFiles);
        let files = folder.as_ref().map(|f| f as &dyn FileSource);
        let out = ph2d_kanban_import::import_trello_into(&mut self.set.borrow_mut(), &json, files);
        match out {
            Ok((id, rep)) => {
                self.open(id);
                let name = self
                    .set
                    .borrow()
                    .get(id)
                    .map(|k| k.header.name.clone())
                    .unwrap_or_default();
                report::show(id.0, name, rep);
            }
            Err(ImportError::NotJson(e)) => self.notices.push(tr_with(
                "kanban.import.not_json",
                &[("path", &shown), ("error", &e)],
            )),
            Err(ImportError::NotABoard) => self
                .notices
                .push(tr_with("kanban.import.not_board", &[("path", &shown)])),
        }
    }

    /// «Export Kanban…»: o Kanban da aba activa, no formato da extensão escolhida.
    pub(crate) fn export_kanban(&mut self) {
        let id = self
            .active
            .filter(|a| *a != HOME && self.tabs.iter().any(|t| t.id() == *a))
            .map(KanbanId);
        let Some((id, name)) =
            id.and_then(|id| Some((id, self.set.borrow().get(id)?.header.name.clone())))
        else {
            self.notices.push(tr("kanban.import.export_none").to_owned());
            return;
        };
        let proposed = format!("{}.{}", file_stem(&name), EXPORT_ORDER[0].extension());
        let Some(chosen) = dialogs().save_export(&proposed) else {
            return;
        };
        let (format, path) = format_of(&chosen);
        let Some(text) = ph2d_kanban_import::export(&self.set.borrow(), id, format) else {
            return;
        };
        let shown = path.display().to_string();
        self.notices.push(match std::fs::write(&path, text) {
            Ok(()) => tr_with(
                "kanban.import.export_done",
                &[("name", &name), ("format", &format_name(format)), ("path", &shown)],
            ),
            Err(e) => tr_with(
                "kanban.import.export_failed",
                &[("path", &shown), ("error", &e.to_string())],
            ),
        });
    }

    /// ⚠️ SONDA: os textos do relatório de importação aberto sobre a aba `id` (`None` = fechado).
    #[must_use]
    pub fn probe_import_report(&self, id: u64) -> Option<Vec<String>> {
        report::texts(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [LossKind; 15] = [
        LossKind::PowerUp,
        LossKind::VoteWithoutMember,
        LossKind::UnknownMember,
        LossKind::UnknownColor,
        LossKind::Orphan,
        LossKind::DuplicateId,
        LossKind::CoverImage,
        LossKind::BackgroundImage,
        LossKind::BoardDescription,
        LossKind::UnsupportedAction,
        LossKind::ActionWithoutAuthor,
        LossKind::BadValue,
        LossKind::AttachmentFileMissing,
        LossKind::CardRole,
        LossKind::Subscription,
    ];

    #[test]
    fn j3_import_every_loss_kind_has_its_own_text() {
        let mut keys: Vec<&str> = ALL.iter().map(|k| loss_key(*k)).collect();
        for k in &keys {
            let t = tr(k);
            assert!(!t.is_empty() && t != *k, "sem texto: {k}");
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), ALL.len(), "duas perdas com a mesma frase");
    }

    #[test]
    fn j3_import_the_extension_decides_the_format() {
        let f = |p: &str| format_of(Path::new(p));
        assert_eq!(f("/x/a.csv").0, Format::Csv);
        assert_eq!(f("/x/a.MD").0, Format::Markdown);
        assert_eq!(f("/x/a.json").0, Format::TrelloJson);
        assert_eq!(f("/x/a"), (Format::Csv, PathBuf::from("/x/a.csv")));
        assert_eq!(file_stem("A/B: c?"), "A-B- c-");
    }
}
