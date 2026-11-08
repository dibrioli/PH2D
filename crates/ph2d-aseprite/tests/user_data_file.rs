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
