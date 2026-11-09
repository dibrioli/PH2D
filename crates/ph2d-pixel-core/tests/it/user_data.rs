//! **User data e preferências** (P12, `docs/PixelArt/P12_plano.md` §9.5–9.6): a porta única com o
//! inverso exacto, o formato MINOR 6 e as preferências de fábrica (O18, O19, O21).

use std::collections::BTreeMap;

use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::frames;
use ph2d_pixel_core::tiles;
use ph2d_pixel_core::{
    ColorMode, History, NewSprite, Owner, Paint, PixelPrefs, PropMap, PropValue, Sprite, Stroke,
    UserData, apply,
};

/// Um sprite com cada dono: o sprite, duas camadas (uma de tilemap), um cel, uma tag, um tileset.
pub fn com_donos() -> Sprite {
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
    let (op, _, _) = tiles::new_tilemap_layer(&mut s, l, 8, 8).expect("tilemap");
    apply(&mut s, op).expect("tilemap");
    s
}

/// Um user data com todos os tipos de propriedade do `.ase`, num mapa do artista e num de extensão.
pub fn cheio(texto: &str) -> UserData {
    let mut dentro = PropMap::new();
    dentro.insert("hp".into(), PropValue::U16(300));
    let mut m = PropMap::new();
    for (k, v) in [
        ("a_bool", PropValue::Bool(true)),
        ("b_i8", PropValue::I8(-5)),
        ("c_u8", PropValue::U8(250)),
        ("d_i16", PropValue::I16(-30000)),
        ("e_u16", PropValue::U16(60000)),
        ("f_i32", PropValue::I32(-2_000_000)),
        ("g_u32", PropValue::U32(4_000_000_000)),
        ("h_i64", PropValue::I64(-9_000_000_000)),
        ("i_u64", PropValue::U64(u64::MAX)),
        ("j_fixed", PropValue::Fixed(0x0001_8000)),
        ("k_f32", PropValue::F32(1.5)),
        ("l_f64", PropValue::F64(-0.25)),
        ("m_str", PropValue::Str("olá".into())),
        ("n_point", PropValue::Point(-3, 4)),
        ("o_size", PropValue::Size(10, 20)),
        ("p_rect", PropValue::Rect(1, -2, 3, 4)),
        (
            "q_vec",
            PropValue::Vector(vec![PropValue::I32(1), PropValue::Str("x".into())]),
        ),
        ("r_map", PropValue::Map(dentro)),
        ("s_uuid", PropValue::Uuid([7; 16])),
    ] {
        m.insert(k.into(), v);
    }
    let mut ext = PropMap::new();
    ext.insert("speed".into(), PropValue::F64(2.0));
    let mut props = BTreeMap::new();
    props.insert(String::new(), m);
    props.insert("ph2d.test".into(), ext);
    UserData {
        color: [255, 0, 128, 200],
        text: texto.into(),
        props,
    }
}

pub fn donos(s: &Sprite) -> Vec<Owner> {
    let l = s.layers()[0].id;
    let f = s.frames()[0].id;
    vec![
        Owner::Sprite,
        Owner::Layer(l),
        Owner::Cel((l, f)),
        Owner::Tag(0),
        Owner::Tileset(s.tilesets()[0].id),
    ]
}

pub fn preenche(s: &mut Sprite) {
    for (i, o) in donos(s).into_iter().enumerate() {
        let op = ph2d_pixel_core::Op::UserData {
            owner: o,
            data: Box::new(cheio(&format!("dono {i}"))),
        };
        apply(s, op).expect("user data");
    }
}

#[test]
fn o_user_data_de_cada_dono_tem_inverso_exacto() {
    let mut s = com_donos();
    let antes = s.clone();
    for o in donos(&s) {
        assert_eq!(
            s.user_of(o),
            Some(&UserData::default()),
            "{o:?} nasce vazio"
        );
        let op = ph2d_pixel_core::Op::UserData {
            owner: o,
            data: Box::new(cheio("x")),
        };
        let inv = apply(&mut s, op).expect("aplica");
        assert_eq!(s.user_of(o), Some(&cheio("x")));
        apply(&mut s, inv).expect("desfaz");
        assert_eq!(s, antes, "{o:?}: o inverso devolve o documento");
    }
    let fora = ph2d_pixel_core::Op::UserData {
        owner: Owner::Tag(9),
        data: Box::new(cheio("x")),
    };
    assert!(
        apply(&mut s, fora).is_err(),
        "dono que não existe recusa-se"
    );
    assert_eq!(s, antes);
}

#[test]
fn o_user_data_e_as_preferencias_vao_e_voltam_no_formato() {
    let mut s = com_donos();
    preenche(&mut s);
    let mut p = PixelPrefs::default();
    p.grid.w = 8;
    p.grid.visible = true;
    p.checker.color_a = [1, 2, 3, 4];
    p.onion.loop_tag = false;
    p.onion.prev = 3;
    s.set_prefs(p);
    let lib = BTreeMap::from([(1, s)]);
    let bytes = write_library(&lib);
    let back = read_library(&bytes).expect("lê");
    assert_eq!(back, lib);
    assert_eq!(write_library(&back), bytes, "regravar dá os mesmos bytes");
}

