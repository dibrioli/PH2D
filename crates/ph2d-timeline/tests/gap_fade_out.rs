//! **O fade-OUT de uma strip, ANTES de um intervalo (gap) e de uma próxima strip, cruza
//! para a PRÓXIMA — sem loop** (Enio, 2026-07-19).
//!
//! Antes: o fade-out afundava para a pose de REPOUSO, e no intervalo o objeto dava um
//! salto de volta para a pose ANTES do fade (a segurada da strip que acabou), e depois
//! outro salto para a segunda strip. O correto: fadear para a pose da próxima strip, PARAR
//! nela no intervalo, e entrar na próxima sem salto.
//!
//! Poses distintas de propósito: **A = +5**, **B = −4**, **repouso = 0** — cada bug leva a
//! um número diferente, então o oráculo (a POSE) separa "cruzou para B" de "afundou no
//! repouso" de "saltou de volta para A".

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

const FRAME: f64 = 1.0 / 60.0;

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

fn x_at(sim: &mut SimWorld, st: &mut TimelineState, bits: u64, t: f64) -> f64 {
    apply_from_doc(sim.world_mut(), &mut st.doc, t);
    let e = ph2d_ecs::Entity::from_bits(bits);
    f64::from(sim.world().get::<Transform>(e).unwrap().translation.x)
}

struct Scene {
    sim: SimWorld,
    st: TimelineState,
    bits: u64,
}

/// **A** em `[0, 4)`, **gap** `[4, 6)`, **B** em `[6, 10)`. Poses `(início, fim)` de cada
/// clipe + os fades das pontas; sem loop.
fn gap_scene(a_x: (f32, f32), b_x: (f32, f32), a_ease_out: f64, b_ease_in: f64) -> Scene {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Gap")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "A".into());
    key(doc, bits, PropKind::TranslationX, 0.0, a_x.0);
    key(doc, bits, PropKind::TranslationX, 4.0, a_x.1);
    let cb = doc.add_clip("B".into());
    doc.set_active(cb);
    key(doc, bits, PropKind::TranslationX, 0.0, b_x.0);
    key(doc, bits, PropKind::TranslationX, 4.0, b_x.1);
    doc.set_active(0);
    let lane = doc.add_lane("Lane 1".into()).unwrap();
    let a = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
    let b = doc.add_strip(lane, cb, 6.0, 10.0).unwrap(); // GAP [4, 6)
    doc.strip_mut(lane, a).unwrap().ease_out = a_ease_out;
    doc.strip_mut(lane, b).unwrap().ease_in = b_ease_in;
    Scene { sim, st, bits }
}

/// **O fade-out cruza para a PRÓXIMA strip e o intervalo a segura.** A (+5) com fade-out de
/// 1 s; B (−4) sem fade. O objeto vai de +5 até −4 durante o fade, segura −4 no intervalo,
/// e B toca −4 sem salto. Sem o fix: afunda para 0 (repouso) no fade, salta para +5 no
/// intervalo, salta para −4 em B.
#[test]
fn a_fade_out_before_a_gap_crosses_to_the_next_strip_and_holds_it() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = gap_scene((5.0, 5.0), (-4.0, -4.0), 1.0, 0.0);

    let before_gap = x_at(&mut sim, &mut st, bits, 4.0 - FRAME); // fim do fade
    let just_after = x_at(&mut sim, &mut st, bits, 4.0 + FRAME); // logo na fronteira do gap
    let in_gap = x_at(&mut sim, &mut st, bits, 5.0); // meio do intervalo
    let entry = x_at(&mut sim, &mut st, bits, 6.0); // B começa

    // Chegou (quase) à pose da PRÓXIMA (−4), não ao repouso (0) nem parou em +5.
    assert!(
        (before_gap + 4.0).abs() < 0.15,
        "o fade cruza para a próxima strip (−4), não para o repouso (0): {before_gap}"
    );
    // O intervalo SEGURA a próxima (−4), não salta de volta para a anterior (+5).
    assert!(
        (in_gap + 4.0).abs() < 1e-6,
        "o intervalo segura −4 (a próxima), não +5 (a anterior): {in_gap}"
    );
    // Sem salto na fronteira do intervalo…
    assert!(
        (just_after - before_gap).abs() < 0.15,
        "sem salto na entrada do intervalo: {before_gap} -> {just_after}"
    );
    // …e sem salto na entrada da próxima strip.
    assert!(
        (entry + 4.0).abs() < 1e-6 && (entry - in_gap).abs() < 1e-6,
        "entra em B sem salto: intervalo {in_gap} -> B {entry}"
    );
}

