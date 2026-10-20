//! W10 — o legado `{focus, gap}` do 0.18.1 DEPOIS de mexer uma forma (estudo 07 §2, XV7): o NOSSO
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
    let r = shape(
        b,
        ShapeType::Rectangle,
        [0.0, 0.0, 200.0, 120.0],
        0.0,
        false,
    );
    let e = shape(
        b,
        ShapeType::Ellipse,
        [400.0, 0.0, 200.0, 120.0],
        0.0,
        false,
    );
    let d = shape(
        b,
        ShapeType::Diamond,
        [800.0, 0.0, 200.0, 120.0],
        0.0,
        false,
    );
    let rr = shape(
        b,
        ShapeType::Rectangle,
        [0.0, 300.0, 200.0, 120.0],
        0.5,
        false,
    );
    let er = shape(
        b,
        ShapeType::Ellipse,
        [400.0, 300.0, 200.0, 120.0],
        0.5,
        false,
    );
    let ro = shape(
        b,
        ShapeType::Rectangle,
        [800.0, 300.0, 200.0, 120.0],
        0.0,
        true,
    );
    let dr = shape(
        b,
        ShapeType::Diamond,
        [1200.0, 0.0, 200.0, 120.0],
        0.3,
        false,
    );
    let bound = |target, anchor| End::Bound { target, anchor };
    let arrow = |b: &mut Board, s: End, t: End, route: Route, via: Vec<[f64; 2]>| {
        let mut c = Connector::new(s, t, route, Style::new(None, Some(ink), ink));
        c.waypoints = via;
        c.heads = [Head::None, Head::Arrow];
        let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
        put(b, el);
    };
    let st = Route::Straight;
    arrow(
        b,
        bound(r, Anchor::Orbit([0.8, 0.2])),
        bound(e, Anchor::Orbit([0.3, 0.7])),
        st,
        vec![],
    );
    // Rectas PARALELAS aos eixos fora do centro (o 0.18.1 espelhava-as): horizontal entre R e E,
    // vertical de baixo para R.
    arrow(
        b,
        bound(r, Anchor::Orbit([1.0, 0.3])),
        bound(e, Anchor::Orbit([0.0, 0.3])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([60.0, 250.0]),
        bound(r, Anchor::Orbit([0.3, 1.0])),
        st,
        vec![],
    );
    arrow(
        b,
        bound(e, Anchor::Center),
        bound(d, Anchor::Orbit([0.5, 0.0])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([100.0, -150.0]),
        bound(r, Anchor::Fixed([0.75, 0.3])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([520.0, -150.0]),
        bound(e, Anchor::Fixed([0.6, 0.4])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([700.0, -150.0]),
        bound(e, Anchor::Fixed([1.0, 0.5])),
        st,
        vec![],
    );
    // Pontos fixos que o 0.18.1 NÃO exprime (auditoria W10, lente 1): do lado de LÁ da recta, e na
    // elipse fundo junto ao eixo maior; e dentro do losango e do rectângulo redondo.
    arrow(
        b,
        End::Free([-200.0, 100.0]),
        bound(r, Anchor::Fixed([0.9, 0.5])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([-200.0, 100.0]),
        bound(r, Anchor::Fixed([0.75, 0.3])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([200.0, 100.0]),
        bound(e, Anchor::Fixed([0.8, 0.5])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([200.0, 200.0]),
        bound(e, Anchor::Fixed([0.6, 0.4])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([200.0, 64.0]),
        bound(e, Anchor::Fixed([0.55, 0.5])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([900.0, -150.0]),
        bound(d, Anchor::Fixed([0.5, 0.3])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([1150.0, 500.0]),
        bound(ro, Anchor::Fixed([0.3, 0.6])),
        st,
        vec![],
    );
    arrow(
        b,
        bound(d, Anchor::Orbit([0.9, 0.5])),
        bound(rr, Anchor::Orbit([0.2, 0.2])),
        st,
        vec![],
    );
    arrow(
        b,
        bound(rr, Anchor::Center),
        bound(er, Anchor::Orbit([0.7, 0.9])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([1150.0, 500.0]),
        bound(ro, Anchor::Orbit([0.1, 0.1])),
        st,
        vec![],
    );
    arrow(
        b,
        bound(er, Anchor::Orbit([0.5, 0.5])),
        bound(ro, Anchor::Center),
        Route::Spline,
        vec![[700.0, 560.0]],
    );
    arrow(
        b,
        End::Free([1300.0, -150.0]),
        bound(dr, Anchor::Orbit([0.3, 0.3])),
        st,
        vec![],
    );
    arrow(
        b,
        bound(r, Anchor::Orbit([0.5, 1.0])),
        bound(er, Anchor::Orbit([0.0, 0.5])),
        Route::Curved,
        vec![],
    );
    // W23 B9: o losango REDONDO e o rectângulo redondo acima do tecto do raio, de medidas
    // fraccionárias e rodados — a lei das peças; pontos fixos do lado de cá, do lado de lá, num canto.
    let drr = shape(
        b,
        ShapeType::Diamond,
        [1200.0, 300.0, 173.4, 97.8],
        0.2,
        true,
    );
    let ror = shape(
        b,
        ShapeType::Rectangle,
        [1500.0, 0.0, 157.3, 211.9],
        0.35,
        true,
    );
    arrow(
        b,
        End::Free([1290.0, 550.0]),
        bound(drr, Anchor::Orbit([0.8, 0.3])),
        st,
        vec![],
    );
    arrow(
        b,
        bound(drr, Anchor::Center),
        bound(ror, Anchor::Orbit([0.1, 0.8])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([1100.0, 600.0]),
        bound(drr, Anchor::Fixed([0.4, 0.55])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([1450.0, 350.0]),
        bound(drr, Anchor::Fixed([0.3, 0.5])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([1850.0, -100.0]),
        bound(ror, Anchor::Fixed([0.7, 0.3])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([1850.0, 320.0]),
        bound(ror, Anchor::Orbit([0.95, 0.97])),
        st,
        vec![],
    );
    arrow(
        b,
        End::Free([1400.0, -50.0]),
        bound(ror, Anchor::Fixed([0.8, 0.6])),
        st,
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

/// A forma e a âncora de cada ponta presa das setas que escrevemos, pela ordem do ficheiro.
fn targets(b: &Board) -> Vec<[Option<(Element, Anchor)>; 2]> {
    setas(&b.doc)
        .into_iter()
        .map(|a| {
            a.connector().unwrap().ends().map(|e| match e {
                End::Bound { target, anchor } => b.doc.get(target).cloned().map(|t| (t, anchor)),
                End::Free(_) => None,
            })
        })
        .collect()
}

/// O 0.18.1 contra a lei, com UMA ponta presa (medido 20/10, W23 B9: 0,0009 — o canto do rectângulo
/// redondo, o Newton dele a 1e-3).
const LAW_ONE: f64 = 0.002;
/// …e com as DUAS: ao empurrar as formas ele re-liga uma ponta de cada vez, a partir da outra ainda
/// deslocada de 1 px (medido 0,1199, o losango redondo rodado ao centro → o rectângulo redondo rodado).
const LAW_TWO: f64 = 0.12;
/// Onde o 0.18.1 põe a ponta em órbita e ao centro, de onde a escrevemos — TODAS as formas (medido
/// 0,1199, a mesma seta de [`LAW_TWO`]; antes da W23 B9 o losango 3,875 e o rectângulo redondo 3,513).
const ORBIT: f64 = 0.12;
/// Um ponto fixo DENTRO que o 0.18.1 não exprime (do lado de LÁ da recta): até onde fica a ponta que
/// a lei dá ao par que escrevemos (medido 70,12, o rectângulo; o losango redondo 35,13, o rectângulo
/// redondo 52,29 — antes da W23 B9 85,6).
const UNEXPRESSIBLE: f64 = 71.0;

/// ⭐⭐ XV7 — o NOSSO ficheiro no 0.18.1. Ao abrir nada mexe e o legado gravado é o que o escritor de
/// HOJE escreve; ao EMPURRAR as formas (ele re-liga cada seta pelo legado) cada ponta cai onde a lei
/// medida diz ([`crate::legacy_focus::landing`]: ≤ [`LAW_ONE`], ≤ [`LAW_TWO`] com as duas pontas
/// presas), e em órbita e ao centro é a que escrevemos (≤ [`ORBIT`], as cinco formas — o losango e o
/// rectângulo redondo pela lei das peças, W23 B9). Um ponto fixo do lado de LÁ da recta não se
/// exprime: fica o par que cai mais perto (≤ [`UNEXPRESSIBLE`]).
#[test]
fn xv7_the_0181_moves_no_tip_when_it_rebinds_our_arrows() {
    let b = sample_w10();
    let ours: Vec<Value> = serde_json::from_str::<Value>(&write(&b)).unwrap()["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "arrow")
        .cloned()
        .collect();
    let d = saida10("X10_nosso", "0181");
    let opened: Vec<&Value> = passo(&d, "restore")
        .iter()
        .filter(|e| e["type"] == "arrow")
        .collect();
    let after: Vec<&Value> = passo(&d, "empurrar")
        .iter()
        .filter(|e| e["type"] == "arrow")
        .collect();
    assert_eq!((opened.len(), after.len()), (ours.len(), ours.len()));
    const KEYS: [&str; 2] = ["startBinding", "endBinding"];
    let mut fails = Vec::new();
    let mut worst = [0.0f64; 4];
    let mut kinds = std::collections::BTreeSet::new();
    for (((o, a), w), t) in opened.iter().zip(&after).zip(&ours).zip(targets(&b)) {
        let (wp, op, ap) = (world(w), world(o), world(a));
        for (q, r) in op.iter().zip(&wp) {
            assert!(dist(*q, *r) < 1e-9, "{}: o 0.18.1 mexeu ao ABRIR", w["id"]);
        }
        let both = t.iter().all(Option::is_some);
        for (k, end) in t.iter().enumerate() {
            let Some((shape, anchor)) = end else { continue };
            let (wb, ob) = (&w[KEYS[k]], &o[KEYS[k]]);
            for f in ["focus", "gap"] {
                let (x, y) = (wb[f].as_f64().unwrap(), ob[f].as_f64().unwrap());
                assert!(
                    (x - y).abs() <= 1e-9 * x.abs().max(1.0),
                    "{} {f}: o oráculo correu {y}, o escritor escreve {x} — corra-o outra vez",
                    w["id"]
                );
            }
            let n = wp.len();
            let (tip, from, got) = if k == 0 {
                (wp[0], wp[1], ap[0])
            } else {
                (wp[n - 1], wp[n - 2], ap[ap.len() - 1])
            };
            let (focus, gap) = (wb["focus"].as_f64().unwrap(), wb["gap"].as_f64().unwrap());
            let kind = shape.shape().unwrap().kind;
            kinds.insert(format!("{kind:?} {}", shape.style().round));
            let label = format!("{} ponta {k} ({kind:?}, {anchor:?})", w["id"]);
            let law = crate::legacy_focus::landing(shape, from, focus, gap)
                .map_or(f64::INFINITY, |land| dist(got, land));
            let real = dist(got, tip);
            eprintln!("{label}: lei {law:.4} · o 0.18.1 a {real:.4} da nossa");
            let (lw, lb) = if both { (1, LAW_TWO) } else { (0, LAW_ONE) };
            let (rw, rb) = if matches!(anchor, Anchor::Fixed(_)) {
                (3, UNEXPRESSIBLE)
            } else {
                (2, ORBIT)
            };
            worst[lw] = worst[lw].max(law);
            worst[rw] = worst[rw].max(real);
            if law > lb || real > rb {
                fails.push(format!(
                    "{label}: a lei erra {law:.4}, a ponta fica a {real:.4} da nossa"
                ));
            }
        }
    }
    eprintln!("lei 1 ponta · 2 pontas · órbita/centro · fixo: {worst:.4?}");
    assert_eq!(kinds.len(), 5, "as cinco formas: {kinds:?}");
    assert!(fails.is_empty(), "o 0.18.1: {fails:#?}");
}

/// ⭐ A LEI do legado (XV7) contra a varredura do oráculo (`saidas/w10_F_*.0181.json`: forma × vão ×
/// rotação × cantos × espessura × proporção, 12 ângulos × 7 focos, presas pelo fim e pelo início):
/// de cada ponta que o 0.18.1 re-ligou, [`crate::legacy_focus::legacy`] devolve o `focus` e o `gap`
/// gravados — o losango e o rectângulo redondo pela lei das peças (W23 B9), salvo a ponta no ponto do
/// foco (um só cruzamento: o vão não se lê dela).
#[test]
fn xv7_the_legacy_law_reproduces_the_0181_sweep() {
    let mut casos = 0;
    let mut pontas = 0;
    let mut espelhadas = 0;
    let (mut gaps, mut no_gap) = (0, 0);
    let mut worst_gap: f64 = 0.0;
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
            if caso.contains("eixos")
                && (tip[0] - from[0]).abs() > 1e-6
                && (tip[1] - from[1]).abs() > 1e-6
            {
                espelhadas += 1;
                continue;
            }
            assert!(
                (focus - want).abs() < tol,
                "{caso} {}: focus {focus} ≠ {want}",
                a["id"]
            );
            let g = b["gap"].as_f64().unwrap();
            let aim = crate::legacy_focus::landing(&shape, from, want, 0.0);
            if aim.is_some_and(|m| dist(m, tip) < 1e-3) {
                no_gap += 1;
                continue;
            }
            if g != 0.0 && !caso.contains("eixos") {
                gaps += 1;
                // A ponta do 0.18.1 nos cantos redondos vem do Newton dele (a 1e-3 px): o vão lido
                // dela também.
                // (medido 1,4e-4)
                let tol = if kind == ShapeType::Diamond || shape.style().round {
                    1e-3
                } else {
                    1e-6
                };
                worst_gap = worst_gap.max((gap - g).abs());
                assert!((gap - g).abs() < tol, "{caso} {}: gap {gap} ≠ {g}", a["id"]);
            }
        }
    }
    assert_eq!(casos, 34, "a varredura inteira");
    eprintln!(
        "{pontas} pontas, {gaps} vãos lidos (pior {worst_gap:.2e}), {no_gap} no ponto do foco"
    );
    assert!(gaps > 2500 && no_gap > 0, "{gaps} · {no_gap}");
    assert!(pontas > 3000, "{pontas}");
    assert!(
        espelhadas > 0,
        "o empate das rectas paralelas aos eixos já não aparece"
    );
}
