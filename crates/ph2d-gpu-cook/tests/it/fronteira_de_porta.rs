//! **A fronteira CPU→GPU é endereçada por `(nó, porta)`** (GEF 08, Motion Input onda 3).
//!
//! O `value.input` (CPU) entrega o par em `xy` (porta 6) e a cor em `r..a` (1–4). Antes, a bomba só
//! entregava a porta 0 e o plano recuava o leitor para a CPU. Agora a bomba entrega cada porta
//! (`boundary_port_streams`), o cozimento recebe-as (`set_boundary_ports`) e o leitor fica na placa.

use super::placa::try_headless_gpu;
use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuSource};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::{Cook, TimeScopes};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;
/// O ε dos irmãos (`gpu_cpu_parity_arith`): só o FMA que o WGSL pode contrair.
const EPS: f32 = 1e-4;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn edge(g: &mut Graph, a: NodeId, pa: u16, b: NodeId, pb: u16) {
    g.connect(Edge {
        from: (a, pa),
        to: (b, pb),
        delayed: false,
    })
    .expect("fio bem tipado");
}

/// `value.input.xy ─▶ motion.move`: o par `(2,5, 0)` (sem publicação, `(out, 0)`) deslocado.
fn par_no_move() -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let vi = g.add_node("value.input");
    g.set_param(vi, "value", 2.5);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 0.5);
    g.set_param(mv, "dy", 1.25);
    edge(&mut g, vi, 6, mv, 0);
    (g, vi, mv)
}

/// `motion.grid ─▶ motion.drive ◀─ value.input.r`: a cor (`r = 0,37`) dirige o Y.
fn cor_no_drive() -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 4.0);
    g.set_param(grid, "cols", 3.0);
    let vi = g.add_node("value.input");
    g.set_param(vi, "kind", 3.0); // Color
    g.set_param(vi, "r", 0.37);
    g.set_param(vi, "g", 0.81);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    edge(&mut g, grid, 0, drive, 0);
    edge(&mut g, vi, 1, drive, 1);
    (g, vi, drive)
}

/// O leitor fica na placa e lê a fronteira PELA PORTA.
fn assert_leitor_na_placa(g: &Graph, reg: &NodeRegistry, vi: NodeId, leitor: NodeId, porta: usize) {
    g.validate(reg).expect("bem tipado");
    let plan = ph2d_gpu_cook::plan(g, reg, reg, leitor);
    let estagio = plan.stages.iter().find(|s| s.node == leitor);
    assert!(
        estagio.is_some_and(|s| s.inputs.contains(&GpuSource::Boundary(vi, porta))),
        "o leitor recuou para a CPU: estágios {:?}, fronteiras {:?}",
        plan.stages.iter().map(|s| s.node).collect::<Vec<_>>(),
        plan.boundaries
    );
    assert_eq!(plan.boundaries, vec![(vi, porta)]);
}

#[test]
fn o_par_do_value_input_fica_na_placa() {
    let reg = registry();
    let (g, vi, mv) = par_no_move();
    assert_leitor_na_placa(&g, &reg, vi, mv, 6);
}

#[test]
fn a_cor_do_value_input_fica_na_placa() {
    let reg = registry();
    let (g, vi, drive) = cor_no_drive();
    assert_leitor_na_placa(&g, &reg, vi, drive, 1);
}

/// Coze pela rota híbrida do produto (a bomba entrega, a placa coze) e devolve `(cpu, placa)` do P.
fn nas_duas(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    g: &Graph,
    leitor: NodeId,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let plan = ph2d_gpu_cook::plan(g, reg, reg, leitor);
    let mut cook = Cook::new();
    let cpu = cook.cook(g, reg, leitor, PLAYHEAD).expect("cpu");
    let cpu_p = match cpu[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("sem P na CPU"),
    };
    let mut pump = ph2d_eval_motion::MotionCookPump::new();
    let nodes: Vec<_> = plan.boundaries.iter().map(|(n, _)| *n).collect();
    pump.advance_or_scrub_to_nodes_scoped(g, reg, &nodes, 0, |_| PLAYHEAD, &TimeScopes::default());
    let handed: Vec<_> = pump
        .boundary_streams()
        .iter()
        .map(|(n, s)| (*n, s))
        .collect();
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.set_boundary_ports(pump.boundary_port_streams().to_vec());
    gc.cook(
        gpu,
        g,
        reg,
        reg,
        &plan,
        &handed,
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .expect("a placa coze a fronteira de porta");
    let placa = gc
        .read_column_vec2(gpu, leitor, "P")
        .expect("P volta da placa");
    // O controlo: sem as portas a placa RECUSA (nunca lê a porta 0 no lugar da 6).
    let mut cego = ph2d_gpu_cook::GpuCook::new();
    let recusa = cego.cook(
        gpu,
        g,
        reg,
        reg,
        &plan,
        &handed,
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    );
    assert!(recusa.is_err(), "sem as portas o cozimento tem de recusar");
    (cpu_p, placa)
}

fn assert_iguais(label: &str, cpu: &[[f32; 2]], placa: &[[f32; 2]]) {
    assert_eq!(cpu.len(), placa.len(), "{label}: contagem");
    for (i, (c, p)) in cpu.iter().zip(placa).enumerate() {
        for k in 0..2 {
            assert!(
                (c[k] - p[k]).abs() <= EPS,
                "{label} [{i}][{k}]: cpu {c:?} placa {p:?}"
            );
        }
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn o_par_e_a_cor_chegam_a_placa_como_na_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, _, mv) = par_no_move();
    let (cpu, placa) = nas_duas(&gpu, &reg, &g, mv);
    assert_iguais("xy → move", &cpu, &placa);
    assert!(
        (cpu[0][0] - 3.0).abs() < EPS && (cpu[0][1] - 1.25).abs() < EPS,
        "fixture: {cpu:?}"
    );

    let (g, _, drive) = cor_no_drive();
    let (cpu, placa) = nas_duas(&gpu, &reg, &g, drive);
    assert_iguais("r → drive", &cpu, &placa);
    // O controlo da fixture: o `g` (0,81) no lugar do `r` daria outro Y.
    let (g0, _, d0) = {
        let (mut g0, vi, d) = cor_no_drive();
        g0.set_param(vi, "r", 0.0);
        (g0, vi, d)
    };
    let mut cook = Cook::new();
    let sem_r = cook.cook(&g0, &reg, d0, PLAYHEAD).expect("cpu");
    let Some(Column::Vec2(sem_r)) = sem_r[0].as_stream().get("P") else {
        panic!("sem P");
    };
    assert!(
        (sem_r[0][1] - cpu[0][1]).abs() > 0.5,
        "o r tem de mover o Y"
    );
}
