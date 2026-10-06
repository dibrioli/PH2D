//! **Escrever dentro de uma forma** — o texto é o da forma (o documento muda letra a letra, e a
//! forma cresce para baixo quando ele não cabe); o desfazer guarda a edição inteira como UM passo.
//! Numa seta, o mesmo no RÓTULO, centrado no meio da rota (W2).

use ph2d_board_geom::{height_for_text, text_origin, text_rect};
use ph2d_board_layout::TextEdit;
use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History};
use ph2d_board_route::{LABEL_WRAP, label_origin};
use ph2d_text::TextSystem;

use crate::{Editor, Pointer, TextKey, TextOverlay, live};

pub(crate) struct Editing {
    pub(crate) id: ElementId,
    /// O elemento como estava ao começar — o passo de desfazer, e a altura abaixo da qual a forma
    /// não encolhe.
    original: Element,
    edit: TextEdit,
}

/// O tamanho da letra e a largura de quebra de `el`, para o moldado.
fn metrics(el: &Element) -> Option<(f32, f32)> {
    let w = match el.shape() {
        Some(s) => text_rect(s.kind, el.w, el.h)[2],
        None => LABEL_WRAP,
    };
    Some((el.style().font_size as f32, w as f32))
}

/// O texto de `el` (o da forma, ou o rótulo da seta).
fn text_of(el: &Element) -> String {
    match (el.shape(), el.connector()) {
        (Some(s), _) => s.text.clone(),
        (_, Some(c)) => c.label.clone(),
        _ => String::new(),
    }
}

/// Onde começa o bloco de texto, nas coordenadas da caixa de `el`. Numa seta a caixa é o próprio
/// mundo (`x = y = 0`, sem rotação) e o bloco centra-se no meio `mid` da rota.
fn origin(el: &Element, block_h: f64, mid: Option<[f64; 2]>) -> Option<[f64; 2]> {
    match el.shape() {
        Some(s) => Some(text_origin(s.kind, el.w, el.h, block_h)),
        None => mid.map(|m| label_origin(m, block_h)),
    }
}

impl Editing {
    /// Mundo → espaço do texto (origem no início do bloco).
    fn text_point(
        &mut self,
        el: &Element,
        ts: &mut TextSystem,
        world: [f64; 2],
        mid: Option<[f64; 2]>,
    ) -> (f32, f32) {
        let [ux, uy] = el.unrotate(world);
        let block_h = f64::from(self.edit.layout(ts).height());
        let [ox, oy] = origin(el, block_h, mid).unwrap_or([0.0, 0.0]);
        ((ux - el.x - ox) as f32, (uy - el.y - oy) as f32)
    }

    /// `world` cai na zona de escrever de `el` (o contorno da forma, ou o bloco do rótulo)?
    fn contains(
        &mut self,
        el: &Element,
        ts: &mut TextSystem,
        world: [f64; 2],
        mid: Option<[f64; 2]>,
    ) -> bool {
        if el.shape().is_some() {
            return ph2d_board_geom::hit(el, world, 0.0);
        }
        let layout = self.edit.layout(ts);
        let block_h = f64::from(layout.height()).max(el.style().font_size);
        let Some([x, y]) = mid.map(|m| label_origin(m, block_h)) else {
            return false;
        };
        world[0] >= x && world[0] <= x + LABEL_WRAP && world[1] >= y && world[1] <= y + block_h
    }

    pub(crate) fn overlay(
        &mut self,
        doc: &BoardDoc,
        ts: &mut TextSystem,
        mid: Option<[f64; 2]>,
    ) -> Option<TextOverlay> {
        let el = doc.get(self.id)?;
        let block_h = f64::from(self.edit.layout(ts).height());
        let (selection, caret) = self.edit.decorations(1.0);
        Some(TextOverlay {
            element: self.id,
            origin: origin(el, block_h, mid)?,
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
        let text = text_of(&el);
        let mut e = Editing {
            id,
            original: el.clone(),
            edit: TextEdit::new(ts, &text, size, width),
        };
        if let Some(world) = at {
            let mid = self.routes(doc).get(id).map(|r| r.mid);
            let (x, y) = e.text_point(&el, ts, world, mid);
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
        let text = e.edit.text();
        if let Some(c) = el.connector_mut() {
            c.label = text;
            live(doc, el);
            return;
        }
        let Some(kind) = el.shape().map(|s| s.kind) else {
            return;
        };
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
        let Some(id) = self.editing.as_ref().map(|e| e.id) else {
            return false;
        };
        let Some(el) = doc.get(id).cloned() else {
            self.editing = None;
            return false;
        };
        let mid = self.routes(doc).get(id).map(|r| r.mid);
        let e = self.editing.as_mut().expect("visto acima");
        if e.contains(&el, ts, p.world, mid) {
            let (x, y) = e.text_point(&el, ts, p.world, mid);
            e.edit.click(ts, x, y, p.mods.shift);
            return true;
        }
        self.commit_text(doc, history);
        false
    }

    pub(crate) fn text_drag(&mut self, ts: &mut TextSystem, doc: &BoardDoc, p: Pointer) {
        let Some(id) = self.editing.as_ref().map(|e| e.id) else {
            return;
        };
        let Some(el) = doc.get(id).cloned() else {
            return;
        };
        let mid = self.routes(doc).get(id).map(|r| r.mid);
        let e = self.editing.as_mut().expect("visto acima");
        let (x, y) = e.text_point(&el, ts, p.world, mid);
        e.edit.drag_to(ts, x, y);
    }

    /// Duplo-clique: numa forma, escreve nela com o cursor onde se clicou (numa palavra,
    /// selecciona-a se já se estava a escrever ali). `false` = não havia forma.
    pub fn double_click(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        p: Pointer,
    ) -> bool {
        // Num ponto de ajuste de uma seta seleccionada, o duplo-clique apaga-o.
        if self.editing.is_none() && self.remove_point_at(doc, history, p) {
            return true;
        }
        if let Some(id) = self.editing.as_ref().map(|e| e.id)
            && let Some(el) = doc.get(id).cloned()
        {
            let mid = self.routes(doc).get(id).map(|r| r.mid);
            let e = self.editing.as_mut().expect("visto acima");
            if e.contains(&el, ts, p.world, mid) {
                let (x, y) = e.text_point(&el, ts, p.world, mid);
                e.edit.select_word_at(ts, x, y);
                return true;
            }
        }
        let Some(id) = self.hit(doc, p) else {
            return false;
        };
        self.gesture = None;
        self.begin_text(doc, ts, id, Some(p.world))
    }
}
