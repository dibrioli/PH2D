//! O formato 16 abre no 17 (a escala de nascença e a tinta limpa de nascença) e o 17 guarda-as.

use crate::{
    Anchor, BirthScale, Board, BoardOp, BoardSet, Caption, Connector, Crop, DEFAULT_INK, Element,
    ElementKind, Embed, End, FORMAT_VERSION, Frame, Image, ImageFile, ImageFilter, Ink, Pen, Pin,
    Rgba, RichText, Route, Shape, ShapeType, Style, Video, Who,
};

/// Gravados pelo build do formato 16 (`b832018d9e`; um exemplo num checkout à parte que escreveu
/// `to_bytes`, apagado): o quadro «Sketch16» que [`sketch16`] refaz com as structs de hoje.
const V16_BYTES: &[u8] = include_bytes!("../fixtures/format_v16.bin");

/// O quadro do fixture, passo a passo como o build do 16 o fez: uma elipse em rascunho, uma seta
/// presa a ela (cotovelo, um texto, um ponto de passagem), um traço do marcador com pressão, um
/// frame, uma imagem (recortada, espelhada, cinzenta, opacidade 60), um vídeo, um fio com menção e a
/// versão «v1».
fn sketch16(b: &mut Board) {
    b.sketch = true;
    let ink_c = Rgba([20, 40, 60, 255]);
    let style = |fill, stroke, w| {
        let mut s = Style::new(fill, stroke, Rgba(DEFAULT_INK));
        if let Some(w) = w {
            s.stroke_width = w;
        }
        s
    };
    let mut st = style(Some(Rgba([255, 200, 0, 255])), Some(ink_c), Some(2.0));
    st.sketch = true;
    st.look.cartoonist = true;
    let shape = Shape {
        kind: ShapeType::Ellipse,
        style: st,
        text: RichText::plain("forma"),
    };
    let s = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        shape,
        [10.0, 20.0, 100.0, 60.0],
    );
    let target = s.id;
    BoardOp::Put(s).apply(&mut b.doc);
    let start = End::Bound {
        target,
        anchor: Anchor::Center,
    };
    let cst = style(None, Some(ink_c), Some(1.5));
    let mut c = Connector::new(start, End::Free([300.0, 200.0]), Route::Elbow, cst);
    c.captions = vec![Caption::new("sim", 0.25)];
    c.insert_waypoint(0, [200.0, 120.0]);
    let a = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
    BoardOp::Put(a).apply(&mut b.doc);
    let pts = [[0.0, 0.0, 0.5], [40.0, 10.0, 0.8], [80.0, 0.0, 0.3]];
    let ist = style(None, Some(ink_c), Some(3.0));
    let (ink, bx) = Ink::from_world(&pts, ist, Pen::Highlighter, true);
    let i = Element::new_ink(b.doc.mint_id(), b.doc.z_on_top(), ink, bx);
    BoardOp::Put(i).apply(&mut b.doc);
    let fr = Frame {
        name: "Quadro".into(),
        style: style(Some(Rgba([250, 250, 250, 255])), None, None),
    };
    let f = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        fr,
        [-50.0, -50.0, 500.0, 400.0],
    );
    BoardOp::Put(f).apply(&mut b.doc);
    let file = b.put_file(ImageFile::new(vec![7u8; 8], [4, 2]));
    let mut mst = style(None, None, None);
    (mst.opacity, mst.round) = (60, true);
    let mut img = Image::new(file, mst);
    img.crop = Some(Crop {
        x: 1.0,
        y: 0.0,
        w: 2.0,
        h: 2.0,
    });
    (img.flip, img.filter) = ([true, false], ImageFilter::Gray);
    let m = Element::new_image(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        img,
        [0.0, 300.0, 80.0, 40.0],
    );
    BoardOp::Put(m).apply(&mut b.doc);
    let emb = Embed {
        link: "https://youtu.be/dQw4w9WgXcQ?t=42".into(),
        video: Video::YouTube {
            id: "dQw4w9WgXcQ".into(),
            start: 42,
            vertical: false,
        },
        style: style(None, Some(ink_c), None),
    };
    let e = Element::new_embed(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        emb,
        [200.0, 300.0, 160.0, 90.0],
    );
    BoardOp::Put(e).apply(&mut b.doc);
    let who = [Who::Person("Enio".into())];
    let at = 1_760_000_000;
    b.team
        .open_calling(Pin::Point([5.0, 5.0]), "Rui", "@Enio olha", &who, at)
        .unwrap();
    b.save_version("v1".into(), at + 100);
}

