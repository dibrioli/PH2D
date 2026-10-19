//! W23 B5(1) — o segmento fixo do Excalidraw 0.18.1 de ida e volta, com a seta que ELE gravou depois
//! do arrasto (oráculo `docs/MiroClone/ferramentas/elbow_oracle/excalidraw_segmento_fixo.json`, caso
//! C0: R1 = 0,0 100×100 · R2 = 400,300 100×100, o segmento do meio de x = 250 para x = 310).

use serde_json::{Value, json};

use ph2d_board_model::{BoardSet, Route};
use ph2d_board_route::RouteCache;

const ORACLE: &str =
    include_str!("../../../docs/MiroClone/ferramentas/elbow_oracle/excalidraw_segmento_fixo.json");

fn c0_arrow() -> Value {
    let d: Value = serde_json::from_str(ORACLE).unwrap();
    let c0 = d["casos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["caso"] == "C0_so_arrasto")
        .unwrap();
    let mut a = c0["passos"][1]["arrow_raw"].clone();
    a["type"] = json!("arrow");
    a
}

fn rect(id: &str, x: f64, y: f64) -> Value {
    json!({ "id": id, "type": "rectangle", "x": x, "y": y, "width": 100.0, "height": 100.0,
            "angle": 0.0, "strokeColor": "#1e1e1e", "backgroundColor": "transparent",
            "boundElements": [{ "id": "A", "type": "arrow" }] })
}

/// ⭐ O segmento que ELE fixou em x = 310 lê-se como a nossa fracção e a nossa rota põe-no em 310;
/// escrito de volta, o `fixedSegments` dele diz o mesmo segmento (vertical, em x = 310).
#[test]
fn the_excalidraw_fixed_middle_comes_in_as_our_fraction_and_goes_back() {
    let file = json!({ "type": "excalidraw", "version": 2,
        "elements": [rect("R1", 0.0, 0.0), rect("R2", 400.0, 300.0), c0_arrow()] });
    let imp = crate::read(&file.to_string()).expect("lê-se");
    let (id, c) = imp
        .doc
        .live()
        .find_map(|e| Some((e.id, e.connector()?.clone())))
        .expect("a seta");
    assert_eq!(c.route, Route::Elbow);
    assert!(c.mid(0).is_some(), "o segmento fixo dele não se perde");
    let mut routes = RouteCache::default();
    routes.sync(&imp.doc);
    let e = routes.get(id).unwrap().elbows[0].unwrap();
    assert!((e.at[0] - 310.0).abs() < 0.5, "x = {}", e.at[0]);

    let mut set = BoardSet::default();
    let b = set.create("C0".into());
    set.get_mut(b).unwrap().doc = imp.doc;
    let out: Value = serde_json::from_str(&crate::write(set.get(b).unwrap())).unwrap();
    let arrow = out["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["type"] == "arrow")
        .unwrap();
    let seg = &arrow["fixedSegments"][0];
    let x = arrow["x"].as_f64().unwrap();
    let (s, t) = (
        seg["start"].as_array().unwrap(),
        seg["end"].as_array().unwrap(),
    );
    assert_eq!(s[0], t[0], "vertical: {seg}");
    assert!((x + s[0].as_f64().unwrap() - 310.0).abs() < 0.5, "{arrow}");
}

/// Uma seta cotovelo sem segmento fixo não escreve `fixedSegments` (o `null` dele).
#[test]
fn an_unpinned_elbow_writes_no_fixed_segments() {
    let mut a = c0_arrow();
    a["fixedSegments"] = Value::Null;
    let file = json!({ "type": "excalidraw", "version": 2,
        "elements": [rect("R1", 0.0, 0.0), rect("R2", 400.0, 300.0), a] });
    let imp = crate::read(&file.to_string()).unwrap();
    assert!(
        imp.doc
            .live()
            .all(|e| e.connector().is_none_or(|c| c.mids.is_empty()))
    );
    let mut set = BoardSet::default();
    let b = set.create("C".into());
    set.get_mut(b).unwrap().doc = imp.doc;
    let out: Value = serde_json::from_str(&crate::write(set.get(b).unwrap())).unwrap();
    let arrow = out["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["type"] == "arrow")
        .unwrap();
    assert!(arrow["fixedSegments"].is_null());
}
