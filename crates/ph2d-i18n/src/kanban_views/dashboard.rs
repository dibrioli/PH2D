//! As palavras da vista `dashboard` do Kanban (chaves `kanban.views.dashboard.*`).

/// A tradução de `dashboard.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("dashboard.")?;
    Some(match k {
        _ => return None,
    })
}
