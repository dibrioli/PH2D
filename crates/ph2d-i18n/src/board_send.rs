//! **«Send to board…»** (MiroClone W15): o modal que escolhe o quadro e os avisos de quem manda uma
//! imagem, um vector ou um Flip da Hierarquia para um quadro.

/// A tradução de uma chave do «Send to board», ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.send.title" => "Send to board",
        "board.send.search" => "Search boards",
        "board.send.new_board" => "New board",
        "board.send.more" => "{count} more \u{2014} type to narrow",
        "board.send.wrong_kind" => {
            "Only pictures, vector shapes and Flip drawings can go to a board"
        }
        "board.send.gone" => "That object is no longer in the scene",
        "board.send.empty" => "{name} has nothing to show on this frame",
        "board.send.unreadable" => "Could not read the pixels of {name}",
        "board.send.refused" => "The board could not take {name}",
        "board.send.no_board" => "That board was closed before the picture arrived",
        _ => return None,
    })
}
