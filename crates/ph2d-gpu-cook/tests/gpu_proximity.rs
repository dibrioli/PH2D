//! **A vizinhança no device** — o `motion.proximity` sobre a grade espacial (ADR-0140).
//!
//! ⚠️ **Aqui a paridade é EXACTA, e isso não é afinação: é uma propriedade das operações.**
//! O irmão `motion.collide` só pode prometer ε porque **SOMA** correcções, e a soma de floats
//! não é associativa — a CPU soma em ordem de índice e o device em ordem de travessia da
//! grade (ADR-0140 D4). Este nó faz `max` (bit-exacto em qualquer ordem) e uma CONTAGEM
//! (inteira), então a ordem de visita simplesmente não é observável, e o gate afirma
//! **igualdade**. Um `eps` aqui esconderia exactamente o defeito que ele existe para pegar:
//! uma célula perdida pelo alcance da grade muda a contagem em UM, que nenhuma tolerância
//! razoável distingue de ruído.
//!
//! ⚠️ E o gate lê as **COLUNAS**, não as instâncias: o produto deste nó não é uma pose, é um
//! número que só existe no stream. O `read_column` (com `retain_streams_for_debug`) é o
//! leitor que a família `value.*` já construiu — a folha 03 dizia que emitir `neighbours`
//! custava *"uma coluna na CPU **mais um harness de paridade que leia COLUNAS**"*, e medido,
//! metade dessa conta já estava paga.
//!
//! `#[ignore]`: precisa de adapter. Na lane de GPU:
//!   cargo test -p ph2d-gpu-cook --test gpu_proximity --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];

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
    ph2d_node_motion_scale::register(&mut reg).unwrap();
    ph2d_node_motion_combine::register(&mut reg).unwrap();
    ph2d_node_motion_proximity::register(&mut reg).unwrap();
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .unwrap();
}

/// A nuvem. Um `motion.grid` apertado (passo bem abaixo de `2·radius`) faz quase todo disco
/// nascer a tocar vários vizinhos — o caso amontoado, e o único em que um contacto perdido
/// pelo alcance da grade aparece.
///
/// `mixed_sizes` acrescenta uma SEGUNDA grade, de passo diferente e **escalada**, e funde as
/// duas: isso é o que faz a lei `r_i + r_j` ser exercida de facto (com tamanhos uniformes ela
/// colapsa em `2·radius` e o `Max` reduce que limita o alcance fica inerte — um gate só de
/// tamanho uniforme passaria sobre um kernel que ignorasse a coluna `size`).
///
/// ⚠️ **O gradiente vem de um `motion.scale` UNIFORME, e não de um `motion.falloff`, porque a
/// FIXTURE de um gate de bit-igualdade tem de ser bit-exacta ela própria.** A primeira versão
/// usava `falloff → scale` para dar um raio por elemento, e a sonda mediu **96 de 512 lanes da
/// coluna `size` já a divergir na ENTRADA** (pior `2,4e-7`): o falloff avalia uma curva e a sua
/// paridade é ε, então o gate acusava este kernel de um defeito de quem o alimentava. Um
/// `motion.scale` de `amount` constante multiplica a identidade `[1, 1]` por um literal, o que
/// é exacto em IEEE-754 dos dois lados, e a variação entre elementos vem de as duas grades
/// carregarem escalas DIFERENTES.
fn proximity_graph(radius: f32, mixed_sizes: bool) -> (Graph, NodeId, usize) {
    let mut g = Graph::new();
    let lattice = |g: &mut Graph, side: f32, gap: f32, amount: f32| {
        let src = g.add_node("motion.grid");
        g.set_param(src, "rows", side);
        g.set_param(src, "cols", side);
        g.set_param(src, "gap_x", gap);
        g.set_param(src, "gap_y", gap);
        let sc = g.add_node("motion.scale");
        g.set_param(sc, "amount", amount);
        g.connect(Edge {
            from: (src, 0),
            to: (sc, 0),
            delayed: false,
        })
        .unwrap();
        sc
    };

    let head = if mixed_sizes {
        // Duas famílias interpenetradas: discos pequenos numa grade fina, grandes numa grossa.
        let a = lattice(&mut g, 16.0, 0.25, 1.0);
        let b = lattice(&mut g, 11.0, 0.36, 2.0);
        let mix = g.add_node("motion.combine");
        for (k, from) in [a, b].into_iter().enumerate() {
            g.connect(Edge {
                from: (from, 0),
                to: (mix, u16::try_from(k).unwrap()),
                delayed: false,
            })
            .unwrap();
        }
        mix
    } else {
        lattice(&mut g, 16.0, 0.25, 1.0)
    };

    let prox = g.add_node("motion.proximity");
    g.set_param(prox, "radius", radius);
    wire(&mut g, head, prox);
    // Medido no plano, não escolhido: `motion.combine` é `PASSTHROUGH` + `StreamOp`, então o
    // número não é `nós − 1`.
    let stages = if mixed_sizes { 5 } else { 3 };
    (g, prox, stages)
}

fn cpu_columns(g: &Graph, reg: &NodeRegistry, target: NodeId) -> (Vec<f32>, Vec<f32>) {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, target, 0.0).expect("cpu cook");
    let s = out[0].as_stream();
    let get = |name: &str| match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        other => panic!("a CPU tem de emitir `{name}` como escalar: {other:?}"),
    };
    (get("neighbours"), get("overlap"))
}

