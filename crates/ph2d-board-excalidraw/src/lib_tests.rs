//! O `.excalidraw` de ida e volta (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 X1–X4). As fixtures
//! são SAÍDAS do oráculo Excalidraw 0.18.1 (`ferramentas/excalidraw_oracle/saidas/w8_*`, com
//! `meta.json`): o nosso ficheiro depois do `restore` + `serializeAsJSON` dele, as cenas que ele
//! embebe no PNG e no SVG, e um ficheiro DELE com um elemento de cada tipo.

use super::*;
use ph2d_board_model::{
    Anchor, Board, BoardDoc, BoardOp, BoardSet, Connector, Element, ElementId, End, Frame, Ink,
    Pen, Rgba, RichText, Route, Shape, ShapeType, Stamp, Style, Tag,
};

const ORACLE: &str = "../../docs/MiroClone/ferramentas/excalidraw_oracle";
const INPUT: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w8_ph2d_quadro.excalidraw"
);
const RESTORED: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w8_ph2d_quadro.serialized.excalidraw"
);
const EMBED_PNG: &[u8] = include_bytes!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w8_ph2d_quadro.embed.png"
);
const EMBED_SVG: &[u8] = include_bytes!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w8_ph2d_quadro.embed.svg"
);
const THEIRS: &str = include_str!(
    "../../../docs/MiroClone/ferramentas/excalidraw_oracle/saidas/w8_tipos.serialized.excalidraw"
);

const INK: Rgba = Rgba([30, 30, 30, 255]);

fn put(b: &mut Board, el: Element) -> ElementId {
    let id = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    id
}

fn shape(b: &mut Board, kind: ShapeType, text: &str, bx: [f64; 4]) -> Element {
    let fill = if kind.is_note() {
        Some(Rgba([255, 249, 177, 255]))
    } else {
        Some(Rgba([165, 216, 255, 255]))
    };
    Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Shape {
            kind,
            style: Style::new(fill, (!kind.is_note()).then_some(INK), INK),
            text: RichText::plain(text),
        },
        bx,
    )
}

/// O quadro de amostra: dois frames (um dentro do outro), uma nota com etiqueta, votos e carimbo,
/// um rectângulo arredondado em rascunho, um cilindro (que o Excalidraw não tem) rodado, um grupo,
/// um travado, uma seta curva com rótulo, um cotovelo com pontos fixos, uma recta solta e um traço.
fn sample() -> Board {
    let mut set = BoardSet::default();
    let a = set.create("Launch".into());
    let b = set.get_mut(a).unwrap();
    let fr = |b: &mut Board, name: &str, bx| {
        Element::new_frame(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Frame {
                name: name.into(),
                style: Style::new(Some(Rgba([255; 4])), None, INK),
            },
            bx,
        )
    };
    let el = fr(b, "Problem", [0.0, 0.0, 900.0, 600.0]);
    let outer = put(b, el);
    let mut el = fr(b, "Detail", [500.0, 300.0, 350.0, 250.0]);
    el.frame = Some(outer);
    let inner = put(b, el);
    let mut n = shape(
        b,
        ShapeType::Sticky,
        "Slow builds",
        [40.0, 40.0, 199.0, 199.0],
    );
    n.frame = Some(outer);
    n.set_votes(3);
    n.add_stamp(Stamp::Heart, 2);
    n.tags_mut().push(Tag {
        name: "urgent".into(),
        color: Rgba([240, 147, 157, 255]),
    });
    let note = put(b, n);
    let mut r = shape(
        b,
        ShapeType::Rectangle,
        "Plan\nship it",
        [300.0, 60.0, 180.0, 100.0],
    );
    r.frame = Some(outer);
    if let Some(s) = r.shape_mut() {
        s.style.round = true;
        s.style.sketch = true;
        s.style.dash = ph2d_board_model::Dash::Dashed;
    }
    r.groups_mut().push(ph2d_board_model::GroupId(7));
    let rect = put(b, r);
    let mut c = shape(b, ShapeType::Cylinder, "DB", [560.0, 340.0, 120.0, 140.0]);
    c.frame = Some(inner);
    c.angle = 0.3;
    c.groups_mut().push(ph2d_board_model::GroupId(7));
    let cyl = put(b, c);
    let mut lk = shape(b, ShapeType::Ellipse, "Locked", [60.0, 400.0, 160.0, 90.0]);
    lk.set_locked(true);
    let locked = put(b, lk);
    let mut cv = Connector::new(
        End::Bound {
            target: note,
            anchor: Anchor::Center,
        },
        End::Bound {
            target: rect,
            anchor: Anchor::Center,
        },
        Route::Curved,
        Style::new(None, Some(INK), INK),
    );
    cv.label = "next".into();
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), cv);
    put(b, el);
    let elbow = Connector::new(
        End::Bound {
            target: rect,
            anchor: Anchor::Fixed([1.0, 0.5]),
        },
        End::Bound {
            target: cyl,
            anchor: Anchor::Fixed([0.5, 0.0]),
        },
        Route::Elbow,
        Style::new(None, Some(INK), INK),
    );
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), elbow);
    put(b, el);
    let mut free = Connector::new(
        End::Free([1000.0, 100.0]),
        End::Bound {
            target: locked,
            anchor: Anchor::Center,
        },
        Route::Straight,
        Style::new(None, Some(INK), INK),
    );
    free.heads = [
        ph2d_board_model::Head::Bar,
        ph2d_board_model::Head::Triangle,
    ];
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), free);
    put(b, el);
    let pts: Vec<[f64; 3]> = (0..12)
        .map(|i| {
            let t = f64::from(i) / 11.0;
            [300.0 + 200.0 * t, 520.0 + 20.0 * (t * 6.0).sin(), 0.5]
        })
        .collect();
    let pen = Style {
        stroke_width: 3.0,
        ..Style::new(None, Some(Rgba([66, 98, 255, 255])), INK)
    };
    let (ink, bx) = Ink::from_world(&pts, pen, Pen::Pen, false);
    let mut stroke = Element::new_ink(b.doc.mint_id(), b.doc.z_on_top(), ink, bx);
    stroke.frame = Some(outer);
    put(b, stroke);
    b.clone()
}

