//! ⭐ **O teclado de um quadro** (MiroClone, W1) — com uma aba de quadro activa e nenhum campo do
//! chrome com o teclado, as teclas são do quadro ANTES de qualquer atalho da cena (senão `Delete`
//! apagaria o objecto seleccionado na cena, que nem está à vista).
//!
//! A shell traduz a tecla física para [`BoardKey`] e chama [`key`]; o que este devolve como não
//! consumido segue o caminho de sempre (`Ctrl+S`, as teclas de função…).

use super::HeroScreen;
pub use ph2d_board_edit::Command;
use ph2d_board_edit::{Move, TextKey, Tool};
use ph2d_board_model::ShapeType;
use ph2d_host::Modifiers;
use ph2d_text::TextSystem;

/// Uma tecla, já sem o desenho físico do teclado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardKey {
    /// Uma letra ou algarismo (minúsculo), para os atalhos.
    Char(char),
    Enter,
    Escape,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Space,
    Other,
}

/// O que a tecla fez.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyOutcome {
    /// O quadro tomou a tecla (a shell não a passa a mais ninguém).
    pub consumed: bool,
    /// Texto que a shell deve pôr na área de transferência do sistema (copiar/cortar texto).
    pub copy: Option<String>,
}

fn taken() -> KeyOutcome {
    KeyOutcome {
        consumed: true,
        copy: None,
    }
}

/// Uma tecla premida (`pressed`) ou solta, com o texto que ela escreve (`text`, já resolvido pelo
/// teclado do sistema) e, para `Ctrl+V`, o texto da área de transferência (`paste`).
pub fn key(
    hero: &mut HeroScreen,
    ts: &mut TextSystem,
    k: BoardKey,
    mods: Modifiers,
    pressed: bool,
    text: Option<&str>,
    paste: Option<String>,
) -> KeyOutcome {
    if hero.documents.active().is_none() {
        return KeyOutcome::default();
    }
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return KeyOutcome::default();
    };
    let editing = live.editor.as_ref().is_some_and(|e| e.is_editing_text());
    if k == BoardKey::Space && !editing {
        live.space = pressed;
        return taken();
    }
    if !pressed {
        return KeyOutcome::default();
    }
    let ctrl = mods.ctrl || mods.meta;
    let history = live.histories.entry(board.id).or_default();
    let ed = super::board_view::editor(&mut live.editor, theme);
    let doc = &mut board.doc;
    if editing {
        let word = ctrl || mods.alt;
        let tk = match k {
            BoardKey::Escape => Some(TextKey::Commit),
            BoardKey::Enter if ctrl => Some(TextKey::Commit),
            BoardKey::Enter => Some(TextKey::Newline),
            BoardKey::Backspace => Some(TextKey::Backspace { word }),
            BoardKey::Delete => Some(TextKey::Delete { word }),
            BoardKey::Left
            | BoardKey::Right
            | BoardKey::Up
            | BoardKey::Down
            | BoardKey::Home
            | BoardKey::End => Some(TextKey::Move {
                to: text_move(k, ctrl),
                extend: mods.shift,
            }),
            BoardKey::Char('a') if ctrl => Some(TextKey::SelectAll),
            _ => None,
        };
        if let Some(tk) = tk {
            ed.text_key(doc, history, ts, tk);
            return taken();
        }
        if ctrl {
            return match k {
                BoardKey::Char('c') => KeyOutcome {
                    consumed: true,
                    copy: ed.selected_text(),
                },
                BoardKey::Char('x') => {
                    let copy = ed.selected_text();
                    ed.text_key(doc, history, ts, TextKey::Backspace { word: false });
                    KeyOutcome {
                        consumed: true,
                        copy,
                    }
                }
                BoardKey::Char('v') => {
                    if let Some(p) = paste {
                        ed.text_input(doc, ts, &p);
                    }
                    taken()
                }
                BoardKey::Char('z' | 'y') => {
                    ed.text_key(doc, history, ts, TextKey::Commit);
                    ed.command(doc, history, ts, chord_undo(k, mods));
                    taken()
                }
                _ => KeyOutcome::default(),
            };
        }
        if let Some(t) = text.filter(|t| !t.chars().any(char::is_control)) {
            ed.text_input(doc, ts, t);
            return taken();
        }
        return taken();
    }
    let nudge = ph2d_board_edit::NUDGE[usize::from(mods.shift)];
    let command = match (k, ctrl) {
        (BoardKey::Char('z' | 'y'), true) => Some(chord_undo(k, mods)),
        (BoardKey::Char('d'), true) => Some(Command::Duplicate),
        (BoardKey::Char('a'), true) => Some(Command::SelectAll),
        (BoardKey::Char('c'), true) => Some(Command::Copy),
        (BoardKey::Char('x'), true) => Some(Command::Cut),
        (BoardKey::Char('v'), true) => Some(Command::Paste),
        (BoardKey::Delete | BoardKey::Backspace, false) => Some(Command::Delete),
        (BoardKey::Escape, false) => Some(Command::Escape),
        (BoardKey::Enter, false) => Some(Command::EditText),
        (BoardKey::Left, false) => Some(Command::Nudge([-nudge, 0.0])),
        (BoardKey::Right, false) => Some(Command::Nudge([nudge, 0.0])),
        (BoardKey::Up, false) => Some(Command::Nudge([0.0, -nudge])),
        (BoardKey::Down, false) => Some(Command::Nudge([0.0, nudge])),
        (BoardKey::Char(c), false) if !mods.alt => tool_key(c).map(Command::Tool),
        _ => None,
    };
    if let Some(c) = command {
        // ⚠️ Um atalho de uma tecla que o quadro reconhece é SEMPRE dele — mesmo sem efeito
        // (`Esc` sem nada seleccionado): deixá-lo cair faria a cena reagir por baixo.
        let _ = ed.command(doc, history, ts, c);
        return taken();
    }
    // ⚠️ Uma letra solta é ATALHO, nunca texto (o idioma do Excalidraw): para escrever numa forma,
    // `Enter` ou duplo-clique. Com «escrever começa a edição», desenhar uma forma e carregar `R`
    // para a seguinte escreveria «r» dentro dela. Uma letra que o quadro não usa também não é da
    // cena (ela está por baixo, escondida).
    KeyOutcome {
        consumed: matches!(k, BoardKey::Char(_)) && !ctrl && !mods.alt,
        copy: None,
    }
}

