//! ⭐⭐⭐ **A leitura dos cartões sem esperar pela placa** (ciclo 12, doc 120 §8.6) — ver o
//! cabeçalho de `ph2d_gpu_cook::tap_voo`.
//!
//! A régua não é um relógio (um relógio aqui seria mais um membro da família de flakes de carga):
//! é o que a leitura DEVOLVE em cada chamada, e que ela lê **as mesmas amostras** que o `tap`
//! síncrono.

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo no' regista");
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

/// ⭐⭐⭐ **Encomenda num quadro, recolhe no seguinte — e o que recolhe é o que o síncrono lê.**
///
/// As quatro metades, cada uma contra uma cura barata:
/// 1. a 1.ª chamada devolve **nada** e deixa um pedido em voo (uma leitura que devolvesse números
///    logo aqui estaria a esperar pela placa — é o defeito);
/// 2. depois de a placa acabar, a chamada seguinte devolve a leitura **igual ao bit** à do `tap`
///    síncrono sobre o mesmo cozimento (as duas rotas partilham o gather e a leitura);
/// 3. descartar esquece o pedido em voo **e** a última leitura (sem a 2.ª parte, voltar à placa
///    mostraria números velhos);
/// 4. o CONTROLO da 3: sem descartar, a última leitura continua disponível.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_leitura_encomenda_num_quadro_e_recolhe_no_seguinte() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 20.0);
    g.set_param(grid, "cols", 20.0);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 1.3);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, osc);
    connect(&mut g, osc, out);
    g.validate(&reg).expect("bem tipado");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(
        plan.is_fully_gpu(),
        "a fixture tem de cozer inteira na placa"
    );

    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.cook(
        &gpu,
        &g,
        &reg,
        &reg,
        &plan,
        &[],
        CookClock::at(0.5),
        [0.0, 0.0, 1.0, 1.0],
        [0.1, 0.1],
        SinkStyle::PLAIN,
    )
    .expect("cozimento na placa");
    let s = ph2d_gpu_cook::tap::TAP_SAMPLES;

    // 1.
    assert!(
        gc.tap_sem_espera(&gpu, s).is_none(),
        "a primeira chamada nao tem leitura completa — devolver numeros aqui e' esperar pela placa"
    );
    assert!(gc.tap_em_voo(), "e deixou um pedido em voo");

    // 2.
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let recolhida = gc
        .tap_sem_espera(&gpu, s)
        .expect("com a placa acabada, a chamada seguinte recolhe");
    let sincrona = gc
        .tap(&gpu, s)
        .expect("o tap sincrono le o mesmo cozimento");
    assert_eq!(
        recolhida.keys().collect::<Vec<_>>(),
        sincrona.keys().collect::<Vec<_>>(),
        "os mesmos nos"
    );
    for (no, st) in &sincrona {
        assert_eq!(
            recolhida.get(no),
            Some(st),
            "{no:?}: a leitura sem espera le as MESMAS amostras que a sincrona"
        );
    }

    // 4. (o controlo da 3.): sem descartar, a última leitura fica.
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    assert!(
        gc.tap_sem_espera(&gpu, s).is_some(),
        "CONTROLO: sem descartar, ha' sempre a ultima leitura"
    );

    // 3.
    gc.descarta_tap_em_voo();
    assert!(!gc.tap_em_voo(), "descartar esquece o pedido em voo");
    assert!(
        gc.tap_sem_espera(&gpu, s).is_none(),
        "e esquece a ultima leitura — senao voltar a' placa mostraria numeros velhos"
    );
}
