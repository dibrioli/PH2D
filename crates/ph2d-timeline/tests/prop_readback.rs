//! **Ler uma propriedade de volta do mundo** — o C4 do [ADR-0146].
//!
//! `write_prop` toma o BINDING inteiro, e o doc-comment dele diz por quê: o valor de
//! [`PropKind::Position`] é uma *distância*, e transformá-la de volta num lugar precisa
//! da trajetória daquele binding. O inverso tomava menos — só o kind — e por isso não
//! conseguia responder por Position (que precisa do path) nem por Morph (uma omissão
//! pura). A assimetria ERA o bug:
//!
//! ```text
//! write_prop(world, e, b, v, orient)   <- binding
//! read_prop (world, e, prop)           <- so o kind    (o buraco)
//! ```
//!
//! O que isso custava, MEDIDO no produto antes do fix:
//!
//! - um Morph sob um fade-in **teleportava 0,700 num frame** (estalava na forma A),
//!   enquanto o mesmo fade num canal de transform partia da pose autorada;
//! - `Morpher.morph` num prop-link resolvia **0,000** com a fonte valendo 0,200;
//! - `Pather.position` resolvia **0,000** com a fonte na distância 3,000 — e este é o
//!   caso que nomeia a doença, porque o `rest` de Position **já funcionava** (o
//!   `refresh_liveness_and_rest` o roteava pelo `apply_path::read_rest`). A MESMA
//!   pergunta tinha DUAS portas, e elas discordavam.
//!
//! O parser já aceitava os dois tokens (`prop.rs`: `"morph"|"m"`, `"position"|"pos"|"p"`)
//! e a ordem topológica já montava a aresta — então não era capacidade ausente, era um
//! **controle que mentia**: o artista digitava um link válido e recebia zero em silêncio.
//!
//! [ADR-0146]: ../../../docs/architecture/decisions/0146-timeline-expressions-are-a-first-class-lane-source-that-fades.md

use ph2d_anim::{AnimTarget, AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecMorph, World};
use ph2d_timeline::{MotionPath, PathAnchor, PropKind, TimelineDoc, TimelineState, apply_from_doc};

fn key(doc: &mut TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// Carimba a expressão GLOBAL de `(bits, prop)` — a porta que o campo Expression da
/// track escreve (`SetBindingExpr`, ADR-0144).
fn drive(doc: &mut TimelineDoc, bits: u64, prop: PropKind, src: &str) {
    let tgt = doc.bind(bits, prop);
    doc.bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .expect("just bound")
        .expr = Some(src.to_string());
}

fn morph_of(w: &World, bits: u64) -> f32 {
    w.get::<VecMorph>(Entity::from_bits(bits)).unwrap().t
}

fn xf_of(w: &World, bits: u64) -> Transform {
    *w.get::<Transform>(Entity::from_bits(bits)).unwrap()
}

// ---------------------------------------------------------------------------
// G1 — a doutrina do `lone_fade`, no canal que ela nunca cobriu
// ---------------------------------------------------------------------------

/// **Um sistema de animação não teleporta** — `lone_fade.rs`, e vale para TODO canal.
///
/// Morph autorado em `t = 0,7`; a key diz `0,2`; a strip só cobre `[4,7)` com um
/// `ease_in`. No primeiro instante do fade o peso é zero, então a resposta tem de ser
/// o **rest** (0,7) e a partir dali uma rampa até 0,2.
///
/// ⚠️ O CONTROLE (um sprite com `TranslationX` na MESMA strip, nascido no mesmo 0,7)
/// é o que torna este gate um oráculo em vez de um espelho: ele já passava, então uma
/// divergência acusa o CANAL e não a cena. Nasceu VERMELHO em 0,700 de passo.
#[test]
fn a_morph_does_not_teleport_at_the_start_of_a_fade() {
    let mut sim = SimWorld::new();
    let morph = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Morpher"),
            VecMorph {
                sources: [0, 0],
                t: 0.7,
            },
        ))
        .id()
        .to_bits();
    let ctrl = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Ctrl")))
        .id()
        .to_bits();
    sim.world_mut()
        .get_mut::<Transform>(Entity::from_bits(ctrl))
        .unwrap()
        .translation
        .x = 0.7;

    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let right = doc.add_clip("Right".into());
    doc.set_active(right);
    key(doc, morph, PropKind::Morph, 0.0, 0.2);
    key(doc, morph, PropKind::Morph, 3.0, 0.2);
    key(doc, ctrl, PropKind::TranslationX, 0.0, 0.2);
    key(doc, ctrl, PropKind::TranslationX, 3.0, 0.2);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip(lane, right, 4.0, 7.0);
    doc.stack_mut()[lane].strips[0].ease_in = 0.5;

    // O instante em que o fade começa: peso zero, logo a resposta é o REST.
    apply_from_doc(sim.world_mut(), &mut st.doc, 4.0);
    assert_eq!(
        morph_of(sim.world(), morph),
        0.7,
        "o fade COMEÇA na pose autorada — nao na forma A"
    );

    // E nada na travessia teleporta. 240 amostras/s (4x um display de 60 fps), então
    // um salto não se esconde entre duas. O fade anda 0,5 em 0,5 s numa rampa
    // smoothstep (pico ~1,5x a linear), logo ~0,006/frame; 0,05 fica muito acima do
    // movimento real e muito abaixo do salto de 0,700 que este gate existe p/ pegar.
    let (mut worst, mut at) = (0.0_f32, 0.0_f64);
    let mut prev = {
        apply_from_doc(sim.world_mut(), &mut st.doc, 3.5);
        morph_of(sim.world(), morph)
    };
    for i in 1..=(240 * 3) {
        let t = 3.5 + f64::from(i) / 240.0;
        apply_from_doc(sim.world_mut(), &mut st.doc, t);
        let m = morph_of(sim.world(), morph);
        if (m - prev).abs() > worst {
            worst = (m - prev).abs();
            at = t;
        }
        prev = m;
    }
    assert!(
        worst < 0.05,
        "o morph teleportou {worst} num frame em t={at}"
    );
}

