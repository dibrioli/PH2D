//! **O guardião dos canais que o guardião do fade não vigiava** — a condição de
//! aceitação nº 6 do [ADR-0146] (C4).
//!
//! `fade_fingerprint.rs` é a joia: ela pina a superfície de fade inteira (crossfade
//! por sobreposição, `lead_out`, instância de container) num hash exato. ⚠️ Mas o
//! corpus dela é **`TranslationX` e só ele** — um canal de nove. Foi por isso que o
//! C4 viveu: um Morph sob um fade-in estalava 0,700 num frame, e o guardião do fade
//! não podia vê-lo, porque não olhava para lá.
//!
//! Esta é a IRMÃ, e cobre exatamente o complemento: **Morph e Position** sob a mesma
//! superfície. Ela nasceu DEPOIS do fix, então o que ela pina é o comportamento
//! correto — e o `0x69dca8811eb0f8f8` não se move, porque não há motivo para mover
//! um pin que já dizia a verdade sobre o canal dele.
//!
//! ⚠️ **O guard de inércia é POR CANAL, e isso é o ponto.** Um hash sobre três canais
//! em que um deles vai a zero continua estável e deixa de provar aquele canal — que é
//! precisamente o modo de falha que esta wave existe para fechar. Um `range > k` global
//! seria satisfeito pelo canal que se mexe mais.
//!
//! ⚠️ CPU-only e hash-safe: `MotionPath::project` é amostragem de contagem FIXA mais
//! ternary search de trip count FIXO, só `+ − * /` e comparação em `f64` (`path.rs`),
//! e o Rust não contrai FMA — o mesmo template do irmão.
//!
//! [ADR-0146]: ../../../docs/architecture/decisions/0146-timeline-expressions-are-a-first-class-lane-source-that-fades.md

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, Transform, VecMorph, World};
use ph2d_timeline::{
    MotionPath, PathAnchor, PropKind, TimelineDoc, TimelineState, apply_from_doc,
};

fn key(doc: &mut TimelineDoc, clip: usize, bits: u64, p: PropKind, t: f64, v: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
    doc.set_active(was);
}

/// **A cena.** Dois clips numa faixa, com um vão antes do primeiro (o fade-in a partir
/// do `rest` — a metade que o C4 quebrava) e uma SOBREPOSIÇÃO entre eles (o crossfade).
///
/// - `Shape` carrega um `VecMorph` autorado em `t = 0,7` e é bindado em **Morph**.
/// - `Runner` percorre um caminho em L e é bindado em **Position** (uma DISTÂNCIA),
///   com a pose autorada em (10, 2) — distância 12 ao longo do L.
///
/// Duas entidades, e não uma: Position e os canais de transform escrevem os dois no
/// `Transform`, então bindá-los no mesmo objeto mediria uma briga, não um fade.
fn build_scene() -> (World, TimelineDoc, u64, u64) {
    let mut world = World::new();
    let shape = world
        .spawn((
            Transform::default(),
            Name::new("Shape"),
            VecMorph {
                sources: [0, 0],
                t: 0.7,
            },
        ))
        .id()
        .to_bits();
    let runner = world
        .spawn((Transform::default(), Name::new("Runner")))
        .id()
        .to_bits();
    world
        .get_mut::<Transform>(Entity::from_bits(runner))
        .expect("runner has a transform")
        .translation = ph2d_core::Vec2::new(10.0, 2.0);

    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "A".into());
    let b = doc.add_clip("B".into());

    doc.bind(runner, PropKind::Position);
    for (clip, m, s) in [(0usize, 0.10_f32, 2.0_f32), (b, 0.95, 14.0)] {
        key(doc, clip, shape, PropKind::Morph, 0.0, m);
        key(doc, clip, shape, PropKind::Morph, 3.0, m);
        key(doc, clip, runner, PropKind::Position, 0.0, s);
        key(doc, clip, runner, PropKind::Position, 3.0, s);
    }
    // O L: 10 para a direita, 6 para cima — distância total 16.
    let i = doc
        .bindings()
        .iter()
        .position(|x| x.prop == PropKind::Position)
        .expect("the key bound one");
    doc.bindings_mut()[i].path = Some(MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        PathAnchor::corner([10.0, 0.0]),
        PathAnchor::corner([10.0, 6.0]),
    ]));

    let lane = doc.add_lane("L".into()).expect("scene lane");
    doc.add_strip(lane, 0, 1.0, 4.0);
    doc.add_strip(lane, b, 3.5, 7.0);
    doc.stack_mut()[lane].strips[0].ease_in = 0.5; // fade-in a partir do REST
    doc.stack_mut()[lane].strips[1].ease_in = 0.5; // e a sobreposição = crossfade

    let doc = std::mem::take(&mut st.doc);
    (world, doc, shape, runner)
}

