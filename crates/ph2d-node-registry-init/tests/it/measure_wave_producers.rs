//! **SONDA — a composição já exprime N PRODUTORES no `motion.wave`?**
//!
//! A folha 06 linha 35 marca `P0/P1` dizendo: *"**NÃO** — dois `motion.wave` montam
//! duas grades (`rows×cols` próprios) e não somam no mesmo campo; a fonte é a
//! célula do centro, e só ela"*. ⚠️ Essa célula foi escrita em **2026-08-10** e o
//! **Grupo P** (`motion.drive(Custom…)`, 2026-08-16) mudou o catálogo **seis dias
//! depois**: hoje um `motion.drive` escreve numa **coluna que o artista batiza**, e
//! `wave_h` é uma coluna `Scalar` que **não** está no `is_bookkeeping_column`.
//!
//! Esta sonda mede as três rotas antes de construir seja o que for:
//!
//! 1. **DOIS `motion.wave`** — a metade da célula que é sobre grades separadas.
//! 2. **`wave --> wave`** encadeados no MESMO campo (a rota que a célula não cita).
//! 3. **`wave.out --pre--> field.box --> value.attribute --> motion.drive(Custom
//!    "wave_h") --> wave.state`** — o produtor por COMPOSIÇÃO que o Grupo P abriu.
//!
//! Ela **imprime e não afirma** (o molde do `measure_*`): o veredito da folha sai
//! dos números, não desta prosa. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_wave_producers -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const SIDE: usize = 21;
const SPACING: f32 = 0.5;
const TICKS: usize = 240;
const DT: f64 = 1.0 / 60.0;

/// O centro do produtor injetado, em unidades de MUNDO. A grade vai de `-5` a `+5`
/// em x (`(SIDE-1)*SPACING/2`), então isto é a coluna 4 da linha do meio.
const BOX_X: f32 = -3.0;
const BOX_Y: f32 = 0.0;
/// Quanto o campo injetado alcança — usado só para separar *dentro da máscara* de
/// *longe dela* na leitura, nunca como limiar de veredito.
const BOX_REACH: f32 = 1.2;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// Um `motion.wave` com o drive de sempre (uma `value.lfo` no centro).
fn wave_with_lfo(g: &mut Graph, cx: f32) -> NodeId {
    let w = g.add_node("motion.wave");
    g.set_param(w, "rows", SIDE as f32);
    g.set_param(w, "cols", SIDE as f32);
    g.set_param(w, "spacing", SPACING);
    g.set_param(w, "speed", 0.35);
    g.set_param(w, "damping", 0.02);
    g.set_param(w, "center_x", cx);
    g.set_param(w, "center_y", 0.0);

    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.5);
    g.set_param(lfo, "amplitude", 1.0);
    wire(g, lfo, 0, w, 0, false);
    w
}

/// Coze `TICKS` tiques e devolve o `wave_h` final do nó pedido.
///
/// ⚠️ O `advance_tick` é o que faz a aresta `pre` CARREGAR estado — sem ele a
/// cadeia nunca integra e todo gate de feedback fica verde por vácuo (a lição que
/// o `rope_thickness` pagou duas vezes).
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId) -> (Vec<f32>, Vec<[f32; 2]>) {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead).expect("o tique avanca");
        let out = cook.cook(g, reg, node, playhead).expect("o campo coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida e' um stream")
        };
        last = s.clone();
    }
    let h = match last.get("wave_h") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    let p = match last.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    (h, p)
}

fn max_abs(v: &[f32]) -> f32 {
    v.iter().fold(0.0f32, |m, x| m.max(x.abs()))
}

fn dist(p: [f32; 2], c: [f32; 2]) -> f32 {
    let d = [p[0] - c[0], p[1] - c[1]];
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// Imprime a linha do meio de um campo (a leitura que um humano confere).
fn print_mid_row(tag: &str, h: &[f32]) {
    let r = SIDE / 2;
    let row: Vec<String> = (0..SIDE)
        .map(|c| format!("{:+.3}", h[r * SIDE + c]))
        .collect();
    eprintln!("  {tag:<10} {}", row.join(" "));
}

/// **ROTA 1 — dois `motion.wave` são duas GRADES.** A metade da célula 35 que é
/// verificável sem nada novo: o segundo campo não vê o primeiro.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn two_waves_are_two_grids() {
    let reg = registry();

    let mut g = Graph::new();
    let a = wave_with_lfo(&mut g, -3.0);
    wire(&mut g, a, 0, a, 1, true);
    // O segundo campo NAO tem drive: se ele visse o campo do primeiro, ondularia.
    let b = g.add_node("motion.wave");
    g.set_param(b, "rows", SIDE as f32);
    g.set_param(b, "cols", SIDE as f32);
    g.set_param(b, "spacing", SPACING);
    g.set_param(b, "center_x", 3.0);
    wire(&mut g, b, 0, b, 1, true);

    let (ha, _) = settle(&g, &reg, a);
    let (hb, _) = settle(&g, &reg, b);
    eprintln!("\n[rota 1] DOIS motion.wave lado a lado (o segundo SEM drive)");
    eprintln!("  campo A (com drive):  max |h| = {:.6}", max_abs(&ha));
    eprintln!("  campo B (sem  drive): max |h| = {:.6}", max_abs(&hb));
    eprintln!("  => se B e' 0, sao duas grades: nenhuma soma no mesmo campo.");
}

