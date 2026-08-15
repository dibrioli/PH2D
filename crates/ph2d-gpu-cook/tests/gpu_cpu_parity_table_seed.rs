//! GPU-vs-CPU parity para **a TABELA e a SEMENTE** — o grupo D da conferência
//! (doc 89 folha 15, a wave W-E).
//!
//! As duas metades da wave carregam ao device algo que **não é um param f32**:
//!
//! - a **TABELA** do `value.pattern` viaja pela LUT (um text param → um buffer
//!   cujo slot 0 é a CONTAGEM), e o kernel lê o buffer DIRETO — nunca o
//!   `_sample`, que interpola;
//! - a **SEMENTE POR NÓ** do `value.instance_field` viaja pelo uniforme
//!   `node_key`, que o gerador declara **só** para um kernel que o pede.
//!
//! Nos dois casos o caminho de device podia divergir do da CPU **em silêncio** e
//! com números plausíveis: uma tabela lida pelo acessor errado sairia
//! interpolada entre passos, e uma identidade que não chegasse ao uniforme
//! deixaria o device a produzir GÊMEOS enquanto a CPU os separa. É isso que
//! estes casos medem, cada um com o seu CONTROLE.
//!
//! `#[ignore]`: precisa de adapter.
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_table_seed --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;

/// Herdado do irmão `gpu_cpu_parity_stats`: o orçamento é RELATIVO à amplitude
/// que a fixture dirige, porque a fonte do erro (a contracção FMA do lowering) é
/// proporcional à magnitude.
const EPS_REL: f32 = 2e-4;
/// Quanto dois casos vizinhos têm de diferir para a comparação significar
/// alguma coisa — muito acima de [`EPS_REL`], de propósito.
const MODE_GAP: f32 = 1e-2;

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
    ph2d_node_motion_drive::register(&mut reg).unwrap();
    ph2d_node_value_pattern::register(&mut reg).unwrap();
    ph2d_node_value_instance_field::register(&mut reg).unwrap();
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

/// Os oito slots legados — o mundo que já shipava, e o controle da tabela.
fn pattern_legacy(g: &mut Graph, src: NodeId) -> NodeId {
    let vp = g.add_node("value.pattern");
    g.set_param(vp, "steps", 3.0);
    g.set_param(vp, "v0", 0.05);
    g.set_param(vp, "v1", 0.55);
    g.set_param(vp, "v2", 0.95);
    connect(g, src, vp);
    vp
}

/// ⚠️ **Onze passos — acima do teto de OITO que a wave removeu.** Se o device
/// caísse no ramo legado, o padrão dele teria três passos e o gate morde.
fn pattern_table_11(g: &mut Graph, src: NodeId) -> NodeId {
    let vp = pattern_legacy(g, src);
    g.set_text_param(
        vp,
        ph2d_node_value_pattern::TABLE_KEY,
        "0.05 0.15 0.25 0.35 0.45 0.55 0.65 0.75 0.85 0.95 1.05",
    );
    vp
}

/// Uma tabela LONGA, para o buffer ser percorrido bem além do cabeçalho.
fn pattern_table_long(g: &mut Graph, src: NodeId) -> NodeId {
    let vp = pattern_legacy(g, src);
    let text: String = (0..257)
        .map(|k| format!("{:.4} ", f64::from(k % 37) * 0.031))
        .collect();
    g.set_text_param(vp, ph2d_node_value_pattern::TABLE_KEY, &text);
    vp
}

/// ⚠️ **Uma tabela MALFORMADA tem de cair no legado nos DOIS lados** — se só um
/// deles a recusasse, o device e a CPU desenhariam padrões diferentes com a
/// mesma string na tela.
fn pattern_table_bad(g: &mut Graph, src: NodeId) -> NodeId {
    let vp = pattern_legacy(g, src);
    g.set_text_param(vp, ph2d_node_value_pattern::TABLE_KEY, "0.1 oops 0.9");
    vp
}

fn field_random(g: &mut Graph, src: NodeId) -> NodeId {
    let f = g.add_node("value.instance_field");
    g.set_param(f, "mode", 2.0); // Random
    g.set_param(f, "seed", 7.0);
    connect(g, src, f);
    f
}

/// A semente decorrelacionada pela IDENTIDADE do nó — o uniforme `node_key`.
fn field_random_unique(g: &mut Graph, src: NodeId) -> NodeId {
    let f = field_random(g, src);
    g.set_param(f, "unique_per_node", 1.0);
    f
}

