//! GPU-vs-CPU parity para **as ESTATÍSTICAS** — o grupo C da conferência (doc 89
//! folha 15): os modos novos do `value.reduce` e os pesos do `value.smooth`.
//!
//! ## Este arquivo tem DUAS metades, e só uma precisa de adapter
//!
//! A metade de **PLANO** é a mais importante desta wave e roda em toda parte: o
//! `value.reduce` passou a RECUSAR o device em três modos (`Variance`/`StdDev`
//! precisam de um segundo passe que leia o resultado do primeiro; `Median` é um
//! rank, que não tem combinador) e em qualquer uma das duas portas novas
//! (`mask`/`group` — o canal de redução dobra UMA coluna de UMA porta por UMA
//! expressão). Uma recusa que não fosse honrada não daria erro: o `switch` do
//! kernel cairia no `default` e o campo inteiro receberia a **SOMA**, um número
//! plausível — e é exactamente por isso que a recusa é gateada, e não descrita.
//!
//! A metade de **PARIDADE** é a de sempre: cada param novo com o seu CONTROLE, e
//! o gate afirma *o device concorda com a CPU* **e** *os dois campos DIFEREM*.
//! Sem a segunda metade um WGSL cego ao param passaria em todas as linhas — o
//! default de todos eles É o mundo anterior.
//!
//! `#[ignore]` só na metade que precisa de adapter:
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_stats --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;

/// O orçamento é RELATIVO à amplitude que a fixture de facto dirige — a mesma
/// forma que o irmão `gpu_cpu_parity_time` mediu e adotou: o erro de uma pilha
/// de somas é proporcional à magnitude, e um ε absoluto é a forma errada para
/// ele. `2e-4` = o valor que aquele gate calibrou, herdado aqui porque a fonte
/// do erro é a mesma (a árvore de redução do device contra a soma sequencial da
/// CPU, mais a contracção FMA dos pesos).
const EPS_REL: f32 = 2e-4;
/// Quanto dois casos vizinhos têm de diferir para a comparação significar alguma
/// coisa. Muito acima de [`EPS_REL`] de propósito.
const MODE_GAP: f32 = 1e-2;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// O registry MÍNIMO — só os nós que estas fixtures montam.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_drive::register(&mut reg).unwrap();
    ph2d_node_value_noise::register(&mut reg).unwrap();
    ph2d_node_value_reduce::register(&mut reg).unwrap();
    ph2d_node_value_smooth::register(&mut reg).unwrap();
    reg
}

fn connect_to(g: &mut Graph, a: NodeId, b: NodeId, port: u16) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, port),
        delayed: false,
    })
    .unwrap();
}

fn connect(g: &mut Graph, a: NodeId, b: NodeId) {
    connect_to(g, a, b, 0);
}

fn grid(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 24.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    grid
}

/// Um campo de valor com ESTRUTURA — degraus e vales que um filtro morde e cujos
/// extremos uma estatística separa. Um campo chato deixaria todo modo e todo peso
/// concordar com todo outro.
fn noise(g: &mut Graph, src: NodeId) -> NodeId {
    let vn = g.add_node("value.noise");
    g.set_param(vn, "frequency", 0.31);
    g.set_param(vn, "speed", 0.0);
    g.set_param(vn, "octaves", 3.0);
    g.set_param(vn, "roughness", 0.6);
    g.set_param(vn, "amplitude", 2.0);
    g.set_param(vn, "seed", 7.0);
    connect(g, src, vn);
    vn
}

/// A cadeia inteira, com o `value` a dirigir Y — a comparação atravessa o MESMO
/// lowering que o produto usa, e não só a coluna solta.
fn chain(g: &mut Graph, value: NodeId, src: NodeId) -> NodeId {
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(g, src, drive);
    connect_to(g, value, drive, 1);
    drive
}

type Build = fn(&mut Graph, NodeId) -> NodeId;

struct Case {
    label: &'static str,
    build: Build,
    /// `true` ⇒ tem de produzir um campo DIFERENTE do anterior (o seu controle).
    differs_from_previous: bool,
}

fn reduce_with(g: &mut Graph, src: NodeId, mode: f32) -> NodeId {
    let vn = noise(g, src);
    let vr = g.add_node("value.reduce");
    g.set_param(vr, "mode", mode);
    connect(g, vn, vr);
    vr
}

