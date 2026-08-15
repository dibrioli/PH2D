//! **Boids on the GPU** (ADR-0140 Phase 3b) — the O(N²) flock, answered by the
//! spatial grid, reconciled against the CPU all-pairs.
//!
//! A sim is `x_{n+1} = f(x_n)`, so ε feeds back and a long trajectory drifts
//! (ADR-0127 D4); the gate therefore asserts the SEED (tick 0, where the integer
//! `hash3` makes it **bit-exact**) and ONE step from it (tick 1, where only the
//! float sum ORDER over an identical neighbour set differs → ε). This is also the
//! first exercise of the grid over the `pre` STATE port and the tick-0 empty-grid
//! path — the neighbour gate used a per-element input port, always present.
//!
//! `#[ignore]`: needs an adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_boids --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::RenderInstance;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const FIXED_DT: f64 = 1.0 / 60.0;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_boids::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_force_wind::register(&mut reg).unwrap();
    reg
}

/// `boids ──> output`, with the `out ──pre──> state` self-loop the editor auto-wires.
fn boids_graph(count: f32) -> (Graph, NodeId, NodeId) {
    boids_graph_spread(count, false)
}

/// As [`boids_graph`], with the √N `spread` mode explicit — its one `sqrt` is the
/// only place the two seeds diverge, so `spread` on is an ε seed (not bit-exact).
/// ⚠️ **Devolve os DOIS nós, e a assinatura é uma cicatriz:** ela dava só o
/// `output`, e um gate que quisesse afinar o BANDO escrevia `set_param(out, ..)`
/// — um param no nó errado, **ignorado em silêncio**. Foi assim que a paridade do
/// cone nasceu verde por vácuo (ela cozia nos defaults e comparava dois bandos que
/// nunca usaram o cone), e quem a pegou foi o gate de CONTROLE ao lado dela.
fn boids_graph_spread(count: f32, spread: bool) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let boids = g.add_node("motion.boids");
    g.set_param(boids, "count", count);
    g.set_param(boids, "spread", if spread { 1.0 } else { 0.0 });
    g.set_param(boids, "seed", 1.0);
    // Non-round, non-default weights so a swapped rule can't hide behind a tidy
    // number ([[feedback_test_with_product_numbers_not_convenient_ones]]).
    g.set_param(boids, "radius", 2.3);
    g.set_param(boids, "separation", 1.4);
    g.set_param(boids, "alignment", 0.9);
    g.set_param(boids, "cohesion", 0.7);
    g.set_param(boids, "seek", 1.1);
    g.set_param(boids, "max_speed", 4.0);
    let out = g.add_node("motion.output");
    // The self-loop (delayed) + the render edge.
    g.connect(Edge {
        from: (boids, 0),
        to: (boids, 2),
        delayed: true,
    })
    .unwrap();
    g.connect(Edge {
        from: (boids, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    (g, boids, out)
}

/// `boids ──pre──> force.wind ──> boids.state`, plus the render edge — the flock
/// with a wind in its state chain (doc 89 §2.1, W1).
///
/// ⚠️ **Esta fixture existe porque as outras NÃO CONTÊM O FENÔMENO.** As quatro
/// paridades acima montam `boids ──> output` com o self-loop nu, então a coluna
/// `accel` está AUSENTE nas duas rotas e elas concordam sobre um termo que
/// nenhuma das duas avalia: apagar o `read_state_accel(i)` do WGSL deixa as
/// quatro VERDES, com a CPU levando o vento e a GPU não. É o modo de falha que o
/// repo mais encontra — duas respostas para a mesma pergunta, divergindo no único
/// lugar onde ninguém lê um número.
fn boids_in_a_wind(count: f32, strength: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let boids = g.add_node("motion.boids");
    g.set_param(boids, "count", count);
    g.set_param(boids, "seed", 1.0);
    g.set_param(boids, "radius", 2.3);
    g.set_param(boids, "separation", 1.4);
    g.set_param(boids, "alignment", 0.9);
    g.set_param(boids, "cohesion", 0.7);
    // Sem seek: a mola do alvo mascararia o vento puxando tudo de volta.
    g.set_param(boids, "seek", 0.0);
    g.set_param(boids, "max_speed", 4.0);
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 30.0);
    g.set_param(wind, "strength", strength);
    g.set_param(wind, "gust", 0.0);
    let out = g.add_node("motion.output");
    // `out --pre--> wind` (a aresta DELAYED sai do gerador, como o plumbing a
    // escreve) · `wind --> state` · a aresta de cena.
    for (from, to, port, delayed) in [
        (boids, wind, 0u16, true),
        (wind, boids, 2u16, false),
        (boids, out, 0u16, false),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed,
        })
        .unwrap();
    }
    (g, out)
}

