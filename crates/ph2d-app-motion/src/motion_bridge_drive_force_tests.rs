//! ⛔ **«UM NUMBER NO STRENGTH DO VORTEX NÃO TEM EFEITO»** — as medições do report do Enio de
//! 2026-10-03 ([BUGS #12](../../../docs/Motion%20Nodes/BUGS_motion_nodes.md)). Elas DERRUBARAM as
//! hipóteses do motor: o fio age na marcha da CPU, na rota `cook_gpu` e ligado a meio da corrida
//! pelo gesto — quem mentia era a row do cartão. Ficam: o gate da CPU (barato, sem placa) e as
//! sondas da placa, que são a bissecção de quem voltar a ouvir *«o fio não faz nada»*.
//!
//! `SONDA_PLACA=1 PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib -- --ignored --nocapture sonda_`

use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};

fn cena(fio: Option<f32>, cartao: f32) -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let grid = g.add_node("motion.grid".to_string());
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 3.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    let ig = g.add_node("motion.integrate".to_string());
    let out = g.add_node("motion.output".to_string());
    let vx = g.add_node("force.vortex".to_string());
    g.set_param(vx, "strength", cartao);
    g.set_param(vx, "radius", 6.0);
    for (a, b, d) in [
        ((grid, 0), (ig, 0), false),
        ((ig, 0), (vx, 0), true),
        ((vx, 0), (ig, 1), false),
        ((ig, 0), (out, 0), false),
    ] {
        g.connect(Edge {
            from: a,
            to: b,
            delayed: d,
        })
        .expect("liga");
    }
    if let Some(v) = fio {
        let num = g.add_node("value.number".to_string());
        g.set_param(num, "value", v);
        g.drive_param(vx, "strength", (num, 0)).expect("conduz");
    }
    m.doc.graph.validate(&m.registry).expect("valida");
    m.sinks = vec![out];
    (m, ig)
}

fn marcha(m: &mut MotionState, ate: u64) -> Vec<[f32; 2]> {
    let sinks = m.sinks.clone();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    for t in 0..=ate {
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            &scopes,
        );
    }
    let out = sinks[0];
    let v = m.pump.cook.peek(out).expect("cozido");
    match v[0].as_stream().get("P") {
        Some(ph2d_nodegraph::attr::Column::Vec2(p)) => p.clone(),
        o => panic!("sem P: {o:?}"),
    }
}

/// ⭐⭐ **O NUMBER NO STRENGTH MOVE A SIMULAÇÃO COMO O CARTÃO MOVERIA** — pela marcha da bomba,
/// ao bit: `cartão 4 + fio 20` é `cartão 20`, `fio 0` é `cartão 0`, e `fio −20` gira ao contrário.
/// CONTROLO: `20` e `0` dão campos diferentes (senão a igualdade passaria sobre um campo parado).
#[test]
fn a_number_on_the_vortex_strength_moves_the_sim_like_the_card_would() {
    let bits = |p: Vec<[f32; 2]>| p.iter().map(|v| v.map(f32::to_bits)).collect::<Vec<_>>();
    let corre = |fio, cartao| {
        let (mut m, _) = cena(fio, cartao);
        marcha(&mut m, 60)
    };
    let vinte = corre(None, 20.0);
    let zero = corre(None, 0.0);
    assert_ne!(vinte, zero, "controlo: a força move o campo");
    assert_eq!(
        bits(corre(Some(20.0), 4.0)),
        bits(vinte.clone()),
        "o fio manda, não o cartão"
    );
    assert_eq!(
        bits(corre(Some(0.0), 4.0)),
        bits(zero),
        "fio 0 = força nenhuma"
    );
    let contra = corre(Some(-20.0), 4.0);
    for (p, q) in vinte.iter().zip(&contra) {
        // O espelho de um giro em torno da origem: o mesmo raio, o ângulo ao contrário.
        assert!(
            (p[0].hypot(p[1]) - q[0].hypot(q[1])).abs() < 1e-4,
            "{p:?} contra {q:?}"
        );
    }
    assert_ne!(contra, vinte, "−20 gira ao contrário");
}

