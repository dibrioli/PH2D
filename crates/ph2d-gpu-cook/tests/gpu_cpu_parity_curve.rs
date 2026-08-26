//! GPU-vs-CPU parity para **a FORMA QUE O ARTISTA DESENHA** — a onda `Custom` do
//! `motion.oscillator` e a ease `Custom` do `motion.stagger` (doc 89, folha 06).
//!
//! ## Por que uma curva chega ao device por LUT, e não por fallback
//!
//! Uma curva não é um número: ela vive num **text param**, que o `KernelParams`
//! congelado não sabe carregar. A saída fácil seria o kernel declarar-se
//! `applicable: false` quando a forma é `Custom` e o nó inteiro cair para a CPU —
//! e ela está **REJEITADA**: o precedente do `field.remap` mostra que a tabela
//! resolve o caso sem tirar o nó do device, e um animador é exactamente o nó que
//! não se quer perder do caminho rápido. O sequenciador amostra a curva
//! ([`LutSpec::fill`], no crate do nó) e liga a tabela; o WGSL lê
//! `osc_curve_sample` / `sg_curve_sample`.
//!
//! ## O que cada par mede
//!
//! A forma do irmão `gpu_cpu_parity_time`: cada caso traz o seu **CONTROLE**, e o
//! gate afirma *o device concorda com a CPU* **e** a relação certa com o vizinho.
//! ⚠️ Aqui a relação NÃO é sempre *"difere"*, e essa é a metade que interessa:
//!
//! | caso | contra o controle | por quê |
//! |---|---|---|
//! | `stagger Custom` **sem curva** | **IDÊNTICO** ao `Linear` | *"nada autorado = a família de hoje"* — a lei do default reduzido |
//! | `stagger Custom` **com curva** | difere | senão a LUT estaria a ser ignorada |
//! | `osc Custom` **sem curva** | difere da `Sine` | a identidade da curva é a SERRA `0→1`, que é uma onda de facto |
//! | `osc Custom` **com curva** | difere da serra | idem |
//!
//! Um gate que só medisse *"difere"* passaria com a LUT cheia de lixo; um que só
//! medisse *"idêntico"* passaria com o ramo `Custom` morto. São as duas.
//!
//! `#[ignore]`: precisa de adapter real.
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_curve --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;

/// **DOIS orçamentos, porque são dois fenômenos** — e um só deixaria o caminho
/// aritmético esconder-se atrás do da tabela.
///
/// Um caso que **lê a LUT** paga, além do ULP, um erro de RECONSTRUÇÃO: o device
/// lê `mix(lut[i0], lut[i1], f)` e a CPU avalia a curva exacta, então a esquina da
/// curva cai dentro de uma célula e a tabela corta-a. Um caso que **não** lê a LUT
/// é aritmética pura e tem de ficar no ULP dos gates irmãos.
///
/// ⚠️ **Os dois números são MEDIDOS, não escolhidos** (a primeira versão deste
/// arquivo trazia `1,2e-2` derivado no papel — 13× o pior real, folgado o bastante
/// para uma regressão caber dentro):
///
/// | caso | lê a LUT | `rel` medido |
/// |---|---|---|
/// | `osc Sine` | não | 1,74e-6 |
/// | `stagger Linear` / `Custom` sem curva | não | 4,89e-8 |
/// | `osc Custom` sem curva (a serra) | sim | 3,98e-7 |
/// | `stagger Custom` com curva | sim | 1,25e-4 |
/// | `osc Custom` com curva | sim | 4,09e-4 |
/// | **`osc Custom` + Min/Max** | sim | **9,29e-4** |
///
/// ⚠️ A serra mede 3,98e-7 **e lê a tabela na mesma**: a identidade amostrada é uma
/// RETA, e uma reta reconstrói-se exacta entre dois nós. *O erro é da esquina, não
/// da tabela* — é a mesma lei que a rampa do halo pagou em 22/08.
const EPS_REL_LUT: f32 = 2.0e-3;
const EPS_REL_ARITH: f32 = 1.0e-5;

