//! **Os fades externos das DUAS pontas provocam a transição sob PingPong** (Enio,
//! 2026-07-31: *"a FADE final externa não provoca nenhum movimento, mas deveria … para
//! manter a coerência do sistema"* — e, na rodada seguinte, *"corrigiu a da direita mas
//! matou a inicial"*).
//!
//! ⚠️ **A cauda e a cabeça eram o MESMO defeito**, e eu só vi metade: as duas decaíam para o
//! `rest`. Consertar uma e afirmar que a outra era "por desenho" (o que este arquivo chegou
//! a pinar num gate) foi ler a lei de 2026-07-23 como sendo sobre o MODO, quando ela é sobre
//! um **vão SECO** — uma strip sem fade, num vão onde a lane não escreve nada.
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

/// **O que distingue a abertura NÃO é o modo, é o FADE** (Enio, 2026-07-31: *"corrigiu a
/// FADE externa final mas matou a inicial"*).
///
/// ⚠️ Este gate nasceu, horas antes, afirmando o OPOSTO (*"sob ping-pong a abertura sai do
/// REST"*) — eu li a lei de 2026-07-23 como sendo sobre o MODO, e ela é sobre um **vão
/// SECO**: a cena dela é uma strip *sem fade*, num vão onde a lane não escreve nada. Com um
/// `lead_in` a lane já escreve, e sair do `rest` era **morte medida** para quem parkou a
/// sprite na pose inicial — `-3,000 -3,000 -3,000 -3,000`, o mesmo mecanismo que matava a
/// cauda, com o mesmo disfarce.
///
/// As duas metades ficam num gate só de propósito: elas são a MESMA pergunta respondida
/// pela presença do fade, e separá-las é como uma metade sobrevive a uma mutação da outra.
#[test]
fn the_opening_fade_travels_under_pingpong_but_a_bare_gap_stays_silent() {
    let x = |sim: &SimWorld, bits: u64| {
        f64::from(
            sim.world()
                .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
                .unwrap()
                .translation
                .x,
        )
    };
    // (a) COM fade: a cabeça cruza da COSTURA — logo o `rest` não tem voto, e é isso que a
    // mantém viva na cena ordinária (sprite parkada onde a animação começa, `rest = -3`).
    let head = |rest: f32| {
        let (mut sim, mut st, bits) = scene(true, rest);
        apply_from_doc(sim.world_mut(), &mut st.doc, 0.0);
        x(&sim, bits)
    };
    assert!(
        (head(5.0) - head(-3.0)).abs() < 1e-9,
        "o rest não pode ter voto na abertura: {} vs {}",
        head(5.0),
        head(-3.0)
    );
    assert!(
        (head(-3.0) + 3.0).abs() > 1.0,
        "e ela tem de SAIR da pose inicial — senão o fade não provoca transição: {}",
        head(-3.0)
    );

    // (b) SEM fade: vão seco, a lane escreve NADA e o objeto fica onde a strip o deixou
    // (a lei de 2026-07-23). `rest = 5` e a costura valem poses distintas da parada, então
    // este oráculo separa os três.
    let (mut sim, mut st, bits) = scene(true, 5.0);
    st.doc.stack_mut()[0].strips[0].lead_in = 0.0;
    apply_from_doc(sim.world_mut(), &mut st.doc, 2.0); // dentro da strip
    let parked = x(&sim, bits);
    apply_from_doc(sim.world_mut(), &mut st.doc, 0.2); // o vão seco
    assert!(
        (x(&sim, bits) - parked).abs() < 1e-9,
        "vão SECO sob ping-pong tem de ficar mudo: {} != {parked}",
        x(&sim, bits)
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

/// **Um fade que não ALCANÇA o começo do alcance não é a abertura** — o espelho do
/// `reaches_the_end`, e a cláusula que impede o fix de mexer no que ninguém reportou.
///
/// Aqui a primeira strip começa em `3.0` com um fade de `0.5`, precedido de um vão SECO
/// `[0, 2.5)`. Ela fadeia, mas não é a abertura da composição: o que ela cruza fica como
/// era (o REST), e o `rest` VOLTA a ter voto — que é exatamente a diferença observável.
#[test]
fn a_fade_that_does_not_reach_the_range_start_is_not_the_opening() {
    let at = |rest: f32| {
        let (mut sim, mut st, bits) = scene(true, rest);
        {
            let s = &mut st.doc.stack_mut()[0].strips[0];
            s.t_start = 3.0;
            s.lead_in = 0.5; // lead_start = 2.5 > a = 0
        }
        apply_from_doc(sim.world_mut(), &mut st.doc, 2.55);
        f64::from(
            sim.world()
                .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
                .unwrap()
                .translation
                .x,
        )
    };
    assert!(
        (at(5.0) - at(-3.0)).abs() > 1.0,
        "fora da abertura o rest ainda manda: {} vs {}",
        at(5.0),
        at(-3.0)
    );
}

/// **A costura da abertura não vaza para o fade-out da PRÓPRIA strip que abre.**
///
/// ⚠️ Este gate existe porque uma mutação SOBREVIVEU (tirar a janela do fade do predicado da
/// abertura) — e o buraco era de FIXTURE, não do produto: nenhuma cena minha tinha uma strip
/// que ABRE a composição e depois fadeia para fora **sem alcançar o fim do alcance**. Ali
/// nada terminou ainda, então o ramo da abertura é alcançável no meio do fade de SAÍDA dela,
/// e sem a janela ele responderia a costura no lugar do decaimento para o rest.
#[test]
fn the_opening_seam_does_not_leak_into_the_first_strips_own_fade_out() {
    let mut sim = SimWorld::new();
    let mut tr = Transform::default();
    tr.translation.x = 100.0; // um rest INCONFUNDÍVEL
    let bits = sim.world_mut().spawn((tr, Name::new("F"))).id().to_bits();
    let mut st = TimelineState::new();
    key(&mut st.doc, bits, 0.0, 0.0);
    key(&mut st.doc, bits, 4.0, 10.0); // a rampa: começo e fim DIFEREM
    let lane = st.doc.add_lane("L".into()).unwrap();
    let a = st.doc.add_strip(lane, 0, 0.5, 4.0).unwrap();
    {
        let s = st.doc.strip_mut(lane, a).unwrap();
        s.lead_in = 0.5; // ela ABRE a composição
        s.ease_out = 0.5; // e fadeia para fora em [3.5, 4.0], longe do fim do alcance (8)
    }
    st.doc.set_active_loop_for(false, Some((0.0, 8.0)));
    st.doc.set_active_ping_pong_for(false, true);
    apply_from_doc(sim.world_mut(), &mut st.doc, 3.9);
    let x = f64::from(
        sim.world()
            .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
            .unwrap()
            .translation
            .x,
    );
    assert!(
        x > 20.0,
        "no fade-out dela a pose decai para o REST (100), não para a costura: {x}"
    );
}

/// **Uma strip que começa ANTES do alcance não abre a composição** — o espelho do
/// `last.t_end <= b` da borda de saída.
///
/// ⚠️ Também nasceu de uma mutação sobrevivente, e a geometria que a torna observável não é
/// óbvia: com um fade para FORA (`lead_in`) a cláusula é redundante (a janela já a implica),
/// e com uma SOBREPOSIÇÃO a cobertura soma 1 e o peso segurado é zero. Quem a acorda é um
/// **`ease_in` autorado** numa lane sem vizinho, com o alcance do loop começando DEPOIS do
/// início da strip: ali o fade dela corre quase todo fora do alcance, e a costura seria a
/// pose de um fim que este playhead nunca visita.
#[test]
fn a_strip_that_starts_before_the_range_does_not_open_the_composition() {
    let mut sim = SimWorld::new();
    let mut tr = Transform::default();
    tr.translation.x = 100.0;
    let bits = sim.world_mut().spawn((tr, Name::new("F"))).id().to_bits();
    let mut st = TimelineState::new();
    key(&mut st.doc, bits, 0.0, 0.0);
    key(&mut st.doc, bits, 4.0, 10.0);
    let lane = st.doc.add_lane("L".into()).unwrap();
    let a = st.doc.add_strip(lane, 0, 0.5, 4.0).unwrap();
    st.doc.strip_mut(lane, a).unwrap().ease_in = 2.5; // fade PARA DENTRO, longo
    st.doc.set_active_loop_for(false, Some((2.0, 8.0))); // o alcance começa DEPOIS dela
    st.doc.set_active_ping_pong_for(false, true);
    apply_from_doc(sim.world_mut(), &mut st.doc, 2.1);
    let x = f64::from(
        sim.world()
            .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
            .unwrap()
            .translation
            .x,
    );
    assert!(
        x > 20.0,
        "ela não é a abertura: o complemento vai para o REST (100), não para a costura: {x}"
    );
}
