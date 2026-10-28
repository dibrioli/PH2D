//! As palavras da vista `table` do Kanban (chaves `kanban.views.table.*`).

/// A tradução de `table.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("table.")?;
    Some(match k {
        _ => return None,
    })
}