fn gpu_columns(
    gpu: &GpuContext,
    g: &Graph,
    reg: &NodeRegistry,
    target: NodeId,
    stages: usize,
) -> (Vec<f32>, Vec<f32>) {
    let plan = plan(g, reg, reg, target);
    assert!(plan.is_fully_gpu(), "boundaries: {:?}", plan.boundaries);
    // ⚠️ EXACTA e não `>= 1`: é ela que prova que não houve recuo silencioso para a CPU, que
    // compararia a CPU com ela mesma e passaria sobre um kernel que nunca rodou.
    assert_eq!(plan.dispatching_stages(reg), stages);
    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        gpu,
        g,
        reg,
        reg,
        &plan,
        &[],
        CookClock {
            playhead: 0.0,
            tick: Some(0),
        },
        DEFAULT_UV,
        DEFAULT_SIZE,
        0,
    )
    .expect("gpu cook");
    let get = |name: &str| {
        gc.read_column(gpu, target, name)
            .unwrap_or_else(|| panic!("a coluna `{name}` nao volta do device"))
    };
    (get("neighbours"), get("overlap"))
}

/// Compara as duas colunas: a CONTAGEM por igualdade, a fracção por um orçamento DERIVADO.
///
/// ⚠️ **As duas colunas não têm a mesma natureza, e a primeira versão deste gate afirmava
/// bit-igualdade para as duas.** A medição refutou metade: com uma fixture cuja entrada é
/// bit-exacta dos dois lados, o `overlap` ainda diverge **um ULP** (elemento 70: `0,8269732`
/// contra `0,82697326`). O mecanismo é o `dot(d, d)` do WGSL, que o compilador de shader pode
/// **FUNDIR** num `fma` — mais exacto que os dois arredondamentos que a CPU faz, e por isso
/// diferente. Não há como pedir ao device que seja menos exacto.
///
/// O que sobrevive intacto é a metade que importa: `neighbours` é uma **DECISÃO** (`d2 <
/// min_dist²`), e um ULP só a muda se um par cair exactamente na fronteira — medida zero. A
/// fracção é um **VALOR aritmético**, e carrega o ε de toda a engine.
///
/// O orçamento é derivado, não escolhido: `overlap ∈ [0, 1]`, o `d2` pode diferir 1 ULP, o
/// `sqrt` e a divisão acrescentam ~½ cada ⇒ ~3 ULP de um número de magnitude 1, ou seja
/// `3 × 2⁻²⁴ ≈ 1,8e-7`. A barra é **4e-7**, com o pior medido impresso ao lado.
const OVERLAP_EPS: f32 = 4e-7;

fn parity(label: &str, cpu: (Vec<f32>, Vec<f32>), gpu: (Vec<f32>, Vec<f32>)) {
    let (cpu_n, cpu_o) = cpu;
    let (gpu_n, gpu_o) = gpu;
    assert_eq!(cpu_n.len(), gpu_n.len(), "{label}: contagem de elementos");
    let mut touching = 0usize;
    let mut worst_overlap = 0.0f32;
    let mut worst_delta = 0.0f32;
    for i in 0..cpu_n.len() {
        assert_eq!(
            cpu_n[i], gpu_n[i],
            "{label}: elemento {i} conta {} vizinhos na CPU e {} no device -- uma celula \
             perdida pelo alcance da grade aparece aqui como um contacto a menos",
            cpu_n[i], gpu_n[i]
        );
        let d = (cpu_o[i] - gpu_o[i]).abs();
        assert!(
            d <= OVERLAP_EPS,
            "{label}: elemento {i} sobrepoe {} na CPU e {} no device (|d| = {d:e} > {OVERLAP_EPS:e})",
            cpu_o[i],
            gpu_o[i]
        );
        worst_delta = worst_delta.max(d);
        if cpu_n[i] > 0.0 {
            touching += 1;
        }
        worst_overlap = worst_overlap.max(cpu_o[i]);
    }
    // A fixture TEM de conter o fenómeno: duas nuvens sem contacto nenhum concordam por vácuo.
    assert!(
        touching > cpu_n.len() / 2,
        "{label}: a fixture tinha de nascer amontoada; so' {touching} de {} tocam alguem",
        cpu_n.len()
    );
    eprintln!(
        "proximity {label}: {} discos, {touching} a tocar, pior sobreposicao {worst_overlap:.4}, \
         contagem EXACTA, max |d| da fraccao = {worst_delta:e}",
        cpu_n.len()
    );
}

/// Tamanhos UNIFORMES: isola a consulta da grade — se o alcance derivado alguma vez perdesse
/// um contacto, aparece aqui como uma contagem diferente, não como deriva.
#[test]
#[ignore = "needs a GPU adapter"]
fn the_neighbourhood_matches_the_cpu_exactly() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_proximity");
        return;
    };
    let reg = registry();
    let (g, prox, stages) = proximity_graph(0.3, false);
    let cpu = cpu_columns(&g, &reg, prox);
    let dev = gpu_columns(&gpu, &g, &reg, prox, stages);
    parity("uniform", cpu, dev);
}

/// Tamanhos MISTOS: a lei `r_i + r_j` e o `Max` reduce que limita o alcance. Sem esta metade
/// um kernel que ignorasse a coluna `size` passaria no gate acima.
#[test]
#[ignore = "needs a GPU adapter"]
fn the_per_element_radius_matches_the_cpu_exactly() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_proximity");
        return;
    };
    let reg = registry();
    let (g, prox, stages) = proximity_graph(0.3, true);
    let cpu = cpu_columns(&g, &reg, prox);
    let dev = gpu_columns(&gpu, &g, &reg, prox, stages);
    parity("mixed sizes", cpu, dev);
}
