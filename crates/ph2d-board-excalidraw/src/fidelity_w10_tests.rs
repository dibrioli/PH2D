//! W10 — o legado `{focus, gap}` do 0.18.1 DEPOIS de mexer uma forma (estudo 07 §2, XV4): o NOSSO
//! ficheiro passado pelas duas versões e as formas EMPURRADAS (o editor re-liga as setas pela lei
//! dele). Módulo filho de `fidelity_tests` (os ajudantes do oráculo são de lá); as saídas são
//! `saidas/w10_*` (`corre_fidelidade.mjs --prefixo w10`).

use super::*;

fn saida10(caso: &str, versao: &str) -> Value {
    let p = format!("{DIR}/saidas/w10_{caso}.{versao}.json");
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .unwrap()
}

fn put(b: &mut Board, el: Element) -> ph2d_board_model::ElementId {
    let id = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    id
}

/// Um quadro com as três formas nativas do Excalidraw — direitas, rodadas e de cantos redondos — e
/// setas presas em órbita fora do centro, ao centro e a pontos fixos (dentro e no contorno), rectas,
/// diagonais, a curva dele e a do Miro.
fn sample_w10() -> Board {
    let mut set = BoardSet::default();
    let a = set.create("W10".into());
    let b = set.get_mut(a).unwrap();
    let ink = Rgba([30, 30, 30, 255]);
    let shape = |b: &mut Board, kind: ShapeType, bx: [f64; 4], angle: f64, round: bool| {
        let mut st = Style::new(Some(Rgba([165, 216, 255, 255])), Some(ink), ink);
        st.round = round;
        let mut el = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Shape {
                kind,
                style: st,
                text: RichText::plain(""),
            },
            bx,
        );
        el.angle = angle;
        put(b, el)
    };
    let r = shape(b, ShapeType::Rectangle, [0.0, 0.0, 200.0, 120.0], 0.0, false);
    let e = shape(b, ShapeType::Ellipse, [400.0, 0.0, 200.0, 120.0], 0.0, false);
    let d = shape(b, ShapeType::Diamond, [800.0, 0.0, 200.0, 120.0], 0.0, false);
    let rr = shape(b, ShapeType::Rectangle, [0.0, 300.0, 200.0, 120.0], 0.5, false);
    let er = shape(b, ShapeType::Ellipse, [400.0, 300.0, 200.0, 120.0], 0.5, false);
    let ro = shape(b, ShapeType::Rectangle, [800.0, 300.0, 200.0, 120.0], 0.0, true);
    let dr = shape(b, ShapeType::Diamond, [1200.0, 0.0, 200.0, 120.0], 0.3, false);
    let bound = |target, anchor| End::Bound { target, anchor };
    let arrow = |b: &mut Board, s: End, t: End, route: Route, via: Vec<[f64; 2]>| {
        let mut c = Connector::new(s, t, route, Style::new(None, Some(ink), ink));
        c.waypoints = via;
        c.heads = [Head::None, Head::Arrow];
        let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
        put(b, el);
    };
    let st = Route::Straight;
    arrow(b, bound(r, Anchor::Orbit([0.8, 0.2])), bound(e, Anchor::Orbit([0.3, 0.7])), st, vec![]);
    // Rectas PARALELAS aos eixos fora do centro (o 0.18.1 espelhava-as): horizontal entre R e E,
    // vertical de baixo para R.
    arrow(b, bound(r, Anchor::Orbit([1.0, 0.3])), bound(e, Anchor::Orbit([0.0, 0.3])), st, vec![]);
    arrow(b, End::Free([60.0, 250.0]), bound(r, Anchor::Orbit([0.3, 1.0])), st, vec![]);
    arrow(b, bound(e, Anchor::Center), bound(d, Anchor::Orbit([0.5, 0.0])), st, vec![]);
    arrow(b, End::Free([100.0, -150.0]), bound(r, Anchor::Fixed([0.75, 0.3])), st, vec![]);
    arrow(b, End::Free([520.0, -150.0]), bound(e, Anchor::Fixed([0.6, 0.4])), st, vec![]);
    arrow(b, End::Free([700.0, -150.0]), bound(e, Anchor::Fixed([1.0, 0.5])), st, vec![]);
    arrow(b, bound(d, Anchor::Orbit([0.9, 0.5])), bound(rr, Anchor::Orbit([0.2, 0.2])), st, vec![]);
    arrow(b, bound(rr, Anchor::Center), bound(er, Anchor::Orbit([0.7, 0.9])), st, vec![]);
    arrow(b, End::Free([1150.0, 500.0]), bound(ro, Anchor::Orbit([0.1, 0.1])), st, vec![]);
    arrow(
        b,
        bound(er, Anchor::Orbit([0.5, 0.5])),
        bound(ro, Anchor::Center),
        Route::Spline,
        vec![[700.0, 560.0]],
    );
    arrow(b, End::Free([1300.0, -150.0]), bound(dr, Anchor::Orbit([0.3, 0.3])), st, vec![]);
    arrow(
        b,
        bound(r, Anchor::Orbit([0.5, 1.0])),
        bound(er, Anchor::Orbit([0.0, 0.5])),
        Route::Curved,
        vec![],
    );
    b.clone()
}

