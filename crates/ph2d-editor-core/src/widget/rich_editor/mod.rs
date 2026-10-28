//! [`RichEditor`] — o editor WYSIWYG de um [`ph2d_richdoc::RichDoc`] (a descrição do cartão).
//!
//! Toda a edição é do [`ph2d_richdoc::Editor`] (inserir, apagar, marcas, tipos de bloco, atalhos
//! markdown); o widget traduz rato, teclado e composição e pinta. Mede o texto pelo MESMO layout
//! do parley que pinta. Dois modos, como no Trello: leitura (um clique entra em edição) e edição
//! (a barra de formatação por cima, o caret, a selecção).

mod keys;
mod layout;
mod paint;
mod toolbar;

pub use keys::{RichKeyOutcome, toolbar_node_id};
pub use layout::{BlockBox, DocLayout, Ink, Marker, font_px, font_weight};
pub use paint::{area_node_id, paint_rich_editor, paint_rich_editor_menu};
pub use toolbar::{RICH_TOOLS, RichTool, TEXT_STYLES, button as toolbar_button, style_option};

use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_host::Modifiers;
use ph2d_richdoc::{Caret, Editor, Mark, RichDoc, Selection};
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// O texto do editor, já traduzido pelo chamador.
#[derive(Clone, Debug, Default)]
pub struct RichEditorLabels {
    /// O que se lê com o documento vazio.
    pub placeholder: String,
    /// O glifo/rótulo curto de cada botão da barra, pela ordem de [`RICH_TOOLS`].
    pub tools: [String; RICH_TOOLS.len()],
    /// O menu «Aa»: normal, título 1, 2, 3 (ordem de [`TEXT_STYLES`]).
    pub styles: [String; TEXT_STYLES.len()],
}

/// O que o editor pede ao chamador.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RichEditorEvent {
    /// Entrou em edição (um clique no modo leitura).
    EnterEdit,
    /// `Esc`: sair da edição (o chamador decide se guarda).
    ExitEdit,
    /// `Ctrl+Enter`: guardar.
    Submit,
    /// O documento mudou.
    Changed,
    /// `Ctrl+K` ou o botão do link: o chamador pede o endereço e chama [`RichEditor::apply_link`].
    LinkRequested,
    /// Clique num link no modo leitura.
    OpenLink(String),
}

/// A chave da disposição guardada: o documento muda de versão a cada edição.
#[derive(Copy, Clone, Debug, PartialEq)]
struct LayoutKey {
    version: u64,
    rect: [u32; 3],
    editing: bool,
}

pub struct RichEditor {
    pub id: NodeId,
    pub labels: RichEditorLabels,
    editor: Editor,
    pub editing: bool,
    /// A pré-edição do IME, mostrada no caret.
    pub preedit: Option<String>,
    /// A pré-edição é o acento de uma tecla morta (o xkb compõe; a letra composta vem na tecla).
    dead: bool,
    pub style_menu_open: bool,
    pub hover: Option<toolbar::ToolbarHit>,
    dragging: bool,
    version: u64,
    cache: Option<(LayoutKey, DocLayout)>,
}

impl std::fmt::Debug for RichEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichEditor")
            .field("id", &self.id)
            .field("editing", &self.editing)
            .field("selection", &self.editor.selection())
            .finish_non_exhaustive()
    }
}

impl RichEditor {
    pub fn new(id: NodeId, doc: RichDoc, labels: RichEditorLabels) -> Self {
        Self {
            id,
            labels,
            editor: Editor::new(doc),
            editing: false,
            preedit: None,
            dead: false,
            style_menu_open: false,
            hover: None,
            dragging: false,
            version: 0,
            cache: None,
        }
    }

    #[must_use]
    pub fn doc(&self) -> &RichDoc {
        self.editor.doc()
    }

    #[must_use]
    pub fn selection(&self) -> Selection {
        self.editor.selection()
    }

    /// O `Editor` para uma mudança que o chamador faz (menção escolhida, desfazer…).
    pub fn editor_mut(&mut self) -> &mut Editor {
        self.version += 1;
        &mut self.editor
    }

    /// Troca o documento (desfazer do chamador, outro cartão).
    pub fn set_doc(&mut self, doc: RichDoc) {
        self.editor = Editor::new(doc);
        self.version += 1;
    }

    pub fn set_selection(&mut self, sel: Selection) {
        self.editor.set_selection(sel);
    }

    /// Aplica o link pedido por [`RichEditorEvent::LinkRequested`] à selecção.
    pub fn apply_link(&mut self, url: impl Into<String>) -> RichEditorEvent {
        self.editor_mut().toggle_mark(Mark::Link(url.into()));
        RichEditorEvent::Changed
    }

    pub(crate) fn changed(&mut self) -> RichEditorEvent {
        self.version += 1;
        RichEditorEvent::Changed
    }

    /// A barra (só em edição) e a área do documento dentro de `rect`.
    #[must_use]
    pub fn regions(&self, rect: Rect) -> (Option<Rect>, Rect) {
        let pad = Spacing::Md.px();
        let (bar, top) = if self.editing {
            let inset = Spacing::Xs.px();
            let bar = Rect::new(
                rect.x + inset,
                rect.y + inset,
                (rect.w - inset * 2.0).max(0.0),
                ROW_H_PX,
            );
            (Some(bar), bar.y + bar.h)
        } else {
            (None, rect.y)
        };
        let doc = Rect::new(
            rect.x + pad,
            top + pad,
            (rect.w - pad * 2.0).max(0.0),
            (rect.y + rect.h - top - pad * 2.0).max(0.0),
        );
        (bar, doc)
    }

