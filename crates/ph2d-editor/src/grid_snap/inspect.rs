//! Read-only inspect subsection — shows the coord-system label,
//! cell IDs for probe A / B, and computed graph distance + line
//! length + neighbor count for the currently-active grid kind.
//!
//! v1 surfaces all values as labels (no interactive NumberInputs
//! for the probe coords yet). The probes default to
//! `[0, 0]` / `[3, 2]`; tweaking them in v1 requires editing
//! `GridSnapState::probe_a` / `probe_b` directly — a v2 release
//! adds NumberInput widgets at [`super::ids::GS_PROBE_A_X`] etc.

use super::state::{GridKind, GridSnapState};
use crate::paint::{paint_text, resolve};
use crate::widget::{SectionHeader, paint_section_header};
use crate::zones::Rect;
use ph2d_grid::GridMath;
use ph2d_grid::Vec2;
use ph2d_grid::hex::{HexCell, axial_to_cube, axial_to_offset};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Shift the probe pair into the active grid's local space by
/// subtracting the kind's origin offset. The cell IDs displayed
/// then reflect the snapped grid the user sees, not raw world.
fn local_probes(state: &GridSnapState) -> (Vec2, Vec2) {
    let o = state.active_origin();
    (
        [state.probe_a[0] - o[0], state.probe_a[1] - o[1]],
        [state.probe_b[0] - o[0], state.probe_b[1] - o[1]],
    )
}

const SECTION_HEADER_H: f32 = 22.0;
const ROW_H: f32 = 22.0;
const ROW_GAP: f32 = 2.0;
const LABEL_FONT_SIZE: f32 = 12.0;

/// Snapshot of computed values for the current probe pair.
#[derive(Clone, Debug)]
pub struct InspectSnapshot {
    /// Free-form description of probe A in the active coord system
    /// ("Square (0, 0)" / "Hex axial (3, -2)" / "Iso (1, 1)" ...).
    pub probe_a_label: String,
    /// Free-form description of probe B.
    pub probe_b_label: String,
    /// Hex-only: full triple display for probe A — offset / axial /
    /// cube — joined on " · ". Empty for non-hex kinds.
    pub probe_a_extra: String,
    /// Graph distance(A, B). `u32::MAX` reads as "n/a" (Quadtree /
    /// Voronoi where there's no canonical graph).
    pub distance: u32,
    /// Number of cells on the discrete A→B line. `0` = n/a.
    pub line_length: u32,
    /// Number of neighbors of probe A's cell.
    pub neighbors: u32,
}

/// Compute the snapshot from `state`. Cheap for regular grids
/// (cell lookups are O(1) and bounded buffers). Tri's BFS-based
/// distance falls back to the bounded-radius cap documented in
/// `ph2d_grid::tri`.
pub fn snapshot(state: &GridSnapState) -> InspectSnapshot {
    match state.kind {
        GridKind::Square => snapshot_square(state),
        GridKind::Hex => snapshot_hex(state),
        GridKind::Iso => snapshot_iso(state),
        GridKind::StaggeredSquare => snapshot_staggered_square(state),
        GridKind::StaggeredHex => snapshot_staggered_hex(state),
        GridKind::Tri => snapshot_tri(state),
        GridKind::Chunks => snapshot_chunks(state),
        GridKind::Quadtree | GridKind::Voronoi => InspectSnapshot {
            probe_a_label: format!("World ({:.2}, {:.2})", state.probe_a[0], state.probe_a[1]),
            probe_b_label: format!("World ({:.2}, {:.2})", state.probe_b[0], state.probe_b[1]),
            probe_a_extra: String::new(),
            distance: u32::MAX,
            line_length: 0,
            neighbors: 0,
        },
    }
}

