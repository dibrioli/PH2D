//! **O fade externo FINAL provoca a transição também sob PingPong** (Enio, 2026-07-31:
//! *"para PingPong a FADE final externa não provoca nenhum movimento, mas deveria provocar
//! a transição mesmo sendo inútil (sem sentido para uso real) mas para manter a coerência
//! do sistema e concordar com o que faz a FADE externa inicial"*).
//!
//! ⚠️ **O oráculo tem de ser o REST "ruim"**, e é essa a lição desta fixture: sob ping-pong a
//! cauda decaía para a pose de REPOUSO, e o `rest` é capturado por binding — quem liga a
//! sprite ONDE A ANIMAÇÃO A DEIXA (o caso ordinário: coloca-se o objeto e depois anima-se)
//! via a influência cair sobre uma pose idêntica, ou seja NADA. Com `rest = 0` o defeito é
//! INVISÍVEL (medido: `5,00 → 0,52`, um movimento enorme), e um gate escrito assim ficaria
//! verde sobre o produto reportado.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        PropKind::TranslationX,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// A cena da foto: duas strips, fade EXTERNO em cada ponta, alcance terminando depois do
/// fade final. `rest_x` é onde o objeto estava quando foi ligado.
fn scene(ping: bool, rest_x: f32) -> (SimWorld, TimelineState, u64) {
    let mut sim = SimWorld::new();
    let mut tr = Transform::default();
    tr.translation.x = rest_x;
    let bits = sim.world_mut().spawn((tr, Name::new("F"))).id().to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    key(doc, bits, 0.0, -3.0);
    key(doc, bits, 4.0, -3.0);
    let c2 = doc.add_clip("Clip 2".into());
    doc.set_active(c2);
    key(doc, bits, 0.0, 5.0);
    key(doc, bits, 4.0, 5.0);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).unwrap();
    let main = doc.add_strip(lane, 0, 0.5, 4.0).unwrap();
    let clip2 = doc.add_strip(lane, c2, 4.0, 7.5).unwrap();
    doc.strip_mut(lane, main).unwrap().lead_in = 0.5;
    doc.strip_mut(lane, clip2).unwrap().lead_out = 0.5;
    doc.set_active_loop_for(false, Some((0.0, 8.0)));
    doc.set_active_ping_pong_for(false, ping);
    (sim, st, bits)
}

/// Quanto o objeto ANDA ao longo da faixa de cauda `[7.5, 8.0]`.
fn travel(ping: bool, rest_x: f32) -> f64 {
    let (mut sim, mut st, bits) = scene(ping, rest_x);
    let e = ph2d_ecs::Entity::from_bits(bits);
    let x = |sim: &mut SimWorld, st: &mut TimelineState, t: f64| {
        apply_from_doc(sim.world_mut(), &mut st.doc, t);
        f64::from(sim.world().get::<Transform>(e).unwrap().translation.x)
    };
    let a = x(&mut sim, &mut st, 7.5);
    let b = x(&mut sim, &mut st, 7.99);
    (b - a).abs()
}

/// **A cauda ANDA sob ping-pong, com o rest que a torna invisível.**
///
/// Nasceu VERMELHO: `0.0000` de percurso — o objeto parado a faixa inteira, que é
/// literalmente o report. O controle é o mesmo cenário sob LOOP, que sempre funcionou.
#[test]
fn the_final_outward_fade_travels_under_pingpong_too() {
    let looped = travel(false, 5.0);
    assert!(
        looped > 3.0,
        "CONTROLE: sob loop a cauda sempre atravessou — {looped}"
    );
    let ping = travel(true, 5.0);
    assert!(
        ping > 3.0,
        "e sob ping-pong tem de atravessar também: andou {ping} (o report era 0)"
    );
}

/// **A abertura NÃO envolve sob ping-pong** — o fix do Enio de 2026-07-23 fica intacto.
///
/// As duas metades vivem no mesmo `loop_range`, e a correção acima o entrega à borda de
/// SAÍDA. Esta prova que ela não vazou para a de ENTRADA: um playhead que reflete nunca
/// cruza a costura, então a pose que o fim do loop deixa não pode aparecer no vão de
/// abertura. Sob ping-pong a cabeça sai do REST; sob loop, da costura.
#[test]
fn the_opening_gap_still_does_not_wrap_under_pingpong() {
    let head = |ping: bool| {
        let (mut sim, mut st, bits) = scene(ping, 5.0);
        apply_from_doc(sim.world_mut(), &mut st.doc, 0.0);
        f64::from(
            sim.world()
                .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
                .unwrap()
                .translation
                .x,
        )
    };
    assert!(
        (head(true) - 5.0).abs() < 0.01,
        "sob ping-pong a abertura sai do REST (5), não da costura: {}",
        head(true)
    );
    assert!(
        (head(false) - 5.0).abs() > 1.0,
        "e sob LOOP ela sai da costura, que é outra pose: {}",
        head(false)
    );
}

/// **A regra "a curva da ÚLTIMA prevalece" é do LOOP, e não vale sob ping-pong.**
///
/// Ela existe porque as duas metades da volta são UMA travessia, e duas curvas a moldariam
/// com um joelho no meio. Sob reflexão não há volta: a cabeça é uma entrada por direito
/// próprio, e usa a curva DELA.
///
/// ⚠️ A fixture autora curvas DIFERENTES nas duas pontas — sem isso a mutação que faz a
/// costura governar a cabeça é INERTE (as duas curvas seriam a de fábrica), e o gate ficaria
/// verde sobre a regra errada.
#[test]
fn the_seam_curve_does_not_govern_the_head_under_pingpong() {
    use ph2d_timeline::{Easing, EasingFamily, EasingMode};
    const LINEAR: Easing = Easing {
        family: EasingFamily::Linear,
        mode: EasingMode::In,
    };
    let head_at = |ping: bool, tail_curve: Option<Easing>| {
        let (mut sim, mut st, bits) = scene(ping, 5.0);
        {
            let lane = 0;
            let ids: Vec<_> = st.doc.stack()[lane].strips.iter().map(|s| s.id).collect();
            st.doc.strip_mut(lane, ids[1]).unwrap().curve_out = tail_curve;
        }
        // ⚠️ `0.125` = um QUARTO da janela, não o meio: em `u = 0.5` o `smoothstep` e o
        // `Linear` valem os DOIS `0.5`, e o controle abaixo nasceu vermelho sobre um produto
        // correto porque eu tinha amostrado exatamente ali.
        apply_from_doc(sim.world_mut(), &mut st.doc, 0.125);
        f64::from(
            sim.world()
                .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
                .unwrap()
                .translation
                .x,
        )
    };
    assert!(
        (head_at(true, Some(LINEAR)) - head_at(true, None)).abs() < 1e-9,
        "sob ping-pong a curva da cauda não pode moldar a entrada da cabeça"
    );
    // CONTROLE: sob LOOP ela molda — é a decisão do Enio, e sem esta metade o gate acima
    // passaria numa build que simplesmente ignorasse a costura em todo lugar.
    assert!(
        (head_at(false, Some(LINEAR)) - head_at(false, None)).abs() > 0.01,
        "e sob loop ela PRECISA moldar — a costura é uma travessia só"
    );
}
