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
//! assert_eq!(tr("tool.trim_transparency.label"), "Trim Transparency");
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
        // Image Tools — action row pills.
        "tool.trim_transparency.label" => "Trim Transparency",
        "tool.trim_transparency.tooltip" => "Trim Transparency",
        "tool.make_square.label" => "Make Square",
        "tool.make_square.tooltip" => "Make Square",
        "tool.bgremoval.label" => "Bg Removal",
        "tool.bgremoval.tooltip" => "Background Removal · 3",
        "tool.real_size.label" => "Real Size",
        "tool.real_size.tooltip" => "Real Size · reset scale to 1:1",
        "tool.padding.label" => "Padding",
        "tool.padding.tooltip" => "Padding · expand or crop canvas edges",
        "tool.color_equalization.label" => "Color EQ",
        "tool.color_equalization.tooltip" => {
            "Color Equalization · CLAHE + brightness/contrast/saturation + auto-WB"
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
        assert_eq!(tr("tool.trim_transparency.label"), "Trim Transparency");
        assert_eq!(tr("tool.make_square.label"), "Make Square");
        assert_eq!(tr("edit.undo.label"), "Undo");
    }

    #[test]
    fn unknown_key_returns_the_key_itself() {
        let leaked = tr("tool.nonexistent.foo");
        assert_eq!(leaked, "tool.nonexistent.foo");
    }
}
