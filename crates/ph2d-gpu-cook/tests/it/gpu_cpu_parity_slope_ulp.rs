//! **SONDA da barra do `value_slope_kernel_matches_the_cpu_on_the_device`** (doc 110 §14.1 (6)).
//!
//! A barra de `1e-4` absoluto no `P` daquele gate nasceu com o kernel (85def0fa2) e nunca foi
//! medida. Esta sonda mede o VALE que a decide: de um lado o pior desvio LEGÍTIMO, por estágio
//! (`noise.v` → `slope.v` → `drive.P`) e em ULPs; do outro o defeito mais pequeno que a barra tem
//! de apanhar — mutantes REAIS do kernel, derivados do WGSL VIVO por `replace` com contagem
//! (nunca uma cópia que envelhece), registados por cima do original numa `NodeRegistry` própria
//! e cozinhados num `GpuCook` novo (a chave do pipeline leva o ponteiro do texto).
//!
//! ⚠️ A soma usa `NaN → ∞`: o `compare_column` do gate dobra com `f32::max`, que IGNORA um `NaN`
//! — a sonda imprime as duas leituras para que a diferença fique à vista.
//!
//! `#[ignore]`: precisa de adapter.
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-gpu-cook --profile smoke --test it \
//!   probe_value_slope_parity_in_ulps -- --ignored --nocapture
//! ```

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
/// O playhead do gate (`gpu_cpu_parity::PLAYHEAD`) — a primeira amostra da varredura.
const PLAYHEAD: f64 = 0.37;
/// Playheads extra: o ruído legítimo é uma DISTRIBUIÇÃO, e um só tempo é uma amostra dela.
const SWEEP: usize = 24;

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
    ph2d_node_value_noise::register(&mut reg).unwrap();
    ph2d_node_value_slope::register(&mut reg).unwrap();
    ph2d_node_motion_drive::register(&mut reg).unwrap();
    reg
}

/// A cadeia do gate, byte a byte (`gpu_cpu_parity.rs`, `value_slope_kernel_matches…`). Com
/// `world`, o MESMO ruído amostrado em `P` (`space = 1`, a fixture do gate do `value.noise`) em
/// vez do índice — a alternativa de fixture, medida ao lado.
fn chain(g: &mut Graph, world: bool) -> [NodeId; 4] {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 24.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let vn = g.add_node("value.noise");
    g.set_param(vn, "frequency", 0.23);
    g.set_param(vn, "speed", 0.7);
    g.set_param(vn, "octaves", 3.0);
    g.set_param(vn, "roughness", 0.55);
    g.set_param(vn, "amplitude", 1.9);
    g.set_param(vn, "seed", 4.0);
    if world {
        g.set_param(vn, "space", 1.0);
    }
    let slope = g.add_node("value.slope");
    g.set_param(slope, "scale", 2.3);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0);
    g.set_param(drive, "mode", 0.0);
    g.set_param(drive, "scale", 2.0);
    for (a, b) in [((grid, 0), (vn, 0)), ((vn, 0), (slope, 0))] {
        g.connect(Edge {
            from: a,
            to: b,
            delayed: false,
        })
        .unwrap();
    }
    for (a, b) in [((grid, 0), (drive, 0)), ((slope, 0), (drive, 1))] {
        g.connect(Edge {
            from: a,
            to: b,
            delayed: false,
        })
        .unwrap();
    }
    [grid, vn, slope, drive]
}

/// O espaçamento f32 em `|x|` (o degrau para cima). Em `0` é o menor subnormal — um ULP
/// por-elemento perto de zero explode, por isso a sonda imprime também o ULP da GRANDEZA.
fn ulp(x: f32) -> f32 {
    let a = x.abs();
    f32::from_bits(a.to_bits() + 1) - a
}

