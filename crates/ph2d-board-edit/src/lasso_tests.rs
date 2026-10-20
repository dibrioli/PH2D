//! ⭐ W23 B12 — o laço pelo caminho de quem o usa (doc 17 §2.1): entra quem tem ≥ 90 % da área dentro
//! (o Miro), pela MESMA porta do rectângulo (travados fora, grupos inteiros), Shift soma, Esc devolve.

use super::*;
use std::collections::BTreeSet;
use crate::select_region::{LASSO_COVER, Region};

/// O contorno de `[x0, y0, x1, y1]` a cada 10 unidades (o rasto de um rato), a fechar no início.
fn square_trail(b: [f64; 4]) -> Vec<[f64; 2]> {
    let mut v = Vec::new();
    let steps = |a: f64, z: f64| (0..=((z - a) / 10.0) as usize).map(move |i| a + 10.0 * i as f64);
    v.extend(steps(b[0], b[2]).map(|x| [x, b[1]]));
    v.extend(steps(b[1], b[3]).map(|y| [b[2], y]));
    v.extend(steps(b[0], b[2]).rev().map(|x| [x, b[3]]));
    v.extend(steps(b[1], b[3]).rev().map(|y| [b[0], y]));
    v
}

impl World {
    fn trace(&mut self, trail: &[[f64; 2]], m: Mods) {
        let p0 = trail[0];
        self.ed.pointer_down(
            &mut self.doc,
            &mut self.h,
            &mut self.ts,
            at(p0[0], p0[1], m),
            VIEW,
        );
        for p in &trail[1..] {
            self.ed
                .pointer_move(&mut self.doc, &mut self.ts, at(p[0], p[1], m));
        }
        let last = trail[trail.len() - 1];
        self.ed
            .pointer_up(&mut self.doc, &mut self.h, at(last[0], last[1], m));
    }

    fn picked(&self) -> BTreeSet<ElementId> {
        self.ed.selection().clone()
    }
}

/// O laço `[0, 0]–[1000, 1000]` e quatro caixas de 100 × 100: dentro a 100 %, 95 %, 89 % e 60 %.
fn four_boxes() -> (World, [ElementId; 4]) {
    let mut w = world();
    let ids = [
        w.rect([100.0, 100.0, 100.0, 100.0]),
        w.rect([905.0, 300.0, 100.0, 100.0]),
        w.rect([911.0, 500.0, 100.0, 100.0]),
        w.rect([940.0, 700.0, 100.0, 100.0]),
    ];
    w.cmd(Command::Escape);
    w.ed.tool = Tool::Lasso;
    (w, ids)
}

#[test]
fn the_lasso_takes_what_is_ninety_percent_inside() {
    let (mut w, [full, most, short, half]) = four_boxes();
    w.trace(&square_trail([0.0, 0.0, 1_000.0, 1_000.0]), NONE);
    let s = w.picked();
    assert!(s.contains(&full) && s.contains(&most), "{s:?}");
    assert!(!s.contains(&short), "89 % dentro não entra");
    assert!(!s.contains(&half));
    assert_eq!(w.ed.tool, Tool::Lasso, "a ferramenta fica (é um modo, como a caneta)");
}

/// A mesma porta do rectângulo: o travado fica de fora, quem é de um grupo traz o grupo.
#[test]
fn the_lasso_and_the_marquee_ask_one_door() {
    let (mut w, [full, most, _, half]) = four_boxes();
    w.ed.select(&w.doc, [most]);
    assert!(w.cmd(Command::Lock));
    w.ed.select(&w.doc, [full, half]);
    assert!(w.cmd(Command::Group));
    w.cmd(Command::Escape);
    w.ed.tool = Tool::Lasso;
    w.trace(&square_trail([0.0, 0.0, 1_000.0, 1_000.0]), NONE);
    let s = w.picked();
    assert!(!s.contains(&most), "o travado entrou");
    assert!(s.contains(&full) && s.contains(&half), "o grupo vem inteiro: {s:?}");
}