/// **Com fade dos DOIS lados a travessia continua sem salto.** A (+5) fade-out; B rampa
/// (−4 → +2) com fade-in. O objeto cruza para −4 (início de B) no fade de A, segura −4 no
/// intervalo, e B **eases a partir do próprio início (−4)** em vez de saltar de volta para
/// +5. É a janela do `fade_out_target` alcançando ATRAVÉS do fade-in da próxima; sem ela,
/// um frame depois do gap o objeto salta para +5.
#[test]
fn both_fades_across_a_gap_are_seamless() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = gap_scene((5.0, 5.0), (-4.0, 2.0), 1.0, 1.0);

    let in_gap = x_at(&mut sim, &mut st, bits, 5.0);
    let b_fade_start = x_at(&mut sim, &mut st, bits, 6.0 + FRAME); // logo dentro do fade-in de B

    assert!(
        (in_gap + 4.0).abs() < 1e-6,
        "o intervalo segura o início de B (−4): {in_gap}"
    );
    // Um frame dentro do fade-in de B o objeto ainda está ~ no início de B (−4), NÃO saltou
    // de volta para +5.
    assert!(
        (b_fade_start + 4.0).abs() < 0.2,
        "B eases a partir do próprio início (−4), sem saltar de volta para +5: {b_fade_start}"
    );
}

/// **CERCA — corte seco não enrola.** A (+5) SEM fade-out; B (−4). No intervalo o objeto
/// segura o FIM de A (+5) e corta para B — o corte que o autor pediu ao não fadear. (Tirar
/// o `blend_out > 0` do `fade_out_target` faria o intervalo segurar −4.)
#[test]
fn a_hard_cut_before_a_gap_still_holds_the_previous_strip() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = gap_scene((5.0, 5.0), (-4.0, -4.0), 0.0, 0.0);

    let in_gap = x_at(&mut sim, &mut st, bits, 5.0);
    assert!(
        (in_gap - 5.0).abs() < 1e-6,
        "sem fade-out, o intervalo segura o fim de A (+5), não a próxima (−4): {in_gap}"
    );
}

/// **Um fade-IN depois de um corte seco continua cruzando da ANTERIOR.** A (+5) sem
/// fade-out; B (−4) com fade-in. B eases a partir do fim de A (+5) — o comportamento de
/// sempre. Guarda que o `fade_out_target` não sequestra um fade-in normal quando a strip
/// anterior NÃO fadeou.
#[test]
fn a_plain_fade_in_after_a_hard_cut_still_crosses_from_the_previous() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = gap_scene((5.0, 5.0), (-4.0, -4.0), 0.0, 1.0);

    // No começo do fade-in de B o objeto vem de +5 (o fim de A), não de −4.
    let b_start = x_at(&mut sim, &mut st, bits, 6.0 + FRAME);
    assert!(
        b_start > 4.0,
        "B eases a partir do fim de A (+5), pois A não fadeou: {b_start}"
    );
}

/// **`lead_out` — o fade-out OUTWARD: o clipe toca INTEIRO e SÓ ENTÃO fadeia, no gap.** A
/// rampa 0 → +5 com `lead_out` de 1 s (janela `[4, 5]`); B chapada (−4). No fim de A o
/// objeto está no ÚLTIMO frame de A (+5) — o clipe tocou inteiro, ao contrário do
/// `ease_out`, que gastaria a cauda no crossfade e já estaria em −4 ali. Depois fadeia
/// +5 → −4 no gap, segura −4, e B toca sem salto.
#[test]
fn a_lead_out_plays_the_clip_fully_then_fades_in_the_gap() {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Gap")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "A".into());
    key(doc, bits, PropKind::TranslationX, 0.0, 0.0);
    key(doc, bits, PropKind::TranslationX, 4.0, 5.0); // A rampa 0 -> +5
    let cb = doc.add_clip("B".into());
    doc.set_active(cb);
    key(doc, bits, PropKind::TranslationX, 0.0, -4.0);
    key(doc, bits, PropKind::TranslationX, 4.0, -4.0);
    doc.set_active(0);
    let lane = doc.add_lane("Lane 1".into()).unwrap();
    let a = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
    doc.add_strip(lane, cb, 6.0, 10.0); // gap [4, 6)
    doc.strip_mut(lane, a).unwrap().lead_out = 1.0; // fade OUTWARD, em [4, 5]

    let at_end = x_at(&mut sim, &mut st, bits, 4.0 - FRAME); // fim de A: último frame
    let mid_lead = x_at(&mut sim, &mut st, bits, 4.5); // meio do lead-out
    let after_lead = x_at(&mut sim, &mut st, bits, 5.5); // gap depois do lead-out
    let entry = x_at(&mut sim, &mut st, bits, 6.0); // B

    // O clipe tocou INTEIRO: no fim de A o objeto está no último frame (~+5), não já em B.
    assert!(
        (at_end - 5.0).abs() < 0.1,
        "lead_out toca o clipe inteiro: fim de A ~ +5, não B (−4): {at_end}"
    );
    // …e SÓ ENTÃO fadeia para B, no gap.
    assert!(
        (-4.0..=5.0).contains(&mid_lead)
            && (mid_lead - 5.0).abs() > 0.5
            && (mid_lead + 4.0).abs() > 0.5,
        "o lead-out fadeia +5 -> −4 no gap: {mid_lead}"
    );
    // Segura −4 no resto do gap e entra em B sem salto.
    assert!(
        (after_lead + 4.0).abs() < 1e-6 && (entry + 4.0).abs() < 1e-6,
        "segura −4 e entra em B sem salto: {after_lead} / {entry}"
    );
}
