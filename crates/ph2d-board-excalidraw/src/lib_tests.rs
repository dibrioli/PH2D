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

/// A mesma árvore JSON: chaves, ordem, textos e tipos iguais; os números a 1e-12 relativos. ⚠️ Medido no
/// fecho da W8: o contorno de uma forma (seno/cosseno) sai com o último dígito diferente noutro
/// perfil de compilação (`114.47324588220454` contra `…46` no nextest do portão) — o byte a byte
/// acusava o escritor por um ULP da conta.
fn same_tree(a: &serde_json::Value, b: &serde_json::Value, at: &str) -> Result<(), String> {
    use serde_json::Value as V;
    match (a, b) {
        (V::Number(x), V::Number(y)) => {
            let (x, y) = (
                x.as_f64().unwrap_or(f64::NAN),
                y.as_f64().unwrap_or(f64::NAN),
            );
            if x == y || (x - y).abs() <= 1e-12 * x.abs().max(y.abs()) {
                Ok(())
            } else {
                Err(format!("{at}: {x} ≠ {y}"))
            }
        }
        (V::Array(x), V::Array(y)) if x.len() == y.len() => x
            .iter()
            .zip(y)
            .enumerate()
            .try_for_each(|(i, (p, q))| same_tree(p, q, &format!("{at}[{i}]"))),
        (V::Object(x), V::Object(y)) if x.keys().eq(y.keys()) => x
            .iter()
            .zip(y.values())
            .try_for_each(|((k, p), q)| same_tree(p, q, &format!("{at}.{k}"))),
        _ if a == b => Ok(()),
        _ => Err(format!("{at}: {a} ≠ {b}")),
    }
}

/// A comparação da árvore tem tolerância de ARREDONDAMENTO, nunca de valor: um ULP passa, 1e-3 não.
#[test]
fn the_tree_comparison_forgives_only_rounding() {
    let n = |v: f64| serde_json::json!({ "x": [v] });
    let x = 114.473_245_882_204_54_f64;
    assert!(same_tree(&n(x), &n(f64::from_bits(x.to_bits() + 1)), "").is_ok());
    assert!(same_tree(&n(x), &n(x + 1e-3), "").is_err());
    assert!(
        same_tree(
            &serde_json::json!({"a": 1}),
            &serde_json::json!({"b": 1}),
            ""
        )
        .is_err()
    );
}

