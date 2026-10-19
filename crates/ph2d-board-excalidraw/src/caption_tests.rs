//! W23 B5(2) — os textos ao longo da linha de uma seta no `.excalidraw` (doc 14): o 1.º preso com a
//! fracção dele, os outros soltos com dono e índice; ida e volta; e o rótulo de uma seta DELE.

use super::*;
use ph2d_board_model::{
    Anchor, BoardOp, BoardSet, Caption, Connector, Element, End, Rgba, Route, Shape, ShapeType,
    Style, TextTurn,
};
use serde_json::{Value, json};

const INK: Rgba = Rgba([30, 30, 30, 255]);

/// Duas caixas e uma seta recta entre elas com dois textos: «sim» a 0,3 e «não» a 0,8, ao longo.
fn board() -> ph2d_board_model::Board {
    let mut set = BoardSet::default();
    let id = set.create("Textos".into());
    let b = set.get_mut(id).unwrap();
    let mut ends = Vec::new();
    for x in [0.0, 400.0] {
        let s = Shape {
            kind: ShapeType::Rectangle,
            style: Style::new(None, Some(INK), INK),
            text: Default::default(),
        };
        let el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), s, [x, 0.0, 160.0, 100.0]);
        ends.push(End::Bound {
            target: el.id,
            anchor: Anchor::Center,
        });
        BoardOp::Put(el).apply(&mut b.doc);
    }
    let mut c = Connector::new(
        ends[0],
        ends[1],
        Route::Straight,
        Style::new(None, Some(INK), INK),
    );
    c.captions = vec![Caption::new("sim", 0.3), Caption::new("não", 0.8)];
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
    BoardOp::Put(el).apply(&mut b.doc);
    set.boards()[0].clone()
}

fn arrow(doc: &ph2d_board_model::BoardDoc) -> Connector {
    doc.live().find_map(|e| e.connector()).unwrap().clone()
}

/// ⭐ O 1.º texto vai PRESO à seta com o `labelPosition` (a fracção), o 2.º vai SOLTO com o dono e o
/// índice, deitado pela linha; os dois voltam ao seu lugar, e um texto mudado LÁ volta mudado.
#[test]
fn the_captions_go_to_excalidraw_and_come_back() {
    let b = board();
    let file: Value = serde_json::from_str(&write(&b)).unwrap();
    let els = file["elements"].as_array().unwrap();
    let arrow_id = els.iter().find(|e| e["type"] == "arrow").unwrap()["id"].clone();
    let bound = els
        .iter()
        .find(|e| e["containerId"] == arrow_id)
        .expect("o 1.º preso");
    assert_eq!(bound["originalText"], "sim");
    assert_eq!(bound["labelPosition"], json!(0.3));
    let loose = els
        .iter()
        .find(|e| e["type"] == "text" && e["originalText"] == "não")
        .expect("o 2.º solto");
    assert!(loose["containerId"].is_null());
    assert_eq!(loose["customData"]["ph2d"]["caption_of"][1], json!(1));
    let back = read(&file.to_string()).unwrap();
    assert_eq!(arrow(&back.doc).captions, arrow(&b.doc).captions);
    assert_eq!(arrow(&back.doc).turn, TextTurn::Along);
    let mut edited = file.clone();
    for e in edited["elements"].as_array_mut().unwrap() {
        if e["originalText"] == "não" {
            e["originalText"] = json!("talvez");
            e["text"] = json!("talvez");
        }
    }
    let back = read(&edited.to_string()).unwrap();
    let caps = arrow(&back.doc).captions;
    assert_eq!(caps[1], Caption::new("talvez", 0.8));
    assert_eq!(
        back.doc.live().filter(|e| e.shape().is_some()).count(),
        2,
        "o texto solto não entra como forma"
    );
}

/// ⭐ Uma seta DELE com o rótulo preso entra com UM texto na fracção do `labelPosition` (sem ele, no
/// meio dos PONTOS — numa de dois pontos, a meio), deitado na horizontal como ele o desenha.
#[test]
fn a_foreign_label_comes_in_level_at_its_label_position() {
    for (lp, at) in [(json!(0.25), 0.25), (Value::Null, Caption::MIDDLE)] {
        let file = json!({
            "type": "excalidraw",
            "elements": [
                { "id": "a", "type": "arrow", "x": 0.0, "y": 0.0, "points": [[0.0, 0.0], [400.0, 0.0]],
                  "boundElements": [{ "id": "t", "type": "text" }] },
                { "id": "t", "type": "text", "x": 180.0, "y": -12.0, "text": "sim", "originalText": "sim",
                  "containerId": "a", "labelPosition": lp }
            ]
        });
        let imp = read(&file.to_string()).unwrap();
        let c = arrow(&imp.doc);
        assert_eq!(c.captions, vec![Caption::new("sim", at)]);
        assert_eq!(c.turn, TextTurn::Level);
    }
}

/// ⭐ Leis 12 e 13 do `label_oracle` (Excalidraw CORRIDO, `excalidraw_rotulo.json`): sem fracção — o
/// next com `labelPosition: null`, o 0.18.1 sempre — o texto fica no meio dos PONTOS (o do meio se são
/// ímpares, senão o meio do trecho do meio), não a meio do comprimento: o centro que ELE desenhou
/// (recolocado por `refreshDimensions`), em fracção do comprimento, é o nosso `middle_of_points`.
#[test]
fn without_a_label_position_the_text_sits_in_the_middle_of_the_points() {
    let oracle: Value = serde_json::from_str(include_str!(
        "../../../docs/MiroClone/ferramentas/label_oracle/excalidraw_rotulo.json"
    ))
    .unwrap();
    let mut seen = 0;
    for (version, all) in [("next", false), ("0181", true)] {
        for case in oracle["versoes"][version]["casos"].as_array().unwrap() {
            let name = case["caso"].as_str().unwrap();
            if !name.starts_with("L_")
                || name.contains("redonda")
                || !(all || name.ends_with("lpnull"))
            {
                continue;
            }
            let step = case["passos"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| {
                    p["passo"]
                        .as_str()
                        .unwrap()
                        .contains("refreshDimensions:true")
                })
                .unwrap();
            let pts: Vec<[f64; 2]> = serde_json::from_value(step["seta"]["abs"].clone()).unwrap();
            let t = &step["texto"];
            let n = |k: &str| t[k].as_f64().unwrap();
            let center = [n("x") + n("width") / 2.0, n("y") + n("height") / 2.0];
            let mut before = 0.0;
            let mut along = None;
            for w in pts.windows(2) {
                let l = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
                let d = (center[0] - w[0][0]).hypot(center[1] - w[0][1]);
                let off = ((w[1][0] - w[0][0]) * (center[1] - w[0][1])
                    - (w[1][1] - w[0][1]) * (center[0] - w[0][0]))
                    .abs()
                    / l;
                if off < 0.5 && d <= l + 0.5 && along.is_none() {
                    along = Some(before + d);
                }
                before += l;
            }
            let frac = along.unwrap_or_else(|| {
                panic!("{version}:{name}: o centro {center:?} não está na seta")
            }) / before;
            let ours = crate::caption::middle_of_points(&pts);
            assert!(
                (frac - ours).abs() < 1e-3,
                "{version}:{name}: ele {frac}, nós {ours}"
            );
            seen += 1;
        }
    }
    assert!(seen >= 8, "casos lidos: {seen}");
}
