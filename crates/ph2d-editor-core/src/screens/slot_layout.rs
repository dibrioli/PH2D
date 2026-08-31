//! ⭐⭐ **A GEOMETRIA dos seis encaixes** — o que faltava para a [`crate::screens::slot::Slot`]
//! deixar de ser um nome e passar a ser um sítio.
//!
//! # A lei: a metade de uma coluna só existe quando a IRMÃ está ocupada
//!
//! Uma coluna vazia não tem metades — dividi-la daria duas faixas de 498 px de que ninguém
//! precisa, e um painel sozinho ficaria com metade da altura por uma razão que não existe. ⇒
//! [`HeroLayout::slot_rects`] recebe **quem está ocupado** e resolve:
//!
//! | `LeftTop` ocupado | `LeftBottom` ocupado | `LeftTop` fica com | `LeftBottom` fica com |
//! |---|---|---|---|
//! | sim | não | a coluna **inteira** | a coluna inteira (ninguém lá) |
//! | sim | sim | a **metade de cima** | a metade de baixo |
//!
//! ⚠️ **O caso «nenhum dos dois» devolve a coluna inteira às duas metades, e isso é inofensivo por
//! construção**: um rect só se vê quando alguém o ocupa. Devolver `None` obrigaria todo leitor a
//! um `unwrap` sobre uma pergunta que ele não tem como responder melhor.
//!
//! # ⛔ Por que isto NÃO deriva de `layout.hierarchy` e `layout.inspector` pelo nome
//!
//! Sob `ui_mirrored` os dois **trocam de lado**: `layout.hierarchy` passa a ser a coluna da
//! direita. Ler o nome daria os encaixes espelhados em silêncio — a esquerda a apontar para a
//! direita. A fonte é [`HeroLayout::side_columns`], que devolve as duas **ordenadas por `x`**.

use crate::screens::layout::HeroLayout;
use crate::screens::slot::{Slot, SlotSet};
use crate::zones::Rect;

/// Os seis rects de um quadro, indexados por [`Slot`].
#[derive(Copy, Clone, Debug)]
pub struct SlotRects([Rect; 6]);

impl SlotRects {
    /// Onde este encaixe está neste quadro.
    #[must_use]
    pub fn get(&self, slot: Slot) -> Rect {
        self.0[slot as usize]
    }
}

/// Divide uma banda ao meio na vertical.
fn halves(band: Rect) -> (Rect, Rect) {
    let h = band.h * 0.5;
    (
        Rect::new(band.x, band.y, band.w, h),
        Rect::new(band.x, band.y + h, band.w, band.h - h),
    )
}

/// Resolve as duas metades de uma coluna a partir de quem a ocupa.
fn column(band: Rect, top_taken: bool, bottom_taken: bool) -> (Rect, Rect) {
    if top_taken && bottom_taken {
        halves(band)
    } else {
        (band, band)
    }
}

impl HeroLayout {
    /// ⭐ **Onde cada um dos seis encaixes está neste quadro**, dado quem está ocupado.
    ///
    /// Ver o cabeçalho do módulo para a lei das metades.
    #[must_use]
    pub fn slot_rects(&self, occupied: SlotSet) -> SlotRects {
        let (left_band, right_band) = self.side_columns();
        let (lt, lb) = column(
            left_band,
            occupied.contains(Slot::LeftTop),
            occupied.contains(Slot::LeftBottom),
        );
        let (rt, rb) = column(
            right_band,
            occupied.contains(Slot::RightTop),
            occupied.contains(Slot::RightBottom),
        );
        SlotRects([
            lt,
            lb,
            rt,
            rb,
            // ⚠️ **A faixa de baixo é a do TIMELINE, e não uma banda nova.** Ela já é a geometria
            // do encaixe inferior — entre as duas colunas, ancorada no fundo — e o `flip_strip` é
            // a mesma faixa mais baixa. Inventar uma terceira poria dois donos na mesma fila.
            self.timeline,
            self.draw_area,
        ])
    }
}

/// ⭐⭐ **A FAIXA DE ABAS sai do encaixe, e o que estava debaixo dela DESCE** (spec §2, regra 1).
///
/// Recebe quantos painéis ocupam cada encaixe e a altura de uma aba, e escreve
/// [`HeroLayout::slot_tabs`] — o rect da faixa de cada encaixe, **zero** onde `n < 2`
/// (*uma aba sozinha é um título a mais, não uma escolha*).
///
/// # ⛔ Por que ele não recebe a lista de campos a encolher
///
/// O [`HeroLayout`] tem **cinco** campos que são o mesmo rect da coluna da direita (`inspector`,
/// `bgremoval`, `padding`, `painter_sidebar`, `painter_layers`) — e sob `ui_mirrored` essa coluna
/// muda de lado. Uma lista escrita à mão ficaria certa hoje e silenciosamente incompleta no dia em
/// que a sexta aparecesse: os painéis dela desenhariam **por baixo** das abas, e nenhum gate o
/// veria (o rect publicado continua dentro da coluna).
///
/// ⇒ a regra é **derivada**: *todo rect docado que TOCA a faixa começa onde ela acaba.* Um que não
/// a toque — a tira do Flip, que é baixa e vive no fundo da faixa inferior — fica intocado, e isso
/// está certo: ela nunca esteve debaixo das abas.
impl HeroLayout {
    /// Ver o doc acima.
    pub fn reserve_slot_tabs(&mut self, counts: [usize; 6], bar_h: f32) {
        let mut occupied = SlotSet::NONE;
        for (i, slot) in Slot::ALL.into_iter().enumerate() {
            if counts[i] > 0 {
                occupied = occupied.union(SlotSet::of(slot));
            }
        }
        let rects = self.slot_rects(occupied);
        let mut bars = [Rect::new(0.0, 0.0, 0.0, 0.0); 6];
        for (i, slot) in Slot::ALL.into_iter().enumerate() {
            // ⛔ O centro nunca tem abas — ele é do editor (spec §2, regra 4).
            if counts[i] < 2 || slot == Slot::Center {
                continue;
            }
            let band = rects.get(slot);
            if band.h <= bar_h || band.w <= 0.0 {
                continue;
            }
            let bar = Rect::new(band.x, band.y, band.w, bar_h);
            bars[i] = bar;
            let band_bottom = band.y + band.h;
            for r in self.docked_rects_mut() {
                if overlaps(*r, bar) {
                    let top = bar.y + bar.h;
                    *r = Rect::new(r.x, top, r.w, (band_bottom - top).max(0.0));
                }
            }
        }
        self.slot_tabs = bars;
    }

    /// Os rects que um encaixe pode conter — a população que [`Self::reserve_slot_tabs`] empurra.
    ///
    /// ⚠️ **Um campo novo de painel docado tem de entrar aqui**, e o gate
    /// `every_docked_layout_rect_is_pushed_by_a_tab_bar` reprova se ficar de fora: ele compara esta
    /// lista com os rects que os painéis de facto publicam.
    fn docked_rects_mut(&mut self) -> [&mut Rect; 8] {
        [
            &mut self.inspector,
            &mut self.bgremoval,
            &mut self.padding,
            &mut self.painter_sidebar,
            &mut self.painter_layers,
            &mut self.hierarchy,
            &mut self.timeline,
            &mut self.flip_strip,
        ]
    }
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

#[cfg(test)]
#[path = "slot_layout_tests.rs"]
mod tests;
