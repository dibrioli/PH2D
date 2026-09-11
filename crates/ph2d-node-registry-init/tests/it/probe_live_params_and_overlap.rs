//! SONDAS do smoke da cena `=50` — dois reports do Enio, medidos ANTES de
//! qualquer hipótese.
//!
//! 1. *"Os parâmetros não mudam o cenário em tempo real. Exemplo: o que está
//!    pinado, se eu mover `spacing` em Soft Body, não muda de posição até apertar
//!    reset."*
//! 2. *"Um problema antigo de Boids: não há parâmetro que estabeleça o grau de
//!    sobreposição entre os indivíduos. Ou a imagem desenhada na tela para cada
//!    indivíduo é maior do que deveria, pois eles se sobrepõem."*
//!
//! As sondas IMPRIMEM e não afirmam — elas existem para dizer o que o produto faz
//! hoje, não para pinar o que ele deveria fazer.
//!
//! ```text
//! cargo test -p ph2d-node-registry-init --test probe_live_params_and_overlap \
//!   --release -- --ignored --nocapture
//! ```

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const DT: f64 = 1.0 / 60.0;
/// O tamanho com que uma instância é DESENHADA quando o stream não traz `size`.
///
/// ⚠️ **É `SIZE_IDENTITY = 1.0`, o que o SHELL passa** (`motion_state.rs`) — e não
/// os `0.4` das fixtures de GPU, que é onde a minha primeira sonda o foi buscar.
/// O número muda o veredito inteiro: com 0.4 o bando default sobrepõe 5%, com 1.0
/// sobrepõe quase todo.
const DRAWN_SIZE: f32 = 1.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("liga");
}

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Avança `ticks` tiques a partir do tique `from`, devolvendo a pose final.
fn run(
    cook: &mut Cook,
    g: &Graph,
    reg: &NodeRegistry,
    node: NodeId,
    from: usize,
    ticks: usize,
) -> Vec<[f32; 2]> {
    let mut last = Stream::new(0);
    for k in from..from + ticks {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead).expect("o tique avança");
        let out = cook.cook(g, reg, node, playhead).expect("coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saída é um stream")
        };
        last = s.clone();
    }
    positions(&last)
}

fn worst(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max)
}

// ---------------------------------------------------------------------------
// 1. OS PARAMS SÃO VIVOS?
// ---------------------------------------------------------------------------

/// Um corpo mole com o pino GENÉRICO (o do `motion.pin_constraint`) na primeira
/// linha — a cena `=50`, banda 4.
fn body(pin_count: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let b = g.add_node("motion.soft_body");
    g.set_param(b, "rows", 6.0);
    g.set_param(b, "cols", 6.0);
    g.set_param(b, "spacing", 0.7);
    g.set_param(b, "pin", 0.0); // o intrínseco DESLIGADO, como na cena
    g.set_param(b, "gravity", 9.8);
    if pin_count > 0.0 {
        let p = g.add_node("motion.pin_constraint");
        g.set_param(p, "first", 0.0);
        g.set_param(p, "count", pin_count);
        g.set_param(p, "strength", 1.0);
        wire(&mut g, b, 0, p, 0, true);
        wire(&mut g, p, 0, b, 2, false);
    } else {
        wire(&mut g, b, 0, b, 2, true);
    }
    (g, b)
}