#[test]
fn um_documento_sem_user_data_grava_como_o_minor_5_e_le_vazio() {
    let s = com_donos();
    let mut bytes = write_library(&BTreeMap::from([(1, s.clone())]));
    // Sem user data nem preferências mudadas nada se acrescenta: um ficheiro do MINOR 5 é igual.
    bytes[11..13].copy_from_slice(&5u16.to_le_bytes());
    let back = read_library(&bytes).expect("lê o MINOR 5");
    let t = &back[&1];
    assert_eq!(t, &s);
    for o in donos(t) {
        assert_eq!(t.user_of(o), Some(&UserData::default()));
    }
    assert_eq!(t.prefs(), &PixelPrefs::default());
}

#[test]
fn as_preferencias_de_fabrica_sao_as_do_aseprite() {
    let p = PixelPrefs::default();
    // O18
    assert_eq!((p.grid.x, p.grid.y, p.grid.w, p.grid.h), (0, 0, 16, 16));
    assert_eq!(p.grid.color, [0, 0, 255, 255]);
    assert_eq!(
        (p.grid.opacity, p.grid.auto_opacity, p.grid.visible),
        (160, true, false)
    );
    assert_eq!(p.pixel_grid.color, [0xc8, 0xc8, 0xc8, 255]);
    assert_eq!(
        (
            p.pixel_grid.opacity,
            p.pixel_grid.auto_opacity,
            p.pixel_grid.visible
        ),
        (160, true, false)
    );
    // O19
    assert_eq!(
        (p.checker.w, p.checker.h, p.checker.apply_zoom),
        (16, 16, true)
    );
    assert_eq!(p.checker.color_a, [0x80, 0x80, 0x80, 255]);
    assert_eq!(p.checker.color_b, [0xc0, 0xc0, 0xc0, 255]);
    // O21: 27 % e 11 % na unidade 0–255.
    let o = p.onion;
    assert_eq!(
        u32::from(o.base) * 100 / 255,
        26,
        "68/255 = 26,7 %: o Trial arredonda a 27"
    );
    assert_eq!((u32::from(o.base) * 100).div_ceil(255), 27);
    assert_eq!((u32::from(o.step) * 100 + 127) / 255, 11);
    assert!(!o.tint && !o.front && o.loop_tag && !o.current_layer_only);
}

#[test]
fn as_preferencias_vao_e_voltam_em_texto() {
    let mut p = PixelPrefs::default();
    p.grid.x = -2;
    p.grid.color = [9, 8, 7, 6];
    p.pixel_grid.visible = true;
    p.checker.apply_zoom = false;
    p.onion.tint = true;
    p.onion.next = 4;
    let linhas = p.to_lines();
    assert!(linhas.iter().all(|l| l.starts_with("pixel.")));
    let mut com_lixo: Vec<&str> = linhas.iter().map(String::as_str).collect();
    com_lixo.push("ui.theme=dark");
    com_lixo.push("pixel.grid.w=0");
    com_lixo.push("pixel.nada=1");
    assert_eq!(PixelPrefs::from_lines(com_lixo), p);
    assert_eq!(PixelPrefs::from_lines([]), PixelPrefs::default());
}

/// A slice também é dono (P12 U): a porta única tem inverso exacto, e os mapas (com os da extensão)
/// vão e voltam no formato ao lado da cor e do texto de sempre.
#[test]
fn a_slice_e_dona_de_user_data_com_inverso_e_formato() {
    let mut s = com_donos();
    let (op, id) =
        ph2d_pixel_core::slice::create(&mut s, 0, ph2d_pixel_core::Rect::new(1, 1, 4, 4))
            .expect("slice");
    apply(&mut s, op).expect("slice");
    let antes = s.clone();
    let o = Owner::Slice(id);
    let inv = apply(
        &mut s,
        ph2d_pixel_core::Op::UserData {
            owner: o,
            data: Box::new(cheio("s")),
        },
    )
    .expect("aplica");
    assert_eq!(s.user_of(o), Some(&cheio("s")));
    assert_eq!(s.slice(id).expect("slice").draw_color(), cheio("s").color);
    let lib = BTreeMap::from([(1, s.clone())]);
    let back = read_library(&write_library(&lib)).expect("lê");
    assert_eq!(back, lib, "cor, texto e mapas da slice no formato");
    apply(&mut s, inv).expect("desfaz");
    assert_eq!(s, antes);
}

/// O peso de um `Op::UserData` (o que o histórico conta contra o tecto) segue os textos e os
/// vectores de verdade, também dentro de mapas aninhados.
#[test]
fn user_data_op_bytes_count_strings_and_nested_values() {
    let peso = |v: PropValue| {
        let mut m = PropMap::new();
        m.insert("k".into(), v);
        ph2d_pixel_core::Op::UserData {
            owner: Owner::Sprite,
            data: Box::new(UserData {
                props: BTreeMap::from([(String::new(), m)]),
                ..UserData::default()
            }),
        }
        .bytes()
    };
    let mega = 1 << 20;
    let pouco = peso(PropValue::Bool(true));
    assert!(peso(PropValue::Str("x".repeat(mega))) >= pouco + mega);
    let mut dentro = PropMap::new();
    dentro.insert("s".into(), PropValue::Str("y".repeat(mega)));
    let fundo = PropValue::Vector(vec![PropValue::Map(dentro); 3]);
    assert!(peso(fundo) >= pouco + 3 * mega);
    let largo = PropValue::Vector(vec![PropValue::U8(1); 100_000]);
    assert!(peso(largo) >= pouco + 100_000);
}
