//! **A [`FieldKey`] lê-se pela VERSÃO de quem a gravou** (F31 §A, `DOC_VERSION` 23 → 24).
//!
//! O postcard é posicional: bytes v23 (`familia, campo, i, j`) lidos pela forma v24 (com `item`)
//! leem lixo bem-formado. Quem conhece a versão do ficheiro abre um âmbito
//! ([`with_field_key_layout_v23`]) em que toda `FieldKey` desserializada nesta thread usa a forma
//! antiga, com `item` `NONE` — o idioma do `with_stroke_layout_v22` (`ph2d-vec-scene`).
//! A migração `i` → id é semântica e corre depois, com o mundo (`ph2d-timeline-persist`).

use std::cell::Cell;

use serde::{Deserialize, Deserializer};

use crate::ItemId;
use crate::canal::FieldKey;

thread_local! {
    static V23: Cell<u32> = const { Cell::new(0) };
}

/// Corre `f` a ler toda [`FieldKey`] na forma v23. Reentrante; o âmbito fecha mesmo num `panic`.
pub fn with_field_key_layout_v23<T>(f: impl FnOnce() -> T) -> T {
    let _v23 = FieldKeyLayoutV23::enter();
    f()
}

/// O mesmo âmbito como GUARDA: dura até o guarda cair.
#[must_use = "o âmbito fecha quando o guarda cai"]
pub struct FieldKeyLayoutV23(());

impl FieldKeyLayoutV23 {
    pub fn enter() -> Self {
        V23.with(|c| c.set(c.get() + 1));
        Self(())
    }
}

impl Drop for FieldKeyLayoutV23 {
    fn drop(&mut self) {
        V23.with(|c| c.set(c.get() - 1));
    }
}

fn reading_v23() -> bool {
    V23.with(|c| c.get() > 0)
}

/// A forma v23 — ⛔ congelada: mudá-la deixa de ler ficheiros reais.
#[derive(Deserialize)]
#[serde(rename = "FieldKey")]
struct FieldKeyV23 {
    familia: u16,
    campo: u16,
    i: u16,
    j: u16,
}

/// O espelho da forma VIVA — a mesma ordem do `Serialize` derivado (gate
/// `a_forma_viva_da_chave_le_o_que_grava`).
#[derive(Deserialize)]
#[serde(rename = "FieldKey")]
struct FieldKeyV24 {
    familia: u16,
    campo: u16,
    i: u16,
    item: ItemId,
    j: u16,
}

impl<'de> Deserialize<'de> for FieldKey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if reading_v23() {
            let k = FieldKeyV23::deserialize(d)?;
            return Ok(FieldKey::new(k.familia, k.campo, k.i, k.j));
        }
        let k = FieldKeyV24::deserialize(d)?;
        Ok(FieldKey {
            familia: k.familia,
            campo: k.campo,
            i: k.i,
            item: k.item,
            j: k.j,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_forma_viva_da_chave_le_o_que_grava() {
        let k = FieldKey {
            familia: 1,
            campo: 300,
            i: 4,
            item: ItemId(70_000),
            j: 9,
        };
        let b = postcard::to_allocvec(&k).unwrap();
        assert_eq!(postcard::from_bytes::<FieldKey>(&b).unwrap(), k);
        let v23 = postcard::to_allocvec(&(1u16, 300u16, 4u16, 9u16)).unwrap();
        let lida = with_field_key_layout_v23(|| postcard::from_bytes::<FieldKey>(&v23)).unwrap();
        assert_eq!(lida, FieldKey::new(1, 300, 4, 9));
        assert!(!reading_v23(), "o âmbito fecha");
    }
}
