//! ⭐ **Os componentes que um objecto TEM, num conjunto de bits** — o que o filtro da Hierarquia
//! pergunta («só o que tem Health», doc ObjectsAndComponents 01 §7.5 e §13, F6).
//!
//! O bit `i` é o `i`-ésimo componente AUTORADO de [`crate::all`] (os que o artista anexa — os únicos
//! que fazem sentido num filtro), e o bit [`ComponentSet::ANIMATION`] é a Animação, que não é tipo do
//! mundo: é derivada dos clips de que o objecto é dono (onda 2). Um `u128` porque a linha da Hierarquia
//! o carrega por quadro e um `Vec` por linha seria uma alocação por objecto; há gate a provar que os
//! autorados cabem (`the_authored_components_fit_the_set`).

use crate::{Attach, ComponentDesc};

/// Um conjunto de componentes autorados (+ a Animação).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ComponentSet(u128);

impl ComponentSet {
    /// O bit da Animação — o último, longe de qualquer autorado.
    pub const ANIMATION: u8 = 127;

    /// Os componentes autorados com o bit de cada um, na ordem do catálogo.
    pub fn authored() -> impl Iterator<Item = (u8, &'static ComponentDesc)> {
        crate::all()
            .filter(|d| matches!(d.attach, Attach::Authored { .. }))
            .enumerate()
            .map(|(i, d)| (u8::try_from(i).unwrap_or(u8::MAX), d))
    }

    /// O descritor do bit `bit`, se é de um componente autorado (a Animação não tem).
    #[must_use]
    pub fn desc_of(bit: u8) -> Option<&'static ComponentDesc> {
        Self::authored().find(|(b, _)| *b == bit).map(|(_, d)| d)
    }

    /// Põe o bit.
    pub fn insert(&mut self, bit: u8) {
        if bit < 128 {
            self.0 |= 1u128 << bit;
        }
    }

    /// Tem o bit?
    #[must_use]
    pub const fn contains(self, bit: u8) -> bool {
        bit < 128 && self.0 & (1u128 << bit) != 0
    }

    /// Vazio?
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Os bits postos, por ordem crescente.
    pub fn iter(self) -> impl Iterator<Item = u8> {
        (0u8..128).filter(move |b| self.contains(*b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_authored_components_fit_the_set() {
        let n = ComponentSet::authored().count();
        assert!(n >= 50, "o catálogo encolheu? {n}");
        assert!(
            n < usize::from(ComponentSet::ANIMATION),
            "{n} autorados não cabem abaixo do bit da Animação"
        );
    }

    #[test]
    fn a_bit_round_trips_to_its_component() {
        for (bit, d) in ComponentSet::authored() {
            let mut s = ComponentSet::default();
            s.insert(bit);
            assert!(s.contains(bit));
            assert_eq!(s.iter().collect::<Vec<_>>(), vec![bit]);
            assert_eq!(
                ComponentSet::desc_of(bit).map(|x| x.canonical_name),
                Some(d.canonical_name)
            );
        }
        assert!(ComponentSet::desc_of(ComponentSet::ANIMATION).is_none());
    }
}
