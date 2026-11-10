//! P13 R3: Color Mode ▸ Indexed — o modo RGB × ImageMagick 7.1.2 (licença ImageMagick), o RGB de
//! hoje ao byte, e `PreservaRampas` (a claridade nunca se inverte numa rampa).

use std::collections::BTreeMap;

use ph2d_pixel_core::convert::{Bayer, Convert, Dither, convert};
use ph2d_pixel_core::{
    Background, Cel, ColorMode, DocOp, NewSprite, Op, Rect, Sprite, Whole, apply,
};
use serde_json::Value;

pub(crate) fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/imagemagick_indexed.json")).expect("fixture")
}

pub(crate) fn hex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

pub(crate) fn indices(v: &Value) -> Vec<u8> {
    v.as_array()
        .expect("lista")
        .iter()
        .map(|x| x.as_u64().expect("índice") as u8)
        .collect()
}

/// A `foto_p13` num sprite RGB (camada Background: o transparente também é candidato) com a
/// `paleta_rampas` e, se `com_rampas`, as três rampas da fixture.
pub(crate) fn sprite(fx: &Value, rgb: &[u8], com_rampas: bool) -> Sprite {
    let (w, h) = (
        fx["largura"].as_u64().unwrap() as u32,
        fx["altura"].as_u64().unwrap() as u32,
    );
    let pal: Vec<[u8; 4]> = fx["paleta"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let b = indices(c);
            [b[0], b[1], b[2], 255]
        })
        .collect();
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::White,
    };
    let mut s = Sprite::with_palette(spec, &pal).expect("sprite");
    let key = (s.layers()[0].id, s.frames()[0].id);
    let data: Vec<u8> = rgb
        .chunks_exact(3)
        .flat_map(|c| [c[0], c[1], c[2], 255])
        .collect();
    let mut cels = BTreeMap::new();
    cels.insert(key, Cel::image(Rect::new(0, 0, w, h), data));
    let whole = Whole {
        mode: ColorMode::Rgba,
        palette: pal,
        transparent: s.transparent_index(),
        cels,
        tilesets: Vec::new(),
    };
    apply(&mut s, Op::Whole(Box::new(whole))).expect("foto");
    if com_rampas {
        for r in fx["rampas"].as_array().unwrap() {
            let add = DocOp::AddRamp {
                name: r["nome"].as_str().unwrap().to_owned(),
                indices: indices(&r["indices"]),
            };
            apply(&mut s, Op::Doc(Box::new(add))).expect("rampa");
        }
    }
    s
}

pub(crate) fn foto(fx: &Value) -> Vec<u8> {
    hex(fx["foto_p13_rgb"].as_str().unwrap())
}

pub(crate) fn indexed_data(w: &Whole) -> Vec<u8> {
    assert_eq!(w.cels.len(), 1);
    w.cels.values().next().unwrap().data.clone()
}

pub(crate) const DITHERS: [(&str, Dither); 4] = [
    ("nenhum", Dither::None),
    ("o8x8", Dither::Ordered(Bayer::M8)),
    ("old4", Dither::Old(Bayer::M4)),
    ("floyd_steinberg", Dither::FloydSteinberg),
];

fn rgb_de_hoje(s: &Sprite) -> String {
    DITHERS
        .iter()
        .map(|(nome, d)| {
            let opt = Convert {
                dither: *d,
                ..Convert::default()
            };
            let w = convert(s, ColorMode::Indexed, opt).expect("converte");
            let dados: String = indexed_data(&w)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            format!("{nome} {dados}\n")
        })
        .collect()
}

/// O modo RGB de hoje não mudou um byte: a fixture `rgb_antes_r3.txt` foi gravada pelo código de
/// ANTES da porta da métrica (`PH2D_REGRAVA_RGB=1` regrava — só com motivo escrito no commit).
#[test]
fn o_indexed_rgb_de_hoje_nao_mudou_um_byte() {
    let fx = fixture();
    let s = sprite(&fx, &foto(&fx), false);
    let agora = rgb_de_hoje(&s);
    let caminho = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/rgb_antes_r3.txt"
    );
    if std::env::var_os("PH2D_REGRAVA_RGB").is_some() {
        std::fs::write(caminho, &agora).expect("grava");
    }
    let antes = std::fs::read_to_string(caminho).expect("fixture rgb_antes_r3.txt");
    for (a, b) in antes.lines().zip(agora.lines()) {
        let nome = a.split(' ').next().unwrap();
        let (da, db) = (hex(&a[nome.len() + 1..]), hex(&b[nome.len() + 1..]));
        let dif = da.iter().zip(&db).filter(|(x, y)| x != y).count();
        assert_eq!(dif, 0, "{nome}: {dif} píxeis mudaram");
    }
    assert_eq!(antes, agora);
}