/// A amplitude mínima para uma fixture CONTER o fenômeno — um campo chato
/// concorda com qualquer kernel.
const MODE_GAP: f32 = 1e-2;

/// **A curva autorada das fixtures** — um V invertido (sobe até ao meio, desce).
///
/// ⚠️ Escolhida por três propriedades, nenhuma decorativa: ela **não é monótona**
/// (uma ease enumerada qualquer é, então confundi-la com uma delas é impossível),
/// os extremos **valem zero** (logo ela difere da identidade no MIOLO, que é onde
/// uma LUT ignorada não seria apanhada por um teste de pontas), e a inclinação é
/// finita em todo lado (uma parada dura mediria a resolução da tabela em vez da
/// aritmética — ver a nota de [`EPS_REL_LUT`]).
const CURVE_V: &str = "c1 0:0:L 0.5:1:L 1:0:L";

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_stagger::register(&mut reg).unwrap();
    reg
}

fn connect(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

fn grid(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 24.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    grid
}

/// Um oscilador no canal Y, com `phase_stagger` a dar variação entre elementos —
/// sem ele todos leriam a MESMA fase e o campo seria constante.
fn osc(g: &mut Graph, src: NodeId) -> NodeId {
    let o = g.add_node("motion.oscillator");
    g.set_param(o, "channel", 1.0); // Y
    g.set_param(o, "amplitude", 1.5);
    g.set_param(o, "frequency", 0.9);
    g.set_param(o, "phase_stagger", 0.031);
    connect(g, src, o);
    o
}

fn osc_sine(g: &mut Graph, src: NodeId) -> NodeId {
    osc(g, src) // wave = 0, o default
}

fn osc_custom_bare(g: &mut Graph, src: NodeId) -> NodeId {
    let o = osc(g, src);
    g.set_param(o, "wave", 5.0);
    o
}

fn osc_custom_authored(g: &mut Graph, src: NodeId) -> NodeId {
    let o = osc_custom_bare(g, src);
    g.set_text_param(o, "curve", CURVE_V);
    o
}

/// A onda `Custom` com a régua de **Min/Max** — o par que mede a faixa NATURAL.
///
/// ⚠️ Ele existe porque a `Custom` é UNIPOLAR (`[0,1]`, o quadrado do editor) e as
/// quatro formas clássicas são bipolares: um WGSL que a tratasse como bipolar
/// entregaria **metade** da faixa com o piso levantado ao centro — exactamente a
/// armadilha que o `Spike` já custou a esta folha, reaberta por uma forma nova.
fn osc_custom_min_max(g: &mut Graph, src: NodeId) -> NodeId {
    let o = osc_custom_authored(g, src);
    g.set_param(o, "range_mode", 1.0);
    g.set_param(o, "min", -2.0);
    g.set_param(o, "max", 3.0);
    o
}

fn stagger(g: &mut Graph, src: NodeId) -> NodeId {
    let s = g.add_node("motion.stagger");
    g.set_param(s, "channel", 1.0); // Y
    g.set_param(s, "min", -2.0);
    g.set_param(s, "max", 2.0);
    connect(g, src, s);
    s
}

fn stagger_linear(g: &mut Graph, src: NodeId) -> NodeId {
    stagger(g, src) // ease_curve = 0, o default
}

fn stagger_custom_bare(g: &mut Graph, src: NodeId) -> NodeId {
    let s = stagger(g, src);
    g.set_param(s, "ease_curve", 8.0);
    s
}

fn stagger_custom_authored(g: &mut Graph, src: NodeId) -> NodeId {
    let s = stagger_custom_bare(g, src);
    g.set_text_param(s, "curve", CURVE_V);
    s
}

/// A `Custom` com a DIREÇÃO girada — tem de ser idêntica à `Custom` sem ela.
fn stagger_custom_out(g: &mut Graph, src: NodeId) -> NodeId {
    let s = stagger_custom_authored(g, src);
    g.set_param(s, "ease_dir", 1.0); // Out
    s
}

/// A relação que cada caso tem com o ANTERIOR.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Versus {
    /// O primeiro de um par — nada a comparar.
    Control,
    /// Tem de produzir um campo DIFERENTE.
    Differs,
    /// Tem de produzir o campo IDÊNTICO (ao bit da paridade).
    Same,
}

