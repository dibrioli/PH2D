//! Os portões da cena `=13` — as DUAS perguntas que um smoke de mistura tem de responder, e das
//! quais o censo de rota só fazia uma.

use super::build;
use crate::motion_bridge::gpu::{GpuRoute, gpu_route};
use crate::motion_state::MotionState;

/// ⭐⭐⭐ **A CENA VAI À PLACA** — sem isto ela mostra a CPU, onde a mistura sempre funcionou, e
/// o smoke aprovaria uma cura que não chegou a ser vista.
///
/// ⚠️ **É a rota HÍBRIDA e não a inteira, e o gate exige-a pelo nome:** o carimbo pára no
/// duplicador (medido na `=1`: *«fronteira sem estagio de GPU que despache»*), e é o
/// `motion.move` depois dele que leva o desenho à placa. Se um dia o duplicador passar a ter
/// kernel, este gate reprova — e a cena continua certa, só com a frase do roteiro a rever.
///
/// **Mutação que deve sangrar:** ligar o duplicador directo à saída (tirar o `move`) — a rota
/// cai para a CPU, que é a `=1`, e a cena deixa de mostrar a cura.
#[test]
fn a_cena_da_mistura_vai_a_placa() {
    let mut m = MotionState::new();
    let out = build(&mut m.doc.graph, "Object");
    let plan = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, out);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let rota = gpu_route(
        true,
        1,
        scopes.is_empty(),
        &plan.boundaries,
        plan.dispatching_stages(&m.registry),
    );
    assert!(
        matches!(rota, GpuRoute::Hybrid),
        "a =13 tem de ir a' placa pela rota HIBRIDA (o carimbo na CPU, o desenho no device) -- \
         noutra rota ela mostra a CPU, onde a mistura sempre funcionou"
    );
    // ⚠️⚠️ **E a cerca do SINK, que o `gpu_route` não pergunta** (doc 118 W4): um sink que mistura
    // em grupo recusa a placa ANTES do plano. O `gpu_route` acima é puro e dizia HÍBRIDA para uma
    // `=13` em `Add` — que no app ia à CPU. *Um gate que pergunta a metade da decisão aprova a
    // cena pela metade.*
    assert!(
        !crate::motion_bridge::gpu::sink_mistura_em_grupo(&m.doc.graph, out),
        "a =13 abre num modo que mistura EM GRUPO -- esse sink recusa a placa e a cena mostra a \
         CPU; o modo dela tem de ser o `Subtract`"
    );
}

/// ⭐⭐ **A CENA NASCE NUMA MISTURA QUE SE VÊ** — o `Normal` é a omissão, e numa cena que abre
/// em `Normal` o passo *«troque para Normal»* não muda nada.
#[test]
fn a_cena_nasce_em_subtract() {
    let mut m = MotionState::new();
    let out = build(&mut m.doc.graph, "Object");
    assert_eq!(
        ph2d_eval_motion::sink_blend_tag(&m.doc.graph, out),
        2,
        "a saida da =13 nasce em Subtract (o tag 2): o roteiro manda troca-la para Normal"
    );
    // O CONTROLO da régua: a omissão do sink é `Normal`, senão o gate acima não afirma nada.
    let mut vazio = MotionState::new();
    let so = vazio.doc.graph.add_node("motion.output");
    assert_eq!(ph2d_eval_motion::sink_blend_tag(&vazio.doc.graph, so), 0);
}
