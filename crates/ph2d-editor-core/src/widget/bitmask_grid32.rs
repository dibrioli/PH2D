//! [`BitmaskGrid32`] — a 32-bit mask rendered as a 4-column × 8-row
//! grid of [`Checkbox`](super::checkbox) cells, bit `n` labeled `n+1`.
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.4). Extracted from the inline
//! grid in the §8 Visibility section so it can be reused by both
//! `VisibilityLayer` and the camera `VisibilityCullMask`. The widget
//! owns geometry (`cell_rects`) + paint + a11y; the consuming section
//! registers each bit's hit rect and toggles the mask on click.

use super::checkbox::{Checkbox, CheckboxValue, paint_checkbox};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Columns in the bitmask grid.
pub const BITMASK_COLS: usize = 4;
/// Rows in the bitmask grid (`32 / BITMASK_COLS`).
pub const BITMASK_ROWS: usize = 32 / BITMASK_COLS;

#[derive(Clone, Debug)]
pub struct BitmaskGrid32 {
    pub id: NodeId,
    /// Caller-provided (localized) a11y label for the whole grid.
    pub label: String,
    /// The 32 per-bit hit ids, indexed by bit (0..32).
    pub bit_ids: [NodeId; 32],
    /// Current mask; bit `n` set → cell `n` checked.
    pub mask: u32,
    /// When false, all cells render in the disabled (unchecked-looking)
    /// state and the a11y node is non-interactive. The grid is still
    /// laid out so a parent can show it greyed.
    pub enabled: bool,
}

impl BitmaskGrid32 {
    pub fn new(id: NodeId, label: impl Into<String>, bit_ids: [NodeId; 32], mask: u32) -> Self {
        Self {
            id,
            label: label.into(),
            bit_ids,
            mask,
            enabled: true,
        }
    }

    pub fn enabled(mut self, yes: bool) -> Self {
        self.enabled = yes;
        self
    }

    pub fn is_set(&self, bit: usize) -> bool {
        bit < 32 && (self.mask >> bit as u32) & 1 == 1
    }

    /// Total height occupied by the grid given a per-cell height.
    pub fn grid_height(cell_h: f32) -> f32 {
        cell_h * BITMASK_ROWS as f32
    }

    /// Hit rect for a single bit cell. `origin` is the grid's top-left;
    /// `w` is the full grid width (split into [`BITMASK_COLS`] columns);
    /// `cell_h` is the per-row height. Column-major within a row:
    /// bit `n` sits at `(col = n % COLS, row = n / COLS)`.
    pub fn cell_rect(origin_x: f32, origin_y: f32, w: f32, cell_h: f32, bit: usize) -> Rect {
        let cell_w = w / BITMASK_COLS as f32;
        let col = bit % BITMASK_COLS;
        let row = bit / BITMASK_COLS;
        Rect::new(
            origin_x + cell_w * col as f32,
            origin_y + cell_h * row as f32,
            cell_w,
            cell_h,
        )
    }

    /// All 32 cell rects, indexed by bit.
    pub fn cell_rects(&self, origin_x: f32, origin_y: f32, w: f32, cell_h: f32) -> [Rect; 32] {
        std::array::from_fn(|bit| Self::cell_rect(origin_x, origin_y, w, cell_h, bit))
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut b = NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h);
        for id in self.bit_ids {
            b = b.child(id);
        }
        b.build()
    }
}

/// Paint the 32 checkbox cells. The caller is responsible for
/// registering each cell's hit rect (via [`BitmaskGrid32::cell_rect`])
/// so clicks toggle the bit.
#[allow(clippy::too_many_arguments)]
pub fn paint_bitmask_grid32(
    grid: &BitmaskGrid32,
    origin_x: f32,
    origin_y: f32,
    w: f32,
    cell_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    for bit in 0..32usize {
        let rect = BitmaskGrid32::cell_rect(origin_x, origin_y, w, cell_h, bit);
        let checked = grid.enabled && grid.is_set(bit);
        let cb = Checkbox::new(grid.bit_ids[bit], format!("{}", bit + 1)).value(if checked {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        });
        paint_checkbox(&cb, rect, scene, text_system, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> [NodeId; 32] {
        std::array::from_fn(|i| NodeId(100 + i as u64))
    }

    fn fixture() -> BitmaskGrid32 {
        BitmaskGrid32::new(NodeId(1), "Visibility Layer", ids(), 0b1011)
    }

    #[test]
    fn dimensions_are_4x8() {
        assert_eq!(BITMASK_COLS, 4);
        assert_eq!(BITMASK_ROWS, 8);
        assert_eq!(BITMASK_COLS * BITMASK_ROWS, 32);
    }

    #[test]
    fn is_set_reads_mask() {
        let g = fixture();
        assert!(g.is_set(0));
        assert!(g.is_set(1));
        assert!(!g.is_set(2));
        assert!(g.is_set(3));
        assert!(!g.is_set(31));
    }

    #[test]
    fn cell_rect_grid_layout() {
        // bit 0 → (col 0, row 0); bit 4 → (col 0, row 1); bit 5 → (col 1, row 1).
        let r0 = BitmaskGrid32::cell_rect(0.0, 0.0, 400.0, 24.0, 0);
        let r4 = BitmaskGrid32::cell_rect(0.0, 0.0, 400.0, 24.0, 4);
        let r5 = BitmaskGrid32::cell_rect(0.0, 0.0, 400.0, 24.0, 5);
        assert_eq!(r0.x, 0.0);
        assert_eq!(r0.y, 0.0);
        assert_eq!(r4.x, 0.0);
        assert_eq!(r4.y, 24.0);
        assert_eq!(r5.x, 100.0);
        assert_eq!(r5.y, 24.0);
    }

    #[test]
    fn grid_height_is_eight_rows() {
        assert_eq!(BitmaskGrid32::grid_height(24.0), 24.0 * 8.0);
    }

    #[test]
    fn a11y_role_is_group() {
        let node = fixture().build_a11y(0.0, 0.0, 400.0, 192.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(g: BitmaskGrid32, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_bitmask_grid32(&g, 0.0, 0.0, 400.0, 24.0, &mut scene, &mut text, theme);
    }

    #[test]
    fn paint_smoke_enabled() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().enabled(false), Theme::Blueprint);
    }
}