struct Case {
    label: &'static str,
    build: fn(&mut Graph, NodeId) -> NodeId,
    versus: Versus,
    /// `true` ⇒ a forma amostrada tem uma **esquina** e paga o erro de reconstrução
    /// da tabela; `false` ⇒ recta ou aritmética pura, e fica no ULP.
    ///
    /// ⚠️ **A pergunta NÃO é «lê a LUT»** — a serra e a `Custom` sem curva leem-na, e
    /// medem `3,98e-7` / `4,89e-8`: a identidade amostrada é uma RECTA, e uma recta
    /// reconstrói-se exacta entre dois nós. Marcá-las como caso de tabela dar-lhes-ia
    /// uma barra 200× folgada sobre um caminho que é aritmético. Ver [`EPS_REL_LUT`].
    samples_a_corner: bool,
}

static CASES: &[Case] = &[
    Case {
        label: "osc Sine (controle)",
        build: osc_sine,
        versus: Versus::Control,
        samples_a_corner: false,
    },
    Case {
        label: "osc Custom sem curva (= serra)",
        build: osc_custom_bare,
        versus: Versus::Differs,
        samples_a_corner: false,
    },
    Case {
        label: "osc Custom com curva",
        build: osc_custom_authored,
        versus: Versus::Differs,
        samples_a_corner: true,
    },
    Case {
        label: "osc Custom + Min/Max",
        build: osc_custom_min_max,
        versus: Versus::Differs,
        samples_a_corner: true,
    },
    Case {
        label: "stagger Linear (controle)",
        build: stagger_linear,
        versus: Versus::Control,
        samples_a_corner: false,
    },
    Case {
        label: "stagger Custom sem curva",
        build: stagger_custom_bare,
        versus: Versus::Same,
        samples_a_corner: false,
    },
    Case {
        label: "stagger Custom com curva",
        build: stagger_custom_authored,
        versus: Versus::Differs,
        samples_a_corner: true,
    },
    Case {
        label: "stagger Custom + dir Out",
        build: stagger_custom_out,
        versus: Versus::Same,
        samples_a_corner: true,
    },
];

fn field_swing(v: &[f32]) -> f32 {
    v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
        - v.iter().fold(f32::INFINITY, |m, x| m.min(*x))
}

fn worst_delta(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "contagem de elementos");
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

/// Coza o grafo nos dois lados e devolve o `Y` que o animador escreveu.
fn cook_both(gpu: &GpuContext, reg: &NodeRegistry, case: &Case, at: f64) -> Vec<f32> {
    let mut g = Graph::new();
    let src = grid(&mut g);
    let sink = (case.build)(&mut g, src);

    g.validate(reg)
        .unwrap_or_else(|e| panic!("{}: {e:?}", case.label));
    let plan = ph2d_gpu_cook::plan(&g, reg, reg, sink);
    assert!(
        plan.is_fully_gpu(),
        "{}: a cadeia tem de ser reivindicada de ponta a ponta — uma curva NÃO derruba o nó para a CPU",
        case.label
    );

    let mut cook = Cook::new();
    let cpu = cook
        .cook(&g, reg, sink, at)
        .unwrap_or_else(|e| panic!("{}: cpu cook {e:?}", case.label));
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        gpu,
        &g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(at),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .unwrap_or_else(|e| panic!("{}: gpu cook {e:?}", case.label));

    let cpu_y: Vec<f32> = match cpu[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("{}: sem coluna P", case.label),
    };
    let gpu_p = gc
        .read_column_vec2(gpu, sink, "P")
        .unwrap_or_else(|| panic!("{}: P não volta do device", case.label));
    let gpu_y: Vec<f32> = gpu_p.iter().map(|p| p[1]).collect();

    let worst = worst_delta(&cpu_y, &gpu_y);
    let swing = field_swing(&cpu_y);
    let eps = if case.samples_a_corner {
        EPS_REL_LUT
    } else {
        EPS_REL_ARITH
    };
    let budget = eps * swing;
    eprintln!(
        "{:<32} max |d| = {worst:e}  amplitude = {swing:.3}  rel = {:e}  (barra {eps:e})",
        case.label,
        worst / swing
    );
    assert!(
        worst < budget,
        "{}: max |d| = {worst:e} sobre amplitude {swing:.3} ({:e} relativo, orçamento {eps:e})",
        case.label,
        worst / swing
    );
    cpu_y
}

