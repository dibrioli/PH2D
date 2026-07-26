#![forbid(unsafe_code)]
//! ph2d-i18n — internationalization.
//!
//! **M13 stub.** This crate currently ships a **static English string
//! table** keyed by Fluent-style identifiers (`tool.trim_transparency.label`,
//! `tool.make_square.label`, …). The full Fluent/ICU MessageFormat
//! implementation is deferred per the milestone plan; this stub
//! gives consumers a stable call-site shape (`tr("key")`) that the
//! eventual Fluent bundle replaces transparently.
//!
//! # Why a stub, not Fluent right now
//!
//! Fluent requires runtime locale state + per-locale bundles +
//! plural-form resolution + a bundler-source story. None of those are
//! load-bearing for the current milestone (single-locale en-US editor).
//! Shipping the table now centralizes the **strings** so the Fluent
//! migration is a one-touch impl swap, not a 100-callsite rename.
//!
//! # Usage
//!
//! ```
//! use ph2d_i18n::tr;
//! // Image-tool labels are abreviados (cap 5 chars) pra caber no chip;
//! // o tooltip mantém o nome legível por extenso.
//! assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
//! assert_eq!(tr("tool.trim_transparency.tooltip"), "Trim Transparency");
//! assert_eq!(tr("tool.unknown.key"), "tool.unknown.key"); // missing-key passthrough
//! ```