// ---------------------------------------------------------------------------
// G2 / G3 — a SEQUÊNCIA leva a algum lugar (a 4ª condição de UI)
// ---------------------------------------------------------------------------

/// Um prop-link que NOMEIA um canal de Morph lê o valor dele, não zero.
///
/// A fonte é **keyada** de propósito (não é de expressão): é o caminho que resolve
/// pelo mundo, e era exatamente o que devolvia `None`.
#[test]
fn a_prop_link_reads_a_morph_channel() {
    let mut sim = SimWorld::new();
    let src = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Morpher"),
            VecMorph {
                sources: [0, 0],
                t: 0.7,
            },
        ))
        .id()
        .to_bits();
    let reader = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Reader")))
        .id()
        .to_bits();

    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    key(doc, src, PropKind::Morph, 0.0, 0.2);
    key(doc, src, PropKind::Morph, 3.0, 0.2);
    key(doc, reader, PropKind::TranslationX, 0.0, 0.0);
    drive(doc, reader, PropKind::TranslationX, "Morpher.morph");

    for t in [0.5, 1.0, 2.0] {
        apply_from_doc(sim.world_mut(), &mut st.doc, t);
        assert!(
            (xf_of(sim.world(), reader).translation.x - 0.2).abs() < 1e-6,
            "t={t}: o link leu {} e a fonte vale 0,2",
            xf_of(sim.world(), reader).translation.x
        );
    }
}

/// O mesmo para Position — **o gate que prova a porta ÚNICA**.
///
/// O `rest` de Position já era lido corretamente (pelo `apply_path::read_rest`), então
/// consertar só o `rest` deixa este gate VERMELHO: a cura tem de ser a mesma porta para
/// as duas perguntas, não um segundo remendo ao lado do primeiro.
#[test]
fn a_prop_link_reads_a_position_channel() {
    let mut sim = SimWorld::new();
    let src = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Pather")))
        .id()
        .to_bits();
    let reader = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Reader")))
        .id()
        .to_bits();

    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.bind(src, PropKind::Position);
    key(doc, src, PropKind::Position, 0.0, 3.0);
    key(doc, src, PropKind::Position, 3.0, 3.0);
    let i = doc
        .bindings()
        .iter()
        .position(|b| b.prop == PropKind::Position)
        .unwrap();
    doc.bindings_mut()[i].path = Some(MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        PathAnchor::corner([10.0, 0.0]),
    ]));
    key(doc, reader, PropKind::TranslationY, 0.0, 0.0);
    drive(doc, reader, PropKind::TranslationY, "Pather.position");

    for t in [0.5, 1.0, 2.0] {
        apply_from_doc(sim.world_mut(), &mut st.doc, t);
        let y = xf_of(sim.world(), reader).translation.y;
        assert!(
            (y - 3.0).abs() < 1e-3,
            "t={t}: o link leu {y} e a fonte esta na distancia 3,0"
        );
    }
}

// ---------------------------------------------------------------------------
// G4 — a PROPRIEDADE, não a lista
// ---------------------------------------------------------------------------

