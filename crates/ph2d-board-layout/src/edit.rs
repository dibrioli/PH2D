//! ⭐ **Editar o texto de uma forma**, com estilo por trecho — o cursor e a selecção são os do
//! parley (`Selection`/`Cursor`: palavras, linhas, grafemas, bidi) sobre o MESMO moldado que se
//! desenha ([`crate::shape`]), para o que se edita ser o que se vê.
//!
//! ⚠️ Não é o `PlainEditor` do parley: aquele tem UM estilo para o texto inteiro, e um trecho a
//! negrito (mais largo) punha o cursor fora das letras. Os passos de apagar são os dele (Apache-2.0
//! OR MIT): um grafema para trás exceto quebra de linha e emoji, que vão inteiros.

use std::ops::Range;

use parley::{Affinity, Cursor, Layout, LayoutContext, Selection};
use ph2d_board_model::{Mark, Marks, Rgba, RichText};
use ph2d_text::TextSystem;

use crate::Ink;

/// Um movimento do cursor (com `extend` = Shift, a selecção cresce em vez de colapsar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    LineStart,
    LineEnd,
    TextStart,
    TextEnd,
}

/// O texto em edição: o documento ([`RichText`]), a selecção e o moldado em dia.
pub struct TextEdit {
    rich: RichText,
    sel: Selection,
    /// As marcas com que o PRÓXIMO texto entra num cursor sem selecção (um Ctrl+B sem selecção
    /// liga o negrito para o que se vai escrever). `None` = as do carácter antes do cursor.
    typing: Option<Marks>,
    font_size: f32,
    width: f32,
    lcx: LayoutContext<Ink>,
    layout: Layout<Ink>,
    /// O texto (ou o estilo) mudou sem o moldador à mão: remolda no próximo uso.
    dirty: bool,
    /// Abriu com TUDO seleccionado e ainda não moldou: a selecção resolve-se no próximo uso.
    all: bool,
    /// Na letra à mão (o texto de uma forma em rascunho) — a MESMA que o desenho usa.
    hand: bool,
}

impl TextEdit {
    /// Abre `text` para edição com tudo seleccionado (escrever substitui; uma seta colapsa). Molda
    /// no primeiro uso — quem abre (um botão da barra) pode não ter o moldador à mão.
    #[must_use]
    pub fn new(text: &RichText, font_size: f32, max_width: f32) -> Self {
        Self {
            rich: text.clone(),
            sel: Selection::default(),
            typing: None,
            font_size,
            width: max_width.max(1.0),
            lcx: LayoutContext::new(),
            layout: Layout::new(),
            dirty: true,
            all: true,
            hand: false,
        }
    }

    /// Na letra à mão (`true`) — a de uma forma em rascunho.
    #[must_use]
    pub fn hand(mut self, hand: bool) -> Self {
        self.hand = hand;
        self.dirty = true;
        self
    }

    /// Molda o que ficou por moldar (e resolve o «tudo seleccionado» de quem abriu).
    fn ensure(&mut self, ts: &mut TextSystem) {
        if self.dirty {
            self.relayout(ts);
        }
        if self.all {
            self.all = false;
            self.sel = Selection::from_byte_index(&self.layout, 0, Affinity::default()).move_lines(
                &self.layout,
                isize::MAX,
                true,
            );
        }
    }

    /// Os bytes seleccionados, mesmo antes do primeiro moldar.
    fn sel_range(&self) -> Range<usize> {
        if self.all {
            0..self.rich.len()
        } else {
            self.sel.text_range()
        }
    }

    fn relayout(&mut self, ts: &mut TextSystem) {
        self.dirty = false;
        self.layout = crate::shape(
            ts,
            &mut self.lcx,
            self.rich.as_str(),
            self.rich.spans(),
            self.font_size,
            self.width,
            self.hand,
        );
        self.sel = self.sel.refresh(&self.layout);
    }

    fn set_sel(&mut self, sel: Selection) {
        if sel.focus() != self.sel.focus() || sel.anchor() != self.sel.anchor() {
            self.typing = None;
        }
        self.sel = sel;
    }

    fn cursor(&self, index: usize, affinity: Affinity) -> Selection {
        Cursor::from_byte_index(&self.layout, index, affinity).into()
    }

    /// O texto, com os trechos.
    #[must_use]
    pub fn rich(&self) -> &RichText {
        &self.rich
    }

    #[must_use]
    pub fn text(&self) -> String {
        self.rich.as_str().to_owned()
    }