fn snapshot_square(state: &GridSnapState) -> InspectSnapshot {
    let g = state.make_square();
    let (pa, pb) = local_probes(state);
    let a = g.world_to_cell(pa);
    let b = g.world_to_cell(pb);
    let mut buf = Vec::new();
    g.neighbors(a, &mut buf);
    let n = buf.len() as u32;
    g.line(a, b, &mut buf);
    let line_len = buf.len() as u32;
    InspectSnapshot {
        probe_a_label: format!("Square ({}, {})", a.0, a.1),
        probe_b_label: format!("Square ({}, {})", b.0, b.1),
        probe_a_extra: String::new(),
        distance: g.distance(a, b),
        line_length: line_len,
        neighbors: n,
    }
}

fn snapshot_hex(state: &GridSnapState) -> InspectSnapshot {
    let g = state.make_hex();
    let (pa, pb) = local_probes(state);
    let a = g.world_to_cell(pa);
    let b = g.world_to_cell(pb);
    let a_cube = axial_to_cube(a);
    let a_offset = axial_to_offset(a, state.hex_cfg.offset_variant);
    let mut buf: Vec<HexCell> = Vec::new();
    g.neighbors(a, &mut buf);
    let n = buf.len() as u32;
    g.line(a, b, &mut buf);
    let line_len = buf.len() as u32;
    InspectSnapshot {
        probe_a_label: format!("Hex axial ({}, {})", a.q, a.r),
        probe_b_label: format!("Hex axial ({}, {})", b.q, b.r),
        probe_a_extra: format!(
            "offset ({}, {}) \u{00B7} cube ({}, {}, {})",
            a_offset.col, a_offset.row, a_cube.x, a_cube.y, a_cube.z
        ),
        distance: g.distance(a, b),
        line_length: line_len,
        neighbors: n,
    }
}

fn snapshot_iso(state: &GridSnapState) -> InspectSnapshot {
    let g = state.make_iso();
    let (pa, pb) = local_probes(state);
    let a = g.world_to_cell(pa);
    let b = g.world_to_cell(pb);
    let mut buf = Vec::new();
    g.neighbors(a, &mut buf);
    let n = buf.len() as u32;
    g.line(a, b, &mut buf);
    InspectSnapshot {
        probe_a_label: format!("Iso ({}, {})", a.0, a.1),
        probe_b_label: format!("Iso ({}, {})", b.0, b.1),
        probe_a_extra: String::new(),
        distance: g.distance(a, b),
        line_length: buf.len() as u32,
        neighbors: n,
    }
}

fn snapshot_staggered_square(state: &GridSnapState) -> InspectSnapshot {
    let g = state.make_staggered_square();
    let (pa, pb) = local_probes(state);
    let a = g.world_to_cell(pa);
    let b = g.world_to_cell(pb);
    let mut buf = Vec::new();
    g.neighbors(a, &mut buf);
    let n = buf.len() as u32;
    g.line(a, b, &mut buf);
    InspectSnapshot {
        probe_a_label: format!("StagSq ({}, {})", a.0, a.1),
        probe_b_label: format!("StagSq ({}, {})", b.0, b.1),
        probe_a_extra: String::new(),
        distance: g.distance(a, b),
        line_length: buf.len() as u32,
        neighbors: n,
    }
}

fn snapshot_staggered_hex(state: &GridSnapState) -> InspectSnapshot {
    let g = state.make_staggered_hex();
    let (pa, pb) = local_probes(state);
    let a = g.world_to_cell(pa);
    let b = g.world_to_cell(pb);
    let mut buf = Vec::new();
    g.neighbors(a, &mut buf);
    let n = buf.len() as u32;
    g.line(a, b, &mut buf);
    InspectSnapshot {
        probe_a_label: format!("Hex offset ({}, {})", a.col, a.row),
        probe_b_label: format!("Hex offset ({}, {})", b.col, b.row),
        probe_a_extra: String::new(),
        distance: g.distance(a, b),
        line_length: buf.len() as u32,
        neighbors: n,
    }
}

