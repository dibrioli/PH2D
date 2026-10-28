//! W4 do Kanban — um Kanban no `.excalidraw`: o `rectangle` de cantos redondos com o NOME preso dentro
//! (o que o Excalidraw mostra), e o nosso elemento no `customData` (volta a ser o Kanban).

use ph2d_board_model::{
    BoardKanban, BoardOp, BoardSet, Element, KanbanId, Rgba, Style,
};
use serde_json::Value;

use crate::{read, write, write_with_names};

fn board() -> ph2d_board_model::Board {
    let mut set = BoardSet::default();
    let b = set.create("K".into());
    let doc = &mut set.get_mut(b).unwrap().doc;
    let k = BoardKanban::new(KanbanId(3), Style::new(None, None, Rgba([0, 0, 0, 255])));
    let el = Element::new_kanban(doc.mint_id(), doc.z_on_top(), k, [10.0, 20.0, 640.0, 360.0]);
    BoardOp::Put(el).apply(doc);
    set.boards()[0].clone()
}

#[test]
fn a_kanban_exports_as_a_named_rounded_rectangle_and_comes_back() {
    let b = board();
    let names = |k: KanbanId| (k == KanbanId(3)).then(|| "Sprint 12".to_owned());
    let text = write_with_names(&b, &names);
    let v: Value = serde_json::from_str(&text).unwrap();
    let els = v["elements"].as_array().unwrap();
    let rect = els.iter().find(|e| e["type"] == "rectangle").expect("o rectângulo");
    assert!(!rect["roundness"].is_null());
    assert_eq!(rect["width"], 640.0);
    let label = els.iter().find(|e| e["type"] == "text").expect("o nome");
    assert_eq!(label["containerId"], rect["id"]);
    assert_eq!(label["text"], "Sprint 12");
    let back = read(&text).unwrap();
    assert!(back.skipped.is_empty(), "{:?}", back.skipped);
    let k: Vec<_> = back.doc.live().filter_map(|e| e.kanban()).collect();
    assert_eq!(k.len(), 1);
    assert_eq!(k[0].kanban, KanbanId(3));
    // Sem nomes, o rectângulo sai sem texto (nunca um nome inventado).
    assert!(!write(&b).contains("\"type\":\"text\""));
}
