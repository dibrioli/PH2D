//! A configuração de cada um dos nove tipos de grelha, em cartão: as mesmas linhas do `paint_kinds`.

use super::*;

fn par(c: &mut CardCtx<'_>, a: (NodeId, &str), b: (NodeId, &str), y: f32) -> f32 {
    let (la, lb) = (com_unidade(a.1), com_unidade(b.1));
    c.row(&[campo(a.0, &la), campo(b.0, &lb)], y)
}

fn origem(c: &mut CardCtx<'_>, y: f32) -> f32 {
    par(
        c,
        (g::GS_CFG_ORIGIN_X, tr("panel.grid_snap.rows.origin_x")),
        (g::GS_CFG_ORIGIN_Y, tr("panel.grid_snap.rows.origin_y")),
        y,
    )
}

fn vizinhanca(c: &mut CardCtx<'_>, s: &GridSnapState, y: f32) -> f32 {
    let label = Some(tr("panel.grid_snap.options.neighborhood"));
    escolha(c, label, &segs_da_vizinhanca(s), y)
}

/// O tamanho da célula e a escolha curta que o acompanha; `row` desce a escolha se ela não couber.
fn celula_e_escolha(
    c: &mut CardCtx<'_>,
    chave: &str,
    (rotulo, segs): (&str, &[Seg<'_>]),
    y: f32,
) -> f32 {
    let l = com_unidade(tr(chave));
    c.row(
        &[
            campo(g::GS_CFG_CELL_SIZE, &l),
            Cell::new(Widget::Choice(segs)).label(tr(rotulo)),
        ],
        y,
    )
}

fn segs_da_vizinhanca(s: &GridSnapState) -> [Seg<'static>; 2] {
    if s.kind == GridKind::Tri {
        let v = s.tri_cfg.neighborhood == TriNeighborhood::Vertex12;
        [
            seg(g::GS_CFG_TRI_EDGE3, "panel.grid_snap.options.edge3", !v),
            seg(
                g::GS_CFG_TRI_VERTEX12,
                "panel.grid_snap.options.vertex12",
                v,
            ),
        ]
    } else {
        let m8 = matches!(neighborhood_for_active_kind(s), SquareNeighborhood::Moore8);
        [
            seg(
                g::GS_CFG_NEIGHBORHOOD_4,
                "panel.grid_snap.options.von4",
                !m8,
            ),
            seg(
                g::GS_CFG_NEIGHBORHOOD_8,
                "panel.grid_snap.options.moore8",
                m8,
            ),
        ]
    }
}

fn hex(c: &mut CardCtx<'_>, s: &GridSnapState, y: f32) -> f32 {
    let cfg = if s.kind == GridKind::StaggeredHex {
        &s.staggered_hex_cfg.hex
    } else {
        &s.hex_cfg
    };
    let pontas = cfg.orientation == HexOrientation::Pointy;
    let orientacao = [
        seg(g::GS_CFG_HEX_POINTY, "panel.grid_snap.kinds.pointy", pontas),
        seg(g::GS_CFG_HEX_FLAT, "panel.grid_snap.kinds.flat", !pontas),
    ];
    let y = celula_e_escolha(
        c,
        "panel.grid_snap.kinds.cell_size",
        ("panel.grid_snap.kinds.orientation", &orientacao),
        y,
    );
    let y = origem(c, y);
    let o = cfg.offset_variant;
    escolha(
        c,
        Some(tr("panel.grid_snap.kinds.offset")),
        &[
            seg(
                g::GS_CFG_HEX_OFFSET_ODDR,
                "panel.grid_snap.kinds.oddr",
                o == HexOffset::OddR,
            ),
            seg(
                g::GS_CFG_HEX_OFFSET_EVENR,
                "panel.grid_snap.kinds.evenr",
                o == HexOffset::EvenR,
            ),
            seg(
                g::GS_CFG_HEX_OFFSET_ODDQ,
                "panel.grid_snap.kinds.oddq",
                o == HexOffset::OddQ,
            ),
            seg(
                g::GS_CFG_HEX_OFFSET_EVENQ,
                "panel.grid_snap.kinds.evenq",
                o == HexOffset::EvenQ,
            ),
        ],
        y,
    )
}

/// Os quatro números de uma caixa delimitadora, com os nomes de sempre.
fn caixa(c: &mut CardCtx<'_>, prefixo: &str, ids: [NodeId; 4], y: f32) -> f32 {
    let nomes = crate::paint_rows::aabb_labels(prefixo);
    let y = par(c, (ids[0], &nomes[0]), (ids[1], &nomes[1]), y);
    par(c, (ids[2], &nomes[2]), (ids[3], &nomes[3]), y)
}

/// A configuração do tipo activo: as mesmas linhas do `paint_kinds` de antes.
pub(super) fn configuracao(c: &mut CardCtx<'_>, s: &GridSnapState, y: f32) -> f32 {
    match s.kind {
        GridKind::Square => {
            let (a, b) = (
                com_unidade(tr("panel.grid_snap.kinds.cell_size")),
                com_unidade(tr("panel.grid_snap.kinds.major_every")),
            );
            let y = c.row(
                &[
                    campo(g::GS_CFG_CELL_SIZE, &a),
                    campo(g::GS_CFG_SPACING_MAJOR, &b),
                ],
                y,
            );
            let y = origem(c, y);
            vizinhanca(c, s, y)
        }
        GridKind::Hex | GridKind::StaggeredHex => hex(c, s, y),
        GridKind::Iso => {
            let y = par(
                c,
                (g::GS_CFG_ISO_TILE_W, tr("panel.grid_snap.kinds.tile_width")),
                (
                    g::GS_CFG_ISO_TILE_H,
                    tr("panel.grid_snap.kinds.tile_height"),
                ),
                y,
            );
            let y = origem(c, y);
            vizinhanca(c, s, y)
        }
        GridKind::StaggeredSquare => {
            let impar = s.staggered_square_cfg.parity == StaggerParity::OddRows;
            let paridade = [
                seg(
                    g::GS_CFG_STAGGER_PARITY_ODD,
                    "panel.grid_snap.kinds.odd_rows",
                    impar,
                ),
                seg(
                    g::GS_CFG_STAGGER_PARITY_EVEN,
                    "panel.grid_snap.kinds.even_rows",
                    !impar,
                ),
            ];
            let y = celula_e_escolha(
                c,
                "panel.grid_snap.kinds.cell_size",
                ("panel.grid_snap.kinds.parity", &paridade),
                y,
            );
            let y = origem(c, y);
            vizinhanca(c, s, y)
        }
        GridKind::Tri => {
            let vizinhos = segs_da_vizinhanca(s);
            let y = celula_e_escolha(
                c,
                "panel.grid_snap.kinds.edge_length",
                ("panel.grid_snap.options.neighborhood", &vizinhos),
                y,
            );
            origem(c, y)
        }
        GridKind::Chunks => {
            let a = com_unidade(tr("panel.grid_snap.kinds.cell_size"));
            let y = c.row(
                &[
                    campo(g::GS_CFG_CELL_SIZE, &a),
                    campo(
                        g::GS_CFG_CHUNKS_SIZE,
                        tr("panel.grid_snap.kinds.chunk_size_cells"),
                    ),
                ],
                y,
            );
            let y = origem(c, y);
            vizinhanca(c, s, y)
        }
        GridKind::Quadtree => {
            let y = c.row(
                &[
                    campo(
                        g::GS_CFG_QT_MAX_PER_LEAF,
                        tr("panel.grid_snap.bounded.max_leaf"),
                    ),
                    campo(
                        g::GS_CFG_QT_MAX_DEPTH,
                        tr("panel.grid_snap.bounded.max_depth"),
                    ),
                ],
                y,
            );
            let y = caixa(
                c,
                tr("panel.grid_snap.bounded.qt_bounds"),
                [
                    g::GS_CFG_QT_BOUNDS_MIN_X,
                    g::GS_CFG_QT_BOUNDS_MIN_Y,
                    g::GS_CFG_QT_BOUNDS_MAX_X,
                    g::GS_CFG_QT_BOUNDS_MAX_Y,
                ],
                y,
            );
            c.row(
                &[
                    campo(
                        g::GS_CFG_QT_DEMO_POINTS,
                        tr("panel.grid_snap.bounded.demo_points"),
                    ),
                    campo(
                        g::GS_CFG_QT_DEMO_SEED,
                        tr("panel.grid_snap.bounded.demo_seed"),
                    ),
                ],
                y,
            )
        }
        GridKind::Voronoi => {
            let y = c.row(
                &[
                    campo(
                        g::GS_CFG_VORONOI_SEED_COUNT,
                        tr("panel.grid_snap.bounded.seed_count"),
                    ),
                    campo(
                        g::GS_CFG_VORONOI_RNG_SEED,
                        tr("panel.grid_snap.bounded.rng_seed"),
                    ),
                ],
                y,
            );
            let y = c.row(
                &[
                    campo(
                        g::GS_CFG_VORONOI_LLOYD_ITERS,
                        tr("panel.grid_snap.bounded.lloyd_iters"),
                    ),
                    Cell::new(Widget::Button {
                        id: g::GS_CFG_VORONOI_RESEED,
                        text: tr("panel.grid_snap.bounded.reseed_next_rng"),
                        primary: false,
                    }),
                ],
                y,
            );
            caixa(
                c,
                tr("panel.grid_snap.bounded.voronoi_bounds"),
                [
                    g::GS_CFG_VORONOI_BOUNDS_MIN_X,
                    g::GS_CFG_VORONOI_BOUNDS_MIN_Y,
                    g::GS_CFG_VORONOI_BOUNDS_MAX_X,
                    g::GS_CFG_VORONOI_BOUNDS_MAX_Y,
                ],
                y,
            )
        }
    }
}