/// Cook `0..=ticks` on the canonical CPU path; return each tick's lowering.
fn cpu_ticks(g: &Graph, reg: &NodeRegistry, out: NodeId, ticks: u64) -> Vec<Vec<RenderInstance>> {
    let mut cook = Cook::new();
    let mut frames = Vec::new();
    for t in 0..=ticks {
        let playhead = t as f64 * FIXED_DT;
        let mut lowered = Vec::new();
        ph2d_eval_motion::evaluate_motion_into(
            &mut cook,
            g,
            reg,
            out,
            playhead,
            DEFAULT_UV,
            DEFAULT_SIZE,
            &mut lowered,
        )
        .expect("cpu cook");
        cook.advance_tick(g, reg, playhead).expect("cpu tick");
        frames.push(lowered);
    }
    frames
}

/// Cook `0..=ticks` on the GPU; return the last tick's lowering. Proves the plan
/// claims the loop and dispatches — a silent CPU fallback would compare CPU to CPU.
fn gpu_ticks(
    gpu: &GpuContext,
    g: &Graph,
    reg: &NodeRegistry,
    out: NodeId,
    ticks: u64,
    stages: usize,
) -> Vec<RenderInstance> {
    let plan = plan(g, reg, reg, out);
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    assert!(plan.drives_a_loop(), "the flock state must live on the GPU");
    // ⚠️ A contagem é EXATA por fixture, não `>= 1`: ela é o que prova que não
    // houve fallback silencioso para a CPU (que compararia CPU com CPU) — e o
    // grafo com vento tem legitimamente DOIS estágios, o flock e a força. Afrouxar
    // para `>= 1` trocaria a prova por uma frase.
    assert_eq!(
        plan.dispatching_stages(reg),
        stages,
        "estagios que despacham (output e pass-through)"
    );
    let mut gc = GpuCook::new();
    for t in 0..=ticks {
        gc.cook(
            gpu,
            g,
            reg,
            reg,
            &plan,
            &[],
            CookClock {
                playhead: t as f64 * FIXED_DT,
                tick: Some(t),
            },
            DEFAULT_UV,
            DEFAULT_SIZE,
            0,
        )
        .expect("gpu cook");
    }
    read_instances(gpu, gc.instances().expect("cooked"))
}

fn parity(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance], eps: f32) {
    assert_eq!(cpu.len(), gpu.len(), "{label}: instance count");
    let mut max_pos = 0.0f32;
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            let d = (c.world_pos[k] - g.world_pos[k]).abs();
            max_pos = max_pos.max(d);
            assert!(
                d <= eps,
                "{label}: instance {i} world_pos[{k}]: cpu {} vs gpu {} (|diff| {d} > {eps})",
                c.world_pos[k],
                g.world_pos[k]
            );
        }
    }
    eprintln!(
        "boids {label}: {} agents, max |Δpos| = {max_pos:e}",
        cpu.len()
    );
}

#[test]
#[ignore = "needs a GPU adapter"]
fn the_boids_seed_matches_the_cpu_bit_for_bit() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, _boids, out) = boids_graph(400.0);
    let cpu = cpu_ticks(&g, &reg, out, 0);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 0, 1);
    // The seed is the integer `hash3` on both sides → bit-exact.
    parity("seed", &cpu[0], &gpu_out, 0.0);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn one_boids_step_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, _boids, out) = boids_graph(400.0);
    // Tick 1 is ONE step from the seed (which already carries a muzzle velocity),
    // so the three urges + seek all fire. The neighbour SET is identical (grid =
    // all-pairs within radius); only the float SUM order differs ⇒ ε.
    let cpu = cpu_ticks(&g, &reg, out, 1);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 1, 1);
    parity("one step", &cpu[1], &gpu_out, 2e-3);
}

