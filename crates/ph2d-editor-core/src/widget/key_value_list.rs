//! [`KeyValueList`] — an editable list of `key → value` rows with a
//! per-row remove button and a trailing "add" button.
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.8). Backs the
//! `InstanceShaderParams` editor (W4): each row maps a string key to a
//! value preview (the value control itself — a [`VariantEditor`] or a
//! scalar field — is wired by the consuming section against the ids
//! this widget exposes). The widget owns layout + paint + a11y.

use super::button::{Button, ButtonKind, paint_button};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Fraction of the row (minus the remove button) given to the key
/// column; the remainder is the value column.
const KEY_FRACTION: f32 = 0.4; // LITERAL-PX-OK: layout proportion (key column share of the row)

#[derive(Clone, Debug)]
pub struct KeyValueEntry {
    pub key_id: NodeId,
    pub value_id: NodeId,
    pub remove_id: NodeId,
    pub key: String,
    /// One-line preview of the value (the live value control is the
    /// consumer's responsibility, registered against `value_id`).
    pub value: String,
}

impl KeyValueEntry {
    pub fn new(
        key_id: NodeId,
        value_id: NodeId,
        remove_id: NodeId,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            key_id,
            value_id,
            remove_id,
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyValueList {
    pub id: NodeId,
    pub label: String,
    pub entries: Vec<KeyValueEntry>,
    /// Id of the trailing "add row" button.
    pub add_id: NodeId,
}

impl KeyValueList {
    pub fn new(
        id: NodeId,
        label: impl Into<String>,
        entries: Vec<KeyValueEntry>,
        add_id: NodeId,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            entries,
            add_id,
        }
    }

    /// Square side of the per-row remove button.
    fn remove_w(row_h: f32) -> f32 {
        row_h
    }

    pub fn row_rect(host: Rect, row_h: f32, i: usize) -> Rect {
        Rect::new(
            host.x,
            host.y + i as f32 * (row_h + Spacing::Xs.px()),
            host.w,
            row_h,
        )
    }

    pub fn key_rect(row: Rect, row_h: f32) -> Rect {
        let avail = (row.w - Self::remove_w(row_h) - Spacing::Xs.px() * 2.0).max(0.0);
        Rect::new(row.x, row.y, avail * KEY_FRACTION, row.h)
    }

    pub fn value_rect(row: Rect, row_h: f32) -> Rect {
        let avail = (row.w - Self::remove_w(row_h) - Spacing::Xs.px() * 2.0).max(0.0);
        let key_w = avail * KEY_FRACTION;
        Rect::new(
            row.x + key_w + Spacing::Xs.px(),
            row.y,
            avail - key_w,
            row.h,
        )
    }

    pub fn remove_rect(row: Rect, row_h: f32) -> Rect {
        let w = Self::remove_w(row_h);
        Rect::new(row.x + row.w - w, row.y, w, row.h)
    }

    /// Rect of the trailing "add" button (a row-height square at the
    /// left of the row below the last entry).
    pub fn add_button_rect(&self, host: Rect, row_h: f32) -> Rect {
        let y = host.y + self.entries.len() as f32 * (row_h + Spacing::Xs.px());
        Rect::new(host.x, y, row_h, row_h)
    }

    /// Total height occupied (entries + the add button row).
    pub fn total_height(&self, row_h: f32) -> f32 {
        (self.entries.len() + 1) as f32 * (row_h + Spacing::Xs.px())
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut b = NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h);
        for e in &self.entries {
            b = b.child(e.key_id).child(e.value_id).child(e.remove_id);
        }
        b.child(self.add_id).build()
    }
}

fn paint_field(
    text: &str,
    rect: Rect,
    placeholder: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    fill_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::Bg3, theme),
    );
    stroke_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    let font = TypeToken::Sm.px();
    let color = if placeholder {
        ColorToken::Text3
    } else {
        ColorToken::Text1
    };
    paint_text(
        text_system,
        scene,
        text,
        rect.x + Spacing::Sm.px(),
        rect.y + (rect.h - font) * 0.5,
        font,
        (rect.w - Spacing::Sm.px() * 2.0).max(0.0),
        resolve(color, theme),
    );
}