#[test]
fn shift_adds_and_escape_gives_the_selection_back() {
    let (mut w, [full, _, _, half]) = four_boxes();
    w.ed.select(&w.doc, [half]);
    w.trace(&square_trail([0.0, 0.0, 400.0, 400.0]), SHIFT);
    assert_eq!(w.picked(), BTreeSet::from([full, half]));
    // A meio de um laço com Shift, Esc devolve o que havia (sem Shift o laço começa por limpar, como
    // o rectângulo).
    w.ed.select(&w.doc, [half]);
    let t = square_trail([0.0, 0.0, 400.0, 400.0]);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(t[0][0], t[0][1], SHIFT), VIEW);
    for p in &t[1..] {
        w.ed.pointer_move(&mut w.doc, &mut w.ts, at(p[0], p[1], SHIFT));
    }
    assert!(w.picked().contains(&full));
    assert!(w.cmd(Command::Escape));
    assert_eq!(w.picked(), BTreeSet::from([half]));
}

/// `Ctrl+Alt`+arrastar na Selecção é o laço — MESMO a começar em cima de uma forma (não a move).
#[test]
fn ctrl_alt_drag_with_select_is_the_lasso() {
    let (mut w, [full, ..]) = four_boxes();
    w.ed.tool = Tool::Select;
    let before = [w.el(full).x, w.el(full).y];
    let ctrl_alt = Mods {
        shift: false,
        ctrl: true,
        alt: true,
    };
    let mut t = vec![[150.0, 150.0]];
    t.extend(square_trail([0.0, 0.0, 400.0, 400.0]));
    w.trace(&t, ctrl_alt);
    assert_eq!([w.el(full).x, w.el(full).y], before, "moveu a forma");
    assert_eq!(w.picked(), BTreeSet::from([full]));
    assert_eq!(w.doc.live_len(), 4, "o Alt não duplicou");
}

/// Uma seta conta pelo COMPRIMENTO: a que sai do laço a meio não entra; a que fica dentro, sim.
#[test]
fn an_arrow_counts_by_its_length() {
    let (mut w, _) = four_boxes();
    w.ed.tool = Tool::Connector;
    w.drag([300.0, 800.0], [600.0, 800.0], CTRL);
    let inside = *w.ed.selection().iter().next().unwrap();
    w.ed.tool = Tool::Connector;
    w.drag([700.0, 900.0], [1_300.0, 900.0], CTRL);
    let across = *w.ed.selection().iter().next().unwrap();
    w.cmd(Command::Escape);
    w.ed.tool = Tool::Lasso;
    w.trace(&square_trail([0.0, 0.0, 1_000.0, 1_000.0]), NONE);
    let s = w.picked();
    assert!(s.contains(&inside), "{s:?}");
    assert!(!s.contains(&across));
}

/// Uma faixa de meia largura 20 à volta do segmento `a`–`b`, prolongada 10 de cada ponta.
fn band(a: [f64; 2], b: [f64; 2]) -> Vec<[f64; 2]> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l = dx.hypot(dy);
    let (d, n) = ([dx / l, dy / l], [-dy / l, dx / l]);
    let at = |p: [f64; 2], s: f64, t: f64| [p[0] + s * d[0] + t * n[0], p[1] + s * d[1] + t * n[1]];
    vec![at(a, -10.0, -20.0), at(b, 10.0, -20.0), at(b, 10.0, 20.0), at(a, -10.0, 20.0)]
}

/// Uma tinta recta de `a` a `b` (onze pontos), rodada de `angle` à volta do centro dela.
fn ink(w: &mut World, a: [f64; 2], b: [f64; 2], angle: f64) -> ElementId {
    use ph2d_board_model::{BoardOp, Ink, Pen, Style};
    let pts: Vec<[f64; 3]> = (0..=10)
        .map(|i| {
            let t = f64::from(i) / 10.0;
            [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]), 0.5]
        })
        .collect();
    let style = Style::new(None, Some(Rgba([0, 0, 0, 255])), Rgba([0, 0, 0, 255]));
    let (stroke, bx) = Ink::from_world(&pts, style, Pen::Pen, false);
    let (id, z) = (w.doc.mint_id(), w.doc.z_on_top());
    let mut el = Element::new_ink(id, z, stroke, bx);
    el.angle = angle;
    let _ = BoardOp::Put(el).apply(&mut w.doc);
    id
}