#[derive(Clone, Copy, Default)]
struct Stage {
    n: usize,
    max_cpu: f32,
    /// `max |d|` com `NaN → ∞`.
    max_d: f32,
    /// A mesma dobra do `compare_column` (`f32::max`, cega ao `NaN`).
    max_d_gate: f32,
    nan: usize,
    /// `max |d| / ulp(cpu[i])`.
    max_ulp_elem: f32,
    /// `max |d| / ulp(max |cpu|)` — o ULP da grandeza da coluna.
    ulp_col: f32,
    worst: usize,
    worst_cpu: f32,
    worst_gpu: f32,
}

fn measure(cpu: &[f32], gpu: &[f32]) -> Stage {
    assert_eq!(cpu.len(), gpu.len(), "contagem de elementos");
    let mut s = Stage {
        n: cpu.len(),
        ..Default::default()
    };
    for (i, (&c, &g)) in cpu.iter().zip(gpu).enumerate() {
        s.max_cpu = s.max_cpu.max(c.abs());
        let raw = (g - c).abs();
        s.max_d_gate = s.max_d_gate.max(raw);
        let d = if raw.is_nan() {
            s.nan += 1;
            f32::INFINITY
        } else {
            raw
        };
        if d > s.max_d || (i == 0 && d >= s.max_d) {
            s.max_d = d;
            s.worst = i;
            s.worst_cpu = c;
            s.worst_gpu = g;
        }
        s.max_ulp_elem = s.max_ulp_elem.max(d / ulp(c));
    }
    s.ulp_col = s.max_d / ulp(s.max_cpu);
    s
}

fn scalar(c: Option<&Column>, what: &str) -> Vec<f32> {
    match c {
        Some(Column::Scalar(v)) => v.clone(),
        other => panic!("{what}: não é escalar: {other:?}"),
    }
}

fn vec2(c: Option<&Column>, what: &str) -> Vec<[f32; 2]> {
    match c {
        Some(Column::Vec2(v)) => v.clone(),
        other => panic!("{what}: não é vec2: {other:?}"),
    }
}

/// As três medições de um cozimento: `noise.v`, `slope.v`, `drive.P.y` — e `P.x` (que nada
/// conduz, o controlo: tem de dar `0`).
struct Run {
    noise: Stage,
    slope: Stage,
    py: Stage,
    px: Stage,
    /// `max |dnoise|` nos 64 primeiros e nos 64 últimos índices — a coordenada do reticulado
    /// cresce com `i`, e o espaçamento f32 com ela.
    noise_band: (f32, f32),
    p_range: ([f32; 2], [f32; 2]),
}

fn run(gpu: &GpuContext, reg: &NodeRegistry, t: f64, world: bool) -> Run {
    let mut g = Graph::new();
    let [_grid, vn, slope, drive] = chain(&mut g, world);
    g.validate(reg).expect("bem tipada");
    let plan = ph2d_gpu_cook::plan(&g, reg, reg, drive);
    assert!(plan.is_fully_gpu(), "a cadeia tem de ser do device");

    let mut cook = Cook::new();
    let cpu_noise = scalar(
        cook.cook(&g, reg, vn, t).expect("cpu noise")[0]
            .as_stream()
            .get("v"),
        "noise.v",
    );
    let cpu_slope = scalar(
        cook.cook(&g, reg, slope, t).expect("cpu slope")[0]
            .as_stream()
            .get("v"),
        "slope.v",
    );
    let cpu_p = vec2(
        cook.cook(&g, reg, drive, t).expect("cpu drive")[0]
            .as_stream()
            .get("P"),
        "drive.P",
    );

    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        gpu,
        &g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(t),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .expect("gpu cook");
    let gpu_noise = gc.read_column(gpu, vn, "v").expect("noise.v volta");
    let gpu_slope = gc.read_column(gpu, slope, "v").expect("slope.v volta");
    let gpu_p = gc.read_column_vec2(gpu, drive, "P").expect("P volta");

    let comp = |v: &[[f32; 2]], k: usize| v.iter().map(|p| p[k]).collect::<Vec<f32>>();
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [0.0f32; 2];
    for p in &cpu_p {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k].abs());
            hi[k] = hi[k].max(p[k].abs());
        }
    }
    let band = |r: std::ops::Range<usize>| {
        r.map(|i| (gpu_noise[i] - cpu_noise[i]).abs())
            .fold(0.0f32, f32::max)
    };
    let n = cpu_noise.len();
    Run {
        noise_band: (band(0..64), band(n - 64..n)),
        noise: measure(&cpu_noise, &gpu_noise),
        slope: measure(&cpu_slope, &gpu_slope),
        py: measure(&comp(&cpu_p, 1), &comp(&gpu_p, 1)),
        px: measure(&comp(&cpu_p, 0), &comp(&gpu_p, 0)),
        p_range: (lo, hi),
    }
}