/// Paint the list. The caller registers each entry's `key_id`,
/// `value_id`, `remove_id` (via the `*_rect` helpers) plus `add_id`
/// (via [`KeyValueList::add_button_rect`]) for hit testing.
pub fn paint_key_value_list(
    list: &KeyValueList,
    host: Rect,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    for (i, entry) in list.entries.iter().enumerate() {
        let row = KeyValueList::row_rect(host, row_h, i);
        let key_rect = KeyValueList::key_rect(row, row_h);
        let value_rect = KeyValueList::value_rect(row, row_h);
        let remove_rect = KeyValueList::remove_rect(row, row_h);
        paint_field(
            &entry.key,
            key_rect,
            entry.key.is_empty(),
            scene,
            text_system,
            theme,
        );
        paint_field(
            &entry.value,
            value_rect,
            entry.value.is_empty(),
            scene,
            text_system,
            theme,
        );
        let remove = Button::new(entry.remove_id, "Remove").kind(ButtonKind::IconOnly {
            icon: IconId::Trash,
        });
        paint_button(&remove, remove_rect, scene, text_system, theme);
    }
    let add =
        Button::new(list.add_id, "Add parameter").kind(ButtonKind::IconOnly { icon: IconId::Add });
    paint_button(
        &add,
        list.add_button_rect(host, row_h),
        scene,
        text_system,
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> KeyValueList {
        KeyValueList::new(
            NodeId(1),
            "Shader Params",
            vec![
                KeyValueEntry::new(NodeId(10), NodeId(11), NodeId(12), "hue_shift", "0.25"),
                KeyValueEntry::new(NodeId(20), NodeId(21), NodeId(22), "outline", "#FF0000FF"),
            ],
            NodeId(99),
        )
    }

    #[test]
    fn columns_partition_row_without_overlap() {
        let row = KeyValueList::row_rect(Rect::new(0.0, 0.0, 300.0, 24.0), 24.0, 0);
        let k = KeyValueList::key_rect(row, 24.0);
        let v = KeyValueList::value_rect(row, 24.0);
        let r = KeyValueList::remove_rect(row, 24.0);
        assert!(k.x + k.w <= v.x + 0.01);
        assert!(v.x + v.w <= r.x + 0.01);
        assert!(r.x + r.w <= row.x + row.w + 0.01);
    }

    #[test]
    fn add_button_sits_below_last_entry() {
        let list = fixture();
        let host = Rect::new(0.0, 0.0, 300.0, 200.0);
        let add = list.add_button_rect(host, 24.0);
        let last = KeyValueList::row_rect(host, 24.0, list.entries.len() - 1);
        assert!(add.y > last.y);
    }

    #[test]
    fn total_height_counts_add_row() {
        let list = fixture();
        let rows = (list.entries.len() + 1) as f32;
        assert_eq!(list.total_height(24.0), rows * (24.0 + Spacing::Xs.px()));
    }

    #[test]
    fn a11y_role_is_group() {
        assert_eq!(
            fixture().build_a11y(0.0, 0.0, 300.0, 200.0).role(),
            Role::Group
        );
    }

    #[test]
    fn paint_smoke() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_key_value_list(
            &fixture(),
            Rect::new(0.0, 0.0, 300.0, 200.0),
            24.0,
            &mut scene,
            &mut text,
            Theme::Forge,
        );
    }

    #[test]
    fn paint_smoke_empty() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let empty = KeyValueList::new(NodeId(1), "Empty", vec![], NodeId(99));
        paint_key_value_list(
            &empty,
            Rect::new(0.0, 0.0, 300.0, 60.0),
            24.0,
            &mut scene,
            &mut text,
            Theme::Blueprint,
        );
    }
}