/// **A paridade com o CONE LIGADO.**
///
/// ⚠️ **A fixture irmã coze nos DEFAULTS, logo é CEGA ao cone** — com `fov = 360`
/// o ramo angular não roda em nenhuma das duas rotas, e as duas concordariam
/// **por vácuo** sobre um teste que nenhuma executa. É a mesma armadilha que a
/// wave do `value.noise` pagou (a paridade cozinhava `kernel = 0` e os kernels
/// novos concordavam por não serem exercidos).
///
/// ⚠️ **E a tolerância é ε, não bit, por um motivo NOMEADO:** o cosseno do
/// meio-ângulo é o único transcendental do modelo, a CPU corre a `libm` e o
/// device o `cos` do vendedor. Um vizinho EXATAMENTE na borda do cone pode ser
/// contado por uma rota e não pela outra — medida-zero em float, e é por isso que
/// o `>= 360 ⇒ -1` é **literal** dos dois lados: o caso comum, o disco, não
/// depende de nenhum dos dois `cos`.
#[test]
#[ignore = "needs a GPU adapter"]
fn one_boids_step_with_the_view_cone_matches_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (mut g, boids, out) = boids_graph(400.0);
    // Um cone ESTREITO e não-redondo: 360 seria o controle disfarçado de teste, e
    // um número redondo esconderia um erro de meio-ângulo (180 contra 90).
    // ⚠️ **No `boids`, não no `out`** — ver [`boids_graph_spread`].
    g.set_param(boids, "fov", 110.0);
    g.set_param(boids, "speed_floor", 0.35);
    let cpu = cpu_ticks(&g, &reg, out, 1);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 1, 1);
    parity("one step, view cone", &cpu[1], &gpu_out, 2e-3);
}

/// O **CONTROLE** do gate acima: com o cone ligado o bando de facto anda para
/// outro lugar. Sem ele, um `fov` que o kernel ignorasse nos dois lados passaria
/// na paridade e a wave seria invisível.
#[test]
#[ignore = "needs a GPU adapter"]
fn the_view_cone_actually_changes_where_the_flock_goes() {
    let reg = registry();
    let (wide, boids, out) = boids_graph(400.0);
    let mut narrow = wide.clone();
    narrow.set_param(boids, "fov", 110.0);
    let a = cpu_ticks(&wide, &reg, out, 2);
    let b = cpu_ticks(&narrow, &reg, out, 2);
    let moved = a[2]
        .iter()
        .zip(&b[2])
        .flat_map(|(x, y)| (0..2).map(move |k| (x.world_pos[k] - y.world_pos[k]).abs()))
        .fold(0.0f32, f32::max);
    assert!(moved > 1e-4, "o cone move o bando: max delta {moved:e}");
    eprintln!("[cone] o cone de 110 graus move o bando em {moved:e}");
}

#[test]
#[ignore = "needs a GPU adapter"]
fn the_spread_seed_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    // √N spread at a count whose √(count/64) is IRRATIONAL (300/64 = 4.6875 →
    // √ ≈ 2.165), so the CPU/GPU `sqrt` genuinely differs — this exercises the ε,
    // where 400 (=√6.25=2.5 exact) would have hidden it at 0.
    let (g, _boids, out) = boids_graph_spread(300.0, true);
    let cpu = cpu_ticks(&g, &reg, out, 0);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 0, 1);
    // Half-extent ≈ 3·√(300/64) ≈ 6.5 world units → a 1-ULP sqrt is ~1e-6.
    parity("spread seed", &cpu[0], &gpu_out, 1e-4);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn one_spread_step_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, _boids, out) = boids_graph_spread(400.0, true);
    let cpu = cpu_ticks(&g, &reg, out, 1);
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 1, 1);
    parity("spread step", &cpu[1], &gpu_out, 2e-3);
}

