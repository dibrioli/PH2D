//! Inspect subsection — shows the coord-system label, cell IDs for
//! probe A / B, and computed graph distance + line length + neighbor
//! count for the currently-active grid kind, followed by the two probe
//! rows ([`super::ids::GS_PROBE_A_X`] etc.) where the probes are edited.
//!
//! Every row goes through the house property-row doors
//! ([`crate::property_row`]): the name right-aligned against the panel's
//! value column, the fields the standard number boxes.

use super::state::{GridKind, GridSnapState};
use crate::paint::{paint_text, resolve};
use crate::widget::{SectionHeader, paint_section_header};
use crate::zones::Rect;
use ph2d_grid::GridMath;
use ph2d_grid::Vec2;
use ph2d_grid::hex::{HexCell, axial_to_cube, axial_to_offset};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme, TypeToken, list_row_gap_px};
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
// ⭐ **A linha de uma LISTA e a linha do app** (wave 17): o `22.0` a mao coincidia com o
// token, e uma coincidencia nao segue quem mexe no token.
const ROW_H: f32 = ROW_H_PX;

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
            probe_a_label: ph2d_i18n::tr_with(
                "chrome.grid_snap.coord.world",
                &[
                    ("a", &format!("{:.2}", state.probe_a[0])),
                    ("b", &format!("{:.2}", state.probe_a[1])),
                ],
            ),
            probe_b_label: ph2d_i18n::tr_with(
                "chrome.grid_snap.coord.world",
                &[
                    ("a", &format!("{:.2}", state.probe_b[0])),
                    ("b", &format!("{:.2}", state.probe_b[1])),
                ],
            ),
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
        probe_a_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.square",
            &[("a", &a.0), ("b", &a.1)],
        ),
        probe_b_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.square",
            &[("a", &b.0), ("b", &b.1)],
        ),
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
        probe_a_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.hex_axial",
            &[("a", &a.q), ("b", &a.r)],
        ),
        probe_b_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.hex_axial",
            &[("a", &b.q), ("b", &b.r)],
        ),
        probe_a_extra: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.hex_extra",
            &[
                ("col", &a_offset.col),
                ("row", &a_offset.row),
                ("x", &a_cube.x),
                ("y", &a_cube.y),
                ("z", &a_cube.z),
            ],
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
        probe_a_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.iso",
            &[("a", &a.0), ("b", &a.1)],
        ),
        probe_b_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.iso",
            &[("a", &b.0), ("b", &b.1)],
        ),
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
        probe_a_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.stag_sq",
            &[("a", &a.0), ("b", &a.1)],
        ),
        probe_b_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.stag_sq",
            &[("a", &b.0), ("b", &b.1)],
        ),
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
        probe_a_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.hex_offset",
            &[("a", &a.col), ("b", &a.row)],
        ),
        probe_b_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.hex_offset",
            &[("a", &b.col), ("b", &b.row)],
        ),
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
        probe_a_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.tri_facing",
            &[
                ("k", &a.k),
                ("r", &a.r),
                (
                    "facing",
                    &ph2d_i18n::tr(if a.is_up() {
                        "chrome.grid_snap.coord.up"
                    } else {
                        "chrome.grid_snap.coord.down"
                    }),
                ),
            ],
        ),
        probe_b_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.tri",
            &[("k", &b.k), ("r", &b.r)],
        ),
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
        probe_a_label: ph2d_i18n::tr_with(
            "chrome.grid_snap.coord.sq_chunk",
            &[("a", &a.0), ("b", &a.1), ("ca", &chunk.0), ("cb", &chunk.1)],
        ),
        probe_b_label: ph2d_i18n::tr_with("chrome.grid_snap.coord.sq", &[("a", &b.0), ("b", &b.1)]),
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
    SECTION_HEADER_H + list_row_gap_px() + 8.0 * (ROW_H + list_row_gap_px())
}

