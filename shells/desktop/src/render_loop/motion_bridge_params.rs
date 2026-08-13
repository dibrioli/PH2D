//! Params-panel side of the Motion bridge (split out of `motion_bridge.rs` to
//! keep each file under the HR-18 LOC cap). Owns the selected node's param edits
//! and the params-panel snapshot: scalar sliders, named enum / checkbox rows,
//! and the OKLCH colour-swatch authoring (the sRGB↔linear boundary lives here).
//!
//! The whole module is gated on both Motion panels — its parent declares it with
//! the same `all(panel-motion-graph, panel-motion-params)` cfg — so the helpers
//! carry no per-fn gate. Entry points are `pub(super)` for `dispatch`; the snapshot
//! builder is `pub(crate)` so a sibling shape test can drive it (ADR-0154 gates).

use super::color::{color_groups, linear_rgba_to_srgb8};
use crate::motion_state::MotionState;
/// The unit vocabulary (what a param's number IS) and the panel's display face
/// (how it reads). Module scope because `display_face` — the single conversion
/// point — sits beside `contain`, not inside the builder.
use ph2d_node_registry::ParamUnit;
use ph2d_panel_motion_params::RowDisplay;

/// The peek/stream-reading helpers (the live number a wire drives, the live columns
/// the Custom picker offers) — a child so `motion_bridge.rs` stays under the cap.
#[path = "motion_bridge_params_stream.rs"]
mod params_stream;

/// The channel-aware magnitude presets (a sibling child, shell LOC cap).
#[path = "motion_bridge_params_channel.rs"]
mod params_channel;
/// Re-exported under its old path so the params tests (a sibling module) and this
/// module keep naming the presets `params::apply_channel_presets` after the split.
pub(super) use params_channel::apply_channel_presets;
use params_channel::channel_unit;

/// The **write-back** half — this frame's edits applied to the node, and the one
/// reader of a param's current value (a sibling child, shell LOC cap). The seam is
/// *what the panel sends back* × *what the panel sees*; the builder below is the
/// second half and never writes.
/// A FAIXA que uma row oferece — irmã de `params_channel` (o que a row É) e de
/// `params_stream` (o que os fios carregam). Cortada aqui pelo cap de 600 LOC da shell,
/// por ASSUNTO: as duas funções respondem *"até onde este controle alcança?"*.
#[path = "motion_bridge_params_range.rs"]
mod params_range;
use params_range::{channel_range_override, contain};

#[path = "motion_bridge_params_edit.rs"]
mod params_edit;
use params_edit::apply_param_edits;
/// Re-exported under its old path: `param_value` is read from OUTSIDE this module
/// (`field_gizmo`, and two sibling test modules) as `params::param_value`, so the
/// split must not move where callers name it.
pub(crate) use params_edit::param_value;

