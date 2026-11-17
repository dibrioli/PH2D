//! Os gates da cena `=143`, pela bomba do produto (publicar, escopos, cozer): a barra segue a vida
//! ATRAVÉS do `Send signal`, `low_health` sai UMA vez ao cruzar `0.25`, e o rastro aponta para a velocidade.

use super::*;
use crate::motion_bridge::signals::{collect_signals, signal_nodes};
use crate::motion_state::MotionState;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_nodegraph::attr::Column;

fn nova() -> (MotionState, Cena) {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let c = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = vec![c.barra, c.saida_rastro];
    m.signal_taps = signal_nodes(&m.doc.graph);
    m.pump.set_taps(&m.signal_taps);
    m.pump.define_a_lei(true);
    m.pump.mark_dirty();
    (m, c)
}

/// Um tique da CPU do produto; devolve quantos `low_health` saíram nele.
fn tique(m: &mut MotionState, t: u64) -> usize {
    #[expect(clippy::cast_precision_loss, reason = "um tique")]
    crate::motion_externals::publish_all(m, t as f64 / 60.0);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
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
    assert!(m.pump.last_error().is_none(), "{:?}", m.pump.last_error());
    collect_signals(m);
    m.pump.clear_tap_fires();
    let n = m.signals_out.iter().filter(|s| s.name == SINAL).count();
    m.signals_out.clear();
    n
}

fn pontos(m: &MotionState, no: NodeId, coluna: &str) -> Vec<[f32; 2]> {
    match m.pump.cook.peek(no).expect("cozeu")[0]
        .as_stream()
        .get(coluna)
    {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐ A vida desce em degraus de 0.2 (1 → 0.2) e fica: a barra esvazia pelo fio do nó, e `low_health`
/// sai UMA vez, no degrau que cruza 0.25. Controlo: a mesma descida até 0.4 não manda nada.
#[test]
fn low_health_is_sent_once_when_the_bar_crosses_a_quarter() {
    for (fundo, esperado) in [(0.2f32, 1usize), (0.4, 0)] {
        let (mut m, c) = nova();
        let mut enviados = 0;
        let mut cheias = Vec::new();
        for t in 0..240u64 {
            #[expect(clippy::cast_precision_loss, reason = "um tique")]
            let vida = (1.0 - 0.2 * (t / 30) as f32).max(fundo);
            m.entradas
                .jogo
                .set("vida", Escritor::Script, Valor::Numero(vida));
            enviados += tique(&mut m, t);
            let n = pontos(&m, c.barra, "size")
                .iter()
                .filter(|s| s[0] >= 0.99 * super::super::jogo_demo::PECA)
                .count();
            cheias.push(n);
        }
        assert_eq!(enviados, esperado, "fundo {fundo}");
        assert!(
            cheias[10] > *cheias.last().unwrap(),
            "a barra esvazia: {cheias:?}"
        );
        assert!(m.doc.graph.param_sources(c.remap).is_some());
    }
}

/// ⭐ O rastro aponta para onde o herói corre: o par `(±4, 0)` / `(0, 4)` empurra o centro das estrelas
/// para esse lado; parado, elas ficam à volta da boca.
#[test]
fn the_trail_points_where_the_hero_runs() {
    let centro = |par: [f32; 2]| {
        let (mut m, c) = nova();
        for t in 0..90u64 {
            m.entradas
                .jogo
                .set(RASTRO, Escritor::Script, Valor::Vetor(par));
            tique(&mut m, t);
        }
        let p = pontos(&m, c.saida_rastro, "P");
        assert!(p.len() > 10, "o emissor soltou estrelas: {}", p.len());
        #[expect(clippy::cast_precision_loss, reason = "dezenas de estrelas")]
        let n = p.len() as f32;
        let s = p.iter().fold([0.0, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
        [s[0] / n - RASTRO_AT[0], s[1] / n - RASTRO_AT[1]]
    };
    let (parado, direita, esquerda, cima) = (
        centro([0.0; 2]),
        centro([4.0, 0.0]),
        centro([-4.0, 0.0]),
        centro([0.0, 4.0]),
    );
    eprintln!("rastro: parado {parado:?} direita {direita:?} esquerda {esquerda:?} cima {cima:?}");
    assert!(parado[0].hypot(parado[1]) < 0.2, "{parado:?}");
    assert!(direita[0] > 0.3 && direita[1].abs() < 0.2, "{direita:?}");
    assert!(esquerda[0] < -0.3, "{esquerda:?}");
    assert!(cima[1] > 0.3 && cima[0].abs() < 0.2, "{cima:?}");
}
