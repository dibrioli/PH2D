//! W9 — o NOSSO lado do ficheiro (estudo 07 §6.2-5/6, M5, M7, M8, M10): a ida e volta com o que a W9
//! trouxe, as duas gramáticas que escrevemos e o que as duas versões fazem com elas, a nota, a caneta e
//! o aspecto que vêm do Excalidraw, e o ficheiro da W8 que continua a voltar como nosso. Módulo filho de
//! `fidelity_tests` (os ajudantes do oráculo são de lá).

use super::*;

fn put(b: &mut Board, el: Element) -> ph2d_board_model::ElementId {
    let id = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    id
}

/// Um quadro com o que a W9 trouxe: órbita, ponto fixo e centro; a curva dele; as pontas novas; o
/// aspecto (riscas, rugosidade 2, semente, texto à esquerda e em cima, letra fixa).
fn sample_w9() -> Board {
    let mut set = BoardSet::default();
    let a = set.create("W9".into());
    let b = set.get_mut(a).unwrap();
    let ink = Rgba([30, 30, 30, 255]);
    let box_at = |b: &mut Board, x: f64, look: bool| {
        let mut st = Style::new(Some(Rgba([165, 216, 255, 255])), Some(ink), ink);
        if look {
            st.sketch = true;
            st.look.cartoonist = true;
            st.look.fill_style = FillStyle::CrossHatch;
            st.look.align = TextAlign::Left;
            st.look.valign = VAlign::Top;
            st.look.letter = Letter::Sans;
        }
        let el = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Shape {
                kind: ShapeType::Rectangle,
                style: st,
                text: RichText::plain("caixa"),
            },
            [x, 0.0, 160.0, 90.0],
        );
        put(b, el)
    };
    let (r1, r2, r3) = (
        box_at(b, 0.0, true),
        box_at(b, 400.0, false),
        box_at(b, 800.0, false),
    );
    let mut seeded = b.doc.get(r1).unwrap().clone();
    seeded.set_imported_seed(Some(4242));
    put(b, seeded);
    let arrow = |b: &mut Board, s: End, e: End, route: Route, heads: [Head; 2]| {
        let mut c = Connector::new(s, e, route, Style::new(None, Some(ink), ink));
        c.heads = heads;
        let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
        put(b, el);
    };
    arrow(
        b,
        End::Bound {
            target: r1,
            anchor: Anchor::Orbit([0.75, 0.3]),
        },
        End::Bound {
            target: r2,
            anchor: Anchor::Fixed([0.0, 0.5]),
        },
        Route::Spline,
        [Head::TriangleOpen, Head::ZeroOrMany],
    );
    arrow(
        b,
        End::Bound {
            target: r2,
            anchor: Anchor::Center,
        },
        End::Bound {
            target: r3,
            anchor: Anchor::Orbit([0.5, 0.0]),
        },
        Route::Straight,
        [Head::One, Head::ExactlyOne],
    );
    let mut c = Connector::new(
        End::Free([0.0, 300.0]),
        End::Bound {
            target: r3,
            anchor: Anchor::Orbit([0.5, 1.0]),
        },
        Route::Spline,
        Style::new(None, Some(ink), ink),
    );
    c.waypoints = vec![[300.0, 200.0], [600.0, 380.0]];
    c.heads = [Head::OneOrMany, Head::ZeroOrOne];
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
    put(b, el);
    b.clone()
}

