//! ⭐⭐⭐ **As cenas do passe com o `Collide` LIGADO, pela placa, contra a CPU** (doc 130 §6).
//!
//! As cenas `=114` (a pilha, `sim.step` na CPU ⇒ rota híbrida) e `=121`–`=123` (o passe sozinho,
//! com simulação e em cadeia) com o `Collide` de toda forma ligado: a rota da placa tem de ACEITAR
//! a cena, separar na placa, e desenhar as formas onde a CPU as desenha.

use super::{GpuOutcome, RECUSA_PASSE, cook_gpu};
use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_node_motion_shape::param;
use ph2d_nodegraph::node::NodeTypeId;

const TIQUE: u64 = 6;
const DT: f64 = 1.0 / 60.0;
/// Medido (11/10, RTX, `TIQUE = 6`): o pior desvio entre as rotas nas quatro cenas foi `5,96e-8`
/// (a `=122`); a `=114` deu `0`. Folga de ~17×.
const EPS: f32 = 1e-6;
/// O controlo «o passe MEXE na cena»: medido `2,65e-2` (`=121`), `1,68` (`=122`), `1,05e-1`
/// (`=123`). ⚠️ A `=114` dá `7,45e-9` e fica fora: lá quem separa é o `sim.step` (o rapier), e o
/// passe do fim encontra a pilha já separada — ela prova a ROTA (híbrida, aceite) e a igualdade.
const MEXE: f32 = 1e-3;
const CENAS: [u32; 4] = [114, 121, 122, 123];

fn estado(cena: u32, colide: bool) -> MotionState {
    let mut m = MotionState::new();
    let montadas = crate::motion_demo_legend::monta(&cena.to_string(), &mut m.doc, &m.registry).0;
    assert!(!montadas.is_empty(), "a cena {cena} existe");
    let formas: Vec<_> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| NodeTypeId::of(n.type_name.as_str()) == NodeTypeId::of("source.shape"))
        .map(|n| n.id)
        .collect();
    assert!(!formas.is_empty(), "a cena {cena} tem formas");
    for f in formas {
        m.doc
            .graph
            .set_param(f, param::COLLIDE, if colide { 1.0 } else { 0.0 });
    }
    m.sinks = super::super::remove::output_nodes(&m.doc.graph);
    m
}

fn formas_da_cpu(m: &mut MotionState) -> Vec<[f32; 2]> {
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    crate::motion_externals::publish_all(m, TIQUE as f64 * DT);
    for tick in 0..=TIQUE {
        m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &m.sinks,
            tick,
            |t| t as f64 * DT,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    let mut v: Vec<[f32; 2]> = m
        .pump
        .vector_instances
        .iter()
        .map(|i| i.world_pos)
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).expect("finito"));
    v
}

fn formas_da_placa(gpu: &GpuContext, m: &MotionState) -> Vec<[f32; 2]> {
    let w = ph2d_gpu_cook::lower_forma::FORMA_WORDS as usize;
    let mut v: Vec<[f32; 2]> = m
        .gpu_cook
        .formas()
        .map(|f| ph2d_gpu_cook::read_formas(gpu, f))
        .unwrap_or_default()
        .chunks_exact(w)
        .filter(|c| c[10] != ph2d_gpu_cook::lower_forma::SEM_GEOMETRIA)
        .map(|c| [f32::from_bits(c[0]), f32::from_bits(c[1])])
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).expect("finito"));
    v
}

fn pior(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
        .fold(0.0, f32::max)
}

