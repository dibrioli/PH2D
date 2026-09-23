//! Os portões da cena `=16` — o que a regra do dono pede (formas, simulação, várias saídas) e que
//! ela vá, de facto, à PLACA.

use super::build;
use crate::motion_bridge::gpu::{GpuRoute, gpu_route};
use crate::motion_state::MotionState;

/// ⭐⭐⭐ **A CENA VAI À PLACA, com as DUAS saídas encenadas** — sem isto ela mostraria a CPU e o
/// smoke do ciclo 11 aprovaria o que não foi visto.
#[test]
fn o_enxame_vai_a_placa_com_as_duas_saidas() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc.graph, "Particle");
    m.doc.graph.validate(&m.registry).expect("bem tipado");
    assert_eq!(sinks.len(), 2, "o ciclo 11 é sobre VÁRIAS saídas");
    let plan = ph2d_gpu_cook::plan_driven_many(
        &m.doc.graph,
        &m.registry,
        &m.registry,
        &sinks,
        &ph2d_gpu_cook::DrivenParams::new(),
    );
    assert_eq!(plan.sinks, sinks, "as duas saídas são encenadas na placa");
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let rota = gpu_route(
        true,
        sinks.len(),
        scopes.is_empty(),
        &plan.boundaries,
        plan.dispatching_stages(&m.registry),
    );
    assert!(
        matches!(rota, GpuRoute::Hybrid),
        "a =16 vai à placa pela rota HÍBRIDA (o carimbo na CPU); rota {rota:?}"
    );
    for &s in &sinks {
        assert!(!crate::motion_bridge::gpu::sink_mistura_em_grupo(
            &m.doc.graph,
            s
        ));
    }
}

/// ⭐⭐ **A regra do dono, afirmada na própria cena:** uma FORMA (o objecto) e SIMULAÇÃO com
/// CAMPOS (um integrador com forças em cada ramo). Sem isto a próxima redacção da cena podia
/// perder uma das duas sem nada reprovar — que é exactamente como elas foram esquecidas.
#[test]
fn o_enxame_tem_forma_e_simulacao_com_campos() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc.graph, "Particle");
    let tipos: Vec<String> = m
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.type_name.clone())
        .collect();
    let conta = |t: &str| tipos.iter().filter(|x| x.as_str() == t).count();
    assert_eq!(
        conta("source.object"),
        1,
        "a FORMA que a placa desenha, partilhada"
    );
    assert_eq!(
        conta("motion.integrate"),
        sinks.len(),
        "uma simulação por saída"
    );
    assert!(
        tipos.iter().filter(|t| t.starts_with("force.")).count() >= 2 * sinks.len(),
        "cada simulação tem campos de força"
    );
    assert_eq!(conta("source.shape"), 0, "uma forma viva recusaria a placa");
}

/// Quanto, em média, as peças de um enxame têm de se afastar da grelha de partida para ele se
/// ler como uma SIMULAÇÃO: um quarto do lado de partida (`40 × 0,06 = 2,4 m`). Medido na afinação
/// que shipa — ver o `eprintln!` do gate.
const MEXE: f32 = 0.6;

/// ⭐⭐ **OS ENXAMES FICAM NA TELA, cada um no seu lado** — a régua que afinou as forças, virada
/// gate (a 1.ª afinação atirava as peças a `33 m` em 15 s: a foto mostrava o ecrã inteiro
/// salpicado).
///
/// A barra NÃO é escolhida: cada enxame vive a [`super::CENTRO`] do meio da tela, logo uma peça
/// que se afaste mais do que isso do centro do SEU enxame invade o do vizinho. Medido na afinação
/// que shipa (15 s, 60 Hz, pela bomba da CPU — as leis são as mesmas nas duas rotas): o
/// redemoinho estabiliza com o p95 a `~1,9 m` e o máximo a `~2,3 m`, a nuvem com o máximo a
/// `~2,3 m`.
#[test]
fn os_enxames_ficam_cada_um_no_seu_lado() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc.graph, "Particle");
    let igs: Vec<_> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.integrate")
        .map(|n| n.id)
        .collect();
    assert_eq!(igs.len(), 2);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    m.pump.set_taps(&igs);
    let dt = 1.0 / 60.0;
    let mut pior = 0.0f32;
    let mut medidas = 0usize;
    // ⭐ E cada enxame tem de MEXER — sem esta metade uma cena com as forças desligadas (peças
    // paradas na grelha) passava, e foi exactamente essa a queixa do dono (*«não colocou campos
    // de simulação»*). A mutação que desliga a cadeia de forças SOBREVIVIA à 1.ª redacção.
    let mut inicio: std::collections::BTreeMap<_, Vec<[f32; 2]>> = Default::default();
    let mut andou: std::collections::BTreeMap<_, f32> = Default::default();
    for tick in 0..=900u64 {
        m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &igs,
            tick,
            |t| t as f64 * dt,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        if tick % 30 == 0 {
            for (n, st) in m.pump.tap_streams() {
                if let Some(ph2d_nodegraph::attr::Column::Vec2(v)) = st.get("P") {
                    let base = inicio.entry(*n).or_insert_with(|| v.clone());
                    let media = v
                        .iter()
                        .zip(base.iter())
                        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                        .sum::<f32>()
                        / v.len().max(1) as f32;
                    let a = andou.entry(*n).or_insert(0.0);
                    *a = a.max(media);
                    medidas += v.len();
                    pior = v.iter().map(|p| p[0].hypot(p[1])).fold(pior, f32::max);
                }
            }
        }
    }
    // O CONTROLO da régua: ela mediu peças (uma bomba sem tomadas lia 0 e passava).
    assert!(medidas > 1000, "a régua não viu peça nenhuma ({medidas})");
    eprintln!("afastamento medio maximo por enxame: {andou:?}");
    assert_eq!(andou.len(), 2);
    for (n, a) in &andou {
        assert!(
            *a > MEXE,
            "o enxame {n:?} quase não se mexeu ({a:.3} m em média) — a simulação não está a correr"
        );
    }
    assert!(
        pior < super::CENTRO,
        "uma peça chegou a {pior:.2} m do centro do seu enxame — passa do meio da tela ({} m) e \
         invade o do vizinho",
        super::CENTRO
    );
}