fn line(label: &str, s: &Stage) {
    eprintln!(
        "  {label:<8} n={:>4} max|cpu|={:<12e} max|d|={:<12e} (gate-fold {:<12e}, NaN={}) \
         ulp(max|cpu|)={:<12e} → {:>10.1} ULP-col · {:>12.1} ULP-elem · worst i={} cpu={:e} gpu={:e}",
        s.n,
        s.max_cpu,
        s.max_d,
        s.max_d_gate,
        s.nan,
        ulp(s.max_cpu),
        s.ulp_col,
        s.max_ulp_elem,
        s.worst,
        s.worst_cpu,
        s.worst_gpu,
    );
}

/// O WGSL vivo do `value.slope` com UMA troca (contagem exata `1`, senão a sonda mede o original
/// sob outro nome — o mutante-fantasma).
fn mutant(reg: &NodeRegistry, edits: &[(&str, &str)]) -> NodeRegistry {
    let id = NodeTypeId::of("value.slope");
    let live = *reg.gpu_kernel(id).expect("value.slope tem kernel");
    let mut wgsl = live.wgsl.to_string();
    for (from, to) in edits {
        assert_eq!(
            wgsl.matches(from).count(),
            1,
            "a âncora `{from}` tem de casar UMA vez no WGSL vivo"
        );
        wgsl = wgsl.replacen(from, to, 1);
    }
    let mut out = registry();
    out.register_gpu_kernel(
        id,
        ph2d_nodegraph::gpu::GpuKernel {
            wgsl: Box::leak(wgsl.into_boxed_str()),
            ..live
        },
    );
    out
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn probe_value_slope_parity_in_ulps() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let times: Vec<f64> = std::iter::once(PLAYHEAD)
        .chain((0..SWEEP).map(|k| k as f64 * 0.173 + 0.01))
        .collect();
    let mutants: &[(&str, &[(&str, &str)])] = &[
        (
            "M1 diferença para a frente em todo o lado (span 1)",
            &[
                (
                    "let vsl_lo = clamp(i32(i) - 1, 0, vsl_last);",
                    "let vsl_lo = clamp(i32(i), 0, vsl_last - 1);",
                ),
                (
                    "let vsl_hi = clamp(i32(i) + 1, 0, vsl_last);",
                    "let vsl_hi = vsl_lo + 1;",
                ),
            ],
        ),
        (
            "M1b centrada sem dividir pelo span (span ≡ 1)",
            &[(
                "let vsl_span = f32(vsl_hi - vsl_lo);",
                "let vsl_span = 1.0;",
            )],
        ),
        ("M2 `scale` ignorado", &[("* params.scale", "* 1.0")]),
        (
            "M3 bordas a meio (span ≡ 2: o `/2` cru)",
            &[(
                "let vsl_span = f32(vsl_hi - vsl_lo);",
                "let vsl_span = 2.0;",
            )],
        ),
        (
            "M4 hi da borda fora por um (clamp a last−1)",
            &[(
                "let vsl_hi = clamp(i32(i) + 1, 0, vsl_last);",
                "let vsl_hi = clamp(i32(i) + 1, 0, vsl_last - 1);",
            )],
        ),
        (
            "M5 hi da 1.ª borda fora por um (hi = 2 em i = 0)",
            &[(
                "let vsl_hi = clamp(i32(i) + 1, 0, vsl_last);",
                "let vsl_hi = select(clamp(i32(i) + 1, 0, vsl_last), 2, i32(i) == 0);",
            )],
        ),
    ];
    for (fixture, world) in [
        ("ÍNDICE (a fixture do gate)", false),
        ("WORLD (space = 1)", true),
    ] {
        eprintln!("════ FIXTURE {fixture} ════");
        probe_fixture(&gpu, &reg, &times, world, mutants);
    }
}

