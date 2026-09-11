//! **Sob um LOOP a faixa é CÍCLICA** — o fade-in da primeira strip cruza a partir do
//! que o fim do loop deixa asserido, não a partir da pose de repouso (Enio, 2026-07-16:
//! *"o fade não consegue fazer a transição corretamente (a começar do fim da segunda
//! strip) mas dá um salto"*).
//!
//! O oráculo é a **POSE**, e especificamente o SALTO: a distância que o objeto percorre
//! num único frame na volta do loop. Um teste que afirmasse "o `hold_at` devolve a
//! última strip" ficaria verde sobre uma pose que ainda pula — a regra não é o que o
//! artista vê.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// A cena do relato, em números redondos: **Main** em `[0, 4)` com **fade-in de 1 s**
/// (x = −3 o tempo todo) e **Clip 2** em `[4, 8)` (x = +5 o tempo todo). Sem
/// sobreposição — o fade da primeira é dela, autorado, como na foto.
fn scene() -> (SimWorld, TimelineState, u64) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Loop")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Main".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 4.0, -3.0);
    let c2 = doc.add_clip("Clip 2".into());
    doc.set_active(c2);
    key(doc, bits, PropKind::TranslationX, 0.0, 5.0);
    key(doc, bits, PropKind::TranslationX, 4.0, 5.0);
    doc.set_active(0);
    let lane = doc.add_lane("Lane 1".into()).unwrap();
    let main = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
    doc.add_strip(lane, c2, 4.0, 8.0);
    doc.strip_mut(lane, main).unwrap().ease_in = 1.0; // o fade que o Enio criou
    (sim, st, bits)
}

fn x_at(sim: &mut SimWorld, st: &mut TimelineState, bits: u64, t: f64) -> f64 {
    apply_from_doc(sim.world_mut(), &mut st.doc, t);
    let e = ph2d_ecs::Entity::from_bits(bits);
    f64::from(sim.world().get::<Transform>(e).unwrap().translation.x)
}

/// **O salto na volta do loop.** Um frame antes do fim do loop o objeto está na pose de
/// Clip 2 (+5); um frame depois da volta, no topo do fade da Main, tem de estar
/// *praticamente lá ainda* — o fade começa DE onde o objeto está. Sem o wrap ele
/// aparecia na pose de repouso (0) e o passo de um frame media ~5 unidades.
#[test]
fn under_a_loop_the_first_fade_crosses_from_where_the_last_strip_left_the_object() {
    let (mut sim, mut st, bits) = scene();
    st.doc.set_active_loop_for(false, Some((0.0, 8.0)));
    let frame = 1.0 / 60.0;

    let before_wrap = x_at(&mut sim, &mut st, bits, 8.0 - frame);
    let after_wrap = x_at(&mut sim, &mut st, bits, 0.0);
    let jump = (after_wrap - before_wrap).abs();

    assert!(
        (before_wrap - 5.0).abs() < 1e-6,
        "a última strip termina em +5: {before_wrap}"
    );
    // Um frame de fade de 1 s move ~0,1% de 8 unidades de percurso; qualquer coisa
    // acima de meia unidade é o salto, não a animação.
    assert!(
        jump < 0.5,
        "a volta do loop tem de ser contínua — o fade cruza a partir de +5, \
         não da pose de repouso: {before_wrap} -> {after_wrap} (salto {jump})"
    );
}

/// …e o fade de fato ANDA: contínuo não pode significar congelado. No meio da janela de
/// fade o objeto está entre as duas poses, e no fim dela chegou na Main.
#[test]
fn the_wrapped_fade_still_travels_from_the_last_pose_to_the_first() {
    let (mut sim, mut st, bits) = scene();
    st.doc.set_active_loop_for(false, Some((0.0, 8.0)));

    let mid = x_at(&mut sim, &mut st, bits, 0.5);
    let done = x_at(&mut sim, &mut st, bits, 1.0);
    assert!(
        (-3.0..=5.0).contains(&mid) && (mid - 5.0).abs() > 0.5 && (mid + 3.0).abs() > 0.5,
        "no meio do fade o objeto está ENTRE +5 e −3: {mid}"
    );
    assert!(
        (done + 3.0).abs() < 1e-6,
        "no fim da janela ele chegou na pose da Main: {done}"
    );
}

/// **Sem loop, o topo da timeline continua entrando a partir do REPOUSO.** É a cerca de
/// Chesterton do `hold_at` (*"fading in from the rest pose at the top of a timeline is a
/// real thing to want"*) — e é ela que dá sentido ao gate acima: sem este par, "sempre
/// enrole" passaria.
#[test]
fn with_no_loop_the_top_of_the_timeline_still_fades_in_from_rest() {
    let (mut sim, mut st, bits) = scene();
    // sem `set_active_loop_for` — o loop está desarmado
    let top = x_at(&mut sim, &mut st, bits, 0.0);
    assert!(
        top.abs() < 1e-6,
        "sem loop não há nada atrás da primeira strip: {top}"
    );
}

/// **O wrap só vale DENTRO do laço.** Um loop que fecha antes do fim da timeline não
/// autoriza a primeira strip a puxar a pose da última: fora do intervalo do loop a
/// timeline é linear e o topo entra do repouso.
#[test]
fn a_loop_that_does_not_bracket_the_top_does_not_wrap_it() {
    let (mut sim, mut st, bits) = scene();
    st.doc.set_active_loop_for(false, Some((4.0, 8.0))); // só a segunda metade
    let top = x_at(&mut sim, &mut st, bits, 0.0);
    assert!(
        top.abs() < 1e-6,
        "t=0 está fora do loop: nada a enrolar: {top}"
    );
}

/// **Uma strip que ATRAVESSA o fim do loop é segurada NO instante da volta**, não no fim
/// dela — frames que o loop nunca alcança não podem decidir a pose de onde o fade parte.
#[test]
fn a_strip_straddling_the_loop_end_is_held_at_the_wrap_not_at_its_own_end() {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Loop")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Main".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 4.0, -3.0);
    // Clip 2 RAMPA de 0 a 8 ao longo dos seus 4 s, então "onde ele está" é legível: a
    // metade é 4, o fim é 8.
    let c2 = doc.add_clip("Clip 2".into());
    doc.set_active(c2);
    key(doc, bits, PropKind::TranslationX, 0.0, 0.0);
    key(doc, bits, PropKind::TranslationX, 4.0, 8.0);
    doc.set_active(0);
    let lane = doc.add_lane("Lane 1".into()).unwrap();
    let main = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
    doc.add_strip(lane, c2, 4.0, 8.0);
    doc.strip_mut(lane, main).unwrap().ease_in = 1.0;
    // O loop fecha NO MEIO da Clip 2: a volta acontece quando ela está em x = 4.
    doc.set_active_loop_for(false, Some((0.0, 6.0)));

    let before = x_at(&mut sim, &mut st, bits, 6.0 - 1.0 / 60.0);
    let after = x_at(&mut sim, &mut st, bits, 0.0);
    assert!(
        (before - 4.0).abs() < 0.2,
        "no fim do loop a Clip 2 está na metade da rampa: {before}"
    );
    assert!(
        (after - before).abs() < 0.5,
        "e o topo parte DALI (4), não do fim dela (8) nem do repouso (0): \
         {before} -> {after}"
    );
}
