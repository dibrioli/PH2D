//! ⭐⭐⭐ **PARIDADE GPU↔CPU do `motion.clone`** — a bancada da W1(a) do ciclo 10 (doc 116 §5.4).
//!
//! Ele é o **caso puro** da lei de contagem: mede-se sem uma forma no caminho, e a cadeia inteira
//! (gerador → multiplicador → campo → tinta → saída) fica na placa.
//!
//! ## O que esta bancada mede que uma comparação de POSIÇÕES não mediria
//!
//! ⛔⛔ **A RENUMERAÇÃO não aparece na posição.** `Index`/`Count` são colunas que o desenho não
//! carrega, e a cópia `c` põe as peças exactamente onde a translação manda quer o `Index` tenha
//! sido renumerado quer não. ⇒ a cadeia projecta **`Index / Count`** num campo
//! (`value.attribute` × 2 + `value.math` ▸ Divide) e pinta-o com o `motion.color_ramp`: `t` varre
//! `[0, 1)` **uma vez sobre o conjunto multiplicado**, e a TINTA passa a ser a renumeração à vista.
//!
//! ⭐ **O discriminador é o par `(j, j + n)`** — o mesmo elemento da fonte em duas cópias
//! vizinhas. Com a renumeração ele tem tintas DIFERENTES (`Index` salta `n`); sem ela teria a
//! MESMA (cada cópia recomeça em zero), e com a `Count` cunhada a zeros `t` seria `0` em toda a
//! parte. *As três avarias que a cruza e a escrita-sem-leitura existem para impedir são
//! distinguíveis nesta fixtura, e o gate afirma-o no lado da CPU antes de comparar seja o que for.*
//!
//! `#[ignore]`: precisa de adaptador. Na pista da GPU:
//!   `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-gpu-cook --test it \
//!        gpu_cpu_parity_clone --release -- --ignored --nocapture`

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
    ph2d_node_motion_clone::register(&mut reg).unwrap();
    ph2d_node_value_attribute::register(&mut reg).unwrap();
    ph2d_node_value_math::register(&mut reg).unwrap();
    ph2d_node_motion_color_ramp::register(&mut reg).unwrap();
    reg
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.41;
/// O lado da grelha fonte: `SIDE² · count` é o dispatch, e fica leve (`32² × 6 = 6 144`).
const SIDE: f32 = 32.0;
const N: usize = 32 * 32;

/// Barra de POSIÇÃO — **MEDIDA, não emprestada**. A lei é uma soma (`p + posto · passo`) sobre
/// uma grelha de poucas unidades: nada é amplificado, e o que sobra é a contracção que a placa
/// pode fazer no `a + b·c`. Pior observado nos quatro casos: `0`, `0`, `0` e **`4,77e-7`** (um ULP
/// à magnitude `~4`). `5e-6` é ~10× isso.
const EPS_POS: f32 = 5e-6;
/// Barra de TINTA — **MEDIDA, e ela NÃO é do cloner**, que é o que a torna legível:
///
/// | caso | pior \|Δpos\| | pior \|Δtint\| |
/// |---|---|---|
/// | `count 6` · dist 2 · 0° | `0` | `5,8826e-3` |
/// | `count 4` · dist 1,25 · 37° | `4,77e-7` | `5,8824e-3` |
/// | `count 5` · dist 0,9 · −110° · centrado | `0` | `5,8824e-3` |
/// | **`count 1`** (o cloner em PASSAGEM) | `0` | **`5,8824e-3`** |
///
/// ⭐⭐ **A última linha é o CONTROLO:** com uma cópia só o multiplicador não multiplica nada, e a
/// tinta lê **o mesmo** desvio ⇒ *ele é a quantização da LUT da rampa*, não uma deriva do kernel
/// novo. A `LUT_RESOLUTION` dela é `256`, logo a célula mede `1/255 = 3,92e-3` e o canto de uma
/// parada a cair dentro de uma célula vale `1,5/255 = 5,88e-3` — o número medido, ao dígito. A
/// casa já tinha chegado a `6e-3` por outro caminho no `gpu_stream_ops`, e esta medição reproduz o
/// número sem o copiar.
///
/// ⛔ **E a barra continua a discriminar por DUAS ordens de grandeza:** a avaria que este gate
/// existe para apanhar — a renumeração a evaporar-se — desloca `t` em `1/k` (a `k = 6`, `0,167`),
/// que é **`17×`** a barra e **`28×`** o pior desvio medido.
const EPS_TINT: f32 = 1e-2;

fn liga(g: &mut Graph, from: NodeId, to: NodeId, port: u16) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed: false,
    })
    .expect("aresta bem formada");
}

