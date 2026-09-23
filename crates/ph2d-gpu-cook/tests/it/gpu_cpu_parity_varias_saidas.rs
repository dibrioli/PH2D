//! ⭐⭐⭐ **VÁRIAS SAÍDAS NA PLACA dão o que a CPU dá** (doc 119 W3).
//!
//! A CPU compõe N saídas num buffer só (`lower_to_instances_onto`: limpa uma vez e cada
//! `motion.output` acrescenta). A placa passa a fazer o MESMO: o plano da união (W2) coze cada nó
//! uma vez e `cook_many` baixa as saídas uma a seguir à outra no mesmo buffer. Este gate compara
//! os dois buffers **linha a linha**, na ordem das saídas.
//!
//! | caso | o que prende |
//! |---|---|
//! | duas saídas, a 2.ª em `Add` | a concatenação, o nó partilhado cozido uma vez, e a partição a cobrir as duas faixas |
//! | uma saída CALADA no meio | o deslocamento é o que FOI escrito — somar a contagem da calada deixaria lixo entre as vizinhas |
//!
//! `#[ignore]`: precisa de adapter real.
//!   cargo test -p ph2d-gpu-cook --test it -- --ignored gpu_cpu_parity_varias_saidas

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan_driven_many, read_instances};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::{GpuTexRun, RenderInstance, SinkStyle};

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
/// A barra dos gates de paridade de posição desta crate (`gpu_collide`): o oscilador corre `sin`
/// nos dois motores.
const EPS: f32 = 1e-5;
const AT: f64 = 0.37;

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
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

fn edge(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

/// Uma grelha PARTILHADA por três saídas: `grid → osc → move → A`, `grid → move → B` e
/// `grid → C`. Devolve o grafo e `[A, B, C]`.
fn tres_saidas(reg: &NodeRegistry) -> (Graph, [NodeId; 3]) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 5.0);
    g.set_param(grid, "cols", 7.0);
    let osc = g.add_node("motion.oscillator");
    let mv_a = g.add_node("motion.move");
    g.set_param(mv_a, "dx", 1.5);
    let a = g.add_node("motion.output");
    edge(&mut g, grid, osc);
    edge(&mut g, osc, mv_a);
    edge(&mut g, mv_a, a);
    let mv_b = g.add_node("motion.move");
    g.set_param(mv_b, "dx", -2.0);
    let b = g.add_node("motion.output");
    edge(&mut g, grid, mv_b);
    edge(&mut g, mv_b, b);
    let c = g.add_node("motion.output");
    edge(&mut g, grid, c);
    g.validate(reg).expect("bem tipado");
    (g, [a, b, c])
}

/// A CPU: cada saída cozida e baixada ONTO o mesmo buffer, pela ordem — o que o pump faz.
fn cpu(
    g: &Graph,
    reg: &NodeRegistry,
    sinks: &[NodeId],
    styles: &[SinkStyle],
) -> Vec<RenderInstance> {
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for (&s, &style) in sinks.iter().zip(styles) {
        let r = cook.cook(g, reg, s, AT).expect("cpu cook");
        ph2d_eval_motion::lower_to_instances_onto(
            r[0].as_stream(),
            DEFAULT_UV,
            DEFAULT_SIZE,
            style,
            &mut out,
        );
    }
    out
}

fn placa(
    gpu: &GpuContext,
    g: &Graph,
    reg: &NodeRegistry,
    sinks: &[NodeId],
    styles: &[SinkStyle],
) -> (Vec<RenderInstance>, Vec<GpuTexRun>, u32) {
    let plan = plan_driven_many(g, reg, reg, sinks, &Default::default());
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    assert_eq!(plan.sinks, sinks, "todas as saídas vão à placa");
    let mut gc = GpuCook::new();
    let total = gc
        .cook_many(
            gpu,
            g,
            reg,
            reg,
            &plan,
            &[],
            CookClock::at(AT),
            DEFAULT_UV,
            DEFAULT_SIZE,
            styles,
        )
        .expect("gpu cook");
    let inst = read_instances(gpu, gc.instances().expect("cozido"));
    (inst, gc.texture_runs().to_vec(), total)
}

fn paridade(cpu: &[RenderInstance], gpu: &[RenderInstance]) {
    assert_eq!(cpu.len(), gpu.len(), "a contagem de linhas");
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            let d = (c.world_pos[k] - g.world_pos[k]).abs();
            assert!(
                d <= EPS,
                "linha {i} world_pos[{k}]: cpu {} placa {}",
                c.world_pos[k],
                g.world_pos[k]
            );
        }
        assert_eq!(c.flip_uv, g.flip_uv, "linha {i}: a mistura empacotada");
        assert_eq!(c.size, g.size, "linha {i}: o tamanho");
    }
}

