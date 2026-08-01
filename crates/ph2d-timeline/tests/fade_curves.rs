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
///
/// ⚠️ **A DIREÇÃO desta asserção inverteu quando a travessia virou uma só** (2026-08-01), e a
/// inversão é a consequência correta: a cabeça deixou de correr a curva INTEIRA na própria
/// janela e passa a correr a **segunda fatia** dela (`[f, 1]`). No começo de um `smoothstep`
/// o Linear vai na frente; na segunda metade é o `smoothstep` que vai — então a mesma
/// governança que empurrava a pose para BAIXO agora a segura mais ALTO. Medido: default
/// `-0,469`, governada `0,000`. O que o gate afirma continua sendo *"a curva da Clip 2
/// governa a entrada da Main"*; o sinal é derivado disso, não o contrário.
#[test]
fn under_a_loop_the_last_strips_curve_governs_the_head_fade() {
    let (mut sim, mut st, bits) = scene(None, None, true);
    let default_at = x_at(&mut sim, &mut st, bits, 0.125);

    let (mut sim, mut st, bits) = scene(Some(LINEAR), None, true);
    let governed_at = x_at(&mut sim, &mut st, bits, 0.125);

    assert!(
        governed_at > default_at + 0.05,
        "a entrada da Main tem de ser moldada pela curva de SAÍDA da Clip 2: \
         default={default_at} governada={governed_at}"
    );
}

