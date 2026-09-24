//! ⭐⭐⭐ **PARIDADE GPU↔CPU do `motion.duplicator`** — a W1(b) do ciclo 10, reaberta pelo ciclo 12
//! ([doc 120 §8](../../../../docs/Motion%20Nodes/120_ciclo_12_os_tectos_confortaveis.md)).
//!
//! O carimbo é o `SourceRows` com **duas** portas lidas na fonte: a forma em `i / np` (e o gather de
//! toda a aparência dela) e o ponto em `i % np`. As três metades da lei da CPU são três perguntas
//! que uma comparação de POSIÇÕES sozinha não faz:
//!
//! 1. **A APARÊNCIA vem da forma CERTA.** Cada forma é pintada de uma cor diferente (uma rampa sobre
//!    o `Index` das formas, ANTES do carimbo) — logo a tinta da cópia `i` só está certa se o gather
//!    colheu a forma `i / np` e não a `i % np` nem a `0`. *Com uma forma só, ou com formas iguais,
//!    esta metade seria cega.*
//! 2. **A POSIÇÃO é a SOMA** forma + ponto — as formas estão em sítios diferentes, e os pontos
//!    também.
//! 3. **A RENUMERAÇÃO** — a segunda cadeia projecta `Index / Count` DEPOIS do carimbo num campo e
//!    pinta-o, como a bancada do `motion.clone`: sem a renumeração, ou com a `Count` errada, a tinta
//!    muda.
//!
//! E um caso passa o **orçamento** (`3` formas × `22 500` pontos > `32 768`), onde o `np` que o corpo
//! divide é o CORTADO: um `np` cru desenharia outro número de coisas.
//!
//! `#[ignore]`: precisa de adaptador. Na pista da GPU:
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-gpu-cook --test it \
//!        gpu_cpu_parity_duplicator --release -- --ignored --nocapture`

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::{RenderInstance, SinkStyle};

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
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_duplicator::register(&mut reg).unwrap();
    ph2d_node_motion_rotate::register(&mut reg).unwrap();
    ph2d_node_value_attribute::register(&mut reg).unwrap();
    ph2d_node_value_math::register(&mut reg).unwrap();
    ph2d_node_motion_color_ramp::register(&mut reg).unwrap();
    reg
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.41;

/// Barra de POSIÇÃO — a do `motion.clone`, pela mesma razão: a lei é UMA soma de duas posições de
/// poucas unidades, nada a amplifica, e o que sobra é o ULP.
const EPS_POS: f32 = 5e-6;
/// Barra de TINTA — a da quantização da LUT da rampa (`1,5/255`), MEDIDA na bancada irmã com o
/// controlo em passagem; a avaria que este gate apanha (a forma errada · a renumeração perdida)
/// move a tinta por um passo inteiro da rampa, ordens de grandeza acima.
const EPS_TINT: f32 = 1e-2;
/// Barra da BASE (o `sin`/`cos` da rotação) — **a da casa** (`gpu_cpu_parity.rs`,
/// `gpu_cpu_parity_sim.rs`): a base é trigonometria da MESMA `rot`, calculada no desenho de cada
/// rota, e não é o carimbo que a calcula. Medido aqui: **um ULP** (`0,92050487` contra `0,9205048`,
/// `6e-8`). ⛔ A 1.ª redacção comparava AO BIT e reprovou por esse ULP — que é do `cos` da placa, a
/// jusante do kernel.
const EPS_BASIS: f32 = 1e-4;

fn liga(g: &mut Graph, from: NodeId, to: NodeId, port: u16) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed: false,
    })
    .expect("aresta bem formada");
}

/// `fonte → (Index / Count) → rampa`: pinta cada elemento pela sua posição na lista. Devolve a rampa.
fn pinta_pelo_indice(g: &mut Graph, fonte: NodeId) -> NodeId {
    let idx = g.add_node("value.attribute");
    g.set_text_param(idx, "attr", "Index");
    let cnt = g.add_node("value.attribute");
    g.set_text_param(cnt, "attr", "Count");
    let div = g.add_node("value.math");
    g.set_param(div, "op", 3.0); // Divide
    let rampa = g.add_node("motion.color_ramp");
    liga(g, fonte, idx, 0);
    liga(g, fonte, cnt, 0);
    liga(g, idx, div, 0);
    liga(g, cnt, div, 1);
    liga(g, fonte, rampa, 0);
    liga(g, div, rampa, 1);
    rampa
}

/// Uma grelha `linhas × colunas` com os passos dados.
fn grelha(g: &mut Graph, linhas: f32, colunas: f32, gx: f32, gy: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", linhas);
    g.set_param(n, "cols", colunas);
    g.set_param(n, "gap_x", gx);
    g.set_param(n, "gap_y", gy);
    n
}

/// Que metade da lei a cadeia expõe na tinta.
#[derive(Clone, Copy, Debug)]
enum Olhar {
    /// As formas pintadas ANTES do carimbo: a tinta da saída é a da forma de que ela nasceu.
    AFormaCerta,
    /// A saída pintada DEPOIS do carimbo por `Index / Count`: a tinta é a renumeração.
    ARenumeracao,
}