fn probe_fixture(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    times: &[f64],
    world: bool,
    mutants: &[(&str, &[(&str, &str)])],
) {
    // ── O lado LEGÍTIMO: o kernel vivo, no playhead do gate e na varredura. ──
    let at_gate = run(gpu, reg, PLAYHEAD, world);
    let (lo, hi) = at_gate.p_range;
    eprintln!(
        "BASE @ t={PLAYHEAD}: |P.x| ∈ [{:e}, {:e}] · |P.y| ∈ [{:e}, {:e}] · ulp(max|P.y|)={:e}",
        lo[0],
        hi[0],
        lo[1],
        hi[1],
        ulp(hi[1])
    );
    line("noise.v", &at_gate.noise);
    line("slope.v", &at_gate.slope);
    line("drive.Py", &at_gate.py);
    line("drive.Px", &at_gate.px);
    // A propagação esperada: `slope` herda `(dn[hi] − dn[lo]) / span · 2,3`; o `P.y` herda
    // `2 · dslope` mais o arredondamento da soma em `|P.y|`.
    eprintln!(
        "  propagação: 2,3·max|dnoise| = {:e} · 2·max|dslope| = {:e} · ½ulp(max|P.y|) = {:e}",
        2.3 * at_gate.noise.max_d,
        2.0 * at_gate.slope.max_d,
        0.5 * ulp(hi[1])
    );

    let mut floor = [Stage::default(); 3];
    let mut band = (0.0f32, 0.0f32);
    for &t in times {
        let r = run(gpu, reg, t, world);
        eprintln!(
            "  sweep t={t:>6.3}: noise {:e} ({:.1} ULP-col; i<64 {:e} · i≥512 {:e}) · slope {:e} ({:.1}) · P.y {:e} ({:.1})",
            r.noise.max_d,
            r.noise.ulp_col,
            r.noise_band.0,
            r.noise_band.1,
            r.slope.max_d,
            r.slope.ulp_col,
            r.py.max_d,
            r.py.ulp_col
        );
        band = (band.0.max(r.noise_band.0), band.1.max(r.noise_band.1));
        for (f, s) in floor.iter_mut().zip([r.noise, r.slope, r.py]) {
            if s.max_d > f.max_d {
                *f = s;
            }
        }
    }
    eprintln!(
        "RUÍDO (pior sobre {} playheads; |dnoise| i<64 = {:e} · i≥512 = {:e}):",
        times.len(),
        band.0,
        band.1
    );
    line("noise.v", &floor[0]);
    line("slope.v", &floor[1]);
    line("drive.Py", &floor[2]);

    // ── O lado do DEFEITO: mutantes reais do kernel, o MÍNIMO sobre os playheads (o pior caso
    // para quem quer apanhá-los). ──
    eprintln!(
        "DEFEITOS (mínimo sobre {} playheads do max|d| de P.y):",
        times.len()
    );
    for (name, edits) in mutants {
        let mreg = mutant(reg, edits);
        let mut min_py: Option<Stage> = None;
        let mut gate_at = Stage::default();
        for &t in times {
            let r = run(gpu, &mreg, t, world);
            if t == PLAYHEAD {
                gate_at = r.py;
            }
            if min_py.is_none_or(|m| r.py.max_d < m.max_d) {
                min_py = Some(r.py);
            }
        }
        eprintln!("{name}");
        line("@gate", &gate_at);
        line("min", &min_py.expect("pelo menos um playhead"));
    }
}