/// Grava a ENTRADA do oráculo W10 (o nosso ficheiro, como `grava_a_entrada_w9_nosso`): `cargo test -p
/// ph2d-board-excalidraw grava_a_entrada_w10_nosso -- --ignored`, depois `corre_fidelidade.mjs
/// --prefixo w10 entradas/w10_nosso_casos.json saidas`.
#[test]
#[ignore = "grava a entrada do oráculo"]
fn grava_a_entrada_w10_nosso() {
    let file: Value = serde_json::from_str(&write(&sample_w10())).unwrap();
    let els = file["elements"].as_array().unwrap();
    let shapes: Vec<&Value> = els
        .iter()
        .filter(|e| e["type"] != "arrow" && e["type"] != "text")
        .map(|e| &e["id"])
        .collect();
    let casos = json!({ "casos": [{
        "nome": "X10_nosso",
        "descricao": "o NOSSO .excalidraw (W10): rectângulo, elipse e losango (direitos, rodados, de cantos redondos), órbita fora do centro, centro e ponto fixo, recta, a curva dele e a do Miro — escrito por ph2d-board-excalidraw::write",
        "elementos": els,
        "passos": [
            { "tipo": "restore" }, { "tipo": "montar" },
            { "tipo": "empurrar", "ids": shapes }, { "tipo": "svg" }
        ]
    }]});
    std::fs::write(
        format!("{DIR}/entradas/w10_nosso_casos.json"),
        serde_json::to_string_pretty(&casos).unwrap() + "\n",
    )
    .unwrap();
}

/// A forma de cada ponta presa das setas que escrevemos, pela ordem do ficheiro.
fn targets(b: &Board) -> Vec<[Option<Element>; 2]> {
    setas(&b.doc)
        .into_iter()
        .map(|a| {
            a.connector().unwrap().ends().map(|e| match e {
                End::Bound { target, .. } => b.doc.get(target).cloned(),
                End::Free(_) => None,
            })
        })
        .collect()
}

