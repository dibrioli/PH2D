//! A facilitação (W7) pelo caminho de quem a usa — `docs/MiroClone/05_facilitacao.md` §2.
//!
//! Números do Miro transcritos: o `+` de votar Ø 49 numa nota de 236 (`vote_03`), centro no canto; o
//! máximo de votos por pessoa 99 (VO); o cronómetro: `+1m`/`+5m` (TM), passo de 1 min até 10 e de 5
//! acima [comm c16256], tecto 99:59 (FigJam FT).

use std::time::{Duration, Instant};

use super::*;
use ph2d_board_model::{Rgba, STICKY_COLORS, Stamp};

impl World {
    /// Uma nota de 199 em `[x, y]` com `text`, da cor `c` de [`STICKY_COLORS`] e com `tags`.
    pub(super) fn sticky(&mut self, at: [f64; 2], text: &str, c: usize, tags: &[&str]) -> ElementId {
        let ink = Rgba([20, 20, 20, 255]);
        let mut el = Element::new_shape(
            self.doc.mint_id(),
            self.doc.z_on_top(),
            Shape {
                kind: ShapeType::Sticky,
                style: Style::new(Some(Rgba(STICKY_COLORS[c])), None, ink),
                text: ph2d_board_model::RichText::plain(text),
            },
            [at[0], at[1], 199.0, 199.0],
        );
        for t in tags {
            el.tags_mut().push(ph2d_board_model::Tag {
                name: (*t).to_owned(),
                color: ink,
            });
        }
        let id = el.id;
        self.h.apply(&mut self.doc, vec![BoardOp::Put(el)]);
        id
    }

    /// O centro do `+` de `id` (mundo).
    fn plus(&self, id: ElementId) -> [f64; 2] {
        let m = self.ed.vote_marks(&self.doc);
        m.iter().find(|m| m.id == id).expect("é votável").plus.0
    }
}

// ─── Cronómetro (F1–F6) ─────────────────────────────────────────────────────────────────────────

/// ⭐ F2: o `±` anda de 1 em 1 até 10 e de 5 em 5 acima; nasce em 5:00; entre 1 e 99.
#[test]
fn the_step_is_one_minute_below_ten_and_five_above() {
    let mut t = Timer::default();
    assert_eq!(t.minutes(), 5);
    let mut seen = vec![t.minutes()];
    for _ in 0..8 {
        t.step(true);
        seen.push(t.minutes());
    }
    assert_eq!(seen, [5, 6, 7, 8, 9, 10, 15, 20, 25]);
    for _ in 0..3 {
        t.step(false);
    }
    assert_eq!(t.minutes(), 10, "a descer, de 5 em 5 até 10");
    t.step(false);
    assert_eq!(t.minutes(), 9);
    for _ in 0..20 {
        t.step(false);
    }
    assert_eq!(t.minutes(), 1, "nunca abaixo de 1");
    for _ in 0..40 {
        t.step(true);
    }
    assert_eq!(t.minutes(), 99, "nunca acima de 99");
}

/// ⭐ F3/F4: `+1m`/`+5m` a correr ou pausado, nunca acima de 99:59; parado não soma.
#[test]
fn adding_time_works_running_or_paused_and_never_passes_99_59() {
    let now = Instant::now();
    let mut t = Timer::default();
    assert!(!t.add(now, Duration::from_secs(60)), "armado não soma (arma-se com ±)");
    t.play(now);
    assert!(t.add(now, Duration::from_secs(60)));
    assert_eq!(t.display(now), (6, 0));
    t.pause(now + Duration::from_secs(30));
    assert!(t.add(now, Duration::from_secs(300)));
    assert_eq!(t.display(now + Duration::from_secs(999)), (10, 30), "pausado não anda");
    for _ in 0..30 {
        t.add(now, Duration::from_secs(300));
    }
    assert_eq!(t.left(now), TIMER_MAX, "o tecto é 99:59");
    assert!(!t.step(true), "a correr ou pausado o ± não mexe");
}

/// ⭐ F5: o ■ volta à duração armada; ▶ retoma do sítio onde pausou.
#[test]
fn stop_resets_and_play_resumes() {
    let now = Instant::now();
    let mut t = Timer::default();
    t.play(now);
    t.pause(now + Duration::from_secs(61));
    t.play(now + Duration::from_secs(500));
    assert_eq!(t.display(now + Duration::from_secs(500)), (3, 59));
    t.confirm_stop = true;
    t.stop();
    assert!(!t.confirm_stop);
    assert_eq!(t.run(), TimerRun::Set);
    assert_eq!(t.display(now), (5, 0));
}