static CASES: &[Case] = &[
    Case {
        label: "pattern oito slots (controle)",
        build: pattern_legacy,
        differs_from_previous: false,
    },
    Case {
        label: "pattern tabela de 11 (acima do teto)",
        build: pattern_table_11,
        differs_from_previous: true,
    },
    Case {
        label: "pattern tabela de 257",
        build: pattern_table_long,
        differs_from_previous: true,
    },
    Case {
        label: "pattern tabela malformada (volta ao legado)",
        build: pattern_table_bad,
        differs_from_previous: true,
    },
    Case {
        label: "instance_field Random (controle)",
        build: field_random,
        differs_from_previous: true,
    },
    Case {
        label: "instance_field Random + Unique Per Node",
        build: field_random_unique,
        differs_from_previous: true,
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
        "{:<44} max |d| = {worst:e}  amplitude = {swing:.3}  rel = {:e}",
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

/// **Os seis casos do grupo D, no device, cada um com o seu controle.**
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_table_and_the_seed_agree_with_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let reg = registry();
    let mut prev: Option<(&str, Vec<f32>)> = None;
    for case in CASES {
        let y = drive_and_compare(&gpu, &reg, case, PLAYHEAD);
        if case.differs_from_previous {
            let (plabel, pv) = prev.as_ref().expect("um caso anterior");
            let gap = worst_delta(pv, &y);
            eprintln!("    ^ difere de `{plabel}` por {gap:.4}");
            assert!(
                gap > MODE_GAP,
                "{} tem de desenhar um campo DIFERENTE de `{plabel}` (gap {gap:e})",
                case.label
            );
        }
        prev = Some((case.label, y));
    }
}

/// **A tabela malformada cai no legado nos DOIS lados, AO BIT.**
///
/// ⚠️ O caso acima só prova que ela difere do vizinho; este prova a coisa mais
/// forte, e é a que importa: a string ruim reproduz **exactamente** o campo dos
/// oito slots. Uma recusa que valesse só na CPU deixaria o device a ler um
/// cabeçalho de lixo.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_malformed_table_is_the_legacy_field_on_both_sides() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let reg = registry();
    let legacy = drive_and_compare(&gpu, &reg, &CASES[0], PLAYHEAD);
    let bad = drive_and_compare(&gpu, &reg, &CASES[3], PLAYHEAD);
    assert_eq!(legacy, bad, "malformada == legado, ao bit");
}

/// **A identidade chega ao device: dois nós deixam de ser gêmeos LÁ também.**
///
/// ⚠️ O caso da lista compara o campo do nó com o de um vizinho de outro grafo;
/// este monta os **DOIS** `value.instance_field` no MESMO grafo, que é o único
/// arranjo em que o defeito existe — e sem o uniforme `node_key` o device
/// devolveria campos idênticos enquanto a CPU os separa, uma divergência que o
/// gate de paridade sozinho não veria (ele compara CPU e GPU do MESMO nó).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_device_stops_making_twins_too() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — pulado");
        return;
    };
    let reg = registry();
    for (unique, want_twins) in [(0.0f32, true), (1.0, false)] {
        let mut g = Graph::new();
        let src = grid(&mut g);
        let a = field_random(&mut g, src);
        let b = field_random(&mut g, src);
        g.set_param(a, "unique_per_node", unique);
        g.set_param(b, "unique_per_node", unique);
        // Cada campo dirige a sua própria cadeia, para as duas voltarem do device.
        let da = chain(&mut g, a, src);
        let db = chain(&mut g, b, src);
        g.validate(&reg).unwrap();

        let read = |node: NodeId| -> Vec<f32> {
            let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, node);
            assert!(plan.is_fully_gpu());
            let mut gc = ph2d_gpu_cook::GpuCook::new();
            gc.retain_streams_for_debug(true);
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plan,
                &[],
                CookClock::at(PLAYHEAD),
                DEFAULT_UV,
                DEFAULT_SIZE,
                0,
            )
            .unwrap();
            gc.read_column_vec2(&gpu, node, "P")
                .unwrap()
                .iter()
                .map(|p| p[1])
                .collect()
        };
        let (ya, yb) = (read(da), read(db));
        let gap = worst_delta(&ya, &yb);
        eprintln!("unique = {unique}: gap entre os dois nos = {gap:.4}");
        if want_twins {
            assert!(
                gap < 1e-6,
                "sem o toggle os dois nos sao GEMEOS (o defeito)"
            );
        } else {
            assert!(gap > MODE_GAP, "com o toggle deixam de ser (gap {gap:e})");
        }
    }
}