/// `formas (grelha ns×1, opcionalmente rodadas) → carimbo ← pontos (grelha) → … → saída`.
fn cadeia(
    reg: &NodeRegistry,
    ns: f32,
    lado_dos_pontos: f32,
    olhar: Olhar,
    formas_rodadas: bool,
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let formas = grelha(&mut g, 1.0, ns, 1.7, 0.0);
    let mut lado_forma = formas;
    if formas_rodadas {
        let rot = g.add_node("motion.rotate");
        g.set_param(rot, "angle", 23.0);
        liga(&mut g, lado_forma, rot, 0);
        lado_forma = rot;
    }
    if matches!(olhar, Olhar::AFormaCerta) {
        lado_forma = pinta_pelo_indice(&mut g, lado_forma);
    }
    let pontos = grelha(&mut g, lado_dos_pontos, lado_dos_pontos, 0.13, 0.07);
    let dup = g.add_node("motion.duplicator");
    liga(&mut g, lado_forma, dup, 0);
    liga(&mut g, pontos, dup, 1);
    let ultimo = match olhar {
        Olhar::AFormaCerta => dup,
        Olhar::ARenumeracao => pinta_pelo_indice(&mut g, dup),
    };
    let out = g.add_node("motion.output");
    liga(&mut g, ultimo, out, 0);
    g.validate(reg).expect("bem tipada");
    (g, out)
}

fn cozer_cpu(reg: &NodeRegistry, g: &Graph, out: NodeId) -> Vec<RenderInstance> {
    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        g,
        reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
    )
    .expect("cook da CPU");
    cpu
}

/// Coze no dispositivo. ⛔ **Estoura se o plano não reclamar a cadeia INTEIRA** — uma fronteira de
/// CPU faria este gate comparar a CPU consigo própria.
fn cozer_gpu(gpu: &GpuContext, reg: &NodeRegistry, g: &Graph, out: NodeId) -> Vec<RenderInstance> {
    let plan = ph2d_gpu_cook::plan(g, reg, reg, out);
    assert!(
        plan.is_fully_gpu(),
        "a cadeia do carimbo tem de ser reclamada INTEIRA pelo dispositivo"
    );
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.cook(
        gpu,
        g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .expect("cook do dispositivo");
    ph2d_gpu_cook::read_instances(gpu, gc.instances().expect("cozido"))
}

/// ⭐ **A fixtura CONTÉM o fenómeno** — afirmado no lado canónico antes de se comparar nada.
fn a_fixtura_ve(rot: &str, olhar: Olhar, cpu: &[RenderInstance], ns: usize, np: usize) {
    assert_eq!(cpu.len(), ns * np, "{rot}: a saida e' `ns · np`");
    match olhar {
        // O mesmo PONTO carimbado por duas formas vizinhas tem tintas DIFERENTES: é o gather da
        // forma `i / np`. Um gather na forma `0` (ou na `i % np`) dá-lhes a mesma.
        Olhar::AFormaCerta => {
            for s in 0..ns - 1 {
                assert_ne!(
                    cpu[s * np + 5].tint,
                    cpu[(s + 1) * np + 5].tint,
                    "{rot}: as formas {s} e {} teriam a MESMA cor",
                    s + 1
                );
            }
        }
        // Dois elementos vizinhos pela renumeração têm tintas diferentes, e a rampa varre tudo.
        Olhar::ARenumeracao => {
            assert_ne!(cpu[0].tint, cpu[cpu.len() - 1].tint, "{rot}: a rampa varre");
            assert_ne!(
                cpu[5].tint,
                cpu[np + 5].tint,
                "{rot}: sem renumeracao a forma 0 e a 1 no mesmo ponto pintariam igual"
            );
        }
    }
}

fn comparar(rot: &str, cpu: &[RenderInstance], dev: &[RenderInstance]) {
    assert_eq!(cpu.len(), dev.len(), "{rot}: contagem de instancias");
    let (mut pior_p, mut pior_t) = (0.0f32, 0.0f32);
    for (i, (c, d)) in cpu.iter().zip(dev).enumerate() {
        for k in 0..2 {
            let e = (c.world_pos[k] - d.world_pos[k]).abs();
            pior_p = pior_p.max(e);
            assert!(
                e <= EPS_POS,
                "{rot}: instancia {i} world_pos[{k}]: cpu {} vs gpu {} (|dif| {e:e})",
                c.world_pos[k],
                d.world_pos[k]
            );
        }
        for k in 0..4 {
            let e = (c.tint[k] - d.tint[k]).abs();
            pior_t = pior_t.max(e);
            assert!(
                e <= EPS_TINT,
                "{rot}: instancia {i} tint[{k}]: cpu {} vs gpu {} (|dif| {e:e})",
                c.tint[k],
                d.tint[k]
            );
        }
        // A rotação da forma viaja (ou não existe) nas duas rotas igual.
        for k in 0..4 {
            let e = (c.basis[k] - d.basis[k]).abs();
            assert!(
                e <= EPS_BASIS,
                "{rot}: instancia {i} basis[{k}] (a ROTACAO da forma): cpu {} vs gpu {} (|dif| {e:e})",
                c.basis[k],
                d.basis[k]
            );
        }
    }
    eprintln!(
        "{rot}: {} instancias, pior |Δpos| = {pior_p:e}, pior |Δtint| = {pior_t:e}",
        cpu.len()
    );
}

/// ⭐⭐⭐ **O carimbo, no dispositivo, concorda com a CPU — a forma certa, a soma e a renumeração.**
#[test]
#[ignore = "requires a GPU adapter"]
fn o_carimbo_concorda_com_a_cpu_dentro_do_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador de GPU — a saltar");
        return;
    };
    let reg = registry();
    // (formas, lado dos pontos, o que olhar, formas rodadas)
    for (ns, lado, olhar, rodadas) in [
        (3.0f32, 24.0f32, Olhar::AFormaCerta, false),
        (3.0, 24.0, Olhar::ARenumeracao, false),
        (4.0, 17.0, Olhar::AFormaCerta, true),
        // O ORÇAMENTO: 3 × 22 500 > 32 768 ⇒ o `np` do corpo é o cortado.
        (3.0, 150.0, Olhar::ARenumeracao, false),
    ] {
        let (g, out) = cadeia(&reg, ns, lado, olhar, rodadas);
        let cpu = cozer_cpu(&reg, &g, out);
        let rot = format!("carimbo ns {ns} pontos {lado}² {olhar:?} rodadas {rodadas}");
        let ns_u = ns as usize;
        let np = cpu.len() / ns_u;
        a_fixtura_ve(&rot, olhar, &cpu, ns_u, np);
        if rodadas {
            // O CONTROLO da metade da rotação: a fixtura roda de facto (sem isto a base seria a
            // identidade nas duas rotas e a comparação dela não afirmaria nada).
            assert!(
                cpu[0].basis[1].abs() > 0.1,
                "{rot}: a fixtura tem de rodar as formas"
            );
        }
        let dev = cozer_gpu(&gpu, &reg, &g, out);
        comparar(&rot, &cpu, &dev);
    }
}

