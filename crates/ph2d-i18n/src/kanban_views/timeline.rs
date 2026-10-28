//! As palavras da vista `timeline` do Kanban (chaves `kanban.views.timeline.*`).

/// A tradução de `timeline.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("timeline.")?;
    Some(match k {
        _ => return None,
    })
}
