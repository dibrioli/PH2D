//! **Sob um LOOP a faixa é CÍCLICA — o OUTRO lado da costura.** O fade-in da primeira
//! strip já cruza a partir do fim da última (`loop_wrap.rs`, d326ff28). Este arquivo é o
//! ESPELHO: o fade-OUT da última strip cruza para o INÍCIO da primeira (Enio, 2026-07-19:
//! *"o fade do fim da segunda strip … ainda não consegue fazer a transição suave para o
//! início da primeira strip e assim dá um salto"*).
//!
//! O oráculo é a **POSE**, e o SALTO na volta — nunca a regra. Um teste que afirmasse "o
//! `hold_at` devolve a primeira strip" ficaria verde sobre uma pose que ainda pula.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

/// Um frame a 60 fps — a distância em que um salto de loop é medido.
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

/// **Main** em `[0, 4)` e **Clip 2** em `[4, 8)`, um sobre o outro na Lane 1. Cada teste
/// escolhe as poses `(início, fim)` de cada clipe e os fades das pontas — a cena é a foto
/// que ele precisa provar.
fn two_strips(
    main_x: (f32, f32),
    clip2_x: (f32, f32),
    main_ease_in: f64,
    clip2_ease_out: f64,
) -> Scene {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Loop")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Main".into());
    key(doc, bits, PropKind::TranslationX, 0.0, main_x.0);
    key(doc, bits, PropKind::TranslationX, 4.0, main_x.1);
    let c2 = doc.add_clip("Clip 2".into());
    doc.set_active(c2);
    key(doc, bits, PropKind::TranslationX, 0.0, clip2_x.0);
    key(doc, bits, PropKind::TranslationX, 4.0, clip2_x.1);
    doc.set_active(0);
    let lane = doc.add_lane("Lane 1".into()).unwrap();
    let main = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
    let clip2 = doc.add_strip(lane, c2, 4.0, 8.0).unwrap();
    doc.strip_mut(lane, main).unwrap().ease_in = main_ease_in;
    doc.strip_mut(lane, clip2).unwrap().ease_out = clip2_ease_out;
    Scene { sim, st, bits }
}

/// **O salto na volta, agora pelo fade de SAÍDA.** A Main anima (−3 → +2), sem fade-in; a
/// Clip 2 é chapada (+5) com fade-out de 1 s. Sob o loop, o fade-out da Clip 2 tem de
/// chegar ao **início** da Main (−3) — a pose em que o loop reinicia —, não ao **fim**
/// dela (+2), que era o que o `hold_at` segurava (a strip que acabou antes). Sem o wrap
/// de saída o passo de um frame na volta media ~5 unidades.
#[test]
fn under_a_loop_the_last_fade_out_crosses_to_the_first_strips_start() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = two_strips((-3.0, 2.0), (5.0, 5.0), 0.0, 1.0);
    st.doc.set_active_loop_for(false, Some((0.0, 8.0)));

    let before_wrap = x_at(&mut sim, &mut st, bits, 8.0 - FRAME);
    let after_wrap = x_at(&mut sim, &mut st, bits, 0.0);

    // O fade-out chegou ao INÍCIO da Main (−3), não ao fim dela (+2).
    assert!(
        (before_wrap + 3.0).abs() < 0.1,
        "o fade-out cruza para o início da primeira strip (−3), não o fim (+2): {before_wrap}"
    );
    // …e a volta é contínua.
    let jump = (after_wrap - before_wrap).abs();
    assert!(
        jump < 0.5,
        "a volta tem de ser suave: {before_wrap} -> {after_wrap} (salto {jump})"
    );
}