#[test]
#[ignore = "sonda, precisa de adapter"]
fn sonda_gpu_number_no_strength_do_vortex() {
    let Some(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    else {
        eprintln!("sem adapter");
        return;
    };
    for (nome, fio, cartao) in [
        ("cartao 0, sem fio", None, 0.0),
        ("cartao 20, sem fio", None, 20.0),
        ("cartao 4, fio 0", Some(0.0), 4.0),
        ("cartao 4, fio 20", Some(20.0), 4.0),
        ("cartao 4, fio -20", Some(-20.0), 4.0),
    ] {
        let (mut m, ig) = cena(fio, cartao);
        m.gpu_cook.retain_streams_for_debug(true);
        let scopes = ph2d_nodegraph::cook::TimeScopes::new();
        let mut rotas = Vec::new();
        for t in 0..=60u64 {
            let o = super::super::gpu::cook_gpu(&mut m, &gpu, t, 1.0 / 60.0, &scopes);
            if t == 0 || t == 60 {
                rotas.push(format!(
                    "{:?}/{}/{:?}",
                    matches!(o, super::super::gpu::GpuOutcome::Handled),
                    m.gpu_live,
                    m.route_said
                ));
            }
        }
        let p = m
            .gpu_cook
            .read_column_vec2(&gpu, ig, "P")
            .unwrap_or_default();
        eprintln!(
            "{nome:<22} n={} p[0]={:?} p[2]={:?} rotas={rotas:?}",
            p.len(),
            p.first(),
            p.get(2)
        );
    }
}

/// Liga o fio A MEIO da corrida, pela porta do gesto (`subgraph::drive`), e edita o Number depois.
#[test]
#[ignore = "sonda, precisa de adapter"]
fn sonda_fio_a_meio_da_corrida() {
    let gpu = std::env::var_os("SONDA_PLACA").and_then(|_| {
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    });
    for placa in [false, true] {
        if placa && gpu.is_none() {
            continue;
        }
        for (nome, liga, edita) in [
            ("controlo: sem fio", false, None),
            ("fio no tique 30 (Number 20)", true, None),
            ("fio no 30, Number -> -20 no 45", true, Some(-20.0)),
        ] {
            let (mut m, ig) = cena(None, 4.0);
            m.gpu_enabled = placa;
            m.gpu_cook.retain_streams_for_debug(true);
            let num = m.doc.graph.add_node("value.number".to_string());
            m.doc.graph.set_param(num, "value", 20.0);
            let vx = m
                .doc
                .graph
                .nodes()
                .iter()
                .find(|n| n.type_name == "force.vortex")
                .unwrap()
                .id;
            let scopes = ph2d_nodegraph::cook::TimeScopes::new();
            let mut toasts = ph2d_editor_core::ToastQueue::new();
            for t in 0..=60u64 {
                if t == 30 && liga {
                    super::super::subgraph::drive(&mut m, &mut toasts, (num, 0), vx, "strength");
                }
                if t == 45
                    && let Some(v) = edita
                {
                    m.doc.graph.set_param(num, "value", v);
                    m.pump.mark_dirty();
                }
                if placa {
                    let o = super::super::gpu::cook_gpu(
                        &mut m,
                        gpu.as_ref().unwrap(),
                        t,
                        1.0 / 60.0,
                        &scopes,
                    );
                    assert!(
                        matches!(o, super::super::gpu::GpuOutcome::Handled),
                        "{:?}",
                        m.route_said
                    );
                } else {
                    let sinks = m.sinks.clone();
                    let _ = m.pump.advance_or_scrub_scoped(
                        &m.doc.graph,
                        &m.registry,
                        &sinks,
                        t,
                        |t| t as f64 / 60.0,
                        [0.0, 0.0, 1.0, 1.0],
                        [1.0, 1.0],
                        &scopes,
                    );
                }
            }
            let p: Vec<[f32; 2]> = if placa {
                m.gpu_cook
                    .read_column_vec2(gpu.as_ref().unwrap(), ig, "P")
                    .unwrap_or_default()
            } else {
                match m.pump.cook.peek(m.sinks[0]).unwrap()[0]
                    .as_stream()
                    .get("P")
                {
                    Some(ph2d_nodegraph::attr::Column::Vec2(p)) => p.clone(),
                    _ => vec![],
                }
            };
            let fontes = m.doc.graph.all_param_sources().len();
            eprintln!(
                "placa={placa} {nome:<32} fontes={fontes} toasts={} p[0]={:?}",
                toasts.len(),
                p.first()
            );
        }
    }
}

/// A cena `=127` do dono (estrelas + galáxia), o fio ligado a meio pela porta do gesto.
#[test]
#[ignore = "sonda"]
fn sonda_127_fio_a_meio() {
    use crate::motion_state::traco_esticado_demo as d;
    let gpu = std::env::var_os("SONDA_PLACA").and_then(|_| {
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    });
    for placa in [false, true] {
        if placa && gpu.is_none() {
            continue;
        }
        for (nome, valor) in [
            ("controlo: sem fio", None),
            ("fio no 30, Number 1", Some(1.0)),
            ("fio no 30, Number 30", Some(30.0)),
        ] {
            let mut m = MotionState::new();
            let sinks =
                d::monta(&mut m.doc, &m.registry, d::arranjo_por(None), false).expect("monta");
            m.sinks = sinks;
            m.gpu_enabled = placa;
            m.gpu_cook.retain_streams_for_debug(true);
            let ig = m
                .doc
                .graph
                .nodes()
                .iter()
                .find(|n| n.type_name == "motion.integrate")
                .unwrap()
                .id;
            let vx = m
                .doc
                .graph
                .nodes()
                .iter()
                .find(|n| n.type_name == "force.vortex")
                .unwrap()
                .id;
            let num = m.doc.graph.add_node("value.number".to_string());
            if let Some(v) = valor {
                m.doc.graph.set_param(num, "value", v);
            }
            let scopes = ph2d_nodegraph::cook::TimeScopes::new();
            let mut toasts = ph2d_editor_core::ToastQueue::new();
            let mut rota = None;
            for t in 0..=90u64 {
                crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
                if t == 30 && valor.is_some() {
                    super::super::subgraph::drive(&mut m, &mut toasts, (num, 0), vx, "strength");
                }
                if placa {
                    let _ = super::super::gpu::cook_gpu(
                        &mut m,
                        gpu.as_ref().unwrap(),
                        t,
                        1.0 / 60.0,
                        &scopes,
                    );
                    rota = m.route_said;
                } else {
                    let s = m.sinks.clone();
                    let _ = m.pump.advance_or_scrub_scoped(
                        &m.doc.graph,
                        &m.registry,
                        &s,
                        t,
                        |t| t as f64 / 60.0,
                        m.default_uv_rect,
                        m.default_size,
                        &scopes,
                    );
                }
            }
            let p: Vec<[f32; 2]> = if placa {
                m.gpu_cook
                    .read_column_vec2(gpu.as_ref().unwrap(), ig, "P")
                    .unwrap_or_default()
            } else {
                match m
                    .pump
                    .cook
                    .peek(ig)
                    .map(|v| v[0].as_stream().get("P").cloned())
                {
                    Some(Some(ph2d_nodegraph::attr::Column::Vec2(p))) => p,
                    _ => vec![],
                }
            };
            eprintln!(
                "placa={placa} {nome:<24} toasts={} n={} p[0]={:?} p[7]={:?} rota={rota:?}",
                toasts.len(),
                p.len(),
                p.first(),
                p.get(7)
            );
        }
    }
}
