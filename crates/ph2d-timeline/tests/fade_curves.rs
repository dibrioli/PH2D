//! **A curva de cada fade é autorada** (Enio, 2026-07-31: *"no menu do botão direito sobre
//! o fade de uma strip vamos colocar as mesmas opções de easing que temos nos clips"*), uma
//! por BORDA — e na costura de um loop **a da última prevalece**.
//!
//! ⚠️ O oráculo é a POSE ao longo da janela, e as asserções são de FORMA (onde o objeto
//! está no primeiro quarto do fade), não de valor absoluto: um teste que fixasse o número
//! estaria pinando a álgebra da mistura, e o que se está afirmando é *"esta curva moldou
//! este fade"*.

use ph2d_anim::{AnimValue, Easing, EasingFamily, EasingMode, Interp, RationalTime};
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

fn x_at(sim: &mut SimWorld, st: &mut TimelineState, bits: u64, t: f64) -> f64 {
    apply_from_doc(sim.world_mut(), &mut st.doc, t);
    let e = ph2d_ecs::Entity::from_bits(bits);
    f64::from(sim.world().get::<Transform>(e).unwrap().translation.x)
}

/// `Linear` — a curva mais distante do `smoothstep` de fábrica no primeiro quarto da
/// janela (`0,25` contra `0,15625`), que é o que torna a diferença mensurável.
const LINEAR: Easing = Easing {
    family: EasingFamily::Linear,
    mode: EasingMode::In,
};

/// A cena da foto: Main (chapada em −3) com fade externo à esquerda, Clip 2 (chapada em
/// +5) com fade externo à direita, janelas de 0,5 s, e o loop `[0, 8]` armável.
fn scene(
    curve_out: Option<Easing>,
    curve_in: Option<Easing>,
    looped: bool,
) -> (SimWorld, TimelineState, u64) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Fade")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 4.0, -3.0);
    let c2 = doc.add_clip("Clip 2".into());
    doc.set_active(c2);
    key(doc, bits, PropKind::TranslationX, 0.0, 5.0);
    key(doc, bits, PropKind::TranslationX, 4.0, 5.0);
    doc.set_active(0);
    let lane = doc.add_lane("Lane 1".into()).unwrap();
    let main = doc.add_strip(lane, 0, 0.5, 4.0).unwrap();
    let clip2 = doc.add_strip(lane, c2, 4.0, 7.5).unwrap();
    {
        let s = doc.strip_mut(lane, main).unwrap();
        s.lead_in = 0.5;
        s.curve_in = curve_in;
    }
    {
        let s = doc.strip_mut(lane, clip2).unwrap();
        s.lead_out = 0.5;
        s.curve_out = curve_out;
    }
    if looped {
        doc.set_active_loop_for(false, Some((0.0, 8.0)));
    }
    (sim, st, bits)
}

/// **A curva escolhida molda o fade** — e a de fábrica continua sendo a de fábrica.
///
/// No primeiro quarto da janela de saída o `Linear` já andou `0,25` do caminho e o
/// `smoothstep` só `0,15625`, então o objeto está mais LONGE da pose da Clip 2 com a curva
/// autorada. A comparação é entre as duas cenas, não contra um número: é ela que afirma
/// *"a escolha chegou ao motor"* sem pinar a álgebra da mistura.
#[test]
fn an_authored_curve_shapes_the_fade() {
    let (mut sim, mut st, bits) = scene(None, None, true);
    let default_at = x_at(&mut sim, &mut st, bits, 7.625);

    let (mut sim, mut st, bits) = scene(Some(LINEAR), None, true);
    let linear_at = x_at(&mut sim, &mut st, bits, 7.625);

    assert!(
        linear_at < default_at - 0.05,
        "a curva Linear tem de ter andado MAIS que o smoothstep no primeiro quarto \
         (partindo de +5 rumo a −3): default={default_at} linear={linear_at}"
    );
}

