//! What the signal smoke AUTHORS is testable without a window: the Mover's track and
//! the marker/signal pattern. (The fire itself is gated in
//! `render_loop::timeline_bridge::signal_tests`; the crossing law in `ph2d-timeline`.)

use super::{author_mover, smoke_level};
use ph2d_anim::RationalTime;
use ph2d_timeline::{PropKind, TimelineDoc, signals_crossed};

#[test]
fn a_typo_falls_back_to_the_approved_scene_never_to_the_new_one() {
    let lvl = |s: &str| smoke_level(std::ffi::OsStr::new(s));
    assert_eq!(lvl("1"), 1);
    assert_eq!(lvl("2"), 2);
    assert_eq!(lvl(" 2 "), 2, "espaço não muda a cena");
    // ⚠️ O default de um smoke é a cena que JÁ EXISTIA. `=sim`, `=true`, `=0` — todos caem em 1;
    // se caíssem na mais nova, um erro de digitação promoveria a demo não-aprovada.
    for junk in ["sim", "true", "", "0", "-3", "dois"] {
        assert_eq!(lvl(junk), 1, "`{junk}` tem de cair na cena aprovada");
    }
}

/// **A `=2` arma as DUAS metades — e a metade da física precisa do RELÓGIO.**
///
/// A cena chama a `build_signal_scene` da cena 73 (gateada lá) e liga o
/// `TimelineFlags::simulate_physics`, que nasce DESMARCADO (W4b). Sem esse flag o solver não
/// roda: as bolas ficam paradas no ar, nenhum sinal de física dispara, e o smoke demonstra
/// metade da wave **parecendo inteiro** — que é exatamente a classe de cena que este repo já
/// pagou ("a cena afirma o que a medição desmente").
///
/// O gate lê o FONTE porque a função exige janela e device.
#[test]
fn the_second_scene_builds_both_halves_and_gives_physics_a_clock() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/signal_smoke.rs");
    let text = std::fs::read_to_string(&src).expect("read signal_smoke.rs");
    let branch = text
        .find("if level >= 2 {")
        .expect("o ramo da cena 2 sumiu — este gate não está lendo o que pensa que lê");
    for (needle, why) in [
        (
            "build_signal_scene(",
            "a metade da FÍSICA (a porta e o sino da cena 73)",
        ),
        (
            "simulate_physics = true",
            "o RELÓGIO da física, que nasce desmarcado (W4b)",
        ),
    ] {
        assert!(
            text[branch..].contains(needle),
            "a cena `=2` não arma {why}: `{needle}` não aparece no ramo dela.\n\
             Sem isso a cena mostra a metade da timeline e PARECE completa."
        );
    }
}

#[test]
fn the_mover_is_keyed_on_x() {
    let mut doc = TimelineDoc::new();
    author_mover(&mut doc, 7);
    let n = doc
        .binding_for(7, PropKind::TranslationX)
        .and_then(|b| doc.active_clip().track(b.target))
        .map_or(0, |t| t.keys().len());
    assert_eq!(n, 2, "X keyed at both ends so there is motion to sync to");
}

#[test]
fn the_scene_fires_exactly_the_two_signals_over_one_pass() {
    // Mirror the smoke's markers: two that emit, one pure annotation.
    let mut doc = TimelineDoc::new();
    let a = doc.add_marker(RationalTime::from_seconds(1.0), "step");
    doc.set_marker_signal(a, Some("footstep".to_string()));
    let b = doc.add_marker(RationalTime::from_seconds(2.5), "beat");
    doc.set_marker_signal(b, Some("beat".to_string()));
    doc.add_marker(RationalTime::from_seconds(3.5), "chapter"); // no signal
    // One forward pass over the loop crosses both signals, in time order, and never
    // the plain annotation — the smoke's whole premise, made executable.
    assert_eq!(
        signals_crossed(doc.markers(), 0.0, 4.0, Some((0.0, 4.0))),
        ["footstep", "beat"],
        "the pass fires the two signals, not the annotation"
    );
}
