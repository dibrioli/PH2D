//! As imagens do quadro (W14): a chave por conteúdo, os bytes que gravam só o que está em uso, o
//! registo da imagem e o formato 11 que continua a abrir.

use std::collections::BTreeSet;

use super::*;
use crate::{BoardOp, BoardSet, Element, FORMAT_VERSION, Field, Rgba};

fn style() -> Style {
    Style::new(None, None, Rgba([0, 0, 0, 255]))
}

/// Um ficheiro de `n` bytes distintos (não precisa de ser uma imagem: o modelo não descodifica).
fn file(n: u8) -> ImageFile {
    ImageFile::new(vec![n; usize::from(n) + 1], [u32::from(n) + 2, 3])
}

/// Um quadro com uma imagem do ficheiro `f` na caixa `bx`; devolve o id do elemento.
fn put(b: &mut Board, f: ImageFile, bx: [f64; 4]) -> crate::ElementId {
    let id = b.put_file(f);
    let el = Element::new_image(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Image::new(id, style()),
        bx,
    );
    let eid = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    eid
}

/// O sha256 é o da norma (FIPS 180-2, «abc»), e os 64 dígitos são os do `fileId` que escrevemos.
#[test]
fn the_file_key_is_the_sha256_of_the_bytes() {
    assert_eq!(
        FileId::of(b"abc").hex(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

/// Os bytes partilhados gravam-se EXACTAMENTE como um `Vec<u8>` (o postcard não muda) e voltam iguais.
#[test]
fn a_blob_is_the_bytes_of_a_vec_in_postcard() {
    let v: Vec<u8> = (0..=255).collect();
    let b = Blob(v.clone().into());
    let enc = postcard::to_allocvec(&b).unwrap();
    assert_eq!(enc, postcard::to_allocvec(&v).unwrap());
    assert_eq!(postcard::from_bytes::<Blob>(&enc).unwrap(), b);
}

/// O MIME pelos bytes mágicos de cada formato que um navegador mostra; texto não é imagem.
#[test]
fn the_mime_comes_from_the_magic_bytes() {
    let svg = Some("image/svg+xml");
    let cases: [(&[u8], Option<&str>); 14] = [
        (b"\x89PNG\r\n\x1a\nxxxx", Some("image/png")),
        (&[0xff, 0xd8, 0xff, 0xe0], Some("image/jpeg")),
        (b"GIF89a....", Some("image/gif")),
        (b"RIFF\0\0\0\0WEBPVP8 ", Some("image/webp")),
        (b"BM\0\0\0\0", Some("image/bmp")),
        (b"\0\0\0\x1cftypavif", Some("image/avif")),
        // W15: um SVG é imagem (texto, depois do BOM, da declaração, de comentários e do DOCTYPE).
        (b"<svg xmlns=", svg),
        (b"\xef\xbb\xbf  <?xml version=\"1.0\"?>\n<!-- a -> b -->\n<!DOCTYPE svg>\n<svg>", svg),
        (b"<svg/>", svg),
        // O cabeçalho do Illustrator: um DOCTYPE com entidades (o 1.º `>` é o de um `<!ENTITY>`).
        (b"<?xml version=\"1.0\"?>\n<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"x.dtd\" [\n\t<!ENTITY ns_extend \"http://ns.adobe.com/Extensibility/1.0/\">\n]>\n<svg version=\"1.1\">", svg),
        (b"<svgx>", None),
        (b"<html><svg>", None),
        (b"<!-- sem fim <svg>", None),
        (b"hello", None),
    ];
    for (bytes, want) in cases {
        assert_eq!(mime_of(bytes), want, "{bytes:?}");
    }
}

/// Dois elementos da MESMA imagem guardam os bytes uma vez.
#[test]
fn the_same_bytes_are_stored_once() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.get_mut(a).unwrap();
    put(b, file(1), [0.0, 0.0, 10.0, 10.0]);
    put(b, file(1), [20.0, 0.0, 10.0, 10.0]);
    assert_eq!(b.files.len(), 1);
    assert_eq!(b.doc.live_len(), 2);
}

/// ⭐ Gravar leva só os ficheiros em uso: o de uma imagem apagada sai; o de uma imagem apagada que uma
/// VERSÃO ainda mostra fica; e o resto do quadro volta igual.
#[test]
fn saving_keeps_only_the_files_in_use_by_an_element_or_a_version() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.get_mut(a).unwrap();
    let kept = put(b, file(1), [0.0, 0.0, 10.0, 10.0]);
    let gone = put(b, file(2), [20.0, 0.0, 10.0, 10.0]);
    let in_version = put(b, file(3), [40.0, 0.0, 10.0, 10.0]);
    b.save_version("v".into(), 1);
    assert_eq!(b.team.versions[0].files.len(), 3);
    for id in [gone, in_version] {
        BoardOp::Delete(id).apply(&mut b.doc);
    }
    b.save_version("v2".into(), 2);
    b.team.versions.remove(0);
    let mut v = b.team.versions[0].clone();
    v.files.insert(file(3).id());
    b.team.versions[0] = v;
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    let got: BTreeSet<FileId> = back.boards()[0].files.keys().copied().collect();
    assert_eq!(got, BTreeSet::from([file(1).id(), file(3).id()]));
    let mut want = set.clone();
    want.get_mut(a).unwrap().files.remove(&file(2).id());
    assert_eq!(back, want);
    let _ = kept;
}

/// ⭐ Gate de desenho: com todos os ficheiros em uso, os bytes gravados são os do `BoardSet` inteiro
/// (o `BoardOut` de [`BoardSet::to_bytes`] tem os campos de [`Board`], pela mesma ordem).
#[test]
fn the_saved_set_is_the_set_with_its_files_in_use() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.get_mut(a).unwrap();
    put(b, file(4), [0.0, 0.0, 10.0, 10.0]);
    b.save_version("v".into(), 7);
    set.create("B".into());
    assert_eq!(
        set.to_bytes().unwrap(),
        postcard::to_allocvec(&set).unwrap()
    );
}