/// Os elementos vivos, comparáveis: sem a chave de z (a ordem é a do array) nem a versão (o `Put` da
/// leitura conta mais uma).
fn norm(doc: &BoardDoc) -> Vec<Element> {
    doc.live_in_z_order()
        .into_iter()
        .map(|e| {
            let mut e = e.clone();
            e.z = ph2d_board_model::FracKey::between(None, None);
            e.version = 0;
            e
        })
        .collect()
}

/// Grava a ENTRADA do oráculo (`entradas/w8_ph2d_quadro.excalidraw`). Corre-se à mão quando o escritor
/// muda — e a seguir o oráculo (README §«W8»), que regrava as `saidas/w8_ph2d_quadro.*`.
#[test]
#[ignore = "grava a entrada do oráculo (à mão, quando o escritor muda)"]
fn grava_a_entrada_do_oraculo() {
    let path = format!(
        "{}/{ORACLE}/entradas/w8_ph2d_quadro.excalidraw",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::write(path, write(&sample())).unwrap();
}

/// ⭐ X4: o escritor é determinístico e continua a escrever a ENTRADA que o oráculo correu.
#[test]
fn our_writer_still_writes_the_oracle_input() {
    assert_eq!(
        write(&sample()),
        INPUT,
        "o escritor mudou: corra o oráculo outra vez"
    );
}

/// ⭐⭐ X2: o nosso quadro → `.excalidraw` → o Excalidraw (`restore` + `serializeAsJSON`) → o nosso
/// leitor = o MESMO quadro (os mesmos ids, o mesmo tudo; a ordem de z pela ordem do array).
#[test]
fn the_excalidraw_round_trip_returns_our_board() {
    let back = read(RESTORED).unwrap();
    assert!(back.skipped.is_empty(), "{:?}", back.skipped);
    let ours = sample();
    let (a, b) = (norm(&ours.doc), norm(&back.doc));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x, y, "o elemento {} não voltou igual", x.id.0);
    }
}

/// ⭐ X3: a cena EMBEBIDA no PNG e no SVG que o Excalidraw exporta abre e volta igual.
#[test]
fn the_embedded_scenes_round_trip_too() {
    let ours = norm(&sample().doc);
    for (name, bytes) in [("png", EMBED_PNG), ("svg", EMBED_SVG)] {
        let back = read(&scene_from_bytes(bytes).unwrap()).unwrap();
        assert_eq!(norm(&back.doc), ours, "{name}");
    }
    assert!(scene_from_bytes(b"\x89PNG\r\n\x1a\nnope").is_err());
    assert!(scene_from_bytes(b"<svg></svg>").is_err());
}