/// Look up a string by Fluent-style key. Missing keys round-trip the
/// key itself so missing entries are visible in the UI (debugging
/// aid) rather than silently rendering as empty.
///
/// Returns `&'static str` — current implementation is a compile-time
/// table. The Fluent migration will widen this to `String` (formatted
/// with arguments) at that point.
pub fn tr(key: &str) -> &'static str {
    match key {
        // Image Tools — action row pills. Labels abreviados (Enio
        // 2026-05-25): cabem na coluna do chip (44 px) sem clip; o
        // tooltip mantém o nome completo + descrição.
        "tool.trim_transparency.label" => "TRIM",
        "tool.trim_transparency.tooltip" => "Trim Transparency",
        "tool.make_square.label" => "SQUAR",
        "tool.make_square.tooltip" => "Make Square",
        "tool.bgremoval.label" => "BGRMV",
        "tool.bgremoval.tooltip" => "Background Removal · 3",
        "tool.real_size.label" => "SIZE",
        "tool.real_size.tooltip" => "Real Size · reset scale to 1:1",
        "tool.padding.label" => "PAD",
        "tool.padding.tooltip" => "Padding · expand or crop canvas edges",
        "tool.color_equalization.label" => "CEQ",
        "tool.color_equalization.tooltip" => {
            "Color Equalization · CLAHE + brightness/contrast/saturation + auto-WB"
        }
        "tool.equalize_sizes.label" => "EQSZ",
        "tool.equalize_sizes.tooltip" => {
            "Equalize Sizes · normalize selection to Max / Fixed / Grid target"
        }
        "tool.rasterize.label" => "RASTR",
        "tool.rasterize.tooltip" => {
            "Rasterize · bake scale + rotation into pixels (reset Transform)"
        }
        "tool.upscale.label" => "UPSC",
        "tool.upscale.tooltip" => "Upscale · resize image up 1x..16x (Lanczos3 / Nearest / xBR)",
        "tool.painter.label" => "PNTR",
        "tool.painter.tooltip" => {
            "Painter · sucessor do Procreate (brush engine GPU, history vetorial, MCP)"
        }
        // Image-edit undo affordance.
        "edit.undo.label" => "Undo",
        "edit.undo.image_edit.toast_hint" => "Undo: Cmd+Z",
        "edit.undo.image_edit.toast_done" => "Undone",
        "edit.undo.image_edit.toast_nothing_to_undo" => "Nothing to undo",
        // Toast strings — Trim / Make Square outcomes. Wiring goes
        // through `tr()` so the shell drainer doesn't hardcode the
        // English copies (HR-15). Format strings ("Trimmed → {w} × {h} px")
        // stay at call-site for now; the Fluent migration moves them
        // here as `format(key, args)`.
        "tool.trim_transparency.toast.nothing" => "Nothing to trim",
        "tool.trim_transparency.toast.unavailable" => "Trim unavailable for this sprite",
        "tool.make_square.toast.already_square" => "Sprite is already square",
        "tool.make_square.toast.unavailable" => "Make Square unavailable for this sprite",
        // Timeline panel (W2.E9) — dope-sheet + graph editor + transport chrome.
        // English by canon (feedback_app_ui_english_only); routed through tr() so
        // the strings live in one table for the eventual Fluent migration.
        "panel.timeline.title" => "Timeline",
        "panel.timeline.summary" => "Summary",
        "panel.timeline.add_track" => "+ Track  \u{25be}",
        "panel.timeline.add_lane" => "+ Lane",
        "panel.timeline.add_container" => "+ Container",
        "panel.timeline.crumb_root" => "Scene",
        // **Where the open container's interior actually plays, in SCENE seconds.**
        // Inside a container the ruler counts the INTERIOR while the transport chip keeps
        // showing the scene's second, so two different numbers sit on screen at once. The
        // research found that no product labels its ruler at all — this readout is what
        // makes the two legible as one arithmetic instead of a contradiction.
        "panel.timeline.host_window" => "plays",
        // ...and when it does not play at the current second, which is WHY the playhead is
        // absent rather than broken.
        "panel.timeline.host_not_playing" => "not playing here",
        // ...and when it plays NOWHERE, which is a different fact: the container exists and
        // is authorable, but no strip instances it, so the ruler is counting its own seconds
        // rather than relating them to any. Printing "plays 0.00 - 3.00" here would label the
        // container's own axis with the scene's name.
        "panel.timeline.host_not_placed" => "not placed",
        "panel.timeline.add_marker" => "+M",
        "panel.timeline.time_seconds" => "Time(s)",
        "panel.timeline.length" => "Dur(s)",
        "panel.timeline.frame" => "Frame",
        "panel.timeline.loop" => "Loop",
        "panel.timeline.ping_pong" => "PingPong",
        "panel.timeline.autokey" => "AutoKey",
        "panel.timeline.record" => "Record",
        // Motion Path keying mode (ADR-0141): a new position key is a trajectory
        // point (on) or separate X/Y (off). "Path" fits the toggle's label column.
        "panel.timeline.motion_path" => "Path",
        "panel.timeline.snap" => "Snap",
        "panel.timeline.speed" => "Speed",
        // Says what the clock DRIVES, not what the scene contains — the scene
        // has physics bodies either way; this is whether Play steps them.
        "panel.timeline.physics" => "Physics",
        // Onion (ADR-0142): ghost poses of the selected object. "Onion" is the term
        // of art; "Keys" toggles pose-to-pose (neighbouring keyframes) vs t±k frames.
        "panel.timeline.onion" => "Onion",
        "panel.timeline.onion_keys" => "Keys",
        // Onion settings modal (ADR-0142 W3b): the floating card's title + row labels. Counts /
        // opacity / colours don't fit the transport bar, so the button opens this card.
        "panel.timeline.onion_settings" => "Onion Settings",
        "panel.timeline.onion_opacity" => "Opacity",
        "panel.timeline.onion_before" => "Ghosts Before",
        "panel.timeline.onion_after" => "Ghosts After",
        "panel.timeline.onion_color_before" => "Past",
        "panel.timeline.onion_color_after" => "Future",
        // The three view tabs, in the order things are assembled: keys make a
        // clip, clips make a container, containers and clips make the scene.
        // Named for what you SEE there, not for a mode you enter.
        "panel.timeline.tab.keys" => "Keys",
        "panel.timeline.tab.containers" => "Containers",
        "panel.timeline.tab.arrange" => "Arrange",
        "panel.timeline.prop.translate_x" => "Translate X",
        "panel.timeline.prop.translate_y" => "Translate Y",
        "panel.timeline.prop.rotation" => "Rotation",
        "panel.timeline.prop.scale_x" => "Scale X",
        "panel.timeline.prop.scale_y" => "Scale Y",
        "panel.timeline.prop.opacity" => "Opacity",
        "panel.timeline.prop.time" => "Time",
        "panel.timeline.prop.morph" => "Morph",
        "panel.timeline.prop.position" => "Position",
        // Per-track extrapolation badges (plan §6) — the dashed-region mode label,
        // shown on the dope-sheet only when the side is not the default Hold.
        "panel.timeline.extrap.loop" => "Loop",
        "panel.timeline.extrap.pingpong" => "Ping-Pong",
        "panel.timeline.extrap.continue" => "Continue",
        // ── Vector panel (ADR-0108/0112) — section headers, tool modes and the
        // shape catalogue. The panel is a 17-section stack; every section title
        // and every chrome word routes through here (the shape NAMES themselves
        // stay in the `ph2d-tool-vector` catalogue, which is their single source).
        "panel.vector.title" => "Vector",
        "panel.vector.section.tool" => "Tool",
        "panel.vector.section.shape" => "Shape",
        "panel.vector.section.blend" => "Blend",
        "panel.vector.section.morph" => "Morph",
        "panel.vector.section.envelope" => "Envelope",
        "panel.vector.section.textpath" => "Text on Path",
        "panel.vector.section.patternpath" => "Pattern on Path",
        "panel.vector.section.contour" => "Contour",
        "panel.vector.section.effects" => "Effects",
        // O Falloff modula a FORÇA do deformador abaixo dele na pilha; o card diz para onde ele
        // aponta, para que um Falloff sozinho (sem deformador abaixo) não pareça quebrado.
        "panel.vector.fx.falloff.modulates" => "modulates the effect below",
        "panel.vector.fx.falloff.inert" => "add a deformer below (Bulge/Warp/Zig Zag)",
        "panel.vector.section.stroke" => "Stroke",
        "panel.vector.section.fill" => "Fill",
        "panel.vector.section.fill_type" => "Fill Type",
        "panel.vector.section.snap" => "Snap",
        "panel.vector.section.transform" => "Transform",
        "panel.vector.section.vertex" => "Vertex",
        "panel.vector.section.boolean" => "Boolean",
        "panel.vector.section.expand" => "Expand",
        "panel.vector.section.align" => "Align",
        "panel.vector.section.arrange" => "Arrange",
        "panel.vector.section.path" => "Path",
        "panel.vector.section.text" => "Text",
        "panel.vector.section.font" => "Font",
        "panel.vector.section.paragraph" => "Paragraph",
        "panel.vector.section.axes" => "Axes",
        // A seção do CONECTOR — só aparece com um conector na seleção. Os RÓTULOS dos três
        // campos (Route / Jetty / Spread) vêm do catálogo em `ph2d-tool-vector::connector`,
        // que é a fonte única deles (a mesma regra do catálogo de formas).
        "panel.vector.section.connector" => "Connector",

        // ── Physics world panel (ADR-0131 D8 / W2b) — the WORLD half of physics
        // authoring. The per-BODY half is the Inspector's "Physics Body" section.
        // Labels say what the number DOES — but ⚠️ that cuts both ways: calling
        // the UNIFORM damping "Air Drag" is exactly what made the first smoke
        // fail (Enio: "todos os objetos grandes e pequenos caem na mesma
        // velocidade"). A label has to promise what the model can deliver.
        // Wet Tuning side panel (doc 22) — labels are the model app's own.
        "panel.wet_tuning.title" => "Wet Tuning",
        "panel.wet_tuning.group.paint" => "Paint",
        "panel.wet_tuning.group.water" => "Water",
        "panel.wet_tuning.group.physics" => "Physics",
        "panel.wet_tuning.group.tools" => "Tools",
        "panel.wet_tuning.group.paper" => "Paper",
        "panel.wet_tuning.group.experimental" => "Experimental",
        "panel.wet_tuning.km_mixing" => "Pigment mixing (K-M)",
        "panel.wet_tuning.km_glaze" => "Glaze layering (K-M)",
        "panel.wet_tuning.note" => {
            "Kubelka-Munk subtractive color. Further gated extensions (diffusion, backrun, fingering, dry-brush, render extras) ship neutral; see the tuning registry's hidden group."
        }
        "panel.wet_tuning.knob.pigmentPerDab" => "Pigment per dab",
        "panel.wet_tuning.knob.paperGate" => "Paper gate",
        "panel.wet_tuning.knob.felt" => "Felt (pores)",
        "panel.wet_tuning.knob.bristleCount" => "Bristle count",
        "panel.wet_tuning.knob.drag" => "Drag",
        "panel.wet_tuning.knob.pickup" => "Pickup",
        "panel.wet_tuning.knob.intensity" => "Intensity",
        "panel.wet_tuning.knob.bristleStrength" => "Bristle strength",
        "panel.wet_tuning.knob.bristleSize" => "Bristle size",
        "panel.wet_tuning.knob.spacing" => "Spacing",
        "panel.wet_tuning.knob.tipClean" => "Tip clean",
        "panel.wet_tuning.knob.blendForce" => "Blend force",
        "panel.wet_tuning.knob.gateSaturation" => "Gate saturation",
        "panel.wet_tuning.knob.waterPerDab" => "Water per dab",
        "panel.wet_tuning.knob.waterCap" => "Water cap",
        "panel.wet_tuning.knob.evaporation" => "Evaporation",
        "panel.wet_tuning.knob.rewet" => "Re-wet",
        "panel.wet_tuning.knob.retention" => "Retention",
        "panel.wet_tuning.knob.edgeDarkening" => "Edge darkening",
        "panel.wet_tuning.knob.baseEvaporation" => "Base evaporation",
        "panel.wet_tuning.knob.leveling" => "Leveling",
        "panel.wet_tuning.knob.capillary" => "Capillary",
        "panel.wet_tuning.knob.brake" => "Brake",
        "panel.wet_tuning.knob.gravity" => "Gravity",
        "panel.wet_tuning.knob.levelClamp" => "Level clamp",
        "panel.wet_tuning.knob.viscosity" => "Viscosity",
        "panel.wet_tuning.knob.maxVelocity" => "Max velocity",
        "panel.wet_tuning.knob.projection" => "Projection",
        "panel.wet_tuning.knob.brakeReach" => "Brake reach",
        "panel.wet_tuning.knob.capillaryGate" => "Capillary gate",
        "panel.wet_tuning.knob.eraser" => "Eraser",
        "panel.wet_tuning.knob.dryer" => "Dryer",
        "panel.wet_tuning.knob.blow" => "Blow",
        "panel.wet_tuning.knob.smear" => "Smear",
        "panel.wet_tuning.knob.wetLift" => "Rewet lift",
        "panel.wet_tuning.knob.extStaining" => "Staining",
        "panel.wet_tuning.knob.paperContrast" => "Contrast",
        "panel.wet_tuning.knob.paperFibres" => "Fibres",
        "panel.wet_tuning.knob.paperGrooves" => "Grooves",
        "panel.wet_tuning.knob.visualGrain" => "Visual grain",
        "panel.wet_tuning.knob.emboss" => "Emboss",
        "panel.wet_tuning.knob.paperVisibility" => "Paper visibility",
        "panel.physics.title" => "Physics",
        "panel.physics.section.world" => "World",
        "panel.physics.section.solver" => "Solver",
        // Two DIFFERENT models, and the section headers are what keeps them
        // apart: "Air Drag" scales with a body's cross-section and is resisted
        // by its mass (big things fall faster); "Damping" is a uniform velocity
        // decay that mass cannot enter (everything slows equally). Labelling
        // the uniform one "Air Drag" is what made the first smoke fail.
        "panel.physics.section.air" => "Air Drag",
        "panel.physics.section.damping" => "Damping",
        "panel.physics.air_drag" => "Density",
        "panel.physics.section.layers" => "Collision Layers",
        "panel.physics.section.sleep" => "Sleep",
        "panel.physics.section.debug" => "Debug",
        "panel.physics.gravity_x" => "Gravity X",
        "panel.physics.gravity_y" => "Gravity Y",
        "panel.physics.substeps" => "Sub-steps",
        "panel.physics.iterations" => "Iterations",
        "panel.physics.contact_hz" => "Contact Hz",
        "panel.physics.linear_damping" => "Linear",
        "panel.physics.angular_damping" => "Angular",
        "panel.physics.sleep_speed" => "Speed",
        "panel.physics.sleep_spin" => "Spin",
        "panel.physics.sleep_delay" => "Delay",
        "panel.physics.show_colliders" => "Show Colliders",
        "panel.physics.reset_defaults" => "Reset to Defaults",
        // The world scale is `ProjectSettings::pixels_per_meter` — a PROJECT
        // setting. This panel shows it so the metre-valued knobs above can be
        // read in pixels, and deliberately does not own or duplicate it (D4).
        "panel.physics.scale" => "Scale",
        "panel.physics.bodies" => "Bodies",
        "panel.vector.mode.build" => "Build",
        "panel.vector.mode.select" => "Select",
        "panel.vector.mode.node" => "Node",
        "panel.vector.mode.pen" => "Pen",
        "panel.vector.mode.shape" => "Shape",
        "panel.vector.mode.text" => "Text",
        "panel.vector.mode.connect" => "Connect",
        "panel.vector.mode.fillet" => "Fillet",
        "panel.vector.mode.chamfer" => "Chamfer",
        "panel.vector.category" => "Category",
        "panel.vector.shape.no_params" => "No parameters",
        // Stroke markers (arrowheads) — the two selectors in the STROKE section.
        // Only the ROW labels live here: the marker NAMES ("Arrow", "Diamond",
        // "Bar"…) come from `ph2d_vec_scene::Marker::label()`, which is their
        // single source — the same rule the shape catalogue follows.
        "panel.vector.marker.start" => "Start",
        "panel.vector.marker.end" => "End",
        "panel.vector.group.basic" => "Basic",
        "panel.vector.group.round" => "Round",
        "panel.vector.group.arrows" => "Arrows",
        "panel.vector.group.flow" => "Flow",
        "panel.vector.group.bubbles" => "Bubbles",
        "panel.vector.group.symbols" => "Symbols",
        "panel.vector.group.iso" => "3D",
        // Pass-through for unknown keys so the missing entry is
        // visible in the UI (the rendered key is intentionally ugly).
        _ => leak_key(key),
    }
}

