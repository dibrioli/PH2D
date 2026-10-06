//! **Escrever dentro de uma forma** — o texto é o da forma (o documento muda letra a letra, e a
//! forma cresce para baixo quando ele não cabe); o desfazer guarda a edição inteira como UM passo.

use ph2d_board_geom::{height_for_text, text_origin, text_rect};
use ph2d_board_layout::TextEdit;
use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History};
use ph2d_text::TextSystem;

use crate::{Editor, Pointer, TextKey, TextOverlay, live};

pub(crate) struct Editing {
    pub(crate) id: ElementId,
    /// O elemento como estava ao começar — o passo de desfazer, e a altura abaixo da qual a forma
    /// não encolhe.
    original: Element,
    edit: TextEdit,
}

/// A largura de quebra e o tamanho da letra de `el`, para o moldado.
fn metrics(el: &Element) -> Option<(f32, f32)> {
    let s = el.shape()?;
    let [_, _, w, _] = text_rect(s.kind, el.w, el.h);
    Some((s.style.font_size as f32, w as f32))
}

impl Editing {
    /// Mundo → espaço do texto (origem no início do bloco).
    fn text_point(&mut self, el: &Element, ts: &mut TextSystem, world: [f64; 2]) -> (f32, f32) {
        let [ux, uy] = el.unrotate(world);
        let kind = el.shape().map(|s| s.kind).expect("só se edita uma forma");
        let block_h = f64::from(self.edit.layout(ts).height());
        let [ox, oy] = text_origin(kind, el.w, el.h, block_h);
        ((ux - el.x - ox) as f32, (uy - el.y - oy) as f32)
    }

    pub(crate) fn overlay(&mut self, doc: &BoardDoc, ts: &mut TextSystem) -> Option<TextOverlay> {
        let el = doc.get(self.id)?;
        let kind = el.shape()?.kind;
        let block_h = f64::from(self.edit.layout(ts).height());
        let (selection, caret) = self.edit.decorations(1.0);
        Some(TextOverlay {
            element: self.id,
            origin: text_origin(kind, el.w, el.h, block_h),
            selection,
            caret,
        })
    }
}

impl Editor {
    /// Começa a escrever em `id`. `at` = onde o clique pôs o cursor; sem ele, tudo seleccionado.
    pub fn begin_text(
        &mut self,
        doc: &mut BoardDoc,
        ts: &mut TextSystem,
        id: ElementId,
        at: Option<[f64; 2]>,
    ) -> bool {
        let Some(el) = doc.get(id).cloned() else {
            return false;
        };
        let Some((size, width)) = metrics(&el) else {
            return false;
        };
        let text = el.shape().map_or(String::new(), |s| s.text.clone());
        let mut e = Editing {
            id,
            original: el.clone(),
            edit: TextEdit::new(ts, &text, size, width),
        };
        if let Some(world) = at {
            let (x, y) = e.text_point(&el, ts, world);
            e.edit.click(ts, x, y, false);
        }
        self.selection = std::iter::once(id).collect();
        self.editing = Some(e);
        true
    }

    /// Escrever `s` no texto em edição. `false` = não se está a escrever.
    pub fn text_input(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem, s: &str) -> bool {
        let Some(e) = self.editing.as_mut() else {
            return false;
        };
        e.edit.insert(ts, s);
        self.sync(doc, ts);
        true
    }

    /// Uma tecla dentro do texto em edição. `false` = não se está a escrever.
    pub fn text_key(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        k: TextKey,
    ) -> bool {
        let Some(e) = self.editing.as_mut() else {
            return false;
        };
        match k {
            TextKey::Backspace { word } => e.edit.backspace(ts, word),
            TextKey::Delete { word } => e.edit.delete(ts, word),
            TextKey::Move { to, extend } => e.edit.motion(ts, to, extend),
            TextKey::SelectAll => e.edit.select_all(ts),
            TextKey::Newline => e.edit.insert(ts, "\n"),
            TextKey::Commit => {
                self.commit_text(doc, history);
                return true;
            }
        }
        self.sync(doc, ts);
        true
    }

    /// O texto seleccionado na edição (para copiar).
    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        self.editing.as_ref().and_then(|e| e.edit.selected())
    }

    /// Termina a edição: o que mudou vira UM passo de desfazer.
    pub fn commit_text(&mut self, doc: &mut BoardDoc, history: &mut History) {
        let Some(e) = self.editing.take() else {
            return;
        };
        if doc.get(e.id).is_some_and(|now| *now != e.original) {
            history.record(vec![BoardOp::Put(e.original)]);
        }
        let _ = doc;
    }

    /// Escreve o texto da edição na forma e faz a forma crescer (ou voltar) à altura que ele pede.
    fn sync(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem) {
        let Some(e) = self.editing.as_mut() else {
            return;
        };
        let Some(mut el) = doc.get(e.id).cloned() else {
            return;
        };
        let Some(kind) = el.shape().map(|s| s.kind) else {
            return;
        };
        let text = e.edit.text();
        if let Some(s) = el.shape_mut() {
            s.text = text;
        }
        let block_h = f64::from(e.edit.layout(ts).height());
        el.h = height_for_text(kind, e.original.h, block_h);
        live(doc, el);
    }

    /// Um carregar enquanto se escreve: dentro da forma move o cursor (e devolve `true`); fora
    /// termina a edição e deixa o carregar seguir o caminho normal.
    pub(crate) fn text_pointer_down(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        p: Pointer,
    ) -> bool {
        let Some(e) = self.editing.as_mut() else {
            return false;
        };
        let Some(el) = doc.get(e.id).cloned() else {
            self.editing = None;
            return false;
        };
        if ph2d_board_geom::hit(&el, p.world, 0.0) {
            let (x, y) = e.text_point(&el, ts, p.world);
            e.edit.click(ts, x, y, p.mods.shift);
            return true;
        }
        self.commit_text(doc, history);
        false
    }

    pub(crate) fn text_drag(&mut self, ts: &mut TextSystem, doc: &BoardDoc, p: Pointer) {
        let Some(e) = self.editing.as_mut() else {
            return;
        };
        let Some(el) = doc.get(e.id).cloned() else {
            return;
        };
        let (x, y) = e.text_point(&el, ts, p.world);
        e.edit.drag_to(ts, x, y);
    }

    /// Duplo-clique: numa forma, escreve nela com o cursor onde se clicou (numa palavra,
    /// selecciona-a se já se estava a escrever ali). `false` = não havia forma.
    pub fn double_click(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem, p: Pointer) -> bool {
        if let Some(e) = self.editing.as_mut()
            && let Some(el) = doc.get(e.id).cloned()
            && ph2d_board_geom::hit(&el, p.world, 0.0)
        {
            let (x, y) = e.text_point(&el, ts, p.world);
            e.edit.select_word_at(ts, x, y);
            return true;
        }
        let Some(id) = self.hit(doc, p) else {
            return false;
        };
        self.gesture = None;
        self.begin_text(doc, ts, id, Some(p.world))
    }
}
