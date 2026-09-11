//! GPU-vs-CPU parity para o canal **`Position XY`** — o `Separate Channels` da
//! folha 06, no `motion.noise` e no `motion.wiggle`.
//!
//! ## Por que este arquivo existe, e não uma linha nos gates de crate
//!
//! O deslocamento que decorrelaciona os dois eixos está escrito **duas vezes**: uma
//! const de Rust e um literal dentro da string WGSL do variant. Os gates de crate
//! (`the_wgsl_carries_the_same_axis_offset_as_the_rust`) comparam as duas ESCRITAS —
//! e uma escrita igual não prova um resultado igual. ⚠️ *Só o device diz se o número
//! faz lá o que faz cá*: um `i32` que estoure, um `f32` que arredonde diferente, um
//! `+` na ordem trocada, e a string continua idêntica.
//!
//! ## E ele mede as DUAS coisas, porque um par de eixos tem dois modos de falha
//!
//! 1. **Paridade** — cada eixo concorda com a CPU dentro do ULP.
//! 2. **Decorrelação NO DEVICE** — os dois eixos continuam campos diferentes lá.
//!    ⚠️ Sem a segunda, um WGSL que deslocasse por `0` passaria na paridade **se a
//!    CPU também o fizesse** — e passaria sozinho na paridade de UM eixo, porque
//!    `dx` estaria certo e só o `dy` errado. A diagonal a 45° é invisível a qualquer
//!    régua de "concorda com a CPU" aplicada a um eixo de cada vez.
//!
//! `#[ignore]`: precisa de adapter real.
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_xy --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.41;
/// O ULP relativo dos gates irmãos — aqui não há tabela nenhuma, é aritmética pura.
const EPS_REL: f32 = 1.0e-5;
/// A barra da decorrelação, a mesma dos gates de crate. ⚠️ Ela fica longe de **1** (o
/// valor exacto do defeito) e não colada em **0**, porque o resíduo é erro de
/// amostragem e não acoplamento: a sonda do `motion.noise` vê-o cair de `0,120` para
/// `0,009` só ao afinar o campo. Medido NO DEVICE nesta fixture: `−0,222` (noise) e
/// `0,079` (wiggle).
const MAX_R: f32 = 0.5;

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
    ph2d_node_motion_noise::register(&mut reg).unwrap();
    ph2d_node_motion_wiggle::register(&mut reg).unwrap();
    reg
}

fn grid(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 24.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    grid
}

/// As posições dos dois lados, para o mesmo grafo.
fn cook_both(gpu: &GpuContext, reg: &NodeRegistry, node: &str) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut g = Graph::new();
    let src = grid(&mut g);
    let sink = g.add_node(node);
    g.connect(Edge {
        from: (src, 0),
        to: (sink, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(sink, "amplitude", 1.4);
    // O canal dos DOIS eixos — o índice 4 nos dois nós.
    g.set_param(sink, "channel", 4.0);
    g.validate(reg).unwrap_or_else(|e| panic!("{node}: {e:?}"));
    let plan = ph2d_gpu_cook::plan(&g, reg, reg, sink);
    assert!(
        plan.is_fully_gpu(),
        "{node}: o canal XY tem de ser reivindicado pelo device"
    );

    let mut cook = Cook::new();
    let cpu = cook
        .cook(&g, reg, sink, PLAYHEAD)
        .unwrap_or_else(|e| panic!("{node}: cpu cook {e:?}"));
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        gpu,
        &g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .unwrap_or_else(|e| panic!("{node}: gpu cook {e:?}"));

    let cpu_p = match cpu[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("{node}: sem P"),
    };
    let gpu_p = gc
        .read_column_vec2(gpu, sink, "P")
        .unwrap_or_else(|| panic!("{node}: P não volta do device"));
    (cpu_p, gpu_p)
}

fn pearson(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() as f32;
    let ma = a.iter().sum::<f32>() / n;
    let mb = b.iter().sum::<f32>() / n;
    let (mut cov, mut va, mut vb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        cov += (x - ma) * (y - mb);
        va += (x - ma) * (x - ma);
        vb += (y - mb) * (y - mb);
    }
    if va <= 0.0 || vb <= 0.0 {
        return 0.0;
    }
    cov / (va * vb).sqrt()
}

fn swing(v: &[f32]) -> f32 {
    v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
        - v.iter().fold(f32::INFINITY, |m, x| m.min(*x))
}

/// **OS DOIS EIXOS, NOS DOIS NÓS, CONCORDAM COM A CPU — E CONTINUAM DIFERENTES.**
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_two_axis_channel_matches_the_cpu_and_stays_decorrelated_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for node in ["motion.noise", "motion.wiggle"] {
        let (cpu, dev) = cook_both(&gpu, &reg, node);
        assert_eq!(cpu.len(), dev.len(), "{node}: contagem");
        // A base é a grelha; o deslocamento é o que os dois nós escreveram. Como o
        // eixo X da grelha NÃO é zero, a paridade mede a posição final (que é o que
        // o produto desenha) e a decorrelação mede o DESLOCAMENTO — as duas coisas
        // certas, cada uma sobre a grandeza dela.
        for axis in 0..2 {
            let a: Vec<f32> = cpu.iter().map(|p| p[axis]).collect();
            let b: Vec<f32> = dev.iter().map(|p| p[axis]).collect();
            let worst = a
                .iter()
                .zip(&b)
                .map(|(x, y)| (x - y).abs())
                .fold(0.0, f32::max);
            let sw = swing(&a);
            eprintln!(
                "{node:<16} eixo {axis}: max |d| = {worst:e}  amplitude = {sw:.3}  rel = {:e}",
                worst / sw
            );
            assert!(
                worst < EPS_REL * sw,
                "{node} eixo {axis}: {worst:e} sobre {sw:.3}"
            );
        }
        // ⚠️ A decorrelação MEDIDA NO DEVICE, sobre os deslocamentos: um WGSL que
        // deslocasse por zero passaria em tudo acima.
        let mut g = Graph::new();
        let src = grid(&mut g);
        let mut cook = Cook::new();
        let base = match cook.cook(&g, &reg, src, PLAYHEAD).unwrap()[0]
            .as_stream()
            .get("P")
        {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("base"),
        };
        let dx: Vec<f32> = base.iter().zip(&dev).map(|(a, b)| b[0] - a[0]).collect();
        let dy: Vec<f32> = base.iter().zip(&dev).map(|(a, b)| b[1] - a[1]).collect();
        assert!(swing(&dx) > 0.5, "{node}: o eixo X anda no device");
        assert!(swing(&dy) > 0.5, "{node}: o eixo Y anda no device");
        let r = pearson(&dx, &dy);
        eprintln!("{node:<16} correlacao entre os eixos NO DEVICE: {r}");
        assert!(
            r.abs() < MAX_R,
            "{node}: os dois eixos tem de ser campos DIFERENTES no device, r = {r}"
        );
    }
}