fn reduce_sum(g: &mut Graph, src: NodeId) -> NodeId {
    reduce_with(g, src, 0.0)
}
fn reduce_mean(g: &mut Graph, src: NodeId) -> NodeId {
    reduce_with(g, src, 1.0)
}
fn reduce_min(g: &mut Graph, src: NodeId) -> NodeId {
    reduce_with(g, src, 2.0)
}
fn reduce_max(g: &mut Graph, src: NodeId) -> NodeId {
    reduce_with(g, src, 3.0)
}
fn reduce_range(g: &mut Graph, src: NodeId) -> NodeId {
    reduce_with(g, src, 4.0)
}

fn smooth_with(g: &mut Graph, src: NodeId, weight: f32) -> NodeId {
    let vn = noise(g, src);
    let vs = g.add_node("value.smooth");
    g.set_param(vs, "radius", 5.0);
    g.set_param(vs, "weight", weight);
    connect(g, vn, vs);
    vs
}

fn smooth_box(g: &mut Graph, src: NodeId) -> NodeId {
    smooth_with(g, src, 0.0)
}
fn smooth_triangle(g: &mut Graph, src: NodeId) -> NodeId {
    smooth_with(g, src, 1.0)
}
fn smooth_smooth(g: &mut Graph, src: NodeId) -> NodeId {
    smooth_with(g, src, 2.0)
}
/// ⚠️ Um raio GRANDE, acima do curso do slider — é ele que o `ParamHardMax` novo
/// destrancou, e o gate prova que o device o percorre inteiro (o laço do WGSL é
/// `2r+1` iterações, e um `r` que o kernel truncasse divergiria da CPU aqui).
fn smooth_wide(g: &mut Graph, src: NodeId) -> NodeId {
    let vs = smooth_with(g, src, 2.0);
    g.set_param(vs, "radius", 40.0);
    vs
}

static CASES: &[Case] = &[
    Case {
        label: "reduce Sum (controle)",
        build: reduce_sum,
        differs_from_previous: false,
    },
    Case {
        label: "reduce Mean",
        build: reduce_mean,
        differs_from_previous: true,
    },
    Case {
        label: "reduce Min",
        build: reduce_min,
        differs_from_previous: true,
    },
    Case {
        label: "reduce Max",
        build: reduce_max,
        differs_from_previous: true,
    },
    Case {
        label: "reduce Range",
        build: reduce_range,
        differs_from_previous: true,
    },
    Case {
        label: "smooth Box (controle)",
        build: smooth_box,
        differs_from_previous: false,
    },
    Case {
        label: "smooth Triangle",
        build: smooth_triangle,
        differs_from_previous: true,
    },
    Case {
        label: "smooth Smooth",
        build: smooth_smooth,
        differs_from_previous: true,
    },
    Case {
        label: "smooth Smooth r=40 (acima do slider)",
        build: smooth_wide,
        differs_from_previous: true,
    },
];

/// A amplitude que a fixture de facto dirige — o denominador do orçamento, e a
/// prova de que ela CONTÉM o fenômeno (um campo chato concorda com tudo).
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

/// Coza o grafo nos dois lados e devolve o `Y` que o valor dirigiu.
fn drive_and_compare(gpu: &GpuContext, reg: &NodeRegistry, case: &Case, at: f64) -> Vec<f32> {
    let mut g = Graph::new();
    let src = grid(&mut g);
    let value = (case.build)(&mut g, src);
    let drive = chain(&mut g, value, src);

    g.validate(reg)
        .unwrap_or_else(|e| panic!("{}: {e:?}", case.label));
    let plan = ph2d_gpu_cook::plan(&g, reg, reg, drive);
    assert!(
        plan.is_fully_gpu(),
        "{}: a cadeia tem de ser reivindicada de ponta a ponta",
        case.label
    );

    let mut cook = Cook::new();
    let cpu = cook
        .cook(&g, reg, drive, at)
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
        0,
    )
    .unwrap_or_else(|e| panic!("{}: gpu cook {e:?}", case.label));

    let cpu_stream = cpu[0].as_stream();
    let cpu_y: Vec<f32> = match cpu_stream.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("{}: sem coluna P", case.label),
    };
    let gpu_p = gc
        .read_column_vec2(gpu, drive, "P")
        .unwrap_or_else(|| panic!("{}: P não volta do device", case.label));
    let gpu_y: Vec<f32> = gpu_p.iter().map(|p| p[1]).collect();

    let worst = worst_delta(&cpu_y, &gpu_y);
    let swing = field_swing(&cpu_y);
    let budget = EPS_REL * swing;
    eprintln!(
        "{:<40} max |d| = {worst:e}  amplitude = {swing:.3}  rel = {:e}",
        case.label,
        worst / swing
    );
    assert!(
        worst < budget,
        "{}: max |d| = {worst:e} sobre amplitude {swing:.3} ({:e} relativo, orçamento {EPS_REL:e})",
        case.label,
        worst / swing
    );
    cpu_y
}

