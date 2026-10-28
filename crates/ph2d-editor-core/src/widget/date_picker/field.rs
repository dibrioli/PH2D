//! A caixa de texto de uma linha do seletor de datas (data, hora): texto + cursor em bytes.

use crate::doc_surface::Key;
use ph2d_a11y::{Node, NodeBuilder, Role};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineField {
    pub text: String,
    /// O cursor, em bytes, sempre numa fronteira de carácter.
    pub caret: usize,
}

/// O que uma tecla fez à caixa.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FieldKey {
    Edited,
    Moved,
    Commit,
    Cancel,
    Ignored,
}

impl LineField {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let caret = text.len();
        Self { text, caret }
    }

    pub fn insert(&mut self, s: &str) {
        let s: String = s.chars().filter(|c| !c.is_control()).collect();
        self.text.insert_str(self.caret, &s);
        self.caret += s.len();
    }

    fn prev(&self) -> usize {
        self.text[..self.caret]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    fn next(&self) -> usize {
        self.text[self.caret..]
            .chars()
            .next()
            .map_or(self.caret, |c| self.caret + c.len_utf8())
    }

    pub fn key(&mut self, key: Key, text: Option<&str>) -> FieldKey {
        match key {
            Key::Enter => FieldKey::Commit,
            Key::Escape => FieldKey::Cancel,
            Key::Backspace => {
                let p = self.prev();
                self.text.replace_range(p..self.caret, "");
                self.caret = p;
                FieldKey::Edited
            }
            Key::Delete => {
                let n = self.next();
                self.text.replace_range(self.caret..n, "");
                FieldKey::Edited
            }
            Key::Left => {
                self.caret = self.prev();
                FieldKey::Moved
            }
            Key::Right => {
                self.caret = self.next();
                FieldKey::Moved
            }
            Key::Home => {
                self.caret = 0;
                FieldKey::Moved
            }
            Key::End => {
                self.caret = self.text.len();
                FieldKey::Moved
            }
            _ => match text {
                Some(t) if !t.is_empty() && t.chars().all(|c| !c.is_control()) => {
                    self.insert(t);
                    FieldKey::Edited
                }
                _ => FieldKey::Ignored,
            },
        }
    }

    pub fn build_a11y(&self, label: &str, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::TextInput)
            .label(label)
            .bounds(x, y, w, h)
            .focusable(true)
            .build()
    }
}