/// ⭐⭐ **AS RECUSAS entregam o nó à CPU** — as três do `applicable` e a da ROTAÇÃO dos pontos (uma
/// recusa de PLANO, pela coluna). O CONTROLO é a mesma cadeia nos valores de fábrica.
///
/// ⚠️ **Sem placa de propósito:** é uma pergunta de PLANO, e um gate `#[ignore]` nunca correria no
/// CI. *A recusa que ninguém mede é a que se lê como caminho rápido.*
#[test]
fn as_recusas_do_carimbo_entregam_o_no_a_cpu() {
    let reg = registry();
    let (g, out) = cadeia(&reg, 3.0, 8.0, Olhar::AFormaCerta, false);
    assert!(
        ph2d_gpu_cook::plan(&g, &reg, &reg, out).is_fully_gpu(),
        "CONTROLO: nos valores de fabrica a cadeia inteira e' do dispositivo"
    );
    let dup_de = |g: &Graph| {
        g.nodes()
            .iter()
            .find(|n| n.type_name == "motion.duplicator")
            .expect("a cadeia tem o carimbo")
            .id
    };
    for (nome, param, valor) in [
        ("o modo Cycle", "pick", 1.0f32),
        ("o modo Random", "pick", 2.0),
        ("a transferencia", "transfer", 1.0),
        ("a escala do ponto", "point_scale", 1.0),
    ] {
        let (mut g, out) = cadeia(&reg, 3.0, 8.0, Olhar::AFormaCerta, false);
        let dup = dup_de(&g);
        g.set_param(dup, param, valor);
        assert!(
            !ph2d_gpu_cook::plan(&g, &reg, &reg, out).is_fully_gpu(),
            "{nome}: o plano tem de RECUAR para a CPU"
        );
    }
    // Os PONTOS com rotação própria: a CPU somaria uma coluna que a forma não tem.
    let mut g = Graph::new();
    let formas = grelha(&mut g, 1.0, 3.0, 1.7, 0.0);
    let pontos = grelha(&mut g, 8.0, 8.0, 0.13, 0.07);
    let rot = g.add_node("motion.rotate");
    g.set_param(rot, "angle", 11.0);
    liga(&mut g, pontos, rot, 0);
    let dup = g.add_node("motion.duplicator");
    liga(&mut g, formas, dup, 0);
    liga(&mut g, rot, dup, 1);
    let out = g.add_node("motion.output");
    liga(&mut g, dup, out, 0);
    g.validate(&reg).expect("bem tipada");
    assert!(
        !ph2d_gpu_cook::plan(&g, &reg, &reg, out).is_fully_gpu(),
        "pontos com rot: o plano tem de RECUAR para a CPU"
    );
}
