//! W12 — a convergência AO ACASO com o modo privado ligado: edições, quedas e voltas, e nenhum texto
//! selado de outro em nada que cada um ouviu. Módulo filho de `fac_tests` (o harness `Room` é de lá).

use super::*;

/// ⭐ Convergência com o modo privado: anfitrião + 3, edições de texto e de posição ao acaso com o
/// modo ligado, quedas e voltas — e nenhum texto selado de outro em nada que cada um ouviu; depois
/// da revelação, os quatro com o mesmo quadro.
#[test]
fn a_private_room_with_drops_ends_with_one_board_and_no_early_text() {
    for seed in 1..=12u64 {
        let mut s = seed * 0x9e37_79b9 + 7;
        let mut rnd = |n: u64| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s % n
        };
        let mut hb = board();
        let ids: Vec<ElementId> = (0..4)
            .map(|i| note(&mut hb, "", f64::from(i) * 20.0))
            .collect();
        let mut r = Room::new(hb, &["p0", "p1", "p2"]);
        r.ask(0, private_on());
        let mut written: Vec<(usize, String)> = Vec::new();
        for step in 0..120u64 {
            let who = rnd(4) as usize;
            let id = ids[rnd(4) as usize];
            let secret = format!("s{seed}x{step}w{who}");
            match rnd(3) {
                0 => {
                    if who == 3 {
                        write(&mut r.hb, id, &secret);
                    } else if r.peers[who].online {
                        write(r.b(who), id, &secret);
                    } else {
                        continue;
                    }
                    written.push((who, secret));
                }
                1 if who < 3 => {
                    if r.peers[who].online {
                        r.drop(who);
                    } else {
                        r.back(who);
                    }
                }
                _ => {
                    let b = if who == 3 || !r.peers[who].online {
                        &mut r.hb
                    } else {
                        r.b(who)
                    };
                    let mut el = b.doc.get(id).unwrap().clone();
                    el.translate([1.0, 0.0]);
                    let _ = BoardOp::Put(el).apply(&mut b.doc);
                }
            }
            r.at(step);
        }
        for i in 0..3 {
            if !r.peers[i].online {
                r.back(i);
            }
        }
        r.at(200);
        for (author, secret) in &written {
            for i in (0..3).filter(|i| i != author) {
                assert!(!r.heard(i, secret), "semente {seed}: {i} ouviu {secret}");
            }
        }
        r.ask(0, Ask::PrivateOff);
        r.at(200 + REVEAL_COUNTDOWN_MS);
        r.all_same();
        assert!(r.fac(0).hidden.is_empty());
    }
}
