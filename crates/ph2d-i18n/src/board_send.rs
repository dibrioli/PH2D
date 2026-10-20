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
        // W16 — vários objectos de uma vez: os que não foram.
        "board.send.left_out" => "Left out: {names}",
        // W17 — o nome nos avisos de um objecto sem nome (pelo tipo), de um apagado entre o pedido e a
        // escolha, e o aviso de quando TODOS sumiram.
        "board.send.unnamed.image" => "Unnamed picture",
        "board.send.unnamed.vector" => "Unnamed shape",
        "board.send.unnamed.flip" => "Unnamed drawing",
        "board.send.unnamed.other" => "Unnamed object",
        "board.send.deleted" => "Deleted object",
        "board.send.repeated" => "{name} \u{d7}{count}",
        "board.send.all_gone" => "Those objects are no longer in the scene",
        // W23 B15 — a selecção de um quadro para outro.
        "board.send.copy_title" => "Copy to board",
        "board.send.move_title" => "Move to board",
        "board.send.copied_one" => "Copied 1 object to \u{201c}{name}\u{201d}",
        "board.send.copied" => "Copied {count} objects to \u{201c}{name}\u{201d}",
        "board.send.moved_one" => "Moved 1 object to \u{201c}{name}\u{201d}",
        "board.send.moved" => "Moved {count} objects to \u{201c}{name}\u{201d}",
        "board.send.undo" => "Undo",
        "board.send.undo_tip" => "Put them back on the first board and take them off this one",
        _ => return None,
    })
}
