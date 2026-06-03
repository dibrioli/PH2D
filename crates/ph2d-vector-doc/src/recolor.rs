//! Apply a solid fill color to the selected networks — the document-side
//! of **T2.4 Color** (Coord handoff `c8ad559` §2).
//!
//! The Coord owns the picker UI + the `ph2d-panel-vector-inspector` host +
//! the read-back into `App.vector_fill_color`; this is the by-ref document
//! edit it calls: for each selected network, allocate a fresh fill in the
//! style table and point every region at it via a logged
//! [`VectorOp::SetRegionFill`] — so the recolor rides the same
//! event-sourced edit_log as everything else and **undoes via
//! [`crate::EditLog::revert_last_op`]** (the redo replay re-applies it).
//!
//! sRGB8 → [`ph2d_color::OklchColor`] is converted **here, at the
//! boundary** (the picker speaks sRGB8; [`crate::FillSolid`] stores OKLCH),
//! via the canonical OKLab intermediate.

use ph2d_color::{OklabColor, OklchColor, SrgbRgba};

use crate::edit_log::VectorOp;
use crate::postcard_schema::Ph2dVectorAsset;
use crate::region::RegionId;
use crate::selection::VectorSelection;
use crate::style::{FillRef, FillSolid};

/// Apply solid fill `rgba` (sRGB8) to every region of every network in
/// `selection`. Each network gets a fresh fill entry + one logged
/// [`VectorOp::SetRegionFill`] per region (undoable). Networks with no
/// regions (open stroked paths) and stale indices are skipped.
pub fn apply_fill_to_selection(
    committed: &mut [Ph2dVectorAsset],
    selection: &VectorSelection,
    rgba: [u8; 4],
) {
    let color = srgb8_to_oklch(rgba);
    for &idx in &selection.networks {
        let Some(asset) = committed.get_mut(idx) else {
            continue;
        };
        if asset.network.regions.is_empty() {
            continue;
        }
        let fill_ref = asset.styles.insert_fill(FillSolid { color });
        let region_ids: Vec<RegionId> = asset.network.regions.iter().map(|r| r.id).collect();
        for id in region_ids {
            let _ = asset.edit_log.push_and_apply(
                VectorOp::SetRegionFill {
                    id,
                    fill: Some(fill_ref),
                },
                &mut asset.network,
            );
        }
    }
}

/// Live-preview recolor (NO log, NO new fill): set the **color** of every fill
/// slot referenced by the selected networks' regions IN PLACE, for real-time
/// picker feedback. Call [`apply_fill_to_selection`] once first to establish a
/// logged region→fill assignment (so the recolor stays undoable); this only
/// repaints the already-assigned slots as the user drags the picker, so it
/// never grows the style table or the edit_log. Regions with no fill yet are
/// skipped (the first logged apply gives them one). sRGB8 → OKLCH at the
/// boundary, same as [`apply_fill_to_selection`].
pub fn preview_fill_on_selection(
    committed: &mut [Ph2dVectorAsset],
    selection: &VectorSelection,
    rgba: [u8; 4],
) {
    let color = srgb8_to_oklch(rgba);
    for &idx in &selection.networks {
        let Some(asset) = committed.get_mut(idx) else {
            continue;
        };
        let refs: Vec<FillRef> = asset
            .network
            .regions
            .iter()
            .filter_map(|r| r.fill)
            .collect();
        for fref in refs {
            if let Some(slot) = asset.styles.fills.get_mut(&fref) {
                slot.color = color;
            }
        }
    }
}

