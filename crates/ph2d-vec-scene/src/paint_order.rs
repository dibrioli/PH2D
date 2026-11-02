//! ⭐⭐ **A ORDEM da pilha inteira** (F45, dono 01/11: *«a fila de sobreposição funcione na ordem em
//! que são colocadas. Assim fill pode sobrepor stroke e vice-versa»*).
//!
//! A tinta de base fica nos campos `fill`/`stroke` — é lá que centenas de leitores perguntam «de que
//! cor é esta forma?» (medido 01/11: `.fill` em 641 ficheiros, a ordem fixa em só três) — e ganha uma
//! POSIÇÃO na pilha ([`BaseOrder`], v27). A ordem de desenho é a lista intercalada de
//! [`VecPath::draw_order`], de baixo para cima; a porta [`VecPath::paint_stack`] e o renderer seguem-na.
//!
//! ⚠️ O neutro (`fill` 0, `stroke` 1) é a ordem de antes: preenchimento de base, traço de base, camadas.
//! Uma posição fora da pilha (uma lista trocada inteira por outro sítio) prende-se ao topo, nunca panica.

use crate::VecPath;

/// Onde a tinta de base desenha: a posição de cada uma na lista intercalada (`0` = a mais baixa).
///
/// ⚠️ **Newtype pelo `Default`**: um `[u8; 2]` derivado nasceria `[0, 0]` — as duas no mesmo sítio.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BaseOrder {
    pub fill: u8,
    pub stroke: u8,
}

impl Default for BaseOrder {
    fn default() -> Self {
        Self { fill: 0, stroke: 1 }
    }
}

/// Uma linha da pilha inteira: a tinta de base ou a camada `i` de `paints`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackItem {
    BaseFill,
    BaseStroke,
    Entry(usize),
}

/// Onde uma linha movida cai, em relação a outra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Logo POR CIMA de (desenha depois dela).
    Above(StackItem),
    /// Logo POR BAIXO de.
    Below(StackItem),
}

impl VecPath {
    /// As posições da base, já presas à pilha de hoje e distintas.
    fn base_positions(&self) -> (usize, usize) {
        let top = self.paints.len() + 1;
        let f = usize::from(self.base_order.fill).min(top);
        let mut s = usize::from(self.base_order.stroke).min(top);
        if s == f {
            s = if f == top { f - 1 } else { f + 1 };
        }
        (f, s)
    }

    /// ⭐⭐ **A pilha inteira, de baixo para cima** — as duas tintas de base (mesmo ausentes) e cada
    /// camada, pela ordem em que desenham. Não aloca: é o que o renderer pergunta por forma.
    pub fn draw_order(&self) -> impl Iterator<Item = StackItem> + use<> {
        let (f, s) = self.base_positions();
        let mut next = 0;
        (0..self.paints.len() + 2).map(move |k| {
            if k == f {
                StackItem::BaseFill
            } else if k == s {
                StackItem::BaseStroke
            } else {
                next += 1;
                StackItem::Entry(next - 1)
            }
        })
    }

    /// A linha existe nesta forma? (A base sem tinta, nem à vista nem guardada, não tem linha.)
    #[must_use]
    pub fn has_item(&self, it: StackItem) -> bool {
        match it {
            StackItem::BaseFill => self.base_fill().is_some(),
            StackItem::BaseStroke => self.base_stroke().is_some(),
            StackItem::Entry(i) => i < self.paints.len(),
        }
    }

    /// **Reescreve a ordem** a partir de uma permutação de [`Self::draw_order`]: as camadas mudam de
    /// índice e a base de posição. Devolve se mudou; uma lista que não é permutação é recusada.
    pub fn set_draw_order(&mut self, order: &[StackItem]) -> bool {
        let n = self.paints.len();
        let mut vistos = vec![false; n + 2];
        for it in order {
            let k = match *it {
                StackItem::BaseFill => n,
                StackItem::BaseStroke => n + 1,
                StackItem::Entry(i) if i < n => i,
                StackItem::Entry(_) => return false,
            };
            if std::mem::replace(&mut vistos[k], true) {
                return false;
            }
        }
        if order.len() != n + 2 || self.draw_order().eq(order.iter().copied()) {
            return false;
        }
        let mut velhas: Vec<_> = std::mem::take(&mut self.paints)
            .into_iter()
            .map(Some)
            .collect();
        for it in order {
            if let StackItem::Entry(i) = *it
                && let Some(e) = velhas[i].take()
            {
                self.paints.push(e);
            }
        }
        self.base_order = base_of(order);
        true
    }

    /// ⭐ **Move a linha `item` para logo acima/abaixo de outra** — o gesto de arrastar pelo ⚙.
    pub fn move_paint(&mut self, item: StackItem, place: Place) -> bool {
        let (alvo, acima) = match place {
            Place::Above(a) => (a, true),
            Place::Below(a) => (a, false),
        };
        let mut order: Vec<_> = self.draw_order().collect();
        let Some(de) = order.iter().position(|x| *x == item) else {
            return false;
        };
        if alvo == item {
            return false;
        }
        order.remove(de);
        let Some(k) = order.iter().position(|x| *x == alvo) else {
            return false;
        };
        order.insert(if acima { k + 1 } else { k }, item);
        self.set_draw_order(&order)
    }

    /// **Apaga a camada `i`**, e a base fica entre as mesmas vizinhas.
    pub fn remove_paint(&mut self, i: usize) -> bool {
        if i >= self.paints.len() {
            return false;
        }
        let order: Vec<_> = self
            .draw_order()
            .filter(|x| *x != StackItem::Entry(i))
            .collect();
        self.paints.remove(i);
        self.base_order = base_of(&order);
        true
    }
}

/// As posições da base numa ordem completa.
fn base_of(order: &[StackItem]) -> BaseOrder {
    let at = |it| {
        order
            .iter()
            .position(|x| *x == it)
            .and_then(|k| u8::try_from(k).ok())
            .unwrap_or(u8::MAX)
    };
    BaseOrder {
        fill: at(StackItem::BaseFill),
        stroke: at(StackItem::BaseStroke),
    }
}

#[cfg(test)]
#[path = "paint_order_tests.rs"]
mod tests;
