//! Fuzz curto e determinístico: entrada defeituosa nunca entra em pânico, e o que importa volta a
//! dar o mesmo Kanban pela ida-e-volta (a propriedade do gate, sobre quadros estragados).

use ph2d_kanban_import::{export_csv, export_markdown, export_trello_json, import_trello};
use ph2d_kanban_model::KanbanId;
use serde_json::{Value, json};

use crate::fixture_bytes;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn exercise(bytes: &[u8]) -> bool {
    let Ok(imp) = import_trello(bytes, KanbanId(3), None) else {
        return false;
    };
    let _ = (export_csv(&imp.kanban), export_markdown(&imp.kanban));
    let json = export_trello_json(&imp.kanban);
    let again = import_trello(json.as_bytes(), KanbanId(3), None).expect("a exportação reimporta");
    assert_eq!(
        imp.kanban, again.kanban,
        "ida-e-volta sobre entrada estragada"
    );
    true
}

#[test]
fn fuzz_byte_mutations_never_panic() {
    let base = fixture_bytes();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..300 {
        let mut b = base.clone();
        match rng.below(3) {
            0 => b.truncate(rng.below(b.len())),
            1 => {
                for _ in 0..1 + rng.below(8) {
                    let i = rng.below(b.len());
                    b[i] = rng.next() as u8;
                }
            }
            _ => {
                let i = rng.below(b.len());
                b.insert(i, b"{}[]\",:0-e"[rng.below(10)]);
            }
        }
        exercise(&b);
    }
}

fn junk(rng: &mut Rng) -> Value {
    match rng.below(10) {
        0 => Value::Null,
        1 => json!(rng.next().is_multiple_of(2)),
        2 => json!(-1.5e300),
        3 => json!(i64::MIN),
        4 => json!("2026-13-45T99:99:99Z"),
        5 => json!("top"),
        6 => json!([]),
        7 => json!({}),
        8 => json!(""),
        _ => json!("5f0000000000000000000001"),
    }
}

/// Troca um nó ao acaso (a `n`-ésima folha ou objecto numa travessia) por lixo.
fn mutate(v: &mut Value, rng: &mut Rng, budget: &mut usize) {
    if *budget == 0 {
        *v = junk(rng);
        return;
    }
    *budget -= 1;
    match v {
        Value::Array(a) if !a.is_empty() => {
            let i = rng.below(a.len());
            if rng.below(8) == 0 {
                let dup = a[i].clone();
                a.push(dup);
            } else {
                mutate(&mut a[i], rng, budget);
            }
        }
        Value::Object(m) if !m.is_empty() => {
            let keys: Vec<String> = m.keys().cloned().collect();
            let k = &keys[rng.below(keys.len())];
            if rng.below(8) == 0 {
                m.remove(k);
            } else if let Some(x) = m.get_mut(k) {
                mutate(x, rng, budget);
            }
        }
        _ => *v = junk(rng),
    }
}

#[test]
fn fuzz_structural_mutations_never_panic_and_round_trip() {
    let base: Value = serde_json::from_slice(&fixture_bytes()).expect("json");
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let mut imported = 0;
    for _ in 0..400 {
        let mut v = base.clone();
        for _ in 0..1 + rng.below(4) {
            let mut budget = 1 + rng.below(6);
            mutate(&mut v, &mut rng, &mut budget);
        }
        imported += usize::from(exercise(serde_json::to_string(&v).expect("s").as_bytes()));
    }
    assert!(
        imported > 300,
        "a maior parte das mutações ainda é um quadro: {imported}"
    );
    for bad in [
        &b""[..],
        b"null",
        b"[]",
        b"{}",
        b"\"x\"",
        b"{\"lists\":7}",
        b"{\"cards\":[1,2]}",
    ] {
        let _ = import_trello(bad, KanbanId(1), None);
    }
    let deep = format!("{}{}", "[".repeat(5000), "]".repeat(5000));
    assert!(import_trello(deep.as_bytes(), KanbanId(1), None).is_err());
}
