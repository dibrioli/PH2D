//! ⭐ **O texto escrito NO LUGAR** — o título de um cartão, de uma lista, do Kanban, e os dois
//! compositores («Add a card», «Add another list»). O idioma do Trello: abrir seleciona tudo, `Enter`
//! confirma, `Esc` desiste, clicar fora confirma. No compositor, `Enter` cria e o campo fica aberto
//! para o seguinte.

use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_kanban_model::{CardId, ListId};

/// O que o campo escreve quando confirma.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    KanbanName,
    ListTitle(ListId),
    CardTitle(CardId),
    /// Um cartão novo na lista, depois de `after` (`None` = no fim).
    NewCard { list: ListId, after: Option<CardId> },
    NewList,
}

impl Target {
    /// O campo cria (fica aberto depois do `Enter`).
    pub fn composes(self) -> bool {
        matches!(self, Self::NewCard { .. } | Self::NewList)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Field {
    pub target: Target,
    pub text: String,
    /// O cursor e a âncora da selecção, em bytes (iguais = sem selecção).
    pub caret: usize,
    pub anchor: usize,
}

/// O que uma tecla fez ao campo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Tomada (o texto ou o cursor mudou, ou nada a fazer).
    #[default]
    Typed,
    Commit,
    Cancel,
    /// Copiar ou cortar: o texto vai para a área de transferência.
    Copy(String),
    /// Não é do campo.
    Ignored,
}

impl Field {
    /// Abre com `text` todo seleccionado.
    pub fn open(target: Target, text: &str) -> Self {
        Self {
            target,
            text: text.to_owned(),
            caret: text.len(),
            anchor: 0,
        }
    }

    fn range(&self) -> (usize, usize) {
        (self.caret.min(self.anchor), self.caret.max(self.anchor))
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        let (a, b) = self.range();
        (a != b).then_some((a, b))
    }

    /// Escreve `s` no lugar da selecção. As quebras de linha viram espaços (um título é uma linha).
    pub fn insert(&mut self, s: &str) {
        let clean: String = s
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .filter(|c| !c.is_control())
            .collect();
        let (a, b) = self.range();
        self.text.replace_range(a..b, &clean);
        self.caret = a + clean.len();
        self.anchor = self.caret;
    }

    fn prev(&self, i: usize) -> usize {
        self.text[..i].char_indices().next_back().map_or(0, |(j, _)| j)
    }

    fn next(&self, i: usize) -> usize {
        self.text[i..]
            .chars()
            .next()
            .map_or(self.text.len(), |c| i + c.len_utf8())
    }

    fn go(&mut self, to: usize, extend: bool) {
        self.caret = to;
        if !extend {
            self.anchor = to;
        }
    }

    /// Uma tecla premida. `text` = o que ela escreve; `paste` = a área de transferência (Ctrl+V).
    pub fn key(&mut self, key: Key, mods: Modifiers, text: Option<&str>, paste: Option<&str>) -> Outcome {
        let chord = mods.ctrl || mods.meta;
        let sel = self.selection();
        match key {
            Key::Enter => return Outcome::Commit,
            Key::Escape => return Outcome::Cancel,
            Key::Left => {
                let to = match sel {
                    Some((a, _)) if !mods.shift => a,
                    _ => self.prev(self.caret),
                };
                self.go(to, mods.shift);
            }
            Key::Right => {
                let to = match sel {
                    Some((_, b)) if !mods.shift => b,
                    _ => self.next(self.caret),
                };
                self.go(to, mods.shift);
            }
            Key::Home | Key::Up => self.go(0, mods.shift),
            Key::End | Key::Down => self.go(self.text.len(), mods.shift),
            Key::Backspace => {
                if sel.is_none() {
                    self.anchor = self.prev(self.caret);
                }
                self.insert("");
            }
            Key::Delete => {
                if sel.is_none() {
                    self.anchor = self.next(self.caret);
                }
                self.insert("");
            }
            Key::Char('a') if chord => {
                self.anchor = 0;
                self.caret = self.text.len();
            }
            Key::Char(c @ ('c' | 'x')) if chord => {
                let Some((a, b)) = sel else {
                    return Outcome::Typed;
                };
                let out = self.text[a..b].to_owned();
                if c == 'x' {
                    self.insert("");
                }
                return Outcome::Copy(out);
            }
            Key::Char('v') if chord => {
                if let Some(p) = paste {
                    self.insert(p);
                }
            }
            _ if chord || mods.alt => return Outcome::Ignored,
            _ => match text {
                Some(t) if !t.is_empty() => self.insert(t),
                _ if key == Key::Space => self.insert(" "),
                _ => return Outcome::Ignored,
            },
        }
        Outcome::Typed
    }

    /// O texto a gravar: sem espaços nas pontas; vazio = nada a gravar.
    pub fn value(&self) -> Option<String> {
        let t = self.text.trim();
        (!t.is_empty()).then(|| t.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> Modifiers {
        Modifiers::default()
    }

    #[test]
    fn opening_selects_everything_and_typing_replaces_it() {
        let mut f = Field::open(Target::NewList, "Backlog");
        assert_eq!(f.selection(), Some((0, 7)));
        f.key(Key::Char('T'), none(), Some("T"), None);
        f.key(Key::Char('o'), none(), Some("o"), None);
        assert_eq!(f.text, "To");
        assert_eq!(f.selection(), None);
    }

    #[test]
    fn backspace_and_arrows_walk_whole_characters() {
        let mut f = Field::open(Target::NewList, "");
        f.insert("Revisão");
        f.key(Key::Left, none(), None, None);
        f.key(Key::Backspace, none(), None, None);
        assert_eq!(f.text, "Revisão".replace('ã', ""));
        assert!(f.text.is_char_boundary(f.caret));
    }

    #[test]
    fn enter_commits_escape_cancels_and_blank_writes_nothing() {
        let mut f = Field::open(Target::NewList, "   ");
        assert_eq!(f.key(Key::Enter, none(), None, None), Outcome::Commit);
        assert_eq!(f.value(), None);
        assert_eq!(f.key(Key::Escape, none(), None, None), Outcome::Cancel);
    }

    #[test]
    fn a_pasted_line_break_becomes_a_space() {
        let mut f = Field::open(Target::NewList, "");
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        f.key(Key::Char('v'), ctrl, None, Some("a\nb"));
        assert_eq!(f.text, "a b");
    }
}
