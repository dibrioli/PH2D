//! Widget NodeIds for the vector inspector (re-exported from editor-core).

pub use ph2d_editor_core::ids::{
    VECTOR_INSPECTOR_CLOSE, VECTOR_INSPECTOR_FILL_SWATCH, VECTOR_INSPECTOR_SHAPE_ELLIPSE,
    VECTOR_INSPECTOR_SHAPE_KIND, VECTOR_INSPECTOR_SHAPE_POLYGON, VECTOR_INSPECTOR_SHAPE_RECT,
    VECTOR_INSPECTOR_SHAPE_SPIRAL, VECTOR_INSPECTOR_SHAPE_STAR,
};

/// The five Shape-kind option ids in `ShapeKind::ALL` order (Rect, Ellipse,
/// Polygon, Star, Spiral) — index `i` ⇒ the tool's `ShapeKind::ALL[i]`. Kept in
/// this exact order so the panel's option index round-trips through the shell
/// bridge to `VectorShapeTool::set_kind` without a per-id lookup table.
pub const SHAPE_OPTION_IDS: [ph2d_a11y::NodeId; 5] = [
    VECTOR_INSPECTOR_SHAPE_RECT,
    VECTOR_INSPECTOR_SHAPE_ELLIPSE,
    VECTOR_INSPECTOR_SHAPE_POLYGON,
    VECTOR_INSPECTOR_SHAPE_STAR,
    VECTOR_INSPECTOR_SHAPE_SPIRAL,
];

/// Display labels for the five options, same index order as [`SHAPE_OPTION_IDS`].
/// App UI is English-only (see `feedback_app_ui_english_only`).
pub const SHAPE_OPTION_LABELS: [&str; 5] = ["Rectangle", "Ellipse", "Polygon", "Star", "Spiral"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_option_ids_and_labels_are_parallel_and_distinct() {
        assert_eq!(
            SHAPE_OPTION_IDS.len(),
            SHAPE_OPTION_LABELS.len(),
            "every option id needs exactly one label"
        );
        for (i, &a) in SHAPE_OPTION_IDS.iter().enumerate() {
            for &b in SHAPE_OPTION_IDS.iter().skip(i + 1) {
                assert_ne!(
                    a, b,
                    "option ids must be distinct so hit-routing is unambiguous"
                );
            }
        }
    }
}