fn chord_undo(k: BoardKey, mods: Modifiers) -> Command {
    if k == BoardKey::Char('y') || mods.shift {
        Command::Redo
    } else {
        Command::Undo
    }
}

/// Um comando do quadro activo (o menu Editar e as teclas de desfazer chegam aqui).
pub fn board_command(hero: &mut HeroScreen, ts: &mut TextSystem, c: Command) -> bool {
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let history = live.histories.entry(board.id).or_default();
    let ed = super::board_view::editor(&mut live.editor, theme);
    ed.command(&mut board.doc, history, ts, c)
}

fn text_move(k: BoardKey, ctrl: bool) -> Move {
    match (k, ctrl) {
        (BoardKey::Left, false) => Move::Left,
        (BoardKey::Left, true) => Move::WordLeft,
        (BoardKey::Right, false) => Move::Right,
        (BoardKey::Right, true) => Move::WordRight,
        (BoardKey::Up, _) => Move::Up,
        (BoardKey::Down, _) => Move::Down,
        (BoardKey::Home, false) => Move::LineStart,
        (BoardKey::Home, true) => Move::TextStart,
        (BoardKey::End, false) => Move::LineEnd,
        _ => Move::TextEnd,
    }
}

/// Os atalhos de uma tecla (os que o Miro e o Excalidraw partilham, plano 01 §4.6): `V`/`1`
/// seleccionar · `H` mão · `R`/`2` rectângulo · `D`/`3` losango · `O`/`4` elipse.
#[must_use]
pub fn tool_key(c: char) -> Option<Tool> {
    Some(match c {
        'v' | '1' => Tool::Select,
        'h' => Tool::Hand,
        'r' | '2' => Tool::Shape(ShapeType::Rectangle),
        'd' | '3' => Tool::Shape(ShapeType::Diamond),
        'o' | '4' => Tool::Shape(ShapeType::Ellipse),
        _ => return None,
    })
}
