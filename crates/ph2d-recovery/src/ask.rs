//! A janela do arranque, uma cópia de cada vez: nome, quando, tamanho, e `Restore` · `Discard` ·
//! `Later`. Hoje é a janela NATIVA do sistema (o `rfd` que a shell já usa no Save/Open): é modal e
//! dona do teclado por construção (o `Esc` = `Later`, nada se perde por fechá-la). A versão em
//! cartões do chrome fica aberta (ver o handoff da P12).

use ph2d_i18n::{tr, tr_with};

use crate::{Choice, Orphan};

/// «há quanto tempo», em inglês.
#[must_use]
pub fn ago(secs: u64) -> String {
    let (key, n) = match secs {
        0..60 => ("recovery.ago_secs", secs),
        60..3600 => ("recovery.ago_mins", secs / 60),
        3600..86_400 => ("recovery.ago_hours", secs / 3600),
        _ => ("recovery.ago_days", secs / 86_400),
    };
    tr_with(key, &[("n", &n)])
}

/// O tamanho legível.
#[must_use]
pub fn size(bytes: u64) -> String {
    if bytes < 1 << 20 {
        tr_with("recovery.size_kb", &[("n", &bytes.div_ceil(1024))])
    } else {
        tr_with("recovery.size_mb", &[("n", &(bytes >> 20))])
    }
}

/// O texto da janela para uma cópia.
#[must_use]
pub fn body(o: &Orphan, now: u64) -> String {
    let ago = ago(now.saturating_sub(o.meta.saved));
    let size = size(o.size);
    tr_with(
        "recovery.body",
        &[("name", &o.meta.name), ("ago", &ago), ("size", &size)],
    )
}

/// Pergunta na janela nativa. Fechar a janela = `Later`.
#[must_use]
pub fn ask_native(o: &Orphan) -> Choice {
    let (r, d, l) = (
        tr("recovery.restore"),
        tr("recovery.discard"),
        tr("recovery.later"),
    );
    let got = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title(tr("recovery.title"))
        .set_description(body(o, crate::now_secs()))
        .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
            r.to_owned(),
            d.to_owned(),
            l.to_owned(),
        ))
        .show();
    match got {
        rfd::MessageDialogResult::Custom(s) if s == r => Choice::Restore,
        rfd::MessageDialogResult::Yes => Choice::Restore,
        rfd::MessageDialogResult::Custom(s) if s == d => Choice::Discard,
        rfd::MessageDialogResult::No => Choice::Discard,
        _ => Choice::Later,
    }
}