/// A escala e a tinta limpa de cada elemento (`None` onde o elemento não tem traço).
fn births(b: &Board) -> Vec<(BirthScale, Option<bool>)> {
    b.doc
        .live_in_z_order()
        .iter()
        .map(|e| match &e.kind {
            ElementKind::Shape(s) => (s.style.birth, None),
            ElementKind::Connector(c) => (c.style.birth, None),
            ElementKind::Ink(i) => (i.style.birth, Some(i.clean)),
            ElementKind::Frame(f) => (f.style.birth, None),
            ElementKind::Image(i) => (i.style.birth, None),
            ElementKind::Embed(e) => (e.style.birth, None),
        })
        .collect()
}

/// ⭐ Um ficheiro do formato 16 abre no 17 com tudo nascido a zoom 1 (`BirthScale(0)`) e todo traço
/// livre (`clean: false`) — e o resto IGUAL ao que o build do 16 gravou: as seis variantes do
/// elemento (forma, seta, traço, frame, imagem, vídeo), o fio com a menção, os ficheiros e a versão,
/// que também lê pelo leitor do 16. Regrava-se no 17 e volta igual.
#[test]
fn a_format_16_file_opens_born_at_zoom_one_with_every_ink_free() {
    assert_eq!(postcard::take_from_bytes::<u32>(V16_BYTES).unwrap().0, 16);
    let set = BoardSet::from_bytes(V16_BYTES).expect("o formato 16 lê-se");
    let b = &set.boards()[0];
    let mut want = BoardSet::default();
    let id = want.create("Sketch16".into());
    sketch16(want.get_mut(id).unwrap());
    let w = &want.boards()[0];
    assert_eq!((b.id, &b.name, b.sketch), (w.id, &w.name, w.sketch));
    assert_eq!(b.doc, w.doc, "as seis variantes, campo a campo");
    assert_eq!(b.team.threads, w.team.threads, "o fio, com a menção");
    assert_eq!(b.files, w.files);
    let kinds = births(b);
    assert_eq!(kinds.len(), 6);
    assert!(
        kinds
            .iter()
            .all(|&(birth, clean)| birth == BirthScale::default() && clean != Some(true)),
        "{kinds:?}"
    );
    assert_eq!(
        kinds.iter().filter(|k| k.1.is_some()).count(),
        1,
        "um traço"
    );
    let v = &b.team.versions[0];
    assert_eq!(v.name, "v1");
    assert_eq!(v.read().expect("a versão do 16 abre").doc, w.doc);
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        FORMAT_VERSION
    );
    assert_eq!(BoardSet::from_bytes(&bytes).unwrap(), set);
}

/// O formato 17 vai e volta com a escala de nascença de uma forma e a tinta limpa de um traço.
#[test]
fn format_17_keeps_the_birth_scale_and_the_clean_ink() {
    let mut set = BoardSet::default();
    let id = set.create("Sketch17".into());
    let b = set.get_mut(id).unwrap();
    sketch16(b);
    let mut born = 0;
    for e in b.doc.elements.values_mut() {
        match &mut e.kind {
            ElementKind::Shape(s) => (s.style.birth, born) = (BirthScale(7), born + 1),
            ElementKind::Ink(i) => (i.clean, born) = (true, born + 1),
            _ => {}
        }
    }
    assert_eq!(born, 2);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    let kinds = births(&back.boards()[0]);
    assert!(kinds.contains(&(BirthScale(7), None)), "{kinds:?}");
    assert!(
        kinds.contains(&(BirthScale::default(), Some(true))),
        "{kinds:?}"
    );
}