/// Uma amostra por instante: o `t` do Morph e o PONTO onde o Position pôs o Runner.
#[derive(Clone, Copy)]
struct Sample {
    t: f64,
    morph: f32,
    x: f32,
    y: f32,
}

fn fingerprint() -> (u64, Vec<Sample>) {
    let (mut world, mut doc, shape, runner) = build_scene();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    let mut samples = Vec::with_capacity(161);
    for i in 0..=160 {
        let t = f64::from(i) * 0.05;
        apply_from_doc(&mut world, &mut doc, t);
        let morph = world
            .get::<VecMorph>(Entity::from_bits(shape))
            .expect("shape has a morph")
            .t;
        let xf = world
            .get::<Transform>(Entity::from_bits(runner))
            .expect("runner has a transform");
        let s = Sample {
            t,
            morph,
            x: xf.translation.x,
            y: xf.translation.y,
        };
        samples.push(s);
        for v in [s.morph, s.x, s.y] {
            for byte in v.to_bits().to_le_bytes() {
                h ^= u64::from(byte);
                h = h.wrapping_mul(0x0000_0100_0000_01b3); // FNV-1a prime
            }
        }
    }
    (h, samples)
}

/// O pin, capturado 2026-07-28 na árvore com o C4 fechado. Se uma wave o mover, ela
/// tocou o fade **nestes** canais — re-pine no MESMO commit com o motivo, ou reverta.
const CHANNEL_FINGERPRINT: u64 = 0x4706_da93_85d2_8f53;

#[test]
fn the_fade_surface_is_byte_stable_on_morph_and_position() {
    let (h, samples) = fingerprint();

    // Guard de inércia POR CANAL — ver o doc do módulo. Cada canal tem de EXERCITAR
    // o fade sozinho; um canal que emudece não pode ser coberto pela variação do outro.
    let span = |f: fn(&Sample) -> f32| {
        samples
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(f(s)), hi.max(f(s))))
    };
    for (name, (lo, hi), floor) in [
        ("morph", span(|s| s.morph), 0.5_f32),
        ("position.x", span(|s| s.x), 5.0),
        ("position.y", span(|s| s.y), 3.0),
    ] {
        assert!(
            hi - lo > floor,
            "o canal {name} ficou inerte na cena (faixa {lo}..{hi}); \
             ele tem de exercitar o fade por conta propria"
        );
    }

    assert_eq!(
        h, CHANNEL_FINGERPRINT,
        "\nO FADE MOVEU EM MORPH/POSITION. Este gate e o IRMAO do fade_fingerprint: \
         ele cobre os canais que aquele nao ve (ADR-0146 C4).\n\
         Se a mudanca foi intencional, re-pine CHANNEL_FINGERPRINT no MESMO commit \
         com o motivo. Senao, reverta.\n\
         amostras (t, morph, x, y) = {:?}",
        samples
            .iter()
            .map(|s| (s.t, s.morph, s.x, s.y))
            .collect::<Vec<_>>()
    );
}