/// **O vento na cadeia de estado chega às DUAS rotas.** A CPU soma o `accel` como
/// um quarto termo de steering; o WGSL faz o mesmo com `read_state_accel(i)`.
/// FALSIFICADO por apagar a linha do WGSL ou por a CPU parar de ler a coluna.
#[test]
#[ignore = "needs a GPU adapter"]
fn one_boids_step_in_a_wind_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, out) = boids_in_a_wind(400.0, 18.0);
    let cpu = cpu_ticks(&g, &reg, out, 1);
    // DOIS estágios: o flock e a força que alimenta a cadeia de estado dele.
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 1, 2);
    parity("one step in a wind", &cpu[1], &gpu_out, 2e-3);
}

/// **O CONTROLE da fixture acima: o vento de fato MOVE a rota da CPU.** Sem ele a
/// paridade seria verde por vácuo — duas rotas concordando sobre um termo que
/// nenhuma avalia. Roda sem adapter de propósito: é uma afirmação sobre a
/// FIXTURE, não sobre o dispositivo.
#[test]
fn the_wind_fixture_actually_moves_the_flock() {
    let reg = registry();
    // A MESMA topologia nos dois lados — só a intensidade varia, então o que a
    // diferença mede é o vento e nada mais.
    let (windy, out_w) = boids_in_a_wind(64.0, 18.0);
    let (still, out_s) = boids_in_a_wind(64.0, 0.0);
    let mean_x =
        |v: &[RenderInstance]| v.iter().map(|i| i.world_pos[0]).sum::<f32>() / v.len() as f32;
    let a = cpu_ticks(&windy, &reg, out_w, 8);
    let b = cpu_ticks(&still, &reg, out_s, 8);
    assert!(
        (mean_x(&a[8]) - mean_x(&b[8])).abs() > 0.1,
        "a fixture do vento tem de MOVER o bando, senao a paridade e verde por vacuo: \
         {} contra {}",
        mean_x(&a[8]),
        mean_x(&b[8])
    );
}

/// O bando com um ORÇAMENTO DE STEERING (Reynolds GDC 1999) — a mesma topologia
/// nua das quatro paridades acima, só com `max_force` armado.
///
/// ⚠️ **Fixture própria pela MESMA razão do vento:** com o default `0` o clamp
/// nem entra no ramo, então as paridades existentes concordam sobre um termo que
/// nenhuma das duas avalia — apagar o bloco do WGSL as deixaria todas VERDES,
/// com a CPU truncando a aceleração e o device não.
fn boids_graph_clamped(count: f32, max_force: f32) -> (Graph, NodeId) {
    let (mut g, _boids, out) = boids_graph_spread(count, false);
    let boids = g
        .nodes()
        .iter()
        .position(|n| n.type_name == "motion.boids")
        .map(|i| NodeId(i as u32))
        .expect("a fixture monta um motion.boids");
    g.set_param(boids, "max_force", max_force);
    (g, out)
}

/// **O clamp de steering roda IGUAL nas duas rotas.** A CPU trunca por `norm()`
/// (comprimento, early-out no EPS, unidade × orçamento) e o WGSL repete a mesma
/// ordem de operações. FALSIFICADO por apagar o bloco `max_force` do WGSL.
///
/// ⚠️ **QUATRO tiques, e o número decide** (`probe_what_the_steering_budget_changes`):
/// o sinal que um device sem o clamp produziria é a divergência por instância
/// contra o mesmo bando sem teto, e ela **compõe com o tempo** — `0,0047` em um
/// tique contra `0,0441` em quatro. Com o eps de `2e-3` das irmãs, um tique só
/// daria fosso de **2,4×** e quatro dão **22×**. *Um gate cujo sinal mal passa do
/// próprio eps é um gate que a próxima mudança de ordem de soma silencia.*
#[test]
#[ignore = "needs a GPU adapter"]
fn a_clamped_boids_run_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_boids");
        return;
    };
    let reg = registry();
    let (g, out) = boids_graph_clamped(400.0, 0.6);
    let cpu = cpu_ticks(&g, &reg, out, 4);
    // UM estágio: o bando sozinho, sem nó de força na cadeia.
    let gpu_out = gpu_ticks(&gpu, &g, &reg, out, 4, 1);
    parity("four clamped steps", &cpu[4], &gpu_out, 2e-3);
}