    /// A disposição do documento em `rect` (guardada até o documento ou a caixa mudarem).
    pub fn layout(&mut self, ts: &mut TextSystem, rect: Rect) -> &DocLayout {
        let (_, d) = self.regions(rect);
        let key = LayoutKey {
            version: self.version,
            rect: [d.x.to_bits(), d.y.to_bits(), d.w.to_bits()],
            editing: self.editing,
        };
        if self.cache.as_ref().is_none_or(|(k, _)| *k != key) {
            let l = DocLayout::build(ts, self.editor.doc(), d.x, d.y, d.w);
            self.cache = Some((key, l));
        }
        &self.cache.as_ref().expect("acabou de ser posta").1
    }

    /// A altura que o editor pede na largura `w`.
    pub fn preferred_height(&mut self, ts: &mut TextSystem, w: f32) -> f32 {
        let rect = Rect::new(0.0, 0.0, w, 0.0);
        let (_, d) = self.regions(rect);
        let h = self.layout(ts, rect).height;
        d.y + h + Spacing::Md.px()
    }

    /// Um clique. `clicks` = 1, 2 (palavra) ou 3 (bloco), como o sistema os conta.
    pub fn pointer_down(
        &mut self,
        ts: &mut TextSystem,
        rect: Rect,
        x: f32,
        y: f32,
        mods: Modifiers,
        clicks: u8,
    ) -> Option<RichEditorEvent> {
        let (bar, _) = self.regions(rect);
        if self.style_menu_open {
            self.style_menu_open = false;
            if let Some(bar) = bar
                && let Some(i) = toolbar::style_option_at(bar, ts, &self.labels.styles, x, y)
            {
                self.editor_mut().set_block_kind(TEXT_STYLES[i].clone());
                return Some(self.changed());
            }
            return None;
        }
        if let Some(bar) = bar
            && let Some(hit) = toolbar::hit(bar, x, y)
        {
            return self.apply_tool(RICH_TOOLS[hit.0]);
        }
        let l = self.layout(ts, rect);
        if let Some(i) = l.check_at(x, y) {
            self.editor_mut().toggle_check(i);
            return Some(RichEditorEvent::Changed);
        }
        let caret = l.caret_at(x, y);
        let word = l.word_at(x, y);
        if !self.editing {
            let link = self.editor.doc().blocks()[caret.block]
                .inline
                .marks_on_byte(caret.byte)
                .into_iter()
                .find_map(|m| match m {
                    Mark::Link(u) => Some(u),
                    _ => None,
                });
            if let Some(u) = link {
                return Some(RichEditorEvent::OpenLink(u));
            }
            self.editing = true;
            self.editor.set_selection(Selection::caret(caret));
            return Some(RichEditorEvent::EnterEdit);
        }
        let sel = if mods.shift {
            Selection::new(self.editor.selection().anchor, caret)
        } else if clicks == 2 {
            word
        } else if clicks >= 3 {
            let len = self.editor.doc().blocks()[caret.block].inline.len();
            Selection::new(Caret::new(caret.block, 0), Caret::new(caret.block, len))
        } else {
            Selection::caret(caret)
        };
        self.editor.set_selection(sel);
        self.dragging = clicks == 1;
        None
    }

    /// O rato a arrastar com o botão em baixo: estende a selecção.
    pub fn pointer_drag(&mut self, ts: &mut TextSystem, rect: Rect, x: f32, y: f32) {
        if !self.dragging {
            return;
        }
        let caret = self.layout(ts, rect).caret_at(x, y);
        let anchor = self.editor.selection().anchor;
        self.editor.set_selection(Selection::new(anchor, caret));
    }

    pub fn pointer_up(&mut self) {
        self.dragging = false;
    }

    /// O rato a passar pela barra: `true` quando o realce mudou.
    pub fn pointer_move(&mut self, rect: Rect, x: f32, y: f32) -> bool {
        let (bar, _) = self.regions(rect);
        let h = bar.and_then(|b| toolbar::hit(b, x, y));
        let changed = h != self.hover;
        self.hover = h;
        changed
    }

    /// Um botão da barra (ou o atalho de teclado dele).
    pub fn apply_tool(&mut self, tool: RichTool) -> Option<RichEditorEvent> {
        match toolbar::action(tool, self.editor.doc(), self.editor.selection()) {
            toolbar::Action::StyleMenu => {
                self.style_menu_open = !self.style_menu_open;
                None
            }
            toolbar::Action::Link => Some(RichEditorEvent::LinkRequested),
            toolbar::Action::Mark(m) => {
                self.editor.toggle_mark(m);
                Some(self.changed())
            }
            toolbar::Action::Block(k) => {
                self.editor.set_block_kind(k);
                Some(self.changed())
            }
        }
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::MultilineTextInput)
            .label(&self.labels.placeholder)
            .bounds(x, y, w, h)
            .focusable(true)
            .action(Action::Focus)
            .build()
    }
}
