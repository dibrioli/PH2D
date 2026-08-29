//! GPU-vs-CPU parity para **a DURAÇÃO DESIGUAL POR QUADRO** — os *holds* do
//! `motion.sub_uv`.
//!
//! ## Por que este gate existe, e o que ele de facto amarra
//!
//! O amostrador da tabela vive **duas vezes**: o gerador escreve
//! `suv_hold_sample(t)` no WGSL, e o crate do nó tem uma cópia em Rust
//! (`holds::sample`) para a CPU ler a MESMA tabela. A cópia é deliberada — um nó
//! não alcança o gerador de código (ADR-0075) —, e é ela que torna a lei um
//! degrau idêntico nos dois lados em vez de *exacto de um lado e tabelado do
//! outro*. **Este gate é a única coisa que impede as duas de divergirem.**
//!
//! ⚠️ E a divergência seria do pior tipo: numa gramática de *holds*, exacto e
//! tabelado diferem por uma **CÉLULA INTEIRA** perto de cada fronteira. Não é um
//! ε maior — é a imagem errada, num instante, num dos motores.
//!
//! ## O que cada caso mede, e o CONTROLE de cada um
//!
//! | caso | contra o controle | por quê |
//! |---|---|---|
//! | `sub_uv` **sem holds** | **IDÊNTICO** ao de sempre | *"nada autorado = o nó que sempre shipou"* — a sentinela |
//! | `sub_uv` **com holds** | difere | senão a tabela estaria a ser ignorada nos dois lados de igual maneira |
//!
//! Um gate que só medisse *"o device concorda com a CPU"* passaria com o ramo dos
//! *holds* **morto nos dois** — a forma de verde que a auditoria deste módulo
//! apanhou vinte e quatro vezes.
//!
//! `#[ignore]`: precisa de adapter real.
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_holds --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
/// A folha das fixtures: uma tira de 4, e `stagger` a dar uma fase distinta por elemento —
/// sem ele todos leriam a MESMA célula e o campo seria constante.
const COLS: f32 = 4.0;
/// Um ritmo com uma pose SEGURA — o caso que o artista escreve.
const HOLDS: &str = "1 1 3 1";

/// ⚠️ **A barra é ZERO, e é a única honesta aqui.** A `uv_cell` é uma de exactamente
/// `cols × rows` quádruplas possíveis, e os dois motores escolhem-na do MESMO degrau da
/// MESMA tabela. Não há erro de reconstrução a orçamentar: ou concordam na célula, ou a
/// tabela foi lida de maneira diferente — e aí a diferença é uma célula inteira, não um ULP.
const EXACT: f32 = 0.0;

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
    ph2d_node_motion_sub_uv::register(&mut reg).unwrap();
    reg
}

fn build(g: &mut Graph, holds: Option<&str>) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let suv = g.add_node("motion.sub_uv");
    g.set_param(suv, "cols", COLS);
    g.set_param(suv, "rows", 1.0);
    g.set_param(suv, "speed", 1.7);
    // Uma fase distinta por elemento: é o que faz a fixtura CONTER o fenómeno.
    g.set_param(suv, "stagger", 0.37);
    if let Some(h) = holds {
        g.set_text_param(suv, ph2d_node_motion_sub_uv::HOLDS_KEY, h);
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (suv, 0),
        delayed: false,
    })
    .unwrap();
    suv
}

/// Coze nos dois motores e devolve `(cpu, gpu)` da coluna `uv_cell`.
fn cook_both(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    holds: Option<&str>,
    at: f64,
) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let mut g = Graph::new();
    let sink = build(&mut g, holds);
    g.validate(reg).expect("o grafo e' valido");
    let plan = ph2d_gpu_cook::plan(&g, reg, reg, sink);
    assert!(
        plan.is_fully_gpu(),
        "a cadeia tem de ser reivindicada de ponta a ponta — um ritmo autorado NAO derruba \
         o flipbook para a CPU (o `applicable: false` esta' proibido por lei do modulo)"
    );

    let mut cook = Cook::new();
    let cpu = cook.cook(&g, reg, sink, at).expect("cpu cook");
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        gpu,
        &g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(at),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .expect("gpu cook");

    let cpu_cell = match cpu[0].as_stream().get(ph2d_node_motion_sub_uv::CELL_COLUMN) {
        Some(Column::Vec4(v)) => v.clone(),
        _ => panic!("a CPU tem de escrever a uv_cell"),
    };
    let gpu_cell = gc
        .read_column_vec4(gpu, sink, ph2d_node_motion_sub_uv::CELL_COLUMN)
        .expect("a uv_cell volta do device");
    (cpu_cell, gpu_cell)
}

fn worst(a: &[[f32; 4]], b: &[[f32; 4]]) -> f32 {
    assert_eq!(a.len(), b.len(), "as duas contagens tem de bater");
    a.iter()
        .zip(b)
        .flat_map(|(p, q)| p.iter().zip(q).map(|(x, y)| (x - y).abs()))
        .fold(0.0f32, f32::max)
}

/// **Os dois motores escolhem a MESMA célula, com e sem ritmo autorado** — e as duas
/// leituras têm de DIFERIR uma da outra, senão a tabela está a ser ignorada em ambos.
#[test]
#[ignore = "precisa de adapter real"]
fn the_two_engines_pick_the_same_cell_with_and_without_holds() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — o gate nao correu");
        return;
    };
    let reg = registry();
    for at in [0.0, 0.37, 1.9, 3.25] {
        let (cpu_bare, gpu_bare) = cook_both(&gpu, &reg, None, at);
        assert!(
            worst(&cpu_bare, &gpu_bare) <= EXACT,
            "t = {at}: sem holds os dois motores tem de dar a MESMA celula"
        );
        let (cpu_held, gpu_held) = cook_both(&gpu, &reg, Some(HOLDS), at);
        assert!(
            worst(&cpu_held, &gpu_held) <= EXACT,
            "t = {at}: com holds os dois motores tem de dar a MESMA celula \
             (pior delta {})",
            worst(&cpu_held, &gpu_held)
        );
        // ⚠️ **O CONTROLE**: o ritmo tem de MUDAR alguma coisa. Sem isto, um ramo de holds
        // morto nos DOIS motores passaria as duas afirmações acima.
        assert!(
            worst(&cpu_bare, &cpu_held) > 0.0,
            "t = {at}: o ritmo autorado nao mudou celula nenhuma — o ramo esta' morto"
        );
    }
}

/// ⚠️ **SEM ritmo autorado, o device é BYTE-IDÊNTICO ao que sempre shipou.**
///
/// A sentinela cobre a tabela inteira, e o `suv_hold` devolve o `k` intacto. É a metade da
/// wave que o artista não vê — e a que garante que ligar a feature não moveu nenhum
/// documento já autorado.
#[test]
#[ignore = "precisa de adapter real"]
fn an_unauthored_sheet_is_bit_identical_to_the_node_that_always_shipped() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — o gate nao correu");
        return;
    };
    let reg = registry();
    let (cpu, dev) = cook_both(&gpu, &reg, None, 1.234);
    for (i, (p, q)) in cpu.iter().zip(&dev).enumerate() {
        assert_eq!(
            p.map(f32::to_bits),
            q.map(f32::to_bits),
            "o elemento {i} difere: {p:?} contra {q:?}"
        );
    }
}