/// **A travessia da costura é UMA — o objeto não PARA na volta** (Enio, 2026-08-01: *"a
/// curva desenhada não expressa essa continuidade, mas deve"*).
///
/// A regra *"a curva da última prevalece"* dava a mesma CURVA aos dois lados e deixava cada
/// um correr um S INTEIRO na própria janela — o joelho voltava pelo outro caminho. Medido
/// antes: a velocidade subia a `-0,299`, caía a **`0,000` exatamente na volta** e recomeçava
/// do zero. Depois: sobe monotônica até `-0,298`, atravessa, e desce simétrica.
///
/// ⚠️ O oráculo é a VELOCIDADE, não a pose: a pose já era contínua (as duas metades
/// concordavam sobre a pose da costura) — o que faltava era a DERIVADA, e um gate de pose
/// ficaria verde sobre a parada.
#[test]
fn the_seam_crossing_is_one_traversal_with_no_stall_at_the_wrap() {
    let (mut sim, mut st, bits) = scene(None, None, true);
    // A cauda `[7.5, 8.0)` e, pela volta, a cabeça `[0.0, 0.5)` — o mesmo passo nas duas.
    let step = 0.025;
    let mut xs = Vec::new();
    for i in 0..20 {
        xs.push(x_at(&mut sim, &mut st, bits, 7.5 + f64::from(i) * step));
    }
    for i in 0..20 {
        xs.push(x_at(&mut sim, &mut st, bits, f64::from(i) * step));
    }
    let v: Vec<f64> = xs.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
    let peak = v.iter().fold(0.0_f64, |a, b| a.max(*b));
    // A volta cai entre a última amostra da cauda e a primeira da cabeça.
    let at_wrap = v[19];
    assert!(
        at_wrap > peak * 0.8,
        "na volta a travessia está no seu ponto MAIS RÁPIDO (f = 0,5), não parada: \
         v={at_wrap} pico={peak}"
    );
    // …e a subida até lá é monotônica: um S por ponta faria a velocidade subir e DESCER
    // dentro da cauda antes da volta.
    assert!(
        v[..19].windows(2).all(|w| w[1] >= w[0] - 1e-9),
        "a cauda tem de ACELERAR até a volta, sem um S próprio: {:?}",
        &v[..19]
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

/// **Numa SOBREPOSIÇÃO a curva alcança o crossfade — e ele continua somando 1.**
///
/// A decisão ANTIGA era recusá-la ali (a complementaridade valia por acidente:
/// `smoothstep(1−u) == 1−smoothstep(u)`), e ela caiu quando o Enio pediu o menu no corpo
/// INTEIRO do fade — inclusive o de duas strips sobrepostas, que não tem cunha. Um menu que
/// autora um número que ninguém lê é o item morto que este projeto bane, então a saída foi
/// tomar o complemento **explicitamente**: uma travessia, uma curva, e o lado que sai é
/// `1 − c(u)` — exato para QUALQUER curva.
///
/// ⚠️ A metade que prova que a lei não se perdeu mora no `clip_stack.rs`
/// (`the_crossfade_weights_sum_to_exactly_one_through_the_whole_overlap`, agora varrendo
/// curvas ASSIMÉTRICAS). Aqui prova-se só que a curva de fato MOLDA.
#[test]
fn an_overlap_crossfade_wears_the_arriving_strips_curve() {
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
        doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
        let b = doc.add_strip(lane, c2, 3.0, 7.0).unwrap(); // 1 s de SOBREPOSIÇÃO
        doc.strip_mut(lane, b).unwrap().curve_in = curve; // quem CHEGA governa
        (sim, st, bits)
    };
    let (mut sim, mut st, bits) = build(None);
    let plain = x_at(&mut sim, &mut st, bits, 3.25);
    let (mut sim, mut st, bits) = build(Some(LINEAR));
    let curved = x_at(&mut sim, &mut st, bits, 3.25);
    assert!(
        (curved - plain).abs() > 0.05,
        "a curva de quem chega tem de moldar o crossfade: plain={plain} curved={curved}"
    );

    // …e a de quem PARTE não governa: uma travessia, uma curva.
    let (mut sim, mut st, bits) = build(None);
    {
        let lane = 0;
        let a = st.doc.stack()[lane].strips[0].id;
        st.doc.strip_mut(lane, a).unwrap().curve_out = Some(LINEAR);
    }
    let departing = x_at(&mut sim, &mut st, bits, 3.25);
    assert!(
        (departing - plain).abs() < 1e-9,
        "a curva de SAÍDA de quem parte não pode governar: {departing} vs {plain}"
    );
}

/// **O INTENT chega ao avaliador** — a metade que os gates acima não cobrem.
///
/// Eles constroem a curva escrevendo no strip; este dirige a porta que o menu de fato usa
/// (`TimelineIntent::SetStripCurve`), e mede a POSE. Sem ele, a fiação do menu poderia
/// escrever no campo errado — ou em campo nenhum — com os quatro verdes.
#[test]
fn the_intent_reaches_the_pose() {
    use ph2d_core::Playhead;
    use ph2d_timeline::{TimelineIntent, apply_intent};

    let build = || {
        let mut sim = SimWorld::new();
        let bits = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("I")))
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
        // ⚠️ A Main leva `lead_in` porque sem ele há um VÃO no topo do loop: nada cobre a
        // volta, a costura não tem para onde atravessar, e a pose fica chapada em +5 —
        // uma fixture que não contém o fenômeno, com o produto correto.
        let main = doc.add_strip(lane, 0, 0.5, 4.0).unwrap();
        let clip2 = doc.add_strip(lane, c2, 4.0, 7.5).unwrap();
        doc.strip_mut(lane, main).unwrap().lead_in = 0.5;
        doc.strip_mut(lane, clip2).unwrap().lead_out = 0.5;
        doc.set_active_loop_for(false, Some((0.0, 8.0)));
        (sim, st, bits, lane, clip2)
    };

    let (mut sim, mut st, bits, ..) = build();
    let before = x_at(&mut sim, &mut st, bits, 7.625);

    let (mut sim, mut st, bits, lane, clip2) = build();
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetStripCurve {
            lane,
            id: clip2,
            edge: 1, // a borda de SAÍDA — a que fadeia para fora
            curve: Some(LINEAR),
        },
    );
    let after = x_at(&mut sim, &mut st, bits, 7.625);
    assert!(
        after < before - 0.05,
        "o intent tem de moldar o fade de saída: antes={before} depois={after}"
    );

    // …e a borda de ENTRADA é outra: o mesmo intent no `edge: 0` não pode mover este ponto.
    let (mut sim, mut st, bits, lane, clip2) = build();
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetStripCurve {
            lane,
            id: clip2,
            edge: 0,
            curve: Some(LINEAR),
        },
    );
    let other_edge = x_at(&mut sim, &mut st, bits, 7.625);
    assert!(
        (other_edge - before).abs() < 1e-9,
        "escrever a borda de ENTRADA não pode moldar a de SAÍDA: {other_edge} vs {before}"
    );
}