/// ⭐ X3: um ficheiro DO Excalidraw (um elemento de cada tipo, gravado por ele) entra pelo mapeamento
/// directo: formas, texto preso e solto, setas presas, linha, traço, frames com filhos; a imagem, o
/// embeddable e o iframe SALTAM e contam-se.
#[test]
fn an_excalidraw_file_opens_by_the_direct_mapping() {
    let imp = read(THEIRS).unwrap();
    assert_eq!(imp.skipped.get("image"), Some(&1));
    assert_eq!(imp.skipped.get("embeddable"), Some(&1));
    assert_eq!(imp.skipped.get("iframe"), Some(&1));
    let els = imp.doc.live_in_z_order();
    let shapes: Vec<_> = els.iter().filter_map(|e| e.shape()).collect();
    let boxed = shapes
        .iter()
        .find(|s| s.text.as_str() == "Olá caixa")
        .expect("o texto preso");
    assert_eq!(boxed.kind, ShapeType::Rectangle);
    assert!(boxed.style.round && boxed.style.sketch);
    assert_eq!(boxed.style.fill, Some(Rgba([0xa5, 0xd8, 0xff, 255])));
    let arrows: Vec<_> = els.iter().filter_map(|e| e.connector()).collect();
    assert!(
        arrows
            .iter()
            .any(|c| matches!(c.start, End::Bound { .. }) && matches!(c.end, End::Bound { .. })),
        "a seta presa às duas formas"
    );
    assert!(els.iter().any(|e| e.ink().is_some()), "o traço");
    let frames: Vec<_> = els.iter().filter(|e| e.as_frame().is_some()).collect();
    assert!(!frames.is_empty());
    assert!(
        els.iter().any(|e| e.frame == Some(frames[0].id)),
        "um filho no frame"
    );
}

/// As cores: `transparent`, `#rgb`, `#rrggbb`, `#rrggbbaa`; o resto não se percebe.
#[test]
fn colors_parse_and_print() {
    assert_eq!(parse_color("transparent"), Some(None));
    assert_eq!(parse_color("#fff"), Some(Some(Rgba([255; 4]))));
    assert_eq!(parse_color("#1e1e1e"), Some(Some(Rgba([30, 30, 30, 255]))));
    assert_eq!(parse_color("#00000080"), Some(Some(Rgba([0, 0, 0, 128]))));
    assert_eq!(parse_color("red"), None);
    assert_eq!(hex(Some(Rgba([0, 0, 0, 128]))), "#00000080");
    assert_eq!(hex(None), "transparent");
    for h in [
        ph2d_board_model::Head::None,
        ph2d_board_model::Head::Arrow,
        ph2d_board_model::Head::DiamondOpen,
        ph2d_board_model::Head::Bar,
    ] {
        assert_eq!(head_of(head_name(h)), h);
    }
}

/// ⭐⭐ X2: o que se EDITA no Excalidraw volta editado — mover, reescrever o texto de uma nota, mudar a
/// cor, duplicar (a cópia leva o nosso `customData` e ganha um id novo) e apagar —, e o resto continua
/// o nosso (a nota continua NOTA, com etiqueta e votos).
#[test]
fn edits_made_in_excalidraw_come_back() {
    let mut file: serde_json::Value = serde_json::from_str(RESTORED).unwrap();
    let els = file["elements"].as_array_mut().unwrap();
    let find = |els: &mut Vec<serde_json::Value>, id: &str| {
        els.iter().position(|e| e["id"] == id).unwrap()
    };
    let i = find(els, "ph2d-4");
    els[i]["x"] = serde_json::json!(350.0);
    let i = find(els, "ph2d-3-text");
    els[i]["originalText"] = serde_json::json!("Fast builds");
    let i = find(els, "ph2d-6");
    els[i]["backgroundColor"] = serde_json::json!("#ffc9c9");
    let mut copy = els[i].clone();
    copy["id"] = serde_json::json!("copy1");
    copy["x"] = serde_json::json!(400.0);
    copy["boundElements"] = serde_json::json!([]);
    els.push(copy);
    let i = find(els, "ph2d-10");
    els[i]["isDeleted"] = serde_json::json!(true);
    let back = read(&serde_json::to_string(&file).unwrap()).unwrap();
    let doc = &back.doc;
    assert_eq!(doc.get(ElementId(4)).unwrap().x, 350.0, "movido");
    let note = doc.get(ElementId(3)).unwrap();
    assert_eq!(note.shape().unwrap().text.as_str(), "Fast builds");
    assert_eq!(
        note.shape().unwrap().kind,
        ShapeType::Sticky,
        "continua nota"
    );
    assert_eq!((note.votes(), note.tags().len()), (3, 1));
    let ell = doc.get(ElementId(6)).unwrap();
    assert_eq!(
        ell.shape().unwrap().style.fill,
        Some(Rgba([0xff, 0xc9, 0xc9, 255]))
    );
    let copies: Vec<_> = doc
        .live()
        .filter(|e| e.shape().is_some_and(|s| s.kind == ShapeType::Ellipse))
        .collect();
    assert_eq!(copies.len(), 2, "a cópia entrou");
    let c = copies.iter().find(|e| e.id != ElementId(6)).unwrap();
    // O id do traço apagado (10) está livre no quadro novo: a cópia pode tê-lo — o que conta é ser ÚNICO.
    assert_eq!(c.x, 400.0, "no sítio dela");
    assert_eq!(
        doc.live().filter(|e| e.id == c.id).count(),
        1,
        "um id só dela"
    );
    assert!(
        doc.live().all(|e| e.ink().is_none()),
        "o traço apagado não entra"
    );
    assert!(back.skipped.is_empty());
}
