//! W18 — o vídeo de ida e volta pelo `.excalidraw`, sobre o ficheiro que o PRÓPRIO oráculo gravou
//! (`saidas/w18/w18_offline_Z04_gravado.excalidraw`: um `embeddable` 560×315 com o link do YouTube).

use ph2d_board_model::{Board, BoardSet, Dash, Rgba, Video};
use serde_json::json;

use crate::{read, write};

const GRAVADO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w18/w18_offline_Z04_gravado.excalidraw"
));

fn board_of(doc: ph2d_board_model::BoardDoc) -> Board {
    let mut set = BoardSet::default();
    let id = set.create("v".into());
    set.get_mut(id).unwrap().doc = doc;
    set.boards()[0].clone()
}

/// ⭐⭐ O `embeddable` do oráculo entra como VÍDEO (antes saltava com aviso), com a caixa e o link dele;
/// e sai de novo como `embeddable` com o mesmo link e os cantos redondos.
#[test]
fn the_oracle_s_embeddable_reads_as_a_video_and_writes_back() {
    let imp = read(GRAVADO).unwrap();
    assert!(imp.skipped.is_empty(), "nada salta: {:?}", imp.skipped);
    let el = imp
        .doc
        .live()
        .find(|e| e.embed().is_some())
        .expect("o vídeo");
    let e = el.embed().unwrap();
    assert_eq!(e.link, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(
        e.video,
        Video::YouTube {
            id: "dQw4w9WgXcQ".into(),
            start: 0,
            vertical: false
        }
    );
    assert_eq!([el.x, el.y, el.w, el.h], [640.0, 400.0, 560.0, 315.0]);
    assert!(e.style.round, "roundness {{type: 3}}");
    let out: serde_json::Value = serde_json::from_str(&write(&board_of(imp.doc.clone()))).unwrap();
    let v = out["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["type"] == "embeddable")
        .expect("o embeddable");
    assert_eq!(v["link"], "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(v["roundness"]["type"], 3);
    assert_eq!(
        [v["width"].as_f64(), v["height"].as_f64()],
        [Some(560.0), Some(315.0)]
    );
    // E volta a ler-se igual (o nosso no `customData`).
    let back = read(&out.to_string()).unwrap();
    let el2 = back.doc.live().find(|e| e.embed().is_some()).unwrap();
    assert_eq!(el2.embed(), el.embed());
}

/// O que o oráculo gravou com um vídeo DECORADO (`sonda_w20.mjs` E2): stroke `#e03131` 4 tracejado, fundo
/// `#ffec99` sólido, opacidade 60, cantos rectos.
const DECORADO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w20/w20_E2_gravado.excalidraw"
));

/// ⭐⭐ W20 — a decoração de um vídeo viaja como a de um rectângulo: o `embeddable` decorado do oráculo
/// entra com a borda, o fundo, o tracejado, a opacidade e os cantos dele, e sai de novo com eles.
#[test]
fn a_decorated_embeddable_keeps_its_border_and_fill_both_ways() {
    let imp = read(DECORADO).unwrap();
    assert!(imp.skipped.is_empty(), "{:?}", imp.skipped);
    let el = imp
        .doc
        .live()
        .find(|e| e.embed().is_some())
        .expect("o vídeo");
    let st = &el.embed().unwrap().style;
    assert_eq!(st.stroke, Some(Rgba([0xe0, 0x31, 0x31, 255])), "a borda");
    assert_eq!(st.fill, Some(Rgba([0xff, 0xec, 0x99, 255])), "o fundo");
    assert_eq!((st.stroke_width, st.dash), (4.0, Dash::Dashed));
    assert_eq!((st.opacity, st.round), (60, false));
    let out: serde_json::Value = serde_json::from_str(&write(&board_of(imp.doc.clone()))).unwrap();
    let v = out["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["type"] == "embeddable")
        .expect("o embeddable");
    assert_eq!(
        (&v["strokeColor"], &v["backgroundColor"], &v["strokeStyle"]),
        (&json!("#e03131"), &json!("#ffec99"), &json!("dashed"))
    );
    assert_eq!(
        (v["strokeWidth"].as_f64(), v["opacity"].as_f64()),
        (Some(4.0), Some(60.0))
    );
    assert!(v["roundness"].is_null());
    let back = read(&out.to_string()).unwrap();
    let el2 = back.doc.live().find(|e| e.embed().is_some()).unwrap();
    assert_eq!(el2.embed(), el.embed());
}

/// Um `embeddable` de outro site (o Vimeo) não é um vídeo do quadro: salta, com o aviso de sempre.
#[test]
fn a_foreign_embeddable_still_skips_with_the_warning() {
    let vimeo = GRAVADO.replace(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "https://vimeo.com/76979871",
    );
    let imp = read(&vimeo).unwrap();
    assert_eq!(imp.skipped.get("embeddable"), Some(&1));
    assert!(imp.doc.live().all(|e| e.embed().is_none()));
}