#[test]
#[ignore = "sonda"]
fn probe_does_spacing_move_a_settled_soft_body() {
    let reg = registry();
    println!("\n== 1. O `spacing` do soft body move o corpo JÁ ASSENTADO? ==");
    println!("   (60 tiques, muda o param, mais 60 tiques)\n");

    for (label, pin_count) in [("SEM pino", 0.0), ("pino genérico na 1ª linha", 6.0)] {
        let (mut g, b) = body(pin_count);
        let mut cook = Cook::new();
        let before = run(&mut cook, &g, &reg, b, 0, 60);

        // O gesto do artista: arrastar o slider.
        g.set_param(b, "spacing", 1.4);
        let after = run(&mut cook, &g, &reg, b, 60, 60);

        // Um CONTROLE: o mesmo corpo, mais 60 tiques, SEM tocar no param.
        let (g2, b2) = body(pin_count);
        let mut cook2 = Cook::new();
        run(&mut cook2, &g2, &reg, b2, 0, 60);
        let drift = run(&mut cook2, &g2, &reg, b2, 60, 60);

        let n_pinned = pin_count as usize;
        let moved_pinned = if n_pinned > 0 {
            worst(&before[..n_pinned], &after[..n_pinned])
        } else {
            f32::NAN
        };
        let moved_free = worst(&before[n_pinned..], &after[n_pinned..]);
        let drift_free = worst(&before[n_pinned..], &drift[n_pinned..]);

        println!("  {label}:");
        println!("    pinados  andaram {moved_pinned:.4}");
        println!(
            "    livres   andaram {moved_free:.4}  (controle, sem tocar no param: {drift_free:.4})"
        );
    }
}

#[test]
#[ignore = "sonda"]
fn probe_which_soft_body_params_are_live() {
    let reg = registry();
    println!("\n== 1b. QUAIS params do soft body são vivos? ==");
    println!("   (assenta 60 tiques, muda UM param, mais 30; o número é o maior deslocamento)\n");

    // ⚠️ **A fixture é o corpo PINADO, não o em queda livre.** Um corpo que cai
    // sem deformar tem os goals EM CIMA das próprias partículas, então
    // `stiffness`/`beta`/`pressure` multiplicam zero e a sonda os reporta inertes
    // — verde por vácuo sobre params que funcionam. O pinado ASSENTA (controle
    // 0,0020), e é ali que uma mudança se lê.
    let sweep: &[(&str, f32)] = &[
        ("spacing", 1.4),
        ("stiffness", 0.15),
        ("gravity", 2.0),
        ("damping", 0.4),
        ("beta", 0.8),
        ("pressure", 1.5),
        ("rows", 8.0),
        ("cols", 8.0),
    ];
    for (name, value) in sweep {
        let (mut g, b) = body(6.0);
        let mut cook = Cook::new();
        let before = run(&mut cook, &g, &reg, b, 0, 60);
        g.set_param(b, *name, *value);
        let after = run(&mut cook, &g, &reg, b, 60, 30);

        let (g2, b2) = body(6.0);
        let mut cook2 = Cook::new();
        run(&mut cook2, &g2, &reg, b2, 0, 60);
        let drift = run(&mut cook2, &g2, &reg, b2, 60, 30);

        // `rows`/`cols` mudam a CONTAGEM: o corpo é re-semeado, e comparar poses
        // de tamanhos diferentes não significa nada.
        if after.len() != before.len() {
            println!(
                "  {name:>10} -> {value:<5}  RE-SEMEIA ({} -> {})",
                before.len(),
                after.len()
            );
            continue;
        }
        let moved = worst(&before, &after);
        let d = worst(&before, &drift);
        println!("  {name:>10} -> {value:<5}  andou {moved:.4}   (controle {d:.4})");
    }
}

