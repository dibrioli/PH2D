//! **AS STRINGS DO PAINEL GRID SETTINGS** — `panel.grid_snap.*`.
//!
//! ⚠️ **Um corte por ASSUNTO**, como os irmãos. Migrado em 2026-09-16 por
//! `scripts/migrar-texto-pintado.py` (mapa `docs/UI_New_and_Simple/ferramentas/seccoes_grid_snap.tsv`):
//! o painel escrevia **87** textos no fonte, e **~20** deles eram montados com o sufixo da unidade
//! (`"Cell size (m)"`) — que a spec §7 manda para DENTRO da caixa. Esses nomes chegam aqui sem ela.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave do painel Grid Settings, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "panel.grid_snap.title" => "Grid Settings",
        "panel.grid_snap.sections.grid_kind" => "Grid Kind",
        "panel.grid_snap.sections.target" => "Target",
        "panel.grid_snap.sections.magnetism_radius" => "Magnetism radius",
        "panel.grid_snap.sections.subdivisions" => "Subdivisions",
        "panel.grid_snap.sections.display" => "Display",
        "panel.grid_snap.sections.layer" => "Layer",
        "panel.grid_snap.sections.in_front" => "In front",
        "panel.grid_snap.sections.behind" => "Behind",
        "panel.grid_snap.options.snap_on" => "Snap: ON",
        "panel.grid_snap.options.snap_off" => "Snap: OFF",
        "panel.grid_snap.options.square" => "Square",
        "panel.grid_snap.options.hex" => "Hex",
        "panel.grid_snap.options.iso" => "Iso",
        "panel.grid_snap.options.stag_sq" => "Stag Sq",
        "panel.grid_snap.options.stag_hex" => "Stag Hex",
        "panel.grid_snap.options.tri" => "Tri",
        "panel.grid_snap.options.quadtree" => "Quadtree",
        "panel.grid_snap.options.voronoi" => "Voronoi",
        "panel.grid_snap.options.chunks" => "Chunks",
        "panel.grid_snap.options.center" => "Center",
        "panel.grid_snap.options.intersection" => "Intersection",
        "panel.grid_snap.options.corner" => "Corner",
        "panel.grid_snap.options.center_intersection" => "Center + Intersection",
        "panel.grid_snap.options.center_intersection_corners" => "Center + Intersection + Corners",
        "panel.grid_snap.options.neighborhood" => "Neighborhood",
        "panel.grid_snap.options.von4" => "Von4",
        "panel.grid_snap.options.moore8" => "Moore8",
        "panel.grid_snap.options.edge3" => "Edge3",
        "panel.grid_snap.options.vertex12" => "Vertex12",
        "panel.grid_snap.kinds.cell_size" => "Cell size",
        "panel.grid_snap.kinds.major_every" => "Major every",
        "panel.grid_snap.kinds.orientation" => "Orientation",
        "panel.grid_snap.kinds.pointy" => "Pointy",
        "panel.grid_snap.kinds.flat" => "Flat",
        "panel.grid_snap.kinds.offset" => "Offset",
        "panel.grid_snap.kinds.oddr" => "OddR",
        "panel.grid_snap.kinds.evenr" => "EvenR",
        "panel.grid_snap.kinds.oddq" => "OddQ",
        "panel.grid_snap.kinds.evenq" => "EvenQ",
        "panel.grid_snap.kinds.tile_width" => "Tile width",
        "panel.grid_snap.kinds.tile_height" => "Tile height",
        "panel.grid_snap.kinds.parity" => "Parity",
        "panel.grid_snap.kinds.odd_rows" => "Odd rows",
        "panel.grid_snap.kinds.even_rows" => "Even rows",
        "panel.grid_snap.kinds.edge_length" => "Edge length",
        "panel.grid_snap.kinds.chunk_size_cells" => "Chunk size (cells)",
        "panel.grid_snap.bounded.qt_bounds" => "QT bounds",
        "panel.grid_snap.bounded.max_leaf" => "Max / leaf",
        "panel.grid_snap.bounded.max_depth" => "Max depth",
        "panel.grid_snap.bounded.demo_points" => "Demo points",
        "panel.grid_snap.bounded.demo_seed" => "Demo seed",
        "panel.grid_snap.bounded.voronoi_bounds" => "Voronoi bounds",
        "panel.grid_snap.bounded.seed_count" => "Seed count",
        "panel.grid_snap.bounded.rng_seed" => "RNG seed",
        "panel.grid_snap.bounded.lloyd_iters" => "Lloyd iters",
        "panel.grid_snap.bounded.reseed_next_rng" => "Reseed (next RNG)",
        "panel.grid_snap.rows.origin_x" => "Origin X",
        "panel.grid_snap.rows.origin_y" => "Origin Y",
        "panel.grid_snap.rows.min_x" => "{bounds} min X",
        "panel.grid_snap.rows.min_y" => "{bounds} min Y",
        "panel.grid_snap.rows.max_x" => "{bounds} max X",
        "panel.grid_snap.rows.max_y" => "{bounds} max Y",
        "panel.grid_snap.rows.show_grid" => "Show grid",
        "panel.grid_snap.rows.opacity" => "Opacity",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
