//! **What a behaviour's magnitude MEANS on the channel it drives** — the gates for the
//! one question that ties `motion.stagger` / `motion.oscillator` / `motion.wiggle`
//! together (split from `motion_bridge_param_tests.rs` for the HR-18 LOC cap; `super` is
//! `render_loop::motion_bridge`).
//!
//! The subject is a single fact with three faces: the same param means world metres on
//! Position, DEGREES on Rotation and a bare scale factor on Size. The shell answers it in
//! three places — the reset PRESET, the widget RANGE, and (since doc 88) the display
//! UNIT — and the three live in one file so they cannot drift apart.

use super::params::{apply_channel_presets, build_params_snapshot, param_value};
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// #10 consistency: switching a behaviour's channel resets its magnitude to a
/// channel-sensible default. The Rotation channel writes the `rot` stream column,
/// whose unit is **degrees** — so a stagger driving Rotation gets a ±90 ramp, not
/// the ±1 world-unit range meant for position. Non-behaviour types are untouched.
#[test]
fn channel_switch_resets_behaviour_magnitude_to_channel_defaults() {
    let mut motion = MotionState::new();
    let st = motion.doc.graph.add_node("motion.stagger");

    // -> Rotation (channel 2): a ±90 degree ramp.
    apply_channel_presets(&mut motion, st, "motion.stagger", 2.0);
    assert_eq!(param_value(&motion, st, "min"), -90.0);
    assert_eq!(param_value(&motion, st, "max"), 90.0);
    // -> Size (channel 3): ±½ scale.
    apply_channel_presets(&mut motion, st, "motion.stagger", 3.0);
    assert_eq!(param_value(&motion, st, "min"), -0.5);
    assert_eq!(param_value(&motion, st, "max"), 0.5);
    // -> back to Y (channel 1): the world-unit range returns.
    apply_channel_presets(&mut motion, st, "motion.stagger", 1.0);
    assert_eq!(param_value(&motion, st, "min"), -1.0);
    assert_eq!(param_value(&motion, st, "max"), 1.0);

    // Oscillator amplitude scales the same way (Rotation peaks at 30 degrees).
    let osc = motion.doc.graph.add_node("motion.oscillator");
    apply_channel_presets(&mut motion, osc, "motion.oscillator", 2.0);
    assert_eq!(param_value(&motion, osc, "amplitude"), 30.0);
    apply_channel_presets(&mut motion, osc, "motion.oscillator", 1.0);
    assert_eq!(param_value(&motion, osc, "amplitude"), 1.0);

    // A non-behaviour node (transform) is left alone.
    let xf = motion.doc.graph.add_node("motion.transform");
    apply_channel_presets(&mut motion, xf, "motion.transform", 2.0);
    assert_eq!(
        param_value(&motion, xf, "scale"),
        1.0,
        "transform untouched"
    );
}

/// Any param whose widget RANGE moves with the channel must have its VALUE reset
/// when the channel switches — otherwise it survives into a channel whose range
/// cannot show it. The oscillator's `offset` was exactly that hole: widened to
/// ±360 on Rotation, never reset, so a 300° offset landed in a ±10 world-unit
/// position channel. This pins preset-domain == override-domain.
#[test]
fn every_channel_ranged_param_is_reset_on_a_channel_switch() {
    let mut motion = MotionState::new();
    let osc = motion.doc.graph.add_node("motion.oscillator");

    // Dial an offset that is only legal on the Rotation channel.
    motion.doc.graph.set_param(osc, "channel", 2.0);
    motion.doc.graph.set_param(osc, "offset", 300.0);

    // Switch to a position channel: the preset must bring `offset` back in range.
    apply_channel_presets(&mut motion, osc, "motion.oscillator", 1.0);
    assert_eq!(
        param_value(&motion, osc, "offset"),
        0.0,
        "offset must reset with the channel whose range it borrowed"
    );
    assert_eq!(param_value(&motion, osc, "amplitude"), 1.0);
}