/// ⭐ F6: no zero o relógio fica em 00:00 e a campainha soa UMA vez — e nunca com o altifalante
/// desligado.
#[test]
fn at_zero_it_rings_once_and_stays_at_zero() {
    let now = Instant::now();
    let mut t = Timer::default();
    t.play(now);
    let end = now + Duration::from_secs(300);
    assert!(!t.tick(end - Duration::from_millis(1)));
    assert_eq!(t.display(end - Duration::from_millis(1)), (0, 1), "arredonda para cima");
    assert!(t.tick(end), "soa no zero");
    assert!(!t.tick(end + Duration::from_secs(5)), "e só uma vez");
    assert_eq!(t.display(end + Duration::from_secs(60)), (0, 0));
    assert!(matches!(t.run(), TimerRun::Done { .. }));
    t.play(end + Duration::from_secs(70));
    t.sound = false;
    assert!(!t.tick(end + Duration::from_secs(70 + 300)), "com o altifalante desligado, não soa");
}

/// F4/F6: pausado o mostrador pisca; a correr não.
#[test]
fn a_paused_clock_blinks() {
    let now = Instant::now();
    let mut t = Timer::default();
    t.play(now);
    assert!(!t.blink_off(now + TIMER_BLINK));
    t.pause(now);
    assert!(!t.blink_off(now) && t.blink_off(now + TIMER_BLINK));
}

// ─── Votação (V1–V7) ────────────────────────────────────────────────────────────────────────────

/// ⭐ V1: a sessão vota a selecção (com o que vive nos frames dela) ou o quadro inteiro — só formas e
/// notas; sem nada votável é a recusa.
#[test]
fn a_session_votes_on_the_selection_or_the_board() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    let b = w.sticky([300.0, 0.0], "b", 1, &[]);
    assert!(w.ed.start_vote(&w.doc, VoteSetup::default()));
    assert_eq!(w.ed.vote().unwrap().targets().len(), 2, "sem selecção, o quadro inteiro");
    w.ed.vote = None;
    w.ed.select(&w.doc, [a]);
    assert!(w.ed.start_vote(&w.doc, VoteSetup::default()));
    assert_eq!(*w.ed.vote().unwrap().targets(), [a].into());
    assert!(w.ed.selection().is_empty(), "a sessão não deixa nada seleccionado");
    let _ = b;
    let mut empty = world();
    assert!(!empty.ed.start_vote(&empty.doc, VoteSetup::default()));
    assert_eq!(empty.ed.take_notice(), Some(Notice::NothingToVote));
}

/// ⭐ V2: o `+` mede 0,21 do lado com o centro no canto; clicar dá um voto, o `−` tira-o; sem votos é
/// a recusa.
#[test]
fn a_click_on_the_plus_casts_one_vote() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    w.ed.start_vote(&w.doc, VoteSetup { votes: 2, ..VoteSetup::default() });
    let p = w.plus(a);
    assert_eq!(p, [199.0, 199.0]);
    w.click(p, NONE);
    w.click(p, NONE);
    let s = w.ed.vote().unwrap();
    assert_eq!((s.mine(a), s.left()), (2, 0));
    w.click(p, NONE);
    assert_eq!(w.ed.take_notice(), Some(Notice::NoVotesLeft));
    let minus = w.ed.vote_marks(&w.doc)[0].minus.expect("com votos há o −").0;
    w.click(minus, NONE);
    assert_eq!(w.ed.vote().unwrap().mine(a), 1);
    assert!(w.ed.selection().is_empty(), "votar não selecciona");
    assert_eq!(w.doc.get(a).unwrap().votes(), 0, "a sessão não escreve no documento");
}

/// ⭐ V2: com «One vote per object», o 2.º voto no mesmo objecto é a recusa.
#[test]
fn one_vote_per_object_refuses_the_second() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    w.ed.start_vote(&w.doc, VoteSetup { one_per_object: true, ..VoteSetup::default() });
    let p = w.plus(a);
    w.click(p, NONE);
    w.click(p, NONE);
    assert_eq!(w.ed.take_notice(), Some(Notice::OneVotePerObject));
    assert_eq!(w.ed.vote().unwrap().mine(a), 1);
}

/// ⭐ V3: «I'm done» esconde os votos da pessoa e a seguinte começa do zero.
#[test]
fn the_next_voter_sees_none_of_the_previous() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    w.ed.start_vote(&w.doc, VoteSetup::default());
    let p = w.plus(a);
    w.click(p, NONE);
    assert!(w.ed.next_voter());
    let s = w.ed.vote().unwrap();
    assert_eq!((s.mine(a), s.left(), s.voters()), (0, 3, 1));
    assert!(w.ed.vote_marks(&w.doc)[0].minus.is_none());
}

