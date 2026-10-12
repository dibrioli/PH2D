//! **A DERIVA DA `=129` — lei ou defeito?** (doc 131 §2). Sonda, não gate: mede `max|ΔP|` placa ×
//! CPU por tique, nas duas saídas e no `motion.soft_body` de cada coluna (a ablação do
//! `motion.transform`), e diz onde o corpo foi cozido (fronteira da CPU ou placa).
//!
//!   cargo test -p ph2d-app-motion --lib -- --ignored --nocapture a_deriva_da_129

use super::cook_gpu;
use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;

const DT: f64 = 1.0 / 60.0;
const TIQUES: [u64; 8] = [1, 2, 3, 5, 10, 30, 60, 120];

fn gemeos() -> (MotionState, MotionState, [NodeId; 2]) {
    let monta = || {
        let mut m = MotionState::new();
        crate::motion_demo_legend::monta("129", &mut m.doc, &m.registry);
        m.sinks = super::super::remove::output_nodes(&m.doc.graph);
        // O corpo de cada saída: o `motion.soft_body` no cone dela.
        let corpos: Vec<NodeId> = m
            .sinks
            .iter()
            .map(|&s| {
                ph2d_nodegraph::cook::upstream_cone(&m.doc.graph, s)
                    .into_iter()
                    .find(|n| {
                        m.doc
                            .graph
                            .node(*n)
                            .is_some_and(|i| i.type_name == "motion.soft_body")
                    })
                    .expect("cada saída da =129 tem um corpo")
            })
            .collect();
        (m, [corpos[0], corpos[1]])
    };
    let (placa, corpos) = monta();
    let (mut cpu, _) = monta();
    cpu.gpu_enabled = false;
    (placa, cpu, corpos)
}

fn p_de(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn tomada(m: &MotionState, n: NodeId) -> Vec<[f32; 2]> {
    m.pump
        .tap_streams()
        .iter()
        .find(|(k, _)| *k == n)
        .map(|(_, s)| p_de(s))
        .unwrap_or_default()
}

/// `(max|Δ|, amplitude)` de duas colunas `P`.
fn delta(a: &[[f32; 2]], b: &[[f32; 2]]) -> (f32, f32) {
    let d = a
        .iter()
        .zip(b)
        .map(|(x, y)| (x[0] - y[0]).abs().max((x[1] - y[1]).abs()))
        .fold(0.0f32, f32::max);
    let (lo, hi) = b
        .iter()
        .flat_map(|p| [p[0], p[1]])
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        });
    (d, (hi - lo).max(0.0))
}

#[test]
#[ignore = "sonda: placa (precisa de adapter), não um gate"]
fn a_deriva_da_129_por_tique_e_por_no() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adapter");
    eprintln!("=== A DERIVA DA =129 · max|ΔP| placa × CPU ===");
    for &t in &TIQUES {
        let (mut m, mut cpu, corpos) = gemeos();
        crate::motion_externals::publish_all(&mut m, t as f64 * DT);
        crate::motion_externals::publish_all(&mut cpu, t as f64 * DT);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        m.gpu_cook.retain_streams_for_debug(true);
        let saida = cook_gpu(&mut m, &gpu, t, DT, &scopes);
        // A CPU: as saídas E os corpos como tomadas, marchados até ao mesmo tique.
        let mut taps = cpu.sinks.clone();
        taps.extend(corpos);
        cpu.pump.set_taps(&taps);
        for tick in 0..=t {
            cpu.pump.advance_or_scrub_scoped(
                &cpu.doc.graph,
                &cpu.registry,
                &cpu.sinks,
                tick,
                |k| k as f64 * DT,
                cpu.default_uv_rect,
                cpu.default_size,
                &scopes,
            );
        }
        let fronteiras: Vec<NodeId> = m.pump.boundary_streams().iter().map(|(n, _)| *n).collect();
        let mut linha = format!("T={t:<4} {:?}", m.route_said.unwrap_or("?"));
        for (k, (&sink, &corpo)) in m.sinks.iter().zip(&corpos).enumerate() {
            let sp = m
                .gpu_cook
                .read_column_vec2(&gpu, sink, "P")
                .unwrap_or_default();
            let (ds, amp) = delta(&sp, &tomada(&cpu, sink));
            // O corpo do lado da placa: a fronteira da CPU (rota híbrida) ou a coluna do device.
            let (onde, cp) =
                if let Some((_, s)) = m.pump.boundary_streams().iter().find(|(n, _)| *n == corpo) {
                    ("fronteira", p_de(s))
                } else {
                    (
                        "device",
                        m.gpu_cook
                            .read_column_vec2(&gpu, corpo, "P")
                            .unwrap_or_default(),
                    )
                };
            let (dc, _) = delta(&cp, &tomada(&cpu, corpo));
            linha += &format!(
                "\n    col {k}: saída {ds:.3e} (rel {:.2e}, amp {amp:.3}) · corpo[{onde}] {dc:.3e} · n {}",
                ds / amp.max(f32::MIN_POSITIVE),
                sp.len()
            );
        }
        eprintln!("{linha}\n    fronteiras {fronteiras:?} · {saida:?}");
    }
}

