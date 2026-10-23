//! W24 — o «Draw to shape» do oráculo `next` 0.18.0-143b5b6 contra a caneta do quadro (segurar parado,
//! o QuickShape do vetor), sobre os MESMOS 80 traços (`saidas/bloco6_traco.next.json`, `corre_bloco6.mjs`
//! sobre `entradas/bloco6_traco.json`; doc 17 §10). Módulo filho de `fidelity_tests`.

use super::*;
use ph2d_board_edit::{Editor, Metrics, Mods, Pointer, Tool};
use ph2d_board_model::{BoardDoc, History, Pen};
use ph2d_text::TextSystem;
use std::collections::BTreeMap;

/// O relógio da corrida deles: um evento a cada 16 ms, e a mão parada 600 ms antes de largar.
const DT: u128 = 16_000_000;
const STILL: u128 = 600_000_000;
const T0: u128 = 1_000_000_000_000;

fn json(rel: &str) -> Value {
    let p = format!("{DIR}/{rel}");
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}")))
        .unwrap()
}

fn editor() -> Editor {
    let ink = Rgba([20, 20, 20, 255]);
    Editor::new(
        Style::new(None, Some(ink), ink),
        Metrics {
            handle: 8.0,
            rotate_offset: 24.0,
            snap: 8.0,
            drag: 3.0,
            click_size: [160.0, 100.0],
            paste_offset: 10.0,
            bind: 8.0,
            dot: 40.0,
            eraser: 10.0,
            frame_title: 20.0,
        },
    )
}

/// O que ficou no quadro, numa palavra.
fn word(el: &Element) -> String {
    if let Some(s) = el.shape() {
        return format!("{:?}", s.kind).to_lowercase();
    }
    if let Some(c) = el.connector() {
        return match c.heads {
            [Head::None, Head::None] => "line".into(),
            [Head::None, _] => "arrow".into(),
            _ => "arrow_both".into(),
        };
    }
    match el.ink() {
        Some(i) if i.clean => "ink_clean".into(),
        Some(_) => "ink_raw".into(),
        None => format!("{:?}", el.kind),
    }
}

/// O traço `pts` (ecrã = mundo, zoom 1) com a caneta; `hold` = a mão parada no fim antes de largar.
/// Devolve o que ficou e quantos elementos vivos.
fn draw(pts: &[[f64; 2]], hold: bool) -> (String, usize) {
    let mut ed = editor();
    let (mut doc, mut h, mut ts) = (
        BoardDoc::default(),
        History::default(),
        TextSystem::without_system_fonts(),
    );
    let at = |p: [f64; 2]| Pointer {
        world: p,
        px: 1.0,
        mods: Mods::default(),
    };
    ed.tool = Tool::Pen(Pen::Pen);
    let mut t = T0;
    ed.at_time(t);
    let view = [-10_000.0, -10_000.0, 10_000.0, 10_000.0];
    ed.pointer_down(&mut doc, &mut h, &mut ts, at(pts[0]), view);
    for &p in &pts[1..] {
        t += DT;
        ed.at_time(t);
        ed.pointer_move(&mut doc, &mut ts, at(p));
    }
    let end = pts[pts.len() - 1];
    if hold {
        t += STILL;
        ed.at_time(t);
        ed.pointer_move(&mut doc, &mut ts, at(end));
    }
    ed.pointer_up(&mut doc, &mut h, at(end));
    let live = doc.live_in_z_order();
    (live.last().map_or("nada".into(), |e| word(e)), live.len())
}