/// Uma versão não guarda bytes de imagem; restaurá-la tira-os do quadro de onde vem — só os que ela
/// mostra.
#[test]
fn restoring_a_version_takes_its_files_from_the_board() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.get_mut(a).unwrap();
    put(b, file(5), [0.0, 0.0, 10.0, 10.0]);
    b.save_version("v".into(), 1);
    put(b, file(6), [20.0, 0.0, 10.0, 10.0]);
    let v = b.team.versions[0].read().unwrap();
    assert!(v.files.is_empty(), "a versão não leva bytes");
    let r = set.insert_after(a, "R".into(), v.doc, v.sketch).unwrap();
    let got: Vec<FileId> = set.get(r).unwrap().files.keys().copied().collect();
    assert_eq!(got, [file(5).id()]);
}

/// O registo `Image` lê e escreve o ficheiro, o recorte e o espelho juntos; mudar o recorte é mudar
/// só esse registo; numa forma não existe.
#[test]
fn the_image_register_carries_file_crop_and_flip() {
    let mut a = Element::new_image(
        crate::ElementId(1),
        crate::FracKey::between(None, None),
        Image::new(file(1).id(), style()),
        [0.0, 0.0, 4.0, 2.0],
    );
    let mut b = a.clone();
    let i = b.image_mut().unwrap();
    i.crop = Some(Crop {
        x: 1.0,
        y: 0.0,
        w: 2.0,
        h: 1.0,
    });
    i.flip = [true, false];
    assert_eq!(a.changed_fields(&b), [Field::Image]);
    assert!(a.write(Field::Image, &b.read(Field::Image).unwrap()));
    assert_eq!(a, b);
    let mut shape = crate::Element::new_shape(
        crate::ElementId(2),
        crate::FracKey::between(None, None),
        crate::Shape {
            kind: crate::ShapeType::Rectangle,
            style: style(),
            text: crate::RichText::default(),
        },
        [0.0; 4],
    );
    assert_eq!(shape.read(Field::Image), None);
    assert!(!shape.write(Field::Image, &b.read(Field::Image).unwrap()));
}

/// O rectângulo da imagem natural que a caixa mostra: o recorte, ou ela toda.
#[test]
fn the_source_rect_is_the_crop_or_the_whole_image() {
    let mut i = Image::new(file(1).id(), style());
    assert_eq!(i.source([4, 2]), [0.0, 0.0, 4.0, 2.0]);
    i.crop = Some(Crop {
        x: 2.0,
        y: 0.0,
        w: 2.0,
        h: 2.0,
    });
    assert_eq!(i.source([4, 2]), [2.0, 0.0, 2.0, 2.0]);
}

