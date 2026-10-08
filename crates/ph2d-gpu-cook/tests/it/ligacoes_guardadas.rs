//! ⭐ **As ligações GUARDADAS entre quadros** (doc 122 §3 do Motion) — a memória que tirou o custo de
//! recriar `BindGroup`s e uniforms a cada quadro. Nenhum gate de paridade a vê (guardar ou recriar dá a
//! MESMA imagem): a régua é o TAMANHO dela — guarda alguma coisa, não cresce no regime, e o que saiu de
//! uso morre.

use super::placa::try_headless_gpu;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

fn liga(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

#[test]
#[ignore = "precisa de adapter de GPU"]
fn as_ligacoes_guardadas_nao_crescem_e_as_velhas_morrem() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo no' regista");
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "amplitude", 0.5);
    let out = g.add_node("motion.output");
    liga(&mut g, grid, osc);
    liga(&mut g, osc, out);
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let coze = |gc: &mut ph2d_gpu_cook::GpuCook, g: &Graph, t: f64| {
        let plan = ph2d_gpu_cook::plan(g, &reg, &reg, out);
        assert!(plan.is_fully_gpu());
        gc.cook(
            &gpu,
            g,
            &reg,
            &reg,
            &plan,
            &[],
            CookClock::at(t),
            [0.0, 0.0, 1.0, 1.0],
            [0.1, 0.1],
            SinkStyle::PLAIN,
        )
        .expect("cozimento na placa");
    };
    g.set_param(grid, "rows", 30.0);
    g.set_param(grid, "cols", 30.0);
    let mut vivas = Vec::new();
    for k in 0..30 {
        coze(&mut gc, &g, f64::from(k) / 60.0);
        vivas.push(gc.ligacoes_vivas().1);
    }
    let regime = vivas[29];
    assert!(regime > 0, "a memoria GUARDA as ligacoes das etapas");
    assert_eq!(
        vivas[10], regime,
        "no regime (o mesmo grafo, o mesmo tamanho) a memoria nao cresce: {vivas:?}"
    );
    // Populações novas: o pool dá buffers de outras classes, e as ligações velhas saem de uso.
    for (k, lado) in [8.0f32, 50.0, 12.0, 70.0, 20.0, 90.0]
        .into_iter()
        .enumerate()
    {
        g.set_param(grid, "rows", lado);
        g.set_param(grid, "cols", lado);
        coze(&mut gc, &g, 1.0 + k as f64 / 60.0);
    }
    g.set_param(grid, "rows", 30.0);
    g.set_param(grid, "cols", 30.0);
    for k in 0..20 {
        coze(&mut gc, &g, 2.0 + f64::from(k) / 60.0);
    }
    let depois = gc.ligacoes_vivas().1;
    assert!(
        depois <= 2 * regime,
        "o que saiu de uso morre pela idade: {depois} ligacoes vivas contra {regime} no regime"
    );
}