/// **Ler de volta é o inverso exato de escrever, para TODO kind** — a frase que o
/// doc-comment de `read_prop` sempre afirmou e que era falsa em dois canais.
///
/// ⚠️ Este gate **não enumera** os kinds: ele varre os ids de `AnimTarget` e pergunta
/// ao `PropKind::from_target`, então um 9º kind entra aqui **no dia em que nasce**.
/// Enumeração apodrece — foi enumerando que Morph e Position ficaram de fora — e é por
/// isso que o oráculo aqui é a propriedade, nunca a lista.
///
/// `TimeRemap` é a exceção NOMEADA: ela não escreve propriedade nenhuma na cena (é o
/// relógio da entidade), então "escreve e lê de volta" não é uma pergunta sobre ela.
#[test]
fn reading_a_property_back_is_the_inverse_of_writing_it() {
    // Um valor por kind que é representável nele (Morph/Opacity vivem em [0,1]).
    fn probe_value(p: PropKind) -> f32 {
        match p {
            PropKind::Morph | PropKind::Opacity => 0.625,
            PropKind::Position => 3.0,
            _ => 1.75,
        }
    }

    /// O nome que o artista digita depois do ponto (`Nome.<isto>`), tal como
    /// `PropKind::from_expr_name` o aceita.
    ///
    /// ⚠️ **Exaustivo de propósito, e não é preguiça:** não existe inverso de
    /// `from_expr_name` na crate (o `i18n_suffix` diz `translation_x`, que aquele
    /// parser NÃO aceita), então um 9º `PropKind` faz este match **parar de
    /// compilar** — o aviso mais alto que um gate consegue dar, e o oposto de ser
    /// pulado em silêncio, que é como Morph e Position ficaram de fora.
    fn expr_alias(p: PropKind) -> &'static str {
        match p {
            PropKind::TranslationX => "x",
            PropKind::TranslationY => "y",
            PropKind::Rotation => "rotation",
            PropKind::ScaleX => "scalex",
            PropKind::ScaleY => "scaley",
            PropKind::Opacity => "opacity",
            PropKind::Position => "position",
            PropKind::Morph => "morph",
            // O relógio da entidade não é valor de cena — filtrado pelo chamador.
            PropKind::TimeRemap => "time",
        }
    }

    let mut checked = 0_usize;
    for id in 0..32_u64 {
        let Some(prop) = PropKind::from_target(AnimTarget::new(id)) else {
            continue;
        };
        if prop == PropKind::TimeRemap {
            continue; // não escreve propriedade de cena: fora da pergunta, por definição
        }
        // Opacity precisa da feature `render` (o `Sprite` mora na crate de GPU); sem ela
        // NEM escreve NEM lê, o que é consistente.
        if prop == PropKind::Opacity && cfg!(not(feature = "render")) {
            continue;
        }

        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Transform::default(),
                Name::new("Probe"),
                VecMorph {
                    sources: [0, 0],
                    t: 0.0,
                },
            ))
            .id()
            .to_bits();
        // ⚠️ A sonda tem de CONTER o fenômeno: sem um `Sprite` na entidade, o braço de
        // Opacity nem escreve nem lê, e o gate acusaria o produto por um buraco da
        // fixture. (Ele acusou — foi assim que esta linha nasceu.)
        #[cfg(feature = "render")]
        sim.world_mut()
            .entity_mut(Entity::from_bits(e))
            .insert(ph2d_render::Sprite::atlas(
                0,
                [10.0, 10.0],
                [1.0, 1.0, 1.0, 1.0],
            ));

        let mut st = TimelineState::new();
        let doc = &mut st.doc;
        let v = probe_value(prop);
        doc.bind(e, prop);
        key(doc, e, prop, 0.0, v);
        key(doc, e, prop, 3.0, v);
        if prop == PropKind::Position {
            let i = doc
                .bindings()
                .iter()
                .position(|b| b.prop == prop)
                .unwrap();
            doc.bindings_mut()[i].path = Some(MotionPath::new(vec![
                PathAnchor::corner([0.0, 0.0]),
                PathAnchor::corner([10.0, 0.0]),
            ]));
        }

        // O apply ESCREVE o valor no mundo…
        apply_from_doc(sim.world_mut(), &mut st.doc, 1.0);
        // …e um prop-link é a única forma pública de perguntar o que a porta de LEITURA
        // respondeu: um leitor cuja fórmula é o nome deste canal.
        let reader = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("Reader")))
            .id()
            .to_bits();
        let link = format!("Probe.{}", expr_alias(prop));
        key(&mut st.doc, reader, PropKind::ScaleX, 0.0, 0.0);
        drive(&mut st.doc, reader, PropKind::ScaleX, &link);

        apply_from_doc(sim.world_mut(), &mut st.doc, 1.0);
        let got = xf_of(sim.world(), reader).scale.x;
        assert!(
            (got - v).abs() < 1e-3,
            "{prop:?} ({link}): escreveu {v}, leu de volta {got}"
        );
        checked += 1;
    }

    // Controle positivo: um gate que varre zero kinds passa por vacuidade.
    assert!(
        checked >= 7,
        "a varredura cobriu so {checked} kinds — o iterador quebrou"
    );
}
