//! As defesas contra um par que manda lixo: o carimbo absurdo (o relógio não transborda e o que
//! escrevemos depois continua a ganhar) e as escritas para o que nunca chega (a fila tem tecto).

use ph2d_board_sync::{Lamport, MAX_STAMP};
use ph2d_kanban_model::KanbanId;

use crate::msg::{KWrite, Target};
use crate::test_support::{Net, doc, edit_card, kanban};

/// Um par manda as suas escritas com `at.n = u64::MAX`: são recusadas, o meu relógio não sobe, e a
/// minha edição seguinte sai com um carimbo normal (antes: transbordava — pânico em debug, zero em
/// release, e tudo o que eu escrevesse depois perdia para sempre).
#[test]
fn a_stamp_from_the_far_future_is_refused_and_the_clock_never_overflows() {
    let mut net = Net::new(2);
    let c = *kanban(&net.sets[1]).cards.keys().next().expect("um cartão");
    edit_card(&mut net.sets[1], c, |x| x.title = "forjado".into());
    let mut ws = net.reps[1].local(&mut net.sets[1]);
    assert!(!ws.is_empty());
    for w in &mut ws {
        match w {
            KWrite::Born { at, .. } | KWrite::Set { at, .. } | KWrite::Elem { at, .. } => {
                at.n = u64::MAX;
            }
            _ => {}
        }
    }
    let before = kanban(&net.sets[0]).cards[&c].title.clone();
    net.reps[0].receive(&mut net.sets[0], &ws);
    assert_eq!(
        kanban(&net.sets[0]).cards[&c].title,
        before,
        "o forjado não entra"
    );
    assert!(net.reps[0].clock.n <= MAX_STAMP, "o relógio não salta");
    edit_card(&mut net.sets[0], c, |x| x.title = "meu".into());
    let mine = net.reps[0].local(&mut net.sets[0]);
    assert!(mine.iter().all(|w| w.at().is_none_or(|a| a.n <= MAX_STAMP)));
    // No limite do tecto, o relógio satura em vez de transbordar.
    net.reps[0].clock.n = u64::MAX;
    let _ = net.reps[0].clock.tick();
    let ok = Lamport {
        n: MAX_STAMP,
        peer: 9,
    };
    net.reps[0].clock.see(ok);
}

/// Escritas para um Kanban que nunca chega não crescem sem fim: a fila guarda as mais novas até
/// ao tecto e deita fora as mais velhas.
#[test]
fn writes_for_a_kanban_that_never_arrives_are_capped() {
    let mut net = Net::new(1);
    let ghost = KanbanId(999);
    let n = crate::replica::PENDING_MAX + 500;
    let ws: Vec<KWrite> = (1..=n as u64)
        .map(|i| KWrite::Set {
            kanban: ghost,
            target: Target::Header,
            field: 0,
            value: vec![1],
            at: Lamport { n: i, peer: 7 },
        })
        .collect();
    for chunk in ws.chunks(1000) {
        net.reps[0].receive(&mut net.sets[0], chunk);
    }
    assert_eq!(net.reps[0].pending(), crate::replica::PENDING_MAX);
    let oldest = net.reps[0]
        .pending
        .iter()
        .filter_map(KWrite::at)
        .map(|a| a.n)
        .min();
    assert_eq!(oldest, Some(501), "ficam as mais novas");
    let _ = doc(&mut net.sets[0]);
}

/// MEDIÇÃO (J4, `MEDICOES.md`): o custo de um `receive` vazio com P escritas à espera — intercalado,
/// o mínimo. `cargo test --profile smoke -p ph2d-kanban-sync measure_pending -- --ignored --nocapture`.
#[test]
#[ignore = "medição"]
fn measure_pending_cost() {
    use std::time::Instant;
    let sizes = [1024usize, 4096, 16384, 65536];
    let mut best = [f64::MAX; 4];
    let mut nets: Vec<Net> = sizes
        .iter()
        .map(|&p| {
            let mut net = Net::new(1);
            net.reps[0].pending = (1..=p as u64)
                .map(|i| KWrite::Set {
                    kanban: KanbanId(999),
                    target: Target::Header,
                    field: 0,
                    value: vec![1],
                    at: Lamport { n: i, peer: 7 },
                })
                .collect();
            net
        })
        .collect();
    for _round in 0..30 {
        for (i, net) in nets.iter_mut().enumerate() {
            let t = Instant::now();
            net.reps[0].receive(&mut net.sets[0], &[]);
            best[i] = best[i].min(t.elapsed().as_secs_f64() * 1e3);
        }
    }
    for (p, b) in sizes.iter().zip(best) {
        println!("PENDING {p:>6}: {b:.3} ms");
    }
}