/// **ONDE NASCE O Δ** (doc 131 §2): no tique 1, a 2.ª coluna da `=129` estágio a estágio — o corpo,
/// o `motion.transform` e a saída, dos dois lados — com as colunas que cada lado leva.
#[test]
#[ignore = "sonda: placa (precisa de adapter), não um gate"]
fn a_deriva_da_129_onde_nasce() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adapter");
    for t in [1u64, 3] {
        let (mut m, mut cpu, corpos) = gemeos();
        let sink = m.sinks[1];
        let lugar = ph2d_nodegraph::cook::upstream_cone(&m.doc.graph, sink)
            .into_iter()
            .find(|n| {
                m.doc
                    .graph
                    .node(*n)
                    .is_some_and(|i| i.type_name == "motion.transform")
            })
            .expect("a 2.ª saída tem o transform");
        crate::motion_externals::publish_all(&mut m, t as f64 * DT);
        crate::motion_externals::publish_all(&mut cpu, t as f64 * DT);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        m.gpu_cook.retain_streams_for_debug(true);
        cook_gpu(&mut m, &gpu, t, DT, &scopes);
        let mut taps = cpu.sinks.clone();
        taps.extend([lugar, corpos[1]]);
        cpu.pump.set_taps(&taps);
        for tick in 0..=t {
            cpu.pump.advance_or_scrub_scoped(
                &cpu.doc.graph,
                &cpu.registry,
                &cpu.sinks,
                tick,
                |k| k as f64 * DT,
                cpu.default_uv_rect,
                cpu.default_size,
                &scopes,
            );
        }
        let placa = |n: NodeId| {
            m.gpu_cook
                .read_column_vec2(&gpu, n, "P")
                .unwrap_or_default()
        };
        let colunas = |n: NodeId| {
            cpu.pump
                .tap_streams()
                .iter()
                .find(|(k, _)| *k == n)
                .map(|(_, s)| s.columns().map(|(k, _)| k.clone()).collect::<Vec<_>>())
        };
        eprintln!("=== T={t} · 2.ª coluna · {:?}", m.route_said);
        for (nome, n) in [("corpo", corpos[1]), ("transform", lugar), ("saída", sink)] {
            eprintln!(
                "  {nome:<9} colunas CPU {:?} · placa {:?}",
                colunas(n),
                m.gpu_cook.debug_column_names(n)
            );
        }
        let (ct, cs) = (tomada(&cpu, lugar), tomada(&cpu, sink));
        let (pt, ps) = (placa(lugar), placa(sink));
        eprintln!("  CPU   transform × saída  {:.3e}", delta(&ct, &cs).0);
        eprintln!("  placa transform × saída  {:.3e}", delta(&pt, &ps).0);
        eprintln!(
            "  transform placa × CPU    {:.3e} (n {} × {})",
            delta(&pt, &ct).0,
            pt.len(),
            ct.len()
        );
        eprintln!("  saída     placa × CPU    {:.3e}", delta(&ps, &cs).0);
        let difs: Vec<(usize, f32)> = ps
            .iter()
            .zip(&cs)
            .enumerate()
            .map(|(i, (a, b))| (i, (a[0] - b[0]).abs().max((a[1] - b[1]).abs())))
            .filter(|(_, d)| *d > 1e-5)
            .collect();
        eprintln!("  peças com |Δ| > 1e-5 na saída: {} → {difs:?}", difs.len());
    }
}

/// ⭐⭐ **O GATE: a tomada da saída com `Collide` é o que ela DESENHA** (doc 131 §2). Os números do
/// cartão da placa liam a corrente de ANTES do passe de separação e os da CPU a de depois — duas
/// respostas à pergunta «onde estão as peças?». Medido na `=129` a `T = 3`: `2,742e-3` antes da cura,
/// `1,2e-7` depois. Controlo: a mesma tomada no `motion.transform` (antes do passe) tem de DIFERIR —
/// sem ele, um passe que não separasse passaria aqui com as duas leituras iguais.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_tomada_da_saida_com_collide_e_o_que_ela_desenha() {
    const T: u64 = 3;
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adapter");
    let (mut m, mut cpu, _) = gemeos();
    let sink = m.sinks[1];
    let lugar = ph2d_nodegraph::cook::upstream_cone(&m.doc.graph, sink)
        .into_iter()
        .find(|n| {
            m.doc
                .graph
                .node(*n)
                .is_some_and(|i| i.type_name == "motion.transform")
        })
        .expect("a 2.ª saída tem o transform");
    crate::motion_externals::publish_all(&mut m, T as f64 * DT);
    crate::motion_externals::publish_all(&mut cpu, T as f64 * DT);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    cook_gpu(&mut m, &gpu, T, DT, &scopes);
    // Amostras ≥ linhas ⇒ a leitura traz todas, pela ordem.
    let lida = m.gpu_cook.tap(&gpu, 1024).expect("a placa tem tomadas");
    let da_placa = |n: NodeId| lida.get(&n).map(p_de).unwrap_or_default();
    cpu.pump.set_taps(&[sink]);
    for tick in 0..=T {
        cpu.pump.advance_or_scrub_scoped(
            &cpu.doc.graph,
            &cpu.registry,
            &cpu.sinks,
            tick,
            |k| k as f64 * DT,
            cpu.default_uv_rect,
            cpu.default_size,
            &scopes,
        );
    }
    let (ps, pt, cs) = (da_placa(sink), da_placa(lugar), tomada(&cpu, sink));
    assert_eq!(ps.len(), 49, "a tomada da saída traz as 49 peças");
    assert_eq!(cs.len(), ps.len());
    let (d, _) = delta(&ps, &cs);
    let (separou, _) = delta(&ps, &pt);
    eprintln!("tomada da saída placa × CPU {d:e} · saída × transform na placa {separou:e}");
    assert!(d <= 1e-5, "o cartão da placa lê outra corrente: {d:e}");
    assert!(
        separou > 1e-3,
        "controlo: o passe não separou ({separou:e})"
    );
}