/// …e o fade-out de fato ANDA: contínuo não pode virar congelado. No meio da janela o
/// objeto está ENTRE a pose da Clip 2 (+5) e o início da Main (−3).
#[test]
fn the_wrapped_fade_out_travels_from_the_last_pose_to_the_head() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = two_strips((-3.0, -3.0), (5.0, 5.0), 0.0, 1.0);
    st.doc.set_active_loop_for(false, Some((0.0, 8.0)));

    let full = x_at(&mut sim, &mut st, bits, 7.0); // início do fade: ainda +5
    let mid = x_at(&mut sim, &mut st, bits, 7.5); // meio: entre +5 e −3
    let done = x_at(&mut sim, &mut st, bits, 8.0 - FRAME); // fim: ~ −3

    assert!(
        (full - 5.0).abs() < 0.1,
        "no início do fade ainda está na Clip 2: {full}"
    );
    assert!(
        (-3.0..=5.0).contains(&mid) && (mid - 5.0).abs() > 0.5 && (mid + 3.0).abs() > 0.5,
        "no meio o objeto está ENTRE +5 e −3: {mid}"
    );
    assert!(
        (done + 3.0).abs() < 0.5,
        "no fim chegou ao início da Main (−3): {done}"
    );
}

/// **Com fade dos DOIS lados a costura fica no FIM do loop — sem salto duplo.** A Main tem
/// fade-in; a Clip 2 tem fade-out. O fade-in da Main já cruza do fim da Clip 2 (+5,
/// d326ff28); então o fade-out da Clip 2 tem de cruzar para o MESMO ponto (+5, a própria
/// ponta dela), não para o início da Main (−3) — senão a volta pularia 8 unidades. É o
/// `head_live` do `seam_source`: quando a cabeça também fadeia, ninguém a possui e a
/// costura é o fim do loop. Um espelho ingênuo (sempre o início da primeira) sangra aqui.
#[test]
fn with_both_fades_the_seam_stays_on_the_loop_end_no_double_jump() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = two_strips((-3.0, 2.0), (5.0, 5.0), 1.0, 1.0);
    st.doc.set_active_loop_for(false, Some((0.0, 8.0)));

    let before_wrap = x_at(&mut sim, &mut st, bits, 8.0 - FRAME);
    let after_wrap = x_at(&mut sim, &mut st, bits, 0.0);

    assert!(
        (before_wrap - 5.0).abs() < 0.1,
        "o fade-out cruza para o fim do loop (+5), não o início da Main (−3): {before_wrap}"
    );
    assert!(
        (after_wrap - 5.0).abs() < 0.1,
        "o fade-in da Main também cruza do fim da Clip 2 (+5): {after_wrap}"
    );
    let jump = (after_wrap - before_wrap).abs();
    assert!(
        jump < 0.5,
        "sem salto duplo na costura: {before_wrap} -> {after_wrap} (salto {jump})"
    );
}

/// **Sem loop, o fade-out da última strip revela o FUNDO segurado, não uma "cabeça".** É a
/// cerca de Chesterton: o wrap de saída só existe sob um loop. Sem ele a Clip 2 fadeia
/// para a pose que a Main deixou (o fim dela, +2) — a extrapolação Hold de sempre —, e a
/// régua acaba ali (nada reinicia). Guarda que o `closing` está DENTRO do `if loop`.
#[test]
fn with_no_loop_the_last_fade_out_reveals_the_held_background_not_a_head() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = two_strips((-3.0, 2.0), (5.0, 5.0), 0.0, 1.0);
    // sem `set_active_loop_for` — o loop está desarmado.

    let done = x_at(&mut sim, &mut st, bits, 8.0 - FRAME);
    assert!(
        (done - 2.0).abs() < 0.1,
        "sem loop o fade-out revela o fim da Main (+2), o fundo segurado — não o início: {done}"
    );
}

/// **`bo > 0` — sem fade-out autorado, a última strip NÃO enrola.** A Clip 2 termina em 6
/// (sem `ease_out`) e o loop vai até 8: no vão `[6, 8)` o objeto segura o FIM da Clip 2
/// (+5), a pós-imagem, e só corta na volta. Enrolar aqui poria a cabeça da Main (−3) na
/// tela durante um vão que é uma pausa autorada. (Tirar o `bo > 0` do `closing` acende −3
/// no vão.)
#[test]
fn a_trailing_gap_with_no_fade_out_holds_the_last_pose_not_the_head() {
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
    doc.add_strip(lane, 0, 0.0, 4.0);
    doc.add_strip(lane, c2, 4.0, 6.0); // termina em 6, sem ease_out
    doc.set_active_loop_for(false, Some((0.0, 8.0))); // vão [6, 8)

    let in_gap = x_at(&mut sim, &mut st, bits, 7.0);
    assert!(
        (in_gap - 5.0).abs() < 1e-6,
        "no vão o objeto segura o fim da Clip 2 (+5), não a cabeça da Main (−3): {in_gap}"
    );
}

