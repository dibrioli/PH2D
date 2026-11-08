//! **User data no `.ase`** (P12, `docs/PixelArt/P12_plano.md` §9.5): cor, texto e propriedades de
//! cada dono vão e voltam; a fixture `fixtures/user_data/p12_todos_os_donos.ase` é a que o Trial
//! leu (`fixtures/user_data/p12_todos_os_donos.trial.json`, cabeçalho no `README.md` ao lado).

use std::collections::BTreeMap;

use ph2d_pixel_core::{
    ColorMode, History, NewSprite, Op, Owner, Paint, PropMap, PropValue, Sprite, Stroke, UserData,
    apply, frames, tiles,
};
use ph2d_pixel_io::ase::{escrever, ler};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/user_data/p12_todos_os_donos.ase"
);

fn dados(texto: &str, cor: [u8; 4]) -> UserData {
    let mut m = PropMap::new();
    m.insert("hp".into(), PropValue::I32(-12));
    m.insert("name".into(), PropValue::Str("ok".into()));
    m.insert("speed".into(), PropValue::F64(1.25));
    m.insert("box".into(), PropValue::Rect(1, 2, 3, 4));
    m.insert(
        "list".into(),
        PropValue::Vector(vec![PropValue::U8(1), PropValue::Str("x".into())]),
    );
    let mut dentro = PropMap::new();
    dentro.insert("deep".into(), PropValue::Bool(true));
    m.insert("nested".into(), PropValue::Map(dentro));
    let mut ext = PropMap::new();
    ext.insert("weight".into(), PropValue::U16(7));
    UserData {
        color: cor,
        text: texto.into(),
        props: BTreeMap::from([(String::new(), m), ("ph2d.p12".into(), ext)]),
    }
}

fn documento() -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 16,
        height: 16,
        mode: ColorMode::Rgba,
        ..NewSprite::default()
    })
    .expect("nasce");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let v = s.value_of_rgba([10, 200, 30, 255]);
    let mut st = Stroke::begin(&s, l, f, v, Paint::default(), (1, 1));
    st.to(&s, 6, 4);
    let op = st.op(&s).expect("traço");
    History::default().commit(&mut s, op).expect("pinta");
    let tag = frames::new_tag(&s, "walk", 0, 0).expect("tag");
    apply(&mut s, tag).expect("tag");
    let (op, tl, _) = tiles::new_tilemap_layer(&mut s, l, 8, 8).expect("tilemap");
    apply(&mut s, op).expect("tilemap");
    let _ = tl;
    let donos = [
        (Owner::Sprite, "sprite data", [255, 0, 0, 255]),
        (Owner::Layer(l), "layer data", [0, 255, 0, 255]),
        (Owner::Cel((l, f)), "cel data", [0, 0, 255, 128]),
        (Owner::Tag(0), "tag data", [0; 4]),
        (
            Owner::Tileset(s.tilesets()[0].id),
            "tileset data",
            [0, 255, 255, 255],
        ),
    ];
    for (o, t, c) in donos {
        let op = Op::UserData {
            owner: o,
            data: Box::new(dados(t, c)),
        };
        apply(&mut s, op).expect("user data");
    }
    s
}

#[test]
fn o_ase_com_user_data_em_todos_os_donos_vai_e_volta() {
    let s = documento();
    let (bytes, _) = escrever(&s).expect("escreve");
    let (back, _) = ler(&bytes).expect("lê");
    let l = back
        .layers()
        .iter()
        .find(|l| l.tileset().is_none())
        .expect("camada")
        .id;
    let f = back.frames()[0].id;
    let set = back.tilesets()[0].id;
    for (o, t, c) in [
        (Owner::Sprite, "sprite data", [255, 0, 0, 255]),
        (Owner::Layer(l), "layer data", [0, 255, 0, 255]),
        (Owner::Cel((l, f)), "cel data", [0, 0, 255, 128]),
        (Owner::Tag(0), "tag data", [0; 4]),
        (Owner::Tileset(set), "tileset data", [0, 255, 255, 255]),
    ] {
        assert_eq!(back.user_of(o), Some(&dados(t, c)), "{o:?}");
    }
    // O id do tileset renumera-se ao ler (1.º id livre); da 2.ª volta em diante os bytes param.
    let (de_novo, _) = escrever(&back).expect("regrava");
    let (outra, _) = ler(&de_novo).expect("relê");
    assert_eq!(
        escrever(&outra).expect("regrava").0,
        de_novo,
        "os mesmos bytes"
    );
    if std::env::var_os("PH2D_GRAVA_FIXTURE").is_some() {
        std::fs::write(FIXTURE, &bytes).expect("grava a fixture");
    }
    let gravada = std::fs::read(FIXTURE).expect("fixture (PH2D_GRAVA_FIXTURE=1 para a gravar)");
    assert_eq!(
        gravada, bytes,
        "a fixture que o Trial leu é a que escrevemos hoje"
    );
}

/// O Trial leu a fixture (cabeçalho em `fixtures/user_data/README.md`): o texto, a cor e as
/// propriedades da camada, do cel e da tag são os que escrevemos.
#[test]
fn o_trial_le_o_user_data_que_escrevemos() {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/user_data/p12_todos_os_donos.trial.json"
    ))
    .expect("json");
    let m = &v["meta"];
    assert_eq!(m["version"], "1.3.18.6-x64-trial");
    let l = &m["layers"][0];
    let c = &l["cels"][0];
    let t = &m["frameTags"][0];
    for (dono, texto, cor) in [
        (l, "layer data", "#00ff00ff"),
        (c, "cel data", "#0000ff80"),
        (t, "tag data", "#000000ff"),
    ] {
        assert_eq!(dono["data"], texto);
        assert_eq!(dono["color"], cor);
        let p = &dono["properties"];
        assert_eq!(p["hp"], -12);
        assert_eq!(p["name"], "ok");
        assert_eq!(p["speed"], 1.25);
        assert_eq!(p["nested"]["deep"], true);
        assert_eq!(p["ph2d.p12"]["weight"], 7);
    }
}
