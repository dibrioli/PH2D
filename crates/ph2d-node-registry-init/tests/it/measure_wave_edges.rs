//! **SONDA — o que a folha 06 linha 36 pede já está no catálogo?**
//!
//! A célula marca `P1` no `motion.wave` pedindo **quatro** coisas de uma vez —
//! *Reflect Edges (on/off) · Pre-roll · falloff/decay espacial · Narrowness/Width*
//! (AE *Wave World* ▸ Simulation; Blender *Wave modifier*) — e justifica-se com
//! *"**NÃO** — todos vivem dentro do kernel (borda Neumann hardcoded, sim começa
//! fria)"*. ⚠️ Essa frase é verdadeira sobre o KERNEL e não responde à pergunta da
//! conferência, que é sobre o **catálogo**: o Grupo P (2026-08-16) deu ao
//! `motion.drive` uma coluna que o artista batiza, e a wave de 18/08 mediu que
//! `wave_h` viaja por ela.
//!
//! Esta sonda mede as três perguntas ANTES de escrever knob nenhum:
//!
//! 1. **A LARGURA do anel** (*Narrowness/Width*) — num PDE o comprimento de onda é
//!    `λ = c / f`, e a frequência da fonte é o `period` da `value.lfo` que o artista
//!    já liga na porta `drive`. Se dois períodos dão dois λ, o knob existe desde o
//!    primeiro dia — noutro nó.
//! 2. **O DECAIMENTO ESPACIAL** (*Falloff*) e a **BORDA ABSORVENTE** (*Reflect
//!    Edges = off*) — as duas são a MESMA coisa num campo dinâmico: um amortecimento
//!    que varia no espaço. A cadeia é `wave.out --pre--> motion.falloff(invert) -->
//!    motion.drive(Custom "wave_h", Mul, scale=0) --> wave.state`, e o mecanismo é a
//!    **máscara do próprio drive**: ele mistura `h*(1−f) + alvo*f`, e com o alvo em
//!    zero isso é literalmente `h *= (1 − f)` por tique.
//! 3. **O CONTROLE que torna as duas legíveis:** a borda de hoje de facto REFLETE?
//!    Sem isto, *"a composição absorve"* poderia ser satisfeito por um campo que
//!    nunca chegou à borda.
//!
//! Ela **imprime e não afirma** (o molde do `measure_*`): o veredito da folha sai
//! dos números. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_wave_edges -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const SIDE: usize = 21;
const SPACING: f32 = 0.5;
const DT: f64 = 1.0 / 60.0;
/// Meia-largura da grade em unidades de mundo: `(SIDE−1)·SPACING/2`.
const HALF: f32 = (SIDE as f32 - 1.0) * SPACING * 0.5;

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

/// A grade de sempre, com a fonte de sempre: uma `value.lfo` no pino do centro.
fn wave_driven(g: &mut Graph, period: f32) -> NodeId {
    wave_driven_side(g, period, SIDE)
}

fn wave_driven_side(g: &mut Graph, period: f32, side: usize) -> NodeId {
    let w = g.add_node("motion.wave");
    for (k, v) in [
        ("rows", side as f32),
        ("cols", side as f32),
        ("spacing", SPACING),
        ("speed", 0.35),
        ("damping", 0.02),
    ] {
        g.set_param(w, k, v);
    }
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", period);
    g.set_param(lfo, "amplitude", 1.0);
    wire(g, lfo, 0, w, 0, false);
    w
}

/// **O laço `out --pre--> state`, e ele não é opcional.**
///
/// ⚠️ O editor auto-liga esta aresta ao adicionar o nó; um `Graph` cru **não**. Sem
/// ela o campo re-semeia PLANO em todo tique e o controle lê `0,000000` em toda
/// célula — e aí qualquer cadeia que feche o laço parece *"produzir"* um campo que
/// na verdade só existe porque o controle não tinha nenhum. A primeira corrida
/// desta sonda caiu exactamente nisso.
fn close_loop(g: &mut Graph, w: NodeId) {
    wire(g, w, 0, w, 1, true);
}

