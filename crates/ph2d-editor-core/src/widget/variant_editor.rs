//! [`VariantEditor`] — a recursive editor for a dynamically-typed
//! value: a kind dropdown (`None`/`Str`/`Int`/`Float`/`Color`/`Dict`)
//! plus a per-kind value control. `Dict` nests child variants,
//! recursion capped at [`MAX_VARIANT_DEPTH`].
//!
//! Sprite Inspector v2 W6 (spec §15.7 T6.4.2, §7.12 depth cap).
//! Consumed by `InstanceShaderParams` (W4) and `NamedAnchor.user_data`
//! (W5). The recursive [`VariantValue`] is flattened to a list of
//! indented [`VariantRow`]s the consumer registers hits against; the
//! widget owns layout + paint + a11y, the section owns interaction.

use super::color_swatch::{ColorSwatch, SwatchSize, paint_color_swatch};
use super::dropdown::{Dropdown, DropdownOption, paint_dropdown_chip};
use crate::paint::{paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Recursion cap for nested `Dict` variants (spec §7.12). A child
/// beyond this depth is rendered as a collapsed summary, never as its
/// own editable sub-tree, so a pathological/cyclic structure can't
/// blow the layout.
pub const MAX_VARIANT_DEPTH: usize = 4;

/// Indent applied per nesting level.
const INDENT_PX: f32 = 16.0; // LITERAL-PX-OK: per-level dict indent
/// Fraction of the row width given to a dict child's key column.
const KEY_COL_FRACTION: f32 = 0.3; // LITERAL-PX-OK: layout proportion (key column share)
/// Fraction of the remaining row width given to the kind dropdown chip.
const KIND_CHIP_FRACTION: f32 = 0.45; // LITERAL-PX-OK: layout proportion (kind chip share)

/// The discriminant a [`VariantValue`] can take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantKind {
    None,
    Str,
    Int,
    Float,
    Color,
    Dict,
}

impl VariantKind {
    /// Every kind, in canonical dropdown order.
    pub const ALL: [VariantKind; 6] = [
        VariantKind::None,
        VariantKind::Str,
        VariantKind::Int,
        VariantKind::Float,
        VariantKind::Color,
        VariantKind::Dict,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            VariantKind::None => "None",
            VariantKind::Str => "Text",
            VariantKind::Int => "Integer",
            VariantKind::Float => "Float",
            VariantKind::Color => "Color",
            VariantKind::Dict => "Dictionary",
        }
    }
}

/// A dynamically-typed value. `Dict` makes this recursive.
#[derive(Clone, Debug, PartialEq)]
pub enum VariantValue {
    None,
    Str(String),
    Int(i64),
    Float(f64),
    Color([u8; 4]),
    Dict(Vec<(String, VariantValue)>),
}

impl VariantValue {
    pub fn kind(&self) -> VariantKind {
        match self {
            VariantValue::None => VariantKind::None,
            VariantValue::Str(_) => VariantKind::Str,
            VariantValue::Int(_) => VariantKind::Int,
            VariantValue::Float(_) => VariantKind::Float,
            VariantValue::Color(_) => VariantKind::Color,
            VariantValue::Dict(_) => VariantKind::Dict,
        }
    }

    /// Human-readable one-line preview of the value portion (no key).
    pub fn preview(&self) -> String {
        match self {
            VariantValue::None => "—".to_string(),
            VariantValue::Str(s) => format!("\"{s}\""),
            VariantValue::Int(n) => n.to_string(),
            VariantValue::Float(f) => crate::interaction::format_number(*f),
            VariantValue::Color(rgba) => {
                format!(
                    "#{:02X}{:02X}{:02X}{:02X}",
                    rgba[0], rgba[1], rgba[2], rgba[3]
                )
            }
            VariantValue::Dict(entries) => format!("{{{}}}", entries.len()),
        }
    }
}

/// One flattened row of a (possibly nested) variant tree.
#[derive(Clone, Debug, PartialEq)]
pub struct VariantRow {
    /// Nesting level (0 = root). Indent = `depth * INDENT_PX`.
    pub depth: usize,
    /// Dictionary key for this row, when it is a child of a `Dict`.
    pub key: Option<String>,
    /// The kind shown in the row's dropdown.
    pub kind: VariantKind,
    /// One-line preview of the scalar value (empty for `Dict`).
    pub preview: String,
    /// True when this row was clamped at [`MAX_VARIANT_DEPTH`] and its
    /// children are summarized rather than expanded.
    pub clamped: bool,
}