/// ⭐ A rota ACEITA, a placa SEPARA, e as formas caem onde a CPU as põe — com o controlo de que o
/// passe mexe de facto na cena (o mesmo documento sem `Collide` desenha noutro sítio).
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn as_cenas_do_passe_com_collide_correm_na_placa_e_dao_o_que_a_cpu_da() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    for cena in CENAS {
        let mut m = estado(cena, true);
        let mut cpu = estado(cena, true);
        let mut sem = estado(cena, false);
        cpu.gpu_enabled = false;
        sem.gpu_enabled = false;
        crate::motion_externals::publish_all(&mut m, TIQUE as f64 * DT);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        let saida = cook_gpu(&mut m, &gpu, TIQUE, DT, &scopes);
        assert!(
            matches!(saida, GpuOutcome::Handled),
            "={cena}: a rota recusou — {:?}",
            m.route_said
        );
        assert_ne!(m.route_said, Some(RECUSA_PASSE), "={cena}");
        assert!(
            m.gpu_cook.passe_correu() >= 1,
            "={cena}: a placa nao separou"
        );
        let (fp, fc, fs) = (
            formas_da_placa(&gpu, &m),
            formas_da_cpu(&mut cpu),
            formas_da_cpu(&mut sem),
        );
        let (d, controlo) = (pior(&fp, &fc), pior(&fc, &fs));
        eprintln!(
            "={cena} {:?}: {} formas · |placa − cpu| = {d:.2e} · |com − sem Collide| = {controlo:.2e}",
            m.route_said,
            fp.len()
        );
        assert_eq!(fp.len(), fc.len(), "={cena}: o numero de formas");
        // O painel de medição lê a rota que DESENHOU (doc 130 §5): a placa publica o `Relatorio`
        // sem espera, num quadro seguinte.
        for t in 1..=4 {
            let _ = cook_gpu(&mut m, &gpu, TIQUE + t, DT, &scopes);
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
        let r = m.pump.ultimo_relatorio();
        assert_eq!(
            r.pecas,
            cpu.pump.ultimo_relatorio().pecas,
            "={cena}: pecas do relatorio"
        );
        assert!(
            r.varreduras >= 1,
            "={cena}: o relatorio da placa chegou ao painel"
        );
        if cena != 114 {
            assert!(
                controlo > MEXE,
                "={cena}: o Collide nao mexeu na cena — gate vazio"
            );
        }
        assert!(d <= EPS, "={cena}: a placa desenha a {d} da CPU");
    }
}

/// O P da TOMADA de cada sink — o que o gizmo do colisor desenha.
fn p_da_tomada(m: &MotionState) -> Vec<[f32; 2]> {
    let mut v: Vec<[f32; 2]> = m
        .pump
        .tap_streams()
        .iter()
        .filter(|(n, _)| m.sinks.contains(n))
        .flat_map(|(_, s)| match s.get("P") {
            Some(ph2d_nodegraph::attr::Column::Vec2(p)) => p.clone(),
            _ => Vec::new(),
        })
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).expect("finito"));
    v
}

/// ⭐⭐ **O GIZMO DO COLISOR fica com as formas na rota da placa** (report do dono, 11/10, com foto:
/// *«funciona mas desalinhou os gizmos dos colliders»*). A tomada do sink é cozida na CPU, e a
/// marcha híbrida não lhe dizia que ele era uma saída — guardava a corrente CRUA, e o contorno azul
/// ficava onde as formas estavam ANTES da separação. Placa × CPU, com o controlo `Collide` desligado.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_tomada_do_sink_ve_a_separacao_na_rota_da_placa() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adapter — este gate mede a rota do device e nao tem versao de CPU");
    };
    for cena in [121, 122, 123] {
        let mut m = estado(cena, true);
        let mut cpu = estado(cena, true);
        let mut sem = estado(cena, false);
        cpu.gpu_enabled = false;
        sem.gpu_enabled = false;
        for x in [&mut m, &mut cpu, &mut sem] {
            let sinks = x.sinks.clone();
            x.pump.set_taps(&sinks);
        }
        crate::motion_externals::publish_all(&mut m, TIQUE as f64 * DT);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        let saida = cook_gpu(&mut m, &gpu, TIQUE, DT, &scopes);
        assert!(
            matches!(saida, GpuOutcome::Handled),
            "={cena}: {:?}",
            m.route_said
        );
        let _ = (formas_da_cpu(&mut cpu), formas_da_cpu(&mut sem));
        let (tp, tc, ts) = (p_da_tomada(&m), p_da_tomada(&cpu), p_da_tomada(&sem));
        assert!(
            !tp.is_empty() && tp.len() == tc.len(),
            "={cena}: a tomada do sink"
        );
        assert!(
            pior(&tc, &ts) > MEXE,
            "={cena}: controlo — a separacao mexe no P da tomada"
        );
        assert_eq!(
            tp, tc,
            "={cena}: a tomada da rota da placa e' a da CPU, ao bit"
        );
    }
}