/// Stub for the unknown-key path: leak the input into a `&'static`
/// so the return type stays uniform. Only fires on developer typos
/// (unknown keys at runtime), so the small per-typo leak is fine.
/// The Fluent migration replaces this with a `String` return.
fn leak_key(key: &str) -> &'static str {
    Box::leak(key.to_string().into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_keys_round_trip_to_english() {
        // Image-tool labels were abbreviated 2026-05-25 to fit the
        // 44-px chip column; tooltips keep the long English form.
        assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
        assert_eq!(tr("tool.trim_transparency.tooltip"), "Trim Transparency");
        assert_eq!(tr("tool.make_square.label"), "SQUAR");
        assert_eq!(tr("edit.undo.label"), "Undo");
        // Timeline panel chrome (W2.E9).
        assert_eq!(tr("panel.timeline.title"), "Timeline");
        assert_eq!(tr("panel.timeline.loop"), "Loop");
        assert_eq!(tr("panel.timeline.prop.translate_x"), "Translate X");
        // Vector panel chrome — section headers + the shape-catalogue words.
        assert_eq!(tr("panel.vector.section.tool"), "Tool");
        assert_eq!(tr("panel.vector.mode.shape"), "Shape");
        assert_eq!(tr("panel.vector.category"), "Category");
        assert_eq!(tr("panel.vector.shape.no_params"), "No parameters");
        assert_eq!(tr("panel.vector.group.iso"), "3D");
    }

    #[test]
    fn unknown_key_returns_the_key_itself() {
        let leaked = tr("tool.nonexistent.foo");
        assert_eq!(leaked, "tool.nonexistent.foo");
    }
}
