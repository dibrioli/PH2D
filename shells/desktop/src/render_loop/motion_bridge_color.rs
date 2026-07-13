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