/// `grelha → clone → (Index / Count) → rampa → saída`.
fn cadeia(
    reg: &NodeRegistry,
    count: f32,
    distancia: f32,
    angulo: f32,
    center: bool,
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    g.set_param(grelha, "rows", SIDE);
    g.set_param(grelha, "cols", SIDE);
    g.set_param(grelha, "gap_x", 0.31);
    g.set_param(grelha, "gap_y", 0.19);
    let clone = g.add_node("motion.clone");
    g.set_param(clone, "count", count);
    g.set_param(clone, "distance", distancia);
    g.set_param(clone, "angle", angulo);
    g.set_param(clone, "center", f32::from(center));
    let idx = g.add_node("value.attribute");
    g.set_text_param(idx, "attr", "Index");
    let cnt = g.add_node("value.attribute");
    g.set_text_param(cnt, "attr", "Count");
    let div = g.add_node("value.math");
    g.set_param(div, "op", 3.0); // Divide
    let rampa = g.add_node("motion.color_ramp");
    let out = g.add_node("motion.output");
    liga(&mut g, grelha, clone, 0);
    liga(&mut g, clone, idx, 0);
    liga(&mut g, clone, cnt, 0);
    liga(&mut g, idx, div, 0);
    liga(&mut g, cnt, div, 1);
    liga(&mut g, clone, rampa, 0);
    liga(&mut g, div, rampa, 1);
    liga(&mut g, rampa, out, 0);
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
/// CPU faria este gate comparar a CPU consigo própria, que é a forma de um gate verde sobre uma
/// feature que nunca correu.
fn cozer_gpu(gpu: &GpuContext, reg: &NodeRegistry, g: &Graph, out: NodeId) -> Vec<RenderInstance> {
    let plan = ph2d_gpu_cook::plan(g, reg, reg, out);
    assert!(
        plan.is_fully_gpu(),
        "a cadeia do cloner tem de ser reclamada INTEIRA pelo dispositivo"
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
fn a_fixtura_ve_a_renumeracao(rot: &str, cpu: &[RenderInstance], k: usize) {
    assert_eq!(cpu.len(), N * k, "{rot}: a saida e' `n · k`");
    // O mesmo elemento da FONTE em duas cópias vizinhas: com a renumeração as tintas diferem.
    for c in 0..k - 1 {
        let (a, b) = (cpu[c * N + 7].tint, cpu[(c + 1) * N + 7].tint);
        assert_ne!(
            a,
            b,
            "{rot}: sem renumeracao a copia {c} e a {} pintariam a MESMA cor",
            c + 1
        );
    }
    // …e a rampa varre o conjunto inteiro, e não é uniforme (a `Count` cunhada a zeros daria isso).
    assert_ne!(cpu[0].tint, cpu[cpu.len() - 1].tint, "{rot}: a rampa varre");
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
                "{rot}: instancia {i} tint[{k}] (a RENUMERACAO): cpu {} vs gpu {} (|dif| {e:e})",
                c.tint[k],
                d.tint[k]
            );
        }
    }
    eprintln!(
        "{rot}: {} instancias, pior |Δpos| = {pior_p:e}, pior |Δtint| = {pior_t:e}",
        cpu.len()
    );
}

/// ⭐⭐⭐ **O cloner, no dispositivo, concorda com a fila canónica — posição E renumeração.**
#[test]
#[ignore = "requires a GPU adapter"]
fn o_cloner_concorda_com_a_cpu_dentro_do_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador de GPU — a saltar");
        return;
    };
    let reg = registry();
    // (cópias, distância, ângulo, centrado) — o posto assinado é um BRAÇO da lei, logo as duas
    // metades do `center` correm.
    for (count, distancia, angulo, center) in [
        (6.0f32, 2.0f32, 0.0f32, false),
        (4.0, 1.25, 37.0, false),
        (5.0, 0.9, -110.0, true),
        (1.0, 2.0, 0.0, false),
    ] {
        let (g, out) = cadeia(&reg, count, distancia, angulo, center);
        let cpu = cozer_cpu(&reg, &g, out);
        let rot = format!("clone count {count} dist {distancia} ang {angulo} center {center}");
        let k = count as usize;
        assert_eq!(cpu.len(), N * k, "{rot}: a CPU emite `n · k`");
        if k > 1 {
            a_fixtura_ve_a_renumeracao(&rot, &cpu, k);
        }
        let dev = cozer_gpu(&gpu, &reg, &g, out);
        comparar(&rot, &cpu, &dev);
    }
}

/// ⭐⭐ **AS QUATRO RECUSAS entregam o nó à CPU** — e o CONTROLO é a mesma cadeia sem elas.
///
/// ⚠️ **Sem placa de propósito:** é uma pergunta de PLANO, e um gate `#[ignore]` nunca correria no
/// CI. *A recusa que ninguém mede é a que se lê como caminho rápido.*
#[test]
fn as_recusas_do_cloner_entregam_o_no_a_cpu() {
    let reg = registry();
    let (g, out) = cadeia(&reg, 6.0, 2.0, 0.0, false);
    assert!(
        ph2d_gpu_cook::plan(&g, &reg, &reg, out).is_fully_gpu(),
        "CONTROLO: nos valores de fabrica a cadeia inteira e' do dispositivo"
    );
    for (nome, param, valor) in [
        ("o leque de relogios", "time_offset", 0.5f32),
        ("o modo Radial", "mode", 1.0),
        ("o taper de escala", "scale_taper", 0.5),
        ("o taper de rotacao", "rot_taper", 90.0),
    ] {
        let (mut g, out) = cadeia(&reg, 6.0, 2.0, 0.0, false);
        let clone = g
            .nodes()
            .iter()
            .find(|n| n.type_name == "motion.clone")
            .expect("a cadeia tem o cloner")
            .id;
        g.set_param(clone, param, valor);
        assert!(
            !ph2d_gpu_cook::plan(&g, &reg, &reg, out).is_fully_gpu(),
            "{nome}: o plano tem de RECUAR para a CPU"
        );
    }
}