/// Flatten a variant tree into pre-order rows, capping recursion at
/// [`MAX_VARIANT_DEPTH`]. The root is row 0 with no key.
pub fn flatten(value: &VariantValue) -> Vec<VariantRow> {
    let mut out = Vec::new();
    push_rows(value, 0, None, &mut out);
    out
}

fn push_rows(value: &VariantValue, depth: usize, key: Option<String>, out: &mut Vec<VariantRow>) {
    let is_dict = matches!(value, VariantValue::Dict(_));
    let clamped = is_dict && depth >= MAX_VARIANT_DEPTH;
    out.push(VariantRow {
        depth,
        key,
        kind: value.kind(),
        preview: value.preview(),
        clamped,
    });
    if let VariantValue::Dict(entries) = value
        && !clamped
    {
        for (k, v) in entries {
            push_rows(v, depth + 1, Some(k.clone()), out);
        }
    }
}

#[derive(Clone, Debug)]
pub struct VariantEditor {
    /// Root id; per-row kind dropdowns derive ids from this via
    /// [`VariantEditor::row_kind_id`].
    pub id: NodeId,
    pub label: String,
    pub value: VariantValue,
}

impl VariantEditor {
    pub fn new(id: NodeId, label: impl Into<String>, value: VariantValue) -> Self {
        Self {
            id,
            label: label.into(),
            value,
        }
    }

    /// Deterministic kind-dropdown id for row `i`, offset from the
    /// root id. The consumer must reserve `[id, id + rows)` so these
    /// don't collide with other widgets.
    pub fn row_kind_id(&self, row: usize) -> NodeId {
        NodeId(self.id.0 + 1 + row as u64)
    }

    pub fn rows(&self) -> Vec<VariantRow> {
        flatten(&self.value)
    }

    /// Hit rect for the kind dropdown of row `i`. `host` is the full
    /// row strip; `row_h` the per-row height. The dropdown sits after
    /// the row's indent and (optional) key column.
    pub fn kind_chip_rect(host: Rect, row: &VariantRow, row_index: usize, row_h: f32) -> Rect {
        let indent = row.depth as f32 * INDENT_PX;
        let key_w = if row.key.is_some() {
            host.w * KEY_COL_FRACTION
        } else {
            0.0
        };
        let x = host.x + indent + key_w;
        let y = host.y + row_index as f32 * row_h;
        let chip_w = ((host.w - indent - key_w) * KIND_CHIP_FRACTION).max(0.0);
        Rect::new(x, y, chip_w, row_h)
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let rows = self.rows();
        let mut b = NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h);
        for i in 0..rows.len() {
            b = b.child(self.row_kind_id(i));
        }
        b.build()
    }
}

/// Paint the flattened variant tree. The caller registers each row's
/// kind-chip hit rect (via [`VariantEditor::kind_chip_rect`]) so the
/// kind dropdown is clickable, and handles the per-kind value control.
#[allow(clippy::too_many_arguments)]
pub fn paint_variant_editor(
    editor: &VariantEditor,
    host: Rect,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let rows = editor.rows();
    let font = TypeToken::Sm.px();
    for (i, row) in rows.iter().enumerate() {
        let indent = row.depth as f32 * INDENT_PX;
        let row_y = host.y + i as f32 * row_h;
        // Key column (for dict children).
        let key_w = if row.key.is_some() {
            host.w * KEY_COL_FRACTION
        } else {
            0.0
        };
        if let Some(key) = &row.key {
            paint_text(
                text_system,
                scene,
                key,
                host.x + indent,
                row_y + (row_h - font) * 0.5,
                font,
                key_w,
                resolve(ColorToken::Text2, theme),
            );
        }
        // Kind dropdown chip.
        let chip = VariantEditor::kind_chip_rect(host, row, i, row_h);
        let dd = Dropdown::new(
            editor.row_kind_id(i),
            "Kind",
            VariantKind::ALL
                .iter()
                .map(|k| DropdownOption::new(editor.row_kind_id(i), *k, k.label()))
                .collect(),
        )
        .selected(row.kind);
        paint_dropdown_chip(&dd, chip, scene, text_system, theme);
        // Value control: a swatch for Color, otherwise a text preview.
        let val_x = chip.x + chip.w + Spacing::Sm.px();
        let val_w = (host.x + host.w - val_x).max(0.0);
        let val_rect = Rect::new(val_x, row_y, val_w, row_h);
        match row.kind {
            VariantKind::Color => {
                let rgba = parse_color_preview(&row.preview);
                let sw =
                    ColorSwatch::new(editor.row_kind_id(i), "Value", rgba).size(SwatchSize::Sm);
                paint_color_swatch(&sw, val_rect, scene, theme);
            }
            _ => {
                let txt = if row.clamped {
                    format!("{} (max depth)", row.preview)
                } else {
                    row.preview.clone()
                };
                paint_text(
                    text_system,
                    scene,
                    &txt,
                    val_rect.x,
                    val_rect.y + (row_h - font) * 0.5,
                    font,
                    val_rect.w,
                    resolve(ColorToken::Text1, theme),
                );
            }
        }
    }
}

