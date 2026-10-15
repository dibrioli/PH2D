//! **Sonda (ignorada, W5a): editar uma ETAPA de uma macro do Motion** — o valor de um param gravado, e
//! uma espera antes de um gesto (o relógio da cena continua a correr nela) — nos dois modos, sobre a
//! macro da cena `=2`. Medição, nunca barra. Plano: `docs/Macros/02_plano_W5.md`.
//!
//! Corre: `bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib -- --ignored
//! --nocapture macro_track::etapas_sonda`.

use super::em_cena::MacroEmCena;
use super::gate_tests::retrato;
use super::passo::{IntencaoDeParam, Passo};
use super::smoke::grava_o_tutorial;
use super::tocar::Modo;
use crate::motion_mundo::MundoDaCena;
use ph2d_macro::{Event, MacroDoc, MacroTime};
use std::time::Instant;

const DT_US: u64 = 16_667;

fn passo(e: &Event) -> Passo {
    Passo::descodifica(&e.payload).expect("carga lê-se")
}

/// O valor do `qual`-ésimo `SetParam` gravado, vezes `f`.
fn edita_valor(doc: &MacroDoc, qual: usize, f: f64) -> (MacroDoc, String) {
    let mut d = doc.clone();
    let mut n = 0;
    for e in &mut d.tracks[0].events {
        if let Passo::Params {
            destino,
            mut intencoes,
        } = passo(e)
        {
            for i in &mut intencoes {
                if let IntencaoDeParam::SetParam { param, value, .. } = i {
                    if n == qual {
                        let o = format!("{param} {value} → {}", *value * f);
                        *value *= f;
                        e.payload = Passo::Params { destino, intencoes }.codifica();
                        return (d, o);
                    }
                    n += 1;
                }
            }
        }
    }
    panic!("a macro tem menos de {qual} SetParam");
}

/// `q` quadros de espera antes do `gesto`-ésimo lote do grafo: o relógio da cena continua (quadros
/// sintetizados na cadência do último gravado) e tudo o que vem depois anda `q` quadros.
fn espera(doc: &MacroDoc, gesto: usize, q: u64) -> (MacroDoc, f64) {
    let mut d = doc.clone();
    let ev = &mut d.tracks[0].events;
    let gi = (0..ev.len())
        .filter(|&i| matches!(passo(&ev[i]), Passo::Grafo { .. }))
        .nth(gesto)
        .expect("o gesto existe");
    let ins = (0..=gi)
        .find(|&i| ev[i].t == ev[gi].t)
        .expect("o quadro do gesto");
    let Some((tempo, a_tocar, dt)) = (0..ins).rev().find_map(|i| match passo(&ev[i]) {
        Passo::Quadro {
            tempo, a_tocar, dt, ..
        } => Some((tempo, a_tocar, dt)),
        _ => None,
    }) else {
        panic!("há um quadro antes do gesto");
    };
    let t0 = ev[ins].t.0;
    #[expect(clippy::cast_precision_loss, reason = "poucos quadros")]
    let delta = q as f64 * dt;
    for e in &mut ev[ins..] {
        e.t = MacroTime(e.t.0 + q * DT_US);
        if let Passo::Quadro {
            tempo,
            a_tocar,
            dt,
            sujou,
        } = passo(e)
        {
            e.payload = Passo::Quadro {
                tempo: tempo + delta,
                a_tocar,
                dt,
                sujou,
            }
            .codifica();
        }
    }
    let novos: Vec<Event> = (0..q)
        .map(|j| Event {
            t: MacroTime(t0 + j * DT_US),
            payload: Passo::Quadro {
                #[expect(clippy::cast_precision_loss, reason = "poucos quadros")]
                tempo: tempo + (j + 1) as f64 * dt,
                a_tocar,
                dt,
                sujou: false,
            }
            .codifica(),
        })
        .collect();
    ev.splice(ins..ins, novos);
    #[expect(clippy::cast_precision_loss, reason = "µs de uma macro")]
    let antes = t0 as f64 / 1e6;
    (d, antes)
}

fn toca(doc: &MacroDoc, modo: Modo, ate: f64) -> (Option<String>, Vec<u32>, f64) {
    let bytes = ph2d_macro::codec::write(doc);
    let mut m = MacroEmCena::de(&bytes, modo)
        .expect("lê-se")
        .expect("tem trilha do Motion");
    let a = Instant::now();
    let r = m.segue(ate, &MundoDaCena::default(), None);
    let ms = a.elapsed().as_secs_f64() * 1e3;
    (r.err().map(|e| format!("{e:?}")), retrato(m.palco()), ms)
}

#[test]
#[ignore = "sonda: medição (W5a) — imprime, não reprova"]
fn sonda_etapas_editadas_do_motion() {
    eprintln!(
        "[etapas motion] loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let (bytes, d) = grava_o_tutorial(7).expect("grava");
    let doc = ph2d_macro::codec::read(&bytes).expect("lê-se");
    let grafos = doc.tracks[0]
        .events
        .iter()
        .filter(|e| matches!(passo(e), Passo::Grafo { .. }))
        .count();
    eprintln!(
        "[etapas motion] {} eventos, {grafos} lotes do grafo, {d:.2} s",
        doc.tracks[0].events.len()
    );
    for modo in [Modo::Fiel, Modo::Explicativo] {
        let (e0, r0, ms0) = toca(&doc, modo, d);
        eprintln!("[etapas motion] {modo:?} original: erro {e0:?} · {ms0:.0} ms do zero ao fim");
        for qual in [0, 3] {
            let (ed, o) = edita_valor(&doc, qual, 0.5);
            let (e, r, _) = toca(&ed, modo, d);
            eprintln!(
                "[etapas motion] {modo:?} valor ({o}): erro {e:?} · retrato ≠ original: {}",
                r != r0
            );
        }
        for gesto in [6, 15] {
            let (ed, antes) = espera(&doc, gesto, 60);
            let (e, r, _) = toca(&ed, modo, d + 1.0);
            let (_, pa, _) = toca(&ed, modo, antes - 0.001);
            let (_, po, _) = toca(&doc, modo, antes - 0.001);
            eprintln!(
                "[etapas motion] {modo:?} espera 1 s antes do gesto {gesto} (aos {antes:.2} s): erro {e:?} · \
                 antes dela = original: {} · fim ≠ original: {}",
                pa == po,
                r != r0
            );
        }
    }
}