/// ⭐ Um ficheiro do formato 11 (W13) continua a abrir — gravado pelo build de `72b26c071` (o último
/// do formato 11; teste temporário num checkout à parte que escreveu `to_bytes`, apagado): «Retro11»
/// com duas notas («ideia A», «ideia B»), um fio de 2 mensagens na A, a versão «v1» e uma votação
/// (3 + 1 votos, mostrada); «Outro» vazio. Regrava-se no formato actual, igual.
#[test]
fn a_format_11_file_still_opens_and_rewrites_in_the_current_format() {
    let set = BoardSet::from_bytes(V11_BYTES).expect("o formato 11 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Retro11");
    let texts: Vec<String> = b
        .doc
        .live_in_z_order()
        .iter()
        .map(|e| e.shape().unwrap().text.as_str().to_owned())
        .collect();
    assert_eq!(texts, ["ideia A", "ideia B"]);
    assert_eq!(b.team.threads[0].messages.len(), 2);
    assert_eq!(b.team.versions.len(), 1);
    assert!(b.team.versions[0].files.is_empty());
    assert_eq!(b.team.versions[0].read().unwrap().name, "Retro11");
    assert_eq!(b.team.polls.len(), 1);
    assert_eq!(b.team.shown, Some(b.team.polls[0].id));
    assert!(b.files.is_empty());
    assert_eq!(set.boards()[1].name, "Outro");
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        FORMAT_VERSION
    );
    assert_eq!(BoardSet::from_bytes(&bytes).unwrap(), set);
}

/// Gravados pelo build do formato 11 (`72b26c071`).
const V11_BYTES: &[u8] = include_bytes!("../fixtures/format_v11.bin");

/// ⭐ Um ficheiro do formato 12 (W14) continua a abrir, com as imagens sem filtro — gravado pelo build
/// de `3e45504e5b` (o último do formato 12; teste temporário que escreveu `to_bytes`, apagado):
/// «Fotos12» com duas imagens (a 1.ª do ficheiro 7, recortada `1,0 4×2`, espelhada na horizontal,
/// cantos redondos, opacidade 60; a 2.ª do ficheiro 9) e a versão «v1»; «Vazio».
#[test]
fn a_format_12_file_still_opens_with_its_images_unfiltered() {
    let set = BoardSet::from_bytes(V12_BYTES).expect("o formato 12 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Fotos12");
    let imgs: Vec<&Image> = b
        .doc
        .live_in_z_order()
        .iter()
        .map(|e| match &e.kind {
            ElementKind::Image(i) => &**i,
            _ => panic!("só imagens"),
        })
        .collect();
    assert_eq!(imgs.len(), 2);
    let a = imgs[0];
    assert_eq!(a.file, file(7).id());
    assert_eq!(a.crop, Some(Crop { x: 1.0, y: 0.0, w: 4.0, h: 2.0 }));
    assert_eq!(a.flip, [true, false]);
    assert!(a.style.round);
    assert_eq!(a.style.opacity, 60);
    assert_eq!(imgs[1].file, file(9).id());
    assert!(imgs.iter().all(|i| i.filter == ImageFilter::Original));
    assert_eq!(b.files.len(), 2);
    assert_eq!(b.team.versions.len(), 1);
    assert_eq!(b.team.versions[0].read().unwrap().doc.live_in_z_order().len(), 2);
    assert_eq!(set.boards()[1].name, "Vazio");
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        FORMAT_VERSION
    );
    assert_eq!(BoardSet::from_bytes(&bytes).unwrap(), set);
}

/// Gravados pelo build do formato 12 (`3e45504e5b`).
const V12_BYTES: &[u8] = include_bytes!("../fixtures/format_v12.bin");

/// O filtro viaja no registo da imagem (o par ao vivo): ler e escrever `Field::Image` leva-o.
#[test]
fn the_image_field_carries_the_filter() {
    let mut b = BoardSet::default();
    let id = b.create("A".into());
    let board = b.get_mut(id).unwrap();
    let eid = put(board, file(3), [0.0, 0.0, 4.0, 4.0]);
    let mut gray = board.doc.get(eid).unwrap().clone();
    if let ElementKind::Image(i) = &mut gray.kind {
        i.filter = ImageFilter::Gray;
    }
    let bytes = gray.read(Field::Image).unwrap();
    let mut plain = board.doc.get(eid).unwrap().clone();
    assert!(plain.write(Field::Image, &bytes));
    assert_eq!(plain.kind, gray.kind);
}

/// ⭐ Gate de desenho: o retrato magro é o quadro com os ficheiros vazios (os campos de [`Board`] pela
/// mesma ordem), e lê-se pelo leitor de sempre.
#[test]
fn the_lean_board_is_the_board_without_files() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.get_mut(a).unwrap();
    put(b, file(7), [0.0, 0.0, 10.0, 10.0]);
    b.save_version("v".into(), 3);
    let mut bare = b.clone();
    bare.files.clear();
    let lean = BoardSet::board_bytes_lean(b);
    assert_eq!(lean, BoardSet::board_bytes(&bare));
    assert_eq!(BoardSet::board_from_bytes(&lean).unwrap(), bare);
}
