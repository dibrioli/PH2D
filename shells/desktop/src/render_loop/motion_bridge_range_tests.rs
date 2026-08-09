//! **The widget RANGE invariants** — the family of gates that keep a slider honest
//! (split from `motion_bridge_param_tests.rs` for the 700-LOC file cap). `super` is
//! `render_loop::motion_bridge`.
//!
//! A slider is two things: a value, and the SCALE the value is read against. Every bug
//! gated here is the scale misbehaving — deriving itself from the value it measures
//! (a feedback loop), being guessed because nobody declared it, or being too small to
//! hold the value it must show. Enio's smoke of 2026-07-12 hit the first of those, and
//! the sliders ran to billions.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// Every node type in the registry, with its params — the sweep the older guards
/// missed by filtering on `motion.`, which is precisely why the `sim.*` nodes shipped
/// with no hints at all and every slider in the rain graph ran away.
fn every_type_and_its_params(motion: &MotionState) -> Vec<(&'static str, Vec<&'static str>)> {
    motion
        .registry
        .manifests()
        .map(|m| (m.name, m.params.iter().map(|p| p.name).collect()))
        .collect()
}

/// **A range is not allowed to be a function of the value it ranges over.**
///
/// The bug this pins (Enio, smoke 2026-07-12 — *"os sliders chegam a bilhões e não
/// arrastam linearmente"*): the hintless fallback set `max = value * 4`, so the slider's
/// SCALE grew with the value it was measuring. That is positive feedback with a fixed
/// point at a quarter of the track — drag above it and the value multiplies every frame
/// until it is astronomical; drag below it and it collapses to zero. The knob never maps
/// to a stable number, which is what "não arrasta linearmente" means from the outside.
///
/// So: put an in-range value on the param — exactly what a drag does — and the range must
/// come back IDENTICAL. (Widening to hold an out-of-range value is still allowed; it is
/// idempotent, and a drag inside the range never triggers it.)
#[test]
fn a_drag_inside_the_range_never_moves_the_range() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    // The range AND the face it is expressed in. ⚠️ The face is not decoration
    // here: a row's numbers are what the ARTIST reads, and `set_param` writes what
    // the DOCUMENT stores. Feeding a displayed number straight back would put
    // pixels into a metres param — the very defect the display boundary exists to
    // prevent — and this gate would then report the resulting widened range as a
    // feedback loop in the product. (It did, the first time this wave ran: the
    // gate caught its own fixture skipping the conversion, which is the gate
    // working.) Angle/Seed rows never convert, so they carry the neutral face.
    let range_of = |motion: &MotionState,
                    param: &str|
     -> Option<(f64, f64, ph2d_panel_motion_params::RowDisplay)> {
        build_params_snapshot(motion, ProjectSettings::default())?
            .rows
            .into_iter()
            .find_map(|r| match r {
                ParamRow::Scalar(s) if s.name == param => Some((s.min, s.max, s.display)),
                ParamRow::Angle(a) if a.name == param => {
                    Some((a.min_deg, a.max_deg, Default::default()))
                }
                ParamRow::Seed(s) if s.name == param => Some((s.min, s.max, Default::default())),
                _ => None,
            })
    };

    for (ty, params) in every_type_and_its_params(&motion) {
        for param in params {
            let node = motion.doc.graph.add_node(ty);
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
            let Some(before) = range_of(&motion, param) else {
                continue; // not a continuous row (colour / toggle / enum / text)
            };
            let (min, max, face) = before;
            // Every place the knob can land, including the ends.
            for track in [0.0f64, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
                // What the panel would emit: land the affine in the DISPLAY face,
                // then undo it — exactly the one door `events.rs` uses.
                let stored = face.to_stored(min + track * (max - min));
                motion.doc.graph.set_param(node, param, stored as f32);
                let after = range_of(&motion, param).expect("the row survives its own edit");
                assert_eq!(
                    after, before,
                    "{ty}.{param}: dragging to {track} of the track moved the range from \
                     {before:?} to {after:?} — the scale is a function of the value it \
                     measures, so the drag is a feedback loop, not a drag"
                );
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **No param reaches the panel without a declared range.** A node crate that registers its
/// op but forgets `register_param_ui` used to fall through to a guessed range — which is how
/// `sim.collide.shape` (Floor / Disc / Bowl) came to be painted as a float slider the artist
/// had to decode, and how the whole `sim.*` family got the runaway fallback above. The hint
/// is where a param says what it MEANS (a count, a seed, a toggle, a named choice), so a
/// missing hint is a missing decision, not a cosmetic gap.
#[test]
fn every_scalar_row_comes_from_a_declared_hint() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let mut missing: Vec<String> = Vec::new();

    for (ty, _) in every_type_and_its_params(&motion) {
        let node = motion.doc.graph.add_node(ty);
        let tid = motion.doc.graph.node(node).unwrap().type_id();
        let hints = motion.registry.param_ui(tid);
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        let Some(snap) = build_params_snapshot(&motion, ProjectSettings::default()) else {
            continue;
        };
        for row in &snap.rows {
            let param = match row {
                ParamRow::Scalar(r) => r.name,
                ParamRow::Angle(r) => r.name,
                ParamRow::Seed(r) => r.name,
                _ => continue,
            };
            if hints
                .and_then(|hs| hs.iter().find(|h| h.param == param))
                .is_none()
            {
                missing.push(format!("{ty}.{param}"));
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        missing.is_empty(),
        "these params are painted with a GUESSED range — the node crate never called \
         `register_param_ui`, so nobody ever decided what they mean:\n{}",
        missing.join("\n")
    );
}

/// **Every `ParamSpec` in the registry has a hint — the census, not the sample.**
///
/// The sibling above walks the ROWS the panel produced, which is the right question for
/// "is anything painted with a guessed range?" and blind to two whole populations: a param
/// hidden by a `ParamGate` right now, and a param folded into a colour swatch. The first is
/// a real gap (the gate's `when` value decides whether it paints, so a hintless gated param
/// is one artist click from the fallback); the second is a legitimate exemption, because a
/// swatch's three non-anchor channels are *described by* the anchor's `Color` hint.
///
/// This is what makes the no-hint branch of `build_params_snapshot` **unreachable** rather
/// than merely unvisited: with the census green, the only way to land there is to register
/// a node type and forget `register_param_ui` — which is exactly what it reports.
#[test]
fn every_param_spec_carries_a_hint_or_is_folded_into_a_swatch() {
    use ph2d_node_registry::ParamWidget;
    use ph2d_nodegraph::cook::OpResolver;
    let mut motion = MotionState::new();
    let mut missing: Vec<String> = Vec::new();
    let mut counted = 0usize;

    for (ty, _) in every_type_and_its_params(&motion) {
        let node = motion.doc.graph.add_node(ty);
        let tid = motion.doc.graph.node(node).unwrap().type_id();
        let manifest = motion.registry.resolve(tid).unwrap().manifest();
        let hints = motion.registry.param_ui(tid);
        // The channels a `Color` anchor speaks for, plus the `mode` a `Channels` picker
        // folds in — the two ways a param legitimately reaches the panel without a hint
        // of its own. Both are read from the LIVE hints, so a node that stops folding one
        // starts owing a hint for it on the same commit.
        let mut folded: Vec<&'static str> = Vec::new();
        for h in hints.into_iter().flatten() {
            match h.widget {
                ParamWidget::Color { channels } => folded.extend_from_slice(&channels),
                ParamWidget::Channels { mode_param, .. } => folded.push(mode_param),
                _ => {}
            }
        }
        for spec in manifest.params {
            counted += 1;
            if folded.contains(&spec.name) {
                continue;
            }
            if hints
                .and_then(|hs| hs.iter().find(|h| h.param == spec.name))
                .is_none()
            {
                missing.push(format!("{ty}.{}", spec.name));
            }
        }
    }
    assert!(
        counted > 0,
        "positive control: the registry offered no params at all, so this census \
         proved nothing"
    );
    assert!(
        missing.is_empty(),
        "{} of {counted} params have no `ParamUiHint` — they reach the panel through the \
         hintless FALLBACK, whose range is a guess and whose unit can never be declared \
         (a `ParamUnit` is only consulted for a widget, and the fallback has none):\n{}",
        missing.len(),
        missing.join("\n")
    );
}

/// A param's default must be inside the range the panel paints it in. Otherwise the very
/// first frame widens the range to hold it (`contain`), and the widened bound then shrinks
/// back as the artist drags the value down — a range that chases its value, which is the
/// same class of bug as the one above, just quieter.
#[test]
fn every_param_default_is_inside_its_declared_range() {
    use ph2d_nodegraph::cook::OpResolver;
    let mut motion = MotionState::new();
    for (ty, _) in every_type_and_its_params(&motion) {
        let node = motion.doc.graph.add_node(ty);
        let tid = motion.doc.graph.node(node).unwrap().type_id();
        let manifest = motion.registry.resolve(tid).unwrap().manifest();
        let Some(hints) = motion.registry.param_ui(tid) else {
            continue;
        };
        for spec in manifest.params {
            let Some(h) = hints.iter().find(|h| h.param == spec.name) else {
                continue;
            };
            // Colour channels + free text are not painted on a numeric range.
            if !matches!(
                h.widget,
                ph2d_node_registry::ParamWidget::Slider
                    | ph2d_node_registry::ParamWidget::IntSlider
                    | ph2d_node_registry::ParamWidget::Angle
                    | ph2d_node_registry::ParamWidget::Seed
            ) {
                continue;
            }
            assert!(
                h.min <= spec.default && spec.default <= h.max,
                "{ty}.{}: default {} is outside its declared range [{}, {}]",
                spec.name,
                spec.default,
                h.min,
                h.max
            );
        }
    }
}

/// A `ScalarRow` de `param` no nó `type_name`, montada pela porta REAL do bridge.
fn row_of(type_name: &str, param: &str) -> ph2d_panel_motion_params::ScalarRow {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node(type_name);
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    let rows = build_params_snapshot(&motion, ProjectSettings::default())
        .unwrap_or_else(|| panic!("{type_name} resolve"))
        .rows;
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    rows.into_iter()
        .find_map(|r| match r {
            ParamRow::Scalar(s) if s.name == param => Some(s),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{type_name}.{param} tem row escalar"))
}

/// **A CAIXA VAI ALÉM DO SLIDER** — o slider dual do doc 88 A1, na porta do produto.
///
/// Até esta wave o `hard_max` caía em `unwrap_or(max)` para todo nó menos o `motion.emitter`:
/// a caixa de texto **não passava do slider**, e as duas perguntas — *que faixa o arrasto
/// cobre* × *até onde um número digitado é aceito* — eram um número só. O gate afirma a
/// separação onde ela agora existe, e afirma-a como RELAÇÃO (`hard > soft`), nunca repetindo
/// o literal do nó: uma segunda cópia do número aqui divergiria no dia em que a medição o
/// mover, e é a medição que manda (§0).
#[test]
fn the_typed_ceiling_reaches_past_the_slider() {
    for (ty, param) in [
        ("motion.grid", "rows"),
        ("motion.grid", "cols"),
        ("motion.fibonacci", "count"),
        ("motion.distribute_radial", "count"),
        ("motion.scatter", "count"),
        ("motion.distribute_curve", "count"),
        ("motion.lattice", "rows"),
        ("motion.lattice", "cols"),
        ("motion.kaleidoscope", "segments"),
        ("motion.boids", "count"),
        ("motion.verlet_rope", "count"),
        ("motion.clone", "count"),
        ("motion.pin_constraint", "first"),
        ("motion.pin_constraint", "count"),
    ] {
        let r = row_of(ty, param);
        assert!(
            r.hard_max > r.max,
            "{ty}.{param}: a caixa tem de ir ALÉM do slider — soft {} contra hard {}",
            r.max,
            r.hard_max
        );
    }
}

/// **A PREMISSA deste gate morreu, e ele é o registro disso** (doc 89 W1 · §0.0).
///
/// Ele afirmava que o boids — *uma simulação `O(n²)`* — não podia oferecer a caixa de um gerador
/// linear, e ⚠️ **a LEI continua certa**: o irmão `the_quadratic_node_keeps_a_tight_ceiling` a
/// sustenta sobre o `scatter`, que é quadrático em **toda** configuração e **não tem escape**.
///
/// O que mudou foi o SUJEITO. Os `10,392 ms por 2.000 agentes` eram do caminho de **REFERÊNCIA**
/// (o `Cook` do registry); o nó tem `register_grid` + kernel WGSL e a GPU shipa **ligada por
/// default**. Medido pela porta do produto
/// (`gpu_boids_scale::where_the_flock_leaves_the_frame_budget`): nos MESMOS 2.000 agentes o
/// device custa **0,476 ms** — 21,8× —, e com a densidade limitada ele faz **1.048.576 em
/// 14,283 ms**, que é o que o `PH2D_GPU_COOK_DEMO=7` shipa. *O caminho mais lento definia o teto
/// do mais rápido*, que é o caso literal do §0.0.
///
/// ⚠️ **E a cerca fez o trabalho dela:** ela não impediu a mudança, ela exigiu a MEDIÇÃO. O que
/// ela proibia — "harmonizar" um teto com o dos lineares sem medir — continua proibido; o que
/// aconteceu foi o oposto.
///
/// ⚠️ **O preço da configuração PADRÃO fica NOMEADO, porque ele não sumiu:** sem `spread` a
/// semeadura é uma caixa fixa, a grade não acelera nada e **o device também é `O(N²)`**, saindo
/// do quadro entre 32.768 e 65.536. É por isso que o gate que sobra afirma o **ESCAPE**: o teto
/// de um milhão só é honesto enquanto existe uma configuração que o alcança dentro do quadro, e
/// apagar `spread` tornaria o teto uma promessa que nenhum regime cumpre. Quem afirma o outro
/// lado — *o número que o demo SHIPA tem de ser digitável* — é
/// `the_boid_count_ceiling_quotes_the_device_not_the_reference_path`.
/// ⚠️ **A pergunta é se o artista ALCANÇA o escape, não se o manifesto o declara** — por isso
/// esta varredura passa pela mesma porta do painel que o `row_of`, e não pelo registry: um
/// `spread` declarado e não pintado seria um escape que só o código conhece.
#[test]
fn the_raised_boid_ceiling_still_has_the_escape_that_justifies_it() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("motion.boids");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    let rows = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("motion.boids resolve")
        .rows;
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    let has_escape = rows.iter().any(|r| match r {
        ParamRow::Scalar(s) => s.name == "spread",
        ParamRow::Toggle(t) => t.name == "spread",
        _ => false,
    });
    assert!(
        has_escape,
        "o `spread` do boids e a UNICA configuracao em que o teto de um milhao cabe num quadro; \
         sem ele na tela o teto vira promessa que nenhum regime alcancavel cumpre"
    );
}

/// **E o teto do `scatter` continua APERTADO, porque ali ele é um RECURSO.**
///
/// O blue noise por best-candidate é **O(count²)** e o cook mediu o quadro de 60 fps quebrando
/// entre 3.000 (11,4 ms) e 4.000 (20,7 ms) — a 100.000 ele custa **12,3 segundos**. Este gate
/// existe para que ninguém "harmonize" este teto com o **1.000.000** que os nós LINEARES da
/// mesma wave receberam: os dois números descrevem coisas diferentes, e uniformizá-los daria ao
/// artista uma caixa que aceita um valor capaz de congelar o app por minutos.
///
/// ⚠️ A barra é a contagem MEDIDA em que o quadro quebra, não um número escolhido.
#[test]
fn the_quadratic_node_keeps_a_tight_ceiling() {
    const FRAME_BREAKS_AT: f64 = 4_000.0; // medido: 20,661 ms > 16,6 ms
    let r = row_of("motion.scatter", "count");
    assert!(
        r.hard_max < FRAME_BREAKS_AT,
        "o teto do scatter ({}) tem de ficar ABAIXO da contagem em que o quadro quebra \
         ({FRAME_BREAKS_AT}) — ele é limite de RECURSO, não freio ergonômico",
        r.hard_max
    );
    // E o CONTROLE, a metade que impede a leitura preguiçosa "então aperte todo mundo": um nó
    // LINEAR desta mesma wave alcança MUITO mais longe, porque a medição dele disse isso.
    let linear = row_of("motion.fibonacci", "count");
    assert!(
        linear.hard_max > r.hard_max * 100.0,
        "um nó linear tem de alcançar muito além do quadrático — {} contra {}",
        linear.hard_max,
        r.hard_max
    );
}
