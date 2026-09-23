//! ⭐⭐⭐ **A VARIANTE de um kernel entra na chave da cache de pipelines** (doc 119, achado da W3).
//!
//! Um nó cujo kernel escolhe uma VARIANTE por param (`GpuKernel::variant_by_param` — o `channel`
//! do `motion.noise`: `Y` e `Position XY` são dois módulos WGSL) compila um pipeline por variante.
//! A cache guardava-os por `(tipo, colunas presentes)` — e duas variantes com as MESMAS colunas
//! colidiam: a segunda recebia o pipeline da primeira.
//!
//! ⛔⛔ **Duas formas do mesmo defeito, e a 2.ª é de produto sem multi-sink nenhum:**
//! - dois ruídos de canais diferentes no MESMO plano (a união de duas saídas, ou uma cadeia) — o
//!   2.º encenado saía com a lei do 1.º (medido `0,33` contra a CPU);
//! - o artista troca o canal `Y → Position XY` com a cena na placa: a cache persiste entre
//!   quadros, e **nada mudava no ecrã** (medido `0,26`).
//!
//! Achado pela varredura das cenas de várias saídas contra a CPU
//! (`ph2d-app-motion`, `as_cenas_de_varias_saidas_pela_placa_dao_o_que_a_cpu_da`): as cenas de
//! demo tinham os dois canais lado a lado, e antes do ciclo 11 corriam na CPU.
//!
//! `#[ignore]`: precisa de adapter real.

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

/// A barra dos gates de paridade de nó puro desta crate (`gpu_cpu_parity_xy`): o ruído é o
/// mesmo `f32` nos dois motores, e o que se afirma é a lei, não uma tolerância.
const EPS: f32 = 1e-5;
const AT: f64 = 0.05;
const Y: f32 = 1.0;
const XY: f32 = 4.0;

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
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

fn ruido(g: &mut Graph, canal: f32) -> (NodeId, NodeId) {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 6.0);
    g.set_param(grid, "cols", 6.0);
    let ns = g.add_node("motion.noise");
    g.set_param(ns, "amplitude", 0.6);
    g.set_param(ns, "channel", canal);
    let out = g.add_node("motion.output");
    for (a, b) in [(grid, ns), (ns, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    (ns, out)
}

/// O pior desvio entre a coluna `P` de `ns` na placa e na CPU.
fn desvio(
    gpu: &GpuContext,
    gc: &ph2d_gpu_cook::GpuCook,
    g: &Graph,
    reg: &NodeRegistry,
    ns: NodeId,
) -> f32 {
    let p = gc
        .read_column_vec2(gpu, ns, "P")
        .expect("P volta do device");
    let mut cook = Cook::new();
    let cpu = cook.cook(g, reg, ns, AT).expect("cpu");
    let Some(Column::Vec2(c)) = cpu[0].as_stream().get("P") else {
        panic!("sem P na CPU")
    };
    assert_eq!(p.len(), c.len());
    c.iter()
        .zip(&p)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0f32, f32::max)
}

/// ⭐⭐ **Duas variantes do mesmo nó no MESMO plano** — pelas duas ordens, as duas batem com a CPU.
#[test]
#[ignore = "needs a GPU adapter"]
fn duas_variantes_do_mesmo_no_no_mesmo_plano_batem_as_duas() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltando");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (ny, oy) = ruido(&mut g, Y);
    let (nxy, oxy) = ruido(&mut g, XY);
    g.validate(&reg).expect("bem tipado");
    for ordem in [[oy, oxy], [oxy, oy]] {
        let plan = ph2d_gpu_cook::plan_driven_many(&g, &reg, &reg, &ordem, &Default::default());
        assert!(plan.is_fully_gpu());
        let mut gc = ph2d_gpu_cook::GpuCook::new();
        gc.retain_streams_for_debug(true);
        gc.cook_many(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(AT),
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            &[SinkStyle::PLAIN, SinkStyle::PLAIN],
        )
        .expect("gpu cook");
        for (rotulo, ns) in [("Y", ny), ("XY", nxy)] {
            let d = desvio(&gpu, &gc, &g, &reg, ns);
            assert!(
                d <= EPS,
                "ordem {ordem:?}: o ruido {rotulo} desvia {d:e} da CPU"
            );
        }
    }
}

/// ⭐⭐⭐ **Trocar a variante entre dois quadros, no MESMO cozedor** — o gesto do artista (o painel
/// muda o canal com a cena a correr na placa). A cache persiste entre quadros.
#[test]
#[ignore = "needs a GPU adapter"]
fn trocar_a_variante_entre_quadros_muda_o_que_a_placa_coze() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — saltando");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let (ns, _) = ruido(&mut g, Y);
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    for canal in [Y, XY, Y] {
        g.set_param(ns, "channel", canal);
        let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, ns);
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(AT),
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            SinkStyle::PLAIN,
        )
        .expect("gpu cook");
        let d = desvio(&gpu, &gc, &g, &reg, ns);
        assert!(
            d <= EPS,
            "canal {canal}: a placa desvia {d:e} da CPU depois da troca"
        );
    }
}
