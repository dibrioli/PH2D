//! O teclado e a composição do [`super::RichEditor`]: a tecla vira uma operação do
//! `ph2d_richdoc::Editor`; as deslocações verticais e por palavra leem o layout que se pinta.

use super::{RichEditor, RichEditorEvent, RichTool};
use crate::doc_surface::{Compose, Key, KeyInput};
use crate::zones::Rect;
use parley::{Affinity, Cursor};
use ph2d_a11y::NodeId;
use ph2d_richdoc::{Caret, Selection};
use ph2d_text::TextSystem;

/// O que uma tecla fez.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RichKeyOutcome {
    pub consumed: bool,
    /// Texto para a área de transferência (copiar/cortar).
    pub copy: Option<String>,
    pub event: Option<RichEditorEvent>,
}

fn taken(event: Option<RichEditorEvent>) -> RichKeyOutcome {
    RichKeyOutcome {
        consumed: true,
        copy: None,
        event,
    }
}

/// O id a11y da barra, derivado do do editor.
#[must_use]
pub fn toolbar_node_id(editor: NodeId) -> NodeId {
    NodeId(editor.0.wrapping_add(3))
}

impl RichEditor {
    /// O texto seleccionado, em texto simples (os blocos separados por `\n`).
    #[must_use]
    pub fn selected_text(&self) -> String {
        let (a, b) = self.selection().ordered();
        let doc = self.doc();
        let mut out = String::new();
        for i in a.block..=b.block {
            let t = doc.blocks()[i].inline.text();
            let s = if i == a.block { a.byte } else { 0 };
            let e = if i == b.block { b.byte } else { t.len() };
            if i > a.block {
                out.push('\n');
            }
            out.push_str(t.get(s..e).unwrap_or_default());
        }
        out
    }

    /// Move o caret (estende a selecção com `shift`).
    fn go(&mut self, to: Caret, shift: bool) {
        let anchor = if shift { self.selection().anchor } else { to };
        self.set_selection(Selection::new(anchor, to));
    }

    /// Uma linha visual acima (`dir < 0`) ou abaixo, na mesma coluna.
    fn vertical(&mut self, ts: &mut TextSystem, rect: Rect, dir: f32) -> Caret {
        let head = self.selection().head;
        let l = self.layout(ts, rect);
        let r = l.caret_rect(head);
        let y = if dir < 0.0 { r.y - r.h * 0.5 } else { r.y + r.h + r.h * 0.5 };
        let first = l.blocks.first().map_or(r.y, |b| b.rect.y);
        let last = l.blocks.last().map_or(r.y, |b| b.rect.y + b.rect.h);
        if y < first {
            return Caret::new(0, 0);
        }
        if y > last {
            return self.doc().end();
        }
        l.caret_at(r.x, y)
    }

    /// Uma palavra à esquerda/direita; na fronteira do bloco passa ao vizinho.
    fn word(&mut self, ts: &mut TextSystem, rect: Rect, forward: bool) -> Caret {
        let head = self.selection().head;
        let l = self.layout(ts, rect);
        let b = &l.blocks[head.block];
        let at_edge = if forward { head.byte >= b.len } else { head.byte == 0 };
        if at_edge {
            return if forward {
                self.doc().move_right(head)
            } else {
                self.doc().move_left(head)
            };
        }
        let c = Cursor::from_byte_index(&b.layout, head.byte, Affinity::Downstream);
        let n = if forward {
            c.next_logical_word(&b.layout)
        } else {
            c.previous_logical_word(&b.layout)
        };
        Caret::new(head.block, n.index())
    }

    /// Início/fim da linha VISUAL.
    fn line_edge(&mut self, ts: &mut TextSystem, rect: Rect, end: bool) -> Caret {
        let head = self.selection().head;
        let l = self.layout(ts, rect);
        let b = &l.blocks[head.block];
        let s = parley::Selection::from_byte_index(&b.layout, head.byte, Affinity::Downstream);
        let s = if end {
            s.line_end(&b.layout, false)
        } else {
            s.line_start(&b.layout, false)
        };
        Caret::new(head.block, s.focus().index())
    }

