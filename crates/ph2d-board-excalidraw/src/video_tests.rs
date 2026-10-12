//! W18 — o vídeo de ida e volta pelo `.excalidraw`, sobre o ficheiro que o PRÓPRIO oráculo gravou
//! (`saidas/w18/w18_offline_Z04_gravado.excalidraw`: um `embeddable` 560×315 com o link do YouTube).

use ph2d_board_model::{Board, BoardSet, Video};

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
