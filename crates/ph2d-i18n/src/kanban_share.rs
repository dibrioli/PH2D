//! O que o cabeçalho do Kanban diz (TrelloClone J1): o seletor de vistas e o «Share» — partilhar este
//! Kanban ao vivo, o link com chave, entrar por link.

/// A tradução de uma chave do «Share», ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    let rest = key.strip_prefix("kanban.share.")?;
    Some(match rest {
        "button" => "Share",
        "title" => "Share Kanban",
        "views_title" => "Views",
        "intro" => "Work on this Kanban with others, live.",
        "start" => "Share this Kanban",
        "join_caption" => "Join a shared Kanban",
        "link_hint" => "Paste an invite link",
        "join" => "Join",
        "link_caption" => "Invite link",
        "preparing" => "Preparing the internet link\u{2026}",
        "copy" => "Copy link",
        "copied" => "Link copied.",
        "lan_caption" => "On this network (address and code)",
        "stop" => "Stop sharing",
        "joining" => "Joining\u{2026}",
        "cancel" => "Cancel",
        "guest" => "You are in a shared Kanban.",
        "leave" => "Leave",
        "link_ready" => "Invite link copied. Send it to whoever should join.",
        "link_bad" => "That is not an invite link.",
        "failed" => "Could not share the Kanban: {error}",
        "internet_failed" => "Sharing works on this network only: {error}",
        "join_failed" => "Could not join: {error}",
        "refused" => "That Kanban is shared by another version of PH2D.",
        "ended" => "The host stopped sharing the Kanban.",
        "lost" => "The connection to the shared Kanban was lost.",
        "reconnecting" => "Connection lost \u{2014} reconnecting\u{2026}",
        "reconnected" => "Reconnected to the shared Kanban.",
        "reconnecting_badge" => "Reconnecting\u{2026}",
        "board_link" => "That is a Board invite \u{2014} joining the Board.",
        "kanban_link" => "That is a Kanban invite \u{2014} opening it as a Kanban.",
        _ => return None,
    })
}