/// As divergências ESCOLHIDAS (doc 17 §10): (o traço, o que o nosso deixa, porquê).
const DIVERGE: [(&str, &str, &str); 13] = [
    ("rect_rodado_25", "rectangle", "guardamos a rotação"),
    ("rect_rodado_45", "rectangle", "guardamos a rotação"),
    ("poligono_3_lados", "triangle", "o quadro tem o triângulo"),
    (
        "triangulo_aberto_40",
        "triangle",
        "o quadro tem o triângulo",
    ),
    (
        "triangulo_tremor_06",
        "triangle",
        "o quadro tem o triângulo",
    ),
    (
        "paralelogramo_10",
        "ink_clean",
        "nenhuma forma do quadro o desenha igual",
    ),
    (
        "paralelogramo_15",
        "ink_clean",
        "nenhuma forma do quadro o desenha igual",
    ),
    (
        "paralelogramo_20",
        "ink_clean",
        "nenhuma forma do quadro o desenha igual",
    ),
    (
        "paralelogramo_30",
        "ink_clean",
        "nenhuma forma do quadro o desenha igual",
    ),
    (
        "trapezio",
        "ink_clean",
        "nenhuma forma do quadro o desenha igual",
    ),
    (
        "poligono_6_lados",
        "ink_clean",
        "nenhuma forma do quadro o desenha igual",
    ),
    (
        "poligono_8_lados",
        "ink_clean",
        "nenhuma forma do quadro o desenha igual",
    ),
    (
        "seta_recta_duas_pontas",
        "arrow_both",
        "reconhecemos as duas pontas",
    ),
];

/// A lei do lápis do Vetor não reconhece o rectângulo 8:1 (o Xournal++ também não): o quadro segue-a.
const VECTOR_LAW: [(&str, &str); 1] = [("rect_estreito_500x60", "ink_raw")];

/// ⭐⭐ A caneta do quadro (segurar parado) contra o «Draw to shape» do Excalidraw nos 80 traços: onde
/// eles reconhecem, damos a MESMA forma, salvo as divergências escritas acima; onde eles não reconhecem
/// («nenhum», e o traço SOME da cena deles) fica a tinta, nunca nada. E o controlo, o «Draw» deles
/// (traço livre 80/80): largar sem segurar deixa SEMPRE o traço cru, um elemento só.
#[test]
fn the_pen_held_still_is_their_draw_to_shape_but_for_the_written_divergences() {
    let input = json("entradas/bloco6_traco.json");
    let out = json("saidas/bloco6_traco.next.json");
    assert_eq!(out["cabecalho"]["entrada"], "bloco6_traco.json");
    assert_eq!(out["cabecalho"]["versao"], "0.18.0-143b5b6");
    let theirs = |tool: &str| -> BTreeMap<String, String> {
        out["resultados"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["ferramenta"] == tool)
            .filter_map(|r| {
                Some((
                    r["traco"].as_str()?.to_owned(),
                    r["tipo"].as_str()?.to_owned(),
                ))
            })
            .collect()
    };
    let (shape, free) = (theirs("autoshape"), theirs("freedraw"));
    let written: BTreeMap<&str, &str> = DIVERGE
        .iter()
        .map(|&(n, w, _)| (n, w))
        .chain(VECTOR_LAW)
        .collect();
    let strokes = input["tracos"].as_array().unwrap();
    assert_eq!((strokes.len(), shape.len(), free.len()), (80, 80, 80));
    let (mut agree, mut none) = (0, 0);
    for t in strokes {
        let name = t["nome"].as_str().unwrap();
        let pts: Vec<[f64; 2]> = t["pontos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
            .collect();
        let (held, n) = draw(&pts, true);
        assert_eq!(n, 1, "{name}: um elemento só");
        let want = &shape[name];
        if let Some(w) = written.get(name) {
            assert_eq!(held, *w, "{name}: a divergência escrita (eles: {want})");
            assert_ne!(held, *want, "{name}: já não diverge — tire-o da lista");
        } else if want == "nenhum" {
            assert!(
                held.starts_with("ink_"),
                "{name}: eles apagam o traço; nós deixamos tinta, não {held}"
            );
            none += 1;
        } else {
            assert_eq!(held, *want, "{name}: o mesmo que o Draw to shape");
            agree += 1;
        }
        assert_eq!(free[name], "freedraw", "{name}: o controlo deles");
        assert_eq!(
            draw(&pts, false),
            ("ink_raw".into(), 1),
            "{name}: sem segurar"
        );
    }
    assert!(
        written
            .keys()
            .all(|n| strokes.iter().any(|t| t["nome"] == *n)),
        "um nome da lista que não está na entrada"
    );
    assert_eq!((agree, none), (54, 12), "concordam · eles não reconhecem");
}