fn lassoed(w: &mut World, trail: &[[f64; 2]]) -> BTreeSet<ElementId> {
    w.ed.lasso_pick(&w.doc, trail).into_iter().collect()
}

/// ⭐ O que está na CAIXA do laço mas no entalhe de um «C», sem aresta a tocá-lo, fica de fora (o
/// atalho conta pelo centro, não «dentro da caixa»); o da parte cheia entra.
#[test]
fn what_sits_in_the_notch_of_a_c_stays_out() {
    let (mut w, [full, ..]) = four_boxes();
    let notch = w.rect([600.0, 450.0, 100.0, 100.0]);
    let c = vec![
        [0.0, 0.0],
        [1_000.0, 0.0],
        [1_000.0, 300.0],
        [400.0, 300.0],
        [400.0, 700.0],
        [1_000.0, 700.0],
        [1_000.0, 1_000.0],
        [0.0, 1_000.0],
    ];
    let s = lassoed(&mut w, &c);
    assert!(s.contains(&full), "{s:?}");
    assert!(!s.contains(&notch), "o do entalhe entrou");
}

/// ⭐ Uma forma RODADA conta pelo contorno rodado: o quadrado de 100 rodado 45° dentro de um laço que
/// é o mesmo quadrado direito tem `2√2 − 2` ≈ 82,8 % dentro — não entra (a 80 % entra).
#[test]
fn a_turned_shape_counts_by_its_turned_outline() {
    use ph2d_board_model::BoardOp;
    let mut w = world();
    let id = w.rect([450.0, 450.0, 100.0, 100.0]);
    let mut el = w.el(id).clone();
    el.angle = std::f64::consts::FRAC_PI_4;
    let _ = BoardOp::Put(el).apply(&mut w.doc);
    let trail = square_trail([450.0, 450.0, 550.0, 550.0]);
    assert!(!lassoed(&mut w, &trail).contains(&id), "contou o quadrado direito");
    assert!(w.ed.lasso_pick_covering(&w.doc, &trail, 0.8).contains(&id));
}

/// O frame RODADO (conta pela caixa, mas a caixa rodada) — a mesma conta do quadrado a 45°.
#[test]
fn a_turned_frame_counts_by_its_turned_box() {
    use ph2d_board_model::BoardOp;
    let mut w = world();
    w.ed.tool = Tool::Frame;
    w.drag([450.0, 450.0], [550.0, 550.0], NONE);
    let id = *w.ed.selection().iter().next().unwrap();
    w.cmd(Command::Escape);
    let mut el = w.el(id).clone();
    assert!(el.as_frame().is_some(), "a fixtura precisa de um frame");
    el.angle = std::f64::consts::FRAC_PI_4;
    let _ = BoardOp::Put(el).apply(&mut w.doc);
    let trail = square_trail([450.0, 450.0, 550.0, 550.0]);
    assert!(!lassoed(&mut w, &trail).contains(&id), "contou a caixa direita");
    assert!(w.ed.lasso_pick_covering(&w.doc, &trail, 0.8).contains(&id));
}

/// ⭐ A tinta conta pelo COMPRIMENTO (não pela área da caixa) e pelos pontos RODADOS: uma faixa à
/// volta da diagonal leva a tinta diagonal e a horizontal rodada 45°, nunca a horizontal direita.
#[test]
fn ink_counts_by_its_length_along_its_turned_points() {
    let mut w = world();
    let (a, b) = ([400.0, 400.0], [600.0, 600.0]);
    let diagonal = ink(&mut w, a, b, 0.0);
    let half = 100.0 * std::f64::consts::SQRT_2;
    let flat = [[500.0 - half, 500.0], [500.0 + half, 500.0]];
    let turned = ink(&mut w, flat[0], flat[1], std::f64::consts::FRAC_PI_4);
    let straight = ink(&mut w, flat[0], flat[1], 0.0);
    let s = lassoed(&mut w, &band(a, b));
    assert!(s.contains(&diagonal), "contou a caixa e não o comprimento: {s:?}");
    assert!(s.contains(&turned), "não rodou os pontos");
    assert!(!s.contains(&straight));
}