fn snapshot_tri(state: &GridSnapState) -> InspectSnapshot {
    let g = state.make_tri();
    let (pa, pb) = local_probes(state);
    let a = g.world_to_cell(pa);
    let b = g.world_to_cell(pb);
    let mut buf = Vec::new();
    g.neighbors(a, &mut buf);
    let n = buf.len() as u32;
    g.line(a, b, &mut buf);
    InspectSnapshot {
        probe_a_label: format!(
            "Tri (k={}, r={}, {})",
            a.k,
            a.r,
            if a.is_up() { "up" } else { "down" }
        ),
        probe_b_label: format!("Tri (k={}, r={})", b.k, b.r),
        probe_a_extra: String::new(),
        distance: g.distance(a, b),
        line_length: buf.len() as u32,
        neighbors: n,
    }
}

fn snapshot_chunks(state: &GridSnapState) -> InspectSnapshot {
    let g = state.make_chunks();
    let (pa, pb) = local_probes(state);
    let a = g.world_to_cell(pa);
    let b = g.world_to_cell(pb);
    let mut buf = Vec::new();
    g.neighbors(a, &mut buf);
    let n = buf.len() as u32;
    g.line(a, b, &mut buf);
    let chunk = g.chunk_of(a);
    InspectSnapshot {
        probe_a_label: format!("Sq ({}, {}) chunk ({}, {})", a.0, a.1, chunk.0, chunk.1),
        probe_b_label: format!("Sq ({}, {})", b.0, b.1),
        probe_a_extra: String::new(),
        distance: g.distance(a, b),
        line_length: buf.len() as u32,
        neighbors: n,
    }
}

/// Compute the on-screen height the inspect section will consume —
/// header + 5 label rows + 2 probe-input rows (A X/Y + B X/Y).
/// Used by `panel.rs` for vertical layout.
pub fn height() -> f32 {
    SECTION_HEADER_H + ROW_GAP + 8.0 * (ROW_H + ROW_GAP)
}