/// **Os nove casos das estatísticas, no device, cada um com o seu controle.**
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_statistics_params_match_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut previous: Option<Vec<f32>> = None;

    for case in CASES {
        let field = drive_and_compare(&gpu, &reg, case, PLAYHEAD);
        let spread = field_swing(&field);
        assert!(
            spread > MODE_GAP,
            "{}: o campo é chato ({spread:e}) — a fixture não exercita o param",
            case.label
        );
        if case.differs_from_previous {
            let prev = previous.as_ref().expect("um controle precede cada par");
            let gap = worst_delta(&field, prev);
            eprintln!("{:<40}   contra o anterior: {gap:e}", "");
            assert!(
                gap > MODE_GAP,
                "{}: NÃO se distingue do anterior ({gap:e}) — ou o param é \
                 ignorado no device, ou a fixture não contém a diferença",
                case.label
            );
        }
        previous = Some(field);
    }
}

/// A cadeia inteira, plantada e planeada — sem device. Devolve *o plano
/// reivindicou tudo?*
fn is_claimed(reg: &NodeRegistry, build: &dyn Fn(&mut Graph, NodeId) -> NodeId) -> bool {
    let mut g = Graph::new();
    let src = grid(&mut g);
    let value = build(&mut g, src);
    let drive = chain(&mut g, value, src);
    g.validate(reg).expect("o grafo tem de ser válido");
    ph2d_gpu_cook::plan(&g, reg, reg, drive).is_fully_gpu()
}

/// **O device reivindica os cinco agregados que sabe dobrar e RECUA nos três que
/// não** — e os dois lados importam.
///
/// ⚠️ Sem a metade POSITIVA a recusa poderia ser total (um `applicable` cravado
/// em `false` deixaria a família inteira na CPU e passaria numa metade só);
/// sem a NEGATIVA o sequenciador reivindicaria um nó cujo kernel não tem braço
/// para `Variance`/`StdDev`/`Median` — o `switch` cairia no `default` e todo
/// elemento receberia a SOMA.
#[test]
fn the_plan_claims_the_foldable_modes_and_recedes_from_the_other_three() {
    let reg = registry();
    for (mode, label) in [
        (0.0, "Sum"),
        (1.0, "Mean"),
        (2.0, "Min"),
        (3.0, "Max"),
        (4.0, "Range"),
    ] {
        assert!(
            is_claimed(&reg, &move |g: &mut Graph, s: NodeId| reduce_with(
                g, s, mode
            )),
            "{label}: é dobrável e tinha de ser reivindicado"
        );
    }
    for (mode, label) in [(5.0, "Variance"), (6.0, "StdDev"), (7.0, "Median")] {
        assert!(
            !is_claimed(&reg, &move |g: &mut Graph, s: NodeId| reduce_with(
                g, s, mode
            )),
            "{label}: o device NÃO pode reivindicar um modo sem braço no kernel"
        );
    }
}

/// **Ligar uma das portas novas entrega o nó à CPU; não ligar não muda nada** —
/// a outra metade da recusa, e a que o `RefuseIfPresent` responde.
///
/// ⚠️ A metade do CONTROLE (`mask` desligada ⇒ reivindicado) é o que impede a
/// leitura preguiçosa *"declare a recusa e pronto"*: uma binding declarada na
/// porta errada, ou com o `access` errado, tiraria do device **toda** a família,
/// inclusive quem nunca liga uma máscara.
#[test]
fn wiring_either_optional_port_hands_the_node_to_the_cpu() {
    let reg = registry();
    assert!(
        is_claimed(&reg, &|g: &mut Graph, s: NodeId| reduce_with(g, s, 1.0)),
        "sem as portas o Mean tem de ficar no device (o controle)"
    );
    for (port, label) in [(1u16, "mask"), (2, "group")] {
        let claimed = {
            let mut g = Graph::new();
            let src = grid(&mut g);
            let vr = reduce_with(&mut g, src, 1.0);
            // Uma segunda fonte de VALOR na porta opcional.
            let side = noise(&mut g, src);
            connect_to(&mut g, side, vr, port);
            let drive = chain(&mut g, vr, src);
            g.validate(&reg).expect("válido");
            ph2d_gpu_cook::plan(&g, &reg, &reg, drive).is_fully_gpu()
        };
        assert!(
            !claimed,
            "a porta `{label}` ligada tem de recuar para a CPU — o canal de \
             redução não exprime nem `Σ(v·mask)` nem um fold por bin"
        );
    }
}