    /// Os bytes seleccionados (vazio = só o cursor).
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.sel_range()
    }

    /// A largura de quebra mudou (a forma foi redimensionada).
    pub fn set_width(&mut self, ts: &mut TextSystem, max_width: f32) {
        self.width = max_width.max(1.0);
        self.relayout(ts);
    }

    /// Troca `range` por `s` (com as marcas de escrever ali) e põe o cursor no fim.
    fn replace(&mut self, ts: &mut TextSystem, range: Range<usize>, s: &str) {
        self.ensure(ts);
        let marks = self
            .typing
            .unwrap_or_else(|| self.rich.typing_marks(range.start));
        self.rich.replace(range.clone(), s, marks);
        self.relayout(ts);
        let end = range.start + s.len();
        let affinity = if s.ends_with(['\n', '\r', '\u{2028}', '\u{2029}']) {
            Affinity::Downstream
        } else {
            Affinity::Upstream
        };
        self.sel = self.cursor(end, affinity);
        // O que se escreveu já tem as marcas pedidas, e o carácter seguinte herda-as do anterior
        // (`typing_marks`): guardá-las aqui seria uma segunda fonte (a prova de mutação M8 mostrou-o).
        self.typing = None;
    }

    pub fn insert(&mut self, ts: &mut TextSystem, s: &str) {
        self.ensure(ts);
        self.replace(ts, self.sel.text_range(), s);
    }

    /// Apaga a selecção, ou o grafema (a palavra, com `word`) antes do cursor.
    pub fn backspace(&mut self, ts: &mut TextSystem, word: bool) {
        self.ensure(ts);
        if !self.sel.is_collapsed() {
            return self.replace(ts, self.sel.text_range(), "");
        }
        let focus = self.sel.focus();
        let end = focus.index();
        let start = if word {
            focus.previous_logical_word(&self.layout).index()
        } else {
            let Some(cluster) = focus.logical_clusters(&self.layout)[0] else {
                return;
            };
            let r = cluster.text_range();
            if cluster.is_hard_line_break() || cluster.is_emoji() {
                r.start
            } else {
                match self.rich.as_str()[..r.end].char_indices().next_back() {
                    Some((i, _)) => i,
                    None => return,
                }
            }
        };
        if start < end {
            self.typing = None;
            self.rich.replace(start..end, "", Marks::default());
            self.relayout(ts);
            self.sel = self.cursor(start, Affinity::Downstream);
        }
    }

    /// Apaga a selecção, ou o grafema (a palavra, com `word`) depois do cursor.
    pub fn delete(&mut self, ts: &mut TextSystem, word: bool) {
        self.ensure(ts);
        if !self.sel.is_collapsed() {
            return self.replace(ts, self.sel.text_range(), "");
        }
        let focus = self.sel.focus();
        let start = focus.index();
        let end = if word {
            focus.next_logical_word(&self.layout).index()
        } else {
            match focus.logical_clusters(&self.layout)[1].as_ref() {
                Some(c) => c.text_range().end,
                None => return,
            }
        };
        if start < end {
            self.typing = None;
            self.rich.replace(start..end, "", Marks::default());
            self.relayout(ts);
            self.sel = self.cursor(start, Affinity::Downstream);
        }
    }

    pub fn select_all(&mut self, ts: &mut TextSystem) {
        self.ensure(ts);
        let all = Selection::from_byte_index(&self.layout, 0, Affinity::default()).move_lines(
            &self.layout,
            isize::MAX,
            true,
        );
        self.set_sel(all);
    }

    /// Selecciona `range` (bytes).
    pub fn select_range(&mut self, ts: &mut TextSystem, range: Range<usize>) {
        self.ensure(ts);
        let a = Cursor::from_byte_index(&self.layout, range.start, Affinity::Downstream);
        let b = Cursor::from_byte_index(&self.layout, range.end, Affinity::Upstream);
        self.set_sel(Selection::new(a, b));
    }

    /// O texto seleccionado (para copiar), se há selecção.
    #[must_use]
    pub fn selected(&self) -> Option<String> {
        let r = self.sel_range();
        (!r.is_empty()).then(|| self.rich.as_str()[r].to_owned())
    }

    pub fn motion(&mut self, ts: &mut TextSystem, m: Move, extend: bool) {
        self.ensure(ts);
        let l = &self.layout;
        let s = &self.sel;
        let end = |at: usize, aff| -> Selection {
            let c = Cursor::from_byte_index(l, at, aff);
            if extend { s.extend(c) } else { c.into() }
        };
        let next = match m {
            Move::Left => s.previous_visual(l, extend),
            Move::Right => s.next_visual(l, extend),
            Move::WordLeft => s.previous_visual_word(l, extend),
            Move::WordRight => s.next_visual_word(l, extend),
            Move::Up => s.previous_line(l, extend),
            Move::Down => s.next_line(l, extend),
            Move::LineStart => s.line_start(l, extend),
            Move::LineEnd => s.line_end(l, extend),
            Move::TextStart => end(0, Affinity::Downstream),
            Move::TextEnd => end(self.rich.len(), Affinity::Upstream),
        };
        self.set_sel(next);
    }

    /// Um clique em `(x, y)` (espaço do texto). `extend` = Shift.
    pub fn click(&mut self, ts: &mut TextSystem, x: f32, y: f32, extend: bool) {
        self.ensure(ts);
        let sel = if extend {
            self.sel.shift_click_extension(&self.layout, x, y)
        } else {
            Selection::from_point(&self.layout, x, y)
        };
        self.set_sel(sel);
    }

    /// Arrastar com o botão em baixo: a selecção vai até `(x, y)`.
    pub fn drag_to(&mut self, ts: &mut TextSystem, x: f32, y: f32) {
        self.ensure(ts);
        let sel = self.sel.extend_to_point(&self.layout, x, y);
        self.set_sel(sel);
    }

    /// Duplo-clique: a palavra em `(x, y)`.
    pub fn select_word_at(&mut self, ts: &mut TextSystem, x: f32, y: f32) {
        self.ensure(ts);
        self.set_sel(Selection::word_from_point(&self.layout, x, y));
    }

    /// ⭐ Liga ou desliga `mark` na selecção (o Ctrl+B dos editores: se toda a selecção a tem,
    /// desliga). Sem selecção, vale para o que se escrever a seguir. Remolda no próximo uso.
    pub fn toggle(&mut self, mark: Mark) {
        let r = self.sel_range();
        if r.is_empty() {
            let mut m = self.marks_here();
            let on = !mark.get(&m);
            mark.set(&mut m, on);
            self.typing = Some(m);
            return;
        }
        self.rich.toggle(r, mark);
        self.dirty = true;
    }

    /// Pinta a selecção com `color` (`None` = a tinta da forma). Sem selecção, o que se escrever.
    pub fn set_color(&mut self, color: Option<Rgba>) {
        let r = self.sel_range();
        if r.is_empty() {
            let mut m = self.marks_here();
            m.color = color;
            self.typing = Some(m);
            return;
        }
        self.rich.restyle(r, |m| m.color = color);
        self.dirty = true;
    }

    /// A marca está ligada na selecção (em TODA) — ou para o que se escrever, sem selecção?
    #[must_use]
    pub fn has(&self, mark: Mark) -> bool {
        let r = self.sel_range();
        if r.is_empty() {
            return mark.get(&self.marks_here());
        }
        self.rich.all(r, mark)
    }

    /// A cor única da selecção (`Some(None)` = a da forma; `None` = várias).
    #[must_use]
    pub fn color(&self) -> Option<Option<Rgba>> {
        let r = self.sel_range();
        if r.is_empty() {
            return Some(self.marks_here().color);
        }
        self.rich.color(r)
    }

    fn marks_here(&self) -> Marks {
        self.typing
            .unwrap_or_else(|| self.rich.typing_marks(self.sel_range().end))
    }

    /// O moldado actual (remoldado se o estilo mudou).
    pub fn layout(&mut self, ts: &mut TextSystem) -> &Layout<Ink> {
        self.ensure(ts);
        &self.layout
    }

    /// Os rectângulos da selecção e o do cursor, `[x0, y0, x1, y1]` no espaço do texto. Pede
    /// [`Self::layout`] antes (é ele que põe o moldado em dia).
    #[must_use]
    pub fn decorations(&self, caret_w: f32) -> (Vec<[f64; 4]>, Option<[f64; 4]>) {
        let sel = self
            .sel
            .geometry(&self.layout)
            .into_iter()
            .map(|(b, _)| [b.x0, b.y0, b.x1, b.y1])
            .collect();
        let b = self.sel.focus().geometry(&self.layout, caret_w);
        (sel, Some([b.x0, b.y0, b.x1, b.y1]))
    }
}
