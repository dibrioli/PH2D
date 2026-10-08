//! ⭐⭐⭐ **A leitura dos cartões sem esperar pela placa** (ciclo 12, doc 120 §8.6) — ver o
//! cabeçalho de `ph2d_gpu_cook::tap_voo`.
//!
//! A régua não é um relógio (um relógio aqui seria mais um membro da família de flakes de carga):
//! é o que a leitura DEVOLVE em cada chamada, e que ela lê **as mesmas amostras** que o `tap`
//! síncrono.

use super::placa::try_headless_gpu;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

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

/// ⛔ **Sem nada a amostrar, a leitura antiga é ESQUECIDA** (auditoria do fecho, 2026-09-24): um
/// emissor rebobinado ao zero esvazia todo stream, o `tap` síncrono devolve nada, e a leitura sem
/// espera devolvia a de antes para sempre — um cartão com os números de antes do rebobinar.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn sem_nada_a_amostrar_a_leitura_antiga_e_esquecida() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 12.0);
    g.set_param(grid, "cols", 12.0);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, out);
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let coze = |gc: &mut ph2d_gpu_cook::GpuCook, g: &Graph| {
        let plan = ph2d_gpu_cook::plan(g, &reg, &reg, out);
        assert!(plan.is_fully_gpu());
        gc.cook(
            &gpu,
            g,
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
    };
    let s = ph2d_gpu_cook::tap::TAP_SAMPLES;
    coze(&mut gc, &g);
    let _ = gc.tap_sem_espera(&gpu, s);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    assert!(
        gc.tap_sem_espera(&gpu, s).is_some(),
        "CONTROLO: com a grelha cheia ha' leitura"
    );
    assert!(gc.tap_fresca(), "e ela e' nova");

    g.set_param(grid, "rows", 0.0);
    coze(&mut gc, &g);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    assert!(
        gc.tap_sem_espera(&gpu, s).is_none(),
        "todo stream vazio: a leitura de antes NAO pode ficar nos cartoes"
    );

    // ⚠️⚠️ **A metade que a 1.ª redacção não tinha, e uma mutação SOBREVIVENTE nomeou-a:** a
    // chamada sem nada a amostrar devolve vazio com a cura E sem ela — a leitura velha só volta a
    // aparecer na chamada SEGUINTE, quando há outra vez o que ler e a nova ainda está em voo. É ali
    // que o cartão mostraria os números de antes do rebobinar.
    g.set_param(grid, "rows", 12.0);
    coze(&mut gc, &g);
    // ⚠️ Desde o doc 122 §5 a leitura vai no submit do COZIMENTO: quando esta chamada chega, ela
    // pode já estar pronta. O que não pode é ser a de ANTES do rebobinar — que foi esquecida no
    // vazio, logo qualquer leitura aqui tem de ter sido RECOLHIDA agora.
    let depois = gc.tap_sem_espera(&gpu, s);
    assert!(
        depois.is_none() || gc.tap_fresca(),
        "a 1.a leitura depois do vazio ou ainda esta' em voo ou e' NOVA — devolver uma velha e' \
         devolver a de ANTES do rebobinar"
    );
}

/// ⭐ **A leitura vai no SUBMIT DO COZIMENTO** (doc 122 §5): depois do arranque, um cozimento com
/// pedido deixa a leitura em voo SEM nova chamada — foi ele que a codificou — e o que ela recolhe
/// é, ao bit, o que o `tap` síncrono lê desse cozimento.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_cozimento_leva_a_leitura_no_seu_submit() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 16.0);
    g.set_param(grid, "cols", 16.0);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 0.9);
    let out = g.add_node("motion.output");
    connect(&mut g, grid, osc);
    connect(&mut g, osc, out);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu());
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    let coze = |gc: &mut ph2d_gpu_cook::GpuCook, t: f64| {
        gc.cook(
            &gpu,
            &g,
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
    let s = ph2d_gpu_cook::tap::TAP_SAMPLES;
    // O arranque: o 1.º cozimento não tinha pedido.
    coze(&mut gc, 0.1);
    assert!(
        !gc.tap_em_voo(),
        "CONTROLO: sem pedido o cozimento nao le nada"
    );
    let _ = gc.tap_sem_espera(&gpu, s);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    assert!(gc.tap_sem_espera(&gpu, s).is_some(), "o arranque recolhe");
    assert!(!gc.tap_em_voo(), "e nao encomenda outro por conta propria");
    // O regime: o cozimento codifica a leitura.
    coze(&mut gc, 0.7);
    assert!(
        gc.tap_em_voo(),
        "com pedido, o COZIMENTO deixou a leitura em voo — ela foi no submit dele"
    );
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let recolhida = gc.tap_sem_espera(&gpu, s).expect("recolhe");
    assert!(gc.tap_fresca(), "e e' nova");
    let sincrona = gc
        .tap(&gpu, s)
        .expect("o tap sincrono le o mesmo cozimento");
    assert_eq!(
        recolhida, sincrona,
        "as MESMAS amostras deste cozimento, ao bit"
    );
}
