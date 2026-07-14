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
        "panel.timeline.add_marker" => "+M",
        "panel.timeline.time_seconds" => "Time(s)",
        "panel.timeline.frame" => "Frame",
        "panel.timeline.loop" => "Loop",
        "panel.timeline.ping_pong" => "PingPong",
        "panel.timeline.autokey" => "AutoKey",
        "panel.timeline.record" => "Record",
        "panel.timeline.snap" => "Snap",
        "panel.timeline.speed" => "Speed",
        "panel.timeline.prop.translate_x" => "Translate X",
        "panel.timeline.prop.translate_y" => "Translate Y",
        "panel.timeline.prop.rotation" => "Rotation",
        "panel.timeline.prop.scale_x" => "Scale X",
        "panel.timeline.prop.scale_y" => "Scale Y",
        "panel.timeline.prop.opacity" => "Opacity",
        "panel.timeline.prop.time" => "Time",
        // ── Vector panel (ADR-0108/0112) — section headers, tool modes and the
        // shape catalogue. The panel is a 17-section stack; every section title
        // and every chrome word routes through here (the shape NAMES themselves
        // stay in the `ph2d-tool-vector` catalogue, which is their single source).
        "panel.vector.title" => "Vector",
        "panel.vector.section.tool" => "Tool",
        "panel.vector.section.shape" => "Shape",
        "panel.vector.section.blend" => "Blend",
        "panel.vector.section.stroke" => "Stroke",
        "panel.vector.section.fill" => "Fill",
        "panel.vector.section.fill_type" => "Fill Type",
        "panel.vector.section.snap" => "Snap",
        "panel.vector.section.transform" => "Transform",
        "panel.vector.section.vertex" => "Vertex",
        "panel.vector.section.boolean" => "Boolean",
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
        "panel.vector.mode.build" => "Build",
        "panel.vector.mode.select" => "Select",
        "panel.vector.mode.node" => "Node",
        "panel.vector.mode.pen" => "Pen",
        "panel.vector.mode.shape" => "Shape",
        "panel.vector.mode.text" => "Text",
        "panel.vector.mode.connect" => "Connect",
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
