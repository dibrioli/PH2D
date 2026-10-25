//! ⭐ **A identidade na chave** (v24, F31 §A): a v23 lê-se pelo âmbito da chave antiga — toda chave
//! sai com `item` `NONE` e o `i`/`j` de antes. A troca `i` → id é do `ph2d-timeline-persist`
//! (`o_doc_v23_le_e_migra_para_o_id`), que tem o mundo.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_ecs::{FieldKey, ItemId};
use ph2d_timeline::{DOC_VERSION, PropKind, TimelineDoc};

/// O JSON sem o `item` de cada chave (a régua das fixturas v21–v23, gravadas sem ele).
pub(crate) fn sem_item(j: &mut serde_json::Value) {
    match j {
        serde_json::Value::Object(o) => {
            if o.contains_key("familia") {
                o.remove("item");
            }
            o.values_mut().for_each(sem_item);
        }
        serde_json::Value::Array(a) => a.iter_mut().for_each(sem_item),
        _ => {}
    }
}

#[test]
fn o_doc_v23_le_com_a_chave_antiga() {
    const V23_BIN: &[u8] = include_bytes!("../fixtures/timeline_doc_v23.bin");
    const V23_JSON: &str = include_str!("../fixtures/timeline_doc_v23.json");
    let doc = TimelineDoc::from_bytes(V23_BIN).expect("o v23 tem de abrir");
    assert_eq!(doc.version, DOC_VERSION);
    let chaves: Vec<FieldKey> = doc
        .bindings()
        .iter()
        .filter_map(|b| match b.prop {
            PropKind::Field(k) => Some(k),
            _ => None,
        })
        .collect();
    assert_eq!(chaves.len(), 7);
    assert!(chaves.iter().all(|k| k.item == ItemId::NONE));
    let vermelho = v::TINTA_PREENCH + v::OFF_SOLIDA;
    for esperada in [
        FieldKey::new(VETOR, vermelho, 1, 0),
        FieldKey::new(VETOR, v::PILHA_OPACIDADE, 2, 0),
        FieldKey::new(VETOR, v::EFEITO_PARAM, 1, 2),
        FieldKey::new(VETOR, v::FILTRO_RAIO, 0, 0),
        FieldKey::new(VETOR, v::ANCORA_X, 1, 3),
    ] {
        assert!(chaves.contains(&esperada), "{esperada:?}");
    }
    let sem = |mut j: serde_json::Value| {
        j.as_object_mut().unwrap().remove("version");
        sem_item(&mut j);
        j
    };
    let antes: serde_json::Value = serde_json::from_str(V23_JSON).unwrap();
    assert_eq!(sem(serde_json::to_value(&doc).unwrap()), sem(antes));
    // E a v24 vai e volta com o item.
    let mut doc = doc;
    doc.bindings_mut()[0].prop = PropKind::Field(FieldKey::do_item(VETOR, vermelho, ItemId(77), 0));
    let lido = TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(lido, doc);
}
