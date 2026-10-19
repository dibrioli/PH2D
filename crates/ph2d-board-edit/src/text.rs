//! **Escrever dentro de uma forma** — o texto é o da forma (o documento muda letra a letra, e a
//! forma cresce para baixo quando ele não cabe); o desfazer guarda a edição inteira como UM passo.
//! Numa seta, o mesmo num dos TEXTOS ao longo da linha (W23 B5(2), `caption.rs`): o bloco roda com
//! ele, e o texto que ficou vazio sai ao terminar.

use ph2d_board_geom::{height_for_text, text_origin, text_rect};
use ph2d_board_layout::TextEdit;
use ph2d_board_model::{BoardDoc, BoardOp, Caption, Element, ElementId, History, RichText};
use ph2d_board_route::{CaptionFrame, LABEL_WRAP};
use ph2d_text::TextSystem;

use crate::{Editor, Pointer, TextKey, TextOverlay, live};

pub(crate) struct Editing {
    pub(crate) id: ElementId,
    /// Numa seta, o texto em que se escreve.
    pub(crate) caption: Option<usize>,
    /// O elemento como estava ao começar — o passo de desfazer, e a altura abaixo da qual a forma
    /// não encolhe.
    original: Element,
    pub(crate) edit: TextEdit,
}

/// O tamanho da letra e a largura de quebra de `el`, para o moldado.
fn metrics(el: &Element) -> Option<(f32, f32)> {
    let w = match el.shape() {
        Some(s) => text_rect(s.kind, el.w, el.h)[2],
        None => LABEL_WRAP,
    };
    Some((el.style().font_size as f32, w as f32))
}

/// Em que texto de uma seta se vai escrever.
#[derive(Clone, Copy)]
pub(crate) enum CaptionPick {
    Existing(usize),
    /// Um texto novo, vazio, a esta fracção da linha.
    New(f64),
}

/// O texto de `el` (o da forma, com os trechos, ou o texto `caption` da seta).
fn text_of(el: &Element, caption: Option<usize>) -> RichText {
    match (el.shape(), el.connector()) {
        (Some(s), _) => s.text.clone(),
        (_, Some(c)) => caption
            .and_then(|i| c.captions.get(i))
            .map_or_else(RichText::default, |cap| RichText::plain(&cap.text)),
        _ => RichText::default(),
    }
}

/// Onde começa o bloco de texto: numa forma, nas coordenadas da caixa dela; numa seta, no espaço do
/// texto (origem no centro dele, [`CaptionFrame`]).
fn origin(el: &Element, block_h: f64) -> [f64; 2] {
    match el.shape() {
        Some(s) => text_origin(s.kind, el.w, el.h, block_h, s.style.look.valign),
        None => CaptionFrame::block_origin(block_h),
    }
}

impl Editing {
    /// Mundo → espaço do texto (origem no início do bloco). `frame` = onde está o texto da seta.
    fn text_point(
        &mut self,
        el: &Element,
        ts: &mut TextSystem,
        world: [f64; 2],
        frame: Option<&CaptionFrame>,
    ) -> (f32, f32) {
        let [ux, uy] = match frame {
            Some(f) => f.to_local(world),
            None => {
                let [ux, uy] = el.unrotate(world);
                [ux - el.x, uy - el.y]
            }
        };
        let block_h = f64::from(self.edit.layout(ts).height());
        let [ox, oy] = origin(el, block_h);
        ((ux - ox) as f32, (uy - oy) as f32)
    }

    /// `world` cai na zona de escrever de `el` (o contorno da forma, ou o bloco do texto da seta)?
    fn contains(
        &mut self,
        el: &Element,
        ts: &mut TextSystem,
        world: [f64; 2],
        frame: Option<&CaptionFrame>,
    ) -> bool {
        if el.shape().is_some() {
            return ph2d_board_geom::hit(el, world, 0.0);
        }
        let block_h = f64::from(self.edit.layout(ts).height()).max(el.style().font_size);
        frame.is_some_and(|f| f.contains(world, block_h))
    }