/// Coze `ticks` tiques e devolve `(wave_h, P)` do último.
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId, ticks: usize) -> (Vec<f32>, Vec<[f32; 2]>) {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..ticks {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead).expect("o tique avanca");
        let out = cook.cook(g, reg, node, playhead).expect("o campo coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida e' um stream")
        };
        last = s.clone();
    }
    let col = |n: &str| match last.get(n) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    let p = match last.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    (col("wave_h"), p)
}

fn energy(h: &[f32]) -> f32 {
    h.iter().map(|x| x * x).sum()
}

fn max_abs(v: &[f32]) -> f32 {
    v.iter().fold(0.0f32, |m, x| m.max(x.abs()))
}

/// A meia-linha do meio, do centro para a direita — a leitura de um corte radial.
fn ray(h: &[f32]) -> Vec<f32> {
    ray_side(h, SIDE)
}

fn ray_side(h: &[f32], side: usize) -> Vec<f32> {
    let r = side / 2;
    (side / 2..side).map(|c| h[r * side + c]).collect()
}

fn print_ray(tag: &str, h: &[f32]) {
    let row: Vec<String> = ray(h).iter().map(|x| format!("{x:+.3}")).collect();
    eprintln!("  {tag:<12} {}", row.join(" "));
}

/// As distâncias (em unidades de mundo) dos cruzamentos de zero ao longo do raio —
/// duas delas consecutivas medem **meio** comprimento de onda.
fn zero_crossings_side(h: &[f32], side: usize) -> Vec<f32> {
    let r = ray_side(h, side);
    let mut out = Vec::new();
    for i in 1..r.len() {
        if (r[i - 1] < 0.0) != (r[i] < 0.0) {
            // Interpola linearmente entre as duas células.
            let t = r[i - 1].abs() / (r[i - 1].abs() + r[i].abs()).max(1e-9);
            out.push((i as f32 - 1.0 + t) * SPACING);
        }
    }
    out
}

/// **PERGUNTA 1 — a LARGURA do anel é o `period` da fonte.**
///
/// Um PDE hiperbólico emite `λ = c/f`: dobrar o período da fonte dobra o
/// comprimento de onda. Se isto medir, *Narrowness/Width* não é um knob que falta
/// neste nó — é um knob que já existe **no nó que o artista liga na porta**.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn the_ring_width_is_the_sources_period() {
    // ⚠️ Grade GRANDE e leitura CEDO, de propósito: a 21x21 a frente alcança a parede
    // em ~17 tiques e volta, e o raio passa a medir a INTERFERÊNCIA em vez do
    // comprimento de onda emitido. A 61x61 (meia-largura 15) ela volta depois de ~100.
    const BIG: usize = 61;
    const EARLY: usize = 60;
    let reg = registry();
    eprintln!("\n[1] LARGURA DO ANEL vs `period` da value.lfo (grade {BIG}x{BIG}, {EARLY} tiques)");
    for period in [0.10f32, 0.15, 0.20, 0.30] {
        let mut g = Graph::new();
        let w = wave_driven_side(&mut g, period, BIG);
        close_loop(&mut g, w);
        let (h, _) = settle(&g, &reg, w, EARLY);
        let zc = zero_crossings_side(&h, BIG);
        let halves: Vec<String> = zc
            .windows(2)
            .map(|w| format!("{:.2}", w[1] - w[0]))
            .collect();
        let mean = if halves.is_empty() {
            0.0
        } else {
            (zc[zc.len() - 1] - zc[0]) / (zc.len() - 1) as f32
        };
        eprintln!(
            "  period {period:.2}  max |h| = {:.4}  meia-onda MEDIA {mean:.3}",
            max_abs(&h)
        );
        eprintln!("    meias-ondas: {}", halves.join(" "));
    }
    eprintln!("  => se a meia-onda MEDIA escala com o periodo, a largura ja e' um knob.");
}