/// ⭐⭐⭐ **Duas saídas, a 2.ª em `Add`, dão o buffer da CPU linha a linha** — e a partição cobre
/// as DUAS faixas: a 1.ª com o run de átlas em `Mix`, a 2.ª com o `Add` dela. ⚠️ Sem o run da 1.ª,
/// o ramo dos runs do desenho saltava a 1.ª saída inteira.
#[test]
#[ignore = "needs a GPU adapter"]
fn duas_saidas_na_placa_dao_o_buffer_da_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltando");
        return;
    };
    let reg = registry();
    let (g, [a, b, _]) = tres_saidas(&reg);
    let add = SinkStyle {
        blend: 1,
        ..SinkStyle::PLAIN
    };
    let styles = [SinkStyle::PLAIN, add];
    let c = cpu(&g, &reg, &[a, b], &styles);
    let (p, runs, total) = placa(&gpu, &g, &reg, &[a, b], &styles);
    assert_eq!(c.len(), 70, "35 linhas por saída");
    assert_eq!(total, 70);
    paridade(&c, &p);
    assert_eq!(
        runs,
        vec![
            GpuTexRun {
                texture_id: 0,
                start: 0,
                end: 35,
                blend: 0,
                sampling: 0,
            },
            GpuTexRun {
                texture_id: 0,
                start: 35,
                end: 70,
                blend: 1,
                sampling: 0,
            },
        ]
    );
}

/// ⭐⭐ **Uma saída CALADA no meio não deixa buraco** — a lei do dono cala uma corrente de
/// posições sem forma (`so_com_forma`), e a saída seguinte tem de começar onde a anterior acabou.
/// ⚠️ O CONTROLO é a CPU, que baixa a calada como nada: se a placa somasse a contagem dela, a 3.ª
/// saída ficaria 35 linhas mais à frente e as linhas do meio seriam lixo.
#[test]
#[ignore = "needs a GPU adapter"]
fn uma_saida_calada_no_meio_nao_deixa_buraco() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltando");
        return;
    };
    let reg = registry();
    let (g, [a, b, c_]) = tres_saidas(&reg);
    let calada = SinkStyle {
        so_com_forma: true,
        ..SinkStyle::PLAIN
    };
    let styles = [SinkStyle::PLAIN, calada, SinkStyle::PLAIN];
    let c = cpu(&g, &reg, &[a, b, c_], &styles);
    let (p, _, _) = placa(&gpu, &g, &reg, &[a, b, c_], &styles);
    assert_eq!(c.len(), 70, "a do meio não produz linha nenhuma na CPU");
    paridade(&c, &p);
}

/// ⭐⭐ **Cada saída lê as texturas da SUA fronteira** — duas saídas, cada uma sobre um objecto
/// que vem da CPU (um `sort` sem kernel faz de fronteira), com as DUAS colunas `texture_id` do
/// MESMO comprimento. ⚠️ Com uma saída a partição acha a fronteira pelo comprimento entre todas; com
/// duas do mesmo comprimento essa lei daria à 2.ª saída as texturas da 1.ª — e é a linhagem da
/// porta 0 de cada saída ([`ph2d_gpu_cook::GpuPlan::lineage_boundary`]) que dá o endereço certo.
#[test]
#[ignore = "needs a GPU adapter"]
fn cada_saida_le_as_texturas_da_sua_fronteira() {
    use ph2d_nodegraph::attr::{Column, Stream};
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltando");
        return;
    };
    let mut reg = registry();
    ph2d_node_motion_sort::register(&mut reg).unwrap();
    let mut g = Graph::new();
    let mut fontes = [NodeId(0); 2];
    let mut saidas = [NodeId(0); 2];
    for k in 0..2 {
        let grid = g.add_node("motion.grid");
        let srt = g.add_node("motion.sort");
        let mv = g.add_node("motion.move");
        let out = g.add_node("motion.output");
        edge(&mut g, grid, srt);
        edge(&mut g, srt, mv);
        edge(&mut g, mv, out);
        fontes[k] = srt;
        saidas[k] = out;
    }
    g.validate(&reg).expect("bem tipado");
    let plan = plan_driven_many(&g, &reg, &reg, &saidas, &Default::default());
    assert_eq!(plan.sinks, saidas);
    assert_eq!(plan.lineage_boundary(saidas[0]), Some(fontes[0]));
    assert_eq!(plan.lineage_boundary(saidas[1]), Some(fontes[1]));
    let objecto = |id: f32| {
        let mut s = Stream::new(3);
        s.set("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
        s.set("texture_id", Column::Scalar(vec![id; 3]));
        s
    };
    let (sa, sb) = (objecto(7.0), objecto(9.0));
    let mut gc = GpuCook::new();
    gc.cook_many(
        &gpu,
        &g,
        &reg,
        &reg,
        &plan,
        &[(fontes[0], &sa), (fontes[1], &sb)],
        CookClock::at(AT),
        DEFAULT_UV,
        DEFAULT_SIZE,
        &[SinkStyle::PLAIN, SinkStyle::PLAIN],
    )
    .expect("gpu cook");
    let run = |texture_id, start, end| GpuTexRun {
        texture_id,
        start,
        end,
        blend: 0,
        sampling: 0,
    };
    assert_eq!(
        gc.texture_runs(),
        [run(7, 0, 3), run(9, 3, 6)],
        "a 2.ª saída tem de ler a textura da fronteira DELA"
    );
}
