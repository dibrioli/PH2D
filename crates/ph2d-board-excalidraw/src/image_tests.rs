//! W14 — a IMAGEM no `.excalidraw`, contra o oráculo Excalidraw 0.18.1
//! (`ferramentas/excalidraw_oracle/saidas/w14/`, com `meta.json`): o ficheiro DELE abre com as imagens
//! (espelho, recorte, cantos, opacidade, rotação; a sem ficheiro salta), e o NOSSO faz ida e volta
//! pelo `restore` dele igual.

use super::*;
use ph2d_board_model::{
    Board, BoardOp, BoardSet, Crop, Element, Frame, Image, ImageFile, Rgba, Style,
};
use serde_json::Value;

const ORACLE: &str = "../../docs/MiroClone/ferramentas/excalidraw_oracle";
const THEIRS: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w14_imagens.excalidraw"
);
const OURS: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w14_nosso.excalidraw"
);
const OURS_RESTORED: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w14/w14_nosso.restored.json"
);
const W8_TYPES: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w8_tipos.serialized.excalidraw"
);

/// A porta de entrada dos testes: só PNG (o tamanho pelo cabeçalho).
fn png_ingest(bytes: &[u8]) -> Option<ImageFile> {
    let r = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let i = r.info();
    Some(ImageFile::new(bytes.to_vec(), [i.width, i.height]))
}

/// A PNG 4×2 da entrada do oráculo (oito cores, uma por píxel).
fn png_4x2() -> Vec<u8> {
    let v: Value = serde_json::from_str(THEIRS).unwrap();
    let url = v["files"]["a4x2"]["dataURL"].as_str().unwrap();
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(url.split_once(",").unwrap().1)
        .unwrap()
}

fn by_box(doc: &ph2d_board_model::BoardDoc, x: f64, y: f64) -> Element {
    doc.live()
        .find(|e| e.x == x && e.y == y)
        .unwrap_or_else(|| panic!("nenhuma imagem em ({x}, {y})"))
        .clone()
}

/// ⭐ O ficheiro DELE abre com as imagens: a caixa, o espelho (`scale`), o recorte em píxeis naturais,
/// os cantos, a opacidade e a rotação; a imagem sem ficheiro (`pending`) salta e conta-se; os bytes
/// entram uma vez.
#[test]
fn the_oracle_scene_opens_with_its_images() {
    let imp = read_with(THEIRS, &png_ingest).unwrap();
    assert_eq!(imp.skipped.get("image"), Some(&1), "{:?}", imp.skipped);
    assert_eq!(imp.doc.live_len(), 5);
    assert_eq!(imp.files.len(), 1);
    let f = imp.files.values().next().unwrap();
    assert_eq!(f.size, [4, 2]);
    let img = |x, y| by_box(&imp.doc, x, y);
    let plain = img(0.0, 0.0);
    let i = plain.image().unwrap();
    assert_eq!((i.crop, i.flip, i.file), (None, [false; 2], f.id()));
    assert_eq!([plain.w, plain.h], [160.0, 80.0]);
    assert_eq!(img(200.0, 0.0).image().unwrap().flip, [true, false]);
    let c = img(400.0, 0.0).image().unwrap().crop.unwrap();
    assert_eq!([c.x, c.y, c.w, c.h], [2.0, 0.0, 2.0, 2.0]);
    let r = img(0.0, 120.0);
    let ri = r.image().unwrap();
    assert!(ri.style.round);
    assert_eq!((ri.style.opacity, r.angle), (50, 0.5));
    let both = img(400.0, 120.0);
    let bi = both.image().unwrap();
    assert_eq!(bi.flip, [false, true]);
    let c = bi.crop.unwrap();
    assert_eq!([c.x, c.y, c.w, c.h], [1.0, 0.0, 2.0, 1.0]);
}

/// Sem a porta de entrada as imagens saltam todas (contadas) — o leitor sem descodificador.
#[test]
fn without_an_ingest_every_image_is_skipped() {
    let imp = read(THEIRS).unwrap();
    assert_eq!(imp.skipped.get("image"), Some(&6));
    assert!(imp.files.is_empty());
}

/// A imagem do ficheiro de tipos da W8 (que saltava) entra agora, presa ao frame dela.
#[test]
fn the_w8_image_now_comes_in_inside_its_frame() {
    let imp = read_with(W8_TYPES, &png_ingest).unwrap();
    assert!(!imp.skipped.contains_key("image"), "{:?}", imp.skipped);
    let el = imp.doc.live().find(|e| e.image().is_some()).unwrap();
    assert_eq!([el.x, el.y, el.w, el.h], [280.0, 450.0, 96.0, 96.0]);
    let frame = imp.doc.get(el.frame.expect("no frame")).unwrap();
    assert!(frame.as_frame().is_some());
}

