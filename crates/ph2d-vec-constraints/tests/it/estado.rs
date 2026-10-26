//! Cores do CAD contra o FreeCAD (W4.1 §1.4): uma medida a mais coerente é REDUNDANTE (o editor fá-la nascer REF);
//! uma incoerente lista as regras em conflito — a MESMA lista que o Sketcher dá.

use ph2d_vec_constraints::{EstadoRegra, Liberdade, estado};
use serde_json::Value;

use crate::cena::{Cena, cena};

const FIXTURE: &str =
    include_str!("../../../../docs/BlenderConstraints/fixtures/freecad_1_1_4_sketch_arrasto.json");

fn lista(v: &Value) -> Vec<usize> {
    v.as_array()
        .expect("lista")
        .iter()
        .map(|i| usize::try_from(i.as_u64().expect("índice")).expect("índice"))
        .collect()
}

/// Só as regras da cena: os fixos e os raios que a tradução junta no fim não são regras do Sketcher.
fn com(e: &[EstadoRegra], qual: EstadoRegra, c: &Cena) -> Vec<usize> {
    let n = c.json["regras"].as_array().expect("regras").len();
    e[..n]
        .iter()
        .enumerate()
        .filter(|(_, s)| **s == qual)
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn an_extra_dimension_is_born_ref() {
    let o: Value = serde_json::from_str(FIXTURE).expect("JSON");
    let c = cena("conflito_redundante");
    let e = estado(&c.folha(), &c.regras);
    let esperado = lista(&o["bem"]["conflito_redundante"]["redundantes_regras"]);
    assert_eq!(esperado, vec![6], "a fixture diz a regra 6");
    assert_eq!(com(&e.regras, EstadoRegra::Redundante, &c), esperado);
    assert!(com(&e.regras, EstadoRegra::Conflito, &c).is_empty());

    let mut ref6 = c.regras.clone();
    ref6[6].referencia = true;
    let e = estado(&c.folha(), &ref6);
    assert_eq!(
        com(&e.regras, EstadoRegra::Ref, &c),
        vec![6],
        "marcada REF, deixa de ser redundante"
    );
    assert!(com(&e.regras, EstadoRegra::Redundante, &c).is_empty());
    assert!(
        e.pontos.values().all(|l| *l == Liberdade::Definido),
        "o retângulo cotado fica todo definido"
    );
}

#[test]
fn a_conflict_lists_the_rules_involved() {
    let o: Value = serde_json::from_str(FIXTURE).expect("JSON");
    for (nome, esperado) in [
        ("conflito_inconsistente", vec![1, 3, 4, 6]),
        ("conflito_hv", vec![1, 2]),
    ] {
        assert_eq!(
            lista(&o["bem"][nome]["conflitos_regras"]),
            esperado,
            "{nome}: a fixture"
        );
        let c = cena(nome);
        let e = estado(&c.folha(), &c.regras);
        assert_eq!(
            com(&e.regras, EstadoRegra::Conflito, &c),
            esperado,
            "{nome}"
        );
    }
}

#[test]
fn an_underconstrained_point_is_free() {
    let c = cena("escada_50");
    let e = estado(&c.folha(), &c.regras);
    assert!(
        e.regras.iter().all(|s| *s == EstadoRegra::Ok),
        "a escada cumpre e nada sobra"
    );
    assert!(
        e.pontos.values().any(|l| *l == Liberdade::Livre),
        "a escada é sub-restringida"
    );
}
