//! A MEDIÇÃO da rota de uma cena — a porta do quadro do produto (`o_quadro`, com a placa). Ver o
//! cabeçalho de `motion_state_demo_rota_anunciada_tests.rs`.

use crate::motion_state::MotionState;

pub(super) const QUADROS: u32 = 6;

/// A cena de que uma frase fala.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Cena {
    /// `PH2D_GPU_COOK_DEMO=<n>`.
    Demo(String),
    /// `PH2D_SHAPE_SMOKE=<n>`.
    Forma(u32),
}

// ───────────────────────────── A MEDIÇÃO ─────────────────────────────

/// A rota que o terminal diz, em três classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Rota {
    Inteira,
    Hibrida,
    Processador,
}

/// O que a cena faz, medido.
#[derive(Debug, Clone)]
pub(super) struct Medida {
    /// As linhas `[motion-route] …` pela ordem em que o terminal as imprimiria.
    pub(super) razoes: Vec<&'static str>,
    pub(super) rota: Rota,
    /// Os títulos das fronteiras do plano (o último cartão do processador de cada ramo).
    pub(super) fronteiras: Vec<String>,
    /// Os cartões que correm no processador — o cone das fronteiras, ou a cena toda.
    pub(super) no_processador: Vec<String>,
    pub(super) titulos: Vec<String>,
    /// O passe de formas desenhou na placa? (`None` = a cena não tem formas).
    pub(super) formas: Option<bool>,
}

pub(super) fn rota_de(razao: &str) -> Rota {
    if razao.starts_with("device: o plano inteiro") {
        Rota::Inteira
    } else if razao.starts_with("device: HIBRIDO") {
        Rota::Hibrida
    } else {
        Rota::Processador
    }
}

/// A cena `PH2D_GPU_COOK_DEMO=<n>` montada como o produto a monta (`None` = a cena não existe).
pub(super) fn monta_demo(n: &str) -> Option<MotionState> {
    let mut m = MotionState::new();
    m.cpu_pedida = crate::motion_state::demo_router::cena_pede_a_cpu_em(Some(n));
    (!crate::motion_demo_legend::monta(n, &mut m.doc, &m.registry)
        .0
        .is_empty())
    .then_some(m)
}

/// Coze `QUADROS` quadros da cena pela porta do produto e lê o que o terminal diria.
pub(super) fn mede(
    gpu: &ph2d_gpu::GpuContext,
    cena: &Cena,
    ajusta: impl FnOnce(&mut MotionState),
) -> Option<Medida> {
    let mut m = match cena {
        Cena::Demo(n) => monta_demo(n)?,
        Cena::Forma(n) => {
            let mut m = MotionState::new();
            crate::motion_shape_smoke::grafo_do_smoke(&mut m.doc.graph, *n);
            m
        }
    };
    ajusta(&mut m);
    let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
    playhead.play();
    let mut razoes: Vec<&'static str> = Vec::new();
    for _ in 0..QUADROS {
        playhead.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            &mut m,
            Some(gpu),
            &playhead,
            1.0 / 60.0,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
        if let Some(r) = m.route_said
            && razoes.last() != Some(&r)
        {
            razoes.push(r);
        }
    }
    let rota = rota_de(razoes.last()?);
    let plano = crate::motion_bridge::gpu::plano_do_produto(&mut m, playhead.time());
    let snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    let titulo = |k: ph2d_nodegraph::graph::NodeId| {
        snap.nodes
            .iter()
            .find(|v| v.id == k.0)
            .map(|v| v.display_name.clone())
    };
    let titulos: Vec<String> = snap.nodes.iter().map(|v| v.display_name.clone()).collect();
    let fronteiras: Vec<String> = plano
        .boundaries
        .iter()
        .filter_map(|(k, _)| titulo(*k))
        .collect();
    let mut no_processador: Vec<String> = if rota == Rota::Processador {
        titulos.clone()
    } else {
        plano
            .boundaries
            .iter()
            .flat_map(|(k, _)| ph2d_nodegraph::cook::upstream_cone(&m.doc.graph, *k))
            .filter_map(titulo)
            .collect()
    };
    no_processador.sort();
    no_processador.dedup();
    let ha_formas =
        !m.pump.vector_instances.is_empty() || m.gpu_cook.formas().map_or(0, |f| f.len()) > 0;
    let mut placa = crate::motion_shape_placa::PlacaDeFormas::default();
    let formas = ha_formas.then(|| {
        crate::motion_shape_placa::decide_o_quadro(
            &mut placa,
            gpu,
            &mut m,
            (true, true),
            crate::motion_shape_placa::gpu_tests::camara(),
        )
    });
    Some(Medida {
        razoes,
        rota,
        fronteiras,
        no_processador,
        titulos,
        formas,
    })
}

/// Um nome escrito no texto casa com o título de um cartão — o título, ou o título com o rótulo
/// que a cena lhe deu (`Shape: the shape`).
pub(super) fn casa(titulo: &str, nome: &str) -> bool {
    titulo == nome
        || titulo
            .strip_prefix(nome)
            .is_some_and(|r| r.starts_with(": "))
}

pub(super) fn tem(lista: &[String], nome: &str) -> bool {
    lista.iter().any(|t| casa(t, nome))
}