/// **`t_end <= b` — uma última strip que ATRAVESSA o fim do loop não enrola seu
/// fade-out.** A Clip 2 vai até 8 com um fade-out longo (janela = seu span inteiro), mas o
/// loop fecha em 6: o fade só termina PASSADO a volta, onde o loop nunca chega. Dentro de
/// `[0, 6)` a queda revela o FUNDO (o fim da Main, +2), a extrapolação Hold — não a cabeça
/// (−3). (Tirar o `t_end <= b` faz a Clip 2 cruzar para −3 dentro do loop.)
#[test]
fn a_last_strip_straddling_the_loop_end_does_not_wrap_its_fade_out() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = two_strips((-3.0, 2.0), (5.0, 5.0), 0.0, 4.0);
    st.doc.set_active_loop_for(false, Some((0.0, 6.0))); // Clip 2 (até 8) atravessa

    // Em t≈6⁻ o complemento é maior (peso do fade ≈ 0,5) e o held domina.
    let x = x_at(&mut sim, &mut st, bits, 5.999);
    // Com o guard: held = fim da Main (+2) → objeto no lado positivo alto (~3,5).
    // Sem o guard: closing cruza para o início da Main (−3) → objeto perto de 1,0.
    assert!(
        x > 2.0,
        "a strip que atravessa o fim do loop revela o fundo (+2), não a cabeça (−3): {x}"
    );
}

/// **O fade OUTWARD (`lead_out`) da ÚLTIMA strip também cruza para a costura do loop** (Enio,
/// 2026-07-19): a Clip 2 (última) toca INTEIRA até seu último frame (+5) e então, no gap
/// antes da volta, fadeia para o início da Main (−3) — em vez de segurar o próprio último
/// frame e dar um salto na volta. É o ramo `closing` do `hold_at` passando a aceitar
/// `lead_out`, não só `ease_out`. Sem o fix o fade de saída não fazia transição nenhuma no
/// loop (before_wrap ficava em +5, salto de 8).
#[test]
fn a_lead_out_on_the_last_strip_crosses_to_the_seam_under_a_loop() {
    let Scene {
        mut sim,
        mut st,
        bits,
    } = two_strips((-3.0, 2.0), (5.0, 5.0), 0.0, 0.0); // Main anima −3→+2, sem ease_in; Clip2 chapada +5
    // A Clip 2 (última strip, índice 1) ganha um lead_out de 1 s, no gap [8, 9]; o loop o cobre.
    let clip2 = st.doc.stack()[0].strips[1].id;
    st.doc.strip_mut(0, clip2).unwrap().lead_out = 1.0;
    st.doc.set_active_loop_for(false, Some((0.0, 9.0)));

    let at_end = x_at(&mut sim, &mut st, bits, 8.0); // Clip2 tocou INTEIRA: +5
    let before_wrap = x_at(&mut sim, &mut st, bits, 9.0 - FRAME); // fim do lead-out: ~ início da Main
    let after_wrap = x_at(&mut sim, &mut st, bits, 0.0); // Main começa: −3

    assert!(
        (at_end - 5.0).abs() < 0.1,
        "Clip2 toca inteira: +5 no fim, ANTES do lead-out começar: {at_end}"
    );
    assert!(
        (before_wrap + 3.0).abs() < 0.2,
        "o lead-out cruza para a costura (início da Main, −3), não segura +5: {before_wrap}"
    );
    let jump = (after_wrap - before_wrap).abs();
    assert!(
        jump < 0.5,
        "a volta do loop é suave: {before_wrap} -> {after_wrap} (salto {jump})"
    );
}
