//! A recuperação do projecto inteiro depois de um fecho à força (P12 S, `ph2d-recovery`): o diálogo
//! do arranque, o de fechar com alterações, e as falhas da cópia.

/// A tradução de uma chave `recovery.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "recovery.title" => "Recover project",
        "recovery.body" => {
            "PH2D did not close normally. A recovery copy of \u{201c}{name}\u{201d} was saved {ago} ({size}).\n\nRestore opens it as an unsaved project; Save writes it back to the original file. Later keeps it for next time (up to a week)."
        }
        "recovery.restore" => "Restore",
        "recovery.discard" => "Discard",
        "recovery.later" => "Later",
        "recovery.ago_secs" => "{n} seconds ago",
        "recovery.ago_mins" => "{n} minutes ago",
        "recovery.ago_hours" => "{n} hours ago",
        "recovery.ago_days" => "{n} days ago",
        "recovery.size_kb" => "{n} KB",
        "recovery.size_mb" => "{n} MB",
        "recovery.today_at" => "Today {time}",
        "recovery.yesterday_at" => "Yesterday {time}",
        "recovery.date_at" => "{date}, {time}",
        "recovery.detail" => "{when} \u{00b7} {size}",
        "recovery.quit_title" => "Save changes before closing?",
        "recovery.quit_body" => {
            "\u{201c}{name}\u{201d} has changes that are not saved. If you close without saving, they are lost."
        }
        "recovery.save" => "Save",
        "recovery.dont_save" => "Don\u{2019}t Save",
        "recovery.cancel" => "Cancel",
        // A cópia de recuperação não se gravou (disco cheio, sem permissão…); `{err}` = o erro.
        "recovery.copy_failed" => "Recovery copy could not be saved: {err}",
        // Restore falhou; a cópia não se apaga.
        "recovery.restore_failed" => "Could not restore the copy; it was kept.",
        _ => return None,
    })
}
