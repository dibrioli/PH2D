//! As slices no `.ase` escrito aqui: flags por slice, chaves repetidas e a quem pertence o user data
//! (F3, `P11_oraculos_arquivo.md`). Fixtures da prova de mutação P11.

use ph2d_aseprite::{AseFile, AseFileFrame, AseSlice, AseSliceKey, parse_file, write_file};

fn base(slices: Vec<AseSlice>) -> AseFile {
    AseFile {
        width: 16,
        height: 16,
        depth: 32,
        frames: vec![AseFileFrame {
            duration_ms: 100,
            ..AseFileFrame::default()
        }],
        slices,
        ..AseFile::default()
    }
}

fn key(frame: u32, x: i32) -> AseSliceKey {
    AseSliceKey {
        frame,
        x,
        y: 1,
        w: 4,
        h: 3,
        ..AseSliceKey::default()
    }
}

/// As flags valem para a slice inteira: basta uma key com centro (ou pivot) para todas o gravarem, e
/// a que não tem lê-se sem ele.
#[test]
fn one_key_with_a_center_or_pivot_is_enough_for_the_slice_flags() {
    let mut com = key(0, 0);
    com.center = Some((1, 1, 2, 1));
    com.pivot = Some((3, 2));
    let sem = key(2, 5);
    let s = AseSlice {
        name: "a".into(),
        color: Some([1, 2, 3, 4]),
        keys: vec![com, sem],
        ..AseSlice::default()
    };
    let lido = parse_file(&write_file(&base(vec![s])).unwrap()).unwrap();
    let k = &lido.slices[0].keys;
    assert_eq!(k[0].center, Some((1, 1, 2, 1)));
    assert_eq!(k[0].pivot, Some((3, 2)));
    assert_eq!(k[1].center, None, "centro zero lê-se como sem centro");
    assert_eq!(k[1].pivot, Some((0, 0)), "sem pivot grava 0,0");
}

/// Duas keys no mesmo quadro: a última lida ganha, e fica uma só.
#[test]
fn two_keys_on_the_same_frame_keep_the_last_one_read() {
    let s = AseSlice {
        name: "a".into(),
        keys: vec![key(1, 5), key(1, 9)],
        ..AseSlice::default()
    };
    let lido = parse_file(&write_file(&base(vec![s])).unwrap()).unwrap();
    let k = &lido.slices[0].keys;
    assert_eq!(k.len(), 1);
    assert_eq!(k[0].x, 9);
}

/// Um user data que não vem logo a seguir a uma slice não lhe pinta a cor nem o texto.
#[test]
fn a_user_data_chunk_not_right_after_a_slice_does_not_touch_it() {
    let s = AseSlice {
        name: "a".into(),
        color: Some([1, 2, 3, 4]),
        text: "t".into(),
        keys: vec![key(0, 0)],
        ..AseSlice::default()
    };
    let mut b = write_file(&base(vec![s])).unwrap();
    // Um user data solto no fim do quadro 0, depois do user data da slice.
    let mut c = Vec::new();
    c.extend_from_slice(&(6_u32 + 4 + 1 + 2 + 4).to_le_bytes());
    c.extend_from_slice(&0x2020_u16.to_le_bytes());
    c.extend_from_slice(&3_u32.to_le_bytes());
    c.extend_from_slice(&1_u16.to_le_bytes());
    c.push(b'z');
    c.extend_from_slice(&[9, 9, 9, 9]);
    let tam = u32::from_le_bytes(b[128..132].try_into().unwrap()) + c.len() as u32;
    b[128..132].copy_from_slice(&tam.to_le_bytes());
    let n = u32::from_le_bytes(b[140..144].try_into().unwrap()) + 1;
    b[140..144].copy_from_slice(&n.to_le_bytes());
    b[134..136].copy_from_slice(&(n as u16).to_le_bytes());
    b.extend_from_slice(&c);
    let lido = parse_file(&b).unwrap();
    assert_eq!(lido.slices[0].color, Some([1, 2, 3, 4]));
    assert_eq!(lido.slices[0].text, "t");
}