/// A pergunta ESTRUTURAL: um pino genérico é alcançável por alguma coisa?
///
/// O caso canônico de um corpo mole pregado é *a bandeira num mastro que se
/// move*. Se a ÂNCORA não alcança o pinado, o pino genérico é um congelamento no
/// espaço do mundo — e a bandeira fica inexprimível com ele.
#[test]
#[ignore = "sonda"]
fn probe_does_the_anchor_reach_a_pinned_particle() {
    let reg = registry();
    println!("\n== 1c. A ÂNCORA alcança o pinado? ==\n");

    // ⚠️ `anchor_x`/`anchor_y` são **PORTAS**, não params — um `set_param` com
    // esses nomes é ignorado em SILÊNCIO, e foi assim que a 1ª versão desta sonda
    // mediu `0.0000` nos dois lados e quase me fez acusar o pino intrínseco.
    for (label, intrinsic, pin_count) in [
        ("pino INTRÍNSECO (o param `pin`)", 1.0, 0.0),
        ("pino GENÉRICO (`motion.pin_constraint`)", 0.0, 6.0),
    ] {
        let (mut g, b) = body(pin_count);
        g.set_param(b, "pin", intrinsic);
        let ax = g.add_node("value.lfo");
        g.set_param(ax, "amplitude", 0.0); // constante: a saída é o `offset`
        g.set_param(ax, "offset", 0.0);
        wire(&mut g, ax, 0, b, 0, false);
        let mut cook = Cook::new();
        let before = run(&mut cook, &g, &reg, b, 0, 60);
        g.set_param(ax, "offset", 3.0); // o artista arrasta o mastro
        let after = run(&mut cook, &g, &reg, b, 60, 60);

        // A linha de TOPO é a pinada nos dois casos (o intrínseco prega `0..cols`).
        let top = 6usize;
        let moved_pinned = worst(&before[..top], &after[..top]);
        let moved_free = worst(&before[top..], &after[top..]);
        println!("  {label}:");
        println!(
            "    a linha pinada andou {moved_pinned:.4}   (o resto do corpo: {moved_free:.4})"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. A SOBREPOSIÇÃO DO BANDO
// ---------------------------------------------------------------------------

/// O bando da cena `=50`, com `collide` opcional na cadeia de estado.
fn flock(count: f32, separation: f32, radius: f32, collide_r: Option<f32>) -> (Graph, NodeId) {
    flock_with_space(count, separation, radius, 0.0, collide_r)
}

fn flock_with_space(
    count: f32,
    separation: f32,
    radius: f32,
    sep_radius: f32,
    collide_r: Option<f32>,
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let f = g.add_node("motion.boids");
    g.set_param(f, "count", count);
    g.set_param(f, "seed", 7.0);
    g.set_param(f, "separation", separation);
    g.set_param(f, "separation_radius", sep_radius);
    g.set_param(f, "radius", radius);
    match collide_r {
        None => wire(&mut g, f, 0, f, 2, true),
        Some(r) => {
            let c = g.add_node("motion.collide");
            g.set_param(c, "radius", r);
            g.set_param(c, "iterations", 4.0);
            wire(&mut g, f, 0, c, 0, true);
            wire(&mut g, c, 0, f, 2, false);
        }
    }
    (g, f)
}

/// Distância ao vizinho MAIS PRÓXIMO de cada agente, ordenada.
fn nearest_neighbour(p: &[[f32; 2]]) -> Vec<f32> {
    let mut out = Vec::with_capacity(p.len());
    for (i, a) in p.iter().enumerate() {
        let mut best = f32::INFINITY;
        for (j, b) in p.iter().enumerate() {
            if i == j {
                continue;
            }
            let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
            best = best.min(d);
        }
        out.push(best);
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap());
    out
}

fn report(label: &str, p: &[[f32; 2]]) {
    let nn = nearest_neighbour(p);
    let n = nn.len();
    let median = nn[n / 2];
    let overlapping = nn.iter().filter(|d| **d < DRAWN_SIZE).count();
    println!(
        "  {label:<34} min {:.4}  p25 {:.4}  mediana {:.4}  max {:.4}  | sobrepostos {}/{} ({:.0}%)",
        nn[0],
        nn[n / 4],
        median,
        nn[n - 1],
        overlapping,
        n,
        100.0 * overlapping as f32 / n as f32,
    );
}

#[test]
#[ignore = "sonda"]
fn probe_how_close_do_boids_pack() {
    let reg = registry();
    println!("\n== 2. Quão perto os agentes ficam? (tamanho DESENHADO = {DRAWN_SIZE}) ==");
    println!(
        "   40 agentes, 180 tiques. `sobrepostos` = vizinho mais próximo < tamanho desenhado.\n"
    );

    println!("  -- varrendo o PESO `separation` (o único knob de hoje) --");
    for sep in [0.0f32, 0.8, 1.6, 2.4, 3.2, 6.4, 12.8, 25.6, 51.2, 102.4] {
        let (g, f) = flock(40.0, sep, 2.0, None);
        let mut cook = Cook::new();
        let p = run(&mut cook, &g, &reg, f, 0, 180);
        report(&format!("separation = {sep:>5}"), &p);
    }

    println!("\n  -- varrendo o `radius` (percepção; separação usa o MESMO) --");
    for r in [0.5f32, 1.0, 2.0, 4.0] {
        let (g, f) = flock(40.0, 1.6, r, None);
        let mut cook = Cook::new();
        let p = run(&mut cook, &g, &reg, f, 0, 180);
        report(&format!("radius = {r:>5}"), &p);
    }

    println!("\n  -- a COMPOSIÇÃO: `boids --pre--> collide --> boids.state` --");
    for cr in [0.1f32, 0.2, 0.3, 0.5] {
        let (g, f) = flock(40.0, 1.6, 2.0, Some(cr));
        let mut cook = Cook::new();
        let p = run(&mut cook, &g, &reg, f, 0, 180);
        report(
            &format!("collide radius = {cr:>5} (diam {:.2})", cr * 2.0),
            &p,
        );
    }
}

/// A LEI NOVA: o espaço pessoal é um ALVO? (o equilíbrio tende a `R`?)
#[test]
#[ignore = "sonda"]
fn probe_does_personal_space_land_on_its_number() {
    let reg = registry();
    println!("\n== 2c. O `separation_radius` é um ALVO? (tamanho desenhado {DRAWN_SIZE}) ==\n");

    for r in [0.0f32, 1.0, 2.0, 4.0, 8.0] {
        println!("  -- separation_radius = {r} (0 = desligado; radius = 2.0) --");
        for sep in [1.6f32, 3.2, 6.0] {
            let (g, f) = flock_with_space(40.0, sep, 2.0, r, None);
            let mut cook = Cook::new();
            let p = run(&mut cook, &g, &reg, f, 0, 180);
            report(&format!("  peso {sep:>4}"), &p);
        }
    }
}

/// A pergunta que decide: **o bando CABE?**
///
/// A extensão de um bando é fixada pelo `seek` (a mola até o alvo) contra o
/// `max_speed`, e NÃO pela contagem. Então há uma contagem acima da qual `N`
/// discos do tamanho desenhado não cabem na área que o bando ocupa — e nenhum
/// valor de `separation` a conserta, porque o problema não é a repulsão, é o
/// espaço.
#[test]
#[ignore = "sonda"]
fn probe_does_the_flock_even_fit() {
    let reg = registry();
    println!("\n== 2b. O bando CABE? (disco desenhado de diâmetro {DRAWN_SIZE}) ==\n");

    for count in [8.0f32, 16.0, 40.0, 80.0, 160.0] {
        let (g, f) = flock(count, 1.6, 2.0, None);
        let mut cook = Cook::new();
        let p = run(&mut cook, &g, &reg, f, 0, 180);
        let n = p.len() as f32;
        let cx = p.iter().map(|q| q[0]).sum::<f32>() / n;
        let cy = p.iter().map(|q| q[1]).sum::<f32>() / n;
        let extent = p
            .iter()
            .map(|q| ((q[0] - cx).powi(2) + (q[1] - cy).powi(2)).sqrt())
            .fold(0.0f32, f32::max);
        // A área que o bando ocupa contra a que N discos PRECISAM (empacotamento
        // hexagonal, o mais denso que existe: 90,69% de aproveitamento).
        let have = std::f32::consts::PI * extent * extent;
        let need = n * std::f32::consts::PI * (DRAWN_SIZE * 0.5).powi(2) / 0.9069;
        println!(
            "  {:>4} agentes  raio do bando {extent:.3}  area disponivel {have:7.2}  \
             precisa {need:7.2}  -> {:.2}x {}",
            count as u32,
            need / have,
            if need > have { "NAO CABE" } else { "cabe" },
        );
    }
}