/// **O CONTROLE: o orçamento de fato MORDE na rota da CPU.** Sem esta afirmação a
/// paridade acima seria verde por vácuo — duas rotas concordando sobre um ramo em
/// que nenhuma entra. Roda sem adapter de propósito: é sobre a FIXTURE.
///
/// ⚠️ **O oráculo é a DISPERSÃO, e as duas primeiras tentativas erraram.** A
/// média de `|posição|` num bando quase simétrico não separa nada (medido:
/// `2,5829` contra `2,5835`), e quatro tiques não separam **estatística nenhuma**
/// — um clamp de aceleração precisa de tempo para compor. O que o orçamento faz
/// fisicamente é **impedir o bando de se AGRUPAR** (a coesão e o seek não podem
/// puxar forte), então o discriminante é o raio médio em torno do próprio centro,
/// e ele é monotônico no orçamento (`probe_what_the_steering_budget_changes`,
/// 120 tiques): sem teto **3,0592** · `2,0` → 3,5504 · `0,6` → 4,0613 ·
/// `0,1` → **4,2801**. A barra de `0,5` deixa fosso de 2,4×.
#[test]
fn the_clamped_fixture_actually_bounds_the_steering() {
    let reg = registry();
    // A MESMA topologia nos dois lados — só o orçamento varia.
    let (tight, out_t) = boids_graph_clamped(64.0, 0.1);
    let (open, out_o) = boids_graph_clamped(64.0, 0.0);
    let a = cpu_ticks(&tight, &reg, out_t, 120);
    let b = cpu_ticks(&open, &reg, out_o, 120);
    let (ra, rb) = (mean_radius(&a[120]), mean_radius(&b[120]));
    assert!(
        ra - rb > 0.5,
        "um orcamento apertado tem de deixar o bando MAIS espalhado (ele nao consegue \
         se agrupar), senao a paridade e verde por vacuo: apertado {ra} contra aberto {rb}"
    );
}

/// O raio médio em torno do próprio centro — a DISPERSÃO do bando.
fn mean_radius(v: &[RenderInstance]) -> f32 {
    let n = v.len() as f32;
    let mx = v.iter().map(|i| i.world_pos[0]).sum::<f32>() / n;
    let my = v.iter().map(|i| i.world_pos[1]).sum::<f32>() / n;
    v.iter()
        .map(|i| {
            let d = [i.world_pos[0] - mx, i.world_pos[1] - my];
            (d[0] * d[0] + d[1] * d[1]).sqrt()
        })
        .sum::<f32>()
        / n
}

/// **SONDA — o que o orçamento de steering muda, e em quanto tempo.** É dela que
/// saem as duas barras acima: a dispersão (o oráculo do controle) e a divergência
/// por instância (o fosso que o eps da paridade precisa bater).
///
/// ```text
/// cargo test -p ph2d-gpu-cook --test gpu_boids probe_what_the_steering_budget_changes -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a tabela que calibra as barras acima"]
fn probe_what_the_steering_budget_changes() {
    let reg = registry();
    for ticks in [1u64, 4, 8, 30, 120] {
        let (base, out_b) = boids_graph_clamped(64.0, 0.0);
        let b = cpu_ticks(&base, &reg, out_b, ticks);
        for mf in [0.0f32, 2.0, 0.6, 0.1] {
            let (g, out) = boids_graph_clamped(64.0, mf);
            let v = cpu_ticks(&g, &reg, out, ticks);
            let worst = v[ticks as usize]
                .iter()
                .zip(b[ticks as usize].iter())
                .map(|(a, c)| {
                    (a.world_pos[0] - c.world_pos[0])
                        .abs()
                        .max((a.world_pos[1] - c.world_pos[1]).abs())
                })
                .fold(0.0f32, f32::max);
            eprintln!(
                "ticks {ticks:>3}  max_force {mf:>4}  raio medio {:>7.4}  \
                 pior divergencia/instancia contra sem-teto {worst:>9.6}",
                mean_radius(&v[ticks as usize])
            );
        }
    }
}