/// A cadeia do decaimento espacial: `wave.out --pre--> motion.falloff(invert) -->
/// motion.drive(Custom "wave_h", Mul, scale=0) --> wave.state`.
///
/// `radius` mede de onde a esponja começa a morder; `also_prev` diz se o `wave_prev`
/// leva o mesmo tratamento — o leapfrog lê `2h − h_prev`, então amortecer só metade
/// do par é uma pergunta de mecanismo, não um detalhe.
fn sponged(radius: f32, also_prev: bool) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let w = wave_driven(&mut g, 0.5);

    let fall = g.add_node("motion.falloff");
    g.set_param(fall, "shape", 0.0); // Circle
    g.set_param(fall, "curve", 2.0); // Smooth
    g.set_param(fall, "radius", radius);
    g.set_param(fall, "invert", 1.0); // 0 no centro, 1 na borda: a ESPONJA
    wire(&mut g, w, 0, fall, 0, true);

    // O valor é irrelevante — `scale = 0` zera-o —, mas a porta quer um produtor.
    let rd = g.add_node("value.attribute");
    g.set_param(rd, "mode", 0.0);
    g.set_text_param(rd, "attr", "falloff");
    wire(&mut g, fall, 0, rd, 0, false);

    let mut prev = fall;
    for col in ["wave_h", "wave_prev"].iter().take(1 + also_prev as usize) {
        let dr = g.add_node("motion.drive");
        g.set_param(dr, "channel", 9.0); // Custom
        g.set_param(dr, "mode", 2.0); // Multiply
        g.set_param(dr, "scale", 0.0); // alvo = 0 ⇒ a mistura é `h*(1−f)`
        g.set_text_param(dr, "column", *col);
        wire(&mut g, prev, 0, dr, 0, false);
        wire(&mut g, rd, 0, dr, 1, false);
        prev = dr;
    }
    wire(&mut g, prev, 0, w, 1, false);
    (g, w)
}

/// A ESPONJA DE BORDA LIMPA: `field.box(invert)` vale **zero EXACTO** dentro da caixa,
/// e `h*(1−0)` é `h` ao bit ⇒ o miolo do campo fica intocado e só a moldura absorve.
fn sponged_box(half: f32, soft: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let w = wave_driven(&mut g, 0.5);

    let bx = g.add_node("field.box");
    g.set_param(bx, "width", half * 2.0);
    g.set_param(bx, "height", half * 2.0);
    g.set_param(bx, "soft", soft);
    g.set_param(bx, "invert", 1.0);
    wire(&mut g, w, 0, bx, 0, true);

    let rd = g.add_node("value.attribute");
    g.set_param(rd, "mode", 0.0);
    g.set_text_param(rd, "attr", "falloff");
    wire(&mut g, bx, 0, rd, 0, false);

    let mut prev = bx;
    for col in ["wave_h", "wave_prev"] {
        let dr = g.add_node("motion.drive");
        g.set_param(dr, "channel", 9.0);
        g.set_param(dr, "mode", 2.0);
        g.set_param(dr, "scale", 0.0);
        g.set_text_param(dr, "column", col);
        wire(&mut g, prev, 0, dr, 0, false);
        wire(&mut g, rd, 0, dr, 1, false);
        prev = dr;
    }
    wire(&mut g, prev, 0, w, 1, false);
    (g, w)
}

