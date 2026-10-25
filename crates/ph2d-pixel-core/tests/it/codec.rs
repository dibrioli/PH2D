//! O formato gravado: reler dá o mesmo documento, e regravar dá os mesmos bytes.

use std::collections::BTreeMap;

use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::{
    Background, CodecError, ColorMode, History, NewSprite, Paint, Sprite, Stroke,
};

fn drawn(mode: ColorMode, background: Background) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 32,
        height: 24,
        mode,
        background,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let v = s.value_of_rgba([0x5b, 0x6e, 0xe1, 0xff]);
    let mut st = Stroke::begin(&s, l, f, v, Paint::default(), (-3, 1));
    st.to(&s, 17, 20);
    let op = st.op(&s).expect("op");
    History::default().commit(&mut s, op).expect("commit");
    s
}

fn library() -> BTreeMap<u64, Sprite> {
    let mut lib = BTreeMap::new();
    lib.insert(7, drawn(ColorMode::Rgba, Background::Transparent));
    lib.insert(3, drawn(ColorMode::Indexed, Background::Black));
    lib.insert(1 << 40, drawn(ColorMode::Gray, Background::White));
    lib
}

#[test]
fn the_blob_roundtrips_to_the_same_document_and_the_same_bytes() {
    let lib = library();
    let bytes = write_library(&lib);
    let back = read_library(&bytes).expect("read");
    assert_eq!(back, lib);
    assert_eq!(write_library(&back), bytes, "regravar dá os mesmos bytes");
}

#[test]
fn an_empty_blob_is_an_empty_library() {
    assert!(read_library(&[]).expect("vazio").is_empty());
    let empty = write_library(&BTreeMap::new());
    assert!(read_library(&empty).expect("vazia").is_empty());
}

#[test]
fn a_foreign_major_or_a_cut_blob_is_refused_never_half_read() {
    let mut bytes = write_library(&library());
    let mut other = bytes.clone();
    other[9] = 2;
    assert_eq!(
        read_library(&other),
        Err(CodecError::MajorMismatch { found: 2 })
    );
    for cut in [10, 20, bytes.len() / 2, bytes.len() - 1] {
        assert!(read_library(&bytes[..cut]).is_err(), "corte em {cut}");
    }
    bytes[0] = b'X';
    assert_eq!(read_library(&bytes), Err(CodecError::BadMagic));
}

#[test]
fn an_unknown_field_is_skipped_inside_the_same_major() {
    let lib = library();
    let bytes = write_library(&lib);
    // Uma etiqueta 99 com 3 bytes no topo da biblioteca, antes do 0 final.
    let mut with_extra = bytes[..bytes.len() - 1].to_vec();
    with_extra.extend_from_slice(&[99, 3, 1, 2, 3, 0]);
    assert_eq!(read_library(&with_extra).expect("salta"), lib);
}