/// Uma seta na diagonal conta pela ROTA: a faixa à volta dela leva-a toda, embora tape pouco da caixa.
#[test]
fn a_diagonal_arrow_counts_by_its_route() {
    let mut w = world();
    w.ed.tool = Tool::Connector;
    w.drag([100.0, 100.0], [900.0, 900.0], CTRL);
    let arrow = *w.ed.selection().iter().next().unwrap();
    w.cmd(Command::Escape);
    let mut el = w.el(arrow).clone();
    el.connector_mut().unwrap().route = ph2d_board_model::Route::Straight;
    let _ = ph2d_board_model::BoardOp::Put(el).apply(&mut w.doc);
    let s = lassoed(&mut w, &band([100.0, 100.0], [900.0, 900.0]));
    assert!(s.contains(&arrow), "{s:?}");
}

/// O limiar é 90 %: 92 % dentro entra (o 85 % e o 89 % já se provam acima).
#[test]
fn ninety_two_percent_inside_is_in() {
    let mut w = world();
    let id = w.rect([908.0, 100.0, 100.0, 100.0]);
    w.cmd(Command::Escape);
    assert!(lassoed(&mut w, &square_trail([0.0, 0.0, 1_000.0, 1_000.0])).contains(&id));
}

/// ⭐ M2 (doc 17 §5) — quanto custa UM movimento do laço num quadro de 100 mil caixas, contra o
/// rectângulo da MESMA caixa: variantes no mesmo processo, intercaladas em 7 rodadas, vale o MÍNIMO
/// (`docs/DevOps/MEDIR_VELOCIDADE.md`). Corre-se à mão:
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-board-edit --profile smoke --lib measure_lasso -- --ignored --nocapture`
#[test]
#[ignore = "sonda de velocidade (M2), corre-se à mão"]
fn measure_lasso_move_on_a_hundred_thousand() {
    use ph2d_board_model::{BoardOp, RichText, Shape, ShapeType, Style};
    use std::time::Instant;
    let mut w = world();
    let style = Style::new(Some(Rgba([255, 255, 255, 255])), None, Rgba([0, 0, 0, 255]));
    for i in 0..100_000u32 {
        let (cx, cy) = (f64::from(i % 316) * 50.0, f64::from(i / 316) * 50.0);
        let id = w.doc.mint_id();
        let z = w.doc.z_on_top();
        let s = Shape {
            kind: ShapeType::Rectangle,
            style: style.clone(),
            text: RichText::default(),
        };
        let _ = BoardOp::Put(Element::new_shape(id, z, s, [cx, cy, 40.0, 30.0])).apply(&mut w.doc);
    }
    let c = [7_900.0, 7_900.0];
    let ring = |n: u32, r: f64| -> Vec<[f64; 2]> {
        (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * f64::from(i) / f64::from(n);
                [c[0] + r * a.cos(), c[1] + r * a.sin()]
            })
            .collect()
    };
    let (r, box_) = (800.0, [c[0] - 800.0, c[1] - 800.0, c[0] + 800.0, c[1] + 800.0]);
    let trails = [("laço 600 pts", ring(600, r)), ("laço 2500 pts", ring(2_500, r))];
    let mut best = [f64::INFINITY; 3];
    let mut got = [0usize; 3];
    for round in 0..7 {
        for k in 0..3 {
            let v = (k + round) % 3;
            let t0 = Instant::now();
            let n = match v {
                0 => w.ed.inside(&w.doc, Region::Rect(box_)).len(),
                _ => w
                    .ed
                    .inside(&w.doc, Region::Lasso(&trails[v - 1].1, LASSO_COVER))
                    .len(),
            };
            best[v] = best[v].min(t0.elapsed().as_secs_f64() * 1e3);
            got[v] = n;
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("M2 · 100 000 caixas · loadavg {}", load.trim());
    println!("rectângulo (caixa {box_:?}): {:.2} ms · {} dentro", best[0], got[0]);
    for (i, (name, _)) in trails.iter().enumerate() {
        println!("{name}: {:.2} ms · {} dentro", best[i + 1], got[i + 1]);
    }
}
