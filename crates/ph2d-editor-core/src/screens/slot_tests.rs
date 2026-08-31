//! Os seis encaixes e o conjunto deles.

use super::*;

/// **Cada encaixe tem o seu bit** — a mutação que sangra é dois a partilharem um.
#[test]
fn the_six_slots_have_six_distinct_bits() {
    let mut seen = 0u8;
    for s in Slot::ALL {
        assert_eq!(
            seen & s.bit(),
            0,
            "{s:?} partilha o bit com um encaixe anterior"
        );
        seen |= s.bit();
    }
    assert_eq!(
        seen.count_ones(),
        6,
        "os seis bits não cobrem seis encaixes"
    );
}

/// **`iter` devolve exactamente o que `contains` diz** — as duas leituras do mesmo conjunto.
#[test]
fn iter_and_contains_agree() {
    for set in [
        SlotSet::NONE,
        SlotSet::LEFT,
        SlotSet::RIGHT,
        SlotSet::SIDES,
        SlotSet::BOTTOM,
        SlotSet::CENTER,
        SlotSet::ANY_DOCK,
    ] {
        let by_iter: Vec<Slot> = set.iter().collect();
        let by_contains: Vec<Slot> = Slot::ALL.into_iter().filter(|s| set.contains(*s)).collect();
        assert_eq!(by_iter, by_contains);
        assert_eq!(set.is_empty(), by_iter.is_empty());
    }
}

/// ⛔ **O `ANY_DOCK` NÃO inclui o centro** — o centro é do editor, não de um painel que se encaixa,
/// e um default que o incluísse punha todo painel por cima da viewport (a foto 2).
#[test]
fn the_default_dock_set_never_includes_the_center() {
    assert!(!SlotSet::ANY_DOCK.contains(Slot::Center));
    assert!(SlotSet::ANY_DOCK.contains(Slot::RightTop));
    assert!(SlotSet::ANY_DOCK.contains(Slot::Bottom));
}

/// **A união é a das duas** — e `SIDES` é literalmente `LEFT ∪ RIGHT`.
#[test]
fn the_union_is_the_union() {
    assert_eq!(SlotSet::LEFT.union(SlotSet::RIGHT), SlotSet::SIDES);
    assert_eq!(SlotSet::NONE.union(SlotSet::BOTTOM), SlotSet::BOTTOM);
}

/// ⭐ **A ida-e-volta do nome de ficheiro, e a unicidade.**
///
/// ⚠️ Uma lei em dois sentidos escrita como duas tabelas divergiria em silêncio: o `from_wire` é
/// DERIVADO do `wire`, e este gate é o que impede alguém de os separar «para ficar mais rápido».
#[test]
fn every_slot_survives_a_round_trip_through_its_wire_name() {
    let mut seen = std::collections::BTreeSet::new();
    for s in Slot::ALL {
        assert_eq!(Slot::from_wire(s.wire()), Some(s), "{s:?} não voltou");
        assert!(
            seen.insert(s.wire()),
            "dois encaixes partilham o nome {:?} — a arrumação gravada leria o errado",
            s.wire()
        );
        assert!(
            s.wire().chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "{s:?} tem um nome de ficheiro que não é snake_case ASCII: {:?}",
            s.wire()
        );
    }
    assert_eq!(Slot::from_wire("um_encaixe_de_2030"), None);
    assert_eq!(Slot::from_wire(""), None);
}