fn norm(doc: &ph2d_board_model::BoardDoc) -> Vec<Element> {
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

/// ⭐ §6.2-6 — a ida e volta (X2) com o que a W9 trouxe: nosso → `.excalidraw` → nosso = IGUAL.
#[test]
fn the_w9_board_round_trips() {
    let b = sample_w9();
    let back = read(&write(&b)).unwrap();
    assert!(back.skipped.is_empty());
    assert_eq!(norm(&back.doc), norm(&b.doc));
}

/// ⭐⭐ §6.2-5 (a nossa metade) — XV3: o que escrevemos fala as DUAS gramáticas — o ponto fixo é
/// `inside`, a órbita e o centro são `orbit`, o legado o do 0.18.1 ao re-ligar (W10, XV7) — e lido
/// como ficheiro DE FORA (sem o nosso `customData`) dá as mesmas âncoras; as pontas que gravamos são as
/// roteadas (com o vão), a verdade ao abrir (XB7); a curva dele sai com `roundness` e os pontos dela.
#[test]
fn xv3_we_write_both_grammars_and_a_stranger_reads_the_same_anchors() {
    let b = sample_w9();
    let file: Value = serde_json::from_str(&write(&b)).unwrap();
    let arrows: Vec<&Value> = file["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "arrow")
        .collect();
    let a0 = arrows[0];
    assert_eq!(a0["startBinding"]["mode"], "orbit");
    assert_eq!(a0["startBinding"]["fixedPoint"], json!([0.75, 0.3]));
    let r1 = b.doc.live_in_z_order()[0].clone();
    // O legado é o do 0.18.1 ao re-ligar (W10, XV7): a ponta em órbita a `g` da face ⇒ `gap` = `g`; o
    // ponto fixo NO contorno ⇒ o menor `|gap|` (o `gap 0` dele põe a ponta dentro).
    let g1 = a0["startBinding"]["gap"].as_f64().unwrap();
    assert!((g1 - ph2d_board_route::gap(&r1)).abs() < 1e-6, "{g1}");
    assert_eq!(
        ph2d_board_route::gap(&r1),
        5.0 + r1.style().stroke_width / 2.0
    );
    assert_eq!(a0["endBinding"]["mode"], "inside");
    assert_eq!(a0["endBinding"]["gap"].as_f64().unwrap().abs(), 0.01);
    assert_eq!(a0["roundness"], json!({ "type": 2 }));
    assert_eq!(a0["startArrowhead"], "triangle_outline");
    assert_eq!(a0["endArrowhead"], "cardinality_zero_or_many");
    let a1 = arrows[1];
    assert_eq!(a1["startBinding"]["fixedPoint"], json!([0.5001, 0.5001]));
    assert_eq!(a1["startArrowhead"], "crowfoot_one");
    assert!(a1["roundness"].is_null());
    assert_eq!(world(arrows[2]).len(), 4, "a curva dele com os pontos dela");
    // De fora: sem o nosso elemento no `customData`.
    let stranger: Vec<Value> = file["elements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let mut e = e.clone();
            e.as_object_mut().unwrap().remove("customData");
            e
        })
        .collect();
    let imp = abre(&stranger);
    let ours = setas(&b.doc);
    let theirs = setas(&imp.doc);
    let routes = rotas(&b.doc);
    for (k, (x, y)) in ours.iter().zip(&theirs).enumerate() {
        let (cx, cy) = (x.connector().unwrap(), y.connector().unwrap());
        assert_eq!(cx.route, cy.route, "seta {k}");
        assert_eq!(cx.heads, cy.heads, "seta {k}");
        for (ex, ey) in cx.ends().into_iter().zip(cy.ends()) {
            let (End::Bound { anchor: ax, .. }, End::Bound { anchor: ay, .. }) = (ex, ey) else {
                continue;
            };
            let want = match ax {
                Anchor::Center => Anchor::Orbit([0.5001, 0.5001]),
                a => a,
            };
            assert_eq!(ay, want, "seta {k}");
        }
        let pts = world(arrows[k]);
        let r = routes.get(x.id).unwrap();
        assert!(dist(r.ends()[0], pts[0]) < 1e-9 && dist(r.ends()[1], pts[pts.len() - 1]) < 1e-9);
    }
}

/// ⭐ M5 — a NOTA do next (`stickynote`, `M_nota`: a ferramenta `N`, 200×200, `#ffdf6b`) abre como a
/// nossa nota, sem traço, com o texto dela (`T_formato`). (Escrever continua `rectangle`: o 0.18.1
/// descarta-a — recusa §3.)
#[test]
fn m5_his_sticky_note_opens_as_our_note() {
    let d = saida("M_nota", "next");
    let imp = abre(passo(&d, "desenhar"));
    let els = imp.doc.live_in_z_order();
    let s = els.iter().find_map(|e| e.shape()).expect("a nota");
    assert_eq!(s.kind, ShapeType::Sticky);
    assert_eq!(s.style.fill, Some(Rgba([0xff, 0xdf, 0x6b, 255])));
    assert_eq!(s.style.stroke, None);
    assert!(imp.skipped.is_empty(), "{:?}", imp.skipped);
    let t = saida("T_formato", "next");
    let imp = abre(passo(&t, "restore"));
    let note = imp
        .doc
        .live_in_z_order()
        .into_iter()
        .filter_map(|e| e.shape())
        .find(|s| s.kind == ShapeType::Sticky)
        .expect("a nota do formato");
    assert!(!note.text.is_empty());
}

/// ⭐ M10 — a caneta de largura CONSTANTE do next (`M_caneta`, `strokeOptions.variability
/// "constant"`): a pressão do meio em todos os pontos, medida (não se simula — o traço não afina).
#[test]
fn m10_his_constant_pen_does_not_thin() {
    let d = saida("M_caneta", "next");
    let imp = abre(passo(&d, "desenhar"));
    let ink = imp
        .doc
        .live_in_z_order()
        .into_iter()
        .find_map(|e| e.ink())
        .expect("o traço")
        .clone();
    assert!(ink.pressure);
    assert!(ink.points.iter().all(|p| (p[2] - 0.5).abs() < 1e-6));
}

/// ⭐ M7/M8 — o formato inteiro (`T_formato`): o preenchimento às riscas e cruzado (o ziguezague, não
/// portado, às riscas no desenho e `zigzag` no ficheiro), a rugosidade 2, a semente; o texto solto
/// alinhado e na letra dele — à mão (1, 5, 8) ou a da interface (2, 6, 7, 9 e a mono 3) — na caixa
/// DELE (a nossa alarga a folga: a linha mais larga não quebra).
#[test]
fn m7_m8_his_look_comes_in() {
    let d = saida("T_formato", "next");
    let els = passo(&d, "restore");
    let imp = abre(els);
    let get = |id: &str| -> Element {
        let v = els.iter().find(|e| e["id"] == id).unwrap();
        let k = els
            .iter()
            .filter(|e| e["type"] != "text" || e["containerId"].is_null())
            .filter(|e| !matches!(e["type"].as_str(), Some("embeddable")))
            .position(|e| e["id"] == v["id"])
            .unwrap();
        imp.doc.live_in_z_order()[k].clone()
    };
    let st = |id: &str| get(id).style().clone();
    assert_eq!(st("t_hach").look.fill_style, FillStyle::Hachure);
    assert_eq!(st("t_cross").look.fill_style, FillStyle::CrossHatch);
    assert!(st("t_cross").look.cartoonist && st("t_cross").sketch);
    assert_eq!(st("t_zig").look.fill_style, FillStyle::Zigzag);
    assert_eq!(get("t_hach").seed(), 4_870_185);
    let t1 = st("t_txt1");
    assert_eq!(
        (t1.look.align, t1.look.valign, t1.look.letter),
        (TextAlign::Left, VAlign::Top, Letter::Hand)
    );
    assert_eq!(st("t_txt2").look.letter, Letter::Sans);
    assert_eq!(st("t_txt3").look.align, TextAlign::Right);
    for (id, l) in [
        ("t_txt5", Letter::Hand),
        ("t_txt6", Letter::Sans),
        ("t_txt8", Letter::Hand),
    ] {
        assert_eq!(st(id).look.letter, l, "{id}");
    }
    let t = get("t_txt1");
    let [x0, y0, w, h] = ph2d_board_geom::text_rect(t.shape().unwrap().kind, t.w, t.h);
    assert_eq!([t.x + x0, t.y + y0, w, h], [20.0, 140.0, 120.0, 25.0]);
    // E a volta devolve-o (o ficheiro dele escrito por nós).
    let mut set = BoardSet::default();
    let a = set.create("T".into());
    let board = set.get_mut(a).unwrap();
    board.doc = imp.doc.clone();
    let file: Value = serde_json::from_str(&write(board)).unwrap();
    let mine = |kind: &str, k: &str| -> Vec<Value> {
        file["elements"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["type"] == kind)
            .map(|e| e[k].clone())
            .collect()
    };
    assert!(mine("rectangle", "fillStyle").contains(&json!("hachure")));
    assert!(mine("ellipse", "fillStyle").contains(&json!("zigzag")));
    assert!(mine("rectangle", "roughness").contains(&json!(2)));
    assert!(mine("text", "textAlign").contains(&json!("left")));
    assert!(mine("text", "verticalAlign").contains(&json!("top")));
}

/// ⭐ O ficheiro que escrevemos na W8 (o `customData` sem o aspecto da W9) continua a voltar como o
/// NOSSO elemento — nenhum cai no mapeamento de fora. `fixtures/w8_escrito.excalidraw` = a entrada do
/// oráculo gravada pelo escritor da W8 (`3b4ec7784`), copiada antes de a W9 a regravar.
#[test]
fn a_w8_file_still_carries_our_elements() {
    let file = include_str!("../fixtures/w8_escrito.excalidraw");
    let v: Value = serde_json::from_str(file).unwrap();
    let carried = v["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| !e["customData"]["ph2d"]["element"].is_null())
        .all(|e| {
            serde_json::from_value::<Element>(e["customData"]["ph2d"]["element"].clone()).is_ok()
        });
    assert!(carried);
    assert!(
        read(file)
            .unwrap()
            .doc
            .live_in_z_order()
            .iter()
            .any(|e| matches!(&e.kind, ElementKind::Shape(s) if s.kind == ShapeType::Cylinder))
    );
}

/// Grava a entrada do oráculo da W9 com o NOSSO ficheiro (`entradas/w9_nosso_casos.json`, à parte do
/// `w9_casos.json` — as 220 saídas gravadas levam o sha dele): o quadro [`sample_w9`] escrito por nós,
/// corrido nas duas versões (`restore`, `montar`, `empurrar` as formas). Corre-se à mão quando o
/// escritor muda — e a seguir o `corre_fidelidade.mjs` com este ficheiro (README do oráculo §W9).
#[test]
#[ignore = "grava a entrada do oráculo (à mão, quando o escritor muda)"]
fn grava_a_entrada_w9_nosso() {
    let file: Value = serde_json::from_str(&write(&sample_w9())).unwrap();
    let els = file["elements"].as_array().unwrap();
    let shapes: Vec<&Value> = els
        .iter()
        .filter(|e| e["type"] != "arrow" && e["type"] != "text")
        .map(|e| &e["id"])
        .collect();
    let casos = json!({ "casos": [{
        "nome": "X_nosso",
        "descricao": "o NOSSO .excalidraw (W9): ponto fixo (inside), órbita e centro (orbit), a curva dele, as pontas novas e o aspecto — escrito por ph2d-board-excalidraw::write",
        "elementos": els,
        "passos": [
            { "tipo": "restore" }, { "tipo": "svg" }, { "tipo": "montar" },
            { "tipo": "empurrar", "ids": shapes }, { "tipo": "svg" }
        ]
    }]});
    std::fs::write(
        format!("{DIR}/entradas/w9_nosso_casos.json"),
        serde_json::to_string_pretty(&casos).unwrap() + "\n",
    )
    .unwrap();
}

/// ⭐⭐ §6.2-5 (a metade do oráculo) — o NOSSO ficheiro ([`sample_w9`], `entradas/w9_nosso_casos.json`)
/// passado pelas DUAS versões: ao abrir nenhuma mexe nas pontas (XB7); o next, depois de RE-LIGAR as
/// setas (`empurrar` as formas), deixa cada ponta presa onde a escrevemos — o ponto fixo é o `inside`
/// dele, a órbita e o centro a XB1 dele (≤ 0,05 px); o 0.18.1 guarda as chaves do next tal e qual (XV1).
#[test]
fn xv3_both_versions_keep_the_tips_we_wrote() {
    let ours: Value = serde_json::from_str(&write(&sample_w9())).unwrap();
    let wrote: Vec<Vec<[f64; 2]>> = ours["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "arrow")
        .map(world)
        .collect();
    let arrows = |d: &Value, tipo: &str| -> Vec<Value> {
        passo(d, tipo)
            .iter()
            .filter(|e| e["type"] == "arrow")
            .cloned()
            .collect()
    };
    let next = saida("X_nosso", "next");
    let old = saida("X_nosso", "0181");
    for (versao, d) in [("next", &next), ("0181", &old)] {
        let opened = arrows(d, "restore");
        assert_eq!(opened.len(), wrote.len(), "{versao}");
        for (a, w) in opened.iter().zip(&wrote) {
            let p = world(a);
            assert!(
                dist(p[0], w[0]) < 1e-6 && dist(p[p.len() - 1], w[w.len() - 1]) < 1e-6,
                "{versao} mexeu ao abrir: {p:?} · {w:?}"
            );
        }
    }
    for a in arrows(&old, "restore") {
        for key in ["startBinding", "endBinding"] {
            if a[key].is_object() {
                assert!(
                    a[key]["mode"].is_string() && a[key]["fixedPoint"].is_array(),
                    "{a}"
                );
            }
        }
    }
    let after = arrows(&next, "empurrar");
    let mut medidos = 0;
    for (a, w) in after.iter().zip(&wrote) {
        let p = world(a);
        for (k, key) in ["startBinding", "endBinding"].into_iter().enumerate() {
            if !a[key].is_object() {
                continue;
            }
            medidos += 1;
            let (got, want) = if k == 0 {
                (p[0], w[0])
            } else {
                (p[p.len() - 1], w[w.len() - 1])
            };
            assert!(
                dist(got, want) <= 0.05,
                "{} {key}: o next pôs {got:?}, escrevemos {want:?} ({:.3})",
                a["id"],
                dist(got, want)
            );
        }
    }
    assert_eq!(medidos, 5);
}

/// ⭐ O texto que escrevemos assenta onde o `verticalAlign` que escrevemos diz (auditoria W9: o `y` era
/// sempre o do meio e o ficheiro dizia «top»), e o cotovelo grava o ponto do CONTORNO (a ponta está a
/// `g` dele: num estranho, um `fixedPoint` fora da caixa escalava o vão com a forma).
#[test]
fn the_text_sits_where_its_valign_says_and_an_elbow_writes_a_point_of_the_outline() {
    let mut b = sample_w9();
    let ink = Rgba([30, 30, 30, 255]);
    let a = b.doc.live_in_z_order()[0].id;
    let c = b.doc.live_in_z_order()[1].id;
    let mut el = Connector::new(
        End::Bound {
            target: a,
            anchor: Anchor::Center,
        },
        End::Bound {
            target: c,
            anchor: Anchor::Center,
        },
        Route::Elbow,
        Style::new(None, Some(ink), ink),
    );
    el.heads = [Head::None, Head::Arrow];
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), el);
    put(&mut b, el);
    let file: Value = serde_json::from_str(&write(&b)).unwrap();
    let els = file["elements"].as_array().unwrap();
    let box1 = els
        .iter()
        .find(|e| e["id"] == format!("ph2d-{}", a.0))
        .unwrap();
    let text1 = els
        .iter()
        .find(|e| e["id"] == format!("ph2d-{}-text", a.0))
        .unwrap();
    assert_eq!(text1["verticalAlign"], "top");
    let pad = ph2d_board_geom::TEXT_PADDING;
    assert!((text1["y"].as_f64().unwrap() - (box1["y"].as_f64().unwrap() + pad)).abs() < 1e-9);
    let elbow = els.iter().find(|e| e["elbowed"] == true).unwrap();
    for key in ["startBinding", "endBinding"] {
        let fp = elbow[key]["fixedPoint"].as_array().unwrap();
        for v in fp {
            let v = v.as_f64().unwrap();
            assert!(
                (-1e-9..=1.0 + 1e-9).contains(&v),
                "{key}: {fp:?} sai da caixa"
            );
        }
    }
}
