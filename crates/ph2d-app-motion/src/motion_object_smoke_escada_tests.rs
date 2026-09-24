//! Os portões da cena `=17` (a escada dos tectos) — a rota que ela mede, a regra do dono, e as
//! duas portas de linha de comando.

use super::{N_OMISSAO, build, forma_por, populacao_por};
use crate::motion_bridge::gpu::{GpuRoute, gpu_route};
use crate::motion_state::MotionState;

fn rota(estrela: bool) -> GpuRoute {
    let mut m = MotionState::new();
    let sink = build(&mut m.doc.graph, "Particle", N_OMISSAO, estrela);
    m.doc.graph.validate(&m.registry).expect("bem tipado");
    let plan = ph2d_gpu_cook::plan_driven_many(
        &m.doc.graph,
        &m.registry,
        &m.registry,
        &[sink],
        &ph2d_gpu_cook::DrivenParams::new(),
    );
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    gpu_route(
        true,
        1,
        scopes.is_empty(),
        &plan.boundaries,
        plan.dispatching_stages(&m.registry),
    )
}

/// ⭐⭐ **A ESCADA MEDE A ROTA QUE O ARTISTA TEM: a HÍBRIDA, com as DUAS formas.** A simulação e o
/// carimbo correm na CPU e só o `motion.move` vai à placa (o molde do enxame `=16`) — o
/// `motion.duplicator` é a fronteira do planeador com a imagem **e** com a estrela. ⚠️ A 1.ª
/// redacção deste gate afirmava que a estrela ficava toda na CPU e reprovou: o que a estrela muda
/// é o **DESENHO** (geometria vectorial por instância contra um quad do atlas), não a rota da
/// simulação. Sem este gate a tabela do doc 120 podia estar a medir uma rota que a cena já não
/// toma, e a linha dela continuaria a dizer o mesmo.
#[test]
fn a_escada_mede_a_rota_hibrida_com_as_duas_formas() {
    for estrela in [false, true] {
        let r = rota(estrela);
        assert!(
            matches!(r, GpuRoute::Hybrid),
            "estrela={estrela}: a escada vai a' placa pela rota HIBRIDA; rota {r:?}"
        );
    }
}

/// ⭐ **A regra do dono, afirmada na própria cena:** uma FORMA e SIMULAÇÃO com CAMPOS.
#[test]
fn a_escada_tem_forma_e_simulacao_com_campos() {
    for estrela in [false, true] {
        let mut m = MotionState::new();
        build(&mut m.doc.graph, "Particle", N_OMISSAO, estrela);
        let tipos: Vec<String> = m
            .doc
            .graph
            .nodes()
            .iter()
            .map(|n| n.type_name.clone())
            .collect();
        let conta = |t: &str| tipos.iter().filter(|x| x.as_str() == t).count();
        assert_eq!(
            conta("source.object") + conta("source.shape"),
            1,
            "uma FORMA"
        );
        assert_eq!(conta("motion.integrate"), 1, "uma simulação");
        assert!(
            tipos.iter().filter(|t| t.starts_with("force.")).count() >= 2,
            "a simulação tem campos de força"
        );
    }
}

/// As portas de linha de comando: ausente/lixo/zero ⇒ a omissão; um número ⇒ ele próprio.
#[test]
fn as_portas_da_escada() {
    for v in [None, Some(""), Some("abc"), Some("0"), Some("-5")] {
        assert_eq!(populacao_por(v), N_OMISSAO, "{v:?}");
    }
    assert_eq!(populacao_por(Some(" 262144 ")), 262_144);
    assert!(!forma_por(None) && !forma_por(Some("0")) && forma_por(Some("1")));
}
