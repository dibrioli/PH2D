//! W23 B5(2) — **os textos ao longo da linha de uma seta** ([doc 14]), a lei do Miro: carregar num
//! texto selecciona a seta e agarra-o, e ele anda AO LONGO da linha (assenta no ponto dela mais perto
//! do ponteiro), UM passo de desfazer; o duplo clique na linha cria um texto ali, e num texto escreve
//! nele (`text.rs`); a seta deita-os ao longo dela ou na horizontal ([`Editor::set_text_turn`]).
//!
//! [doc 14]: ../../../docs/MiroClone/14_texto_ao_longo_da_linha.md

use std::collections::BTreeMap;

use ph2d_board_layout::TextCache;
use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History, TextTurn};
use ph2d_board_route::{CAPTION_PAD, LABEL_WRAP};
use ph2d_text::TextSystem;

use crate::gesture::Gesture;
use crate::{Editor, Pointer, live};

/// O bloco de cada texto das setas (a altura e o recorte por baixo das letras), moldado quando há
/// moldador — o clique agarra o que se vê.
#[derive(Default)]
pub(crate) struct CaptionBoxes {
    text: TextCache,
    boxes: BTreeMap<ElementId, Vec<(f64, [f64; 4])>>,
}

impl Editor {
    /// Molda os textos das setas (só o que mudou volta a moldar-se).
    pub(crate) fn measure_captions(&mut self, doc: &BoardDoc, ts: &mut TextSystem) {
        let b = &mut self.caption_boxes;
        b.boxes.clear();
        for el in doc.live() {
            let Some(c) = el.connector().filter(|c| !c.captions.is_empty()) else {
                continue;
            };
            let (size, face) = (c.style.font_size, crate::face(el));
            let hs = c
                .captions
                .iter()
                .enumerate()
                .map(|(i, cap)| {
                    let key = (i as u64, el.id.0);
                    let l = b.text.get(
                        ts,
                        key,
                        &cap.text,
                        &[],
                        size as f32,
                        LABEL_WRAP as f32,
                        face,
                    );
                    let knock = ph2d_board_layout::knock_box(l, size * CAPTION_PAD);
                    (f64::from(l.height()), knock)
                })
                .collect();
            b.boxes.insert(el.id, hs);
        }
        b.text.end_frame();
    }

    /// A altura do bloco do texto `i` de `el` e o recorte nele — antes do 1.º moldado, nenhum.
    fn caption_box(&self, el: &Element, i: usize) -> Option<(f64, [f64; 4])> {
        self.caption_boxes.boxes.get(&el.id)?.get(i).copied()
    }

    /// O texto de `el` (uma seta) sob `world`, o de cima primeiro.
    pub(crate) fn caption_of(&self, el: &Element, world: [f64; 2]) -> Option<usize> {
        let c = el.connector()?;
        let r = self.routes.get(el.id)?;
        (0..c.captions.len()).rev().find(|i| {
            self.caption_box(el, *i).is_some_and(|(h, knock)| {
                r.caption_frame(c.captions[*i].at, c.turn)
                    .hits(world, h, knock)
            })
        })
    }

    /// Carregar num texto de uma seta: selecciona-a e agarra o texto (uma seta travada só se
    /// selecciona). `false` = não havia texto ali.
    pub(crate) fn caption_down(&mut self, doc: &BoardDoc, ts: &mut TextSystem, p: Pointer) -> bool {
        self.measure_captions(doc, ts);
        let Some(el) = self.hit(doc, p).and_then(|id| doc.get(id)) else {
            return false;
        };
        let Some(index) = self.caption_of(el, p.world) else {
            return false;
        };
        if !self.selection.contains(&el.id) {
            self.selection = self.pick(doc, el.id);
        }
        let (Some(c), Some(r)) = (el.connector(), self.routes.get(el.id)) else {
            return false;
        };
        if el.locked() {
            return true;
        }
        let center = r.caption_frame(c.captions[index].at, c.turn).center;
        self.gesture = Some(Gesture::Caption {
            id: el.id,
            index,
            grab: [p.world[0] - center[0], p.world[1] - center[1]],
            start: p.world,
            moved: false,
            original: el.clone(),
        });
        true
    }

    /// Arrastar o texto: o ponto da linha mais perto do ponteiro (menos onde se agarrou).
    pub(crate) fn caption_move(&mut self, doc: &mut BoardDoc, g: &mut Gesture, p: Pointer) {
        let Gesture::Caption {
            id,
            index,
            grab,
            start,
            moved,
            original,
        } = g
        else {
            return;
        };
        let d = [p.world[0] - start[0], p.world[1] - start[1]];
        *moved |= d[0].hypot(d[1]) > self.metrics.drag * p.px;
        if !*moved {
            return;
        }
        self.routes.sync(doc);
        let Some(r) = self.routes.get(*id) else {
            return;
        };
        let at = r.frac_of([p.world[0] - grab[0], p.world[1] - grab[1]]);
        let mut el = original.clone();
        if let Some(cap) = el.connector_mut().and_then(|c| c.captions.get_mut(*index)) {
            cap.at = at;
            live(doc, el);
        }
    }

    /// Largar o texto: UM passo de desfazer; um toque sem arrastar só seleccionou.
    pub(crate) fn caption_up(&mut self, doc: &mut BoardDoc, history: &mut History, g: Gesture) {
        let Gesture::Caption {
            id,
            moved,
            original,
            ..
        } = g
        else {
            return;
        };
        if moved && doc.get(id).is_some_and(|now| now.kind != original.kind) {
            history.record(vec![BoardOp::Put(original)], doc);
        }
    }

    /// Como se deitam os textos das setas seleccionadas — `None` sem seta, ou se não é o mesmo em
    /// todas.
    #[must_use]
    pub fn text_turn(&self, doc: &BoardDoc) -> Option<TextTurn> {
        let mut turns = self
            .selected(doc)
            .into_iter()
            .filter_map(|el| el.connector().map(|c| c.turn));
        let first = turns.next()?;
        turns.all(|t| t == first).then_some(first)
    }

    /// Deita os textos das setas seleccionadas ao longo da linha ou na horizontal (UM passo).
    /// `false` = não há seta seleccionada.
    pub fn set_text_turn(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        turn: TextTurn,
    ) -> bool {
        if !self.selected(doc).iter().any(|el| el.connector().is_some()) {
            return false;
        }
        self.change_connectors(doc, history, |c| c.turn = turn);
        true
    }
}
