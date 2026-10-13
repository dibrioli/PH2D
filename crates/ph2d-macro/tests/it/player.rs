//! O tocador despacha por ordem GLOBAL de tempo, independente de como o tempo lhe chega.

use ph2d_macro::{Event, MacroDoc, MacroPlayer, MacroTime, RefRect, Track};

fn track(family: &str, times_ms: &[u64]) -> Track {
    Track {
        family: family.into(),
        target: 0,
        initial: vec![],
        events: times_ms
            .iter()
            .enumerate()
            .map(|(i, &t)| Event {
                t: MacroTime::from_ms(t),
                payload: vec![i as u8],
            })
            .collect(),
    }
}

/// Duas trilhas entrelaçadas, com um EMPATE em 20 ms e dois passos no mesmo instante em `b`.
fn doc() -> MacroDoc {
    MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![track("a", &[0, 20, 50]), track("b", &[10, 20, 20, 70])],
    }
}

fn run(doc: &MacroDoc, steps_ms: &[u64]) -> Vec<(usize, u8)> {
    let mut p = MacroPlayer::new(doc);
    let mut seen = Vec::new();
    for &t in steps_ms {
        p.advance_to(doc, MacroTime::from_ms(t), |k, _, e| {
            seen.push((k, e.payload[0]));
            Ok::<_, ()>(())
        })
        .unwrap();
    }
    seen
}

const ORDEM: [(usize, u8); 7] = [(0, 0), (1, 0), (0, 1), (1, 1), (1, 2), (0, 2), (1, 3)];

#[test]
fn steps_come_out_in_global_time_order_and_ties_go_to_the_lower_track() {
    assert_eq!(run(&doc(), &[1_000]), ORDEM);
}

#[test]
fn the_order_does_not_depend_on_how_time_is_fed() {
    let quadros: Vec<u64> = (0..=80).collect();
    assert_eq!(run(&doc(), &quadros), ORDEM);
    assert_eq!(run(&doc(), &[5, 5, 19, 20, 49, 71]), ORDEM);
}

#[test]
fn nothing_due_is_dispatched_before_its_time() {
    let d = doc();
    let mut p = MacroPlayer::new(&d);
    let n = p
        .advance_to(&d, MacroTime::from_ms(19), |_, _, _| Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(n, 2);
    assert!(!p.is_finished(&d));
    let n = p
        .advance_to(&d, MacroTime::from_ms(70), |_, _, _| Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(n, 5);
    assert!(p.is_finished(&d));
    assert_eq!(p.dispatched(), 7);
}

#[test]
fn a_failed_step_is_retried_never_skipped() {
    let d = doc();
    let mut p = MacroPlayer::new(&d);
    let mut falha = true;
    let mut seen = Vec::new();
    let r = p.advance_to(&d, MacroTime::from_ms(100), |k, _, e| {
        if (k, e.payload[0]) == (0, 1) && falha {
            falha = false;
            return Err("ainda não");
        }
        seen.push((k, e.payload[0]));
        Ok(())
    });
    assert_eq!(r, Err("ainda não"));
    assert_eq!(seen, ORDEM[..2]);
    p.advance_to(&d, MacroTime::from_ms(100), |k, _, e| {
        seen.push((k, e.payload[0]));
        Ok::<_, &str>(())
    })
    .unwrap();
    assert_eq!(seen, ORDEM);
}

#[test]
fn seek_backwards_resets_every_track_and_replays_to_the_same_point() {
    let d = doc();
    let mut p = MacroPlayer::new(&d);
    let mut log: Vec<String> = Vec::new();
    p.advance_to(&d, MacroTime::from_ms(60), |k, _, e| {
        log.push(format!("{k}{}", e.payload[0]));
        Ok::<_, ()>(())
    })
    .unwrap();
    let log = std::cell::RefCell::new(Vec::new());
    p.seek(
        &d,
        MacroTime::from_ms(20),
        |k, _| {
            log.borrow_mut().push(format!("reset{k}"));
            Ok::<_, ()>(())
        },
        |k, _, e| {
            log.borrow_mut().push(format!("{k}{}", e.payload[0]));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        log.into_inner(),
        ["reset0", "reset1", "00", "10", "01", "11", "12"]
    );
    assert_eq!(p.time(), MacroTime::from_ms(20));
}

#[test]
fn duration_is_the_last_step_of_any_track() {
    assert_eq!(doc().duration(), MacroTime::from_ms(70));
    assert_eq!(
        MacroDoc::new(RefRect { w: 1.0, h: 1.0 }).duration(),
        MacroTime::ZERO
    );
}