/// **ROTA 2 — encadear `wave --> wave` no MESMO campo.** A rota que a célula não
/// cita: o segundo nó recebe o campo do primeiro pela porta `state`.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn chaining_two_waves_onto_one_field() {
    let reg = registry();

    let mut g = Graph::new();
    let a = wave_with_lfo(&mut g, 0.0);
    wire(&mut g, a, 0, a, 1, true);

    // B recebe o campo de A e tem drive PROPRIO (uma constante forte, para o efeito
    // dele ser inconfundivel se chegar).
    let b = g.add_node("motion.wave");
    for (k, v) in [
        ("rows", SIDE as f32),
        ("cols", SIDE as f32),
        ("spacing", SPACING),
        ("speed", 0.35),
        ("damping", 0.02),
    ] {
        g.set_param(b, k, v);
    }
    let lfo_b = g.add_node("value.lfo");
    g.set_param(lfo_b, "period", 0.3);
    g.set_param(lfo_b, "amplitude", 5.0);
    wire(&mut g, lfo_b, 0, b, 0, false);
    wire(&mut g, a, 0, b, 1, false);

    let (ha, _) = settle(&g, &reg, a);
    let (hb, _) = settle(&g, &reg, b);
    eprintln!("\n[rota 2] wave A --> wave B.state (B com drive PROPRIO, 5x mais forte)");
    eprintln!("  A: max |h| = {:.6}", max_abs(&ha));
    eprintln!("  B: max |h| = {:.6}", max_abs(&hb));
    print_mid_row("A", &ha);
    print_mid_row("B", &hb);
    let same = ha
        .iter()
        .zip(hb.iter())
        .all(|(x, y)| (x - y).abs() < f32::EPSILON);
    eprintln!("  B == A bit a bit? {same}  (se sim, o drive de B foi ENGOLIDO)");
}

/// **ROTA 3 — o produtor por COMPOSIÇÃO que o Grupo P abriu.**
/// `wave.out --pre--> field.box --> value.attribute("falloff") -->
///  motion.drive(Custom "wave_h", Add) --> wave.state`
fn composed(inject: bool, amp: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let w = wave_with_lfo(&mut g, 0.0);
    if !inject {
        wire(&mut g, w, 0, w, 1, true);
        return (g, w);
    }
    // ⚠️ O `pre` mora na aresta que ENTRA na cadeia, nunca na que volta ao `state`:
    // e' ela que quebra o ciclo, e os tres nos sao `Pure` => nao carimbam `sim_t`,
    // entao a onda ainda ve' o `dt` do proprio relogio no tique seguinte.
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 0.8);
    g.set_param(bx, "height", 0.8);
    g.set_param(bx, "soft", 0.3);
    g.set_param(bx, "center_x", BOX_X);
    g.set_param(bx, "center_y", BOX_Y);
    wire(&mut g, w, 0, bx, 0, true);

    let rd = g.add_node("value.attribute");
    g.set_param(rd, "mode", 0.0); // a coluna escalar, crua
    g.set_text_param(rd, "attr", "falloff");
    wire(&mut g, bx, 0, rd, 0, false);

    let dr = g.add_node("motion.drive");
    g.set_param(dr, "channel", 9.0); // Custom
    g.set_param(dr, "mode", 0.0); // Add
    g.set_param(dr, "scale", amp);
    g.set_text_param(dr, "column", "wave_h");
    wire(&mut g, bx, 0, dr, 0, false);
    wire(&mut g, rd, 0, dr, 1, false);

    wire(&mut g, dr, 0, w, 1, false);
    (g, w)
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn a_second_producer_by_composition() {
    let reg = registry();

    let (g_bare, w_bare) = composed(false, 0.0);
    let (bare, pos) = settle(&g_bare, &reg, w_bare);
    let (g_two, w_two) = composed(true, 0.6);
    let (two, _) = settle(&g_two, &reg, w_two);

    eprintln!("\n[rota 3] drive(Custom \"wave_h\") no laco de estado, caixa em ({BOX_X}, {BOX_Y})");
    eprintln!("  controle: max |h| = {:.6}", max_abs(&bare));
    eprintln!("  composto: max |h| = {:.6}", max_abs(&two));
    print_mid_row("controle", &bare);
    print_mid_row("composto", &two);

    // A diferenca LONGE da mascara e' a pergunta: um bump que nao propaga nao e' um
    // produtor, e' tinta no campo de altura.
    let mut near = 0.0f32;
    let mut far = 0.0f32;
    let mut far_cells = 0usize;
    for (i, p) in pos.iter().enumerate() {
        let d = (two[i] - bare[i]).abs();
        if dist(*p, [BOX_X, BOX_Y]) <= BOX_REACH {
            near = near.max(d);
        } else {
            far = far.max(d);
            if d > 1e-4 {
                far_cells += 1;
            }
        }
    }
    eprintln!("  |composto - controle|  DENTRO da mascara {near:.6}  FORA dela {far:.6}");
    eprintln!(
        "  celulas FORA da mascara que se moveram (> 1e-4): {far_cells} de {}",
        pos.len()
    );
    eprintln!("  => se `far` e' grande e `far_cells` e' alto, o bump PROPAGA: e' um produtor.");
}

