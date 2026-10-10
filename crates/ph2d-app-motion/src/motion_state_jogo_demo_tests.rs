//! Os gates da cena `=134`: a fixtura CONTÉM os três fenómenos, medidos NA SAÍDA pela bomba do
//! produto (o publicador das entradas, os escopos de tempo do produto, a bomba), em tempos
//! diferentes — e o scrub repete o que se viu.

use super::*;
use crate::motion_state::MotionState;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_nodegraph::attr::{Column, Stream};

struct Corrida {
    m: MotionState,
    c: Cena,
}

impl Corrida {
    fn nova() -> Self {
        let mut m = MotionState::new();
        m.doc = MotionDoc::new();
        let c = monta(&mut m.doc, &m.registry).expect("a cena monta");
        m.sinks = vec![c.barra, c.cruza, c.galaxia];
        m.pump.define_a_lei(true);
        m.pump.mark_dirty();
        Self { m, c }
    }

    /// Um tique pelo caminho da CPU do produto (`coze_o_quadro`): publica, escopos, bomba.
    fn tique(&mut self, t: u64) {
        let m = &mut self.m;
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
    }

    fn jogo(&mut self, nome: &str, v: Valor) {
        self.m.entradas.jogo.set(nome, Escritor::Script, v);
    }

    fn saida(&self, no: NodeId) -> Stream {
        self.m.pump.cook.peek(no).expect("cozeu")[0]
            .as_stream()
            .clone()
    }

    fn v(&self, no: NodeId) -> f32 {
        match self.saida(no).get("v") {
            Some(Column::Scalar(v)) => v[0],
            _ => f32::NAN,
        }
    }
}

fn coluna2(s: &Stream, nome: &str) -> Vec<[f32; 2]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Quantas peças da barra estão CHEIAS (a escala do ponto ≥ 0,99).
fn cheias(r: &Corrida) -> usize {
    coluna2(&r.saida(r.c.barra), "size")
        .iter()
        .filter(|s| s[0] >= 0.99 * PECA)
        .count()
}

/// O raio médio de um quadro, à volta do centro dele.
fn raio_medio(r: &Corrida, no: NodeId, centro: [f32; 2]) -> f32 {
    let p = coluna2(&r.saida(no), "P");
    #[expect(clippy::cast_precision_loss, reason = "centenas de pecas")]
    let n = p.len() as f32;
    p.iter()
        .map(|q| (q[0] - centro[0]).hypot(q[1] - centro[1]))
        .sum::<f32>()
        / n
}

/// ⭐ **A barra é a vida**, e o relógio não conta: o cartão (0,6) e depois o jogo.
#[test]
fn the_bar_shows_the_published_life_whatever_the_clock() {
    let mut r = Corrida::nova();
    r.tique(0);
    let de_fabrica = cheias(&r);
    let mut medido = vec![("cartao 0,6", de_fabrica)];
    for (t, vida) in [(1u64, 0.0f32), (2, 0.25), (3, 0.5), (4, 1.0)] {
        r.jogo("vida", Valor::Numero(vida));
        r.tique(t);
        medido.push(("jogo", cheias(&r)));
    }
    eprintln!("cena 134, a barra: {medido:?}");
    let n: Vec<usize> = medido.iter().map(|m| m.1).collect();
    assert_eq!(n[1], 0, "vida 0: barra vazia");
    assert!(
        n[2] < n[3] && n[3] < n[4],
        "a barra cresce com a vida: {n:?}"
    );
    assert_eq!(n[4], 24, "vida 1: cheia");
    assert!(
        (13..=15).contains(&n[0]),
        "o cartão (0,6) enche ~60%: {}",
        n[0]
    );
    // O relógio não conta: a mesma vida, muito mais tarde, a mesma barra.
    for t in 5..300 {
        r.tique(t);
    }
    assert_eq!(cheias(&r), 24);
}

/// ⭐ **O estado cruza em `0,6 s`** — o `Follow` a meio no meio do tempo, e a grelha vira o anel.
#[test]
fn the_state_crosses_from_the_grid_to_the_ring_in_its_duration() {
    let mut r = Corrida::nova();
    let centro = [-COL_X, super::QUADROS_Y];
    let mut pesos = Vec::new();
    for t in 0..=80u64 {
        r.jogo("estado", Valor::Numero(if t >= 30 { 1.0 } else { 0.0 }));
        r.tique(t);
        pesos.push(r.v(r.c.seguir));
    }
    eprintln!(
        "cena 134, o peso: t30 {:.3} t48 {:.3} t66 {:.3}",
        pesos[30], pesos[48], pesos[66]
    );
    assert_eq!(
        pesos[30], 0.0,
        "no tique do pedido ainda é o de antes (o Godot)"
    );
    assert!((pesos[48] - 0.5).abs() < 0.02, "a meio: {}", pesos[48]);
    assert_eq!(pesos[66], 1.0);
    let raio = raio_medio(&r, r.c.cruza, centro);
    eprintln!("cena 134, o anel: raio medio {raio:.4}");
    assert!((raio - 1.6).abs() < 0.01, "o anel: raio médio {raio}");
}