/// ⭐ V4: o fim escreve os totais (de todos os turnos) num passo, e tira o resultado anterior.
#[test]
fn ending_writes_the_totals_in_one_step() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    let b = w.sticky([300.0, 0.0], "b", 1, &[]);
    let mut old = w.doc.get(b).unwrap().clone();
    old.set_votes(9);
    w.h.apply(&mut w.doc, vec![BoardOp::Put(old)]);
    w.ed.start_vote(&w.doc, VoteSetup::default());
    let p = w.plus(a);
    w.click(p, NONE);
    w.click(p, NONE);
    w.ed.next_voter();
    w.click(p, NONE);
    assert!(w.ed.end_vote(&mut w.doc, &mut w.h));
    assert!(w.ed.vote().is_none());
    assert_eq!((w.doc.get(a).unwrap().votes(), w.doc.get(b).unwrap().votes()), (3, 0));
    assert_eq!(vote_results(&w.doc), [(a, 3)]);
    assert!(w.cmd(Command::Undo));
    assert_eq!((w.doc.get(a).unwrap().votes(), w.doc.get(b).unwrap().votes()), (0, 9));
}

/// V5: os selos vão do menor ao maior total do quadro.
#[test]
fn the_range_of_the_badges_is_the_boards() {
    let mut w = world();
    assert_eq!(vote_range(&w.doc), None);
    for (i, n) in [2, 5, 0].into_iter().enumerate() {
        let id = w.sticky([300.0 * i as f64, 0.0], "x", 1, &[]);
        let mut el = w.doc.get(id).unwrap().clone();
        el.set_votes(n);
        w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    }
    assert_eq!(vote_range(&w.doc), Some([2, 5]));
    assert_eq!(vote_results(&w.doc).iter().map(|r| r.1).collect::<Vec<_>>(), [5, 2]);
}

/// ⭐ V7: uma cópia não leva os votos; «Clear results» tira-os num passo.
#[test]
fn a_copy_carries_no_votes_and_clear_takes_them_off() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    let mut el = w.doc.get(a).unwrap().clone();
    el.set_votes(4);
    el.add_stamp(Stamp::Heart, 2);
    w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    w.ed.select(&w.doc, [a]);
    assert!(w.cmd(Command::Duplicate));
    let copy = *w.ed.selection().iter().next().unwrap();
    assert_ne!(copy, a);
    assert_eq!(w.doc.get(copy).unwrap().votes(), 0, "V7: os votos não vão");
    assert_eq!(w.doc.get(copy).unwrap().stamps()[0].count, 2, "S4: os carimbos vão");
    assert!(w.ed.clear_votes(&mut w.doc, &mut w.h));
    assert_eq!(w.doc.get(a).unwrap().votes(), 0);
    assert!(!w.ed.clear_votes(&mut w.doc, &mut w.h));
}

// ─── Carimbos (S2–S3) ───────────────────────────────────────────────────────────────────────────

/// ⭐ S2: carimbar a selecção dá +1 a cada NOTA (as formas e as travadas não), num passo.
#[test]
fn stamping_the_selection_is_one_step() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    let b = w.sticky([300.0, 0.0], "b", 1, &[]);
    let r = w.rect([600.0, 0.0, 100.0, 100.0]);
    w.ed.select(&w.doc, [a, b, r]);
    assert!(w.ed.stamp(&mut w.doc, &mut w.h, Stamp::Star));
    assert!(w.ed.stamp(&mut w.doc, &mut w.h, Stamp::Star));
    assert_eq!(w.doc.get(a).unwrap().stamps()[0].count, 2);
    assert_eq!(w.doc.get(b).unwrap().stamps()[0].count, 2);
    assert!(w.doc.get(r).unwrap().stamps().is_empty(), "uma forma não leva carimbos");
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.get(a).unwrap().stamps()[0].count, 1, "cada carimbar é UM passo");
}

/// ⭐ S3: clicar numa ficha dá +1, `Shift`+clicar tira um; a 0 a ficha sai — e o clique não
/// selecciona nem começa um laço.
#[test]
fn shift_click_takes_one_back() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    w.ed.select(&w.doc, [a]);
    w.ed.stamp(&mut w.doc, &mut w.h, Stamp::Heart);
    w.ed.select(&w.doc, []);
    let el = w.doc.get(a).unwrap().clone();
    let r = ph2d_board_model::stamp_chips(&el)[0].1;
    let c = [el.x + (r[0] + r[2]) / 2.0, el.y + (r[1] + r[3]) / 2.0];
    assert_eq!(chip_at(&w.doc, c), Some((a, Stamp::Heart)));
    w.click(c, NONE);
    assert_eq!(w.doc.get(a).unwrap().stamps()[0].count, 2);
    assert!(w.ed.selection().is_empty());
    w.click(c, SHIFT);
    w.click(c, SHIFT);
    assert!(w.doc.get(a).unwrap().stamps().is_empty(), "a 0 sai");
}