/// **O snapshot reporta a curva EFETIVA** — e é ela que o painel desenha dentro da cunha.
///
/// As duas bordas de uma sobreposição mostram a MESMA curva (a de quem chega), porque é UMA
/// travessia; fora dela, cada borda mostra a sua. Se o snapshot entregasse a autorada de cada
/// strip, o painel desenharia no lado que PARTE uma curva que o blend não usa: uma mentira
/// que **nenhum gate sobre o modelo pode ver**, porque o modelo está certo.
#[test]
fn the_snapshot_reports_the_curve_the_blend_actually_uses() {
    use ph2d_core::Playhead;
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let c2 = doc.add_clip("B".into());
    let lane = doc.add_lane("L".into()).unwrap();
    let a = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
    let b = doc.add_strip(lane, c2, 3.0, 7.0).unwrap(); // 1 s de SOBREPOSIÇÃO
    {
        // A entrada de A é LIVRE; a saída dela é a sobreposição, governada por B.
        let s = doc.strip_mut(lane, a).unwrap();
        s.ease_in = 0.5;
        s.curve_in = Some(LINEAR);
        s.curve_out = None;
    }
    doc.strip_mut(lane, b).unwrap().curve_in = Some(LINEAR);

    let mut snap = ph2d_timeline::TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &Playhead::new(1.0 / 60.0), false);
    let strips = &snap.lanes[0].strips;
    assert_eq!(
        strips[0].curve_in,
        Some(LINEAR),
        "a borda LIVRE reporta a curva autorada nela"
    );
    assert_eq!(
        strips[0].curve_out,
        Some(LINEAR),
        "a saída SOBREPOSTA reporta a de quem CHEGA — é ela que molda a travessia"
    );
    assert_eq!(
        strips[1].curve_in,
        Some(LINEAR),
        "e o outro lado da mesma sobreposição reporta a mesma"
    );
}

/// **Numa SOBREPOSIÇÃO as duas cunhas autoram a MESMA curva** (Enio, 2026-07-31: *"o fade
/// central entre strips sobrepostas deve ter apenas 1 easing mas está tocando dois"*).
///
/// Um crossfade é UMA travessia: a curva é a de quem CHEGA e o lado que sai é o complemento
/// explícito dela. O avaliador e o desenho já respondiam assim; a AUTORIA escrevia o
/// `curve_out` de quem parte — um campo que ninguém lê ali —, então o menu aceitava o clique
/// e a tela não mudava. As três metades (escreve · desenha · MOVE) ficam juntas porque foi
/// justamente a que faltava que passou despercebida com as outras verdes.
#[test]
fn an_overlap_has_one_easing_authored_from_either_wedge() {
    use ph2d_core::Playhead;
    use ph2d_timeline::{TimelineIntent, apply_intent};

    let build = || {
        let mut sim = SimWorld::new();
        let bits = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("O")))
            .id()
            .to_bits();
        let mut st = TimelineState::new();
        let doc = &mut st.doc;
        key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
        key(doc, bits, PropKind::TranslationX, 6.0, -3.0);
        let c2 = doc.add_clip("B".into());
        doc.set_active(c2);
        key(doc, bits, PropKind::TranslationX, 0.0, 5.0);
        key(doc, bits, PropKind::TranslationX, 6.0, 5.0);
        doc.set_active(0);
        let lane = doc.add_lane("L".into()).unwrap();
        let a = doc.add_strip(lane, 0, 0.0, 4.0).unwrap();
        let b = doc.add_strip(lane, c2, 2.0, 6.0).unwrap(); // sobrepõe [2, 4]
        (sim, st, bits, lane, a, b)
    };
    // Um quarto dentro da janela de sobreposição — onde `Linear` e `smoothstep` mais diferem.
    let probe = 2.5;

    let (mut sim, mut st, bits, ..) = build();
    let plain = x_at(&mut sim, &mut st, bits, probe);

    // Autorado pela cunha de SAÍDA (a de quem parte) — a que era inerte.
    let (mut sim, mut st, bits, lane, a, b) = build();
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetStripCurve {
            lane,
            id: a,
            edge: 1,
            curve: Some(LINEAR),
        },
    );
    let from_out = x_at(&mut sim, &mut st, bits, probe);
    assert!(
        (from_out - plain).abs() > 0.05,
        "a cunha de saída tem de MOVER o crossfade: {from_out} vs {plain}"
    );
    {
        let l = &st.doc.stack()[lane];
        assert_eq!(l.strips[1].curve_in, Some(LINEAR), "escreve no DONO");
        assert_eq!(l.strips[0].curve_out, None, "e não no campo inerte");
        assert_eq!(
            l.effective_curve(0, 1),
            l.effective_curve(1, 0),
            "as duas cunhas mostram a MESMA curva"
        );
    }

    // …e a cunha de ENTRADA autora o mesmo campo: escolher Smooth ali volta ao de fábrica.
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetStripCurve {
            lane,
            id: b,
            edge: 0,
            curve: None,
        },
    );
    let back = x_at(&mut sim, &mut st, bits, probe);
    assert!(
        (back - plain).abs() < 1e-9,
        "Smooth pela outra cunha tem de desfazer a escolha: {back} vs {plain}"
    );
}