/// O número que um fio põe num param, para a SONDA de uma cena de smoke — a mesma porta
/// que a row dirigida lê (`params_stream::driven_value`), nunca uma segunda avaliação: uma
/// sonda que re-implementa a resolução fica CEGA à porta e passa a medir a si mesma.
/// Irmã de `source_options_for_tests`: `#[cfg(test)]`, porque só uma sonda a chama.
#[cfg(test)]
pub(crate) fn driven_value_for_probe(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> Option<(f32, ph2d_nodegraph::graph::NodeId)> {
    params_stream::driven_value(motion, node, param)
}

/// The picker's option list, for the sibling gate that pins the reserved-namespace
/// filter.  is private to this module; the gate lives with the other
/// panel-row gates, which is where a reader looks for "what does the picker offer?".
#[cfg(test)]
pub(crate) fn source_options_for_tests(motion: &MotionState) -> Vec<String> {
    params_stream::source_options(motion)
}

/// **Publish the params panel this frame** (M1.P1) — the dispatch's one call: apply the
/// selected node's edits, rebuild its snapshot, seed the colour swatches from it, hand it
/// to the panel. Needs BOTH motion panels (selection from the graph, rows to params); it
/// lives here rather than inline in `dispatch` so the shell dispatch stays under the LOC
/// cap and the params logic sits together.
pub(super) fn publish(
    motion: &mut MotionState,
    store: &mut ph2d_editor::interaction::WidgetStore,
    motion_active: bool,
    project: ph2d_editor::ProjectSettings,
) {
    if !motion_active {
        ph2d_panel_motion_params::set_current_params(None);
        return;
    }
    // Apply this frame's edits (colour picks + scalar sliders) BEFORE rebuilding, so the
    // panel reflects them; then seed each colour swatch's picker from the fresh snapshot.
    apply_param_edits(motion, store);
    let snap = build_params_snapshot(motion, project);
    if let Some(s) = &snap {
        super::color::seed_color_swatches(store, s);
    }
    ph2d_panel_motion_params::set_current_params(snap);
}

/// **The display face for one param** (doc 88, Wave A) — the single place a
/// declared [`ParamUnit`] becomes the number the artist reads.
///
/// [`ParamUnit::Length`] is the only unit that CONVERTS, and it converts through
/// the project's setting, never a constant of its own: the same
/// `pixels_per_meter` the sprite importer and the gizmo readouts use. Everything
/// else is stored in the unit it is shown in, so it gets a suffix and a scale of
/// exactly `1.0` — the neutral face, byte-identical to before this wave.
///
/// [`ParamUnit::FromChannel`] is resolved first, by asking the channel the node
/// currently drives; a node with no `channel` param cannot answer, and an
/// unanswerable unit is [`ParamUnit::None`] rather than a guess.
fn display_face(
    unit: ParamUnit,
    channel: Option<i32>,
    project: ph2d_editor::ProjectSettings,
) -> RowDisplay {
    let unit = match unit {
        ParamUnit::FromChannel => channel.map(channel_unit).unwrap_or_default(),
        other => other,
    };
    if let Some(fixed) = unit.fixed_suffix() {
        return RowDisplay::new(1.0, fixed);
    }
    if !unit.converts() {
        return RowDisplay::default();
    }
    // A world LENGTH: stored in metres, shown in the project's unit.
    RowDisplay::new(
        f64::from(
            project
                .display_unit
                .from_meters(1.0, project.pixels_per_meter),
        ),
        project.display_unit.suffix(),
    )
}

/// The single selected Motion node's `NodeId.0`, or `None` unless exactly one
/// node is selected (params edit a single node; multi-select is a later step).
pub(crate) fn selected_motion_node() -> Option<u32> {
    match ph2d_panel_motion_graph::current_graph_selection()[..] {
        [only] => Some(only),
        _ => None,
    }
}

/// Build the selected node's [`ParamsSnapshot`](ph2d_panel_motion_params::ParamsSnapshot)
/// (M1.P1): the display title + one row per declared `ParamSpec`, pairing each
/// with its `ParamUiHint` (range / widget / label) and its current value (the
/// per-instance override, else the manifest default). `None` unless exactly one
/// node is selected and resolvable.
pub(crate) fn build_params_snapshot(
    motion: &MotionState,
    project: ph2d_editor::ProjectSettings,
) -> Option<ph2d_panel_motion_params::ParamsSnapshot> {
    use ph2d_node_registry::ParamWidget;
    use ph2d_nodegraph::cook::OpResolver;
    use ph2d_nodegraph::graph::NodeId;
    use ph2d_panel_motion_params::{
        AngleRow, ChannelsRow, ColorRow, CurveRow, EnumRow, GradientRow, ParamRow, ParamsSnapshot,
        ScalarRow, SeedRow, SourceRow, TextRow, ToggleRow,
    };

    // The params panel shows the properties of whatever ONE subject is selected.
    // A backdrop is not a node (no manifest, never cooks), so its rows are built by
    // the module that owns backdrops shell-side.
    if let Some(snap) = super::backdrops::params_snapshot(motion) {
        return Some(snap);
    }
    // A collapsed subgraph is not a node either (doc 57) — its rows are its own.
    if let Some(snap) = super::subgraph::params_snapshot(motion) {
        return Some(snap);
    }

    let only = selected_motion_node()?;
    let nid = NodeId(only);
    let inst = motion.doc.graph.node(nid)?;
    let type_id = inst.type_id();
    let manifest = motion.registry.resolve(type_id)?.manifest();
    let title = motion
        .registry
        .ui_manifest(type_id)
        .map(|u| u.display_name.to_string())
        .unwrap_or_else(|| inst.type_name.clone());
    let hints = motion.registry.param_ui(type_id);
    let gates = motion.registry.param_gates(type_id);
    let gates_text = motion.registry.param_gates_text(type_id);
    let texts = motion.doc.graph.node_text_param_overrides(nid);
    let overrides = motion.doc.graph.node_param_overrides(nid);
    let value_of = |name: &str| -> f32 {
        if let Some(v) = overrides.and_then(|m| m.get(name)).copied() {
            return v;
        }
        manifest
            .params
            .iter()
            .find(|p| p.name == name)
            .map_or(0.0, |p| p.default)
    };
    // Conditional visibility (`ParamGate`): a param with a gate whose `when` value
    // is not one of its `values` is HIDDEN (filtered off both loops below), so a
    // `source.shape` shows only the controls its current `kind` uses.
    // E a mesma pergunta do outro lado da fronteira f32 (`ParamGateText`): um
    // param cuja condição é a PRESENÇA de um TEXT param — o nome de uma forma
    // desenhada. É o que faz os oito sliders de polígono de controle do
    // `motion.spline_wrap` sumirem quando o artista escolhe a curva que desenhou.
    let has_text = |name: &str| -> bool {
        texts
            .and_then(|m| m.get(name))
            .is_some_and(|v| !v.trim().is_empty())
    };
    let shown = |param: &str| -> bool {
        !gates
            .into_iter()
            .flatten()
            .any(|g| g.param == param && !g.values.contains(&(value_of(g.when).round() as i32)))
            && !gates_text
                .into_iter()
                .flatten()
                .any(|g| g.param == param && has_text(g.when_text) != g.when_present)
    };

    // Channels folded into a colour swatch (or into a `Channels` picker's `mode`) —
    // suppress their standalone rows.
    let mut consumed: Vec<&'static str> = color_groups(&motion.registry, type_id)
        .into_iter()
        .flatten()
        .collect();

    // The behaviour channel this node currently drives (`None` for a node that
    // declares no `channel` param) — it selects the magnitude rows' unit, and so
    // their widget range (see `channel_range_override`).
    let channel = manifest
        .params
        .iter()
        .any(|p| p.name == "channel")
        .then(|| value_of("channel").round() as i32);

    let mut rows: Vec<ParamRow> = Vec::new();

    // Text params (a `motion.expression` formula, a `field.remap` Curve) are NOT
    // `ParamSpec`s (f32-only), so they never appear in the manifest loop below — surface
    // each as a row FIRST, reading the graph's text channel (docs/Motion Nodes/32-33).
    // `Curve` rides the SAME text channel as `Text` but paints an interactive curve editor
    // (A1-ui): draggable handles over the serialized curve, never the string.
    for h in hints.into_iter().flatten().filter(|h| shown(h.param)) {
        // A named-channel picker (plan §1.1): the artist-facing face of a stream-column
        // TEXT param. It reads the live column (a text param) + its `mode` (an f32
        // param), matches them to a channel, and folds `mode` in (consumed, so it gets
        // no row of its own). Custom = no match → the raw text field is offered.
        if let ParamWidget::Channels {
            mode_param,
            channels,
        } = h.widget
        {
            let attr = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let mode = value_of(mode_param).round() as i32;
            let selected = channels
                .iter()
                .position(|c| c.column == attr && c.mode == mode)
                .unwrap_or(channels.len());
            // The live upstream columns the Custom picker offers (roadmap: dropdown
            // populated at runtime) — everything the curated channels already cover
            // is excluded, so the chips are the ADVANCED columns, not duplicates.
            let covered: std::collections::BTreeSet<&str> =
                channels.iter().map(|c| c.column).collect();
            let extra = params_stream::upstream_scalar_columns(motion, nid, &covered, &attr);
            rows.push(ParamRow::Channels(ChannelsRow {
                label: h.label.to_string(),
                text_param: h.param,
                mode_param,
                // Resolve to primitives so the panel needs no registry dependency.
                channels: channels
                    .iter()
                    .map(|c| (c.label, c.column, c.mode))
                    .collect(),
                selected,
                custom: attr,
                extra,
            }));
            consumed.push(mode_param);
            continue;
        }
        // A source picker (doc 65): a TEXT param that names a value the app published
        // into the external channel. The options are the LIVE published names — the same
        // `Cook::externals` the node reads from — so the artist picks a shape they drew
        // by name. `motion.pump.cook` holds the externals whether the graph cooked on the
        // CPU or the GPU (the shell republishes them every frame, ADR-0126-independent).
        if h.widget == ParamWidget::Source {
            let current = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let options = params_stream::source_options(motion);
            rows.push(ParamRow::Source(SourceRow {
                label: h.label.to_string(),
                param: h.param,
                options,
                current,
            }));
            continue;
        }
        // A Gradient editor (doc 85) — a `ColorRamp` in a text param (`serialize_gradient`),
        // the colour sibling of the Curve. The panel draws the bar + draggable stops from the
        // string; a stop's COLOUR is read back through the OKLCH picker below.
        if h.widget == ParamWidget::Palette {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            rows.push(ParamRow::Palette(ph2d_panel_motion_params::PaletteRow {
                name: h.param,
                label: h.label.to_string(),
                value,
            }));
            continue;
        }
        if h.widget == ParamWidget::Gradient {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            rows.push(ParamRow::Gradient(GradientRow {
                name: h.param,
                label: h.label.to_string(),
                value,
            }));
            continue;
        }
        if h.widget == ParamWidget::Text || h.widget == ParamWidget::Curve {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let name = h.param;
            let label = h.label.to_string();
            rows.push(if h.widget == ParamWidget::Curve {
                ParamRow::Curve(CurveRow { name, label, value })
            } else {
                ParamRow::Text(TextRow { name, label, value })
            });
        }
    }

    for spec in manifest.params.iter().filter(|s| shown(s.name)) {
        let hint = hints.and_then(|hs| hs.iter().find(|h| h.param == spec.name));
        // A `Color`-anchored param emits ONE swatch row for its 4 channels.
        if let Some(h) = hint
            && let ParamWidget::Color { channels } = h.widget
        {
            let lin = [
                value_of(channels[0]),
                value_of(channels[1]),
                value_of(channels[2]),
                value_of(channels[3]),
            ];
            rows.push(ParamRow::Color(ColorRow {
                label: h.label.to_string(),
                channels,
                srgb: linear_rgba_to_srgb8(lin),
            }));
            continue;
        }
        // A non-anchor colour channel is folded into its swatch — no scalar row.
        if consumed.contains(&spec.name) {
            continue;
        }
        // A boolean → a real checkbox; an enum → a named segmented selector; an
        // angle → a `deg` number box; a seed → a number box + re-roll button.
        if let Some(h) = hint {
            match h.widget {
                ParamWidget::Toggle => {
                    rows.push(ParamRow::Toggle(ToggleRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        on: value_of(spec.name) >= 0.5,
                    }));
                    continue;
                }
                ParamWidget::Enum { labels } => {
                    let selected = (value_of(spec.name).round().max(0.0) as usize)
                        .min(labels.len().max(1) - 1);
                    rows.push(ParamRow::Enum(EnumRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        selected,
                        labels,
                    }));
                    continue;
                }
                ParamWidget::Angle => {
                    // Degrees end to end — the param already stores what the
                    // `deg` box shows, so the row is a straight copy of the hint
                    // (widened to contain the value: a drag-scrub clamps to the
                    // registered range, which would eat an out-of-range angle).
                    let deg = value_of(spec.name);
                    let (min, max) = contain(h.min, h.max, deg);
                    rows.push(ParamRow::Angle(AngleRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        deg: f64::from(deg),
                        min_deg: f64::from(min),
                        max_deg: f64::from(max),
                        step_deg: f64::from(h.step),
                    }));
                    continue;
                }
                ParamWidget::Seed => {
                    let seed = value_of(spec.name).round();
                    let (min, max) = contain(h.min, h.max, seed);
                    rows.push(ParamRow::Seed(SeedRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        value: f64::from(seed),
                        min: f64::from(min),
                        max: f64::from(max),
                    }));
                    continue;
                }
                _ => {}
            }
        }
        // **A driven param shows the number the WIRE is putting in** (doc 58), not the
        // override the wire is overriding — the resolution order the cook uses, made visible.
        // Read from the cook's memo (`peek`), never by evaluating anything a second time.
        let driven = params_stream::driven_value(motion, nid, spec.name);
        let value = f64::from(driven.map_or_else(|| value_of(spec.name), |(v, _)| v));
        // **O nome vem da MESMA resolução que o número** — nunca de uma segunda consulta ao
        // `param_sources` (ver `driven_value`): dirigido É ter um nome, e é isso que impede
        // uma row com dono e sem fio.
        let driven_by = driven.and_then(|(_, src)| params_stream::driver_title(motion, src));
        // The row's face, resolved ONCE and applied to the whole tuple below.
        //
        // The widget answers the unit question first (`unit_of`), so an `Angle` /
        // `Seed` / `Enum` can never be told to scale by `pixels_per_meter` even
        // if a table entry says otherwise. A param with no hint at all still gets
        // to declare a unit — the hint's absence is a missing RANGE, not a
        // missing quantity.
        let face = display_face(
            ph2d_node_registry::unit_of(
                hint.map_or(ParamWidget::Slider, |h| h.widget),
                motion.registry.param_unit_declared(type_id, spec.name),
            ),
            channel,
            project,
        );
        // ⚠️ ONE `.in_display` for BOTH arms of the match — the conversion site is
        // the push, not the construction, so a third arm cannot be added without
        // one.
        rows.push(ParamRow::Scalar(
            (match hint {
                Some(h) => {
                    // A behaviour's magnitude range depends on the channel it drives
                    // (degrees on Rotation vs world units on X/Y) — the static hint
                    // can only describe one, so widen it for ergonomics …
                    let (min, max, step) = channel
                        .and_then(|ch| channel_range_override(&inst.type_name, spec.name, ch))
                        .unwrap_or((h.min, h.max, h.step));
                    // … then widen it again, unconditionally, so it CONTAINS the doc
                    // value. That is the correctness half: no clamp, no lie, no
                    // destroy-on-touch, whatever put the value there.
                    let (min, max) = contain(min, max, value_of(spec.name));
                    // The typed ceiling, when the node declared one wider than the
                    // drag range. `max` is the floor of it: a hard limit that sat
                    // BELOW the slider would silently un-type values the slider can
                    // still reach.
                    let hard_max = motion
                        .registry
                        .param_hard_max(type_id, spec.name)
                        .unwrap_or(max)
                        .max(max);
                    // The typed FLOOR, mirrored: a hard minimum that sat ABOVE the
                    // slider would silently un-type values the slider can still reach,
                    // so `min` is its ceiling exactly as `max` is the ceiling's floor.
                    let hard_min = motion
                        .registry
                        .param_hard_min(type_id, spec.name)
                        .unwrap_or(min)
                        .min(min);
                    ScalarRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        value,
                        min: f64::from(min),
                        hard_min: f64::from(hard_min),
                        max: f64::from(max),
                        hard_max: f64::from(hard_max),
                        step: f64::from(step),
                        integer: h.widget.is_integer(),
                        driven_by: driven_by.clone(),
                        // Neutral here on purpose: the face is applied to BOTH arms
                        // at the push below, so neither arm can carry a different one.
                        display: RowDisplay::default(),
                    }
                }
                // No hint → a neutral range around the param's manifest DEFAULT.
                //
                // **A range must never be a function of the value it ranges over.** A slider's
                // range is the SCALE the value is measured against, and a scale that grows with
                // what it measures is a positive feedback loop. This branch used to read
                // `max = value * 4`, whose fixed point sits at a quarter of the track: drag above
                // it and the value multiplied every frame — to billions in about a second — and
                // drag below it and it collapsed to zero. Neither is a drag; both are a runaway.
                // (Enio, smoke 2026-07-12: *"sliders chegam a bilhões e não arrastam linearmente"*.)
                //
                // The DEFAULT is a manifest constant, so the scale holds still while the knob
                // moves across it. `contain` may still widen the range to hold an out-of-range doc
                // value, which is idempotent — widening for a value the range already holds is a
                // no-op, so a drag inside the range never moves it.
                //
                // Every registered param is hinted (`every_scalar_row_comes_from_a_declared_hint`
                // is the gate), so this branch is the backstop for a node type that forgets one —
                // and a backstop must be INERT, not armed.
                None => {
                    let neutral = (spec.default.abs() * 4.0).max(10.0);
                    let (min, max) = contain(0.0, neutral, value_of(spec.name));
                    ScalarRow {
                        name: spec.name,
                        label: spec.name.to_string(),
                        value,
                        min: f64::from(min),
                        hard_min: f64::from(min),
                        max: f64::from(max),
                        hard_max: f64::from(max),
                        step: 0.1,
                        integer: false,
                        driven_by: driven_by.clone(),
                        display: RowDisplay::default(),
                    }
                }
            })
            .in_display(face),
        ));
    }
    // Quais params deste nó carregam um override — a resposta vem dos DOIS canais (o `f32` do
    // manifesto e o de texto), porque a pergunta do artista é "o que eu mexi neste nó?" e ele
    // não sabe (nem deveria) por qual canal cada param viaja.
    let mut modified: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if let Some(over) = motion.doc.graph.node_param_overrides(NodeId(only)) {
        modified.extend(over.keys().cloned());
    }
    if let Some(over) = motion.doc.graph.node_text_param_overrides(NodeId(only)) {
        modified.extend(over.keys().cloned());
    }
    // ── As SEÇÕES (doc 88 B3) ────────────────────────────────────────────────────────────
    // As rows chegam em ordem de manifesto; aqui elas são reordenadas por GRUPO, com as
    // soltas primeiro. `sort_by_key` é ESTÁVEL, então dentro de um grupo a ordem que o autor
    // do nó escreveu sobrevive — a alternativa (ordenar por nome) reescreveria a intenção dele.
    let order = motion.registry.param_group_order(type_id);
    let group_of = |row: &ParamRow| -> Option<&'static str> {
        row.params()
            .first()
            .and_then(|p| motion.registry.param_group(type_id, p))
    };
    rows.sort_by_key(|r| {
        group_of(r).map_or(0, |g| {
            1 + order.iter().position(|o| *o == g).unwrap_or(order.len())
        })
    });
    // Onde cada seção começa. Uma seção cujo grupo não produziu row nenhuma (todo param dela
    // escondido por um `ParamGate`) simplesmente não aparece — cabeçalho sem conteúdo é a
    // seção-morta irmã do botão-morto.
    let mut sections: Vec<(String, usize)> = Vec::new();
    let mut prev: Option<&'static str> = None;
    for (i, row) in rows.iter().enumerate() {
        let g = group_of(row);
        if g != prev
            && let Some(g) = g
        {
            sections.push((g.to_string(), i));
        }
        prev = g;
    }
    Some(ParamsSnapshot {
        node: only,
        title,
        rows,
        modified,
        sections,
    })
}