/// A behaviour's magnitude WIDGET RANGE follows the channel, not just its value.
/// The Enio caught this: with Channel=Rot the Stagger showed `Min -10 / Max 10`
/// even though the preset had written ±90 into the doc. The static hint range
/// (±10, authored for world units) could not represent ±90, so the slider
/// saturated, DISPLAYED -10, and would have overwritten the doc with -10 on the
/// first touch. On Rotation the range must be degrees-scaled and contain the
/// preset; on a position channel it stays the world-unit hint.
///
/// ⚠️ **The range assertions are made in STORE units, and that is not pedantry.**
/// Since doc 88 these behaviours declare `ParamUnit::FromChannel`, so on a position
/// channel the row is a `Length` and its numbers arrive in the artist's display unit
/// — `±1000 px` for the same `±10 m` hint. Asserting the painted numbers would make
/// this gate a hostage of `ProjectSettings::display_unit`: it would go red the day
/// somebody changed a default, while saying nothing about the channel logic it
/// exists to guard. `to_stored` is the same door `events.rs` uses on the way back.
#[test]
fn rotation_channel_widens_the_magnitude_range_to_contain_its_preset() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    let scalar = |motion: &MotionState, name: &str| {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("resolvable")
            .rows
            .into_iter()
            .find_map(|r| match r {
                ParamRow::Scalar(s) if s.name == name => Some(s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no scalar row {name}"))
    };

    // Stagger on Rot: the preset writes ±90 — the range must hold it.
    let st = motion.doc.graph.add_node("motion.stagger");
    ph2d_panel_motion_graph::set_graph_selection(vec![st.0]);
    motion.doc.graph.set_param(st, "channel", 2.0);
    apply_channel_presets(&mut motion, st, "motion.stagger", 2.0);
    for name in ["min", "max"] {
        let row = scalar(&motion, name);
        assert!(
            row.min <= row.value && row.value <= row.max,
            "{name}: preset {} escapes the widget range [{}, {}]",
            row.value,
            row.min,
            row.max
        );
        assert_eq!(
            (
                row.display.to_stored(row.min),
                row.display.to_stored(row.max)
            ),
            (-360.0, 360.0),
            "{name} is degree-scaled"
        );
    }
    // Back on a position channel the world-unit hint range returns — but ONLY
    // because the preset also brings the value home. (Switch the channel without
    // the preset and `contain` correctly keeps the range wide enough to show the
    // stale ±90 rather than lie about it — that is the other half of the fix.)
    motion.doc.graph.set_param(st, "channel", 1.0);
    apply_channel_presets(&mut motion, st, "motion.stagger", 1.0);
    let back = scalar(&motion, "min");
    assert_eq!(
        (
            back.display.to_stored(back.min),
            back.display.to_stored(back.max)
        ),
        (-10.0, 10.0)
    );

    // The wave behaviours' amplitude, same story (preset 30 vs a 0..10 hint).
    for (ty, node) in [
        (
            "motion.oscillator",
            motion.doc.graph.add_node("motion.oscillator"),
        ),
        ("motion.wiggle", motion.doc.graph.add_node("motion.wiggle")),
    ] {
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        motion.doc.graph.set_param(node, "channel", 2.0);
        apply_channel_presets(&mut motion, node, ty, 2.0);
        let row = scalar(&motion, "amplitude");
        assert!(
            row.min <= row.value && row.value <= row.max,
            "{ty}: amplitude preset {} escapes [{}, {}]",
            row.value,
            row.min,
            row.max
        );
        assert_eq!(row.max, 360.0, "{ty} amplitude is degree-scaled on Rot");
    }

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **A regra é varrida, não listada.** Para TODO tipo que declara uma faixa
/// angular, um valor em graus dialado em Rotation não pode sobreviver a uma troca
/// para um canal de posição — ele ficaria fora da faixa em que vai ser mostrado.
///
/// ⚠️ **Este gate substitui um irmão que se chamava `every_...` e exercitava UM
/// nó.** A varredura de 2026-08-14 mediu o preço da diferença: a tabela que ele
/// vigiava cobria **três dos seis** nós que precisavam dela, e os três ausentes
/// (`motion.drive`, `motion.step`, `motion.noise`) shipavam cada um à espera do
/// próprio report. Iterando o registry, um nó que nasça amanhã entra no gate
/// sozinho.
#[test]
fn every_declared_channel_ranged_param_is_brought_back_in_range() {
    let types: Vec<(String, Vec<(&'static str, f32)>)> = {
        let m = MotionState::new();
        m.registry
            .channel_ranged_types()
            .map(|(id, decls)| {
                let name = m
                    .registry
                    .manifests()
                    .find(|x| x.id == id)
                    .expect("todo tipo declarado tem manifesto")
                    .name
                    .to_string();
                (name, decls.iter().map(|d| (d.param, d.max)).collect())
            })
            .collect()
    };
    assert!(
        types.len() >= 6,
        "a varredura tem de ver os seis nos medidos, e viu {}",
        types.len()
    );

    for (type_name, params) in types {
        let mut motion = MotionState::new();
        let nid = motion.doc.graph.add_node(type_name.clone());
        // Em Rotation, dial no TOPO da faixa angular declarada.
        motion.doc.graph.set_param(nid, "channel", 2.0);
        for (p, deg_max) in &params {
            motion.doc.graph.set_param(nid, *p, *deg_max);
        }
        // Troca para X: o valor em graus nao pode sobreviver.
        apply_channel_presets(&mut motion, nid, &type_name, 0.0);
        let hints = motion
            .registry
            .param_ui(ph2d_nodegraph::node::NodeTypeId::of(type_name.as_str()))
            .unwrap_or(&[]);
        for (p, deg_max) in &params {
            let v = param_value(&motion, nid, p);
            let h = hints
                .iter()
                .find(|h| h.param == *p)
                .expect("um param de faixa declarada e' desenhado");
            assert!(
                v >= h.min && v <= h.max,
                "{type_name}.{p}: {v} ficou fora da faixa do canal de posicao \
                 [{}, {}] depois de {deg_max} graus",
                h.min,
                h.max
            );
        }
    }
}

/// **O REPORT do artista, num gate** (2026-08-14): *"Scale de drive não aceita
/// mais que 4 em sua caixa de texto e 4 não é quase nada para rot"*.
///
/// Duas metades, e nenhuma basta sozinha: o SLIDER passa a falar graus no canal
/// Rotation (o arrasto cobre uma volta) **e** a caixa de texto vai muito além
/// dele. ⚠️ O teto digitável é MEDIDO e nomeia o recurso — `2^24` é o primeiro
/// `f32` em que `x + 1.0 == x`, ou seja onde um passo de um grau deixa de mover o
/// número e o controle deixa de controlar; abaixo dele o kernel honra tudo
/// (o `scale` é multiplicação pura, medido a `1e7` graus com erro `0.000e0`).
#[test]
fn the_drive_scale_speaks_degrees_on_rotation_and_types_far_past_the_slider() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let d = motion.doc.graph.add_node("motion.drive");
    ph2d_panel_motion_graph::set_graph_selection(vec![d.0]);

    let row = |motion: &MotionState| {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("resolvable")
            .rows
            .into_iter()
            .find_map(|r| match r {
                ParamRow::Scalar(s) if s.name == "scale" => Some(s),
                _ => None,
            })
            .expect("o drive desenha o `scale`")
    };

    motion.doc.graph.set_param(d, "channel", 2.0); // Rotation
    let r = row(&motion);
    assert_eq!(
        (r.display.to_stored(r.min), r.display.to_stored(r.max)),
        (-360.0, 360.0),
        "no canal Rotation o arrasto cobre uma VOLTA, nao 4 graus"
    );
    assert!(
        r.display.to_stored(r.hard_max) >= 1.0e6,
        "a caixa de texto vai muito alem do slider: {}",
        r.hard_max
    );
    assert_eq!(
        r.display.to_stored(r.hard_min),
        -r.display.to_stored(r.hard_max),
        "o piso espelha o teto -- um scale negativo INVERTE o drive"
    );

    // O CONTROLE: num canal de POSICAO a faixa do hint volta, em unidades de
    // mundo. Sem ele o gate nao distinguiria *segue o canal* de *ficou largo*.
    motion.doc.graph.set_param(d, "channel", 0.0); // X
    apply_channel_presets(&mut motion, d, "motion.drive", 0.0);
    let back = row(&motion);
    assert_eq!(
        (
            back.display.to_stored(back.min),
            back.display.to_stored(back.max)
        ),
        (-4.0, 4.0),
        "em X o `scale` volta a ser um multiplicador de unidades de mundo"
    );
}