/// ⭐ **A cor do time** chega aos discos pelos canais do `Tint`.
#[test]
fn the_team_colour_reaches_the_discs() {
    let mut r = Corrida::nova();
    r.jogo("time", Valor::Cor([0.2, 0.4, 0.9, 1.0]));
    r.tique(0);
    let s = r.saida(r.c.cruza);
    let Some(Column::Vec4(t)) = s.get("tint") else {
        panic!("a saida tem tint")
    };
    for c in t {
        for k in 0..3 {
            assert!((c[k] - [0.2, 0.4, 0.9][k]).abs() < 1e-4, "{c:?}");
        }
    }
}

/// O tamanho médio das estrelas.
fn tamanho_medio(r: &Corrida) -> f32 {
    let s = coluna2(&r.saida(r.c.galaxia), "size");
    #[expect(clippy::cast_precision_loss, reason = "centenas de pecas")]
    let n = s.len() as f32;
    s.iter().map(|x| x[0]).sum::<f32>() / n
}

/// ⭐ **Cada boom sacode a galáxia SIMULADA**: o redemoinho (o campo) acelera e as estrelas
/// incham — e tudo volta à calma em 2 s, sem mudar o que a simulação era.
#[test]
fn a_boom_shakes_the_simulated_galaxy_and_it_calms_down() {
    let mut r = Corrida::nova();
    let centro = [COL_X, super::QUADROS_Y];
    let mut antes: Vec<[f32; 2]> = Vec::new();
    let (mut giros, mut tamanhos) = (Vec::new(), Vec::new());
    for t in 0..300u64 {
        if t == 60 {
            r.m.entradas.jogo.fire("boom", Escritor::Sinal);
        }
        r.tique(t);
        let agora = coluna2(&r.saida(r.c.galaxia), "P");
        giros.push(if antes.is_empty() {
            0.0
        } else {
            giro(&antes, &agora, centro)
        });
        tamanhos.push(tamanho_medio(&r));
        antes = agora;
    }
    let pico = |v: &[f32]| v[60..120].iter().copied().fold(0.0f32, f32::max);
    eprintln!(
        "cena 134, o boom: giro {:.4} -> pico {:.4} -> 2 s {:.4} voltas/s · tamanho {:.3} -> pico {:.3} -> 2 s {:.3}",
        giros[59],
        pico(&giros),
        giros[180],
        tamanhos[59],
        pico(&tamanhos),
        tamanhos[180]
    );
    assert!(pico(&giros) > 3.0 * giros[59], "o redemoinho acelera");
    assert!(
        (giros[180] - giros[59]).abs() < 0.1 * giros[59],
        "e volta à calma"
    );
    assert!(pico(&tamanhos) > 2.0 * tamanhos[59], "as estrelas incham");
    assert_eq!(tamanhos[180], tamanhos[59], "e voltam ao tamanho");
}

/// ⭐⭐ **O scrub repete o jogo**: toca com a vida, o estado e um boom a mudarem, e voltar a fita
/// mostra, em cada quadro, o que se viu ao tocar.
#[test]
fn scrubbing_back_replays_what_the_game_did() {
    let mut r = Corrida::nova();
    let mut visto = Vec::new();
    for t in 0..=120u64 {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        r.jogo("vida", Valor::Numero(t as f32 / 120.0));
        r.jogo("estado", Valor::Numero(if t >= 30 { 1.0 } else { 0.0 }));
        if t == 60 {
            r.m.entradas.jogo.fire("boom", Escritor::Sinal);
        }
        r.tique(t);
        visto.push((
            cheias(&r),
            coluna2(&r.saida(r.c.galaxia), "P"),
            r.v(r.c.seguir),
        ));
    }
    for alvo in [70u64, 40, 100, 65, 10] {
        r.tique(alvo);
        let agora = (
            cheias(&r),
            coluna2(&r.saida(r.c.galaxia), "P"),
            r.v(r.c.seguir),
        );
        assert!(
            agora == visto[alvo as usize],
            "scrub para {alvo}: a barra {} contra {}, o peso {} contra {}",
            agora.0,
            visto[alvo as usize].0,
            agora.2,
            visto[alvo as usize].2
        );
    }
}

/// A velocidade angular média das estrelas entre dois quadros (voltas por segundo).
fn giro(antes: &[[f32; 2]], agora: &[[f32; 2]], centro: [f32; 2]) -> f32 {
    let ang = |q: &[f32; 2]| (q[1] - centro[1]).atan2(q[0] - centro[0]);
    #[expect(clippy::cast_precision_loss, reason = "centenas de pecas")]
    let n = antes.len() as f32;
    antes
        .iter()
        .zip(agora)
        .map(|(a, b)| {
            let mut d = ang(b) - ang(a);
            if d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            } else if d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            d.abs()
        })
        .sum::<f32>()
        / n
        * 60.0
        / std::f32::consts::TAU
}
