//! Os gates da cena `=136` (doc 131 §3, D.13): o dano sai da vida pelo grafo, com a lei do
//! `Damage` do componente, e sacode a galáxia simulada.

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
        m.sinks = vec![c.saida];
        m.pump.define_a_lei(true);
        m.pump.mark_dirty();
        Self { m, c }
    }

    /// Um tique pelo caminho da CPU do produto: publica, escopos, bomba.
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

    fn saida(&self, no: NodeId) -> Stream {
        self.m.pump.cook.peek(no).expect("cozeu")[0]
            .as_stream()
            .clone()
    }

    fn disparou(&self) -> bool {
        matches!(self.saida(self.c.dano).get("pulse"), Some(Column::Scalar(v)) if v.first().is_some_and(|p| *p > 0.5))
    }

    /// O tamanho médio das estrelas desenhadas.
    fn tamanho(&self) -> f32 {
        match self.saida(self.c.saida).get("size") {
            Some(Column::Vec2(v)) if !v.is_empty() => {
                #[expect(clippy::cast_precision_loss, reason = "centenas de peças")]
                let n = v.len() as f32;
                v.iter().map(|s| s[0]).sum::<f32>() / n
            }
            _ => f32::NAN,
        }
    }
}

/// ⭐ **O DANO É A VIDA A DESCER** — a MESMA sequência do gate do `Damage` do componente
/// (`health_publishes_its_ratio_and_damage_fires_on_each_drop`: `10 → 7, 7, 9, 3` de 10): nascer
/// não dispara, cada descida dispara uma vez, ficar e subir não disparam. ⇒ um objecto com vida
/// basta; o segundo objecto só existia porque o componente tem UMA fonte.
#[test]
fn o_dano_e_a_vida_a_descer_como_o_damage_do_componente() {
    let mut r = Corrida::nova();
    r.tique(0);
    let mut disparos = vec![r.disparou()];
    for (t, vida) in [(1u64, 0.7f32), (2, 0.7), (3, 0.9), (4, 0.3)] {
        r.m.entradas
            .jogo
            .set("vida", Escritor::Script, Valor::Numero(vida));
        r.tique(t);
        disparos.push(r.disparou());
    }
    assert_eq!(
        disparos,
        vec![false, true, false, false, true],
        "nascer · desce · fica · sobe · desce"
    );
}

/// ⭐ **E o dano SACODE a galáxia simulada**: as estrelas incham no pico do envelope e voltam; a
/// vida a subir não as mexe (controlo).
#[test]
fn o_dano_sacode_a_galaxia_e_curar_nao() {
    let mut r = Corrida::nova();
    for t in 0..30 {
        r.tique(t);
    }
    let calma = r.tamanho();
    r.m.entradas
        .jogo
        .set("vida", Escritor::Script, Valor::Numero(1.0));
    let mut t = 30;
    let mut pico_a_subir = calma;
    for _ in 0..20 {
        r.tique(t);
        t += 1;
        pico_a_subir = pico_a_subir.max(r.tamanho());
    }
    r.m.entradas
        .jogo
        .set("vida", Escritor::Script, Valor::Numero(0.6));
    let mut pico = 0.0f32;
    for _ in 0..20 {
        r.tique(t);
        t += 1;
        pico = pico.max(r.tamanho());
    }
    for _ in 0..180 {
        r.tique(t);
        t += 1;
    }
    let depois = r.tamanho();
    eprintln!(
        "cena 136: tamanho calma {calma:.4} · a curar {pico_a_subir:.4} · pico do dano {pico:.4} · 3 s depois {depois:.4}"
    );
    assert!(
        (pico_a_subir - calma).abs() < 1e-4,
        "curar mexeu nas estrelas"
    );
    assert!(
        pico > 1.5 * calma,
        "o dano não as inchou: {calma} -> {pico}"
    );
    assert!(
        (depois - calma).abs() < 0.05 * calma,
        "não voltaram à calma: {depois}"
    );
}