/// A crop vem em píxeis da imagem que ENTROU: uma `naturalWidth` diferente da nossa escala-a.
#[test]
fn a_crop_from_another_natural_size_scales_to_ours() {
    let mut v: Value = serde_json::from_str(THEIRS).unwrap();
    let els = v["elements"].as_array_mut().unwrap();
    let el = els.iter_mut().find(|e| e["id"] == "img3").unwrap();
    el["crop"] = serde_json::json!({ "x": 4, "y": 0, "width": 4, "height": 4,
        "naturalWidth": 8, "naturalHeight": 4 });
    let imp = read_with(&v.to_string(), &png_ingest).unwrap();
    let c = by_box(&imp.doc, 400.0, 0.0).image().unwrap().crop.unwrap();
    assert_eq!([c.x, c.y, c.w, c.h], [2.0, 0.0, 2.0, 2.0]);
}

/// O nosso quadro de amostra com imagens: inteira, espelhada e recortada, redonda e meio opaca e
/// rodada, uma dentro de um frame, e duas que mostram o MESMO ficheiro.
fn sample() -> Board {
    let mut set = BoardSet::default();
    let a = set.create("Fotos".into());
    let b = set.get_mut(a).unwrap();
    let file = b.put_file(png_ingest(&png_4x2()).unwrap());
    let st = Style::new(None, None, Rgba([30, 30, 30, 255]));
    let frame = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Frame {
            name: "Moldura".into(),
            style: st.clone(),
        },
        [-20.0, -20.0, 600.0, 300.0],
    );
    let frame_id = frame.id;
    BoardOp::Put(frame).apply(&mut b.doc);
    let mut add = |bx: [f64; 4], f: &dyn Fn(&mut Element)| {
        let img = Image::new(file, st.clone());
        let mut el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), img, bx);
        f(&mut el);
        BoardOp::Put(el).apply(&mut b.doc);
    };
    add([0.0, 0.0, 160.0, 80.0], &|el| el.frame = Some(frame_id));
    add([200.0, 0.0, 80.0, 80.0], &|el| {
        let i = el.image_mut().unwrap();
        i.flip = [true, false];
        i.crop = Some(Crop {
            x: 2.0,
            y: 0.0,
            w: 2.0,
            h: 2.0,
        });
    });
    add([0.0, 120.0, 160.0, 80.0], &|el| {
        el.angle = 0.5;
        let i = el.image_mut().unwrap();
        i.style.round = true;
        i.style.opacity = 50;
    });
    set.boards()[0].clone()
}

/// Grava a ENTRADA do oráculo (`entradas/w14_nosso.excalidraw`). À mão quando o escritor muda — e a
/// seguir o oráculo: `corre.sh saidas/w14 entradas/w14_nosso.excalidraw`.
#[test]
#[ignore = "grava a entrada do oráculo (à mão, quando o escritor muda)"]
fn grava_a_entrada_do_oraculo_w14() {
    let path = format!(
        "{}/{ORACLE}/entradas/w14_nosso.excalidraw",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::write(path, write(&sample())).unwrap();
}

/// O escritor continua a escrever a entrada que o oráculo correu, e o `files` leva o ficheiro UMA vez.
#[test]
fn our_writer_still_writes_the_w14_oracle_input() {
    let ours = write(&sample());
    assert_eq!(ours, OURS, "o escritor mudou: corra o oráculo outra vez");
    let v: Value = serde_json::from_str(&ours).unwrap();
    assert_eq!(v["files"].as_object().unwrap().len(), 1);
}

/// ⭐⭐ O nosso quadro → `.excalidraw` → o `restore` do Excalidraw → o nosso leitor = as MESMAS imagens
/// (o mesmo ficheiro, recorte, espelho, cantos, opacidade, rotação, frame).
#[test]
fn the_excalidraw_round_trip_returns_our_images() {
    let mut file: Value = serde_json::from_str(OURS).unwrap();
    file["elements"] = serde_json::from_str(OURS_RESTORED).unwrap();
    let back = read_with(&file.to_string(), &png_ingest).unwrap();
    assert!(back.skipped.is_empty(), "{:?}", back.skipped);
    let ours = sample();
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
    assert_eq!(norm(&back.doc), norm(&ours.doc));
    assert_eq!(back.files, ours.files);
}