/// Paint the inspect section at `rect`. The caller is responsible
/// for positioning `rect` after the Display section.
///
/// The probe fields read the store, which the panel reseeds from the
/// state in the active DisplayUnit before painting — so this section no
/// longer needs the unit.
#[allow(clippy::too_many_arguments)]
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
        label: tr("chrome.grid_snap.inspect").to_string(),
        count: None,
        collapsible: None,
        open_t: None,
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
    let mut y = rect.y + SECTION_HEADER_H + list_row_gap_px();

    let rows: [(String, String); 5] = [
        (
            ph2d_i18n::tr("chrome.grid_snap.probe_a").to_string(),
            snap.probe_a_label.clone(),
        ),
        ("".to_string(), snap.probe_a_extra.clone()),
        (
            ph2d_i18n::tr("chrome.grid_snap.probe_b").to_string(),
            snap.probe_b_label.clone(),
        ),
        (
            tr("chrome.grid_snap.distance").to_string(),
            if snap.distance == u32::MAX {
                "n/a".to_string()
            } else {
                snap.distance.to_string()
            },
        ),
        (
            tr("chrome.grid_snap.line_neighbors").to_string(),
            ph2d_i18n::tr_with(
                "chrome.grid_snap.cells_neighbors",
                &[("cells", &snap.line_length), ("neighbors", &snap.neighbors)],
            ),
        ),
    ];

    // ⭐⭐⭐ **Esta secção pinta pelas PORTAS da casa — o nome, a coluna e a caixa são os de toda
    //    linha de propriedade** (ordem do dono, 2026-09-23, com uma foto e duas setas: *«painel grid
    //    fora do padrão»*). Até aí ela pintava o nome À ESQUERDA por `paint_text` e montava as caixas
    //    das sondas à mão, com a metade da coluna calculada aqui — três respostas próprias a
    //    perguntas que a [`crate::property_row`] já responde para o app inteiro, e a do nome era a
    //    ERRADA: o padrão é o nome encostado À DIREITA, contra a coluna do valor.
    //
    // ⚠️ **UMA secção, medida UMA vez, sobre TODOS os nomes que ela pinta** — os das linhas de
    //    leitura e os das sondas, que são os mesmos dois (`Probe A`/`Probe B`). Duas medidas dariam
    //    duas colunas na mesma secção, que é o defeito de 2026-09-19 que o gate desta secção existe
    //    para impedir. `campos = 2` porque a linha mais exigente é a da sonda (X e Y lado a lado).
    let nomes: Vec<&str> = rows
        .iter()
        .map(|(l, _)| l.as_str())
        .filter(|l| !l.is_empty())
        .collect();
    let seccao = crate::property_row::Seccao::medida(text_system, 2, &nomes);
    let fonte = TypeToken::Sm.px();
    let valor_cor = resolve(ColorToken::Text1, theme);
    for (label, value) in &rows {
        if label.is_empty() && value.is_empty() {
            continue;
        }
        // Uma linha sem nome (a continuação hexagonal da sonda A) usa a MESMA geometria e não pinta
        // nome nenhum — o valor fica alinhado com os irmãos.
        let row = if label.is_empty() {
            crate::property_row::colunas_da_linha(rect.x, rect.w, y, ROW_H, seccao)
        } else {
            crate::property_row::paint_label_row(
                scene,
                text_system,
                theme,
                rect.x,
                rect.w,
                y,
                ROW_H,
                label,
                seccao,
            )
        };
        if !value.is_empty() {
            paint_text(
                text_system,
                scene,
                value,
                row.control.x,
                row.control.y + (row.control.h - fonte) * 0.5,
                fonte,
                row.control.w,
                valor_cor,
            );
        }
        y += ROW_H + list_row_gap_px();
    }

    // As sondas: X e Y lado a lado, pela porta de linha de VÁRIAS componentes. O valor sai da
    // loja, que o painel re-semeia do estado na unidade activa antes de pintar
    // (`sync_meter_inputs_to_display_unit_impl`) — é por isso que esta secção já não precisa de
    // saber a unidade.
    y += list_row_gap_px();
    for (label, x_id, y_id) in [
        (
            ph2d_i18n::tr("chrome.grid_snap.probe_a"),
            super::ids::GS_PROBE_A_X,
            super::ids::GS_PROBE_A_Y,
        ),
        (
            ph2d_i18n::tr("chrome.grid_snap.probe_b"),
            super::ids::GS_PROBE_B_X,
            super::ids::GS_PROBE_B_Y,
        ),
    ] {
        y = crate::property_row::paint_fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            rect.x,
            rect.w,
            y,
            label,
            &[x_id, y_id],
            1.0,
            None,
            seccao,
        );
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
