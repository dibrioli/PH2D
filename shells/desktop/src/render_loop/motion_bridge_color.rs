//! **Colour authoring for the Motion params panel** — the sRGB↔linear boundary, in one
//! place (split from `motion_bridge_params.rs` for the shell's 600-LOC file cap). Declared
//! by `motion_bridge` as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.
//!
//! A colour crosses two spaces here, and that crossing is the whole reason this is a module:
//! the **wire is linear-straight** (what the nodes cook with) while the swatch and the OKLCH
//! picker speak **sRGB8** (what a human picks). Every conversion between the two lives below,
//! so there is exactly one place to be wrong — and one place the round-trip guard has to hold.

use crate::motion_state::MotionState;

/// The colour groups declared by a node type — the 4-channel RGBA param names
/// behind each [`ParamWidget::Color`](ph2d_node_registry::ParamWidget) hint.
pub(super) fn color_groups(
    registry: &ph2d_node_registry::NodeRegistry,
    type_id: ph2d_nodegraph::node::NodeTypeId,
) -> Vec<[&'static str; 4]> {
    use ph2d_node_registry::ParamWidget;
    registry
        .param_ui(type_id)
        .into_iter()
        .flatten()
        .filter_map(|h| match h.widget {
            ParamWidget::Color { channels } => Some(channels),
            _ => None,
        })
        .collect()
}