/// **Os oito casos da forma desenhada, no device, cada um com o seu controle.**
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_authored_shape_matches_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut previous: Option<Vec<f32>> = None;

    for case in CASES {
        let field = cook_both(&gpu, &reg, case, PLAYHEAD);
        let spread = field_swing(&field);
        assert!(
            spread > MODE_GAP,
            "{}: o campo é chato ({spread:e}) — a fixture não exercita a forma",
            case.label
        );
        match case.versus {
            Versus::Control => {}
            Versus::Differs => {
                let prev = previous.as_ref().expect("um controle precede cada par");
                let gap = worst_delta(&field, prev);
                eprintln!("{:<32}   contra o anterior: {gap:e}", "");
                assert!(
                    gap > MODE_GAP,
                    "{}: tem de diferir do anterior, difere {gap:e}",
                    case.label
                );
            }
            Versus::Same => {
                let prev = previous.as_ref().expect("um controle precede cada par");
                let gap = worst_delta(&field, prev);
                eprintln!("{:<32}   contra o anterior: {gap:e} (tem de ser 0)", "");
                assert_eq!(
                    gap, 0.0,
                    "{}: tem de ser IDÊNTICO ao anterior (o default reduzido)",
                    case.label
                );
            }
        }
        // ⚠️ O `previous` só avança nos casos que ABREM comparação: um `Same`
        // encadeado tem de continuar a medir contra o mesmo controle, senão dois
        // desvios iguais e sucessivos passariam despercebidos.
        if case.versus != Versus::Same {
            previous = Some(field);
        }
    }
}

/// **SONDA — quanto custa a LUT que agora viaja em TODO cozimento** (não é gate).
///
/// ⚠️ A pergunta que o §0 obriga: a tabela é assada e enviada por quadro e por nó,
/// **mesmo quando a forma não é `Custom`** (o `build_luts` não pergunta o modo). Um
/// oscilador é um nó comum, então isto é um custo que todo grafo passa a pagar — e
/// pagar sem medir é o que esta casa não faz.
///
/// Ela mede exactamente o trabalho do `build_luts` para uma tabela de 512: amostrar
/// a curva, criar o buffer, escrevê-lo. Compare com um quadro de **16,7 ms**.
///
///   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_curve --release -- --ignored --nocapture measure_lut
#[test]
#[ignore = "sonda, não um gate — precisa de adapter"]
fn measure_lut_build_cost_per_cook() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    const N: u32 = 512;
    const ROUNDS: u32 = 200;
    let t0 = std::time::Instant::now();
    for _ in 0..ROUNDS {
        let curve = ph2d_curve::parse(CURVE_V).expect("curva valida");
        let mut table = vec![0.0f32; N as usize];
        for (k, slot) in table.iter_mut().enumerate() {
            *slot = curve.eval(k as f32 / (N - 1) as f32);
        }
        let bytes: Vec<u8> = table.iter().flat_map(|v| v.to_le_bytes()).collect();
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sonda lut"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&buffer, 0, &bytes);
    }
    let per = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS);
    eprintln!("LUT de {N} amostras: {per:.4} ms por cozimento por no'");
    eprintln!(
        "um quadro de 60 fps sao 16,7 ms  =>  {:.3}% do quadro",
        per * 100.0 / 16.7
    );
}
