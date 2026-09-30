//! ⭐ **O CENSO DAS ROTAS DAS FORMAS** (doc 121 W4) — toda cena de demo pela MESMA ponte que o quadro
//! chama, e para cada uma que publica formas vivas: foi à placa, ou porque não.
//!
//! Existe para a W4 escolher por MEDIÇÃO o que falta à rota do dispositivo (o traço sob afim não
//! conforme, os glifos, a tinta própria) — a lista das recusas é contada, não suposta.
//!
//! `#[ignore]`: precisa de adapter real.
//!   cargo nextest run -p ph2d-app-motion --run-ignored all --no-capture -E 'test(/censo_das_rotas/)'

use super::{GpuOutcome, cook_gpu};
use crate::motion_state::MotionState;
use crate::motion_state::demo_router::MAX_DEMO_LEVEL;
use ph2d_gpu::GpuContext;
use std::collections::BTreeMap;

const TIQUE: u64 = 3;
const DT: f64 = 1.0 / 60.0;

#[test]
#[ignore = "sonda de medição — precisa de adapter de GPU"]
fn censo_das_rotas_das_cenas_com_formas() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — esta sonda mede a rota do device");
    };
    let mut por_razao: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    let mut cenas = 0usize;
    let mut textos: Vec<u32> = Vec::new();
    for level in 1..=MAX_DEMO_LEVEL {
        let mut m = MotionState::new();
        if crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry)
            .0
            .is_empty()
        {
            continue;
        }
        m.sinks = super::super::remove::output_nodes(&m.doc.graph);
        if m.sinks.is_empty() {
            continue;
        }
        crate::motion_externals::publish_all(&mut m, TIQUE as f64 * DT);
        let tem_texto = m
            .doc
            .graph
            .nodes()
            .iter()
            .any(|n| n.type_name == "source.text");
        if super::forma::handles_publicados(&m.pump.cook).is_empty() {
            if tem_texto {
                por_razao
                    .entry("TEXTO SEM HANDLES".into())
                    .or_default()
                    .push(level);
            }
            continue;
        }
        if tem_texto {
            textos.push(level);
        }
        cenas += 1;
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        let saida = cook_gpu(&mut m, &gpu, TIQUE, DT, &scopes);
        let razao = if matches!(saida, GpuOutcome::Handled) {
            format!(
                "PLACA ({} copias)",
                if m.gpu_cook.formas().is_some() {
                    "com"
                } else {
                    "sem"
                }
            )
        } else {
            m.route_said.unwrap_or("?").to_string()
        };
        por_razao.entry(razao).or_default().push(level);
    }
    eprintln!("\n=== AS {cenas} CENAS COM FORMAS VIVAS, POR ROTA ===");
    for (razao, levels) in &por_razao {
        eprintln!("  {:>3} · {razao}\n        {levels:?}", levels.len());
    }
    eprintln!("  cenas com source.text e formas publicadas: {textos:?}");
    assert!(
        cenas >= 10,
        "só {cenas} cenas com formas — a sonda deixou de medir"
    );
}
