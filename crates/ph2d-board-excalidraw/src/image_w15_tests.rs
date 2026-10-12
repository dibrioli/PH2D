//! W15 — a MOLDURA e o PRETO E BRANCO de uma imagem no `.excalidraw`. O oráculo (Excalidraw 0.18.1,
//! `saidas/w15/`) não desenha contorno numa imagem nem tem filtro: os dois são nossos e viajam no
//! nosso elemento do `customData`. Uma imagem DELE (sem o nosso) nunca ganha moldura: o `strokeColor`
//! de nascença dele (`#1e1e1e`) está em toda imagem e não se desenha lá.

use super::image_tests::{png_ingest, sample};
use super::*;
use ph2d_board_model::{Board, BoardOp, Dash, Element, ImageFilter, Rgba};
use serde_json::Value;

const ORACLE: &str = "../../docs/MiroClone/ferramentas/excalidraw_oracle";
const OURS: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w15_nosso.excalidraw"
);
const OURS_RESTORED: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w15/w15_nosso.restored.json"
);

/// A amostra da W14 e uma 4.ª imagem com moldura vermelha tracejada de 4 e a preto e branco.
fn sample_w15() -> Board {
    let mut b = sample();
    let file = b.files.keys().next().copied().unwrap();
    let mut img = ph2d_board_model::Image::new(
        file,
        ph2d_board_model::Style::new(None, Some(Rgba([224, 49, 49, 255])), Rgba([30, 30, 30, 255])),
    );
    img.style.stroke_width = 4.0;
    img.style.dash = Dash::Dashed;
    img.filter = ImageFilter::Gray;
    let el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), img, [200.0, 120.0, 160.0, 80.0]);
    BoardOp::Put(el).apply(&mut b.doc);
    b
}

/// Grava a ENTRADA do oráculo (`entradas/w15_nosso.excalidraw`). À mão quando o escritor muda — e a
/// seguir o oráculo: `corre.sh saidas/w15 entradas/w15_nosso.excalidraw`.
#[test]
#[ignore = "grava a entrada do oráculo (à mão, quando o escritor muda)"]
fn grava_a_entrada_do_oraculo_w15() {
    let path = format!(
        "{}/{ORACLE}/entradas/w15_nosso.excalidraw",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::write(path, write(&sample_w15())).unwrap();
}

/// O escritor continua a escrever a entrada que o oráculo correu.
#[test]
fn our_writer_still_writes_the_w15_oracle_input() {
    assert_eq!(
        write(&sample_w15()),
        OURS,
        "o escritor mudou: corra o oráculo outra vez"
    );
}

/// ⭐⭐ O nosso quadro → `.excalidraw` → o `restore` do Excalidraw → o nosso leitor = as MESMAS imagens,
/// com a moldura e o preto e branco (pelo `customData`).
#[test]
fn the_round_trip_keeps_the_frame_and_the_gray() {
    let mut file: Value = serde_json::from_str(OURS).unwrap();
    file["elements"] = serde_json::from_str(OURS_RESTORED).unwrap();
    let back = read_with(&file.to_string(), &png_ingest).unwrap();
    assert!(back.skipped.is_empty(), "{:?}", back.skipped);
    let gray: Vec<&ph2d_board_model::Image> = back
        .doc
        .live()
        .filter_map(Element::image)
        .filter(|i| i.filter == ImageFilter::Gray)
        .collect();
    assert_eq!(gray.len(), 1);
    let st = &gray[0].style;
    assert_eq!(
        (st.stroke, st.stroke_width, st.dash),
        (Some(Rgba([224, 49, 49, 255])), 4.0, Dash::Dashed)
    );
    let norm = |d: &ph2d_board_model::BoardDoc| -> Vec<Element> {
        d.live_in_z_order()
            .into_iter()
            .map(|e| Element {
                z: ph2d_board_model::FracKey::between(None, None),
                version: 0,
                ..e.clone()
            })
            .collect()
    };
    assert_eq!(norm(&back.doc), norm(&sample_w15().doc));
}

/// Uma imagem DELE com o `strokeColor` de nascença (o que o oráculo devolve a uma imagem mínima) entra
/// SEM moldura e a cores.
#[test]
fn a_foreign_image_never_gets_a_frame() {
    let restored: Value = serde_json::from_str(OURS_RESTORED).unwrap();
    let mut el = restored
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["type"] == "image")
        .unwrap()
        .clone();
    el.as_object_mut().unwrap().remove("customData");
    el["strokeColor"] = "#1e1e1e".into();
    el["strokeWidth"] = 2.into();
    let mut file: Value = serde_json::from_str(OURS).unwrap();
    file["elements"] = Value::Array(vec![el]);
    let back = read_with(&file.to_string(), &png_ingest).unwrap();
    let i = back.doc.live().find_map(Element::image).unwrap().clone();
    assert_eq!((i.style.stroke, i.filter), (None, ImageFilter::Original));
}