/// Parse a `#RRGGBBAA` preview back to bytes; falls back to opaque
/// black on a malformed string.
fn parse_color_preview(s: &str) -> [u8; 4] {
    let hex = s.trim_start_matches('#');
    if hex.len() == 8
        && let Ok(n) = u32::from_str_radix(hex, 16)
    {
        return [(n >> 24) as u8, (n >> 16) as u8, (n >> 8) as u8, n as u8];
    }
    [0, 0, 0, 255]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested() -> VariantValue {
        VariantValue::Dict(vec![
            ("hp".to_string(), VariantValue::Int(100)),
            ("name".to_string(), VariantValue::Str("orc".to_string())),
            (
                "stats".to_string(),
                VariantValue::Dict(vec![("str".to_string(), VariantValue::Float(1.5))]),
            ),
        ])
    }

    #[test]
    fn kind_matches_value() {
        assert_eq!(VariantValue::None.kind(), VariantKind::None);
        assert_eq!(VariantValue::Int(1).kind(), VariantKind::Int);
        assert_eq!(VariantValue::Color([1, 2, 3, 4]).kind(), VariantKind::Color);
    }

    #[test]
    fn flatten_preorder_with_keys() {
        let rows = flatten(&nested());
        // root + hp + name + stats + stats.str = 5 rows.
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[0].key, None);
        assert_eq!(rows[1].key.as_deref(), Some("hp"));
        assert_eq!(rows[3].key.as_deref(), Some("stats"));
        assert_eq!(rows[4].depth, 2);
        assert_eq!(rows[4].key.as_deref(), Some("str"));
    }

    #[test]
    fn depth_is_capped() {
        // Build a dict nested deeper than MAX_VARIANT_DEPTH.
        let mut v = VariantValue::Int(0);
        for _ in 0..(MAX_VARIANT_DEPTH + 3) {
            v = VariantValue::Dict(vec![("child".to_string(), v)]);
        }
        let rows = flatten(&v);
        let max_depth = rows.iter().map(|r| r.depth).max().unwrap();
        assert!(
            max_depth <= MAX_VARIANT_DEPTH,
            "max depth {max_depth} exceeds cap"
        );
        assert!(
            rows.iter().any(|r| r.clamped),
            "deep dict should mark a clamped row"
        );
    }

    #[test]
    fn color_preview_round_trips() {
        let p = VariantValue::Color([0x12, 0x34, 0x56, 0x78]).preview();
        assert_eq!(p, "#12345678");
        assert_eq!(parse_color_preview(&p), [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn row_kind_ids_are_distinct() {
        let ed = VariantEditor::new(NodeId(500), "Params", nested());
        let ids: Vec<u64> = (0..ed.rows().len()).map(|i| ed.row_kind_id(i).0).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "row ids must be unique");
    }

    #[test]
    fn a11y_role_is_group() {
        let ed = VariantEditor::new(NodeId(500), "Params", nested());
        assert_eq!(ed.build_a11y(0.0, 0.0, 300.0, 120.0).role(), Role::Group);
    }

    fn smoke(value: VariantValue, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let ed = VariantEditor::new(NodeId(500), "Params", value);
        paint_variant_editor(
            &ed,
            Rect::new(0.0, 0.0, 300.0, 200.0),
            24.0,
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_scalars_and_dict() {
        smoke(VariantValue::Str("hello".to_string()), Theme::Forge);
        smoke(VariantValue::Color([200, 30, 30, 255]), Theme::Sunstone);
        smoke(nested(), Theme::Blueprint);
    }
}