/// ⭐⭐ XV4 — o NOSSO ficheiro no 0.18.1: ao abrir nada mexe, e ao EMPURRAR as formas (o editor re-liga
/// cada seta pelo legado `{focus, gap}` que escrevemos) as pontas ficam onde as escrevemos — a ≤ 0,05 px
/// no rectângulo e na elipse (medido ≤ 0,044: com as DUAS pontas presas ele re-liga uma de cada vez, a
/// partir da outra ainda deslocada). O losango e o rectângulo de cantos redondos: o vão dele não tem lei
/// medida (estudo 07 §3) — ≤ [`UNMEASURED_GAP`] (medido 3,875 e 3,513; antes da W10, até 106).
#[test]
fn xv4_the_0181_moves_no_tip_when_it_rebinds_our_arrows() {
    const UNMEASURED_GAP: f64 = 4.0;
    let b = sample_w10();
    let ours: Value = serde_json::from_str(&write(&b)).unwrap();
    let wrote: Vec<Vec<[f64; 2]>> = ours["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "arrow")
        .map(world)
        .collect();
    let tg = targets(&b);
    let d = saida10("X10_nosso", "0181");
    for (a, w) in passo(&d, "restore").iter().filter(|e| e["type"] == "arrow").zip(&wrote) {
        assert_eq!(&world(a), w, "o 0.18.1 mexeu ao ABRIR");
    }
    let after: Vec<&Value> = passo(&d, "empurrar")
        .iter()
        .filter(|e| e["type"] == "arrow")
        .collect();
    assert_eq!(after.len(), wrote.len());
    let mut fails = Vec::new();
    for ((a, w), t) in after.iter().zip(&wrote).zip(&tg) {
        let p = world(a);
        for k in 0..2 {
            let Some(shape) = &t[k] else { continue };
            let (got, want) = if k == 0 {
                (p[0], w[0])
            } else {
                (p[p.len() - 1], w[w.len() - 1])
            };
            let e = dist(got, want);
            let s = shape.shape().unwrap();
            let bound = if s.kind == ShapeType::Diamond || s.style.round {
                UNMEASURED_GAP
            } else {
                0.05
            };
            if e > bound {
                fails.push(format!(
                    "{} ponta {k} ({:?}): {e:.3}",
                    a["id"],
                    shape.shape().unwrap().kind
                ));
            }
        }
    }
    assert!(fails.is_empty(), "o 0.18.1 mexeu: {fails:#?}");
}

/// ⭐ A LEI do legado (XV4) contra a varredura do oráculo (`saidas/w10_F_*.0181.json`: forma × vão ×
/// rotação × cantos × espessura × proporção, 12 ângulos × 7 focos, presas pelo fim e pelo início):
/// de cada ponta que o 0.18.1 re-ligou, [`crate::legacy_focus::legacy`] devolve o `focus` gravado — e
/// o `gap` no rectângulo de cantos vivos e na elipse (o losango e o rectângulo redondo não têm a lei do
/// vão medida).
#[test]
fn xv4_the_legacy_law_reproduces_the_0181_sweep() {
    let mut casos = 0;
    let mut pontas = 0;
    let mut espelhadas = 0;
    for f in std::fs::read_dir(format!("{DIR}/saidas")).unwrap() {
        let name = f.unwrap().file_name().into_string().unwrap();
        if !(name.starts_with("w10_F_") && name.ends_with(".0181.json")) {
            continue;
        }
        casos += 1;
        let caso = &name["w10_".len()..name.len() - ".0181.json".len()];
        let d = saida10(caso, "0181");
        let els = passo(&d, "empurrar");
        let f = els.iter().find(|e| e["id"] == "F").unwrap();
        let num = |k: &str| f[k].as_f64().unwrap();
        let kind = match f["type"].as_str().unwrap() {
            "rectangle" => ShapeType::Rectangle,
            "ellipse" => ShapeType::Ellipse,
            _ => ShapeType::Diamond,
        };
        let mut st = Style::new(None, Some(Rgba([30, 30, 30, 255])), Rgba([30, 30, 30, 255]));
        st.round = !f["roundness"].is_null();
        let mut shape = Element::new_shape(
            ph2d_board_model::ElementId(1),
            ph2d_board_model::FracKey::between(None, None),
            Shape {
                kind,
                style: st,
                text: RichText::plain(""),
            },
            [num("x"), num("y"), num("width"), num("height")],
        );
        shape.angle = num("angle");
        let c = shape.center();
        let gap_law = kind == ShapeType::Ellipse || (kind == ShapeType::Rectangle && !shape.style().round);
        for a in els.iter().filter(|e| e["type"] == "arrow") {
            let p = world(a);
            let end = a["id"].as_str().unwrap().starts_with('e');
            let (tip, from, b) = if end {
                (p[p.len() - 1], p[p.len() - 2], &a["endBinding"])
            } else {
                (p[0], p[1], &a["startBinding"])
            };
            if dist(tip, c) < 1e-6 {
                continue; // não re-ligada (a recta do foco não toca a forma + vão)
            }
            pontas += 1;
            let (focus, gap) = crate::legacy_focus::legacy(&shape, from, tip, -1.0);
            let want = b["focus"].as_f64().unwrap();
            // A precisão do próprio 0.18.1 nas formas redondas (2,5e-5 no losango, 2,4e-6 no
            // rectângulo) e o `UNTIE` (1e-6).
            let tol = if shape.style().round { 1e-4 } else { 1e-5 };
            // Nas rectas paralelas a um eixo dois cantos empatam e ele escolhe o do OUTRO lado: a
            // recta que ele traça já não é paralela (por isso escrevemos `|focus|·UNTIE`).
            if caso.contains("eixos") && (tip[0] - from[0]).abs() > 1e-6 && (tip[1] - from[1]).abs() > 1e-6 {
                espelhadas += 1;
                continue;
            }
            assert!((focus - want).abs() < tol, "{caso} {}: focus {focus} ≠ {want}", a["id"]);
            let g = b["gap"].as_f64().unwrap();
            if gap_law && g != 0.0 && !caso.contains("eixos") {
                assert!((gap - g).abs() < 1e-6, "{caso} {}: gap {gap} ≠ {g}", a["id"]);
            }
        }
    }
    assert_eq!(casos, 34, "a varredura inteira");
    assert!(pontas > 3000, "{pontas}");
    assert!(espelhadas > 0, "o empate das rectas paralelas aos eixos já não aparece");
}