/// Convert sRGB8 `[r, g, b, a]` → [`OklchColor`] via the canonical OKLab
/// intermediate (sRGB → linear → OKLab → polar OKLCH). Hue in **degrees**
/// `[0, 360)`, matching [`OklchColor`]'s contract.
#[must_use]
fn srgb8_to_oklch(rgba: [u8; 4]) -> OklchColor {
    let lab: OklabColor = OklabColor::from_linear(SrgbRgba(rgba).to_linear());
    let chroma = (lab.a * lab.a + lab.b * lab.b).sqrt();
    let mut hue = lab.b.atan2(lab.a).to_degrees();
    if hue < 0.0 {
        hue += 360.0;
    }
    OklchColor::new(lab.l, chroma, hue, lab.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cubic::VertexKind;
    use crate::network::VectorNetwork;
    use crate::region::WindingRule;
    use crate::style::StyleTable;
    use glam::Vec2;

    /// A filled triangle network (region fill = None initially).
    fn triangle_asset() -> Ph2dVectorAsset {
        let mut net = VectorNetwork::empty();
        let mut log = crate::EditLog::new();
        for (i, p) in [
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 80.0),
        ]
        .into_iter()
        .enumerate()
        {
            let _ = log.push_and_apply(
                VectorOp::AddVertex {
                    id: i as u32,
                    pos: p,
                    kind: VertexKind::Auto,
                },
                &mut net,
            );
        }
        for i in 0..3u32 {
            let _ = log.push_and_apply(
                VectorOp::AddSegment {
                    id: i,
                    start: i,
                    end: (i + 1) % 3,
                    tangents: crate::TangentsCubic::ZERO,
                },
                &mut net,
            );
        }
        let _ = log.push_and_apply(
            VectorOp::AddRegion {
                id: 0,
                segments: [(0u32, true), (1, true), (2, true)].into_iter().collect(),
                winding: WindingRule::NonZero,
            },
            &mut net,
        );
        let mut asset = Ph2dVectorAsset::from_network(net, StyleTable::default());
        asset.edit_log = log;
        asset
    }

    fn fill_color_of(asset: &Ph2dVectorAsset) -> Option<OklchColor> {
        let fill_ref = asset.network.regions.first()?.fill?;
        asset.styles.fills.get(&fill_ref).map(|f| f.color)
    }

    #[test]
    fn applies_fill_to_selected_network() {
        let mut committed = vec![triangle_asset()];
        let mut sel = VectorSelection::new();
        sel.select_only_network(0);
        apply_fill_to_selection(&mut committed, &sel, [200, 40, 40, 255]);
        let color = fill_color_of(&committed[0]).expect("region got a fill");
        // Round-trips back to ~the input sRGB (±2 per channel).
        let back = color.to_srgb();
        assert!(back.r().abs_diff(200) <= 2, "r={}", back.r());
        assert!(back.g().abs_diff(40) <= 2, "g={}", back.g());
        assert!(back.b().abs_diff(40) <= 2, "b={}", back.b());
    }

    #[test]
    fn recolor_is_logged_and_undoable() {
        let mut committed = vec![triangle_asset()];
        let mut sel = VectorSelection::new();
        sel.select_only_network(0);
        let ops_before = committed[0].edit_log.ops.len();
        apply_fill_to_selection(&mut committed, &sel, [10, 200, 30, 255]);
        assert_eq!(
            committed[0].edit_log.ops.len(),
            ops_before + 1,
            "one SetRegionFill logged"
        );
        // Undo reverts the fill (back to None — the pre-recolor state).
        let asset = &mut committed[0];
        asset.edit_log.revert_last_op(&mut asset.network);
        assert_eq!(asset.network.regions[0].fill, None, "fill undone");
    }

    #[test]
    fn skips_networks_without_regions_and_stale_indices() {
        // An open path (no region) + a stale index → no panic, no change.
        let mut open = triangle_asset();
        open.network.regions.clear();
        let mut committed = vec![open];
        let mut sel = VectorSelection::new();
        sel.networks = vec![0, 9]; // 0 has no region; 9 is stale
        apply_fill_to_selection(&mut committed, &sel, [1, 2, 3, 255]);
        assert!(committed[0].network.regions.is_empty());
        // No fills inserted (nothing to fill).
        assert!(committed[0].styles.fills.is_empty());
    }

    #[test]
    fn preview_recolors_in_place_without_logging() {
        let mut committed = vec![triangle_asset()];
        let mut sel = VectorSelection::new();
        sel.select_only_network(0);
        // Establish a logged fill first (one op, one new fill).
        apply_fill_to_selection(&mut committed, &sel, [10, 20, 30, 255]);
        let ops = committed[0].edit_log.ops.len();
        let fills = committed[0].styles.fills.len();
        // Preview a different color: recolors in place — NO new op, NO new fill.
        preview_fill_on_selection(&mut committed, &sel, [200, 40, 40, 255]);
        assert_eq!(committed[0].edit_log.ops.len(), ops, "preview must not log");
        assert_eq!(
            committed[0].styles.fills.len(),
            fills,
            "preview must not insert a fill"
        );
        let back = fill_color_of(&committed[0])
            .expect("region has fill")
            .to_srgb();
        assert!(back.r().abs_diff(200) <= 2, "r={}", back.r());
        assert!(back.g().abs_diff(40) <= 2, "g={}", back.g());
        assert!(back.b().abs_diff(40) <= 2, "b={}", back.b());
    }

    #[test]
    fn white_and_black_round_trip() {
        let w = srgb8_to_oklch([255, 255, 255, 255]);
        assert!(w.l > 0.98 && w.c < 0.01, "white → L≈1 C≈0, got {w:?}");
        let b = srgb8_to_oklch([0, 0, 0, 255]);
        assert!(b.l < 0.02, "black → L≈0, got {b:?}");
    }
}
