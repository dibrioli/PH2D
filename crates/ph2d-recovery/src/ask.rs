//! Os textos de uma cópia: há quanto tempo, o tamanho e o corpo por extenso (a janela em cartões
//! é a [`crate::card`]).

use ph2d_i18n::tr_with;

use crate::Orphan;

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
