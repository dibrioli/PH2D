//! Os gates da cena `=128`: a ESTRUTURA (o fio existe e é o Number que manda) e o que o roteiro
//! PROMETE ao dono, medido na marcha — uma cena que ensina o contrário do que acontece é pior que
//! cena nenhuma (`CLAUDE.md` §5.0).

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

fn cena() -> (MotionState, NodeId, NodeId) {
    let mut m = MotionState::new();
    let (sinks, numero, vortex) = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.doc.graph.validate(&m.registry).expect("bem-tipada");
    m.sinks = sinks;
    (m, numero, vortex)
}

/// Marcha `de..=ate` tiques pela bomba da CPU e devolve o `P` do integrador em `de` e em `ate`.
fn marcha(m: &mut MotionState, de: u64, ate: u64) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let ig = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.integrate")
        .map(|n| n.id)
        .expect("a galáxia tem integrador");
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let mut p_de = Vec::new();
    for t in 0..=ate {
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        if t == de {
            p_de = p_de_ig(m, ig);
        }
    }
    (p_de, p_de_ig(m, ig))
}

fn p_de_ig(m: &MotionState, ig: NodeId) -> Vec<[f32; 2]> {
    match m
        .pump
        .cook
        .peek(ig)
        .map(|v| v[0].as_stream().get("P").cloned())
    {
        Some(Some(Column::Vec2(p))) => p,
        o => panic!("o integrador nao tem P: {o:?}"),
    }
}

/// `(giro médio em rad, raio médio no fim / raio médio no início)` entre os dois instantes.
/// ⚠️ O giro conta-se em Y-para-CIMA (anti-horário positivo).
fn giro_e_raio(a: &[[f32; 2]], b: &[[f32; 2]]) -> (f32, f32) {
    let mut giro = 0.0f32;
    let (mut ra, mut rb) = (0.0f32, 0.0f32);
    for (p, q) in a.iter().zip(b) {
        let mut d = q[1].atan2(q[0]) - p[1].atan2(p[0]);
        if d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        } else if d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        giro += d;
        ra += p[0].hypot(p[1]);
        rb += q[0].hypot(q[1]);
    }
    #[expect(clippy::cast_precision_loss, reason = "mil estrelas")]
    let n = a.len().max(1) as f32;
    (giro / n, rb / ra.max(f32::EPSILON))
}

/// O giro e o raio com o Number em `valor`, medidos entre os tiques `60` e `180` (a galáxia já
/// assentou no primeiro).
fn com_o_number_em(valor: f32) -> (f32, f32) {
    let (mut m, numero, _) = cena();
    m.doc.graph.set_param(numero, "value", valor);
    let (a, b) = marcha(&mut m, 60, 180);
    giro_e_raio(&a, &b)
}

/// **O FIO EXISTE, e o cartão do Vortex diz um número que NÃO é o do Number** — senão a row
/// ligada leria igual com e sem a cura, e o passo (1) não ensinaria nada.
#[test]
fn the_number_drives_the_vortex_strength_and_starts_away_from_the_card() {
    let (m, numero, vortex) = cena();
    let fontes = m.doc.graph.param_sources(vortex).expect("o Vortex tem fio");
    assert_eq!(fontes.get("strength"), Some(&(numero, 0)));
    let cartao = m
        .doc
        .graph
        .node_param_overrides(vortex)
        .and_then(|o| o.get("strength").copied())
        .expect("o cartão tem um Strength escrito");
    assert!(
        (cartao - FIO_INICIAL).abs() > 1.0,
        "o cartão ({cartao}) e o fio ({FIO_INICIAL}) têm de diferir à vista"
    );
}

/// ⭐⭐⭐ **O QUE O ROTEIRO PROMETE, MEDIDO** — passos (2), (3) e (4): mais força gira mais depressa,
/// zero não gira, e abaixo de zero gira AO CONTRÁRIO. CONTROLO: o sentido do arranque é o oposto
/// do negativo (senão «ao contrário» passaria sobre uma galáxia parada).
#[test]
fn the_script_tells_the_truth_faster_still_and_backwards() {
    let (inicio, raio_inicio) = com_o_number_em(FIO_INICIAL);
    let (depressa, raio_depressa) = com_o_number_em(10.0);
    let (parado, raio_parado) = com_o_number_em(0.0);
    let (contrario, raio_contrario) = com_o_number_em(-4.0);
    eprintln!(
        "giro (raio): {FIO_INICIAL} -> {inicio:.3} (x{raio_inicio:.2}) · 10 -> {depressa:.3} \
         (x{raio_depressa:.2}) · 0 -> {parado:.3} (x{raio_parado:.2}) · -4 -> {contrario:.3} \
         (x{raio_contrario:.2})"
    );
    assert!(inicio.abs() > 0.2, "a galáxia gira no arranque ({inicio})");
    assert!(
        depressa.abs() > 1.5 * inicio.abs() && depressa.signum() == inicio.signum(),
        "(2) mais força gira MAIS depressa, no mesmo sentido: {inicio} -> {depressa}"
    );
    assert!(
        parado.abs() < 0.1 * inicio.abs(),
        "(3) com zero o redemoinho para: {parado}"
    );
    // ⚠️ E ELAS FICAM: o ímã é fraco e o Vortex em modo alvo trava (medido x0,99). A 1.ª redacção
    // do roteiro prometia *«caem para o meio»*, e foi este gate que a desmentiu. Em nenhum valor
    // do roteiro a galáxia foge do ecrã nem colapsa.
    for (passo, raio) in [
        ("arranque", raio_inicio),
        ("(2)", raio_depressa),
        ("(3)", raio_parado),
        ("(4)", raio_contrario),
    ] {
        assert!(
            (0.8..1.25).contains(&raio),
            "{passo}: a galáxia muda de tamanho x{raio} — o roteiro não o diz"
        );
    }
    assert!(
        contrario.signum() == -inicio.signum() && contrario.abs() > 0.2,
        "(4) abaixo de zero gira AO CONTRÁRIO: {inicio} -> {contrario}"
    );
}