/// ⭐ X4: o escritor é determinístico (byte a byte no mesmo build) e continua a escrever a ENTRADA que
/// o oráculo correu (a mesma árvore, a menos do último dígito de uma conta de vírgula flutuante).
#[test]
fn our_writer_still_writes_the_oracle_input() {
    let ours = write(&sample());
    assert_eq!(ours, write(&sample()), "o escritor não é determinístico");
    let parse = |s: &str| serde_json::from_str::<serde_json::Value>(s).expect("JSON");
    if let Err(e) = same_tree(&parse(&ours), &parse(INPUT), "") {
        panic!("o escritor mudou ({e}): corra o oráculo outra vez");
    }
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
/// iframe SALTAM e contam-se (a imagem sem ficheiro); W18: o `embeddable` com um link do YouTube entra como
/// VÍDEO.
#[test]
fn an_excalidraw_file_opens_by_the_direct_mapping() {
    let imp = read(THEIRS).unwrap();
    assert_eq!(imp.skipped.get("image"), Some(&1));
    assert_eq!(imp.skipped.get("embeddable"), None, "W18: é um vídeo");
    assert_eq!(imp.doc.live().filter(|e| e.embed().is_some()).count(), 1);
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
/// o nosso (a nota continua NOTA, com etiqueta).
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
    let i = find(els, "ph2d-9");
    els[i]["backgroundColor"] = serde_json::json!("#ffc9c9");
    let mut copy = els[i].clone();
    copy["id"] = serde_json::json!("copy1");
    copy["x"] = serde_json::json!(400.0);
    copy["boundElements"] = serde_json::json!([]);
    els.push(copy);
    let i = find(els, "ph2d-13");
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
    assert_eq!(note.tags().len(), 1);
    let ell = doc.get(ElementId(9)).unwrap();
    assert_eq!(
        ell.shape().unwrap().style.fill,
        Some(Rgba([0xff, 0xc9, 0xc9, 255]))
    );
    let copies: Vec<_> = doc
        .live()
        .filter(|e| e.shape().is_some_and(|s| s.kind == ShapeType::Ellipse))
        .collect();
    assert_eq!(copies.len(), 2, "a cópia entrou");
    let c = copies.iter().find(|e| e.id != ElementId(9)).unwrap();
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

fn edited(f: impl FnOnce(&mut Vec<serde_json::Value>)) -> Import {
    let mut file: serde_json::Value = serde_json::from_str(RESTORED).unwrap();
    f(file["elements"].as_array_mut().unwrap());
    read(&serde_json::to_string(&file).unwrap()).unwrap()
}

fn at(els: &[serde_json::Value], id: &str) -> usize {
    els.iter().position(|e| e["id"] == id).unwrap()
}

/// ⭐ X2 — uma cópia de uma forma que viaja como `line` (o cilindro), feita LÁ com o texto mudado, fica
/// com o SEU texto e o original com o dele (a auditoria W8 viu-os trocados pelo `text_of` herdado).
#[test]
fn a_copy_of_a_line_shape_keeps_its_own_text() {
    let imp = edited(|els| {
        let (l, t) = (at(els, "ph2d-8"), at(els, "ph2d-8-text"));
        let (mut line, mut text) = (els[l].clone(), els[t].clone());
        line["id"] = serde_json::json!("copyline");
        line["x"] = serde_json::json!(line["x"].as_f64().unwrap() + 500.0);
        line["groupIds"] = serde_json::json!(["fresh-group"]);
        text["id"] = serde_json::json!("copytext");
        text["originalText"] = serde_json::json!("Copy");
        text["text"] = serde_json::json!("Copy");
        text["groupIds"] = serde_json::json!(["fresh-group"]);
        // Uma 2.ª cópia noutro grupo: cada texto fica com o anfitrião do SEU grupo (prova de mutação
        // W8: com uma cópia só, «o primeiro grupo» e «o grupo dele» davam o mesmo).
        let (mut line2, mut text2) = (line.clone(), text.clone());
        line2["id"] = serde_json::json!("copyline2");
        line2["x"] = serde_json::json!(line["x"].as_f64().unwrap() + 500.0);
        line2["groupIds"] = serde_json::json!(["another-group"]);
        text2["id"] = serde_json::json!("copytext2");
        text2["originalText"] = serde_json::json!("Second");
        text2["text"] = serde_json::json!("Second");
        text2["groupIds"] = serde_json::json!(["another-group"]);
        els.push(line);
        els.push(text);
        els.push(line2);
        els.push(text2);
    });
    let cyl: Vec<String> = imp
        .doc
        .live_in_z_order()
        .iter()
        .filter_map(|e| {
            e.shape()
                .filter(|s| s.kind == ShapeType::Cylinder)
                .map(|s| s.text.as_str().to_owned())
        })
        .collect();
    assert_eq!(cyl, ["DB", "Copy", "Second"]);
}

/// ⭐ X2 — o tracejado e os cantos mudados lá voltam; um redimensionar de uma forma `line` escala-a;
/// pontos novos numa RECTA são pontos de ajuste; e a opacidade fora de faixa é a de nascença (100).
#[test]
fn style_resize_and_straight_points_edited_there_come_back() {
    let imp = edited(|els| {
        let r = at(els, "ph2d-4");
        els[r]["strokeStyle"] = serde_json::json!("dotted");
        els[r]["roundness"] = serde_json::Value::Null;
        let l = at(els, "ph2d-8");
        let w = els[l]["width"].as_f64().unwrap();
        els[l]["width"] = serde_json::json!(w * 2.0);
        let a = at(els, "ph2d-12");
        let pts = els[a]["points"].as_array().unwrap().clone();
        let last = pts.last().unwrap().clone();
        els[a]["points"] =
            serde_json::json!([pts[0], [last[0].as_f64().unwrap() / 2.0, 0.0], last]);
        let e = at(els, "ph2d-9");
        els[e]["opacity"] = serde_json::json!(-5);
    });
    let doc = &imp.doc;
    let rect = doc.get(ElementId(4)).unwrap().shape().unwrap();
    assert_eq!(rect.style.dash, ph2d_board_model::Dash::Dotted);
    assert!(!rect.style.round, "os cantos tirados lá");
    assert!(
        (doc.get(ElementId(8)).unwrap().w - 240.0).abs() < 1.0,
        "o cilindro escalou: {}",
        doc.get(ElementId(8)).unwrap().w
    );
    assert_eq!(
        doc.get(ElementId(12))
            .unwrap()
            .connector()
            .unwrap()
            .waypoints
            .len(),
        1,
        "o ponto novo"
    );
    assert_eq!(doc.get(ElementId(9)).unwrap().style().opacity, 100);
}

/// ⭐ Decisão do dono (10/10) — a nota num quadro em rascunho tem a letra à mão e o papel LISO: sai
/// com `roughness 0` (o papel que se vê) e a Virgil, e volta em rascunho com a letra do modo.
#[test]
fn a_sketched_note_travels_with_smooth_paper_and_the_hand_letter() {
    let mut set = BoardSet::default();
    let a = set.create("Notes".into());
    let b = set.get_mut(a).unwrap();
    let mut n = shape(b, ShapeType::Sticky, "oi", [0.0, 0.0, 200.0, 200.0]);
    n.style_mut().sketch = true;
    let id = put(b, n);
    let file: serde_json::Value = serde_json::from_str(&write(b)).unwrap();
    let els = file["elements"].as_array().unwrap();
    for e in els {
        assert_eq!(e["roughness"], 0, "{}: o papel treme", e["id"]);
    }
    let text = &els[at(els, &format!("ph2d-{}-text", id.0))];
    assert_eq!(text["fontFamily"], 1, "a letra do modo é a Virgil");
    let back = read(&serde_json::to_string(&file).unwrap()).unwrap();
    let st = back.doc.get(id).unwrap().style().clone();
    assert!(st.sketch, "a nota voltou final");
    assert_eq!(st.look.letter, ph2d_board_model::Letter::Auto);
}

/// ⭐ W13 (ADR-0183 §9): um `.excalidraw` escrito ANTES do histórico trazia os votos no nosso elemento
/// (`customData.ph2d.element.org.votes`, o que o escritor do formato 10 punha): voltam no `Import` para
/// quem cria o quadro os guardar como UMA sessão; a cópia feita lá (id novo) não os leva.
#[test]
fn an_old_file_brings_its_votes_back_for_one_session() {
    let mut file: serde_json::Value = serde_json::from_str(RESTORED).unwrap();
    let els = file["elements"].as_array_mut().unwrap();
    let i = els.iter().position(|e| e["id"] == "ph2d-3").unwrap();
    els[i]["customData"]["ph2d"]["element"]["org"]["votes"] = serde_json::json!(3);
    let mut copy = els[i].clone();
    copy["id"] = serde_json::json!("copy1");
    copy["boundElements"] = serde_json::json!([]);
    els.push(copy);
    let back = read(&serde_json::to_string(&file).unwrap()).unwrap();
    assert_eq!(
        back.votes.into_iter().collect::<Vec<_>>(),
        [(ElementId(3), 3)]
    );
    let now = read(RESTORED).unwrap();
    let carried =
        |v: &serde_json::Value| v["customData"]["ph2d"]["element"]["org"]["votes"].as_u64();
    let restored: serde_json::Value = serde_json::from_str(RESTORED).unwrap();
    let old_votes: u64 = restored["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(carried)
        .sum();
    assert_eq!(
        now.votes.values().map(|&n| u64::from(n)).sum::<u64>(),
        old_votes,
        "o que o ficheiro traz, nem mais nem menos"
    );
}
