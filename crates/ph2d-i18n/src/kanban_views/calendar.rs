//! As palavras da vista `calendar` do Kanban (chaves `kanban.views.calendar.*`).

/// A tradução de `calendar.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("calendar.")?;
    Some(match k {
        _ => return None,
    })
}