/// **O PREÇO DO PINO DE DIRICHLET, com o CONTROLE certo.**
///
/// A geometria é idêntica nos dois braços; a única diferença é a porta `drive`
/// estar LIGADA a uma fonte silenciosa (`value.lfo` de amplitude 0 ⇒ o pino corre,
/// cravando o centro em `0`) ou DESLIGADA (⇒ fonte nenhuma, sem pino). É o A/B mais
/// limpo possível: o mesmo campo, o mesmo produtor, a mesma malha.
///
/// ⚠️ **Uma fonte ligada em zero e uma fonte ausente eram o MESMO caminho** até a
/// guarda do `step`; esta sonda é o que torna a diferença um número em vez de uma
/// afirmação.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_the_pinned_centre_costs_a_composed_field() {
    let reg = registry();

    // `pinned = true` liga uma fonte SILENCIOSA (amplitude 0): o valor chega, logo o
    // pino corre. `false` deixa a porta solta: nao chega valor nenhum.
    let build = |pinned: bool| {
        let mut g = Graph::new();
        let w = g.add_node("motion.wave");
        for (k, v) in [
            ("rows", SIDE as f32),
            ("cols", SIDE as f32),
            ("spacing", SPACING),
            ("speed", 0.35),
            ("damping", 0.02),
        ] {
            g.set_param(w, k, v);
        }
        if pinned {
            let lfo = g.add_node("value.lfo");
            g.set_param(lfo, "amplitude", 0.0);
            g.set_param(lfo, "offset", 0.0);
            wire(&mut g, lfo, 0, w, 0, false);
        }
        let bx = g.add_node("field.box");
        g.set_param(bx, "width", 0.8);
        g.set_param(bx, "height", 0.8);
        g.set_param(bx, "soft", 0.3);
        g.set_param(bx, "center_x", BOX_X);
        g.set_param(bx, "center_y", BOX_Y);
        wire(&mut g, w, 0, bx, 0, true);
        let rd = g.add_node("value.attribute");
        g.set_param(rd, "mode", 0.0);
        g.set_text_param(rd, "attr", "falloff");
        wire(&mut g, bx, 0, rd, 0, false);
        let dr = g.add_node("motion.drive");
        g.set_param(dr, "channel", 9.0);
        g.set_param(dr, "mode", 0.0);
        g.set_param(dr, "scale", 0.6);
        g.set_text_param(dr, "column", "wave_h");
        wire(&mut g, bx, 0, dr, 0, false);
        wire(&mut g, rd, 0, dr, 1, false);
        wire(&mut g, dr, 0, w, 1, false);
        (g, w)
    };

    let (gp, wp) = build(true);
    let (hp, pos) = settle(&gp, &reg, wp);
    let (gf, wf) = build(false);
    let (hf, _) = settle(&gf, &reg, wf);

    let src = (SIDE / 2) * SIDE + SIDE / 2;
    eprintln!("\n[pino] SO' o produtor injetado; a unica diferenca e' a porta `drive`");
    print_mid_row("pinado", &hp);
    print_mid_row("livre", &hf);
    eprintln!(
        "  h[centro]  pinado {:+.6}   livre {:+.6}   (vizinhas do livre: {:+.6} / {:+.6})",
        hp[src],
        hf[src],
        hf[src - 1],
        hf[src + 1]
    );

    // Quanto do campo o pino apaga, e ate' onde.
    let mut worst = 0.0f32;
    let mut worst_at = 0.0f32;
    let mut moved = 0usize;
    for i in 0..hp.len() {
        let d = (hf[i] - hp[i]).abs();
        if d > worst {
            worst = d;
            worst_at = dist(pos[i], [0.0, 0.0]);
        }
        if d > 1e-4 {
            moved += 1;
        }
    }
    eprintln!(
        "  |livre - pinado|  pior {worst:.6} a {worst_at:.2} do centro   celulas afetadas {moved} de {}",
        hp.len()
    );
    // A ENERGIA que o pino come, do outro lado do centro.
    let far_sum = |h: &[f32]| {
        pos.iter()
            .zip(h.iter())
            .filter(|(p, _)| p[0] > 1.0)
            .map(|(_, z)| z.abs())
            .sum::<f32>()
    };
    eprintln!(
        "  soma |h| do LADO OPOSTO ao produtor:  pinado {:.4}   livre {:.4}",
        far_sum(&hp),
        far_sum(&hf)
    );
}