/// **Na costura de um loop, a curva da ÚLTIMA prevalece sobre a da primeira** (decisão do
/// Enio) — as duas metades da volta são UMA travessia, e duas curvas a moldariam com um
/// joelho no meio.
///
/// A Clip 2 leva `Linear` na saída; a Main não escolheu nada. Sob o loop, o fade de ENTRADA
/// da Main tem de sair do `smoothstep` de fábrica e seguir o `Linear` da Clip 2.
#[test]
fn under_a_loop_the_last_strips_curve_governs_the_head_fade() {
    let (mut sim, mut st, bits) = scene(None, None, true);
    let default_at = x_at(&mut sim, &mut st, bits, 0.125);

    let (mut sim, mut st, bits) = scene(Some(LINEAR), None, true);
    let governed_at = x_at(&mut sim, &mut st, bits, 0.125);

    assert!(
        governed_at < default_at - 0.05,
        "a entrada da Main tem de ser moldada pela curva de SAÍDA da Clip 2: \
         default={default_at} governada={governed_at}"
    );
}

/// **CONTROLE: sem loop não há costura, e cada fade usa a PRÓPRIA curva** (decisão do
/// Enio). Sem esta asserção, uma regra que sempre deixasse a última prevalecer passaria no
/// gate acima e mudaria em silêncio um fade que não faz parte de volta nenhuma.
#[test]
fn with_no_loop_the_head_fade_keeps_its_own_curve() {
    let (mut sim, mut st, bits) = scene(None, None, false);
    let default_at = x_at(&mut sim, &mut st, bits, 0.125);

    let (mut sim, mut st, bits) = scene(Some(LINEAR), None, false);
    let with_tail_curve = x_at(&mut sim, &mut st, bits, 0.125);

    assert!(
        (with_tail_curve - default_at).abs() < 1e-9,
        "sem loop a curva da Clip 2 não pode alcançar o fade da Main: \
         default={default_at} com_curva_da_cauda={with_tail_curve}"
    );

    // …e a curva da PRÓPRIA cabeça continua alcançando o fade dela.
    let (mut sim, mut st, bits) = scene(None, Some(LINEAR), false);
    let own = x_at(&mut sim, &mut st, bits, 0.125);
    assert!(
        (own - default_at).abs() > 0.05,
        "a curva autorada NA Main tem de moldar o fade dela: default={default_at} own={own}"
    );
}

/// **A curva autorada NÃO alcança um crossfade de SOBREPOSIÇÃO**, e é a mesma lei que o
/// `ease_in`/`ease_out` já obedece: ali a sobreposição É o blend, e os dois pesos precisam
/// somar exatamente 1 (o que vale porque `smoothstep(1−u) == 1−smoothstep(u)`). Uma curva
/// assimétrica de um lado só faria a lane somar menos que 1 no meio do crossfade e a pose
/// afundaria para as lanes de baixo.
#[test]
fn an_overlap_crossfade_ignores_the_authored_curve() {
    let build = |curve: Option<Easing>| {
        let mut sim = SimWorld::new();
        let bits = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("Ov")))
            .id()
            .to_bits();
        let mut st = TimelineState::new();
        let doc = &mut st.doc;
        key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
        key(doc, bits, PropKind::TranslationX, 4.0, -3.0);
        let c2 = doc.add_clip("B".into());
        doc.set_active(c2);
        key(doc, bits, PropKind::TranslationX, 0.0, 5.0);
        key(doc, bits, PropKind::TranslationX, 4.0, 5.0);
        doc.set_active(0);
        let lane = doc.add_lane("L".into()).unwrap();
        let a = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
        let b = doc.add_strip(lane, c2, 3.0, 7.0).unwrap(); // 1 s de SOBREPOSIÇÃO
        doc.strip_mut(lane, a).unwrap().curve_out = curve;
        doc.strip_mut(lane, b).unwrap().curve_in = curve;
        (sim, st, bits)
    };
    let (mut sim, mut st, bits) = build(None);
    let plain = x_at(&mut sim, &mut st, bits, 3.25);
    let (mut sim, mut st, bits) = build(Some(LINEAR));
    let curved = x_at(&mut sim, &mut st, bits, 3.25);
    assert!(
        (curved - plain).abs() < 1e-9,
        "a sobreposição É o blend: a curva autorada não pode entrar nela — \
         plain={plain} curved={curved}"
    );
}