/// The current linear-straight values of a node's 4 colour channels (per-instance
/// override, else the manifest default). Shared by the read-back change-guard and
/// the snapshot builder so the swatch and the doc agree.
pub(super) fn channel_values(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    channels: [&'static str; 4],
) -> [f32; 4] {
    use ph2d_nodegraph::cook::OpResolver;
    let overrides = motion.doc.graph.node_param_overrides(nid);
    let manifest = motion
        .doc
        .graph
        .node(nid)
        .and_then(|i| motion.registry.resolve(i.type_id()))
        .map(|op| op.manifest());
    let value_of = |name: &str| -> f32 {
        if let Some(v) = overrides.and_then(|m| m.get(name)).copied() {
            return v;
        }
        manifest
            .and_then(|m| m.params.iter().find(|p| p.name == name))
            .map_or(0.0, |p| p.default)
    };
    [
        value_of(channels[0]),
        value_of(channels[1]),
        value_of(channels[2]),
        value_of(channels[3]),
    ]
}

/// Write a picked sRGB colour into a node's 4 linear-straight channel params
/// (RGB via the sRGB transfer function, alpha straight), re-cooking only when the
/// colour actually changed (the picker stays open across idle frames).
pub(super) fn apply_color_to_node(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    channels: [&'static str; 4],
    srgb: [u8; 4],
) {
    let cur = channel_values(motion, nid, channels);
    // Compare in the space the pick actually lives in — sRGB8, what the picker
    // reads and writes. Comparing the LINEAR values instead would fire on a doc
    // colour that is not an exact 8-bit round-trip (say `r = 0.5`): merely OPENING
    // the picker seeds it with the swatch's 8-bit display colour, the read-back
    // decodes to `0.50004…`, and the guard would see a change and quantize the
    // doc — an edit the artist never made, wrapped in an undo step.
    if srgb == linear_rgba_to_srgb8(cur) {
        return;
    }
    let new = srgb8_to_linear_rgba(srgb);
    for (name, v) in channels.into_iter().zip(new) {
        motion.doc.graph.set_param(nid, name, v);
    }
    motion.pump.mark_dirty();
}

/// Seed each colour swatch's `widget_color` from the snapshot's display colour
/// (the OKLCH picker reads it on open + the swatch paints it). Keyed by the
/// anchor channel — the same id the panel registers.
pub(super) fn seed_color_swatches(
    store: &mut ph2d_editor::interaction::WidgetStore,
    snap: &ph2d_panel_motion_params::ParamsSnapshot,
) {
    use ph2d_panel_motion_params::{ParamRow, param_swatch_id};
    for row in &snap.rows {
        if let ParamRow::Color(c) = row {
            store.set_widget_color(param_swatch_id(c.channels[0]), c.srgb);
        }
    }
}

/// Is a colour-swatch or gradient-stop picker open on this node right now? The undo-bracket
/// edge — a whole colour/stop pick coalesces into ONE step (like a slider drag). Read-only;
/// the caller opens the history bracket, THEN calls [`apply_picker_readback`] (writes must
/// land inside the bracket).
pub(super) fn picker_session(
    motion: &MotionState,
    sel: Option<ph2d_nodegraph::graph::NodeId>,
    groups: &[[&'static str; 4]],
    grad_params: &[&'static str],
    store: &ph2d_editor::interaction::WidgetStore,
) -> bool {
    use ph2d_panel_motion_params::param_swatch_id;
    let color = groups
        .iter()
        .any(|ch| store.picker_target() == Some(param_swatch_id(ch[0])));
    let grad = sel.is_some_and(|nid| {
        grad_params
            .iter()
            .any(|p| gradient_picker_stop(motion, nid, p, store).is_some())
    });
    color || grad
}

/// Feed the live OKLCH pick into the node it targets — a colour group's 4 channel params
/// ([`apply_color_to_node`]) or a gradient stop's colour in the ramp string
/// ([`apply_gradient_stop_pick`]). No-op when no picker is open. Must run INSIDE the undo
/// bracket [`picker_session`] opened.
pub(super) fn apply_picker_readback(
    motion: &mut MotionState,
    sel: Option<ph2d_nodegraph::graph::NodeId>,
    groups: &[[&'static str; 4]],
    grad_params: &[&'static str],
    store: &ph2d_editor::interaction::WidgetStore,
) {
    use ph2d_panel_motion_params::param_swatch_id;
    let Some(nid) = sel else { return };
    let pick = || store.blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER);
    for ch in groups {
        if store.picker_target() == Some(param_swatch_id(ch[0]))
            && let Some((value, _, _, _)) = pick()
        {
            apply_color_to_node(motion, nid, *ch, value.rgba);
        }
    }
    for p in grad_params {
        if let Some(stop) = gradient_picker_stop(motion, nid, p, store)
            && let Some((value, _, _, _)) = pick()
        {
            apply_gradient_stop_pick(motion, nid, p, stop, value.rgba);
        }
    }
}

/// The text params of a node type edited by a [`ParamWidget::Gradient`] (doc 85) — the
/// `ColorRamp` string keys whose per-stop swatches the picker read-back writes into.
pub(super) fn gradient_params(
    registry: &ph2d_node_registry::NodeRegistry,
    type_id: ph2d_nodegraph::node::NodeTypeId,
) -> Vec<&'static str> {
    use ph2d_node_registry::ParamWidget;
    registry
        .param_ui(type_id)
        .into_iter()
        .flatten()
        .filter_map(|h| (h.widget == ParamWidget::Gradient).then_some(h.param))
        .collect()
}

/// If the OKLCH picker open right now targets a stop swatch of this node's `param` gradient,
/// the stop's index — else `None`. The stop count comes from parsing the current string, and
/// its swatch id is [`ph2d_panel_motion_params::param_grad_swatch_id`] — the SAME id the
/// panel registers, so the two agree without sharing row order.
pub(super) fn gradient_picker_stop(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
    store: &ph2d_editor::interaction::WidgetStore,
) -> Option<usize> {
    let target = store.picker_target()?;
    let ramp = current_gradient(motion, nid, param);
    (0..ramp.len()).find(|&i| ph2d_panel_motion_params::param_grad_swatch_id(param, i) == target)
}

/// The current `ColorRamp` of a node's gradient text param (override, else the default
/// black→white ramp) — shared by the read-back's stop lookup + its re-serialize so the
/// swatch and the string agree.
fn current_gradient(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> ph2d_color::ColorRamp {
    motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(param))
        .and_then(|s| ph2d_color::parse_gradient(s))
        .unwrap_or_default()
}

/// Write a picked sRGB colour into `stop` of a node's gradient text param (RGB via the sRGB
/// transfer, alpha forced opaque — doc 85 stops carry no alpha), re-serializing the string
/// and re-cooking only when the colour actually changed (the picker stays open across idle
/// frames). The mirror of [`apply_color_to_node`] for a stop-in-a-string.
pub(super) fn apply_gradient_stop_pick(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &'static str,
    stop: usize,
    srgb: [u8; 4],
) {
    let mut ramp = current_gradient(motion, nid, param);
    let Some(cur) = ramp.stops().get(stop).copied() else {
        return;
    };
    // Compare in sRGB8 — the space the pick lives in — so merely OPENING the picker on a
    // colour that is not an exact 8-bit round-trip does not quantize the doc (the exact
    // guard `apply_color_to_node` documents). A stop is opaque (alpha 1.0 → byte 255).
    if srgb == linear_rgba_to_srgb8(cur.color) {
        return;
    }
    let lin = srgb8_to_linear_rgba(srgb);
    ramp.set_color(stop, [lin[0], lin[1], lin[2], 1.0]);
    motion
        .doc
        .graph
        .set_text_param(nid, param, ph2d_color::serialize_gradient(&ramp));
    motion.pump.mark_dirty();
}

/// sRGB8 (straight) → linear-straight RGBA `[0,1]` (the Motion wire space): RGB
/// through the sRGB transfer function, alpha a plain `/255`.
fn srgb8_to_linear_rgba(srgb: [u8; 4]) -> [f32; 4] {
    use ph2d_color::srgb::srgb_to_linear_byte;
    [
        srgb_to_linear_byte(srgb[0]),
        srgb_to_linear_byte(srgb[1]),
        srgb_to_linear_byte(srgb[2]),
        f32::from(srgb[3]) / 255.0,
    ]
}

/// Linear-straight RGBA `[0,1]` → sRGB8 (straight) for the swatch display /
/// picker seed: RGB through the linear→sRGB transfer, alpha a plain `×255`.
pub(super) fn linear_rgba_to_srgb8(lin: [f32; 4]) -> [u8; 4] {
    use ph2d_color::srgb::linear_to_srgb_byte;
    [
        linear_to_srgb_byte(lin[0]),
        linear_to_srgb_byte(lin[1]),
        linear_to_srgb_byte(lin[2]),
        (lin[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor::interaction::WidgetStore;
    use ph2d_panel_motion_params::param_grad_swatch_id;

    /// **The gradient read-back writes a stop's OKLCH pick back into the string** (doc 85).
    /// The bridge sees the `motion.color_ramp` Custom gradient (`gradient_params`), locates
    /// which stop the open picker targets (`gradient_picker_stop`, keyed by the SAME
    /// `param_grad_swatch_id` the panel registers), and re-serializes the `ColorRamp` with the
    /// picked colour (sRGB→linear). RED-first: drop the `set_text_param` in
    /// `apply_gradient_stop_pick` and stop 0 stays black.
    #[test]
    fn a_gradient_stop_pick_re_serializes_the_string() {
        let mut motion = crate::motion_state::MotionState::new();
        motion.doc = ph2d_motion_doc::MotionDoc::new();
        let nid = motion.doc.graph.add_node("motion.color_ramp");
        motion
            .doc
            .graph
            .set_text_param(nid, "ramp", "g1 2 0:0,0,0 1:1,1,1".to_string());

        // The bridge recognizes the Gradient widget as a colour text param.
        let tid = motion.doc.graph.node(nid).unwrap().type_id();
        assert_eq!(gradient_params(&motion.registry, tid), vec!["ramp"]);

        // Open the picker on stop 0's swatch — the SAME id the panel would register.
        let mut store = WidgetStore::with_capacity(4);
        store.set_picker_target(Some(param_grad_swatch_id("ramp", 0)));
        assert_eq!(
            gradient_picker_stop(&motion, nid, "ramp", &store),
            Some(0),
            "the picker targets stop 0"
        );

        // Pick pure red → the string's stop 0 becomes red (sRGB 255 → linear 1.0).
        apply_gradient_stop_pick(&mut motion, nid, "ramp", 0, [255, 0, 0, 255]);
        let value = motion
            .doc
            .graph
            .node_text_param_overrides(nid)
            .and_then(|m| m.get("ramp"))
            .cloned()
            .expect("ramp text param written");
        let ramp = ph2d_color::parse_gradient(&value).expect("valid gradient");
        assert!(
            ramp.stops()[0].color[0] > 0.9 && ramp.stops()[0].color[1] < 0.05,
            "stop 0 is red now: {:?}",
            ramp.stops()[0].color
        );
        // Stop 1 (white) is untouched — the pick lands on ONE stop.
        assert!(ramp.stops()[1].color[0] > 0.9 && ramp.stops()[1].color[2] > 0.9);
    }
}
