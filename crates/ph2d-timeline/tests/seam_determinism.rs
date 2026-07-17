//! **A pose é função do PLAYHEAD** — não do caminho que o playhead fez até ele (ADR-0115).
//!
//! O Enio achou no smoke: dois strips ENCOSTADOS (sem sobreposição) davam, no MESMO `t = 3.0`,
//! `x = -3` se você viesse da esquerda e `x = +3` se viesse da direita.
//!
//! Causa: no instante em que os strips vivos somam peso ZERO (a primeira lasca de um fade-in), o
//! avaliador lia a lane como **silenciosa** — a mesma resposta que ele dá para "esta lane não
//! keya este canal" (esparsidade, R2). Silêncio faz ninguém escrever, de propósito: é o que deixa
//! em paz um canal que ninguém anima. Mas aqui a lane TINHA opinião — "influência 0", ou seja, a
//! pose de repouso — e, lida como silêncio, o objeto segurava o valor que o frame anterior deixou.
//!
//! O peso é DADO, não filtro: quem decide se um strip está ativo é COBRIR o tempo.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
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

/// A cena do smoke: Left `[0,3)` segura em -3, Right `[3,6)` segura em +3, ENCOSTADOS, com um
/// fade autorado na entrada do Right (o gesto do Enio: arrastar a barrinha da quina).
fn seam_scene() -> (SimWorld, TimelineState, u64) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("D")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, -3.0);
    let right = doc.add_clip("Right".into());
    doc.set_active(right);
    key(doc, bits, PropKind::TranslationX, 0.0, 3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, 3.0);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip(lane, 0, 0.0, 3.0);
    let b = doc.add_strip(lane, right, 3.0, 6.0).unwrap();
    doc.strip_mut(lane, b).unwrap().ease_in = 0.5;
    (sim, st, bits)
}

fn x_at(sim: &mut SimWorld, st: &mut TimelineState, bits: u64, t: f64) -> f32 {
    apply_from_doc(sim.world_mut(), &mut st.doc, t);
    sim.world()
        .get::<Transform>(Entity::from_bits(bits))
        .map_or(f32::NAN, |tr| tr.translation.x)
}

/// **O mesmo `t`, dois caminhos, a MESMA pose.** FALSIFICADO por voltar o `if w <= 0 { continue }`
/// ao `StackScratch::rebuild`: o strip de peso zero some da lista, a lane vira "silenciosa", e o
/// objeto segura a pose do frame anterior.
#[test]
fn the_pose_at_a_seam_does_not_depend_on_which_side_you_came_from() {
    let (mut sim, mut st, bits) = seam_scene();

    x_at(&mut sim, &mut st, bits, 0.5); // …chegando pela ESQUERDA
    let from_left = x_at(&mut sim, &mut st, bits, 3.0);
    x_at(&mut sim, &mut st, bits, 5.0); // …e pela DIREITA
    let from_right = x_at(&mut sim, &mut st, bits, 3.0);

    assert_eq!(
        from_left, from_right,
        "a pose no mesmo instante mudou conforme o lado de onde o playhead chegou: \
         {from_left} vs {from_right}"
    );
}

/// E o valor não é só consistente — é o CERTO. Um gate que só comparasse os dois caminhos ficaria
/// verde com os dois igualmente errados, então este crava o número.
///
/// **O número mudou em 2026-07-16, por decisão do Enio, e a espinha do gate não.** Este teste
/// pinava `0.0`: no primeiro instante do fade-in a influência do strip é 0, logo a pose era a de
/// REPOUSO. Path-independente e determinística — e, na tela, um SALTO: o sprite estava em -3
/// (deixado pelo Left) e pulava 3 unidades até o repouso pra só então rampear
/// (*"a sprite não faz a transição a partir de onde está mas pula para mais perto da posição
/// inicial da outra strip"*).
///
/// A causa não era o peso zero (silenciá-lo é o que quebrava a path-independence, ver o teste
/// acima): era a **lacuna nunca ter sido silêncio**. O strip que acabou segue afirmando o último
/// frame dele (`ClipLane::hold_at`), e o fade-in cruza a partir DALI. Continua sendo função pura
/// do playhead — só que agora a função é contínua. Trajetória completa: `tests/lone_fade.rs`.
#[test]
fn a_zero_weight_fade_edge_reports_the_held_pose_not_a_stale_one() {
    let (mut sim, mut st, bits) = seam_scene();
    x_at(&mut sim, &mut st, bits, 0.5); // captura o rest (= 0) e põe o objeto em -3
    assert_eq!(
        x_at(&mut sim, &mut st, bits, 3.0),
        -3.0,
        "influência 0 do Right = a pose que o Left SEGURA, não o repouso e não uma sobra do \
         frame anterior"
    );
    // …e o repouso não sumiu do modelo: sem nada antes pra segurar, um fade-in ainda entra
    // de lá (`lone_fade::the_first_strip_has_nothing_to_hold_and_fades_in_from_rest`).
}
