//! O user data (`0x2020`) de cada dono e o External Files (`0x2008`) no `.ase` escrito aqui (P12).

use ph2d_aseprite::{
    AseCel, AseCelBody, AseExternalFile, AseFile, AseFileFrame, AseLayer, AseProp, AsePropMap,
    AseSlice, AseSliceKey, AseTag, AseTileset, AseUserData, parse_file, write_file,
};

fn ud(texto: &str, cor: Option<[u8; 4]>) -> Option<AseUserData> {
    Some(AseUserData {
        text: Some(texto.into()),
        color: cor,
        maps: vec![
            AsePropMap {
                key: 0,
                props: vec![
                    ("bool".into(), AseProp::Bool(true)),
                    ("i64".into(), AseProp::I64(-7)),
                    ("f32".into(), AseProp::F32(0.5)),
                    ("rect".into(), AseProp::Rect(1, 2, 3, 4)),
                    (
                        "vec_igual".into(),
                        AseProp::Vector(vec![AseProp::U8(1), AseProp::U8(2)]),
                    ),
                    (
                        "vec_misto".into(),
                        AseProp::Vector(vec![AseProp::U8(1), AseProp::Str("a".into())]),
                    ),
                    (
                        "mapa".into(),
                        AseProp::Map(vec![("uuid".into(), AseProp::Uuid([9; 16]))]),
                    ),
                ],
            },
            AsePropMap {
                key: 1,
                props: vec![("fixed".into(), AseProp::Fixed(0x0002_0000))],
            },
        ],
    })
}

#[test]
fn o_user_data_de_cada_dono_vai_e_volta_no_ase() {
    let file = AseFile {
        width: 8,
        height: 8,
        depth: 32,
        palette: vec![[0, 0, 0, 255], [255, 255, 255, 255]],
        layers: vec![AseLayer {
            name: "L".into(),
            flags: 3,
            opacity: 255,
            user_data: ud("camada", Some([1, 2, 3, 4])),
            ..AseLayer::default()
        }],
        frames: vec![AseFileFrame {
            duration_ms: 100,
            cels: vec![AseCel {
                layer: 0,
                x: 0,
                y: 0,
                opacity: 255,
                z: 0,
                body: AseCelBody::Image {
                    w: 1,
                    h: 1,
                    pixels: vec![1, 2, 3, 255],
                },
                user_data: ud("cel", None),
            }],
            palette: None,
        }],
        tags: vec![
            AseTag {
                name: "a".into(),
                from: 0,
                to: 0,
                direction: 0,
                repeat: 0,
                rgb: [0, 0, 0],
                user_data: None,
            },
            AseTag {
                name: "b".into(),
                from: 0,
                to: 0,
                direction: 0,
                repeat: 0,
                rgb: [0, 0, 0],
                user_data: ud("tag b", Some([5, 6, 7, 8])),
            },
        ],
        tilesets: vec![AseTileset {
            id: 0,
            name: "T".into(),
            zero_is_empty: true,
            tile_w: 1,
            tile_h: 1,
            count: 1,
            pixels: vec![0; 4],
            user_data: ud("tileset", None),
        }],
        slices: vec![AseSlice {
            name: "s".into(),
            color: Some([0, 0, 255, 255]),
            text: "slice".into(),
            maps: ud("", None).expect("ud").maps,
            keys: vec![AseSliceKey {
                frame: 0,
                w: 2,
                h: 2,
                ..AseSliceKey::default()
            }],
        }],
        user_data: ud("sprite", Some([9, 9, 9, 9])),
        external_files: vec![AseExternalFile {
            id: 1,
            kind: 3,
            name: "ph2d.test".into(),
        }],
        ..AseFile::default()
    };
    let bytes = write_file(&file).expect("escreve");
    let mut back = parse_file(&bytes).expect("lê");
    // A tag sem user data volta com um vazio (o n.º 0x2020 é da tag n): o mesmo que nada.
    assert_eq!(back.tags[0].user_data, Some(AseUserData::default()));
    back.tags[0].user_data = None;
    assert_eq!(back, file);
    assert_eq!(write_file(&back).expect("regrava"), bytes);
}

/// Mete um chunk `0x2006` (cel extra, 36 bytes de corpo) logo a seguir ao 1.º cel do 1.º quadro.
fn insert_cel_extra(bytes: &[u8]) -> Vec<u8> {
    let u32_at = |b: &[u8], o: usize| u32::from_le_bytes(b[o..o + 4].try_into().expect("4"));
    let frame = 128;
    let frame_len = u32_at(bytes, frame) as usize;
    let mut at = frame + 16;
    let cel_end = loop {
        assert!(at < frame + frame_len, "o quadro não tem cel");
        let len = u32_at(bytes, at) as usize;
        let kind = u16::from_le_bytes([bytes[at + 4], bytes[at + 5]]);
        at += len;
        if kind == 0x2005 {
            break at;
        }
    };
    let mut extra = 42u32.to_le_bytes().to_vec();
    extra.extend_from_slice(&0x2006u16.to_le_bytes());
    extra.extend_from_slice(&1u32.to_le_bytes());
    extra.extend_from_slice(&[0; 32]);
    let mut out = [&bytes[..cel_end], &extra, &bytes[cel_end..]].concat();
    let bump32 = |b: &mut Vec<u8>, o: usize, d: u32| {
        let v = u32::from_le_bytes(b[o..o + 4].try_into().expect("4")) + d;
        b[o..o + 4].copy_from_slice(&v.to_le_bytes());
    };
    bump32(&mut out, 0, 42);
    bump32(&mut out, frame, 42);
    let old = u16::from_le_bytes([out[frame + 6], out[frame + 7]]) + 1;
    out[frame + 6..frame + 8].copy_from_slice(&old.to_le_bytes());
    bump32(&mut out, frame + 12, 1);
    out
}

#[test]
fn a_cel_extra_between_the_cel_and_its_user_data_keeps_the_user_data_on_the_cel() {
    let file = AseFile {
        width: 4,
        height: 4,
        depth: 32,
        layers: vec![AseLayer {
            name: "L".into(),
            flags: 3,
            opacity: 255,
            ..AseLayer::default()
        }],
        frames: vec![AseFileFrame {
            duration_ms: 100,
            cels: vec![AseCel {
                layer: 0,
                opacity: 255,
                body: AseCelBody::Image {
                    w: 1,
                    h: 1,
                    pixels: vec![1, 2, 3, 255],
                },
                user_data: ud("cel", Some([4, 3, 2, 1])),
                x: 0,
                y: 0,
                z: 0,
            }],
            palette: None,
        }],
        ..AseFile::default()
    };
    let bytes = insert_cel_extra(&write_file(&file).expect("escreve"));
    let back = parse_file(&bytes).expect("lê");
    assert_eq!(
        back.frames[0].cels[0].user_data,
        file.frames[0].cels[0].user_data
    );
}
