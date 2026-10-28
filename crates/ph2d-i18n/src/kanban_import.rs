//! Importar do Trello e exportar o Kanban (TrelloClone W10/J3): o menu File, os diálogos, o relatório
//! do que ficou de fora (uma frase por `LossKind`) e os avisos.

/// A tradução de uma chave do importar/exportar, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    let rest = key.strip_prefix("kanban.import.")?;
    Some(match rest {
        "cmd_import" => "Import from Trello\u{2026}",
        "cmd_export" => "Export Kanban\u{2026}",
        "pick_json" => "Import a Trello board",
        "filter_json" => "Trello board (JSON)",
        "pick_folder" => "Folder with the attachment files (Cancel: keep them as links)",
        "read_failed" => "Could not read {path}: {error}",
        "not_json" => "{path} is not a JSON file: {error}",
        "not_board" => "{path} is not a Trello board export.",
        "report_title" => "Imported from Trello",
        "report_intro" => "\u{ab}{name}\u{bb} is open in its own tab.",
        "counts_caption" => "What came in",
        "losses_caption" => "What was left out ({n})",
        "losses_none" => "Nothing was left out.",
        "loss_group" => "{what} ({n})",
        "done" => "Done",
        "count.lists" => "Lists: {n}",
        "count.cards" => "Cards: {n}",
        "count.labels" => "Labels: {n}",
        "count.checklists" => "Checklists: {n}",
        "count.items" => "Checklist items: {n}",
        "count.comments" => "Comments: {n}",
        "count.attachments" => "Attachments: {n}",
        "count.fields" => "Custom fields: {n}",
        "count.activities" => "Activity: {n}",
        "count.members" => "Members: {n}",
        "count.files" => "Files: {n}",
        "loss.power_up" => "Power-Up data (made by other apps)",
        "loss.vote_without_member" => "Votes from people who are not on the board",
        "loss.unknown_member" => "People who are not on the board",
        "loss.unknown_color" => "Colors Trello does not have (they came in without color)",
        "loss.orphan" => "Items whose list or card is missing",
        "loss.duplicate_id" => "Repeated items (the first one stayed)",
        "loss.cover_image" => "Image covers without the image file",
        "loss.background_image" => "The photo background (not in the export: the blue stays)",
        "loss.board_description" => "The board description (a Kanban has no place for it)",
        "loss.unsupported_action" => "Activity with no match in PH2D",
        "loss.action_without_author" => "Activity without an author",
        "loss.bad_value" => "Values of the wrong kind (a date that is not a date\u{2026})",
        "loss.attachment_file_missing" => {
            "Attachment files not in the folder (they came in as links)"
        }
        "loss.card_role" => "Separators, mirrors and link cards (they came in as plain cards)",
        "loss.subscription" => {
            "\u{201c}Watch\u{201d} (it belongs to whoever exported, not to the board)"
        }
        "export_title" => "Export Kanban",
        "format.csv" => "CSV (spreadsheet)",
        "format.md" => "Markdown",
        "format.json" => "Trello JSON",
        "export_done" => "Exported \u{ab}{name}\u{bb} as {format} to {path}",
        "export_failed" => "Could not write {path}: {error}",
        "export_none" => "Open a Kanban to export it.",
        _ => return None,
    })
}