    pub(crate) fn overlay(
        &mut self,
        doc: &BoardDoc,
        ts: &mut TextSystem,
        frame: Option<&CaptionFrame>,
    ) -> Option<TextOverlay> {
        let el = doc.get(self.id)?;
        if el.connector().is_some() && frame.is_none() {
            return None;
        }
        let block_h = f64::from(self.edit.layout(ts).height());
        let (selection, caret) = self.edit.decorations(1.0);
        Some(TextOverlay {
            element: self.id,
            caption: self.caption,
            frame: frame.cloned(),
            origin: origin(el, block_h),
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
        if doc.get(id).is_some_and(|el| el.locked()) {
            self.notice = Some(crate::Notice::Locked);
            return false;
        }
        if self.unread.contains(&id) {
            self.notice = Some(crate::Notice::Private);
            return false;
        }
        let pick = self.caption_pick(doc, ts, id, at);
        if !self.open_text_in(doc, id, pick) {
            return false;
        }
        if let Some(world) = at
            && let Some(el) = doc.get(id).cloned()
        {
            let frame = self.editing_frame(doc);
            let e = self.editing.as_mut().expect("acabou de abrir");
            let (x, y) = e.text_point(&el, ts, world, frame.as_ref());
            e.edit.click(ts, x, y, false);
        }
        true
    }

    /// Numa seta, em que texto se escreve: o que está sob `at`; senão um novo no ponto da linha mais
    /// perto de `at` (o duplo clique do Miro); sem `at`, o 1.º (ou um novo a meio).
    fn caption_pick(
        &mut self,
        doc: &BoardDoc,
        ts: &mut TextSystem,
        id: ElementId,
        at: Option<[f64; 2]>,
    ) -> Option<CaptionPick> {
        let el = doc.get(id)?;
        let c = el.connector()?;
        self.measure_captions(doc, ts);
        self.routes.sync(doc);
        let Some(world) = at else {
            return Some(first_caption(c));
        };
        if let Some(i) = self.caption_of(el, world) {
            return Some(CaptionPick::Existing(i));
        }
        let frac = self
            .routes
            .get(id)
            .map_or(Caption::MIDDLE, |r| r.frac_of(world));
        Some(CaptionPick::New(frac))
    }

    /// Onde está o texto da seta em edição, se se escreve numa.
    pub(crate) fn editing_frame(&self, doc: &BoardDoc) -> Option<CaptionFrame> {
        let e = self.editing.as_ref()?;
        let c = doc.get(e.id)?.connector()?;
        let cap = c.captions.get(e.caption?)?;
        Some(self.routes.get(e.id)?.caption_frame(cap.at, c.turn))
    }

    /// Começa a escrever em `id` com o texto todo seleccionado — sem o moldador (molda no primeiro
    /// uso): o que um botão da barra pode pedir. Numa seta, no 1.º texto (ou num novo a meio).
    pub(crate) fn open_text(&mut self, doc: &mut BoardDoc, id: ElementId) -> bool {
        let pick = doc.get(id).and_then(|el| el.connector()).map(first_caption);
        self.open_text_in(doc, id, pick)
    }

    fn open_text_in(
        &mut self,
        doc: &mut BoardDoc,
        id: ElementId,
        pick: Option<CaptionPick>,
    ) -> bool {
        let Some(el) = doc.get(id).cloned() else {
            return false;
        };
        // A pilha de notas não tem texto: dela tiram-se notas.
        if el
            .shape()
            .is_some_and(|s| s.kind == ph2d_board_model::ShapeType::StickyStack)
        {
            return false;
        }
        let Some((size, width)) = metrics(&el) else {
            return false;
        };
        let caption = match pick {
            Some(CaptionPick::Existing(i)) => Some(i),
            Some(CaptionPick::New(at)) => {
                let mut grown = el.clone();
                let c = grown.connector_mut().expect("só uma seta tem textos");
                c.captions.push(Caption::new("", at));
                let i = c.captions.len() - 1;
                live(doc, grown);
                Some(i)
            }
            None => None,
        };
        let text = text_of(doc.get(id).unwrap_or(&el), caption);
        let hand = hand_lettered(&el);
        let align = el.style().look.align;
        self.selection = std::iter::once(id).collect();
        self.editing = Some(Editing {
            id,
            caption,
            original: el,
            edit: TextEdit::new(&text, size, width).hand(hand).align(align),
        });
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

    /// Termina a edição: o que mudou vira UM passo de desfazer. No rascunho do modo em massa, as
    /// linhas viram notas.
    pub fn commit_text(&mut self, doc: &mut BoardDoc, history: &mut History) {
        let Some(e) = self.editing.take() else {
            return;
        };
        if self.bulk == Some(e.id) {
            self.finish_bulk(doc, history, e.id);
            return;
        }
        if let Some(mut el) = doc.get(e.id).cloned()
            && let Some(c) = el.connector_mut()
            && c.captions.iter().any(|cap| cap.text.is_empty())
        {
            c.drop_empty_captions();
            live(doc, el);
        }
        // Pelos registos (um texto nascido e largado vazio mexeu na versão, não no elemento).
        if doc
            .get(e.id)
            .is_some_and(|now| !now.changed_fields(&e.original).is_empty())
        {
            history.record(vec![BoardOp::Put(e.original)], doc);
        }
        let _ = doc;
    }

    /// Move o cursor do texto em edição (sem estender a selecção).
    pub(crate) fn text_key_motion(&mut self, ts: &mut TextSystem, m: crate::Move) {
        if let Some(e) = self.editing.as_mut() {
            e.edit.motion(ts, m, false);
        }
    }

    /// Ajusta ao texto a altura das notas que nasceram sem o moldador à mão.
    pub(crate) fn fit_pending(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem) {
        for id in std::mem::take(&mut self.unfitted) {
            if let Some(mut el) = doc.get(id).cloned() {
                if el.mind().is_some() {
                    live(doc, crate::mindmap::refit(ts, &el));
                    crate::mindmap::settle_live(doc, id);
                    continue;
                }
                let h = crate::notes::fitted_height(ts, &el);
                if h != el.h {
                    el.h = h;
                    live(doc, el);
                }
            }
        }
    }

    /// Escreve o texto da edição na forma e faz a forma crescer (ou voltar) à altura que ele pede —
    /// numa nota, nunca abaixo da de nascença; numa forma, da de quando começou a edição.
    pub(crate) fn sync(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem) {
        let Some(e) = self.editing.as_mut() else {
            return;
        };
        let Some(mut el) = doc.get(e.id).cloned() else {
            return;
        };
        if let Some(c) = el.connector_mut() {
            if let Some(cap) = e.caption.and_then(|i| c.captions.get_mut(i)) {
                cap.text = e.edit.text();
                live(doc, el);
            }
            return;
        }
        let Some(kind) = el.shape().map(|s| s.kind) else {
            return;
        };
        if let Some(s) = el.shape_mut() {
            s.text = e.edit.rich().clone();
        }
        if el.mind().is_some() {
            // Um nó de mapa cresce de LADO com o texto (M15) e o mapa abre espaço ao vivo.
            let fitted = crate::mindmap::refit(ts, &el);
            let tw = text_rect(kind, fitted.w, fitted.h)[2];
            e.edit.set_width(ts, tw as f32);
            let id = fitted.id;
            live(doc, fitted);
            crate::mindmap::settle_live(doc, id);
            return;
        }
        let block_h = f64::from(e.edit.layout(ts).height());
        let floor = kind.note_aspect().map_or(e.original.h, |a| el.w * a);
        el.h = height_for_text(kind, floor, block_h);
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
        self.routes.sync(doc);
        let frame = self.editing_frame(doc);
        let e = self.editing.as_mut().expect("visto acima");
        if e.contains(&el, ts, p.world, frame.as_ref()) {
            let (x, y) = e.text_point(&el, ts, p.world, frame.as_ref());
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
        self.routes.sync(doc);
        let frame = self.editing_frame(doc);
        let e = self.editing.as_mut().expect("visto acima");
        let (x, y) = e.text_point(&el, ts, p.world, frame.as_ref());
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
        // W16 — no recorte o duplo-clique não sai (o do Excalidraw): segue como um clique (agarra uma
        // pega ou desliza a imagem).
        if self.cropping().is_some() {
            return false;
        }
        // Num ponto de ajuste de uma seta seleccionada, o duplo-clique apaga-o.
        if self.editing.is_none()
            && (self.remove_point_at(doc, history, p) || self.let_go_segment_at(doc, history, p))
        {
            return true;
        }
        if let Some(id) = self.editing.as_ref().map(|e| e.id)
            && let Some(el) = doc.get(id).cloned()
        {
            self.routes.sync(doc);
            let frame = self.editing_frame(doc);
            let e = self.editing.as_mut().expect("visto acima");
            if e.contains(&el, ts, p.world, frame.as_ref()) {
                let (x, y) = e.text_point(&el, ts, p.world, frame.as_ref());
                e.edit.select_word_at(ts, x, y);
                return true;
            }
        }
        let Some(id) = self.hit(doc, p) else {
            return false;
        };
        self.gesture = None;
        // Num membro de um grupo seleccionado, o duplo-clique entra um nível (oráculo B); só no
        // elemento sozinho é que escreve.
        if self.enter_group(doc, id) {
            return true;
        }
        // W16 — numa IMAGEM o duplo-clique recorta (antes abria um texto vazio por cima dela).
        if doc.get(id).is_some_and(|e| e.image().is_some()) {
            self.selection = std::collections::BTreeSet::from([id]);
            return self.begin_crop(doc, history);
        }
        // W18 — num VÍDEO o duplo-clique é do tocador (o app activa-o), nunca um texto por cima.
        if doc.get(id).is_some_and(|e| e.embed().is_some()) {
            self.selection = std::collections::BTreeSet::from([id]);
            return false;
        }
        self.begin_text(doc, ts, id, Some(p.world))
    }
}

/// O texto em que se escreve numa seta sem `at`: o 1.º, ou um novo a meio.
fn first_caption(c: &ph2d_board_model::Connector) -> CaptionPick {
    if c.captions.is_empty() {
        CaptionPick::New(Caption::MIDDLE)
    } else {
        CaptionPick::Existing(0)
    }
}

/// O texto de `el` desenha-se na letra à mão? A letra do modo: uma forma ou seta em rascunho, nunca
/// uma nota; ou a que veio fixa de um `.excalidraw` (W9, `Look::letter`).
#[must_use]
pub fn hand_lettered(el: &Element) -> bool {
    match el.style().look.letter {
        ph2d_board_model::Letter::Auto => el.style().sketch && crate::sketchable(el),
        ph2d_board_model::Letter::Hand => true,
        ph2d_board_model::Letter::Sans => false,
    }
}

/// A cara do texto de `el` (a letra e o alinhamento) — a MESMA para quem o desenha e quem o edita.
#[must_use]
pub fn face(el: &Element) -> ph2d_board_layout::Face {
    ph2d_board_layout::Face {
        hand: hand_lettered(el),
        align: el.style().look.align,
    }
}
