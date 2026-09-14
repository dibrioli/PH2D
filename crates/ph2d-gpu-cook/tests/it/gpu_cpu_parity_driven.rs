//! ⭐⭐⭐ **PARIDADE CPU↔GPU DE UM PARAM DIRIGIDO** (doc 110 §3 · doc 102 W1).
//!
//! Até esta wave um fio de valor a chegar a um param **derrubava a cadeia inteira para a CPU** —
//! medido, `6 de 6`, e a primeira era uma constante. O planeador deixou de recusar; o que ele
//! passou a exigir é o NÚMERO, entregue por quem coze o condutor.
//!
//! ⛔⛔ **E é exactamente aí que mora a única falha grave possível desta wave: as duas rotas a
//! desenharem documentos DIFERENTES.** O gate de plano (`plan_analysis`) prova que o nó fica no
//! dispositivo; ele não prova que o dispositivo lê o número CERTO. Um `drv` que chegasse a zero, um
//! deslocamento trocado no uniform, um valor de outro tique — tudo isso passa num gate de rota e
//! sai no ecrã do artista.
//!
//! ⚠️ **O controlo é a metade que impede este gate de ser vazio:** dois valores dirigidos
//! diferentes têm de produzir campos diferentes. Sem ele, um device que ignorasse o fio e usasse o
//! default concordaria com uma CPU que fizesse o mesmo — e as duas estariam erradas juntas.

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, DrivenParams};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;
/// O orçamento herdado dos gates irmãos (ADR-0126).
const EPS: f32 = 1e-4;

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
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    reg
}

/// A cadeia: `grid → move → output`, com o `dx` do `move` **dirigido** por um `value.lfo` cuja
/// amplitude é `amp` — o número que tem de chegar ao dispositivo.
fn chain(reg: &NodeRegistry, amp: f32) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "amplitude", amp);
    for (a, b) in [(grid, mv), (mv, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    g.drive_param(mv, "dx", (lfo, 0)).unwrap();
    g.validate(reg).expect("a cadeia é bem tipada");
    (g, out, mv)
}

/// Um nó e os fios que chegam aos params dele — o mesmo par que a produção fotografa.
type FioDeParam = (NodeId, Vec<(String, (NodeId, u16))>);

/// **Os valores dirigidos, derivados como a produção os deriva** — o condutor cozido na CPU e o
/// elemento `0` lido pela MESMA porta que o `EvalCtx::param` usa.
///
/// ⚠️ Uma segunda leitura do mesmo valor (um «ler o param do lfo à mão») seria a forma clássica de
/// este gate concordar consigo próprio em vez de com o produto.
fn valores(g: &Graph, reg: &NodeRegistry, cook: &mut Cook) -> DrivenParams {
    let mut fora = DrivenParams::new();
    let fios: Vec<FioDeParam> = g
        .all_param_sources()
        .iter()
        .map(|(n, m)| (*n, m.iter().map(|(p, s)| (p.clone(), *s)).collect()))
        .collect();
    for (node, params) in fios {
        for (param, (src, port)) in params {
            let saida = cook.cook(g, reg, src, PLAYHEAD).expect("o condutor coze");
            let v = saida
                .get(port as usize)
                .and_then(ph2d_nodegraph::param_source::driven_value);
            fora.entry(node).or_default().insert(param, v);
        }
    }
    fora
}

/// Corre as DUAS rotas sobre a mesma cadeia e devolve `(cpu_x, gpu_x)`.
fn cook_on_both(gpu: &GpuContext, reg: &NodeRegistry, amp: f32) -> (Vec<f32>, Vec<f32>) {
    let (g, out, mv) = chain(reg, amp);
    let mut cook = Cook::new();
    let driven = valores(&g, reg, &mut cook);
    assert!(
        driven.get(&mv).is_some_and(|m| m["dx"].is_some()),
        "a fixture tem de ENTREGAR um número, senão mede a lei do condutor vazio"
    );
    let plan = ph2d_gpu_cook::plan_driven(&g, reg, reg, out, &driven);
    assert!(
        plan.is_fully_gpu(),
        "com o valor na mão a cadeia é do dispositivo: {:?}",
        plan.boundaries
    );
    let cpu = cook.cook(&g, reg, out, PLAYHEAD).expect("cpu cook");
    let cpu_x: Vec<f32> = match cpu[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
        _ => panic!("sem coluna P"),
    };
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.set_driven(driven);
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
    .expect("gpu cook");
    let gpu_x: Vec<f32> = gc
        .read_column_vec2(gpu, out, "P")
        .expect("P volta do device")
        .iter()
        .map(|p| p[0])
        .collect();
    (cpu_x, gpu_x)
}

/// ⭐⭐⭐ **O DISPOSITIVO LÊ O NÚMERO QUE O FIO TRAZ, e concorda com a CPU.**
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_device_reads_the_driven_param_and_agrees_with_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (cpu, dev) = cook_on_both(&gpu, &reg, 0.8);
    assert_eq!(cpu.len(), dev.len(), "as duas rotas dão 64 peças");
    let pior = cpu
        .iter()
        .zip(&dev)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    assert!(pior < EPS, "as duas rotas divergem em {pior}");

    // ⚠️ **O CONTROLO**: com outra amplitude o campo TEM de mudar — senão este gate estaria a
    // comparar duas rotas que ignoram o fio da mesma maneira.
    let (_, outro) = cook_on_both(&gpu, &reg, 0.2);
    let mudou = dev
        .iter()
        .zip(&outro)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        mudou > 1e-2,
        "mudar o valor dirigido tem de mudar o campo, e mudou {mudou} -- o device esta' a \
         ignorar o fio e a usar o default"
    );
}
