//! **O COMPLEMENTO de um `Compact` na placa** (doc 110 §14.1 (1)): as portas ≠ 0 do
//! `sim.lifetime` (`died` = as linhas, `pulse` = o evento) deixaram de derrubar o nó para a CPU.
//! O lado `Event` é provado ao bit pelo `sim.spawn` que o consome (§14.1 (7)).

use super::gpu_stream_ops::{cpu_frame, parity_over_ticks, registry, try_headless_gpu};
use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, GpuSource, plan, read_instances};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::{RenderInstance, SinkStyle};

/// O passo do quadro — o MESMO `FIXED_DT` da família.
const DT: f64 = 1.0 / 60.0;

fn fio(g: &mut Graph, from: (NodeId, u16), to: (NodeId, u16), delayed: bool) {
    g.connect(Edge { from, to, delayed })
        .expect("fio bem formado");
}

/// `emitter → lifetime`, e os MORTOS (`died`, porta 1) `→ move → output`. Devolve
/// `(grafo, lifetime, move, output)`.
fn os_mortos() -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 900.0);
    g.set_param(em, "life", 60.0); // a janela do emissor fica aberta…
    let lt = g.add_node("sim.lifetime");
    g.set_param(lt, "life", 1.1); // …e quem mata é o ceifeiro, com vidas sorteadas por id
    g.set_param(lt, "variance", 0.8);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 0.31);
    g.set_param(mv, "dy", -0.17);
    let out = g.add_node("motion.output");
    fio(&mut g, (em, 0), (lt, 0), false);
    fio(&mut g, (lt, 1), (mv, 0), false);
    fio(&mut g, (mv, 0), (out, 0), false);
    (g, lt, mv, out)
}

/// ⭐ **A porta declarada fica na placa, e o consumidor lê a PORTA, não o buffer da 0.**
#[test]
fn the_dead_port_is_claimed_by_the_device() {
    let reg = registry();
    let (g, lt, mv, out) = os_mortos();
    g.validate(&reg).expect("well-typed");
    let p = plan(&g, &reg, &reg, out);
    assert!(p.is_fully_gpu(), "fronteiras: {:?}", p.boundaries);
    let st = p
        .stages
        .iter()
        .find(|s| s.node == mv)
        .expect("o move é estágio");
    assert_eq!(
        st.inputs.first(),
        Some(&GpuSource::StagePort(lt, 1)),
        "o `move` lê a porta 1 (os mortos) — um `Stage(lt)` seria o buffer dos VIVOS em silêncio"
    );
}

/// ⛔ **Um fio ATRASADO de porta ≠ 0 recua**: o estado do quadro anterior só guarda a porta 0.
#[test]
fn a_delayed_edge_from_a_complement_port_still_recedes() {
    let reg = registry();
    let (mut g, lt, _, out) = os_mortos();
    let other = g.add_node("motion.output");
    fio(&mut g, (lt, 1), (other, 0), true);
    let p = plan(&g, &reg, &reg, out);
    assert!(
        !p.stages.iter().any(|s| s.node == lt),
        "um `pre` da porta 1 leria o estado da porta 0: o nó tem de recuar"
    );
    // O CONTROLO: sem o `pre`, o MESMO nó é estágio.
    g.disconnect(other, 0).expect("o fio atrasado existia");
    assert!(
        plan(&g, &reg, &reg, out)
            .stages
            .iter()
            .any(|s| s.node == lt)
    );
}

/// ⭐⭐ **Os mortos saem da placa como eram**, ao bit — mesma contagem a cada tique, e o último
/// quadro comparado elemento a elemento.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_dead_leave_on_the_device_as_they_were() {
    let reg = registry();
    let (g, _, _, out) = os_mortos();
    g.validate(&reg).expect("well-typed");
    // A fixtura: a 1,6 s há mortos (um complemento vazio passaria com o passe morto).
    let mut cook = Cook::new();
    let mut mortos = 0;
    for t in 0..=96u64 {
        mortos = cpu_frame(&mut cook, &g, &reg, out, t as f64 * DT).len();
    }
    assert!(mortos > 0, "fixtura: ninguém morreu em 1,6 s");
    parity_over_ticks("died→move", &g, &reg, out, 96, 1e-5);
    // ⭐ E AO BIT: o complemento é um *gather* e o `move` uma soma — nada de ε.
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (cpu, gpu) = ultimo_quadro_dos_dois_lados(&gpu, &g, &reg, out, 96);
    assert_eq!(cpu.len(), mortos);
    assert!(
        bytemuck::cast_slice::<_, u8>(&cpu) == bytemuck::cast_slice::<_, u8>(&gpu),
        "os mortos têm de sair da placa byte a byte como da CPU"
    );
}

/// O último quadro dos DOIS lados, marchando os mesmos `ticks`.
pub(crate) fn ultimo_quadro_dos_dois_lados(
    gpu: &GpuContext,
    g: &Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    out: NodeId,
    ticks: u64,
) -> (Vec<RenderInstance>, Vec<RenderInstance>) {
    let p = plan(g, reg, reg, out);
    let mut cook = Cook::new();
    let mut gc = GpuCook::new();
    let mut cpu = Vec::new();
    for t in 0..=ticks {
        let playhead = t as f64 * DT;
        cpu = cpu_frame(&mut cook, g, reg, out, playhead);
        gc.cook(
            gpu,
            g,
            reg,
            reg,
            &p,
            &[],
            CookClock {
                playhead,
                tick: p.drives_a_loop().then_some(t),
            },
            [0.25, 0.25, 0.75, 0.75],
            [0.4, 0.4],
            SinkStyle::PLAIN,
        )
        .expect("gpu cook");
    }
    let gpu_out = read_instances(gpu, gc.instances().expect("cooked"));
    (cpu, gpu_out)
}