    /// Uma tecla em edição (no modo leitura nada é tomado).
    pub fn key(&mut self, ts: &mut TextSystem, rect: Rect, input: &KeyInput<'_>) -> RichKeyOutcome {
        if !self.editing || !input.pressed {
            return RichKeyOutcome::default();
        }
        let m = input.mods;
        let cmd = m.ctrl || m.meta;
        if cmd && let Key::Char(c) = input.key {
            let tool = match c.to_ascii_lowercase() {
                'b' => Some(RichTool::Bold),
                'i' => Some(RichTool::Italic),
                'x' if m.shift => Some(RichTool::Strike),
                'e' => Some(RichTool::Code),
                'k' => Some(RichTool::Link),
                _ => None,
            };
            if let Some(t) = tool {
                return taken(self.apply_tool(t));
            }
            match c.to_ascii_lowercase() {
                'a' => {
                    let all = Selection::new(Caret::new(0, 0), self.doc().end());
                    self.set_selection(all);
                    return taken(None);
                }
                'c' | 'x' => {
                    let text = self.selected_text();
                    let ev = if c.eq_ignore_ascii_case(&'x') && !self.selection().is_collapsed() {
                        self.editor_mut().backspace();
                        Some(RichEditorEvent::Changed)
                    } else {
                        None
                    };
                    return RichKeyOutcome {
                        consumed: true,
                        copy: Some(text),
                        event: ev,
                    };
                }
                'v' => {
                    if let Some(p) = input.paste.as_deref() {
                        self.editor_mut().paste(p);
                        return taken(Some(RichEditorEvent::Changed));
                    }
                    return taken(None);
                }
                _ => {}
            }
        }
        let shift = m.shift;
        match input.key {
            Key::Escape => {
                if self.style_menu_open {
                    self.style_menu_open = false;
                    return taken(None);
                }
                self.editing = false;
                taken(Some(RichEditorEvent::ExitEdit))
            }
            Key::Enter if cmd => taken(Some(RichEditorEvent::Submit)),
            Key::Enter if shift => {
                self.editor_mut().line_break();
                taken(Some(RichEditorEvent::Changed))
            }
            Key::Enter => {
                self.editor_mut().enter();
                taken(Some(RichEditorEvent::Changed))
            }
            Key::Backspace => {
                self.editor_mut().backspace();
                taken(Some(RichEditorEvent::Changed))
            }
            Key::Delete => {
                self.editor_mut().delete();
                taken(Some(RichEditorEvent::Changed))
            }
            Key::Tab => {
                if shift {
                    self.editor_mut().outdent();
                } else {
                    self.editor_mut().indent();
                }
                taken(Some(RichEditorEvent::Changed))
            }
            Key::Left | Key::Right => {
                let forward = input.key == Key::Right;
                let sel = self.selection();
                let to = if cmd {
                    self.word(ts, rect, forward)
                } else if !shift && !sel.is_collapsed() {
                    let (a, b) = sel.ordered();
                    if forward { b } else { a }
                } else if forward {
                    self.doc().move_right(sel.head)
                } else {
                    self.doc().move_left(sel.head)
                };
                self.go(to, shift);
                taken(None)
            }
            Key::Up | Key::Down => {
                let dir = if input.key == Key::Up { -1.0 } else { 1.0 };
                let to = self.vertical(ts, rect, dir);
                self.go(to, shift);
                taken(None)
            }
            Key::Home | Key::End => {
                let end = input.key == Key::End;
                let to = if cmd {
                    if end { self.doc().end() } else { Caret::new(0, 0) }
                } else {
                    self.line_edge(ts, rect, end)
                };
                self.go(to, shift);
                taken(None)
            }
            _ if cmd => RichKeyOutcome::default(),
            _ => match input.text.filter(|t| !t.is_empty() && !t.chars().any(char::is_control)) {
                Some(t) => {
                    self.editor_mut().type_text(t);
                    taken(Some(RichEditorEvent::Changed))
                }
                None => RichKeyOutcome::default(),
            },
        }
    }

    /// A composição (IME ou tecla morta): a pré-edição mostra-se no caret; o texto confirmado
    /// entra como se fosse escrito.
    pub fn compose(&mut self, c: Compose<'_>) -> Option<RichEditorEvent> {
        if !self.editing {
            return None;
        }
        match c {
            Compose::Preedit(s, _) => {
                self.preedit = (!s.is_empty()).then(|| s.to_owned());
                None
            }
            Compose::Commit(ch) => {
                self.preedit = None;
                let mut buf = [0u8; 4];
                self.editor_mut().type_text(ch.encode_utf8(&mut buf));
                Some(RichEditorEvent::Changed)
            }
            Compose::DeadKey(_) => None,
        }
    }
}