/// Paint the inspect section at `rect`. The caller is responsible
/// for positioning `rect` after the Display section.
pub fn paint(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut crate::interaction::HitIndex,
    store: &crate::interaction::WidgetStore,
    state: &GridSnapState,
) {
    let header = SectionHeader {
        id: crate::NodeId(0),
        label: "Inspect".to_string(),
        count: None,
        collapsible: None,
        color: None,
    };
    paint_section_header(
        &header,
        Rect::new(rect.x, rect.y, rect.w, SECTION_HEADER_H),
        scene,
        text_system,
        theme,
    );

    let snap = snapshot(state);
    let mut y = rect.y + SECTION_HEADER_H + ROW_GAP;
    let x = rect.x + Spacing::Sm.px();
    let label_color = resolve(ColorToken::Text2, theme);

    let rows: [(String, String); 5] = [
        ("Probe A".to_string(), snap.probe_a_label.clone()),
        ("".to_string(), snap.probe_a_extra.clone()),
        ("Probe B".to_string(), snap.probe_b_label.clone()),
        (
            "Distance".to_string(),
            if snap.distance == u32::MAX {
                "n/a".to_string()
            } else {
                snap.distance.to_string()
            },
        ),
        (
            "Line / Neighbors".to_string(),
            format!("{} cells / {}", snap.line_length, snap.neighbors),
        ),
    ];

    for (label, value) in &rows {
        if label.is_empty() && value.is_empty() {
            continue;
        }
        let row_y = y + (ROW_H - LABEL_FONT_SIZE) * 0.5;
        if !label.is_empty() {
            paint_text(
                text_system,
                scene,
                label,
                x,
                row_y,
                LABEL_FONT_SIZE,
                80.0,
                resolve(ColorToken::Text1, theme),
            );
        }
        // Value right of the label; offset by 90px so labels and
        // values align across rows.
        if !value.is_empty() {
            paint_text(
                text_system,
                scene,
                value,
                x + 90.0,
                row_y,
                LABEL_FONT_SIZE,
                rect.w - 100.0,
                label_color,
            );
        }
        y += ROW_H + ROW_GAP;
    }

    // Probe-input rows — 2 small NumberInputs per probe, side by side.
    y += ROW_GAP;
    paint_probe_pair_row(
        "Probe A",
        super::ids::GS_PROBE_A_X,
        super::ids::GS_PROBE_A_Y,
        state.probe_a,
        rect.x,
        rect.w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y += ROW_H + ROW_GAP;
    paint_probe_pair_row(
        "Probe B",
        super::ids::GS_PROBE_B_X,
        super::ids::GS_PROBE_B_Y,
        state.probe_b,
        rect.x,
        rect.w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
}

/// One probe row: label + X NumberInput + Y NumberInput side by side.
#[allow(clippy::too_many_arguments)]
fn paint_probe_pair_row(
    label: &str,
    x_id: crate::NodeId,
    y_id: crate::NodeId,
    value: Vec2,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut crate::interaction::HitIndex,
    store: &crate::interaction::WidgetStore,
) {
    let label_w = 70.0;
    let gap = 4.0;
    let input_w = (w - label_w - gap - Spacing::Sm.px() * 2.0) / 2.0;
    paint_text(
        text_system,
        scene,
        label,
        x + Spacing::Sm.px(),
        y + (ROW_H - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        label_w,
        resolve(ColorToken::Text1, theme),
    );
    // Probe values are world meters; convert through the active
    // DisplayUnit so the NumberInputs read the right magnitude.
    for (i, (id, v_m)) in [(x_id, value[0]), (y_id, value[1])].iter().enumerate() {
        let v_disp = super::panel::meters_to_display_pub(*v_m);
        let r = Rect::new(
            x + Spacing::Sm.px() + label_w + i as f32 * (input_w + gap),
            y,
            input_w,
            ROW_H,
        );
        let (ti_state, _, buffer, caret, anchor) = store.number_input(*id).unwrap_or((
            crate::widget::TextInputState::Normal,
            v_disp,
            "",
            0,
            None,
        ));
        let buffer_arg = if ti_state == crate::widget::TextInputState::Focused {
            Some(buffer)
        } else {
            None
        };
        let input = crate::widget::NumberInput::new(*id, "", v_disp).state(ti_state);
        crate::widget::paint_number_input_with_buffer(
            &input,
            buffer_arg,
            caret,
            anchor,
            r,
            scene,
            text_system,
            theme,
        );
        hit_index.register(*id, r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_square_defaults() {
        let state = GridSnapState::default();
        let s = snapshot(&state);
        // probe_a = [0,0] → cell (0, 0); probe_b = [3, 2] → cell (3, 2).
        assert!(
            s.probe_a_label.contains("(0, 0)"),
            "got {}",
            s.probe_a_label
        );
        assert!(
            s.probe_b_label.contains("(3, 2)"),
            "got {}",
            s.probe_b_label
        );
        // Manhattan distance Von4 = 5.
        assert_eq!(s.distance, 5);
        assert_eq!(s.neighbors, 4);
    }

    #[test]
    fn snapshot_hex_shows_three_coord_systems() {
        let state = GridSnapState {
            kind: GridKind::Hex,
            ..Default::default()
        };
        let s = snapshot(&state);
        // Hex probe_a label includes axial; extra includes offset + cube.
        assert!(s.probe_a_label.contains("axial"));
        assert!(s.probe_a_extra.contains("offset"));
        assert!(s.probe_a_extra.contains("cube"));
    }

    #[test]
    fn snapshot_voronoi_is_na() {
        let state = GridSnapState {
            kind: GridKind::Voronoi,
            ..Default::default()
        };
        let s = snapshot(&state);
        assert_eq!(s.distance, u32::MAX);
        assert_eq!(s.line_length, 0);
        assert_eq!(s.neighbors, 0);
    }

    #[test]
    fn snapshot_covers_all_nine_kinds_without_panic() {
        // Smoke: build a snapshot for every kind. We're not asserting
        // values, just that the dispatch handles all variants.
        for kind in GridKind::all() {
            let state = GridSnapState {
                kind,
                ..Default::default()
            };
            let _ = snapshot(&state);
        }
    }
}