/// **PERGUNTA 2 — o decaimento espacial (e a borda absorvente) por COMPOSIÇÃO.**
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn the_spatial_decay_comes_out_of_the_drives_own_mask() {
    let reg = registry();
    eprintln!("\n[2] ESPONJA por composicao (falloff invertido -> drive Mul, alvo 0)");

    let mut g0 = Graph::new();
    let w0 = wave_driven(&mut g0, 0.5);
    close_loop(&mut g0, w0);
    let (bare, pos) = settle(&g0, &reg, w0, 240);
    eprintln!(
        "  controle     energia {:.4}  max |h| {:.4}",
        energy(&bare),
        max_abs(&bare)
    );
    print_ray("controle", &bare);

    for (radius, also_prev) in [(5.0f32, false), (5.0, true), (3.0, true)] {
        let (g, w) = sponged(radius, also_prev);
        let (h, _) = settle(&g, &reg, w, 240);
        let tag = format!("r={radius:.0} prev={}", also_prev as u8);
        eprintln!(
            "  {tag:<12} energia {:.4}  max |h| {:.4}",
            energy(&h),
            max_abs(&h)
        );
        print_ray(&tag, &h);
    }

    // A borda: quanto do campo vive na moldura de duas células?
    let border = |h: &[f32]| {
        let mut m = 0.0f32;
        for (i, p) in pos.iter().enumerate() {
            if p[0].abs() > HALF - 2.0 * SPACING || p[1].abs() > HALF - 2.0 * SPACING {
                m = m.max(h[i].abs());
            }
        }
        m
    };
    eprintln!(
        "  max |h| na moldura de 2 celulas — controle {:.4}",
        border(&bare)
    );
    let (g, w) = sponged(5.0, true);
    let (h, _) = settle(&g, &reg, w, 240);
    eprintln!(
        "  max |h| na moldura de 2 celulas — esponja  {:.4}",
        border(&h)
    );

    eprintln!("\n  -- a esponja de BORDA LIMPA (field.box invertido) --");
    for (half, soft) in [(4.0f32, 0.6f32), (4.0, 1.2), (3.5, 0.8)] {
        let (g, w) = sponged_box(half, soft);
        let (h, _) = settle(&g, &reg, w, 240);
        let tag = format!("box {half:.1}/{soft:.1}");
        eprintln!(
            "  {tag:<12} energia {:.4}  max |h| {:.4}  moldura {:.4}",
            energy(&h),
            max_abs(&h),
            border(&h)
        );
        print_ray(&tag, &h);
        // Quantas celulas do MIOLO ficam byte-identicas ao controle?
        let same = pos
            .iter()
            .enumerate()
            .filter(|(_, p)| p[0].abs() <= half - 1.0 && p[1].abs() <= half - 1.0)
            .filter(|(i, _)| h[*i].to_bits() == bare[*i].to_bits())
            .count();
        let inner = pos
            .iter()
            .filter(|p| p[0].abs() <= half - 1.0 && p[1].abs() <= half - 1.0)
            .count();
        eprintln!("    miolo byte-identico ao controle: {same} de {inner}");
    }
    eprintln!("  => se a esponja mata a moldura e deixa o centro vivo, `reflect=off` ja existe.");
}

/// **PERGUNTA 3 — o CONTROLE: a borda de hoje de facto REFLETE.**
///
/// Um pulso único, e depois silêncio: com bordas de Neumann a energia fica presa e
/// volta ao centro; se ela simplesmente se dissipasse, a pergunta 2 não teria
/// fenômeno nenhum para dissolver.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn todays_edge_really_does_bounce() {
    let reg = registry();
    eprintln!("\n[3] CONTROLE — a borda de hoje reflete? (pulso unico, damping 0)");

    let mut g = Graph::new();
    let w = g.add_node("motion.wave");
    for (k, v) in [
        ("rows", SIDE as f32),
        ("cols", SIDE as f32),
        ("spacing", SPACING),
        ("speed", 0.35),
        ("damping", 0.0),
    ] {
        g.set_param(w, k, v);
    }
    // Um pulso: a `value.lfo` Spike é unipolar e curta; um período longo faz dela
    // uma fonte quase-única dentro da janela medida.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "wave", 4.0); // Spike
    g.set_param(lfo, "period", 8.0);
    g.set_param(lfo, "amplitude", 1.0);
    wire(&mut g, lfo, 0, w, 0, false);
    close_loop(&mut g, w);

    for ticks in [30usize, 60, 120, 240] {
        let (h, _) = settle(&g, &reg, w, ticks);
        eprintln!(
            "  {ticks:>3} tiques  energia {:.4}  max |h| {:.4}",
            energy(&h),
            max_abs(&h)
        );
        print_ray(&format!("t={ticks}"), &h);
    }
    eprintln!("  => energia que NAO cai e um raio que reacende sao a assinatura do ricochete.");
}
