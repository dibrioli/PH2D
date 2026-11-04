//! ⭐⭐ **O painel Grid Settings em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): as
//! mesmas quatro secções e os MESMOS ids; o despacho (`event.rs`) não muda. A opacidade é a linha de
//! antes pela ponte `CardCtx::legacy`: é uma barra sem caixa (nem chip registado). Janela FLUTUANTE: a
//! lista pinta-se no rect que o artista moveu.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::grid_snap::ids as g;
use ph2d_editor_core::grid_snap::{GridKind, GridSnapState};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget};
use ph2d_editor_core::panel::{PaintCtx, TextKey};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::panel_chrome::{
    paint_panel_corner_dot, paint_panel_corner_dot_bl, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::zones::Rect;
use ph2d_grid::hex::{HexOffset, HexOrientation};
use ph2d_grid::snap::SnapTarget;
use ph2d_grid::square::SquareNeighborhood;
use ph2d_grid::staggered::StaggerParity;
use ph2d_grid::tri::TriNeighborhood;
use ph2d_i18n::{tr, tr_with};

use crate::paint_helpers::neighborhood_for_active_kind;
use crate::state::{current_display_unit, length_unit, meters_to_display};

#[path = "cards_config.rs"]
mod config;
use config::configuracao;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (a variável de ambiente seria partilhada entre testes paralelos).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// O passo das caixas de comprimento: um px, ou um decímetro em metros.
const PASSO_EM_METROS: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento (metros), não medida de design

fn passo_de_comprimento() -> f64 {
    match current_display_unit() {
        DisplayUnit::Meters => PASSO_EM_METROS,
        DisplayUnit::Pixels => 1.0,
    }
}

/// Os números que o cartão mostra: `(id, valor, é comprimento)`, os que o tipo activo oferece.
fn numeros(s: &GridSnapState) -> Vec<(NodeId, f64, bool)> {
    let len = |m: f32| meters_to_display(m);
    let mut v = vec![
        (
            g::GS_CFG_SNAP_SUBDIVISIONS,
            f64::from(s.snap_subdivisions),
            false,
        ),
        (
            g::GS_CFG_SNAP_MAGNETISM_RADIUS,
            len(s.snap_magnetism_radius),
            true,
        ),
    ];
    v.extend([
        (g::GS_PROBE_A_X, len(s.probe_a[0]), true),
        (g::GS_PROBE_A_Y, len(s.probe_a[1]), true),
        (g::GS_PROBE_B_X, len(s.probe_b[0]), true),
        (g::GS_PROBE_B_Y, len(s.probe_b[1]), true),
    ]);
    let origem = s.active_origin();
    let celula = match s.kind {
        GridKind::Square => Some(s.square_cfg.cell_size),
        GridKind::Hex => Some(s.hex_cfg.cell_size),
        GridKind::StaggeredSquare => Some(s.staggered_square_cfg.cell_w),
        GridKind::StaggeredHex => Some(s.staggered_hex_cfg.hex.cell_size),
        GridKind::Tri => Some(s.tri_cfg.edge_length),
        GridKind::Chunks => Some(s.chunks_cfg.cell_size),
        GridKind::Iso | GridKind::Quadtree | GridKind::Voronoi => None,
    };
    if let Some(c) = celula {
        v.push((g::GS_CFG_CELL_SIZE, len(c), true));
    }
    match s.kind {
        GridKind::Quadtree => {
            let b = s.quadtree_cfg.bounds;
            v.extend([
                (
                    g::GS_CFG_QT_MAX_PER_LEAF,
                    s.quadtree_cfg.max_points_per_leaf as f64,
                    false,
                ),
                (
                    g::GS_CFG_QT_MAX_DEPTH,
                    f64::from(s.quadtree_cfg.max_depth),
                    false,
                ),
                (g::GS_CFG_QT_BOUNDS_MIN_X, len(b.min[0]), true),
                (g::GS_CFG_QT_BOUNDS_MIN_Y, len(b.min[1]), true),
                (g::GS_CFG_QT_BOUNDS_MAX_X, len(b.max[0]), true),
                (g::GS_CFG_QT_BOUNDS_MAX_Y, len(b.max[1]), true),
                (
                    g::GS_CFG_QT_DEMO_POINTS,
                    s.quadtree_cfg.demo_point_count as f64,
                    false,
                ),
                (
                    g::GS_CFG_QT_DEMO_SEED,
                    s.quadtree_cfg.demo_rng_seed as f64,
                    false,
                ),
            ]);
        }
        GridKind::Voronoi => {
            let b = s.voronoi_cfg.bounds;
            v.extend([
                (
                    g::GS_CFG_VORONOI_SEED_COUNT,
                    s.voronoi_cfg.seed_count as f64,
                    false,
                ),
                (
                    g::GS_CFG_VORONOI_RNG_SEED,
                    s.voronoi_cfg.rng_seed as f64,
                    false,
                ),
                (
                    g::GS_CFG_VORONOI_LLOYD_ITERS,
                    f64::from(s.voronoi_cfg.lloyd_iterations),
                    false,
                ),
                (g::GS_CFG_VORONOI_BOUNDS_MIN_X, len(b.min[0]), true),
                (g::GS_CFG_VORONOI_BOUNDS_MIN_Y, len(b.min[1]), true),
                (g::GS_CFG_VORONOI_BOUNDS_MAX_X, len(b.max[0]), true),
                (g::GS_CFG_VORONOI_BOUNDS_MAX_Y, len(b.max[1]), true),
            ]);
        }
        _ => {
            v.extend([
                (g::GS_CFG_ORIGIN_X, len(origem[0]), true),
                (g::GS_CFG_ORIGIN_Y, len(origem[1]), true),
            ]);
        }
    }
    match s.kind {
        GridKind::Square => v.push((
            g::GS_CFG_SPACING_MAJOR,
            len(s.square_cfg.spacing_major),
            true,
        )),
        GridKind::Iso => v.extend([
            (g::GS_CFG_ISO_TILE_W, len(s.iso_cfg.tile_w), true),
            (g::GS_CFG_ISO_TILE_H, len(s.iso_cfg.tile_h), true),
        ]),
        GridKind::Chunks => v.push((
            g::GS_CFG_CHUNKS_SIZE,
            f64::from(s.chunks_cfg.chunk_size_cells),
            false,
        )),
        _ => {}
    }
    v
}

/// Antes de pintar: fecho, campos marcados, valores nas caixas e as duas caixas de marcar a dizer o
/// que o estado diz (o `Toggle` do store é quem o despacho vira). O campo que a mão arrasta ou escreve
/// não se semeia.
pub(crate) fn prepare(store: &mut WidgetStore, s: &GridSnapState) {
    store.mark_card_list_close(ph2d_editor_core::ids::GS_PANEL, g::GS_CLOSE);
    prepara_opacidade(store, s);
    for (id, on) in [
        (g::GS_SNAP_ENABLED, s.snap_enabled),
        (g::GS_SHOW_OVERLAY, s.show_overlay),
    ] {
        if let Some(InteractiveState::Toggle { on: o, .. }) = store.get_mut(id) {
            *o = on;
        }
    }
    let fabrica = numeros(&GridSnapState {
        kind: s.kind,
        ..GridSnapState::default()
    });
    let arrastado = store.number_input_drag().map(|d| d.id);
    for (id, valor, comprimento) in numeros(s) {
        let step = if comprimento {
            passo_de_comprimento()
        } else {
            1.0
        };
        let default = fabrica.iter().find(|f| f.0 == id).map(|f| f.1);
        store.mark_card_field(
            id,
            CardFieldSpec {
                step,
                scrub: step,
                default,
            },
        );
        if arrastado != Some(id) && store.focus_id() != Some(id) {
            store.set_number_value(id, valor);
        }
    }
}

/// O campo da opacidade em %: um chip derivado da barra, ligado a ela; quem avisa o painel continua a ser a barra.
#[must_use]
pub fn chip_da_opacidade() -> NodeId {
    const SAL: u64 = 0x51C3_77A9_2E10_B64D;
    NodeId(g::GS_OPACITY_SLIDER.0 ^ SAL)
}

const PCT: f32 = 100.0; // LITERAL-PX-OK: fração 0..1 mostrada em %, não medida de desenho

fn prepara_opacidade(store: &mut WidgetStore, s: &GridSnapState) {
    let chip = chip_da_opacidade();
    if store.get(chip).is_none() {
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped_integer(g::GS_OPACITY_SLIDER, chip, PCT, 0.0);
        store.set_number_range(chip, 0.0, f64::from(PCT), 1.0);
    }
    let fabrica = GridSnapState::default().opacity;
    store.mark_card_field(
        chip,
        CardFieldSpec {
            step: 1.0,
            scrub: 1.0,
            default: Some(f64::from((fabrica * PCT).round())),
        },
    );
    if store.number_input_drag().map(|d| d.id) != Some(chip) && store.focus_id() != Some(chip) {
        store.set_number_value(chip, f64::from((s.opacity * PCT).round()));
    }
}

/// O nome com a unidade de comprimento do projecto (D13: a unidade mora no rótulo).
fn com_unidade(nome: &str) -> String {
    tr_with(
        "card.grid.with_unit",
        &[("name", &nome), ("unit", &length_unit().suffix())],
    )
}

fn campo<'a>(id: NodeId, label: &'a str) -> Cell<'a> {
    Cell::new(Widget::Field {
        id,
        glyph: Glyph::Text(letra_da_grandeza(label)),
    })
    .label(label)
}

fn seg(id: NodeId, chave: &'static str, on: bool) -> Seg<'static> {
    Seg {
        id,
        icon: None,
        text: tr(chave),
        on,
    }
}

fn escolha(c: &mut CardCtx<'_>, label: Option<&str>, segs: &[Seg<'_>], y: f32) -> f32 {
    c.full(label, Widget::Choice(segs), y)
}

/// O tipo: os nove, 3 × 3, à vista como antes (cada um muda o que está abaixo: um clique, sem abrir lista).
fn tipos(c: &mut CardCtx<'_>, s: &GridSnapState, y: f32) -> f32 {
    const NOMES: [&str; 9] = [
        "panel.grid_snap.options.square",
        "panel.grid_snap.options.hex",
        "panel.grid_snap.options.iso",
        "panel.grid_snap.options.stag_sq",
        "panel.grid_snap.options.stag_hex",
        "panel.grid_snap.options.tri",
        "panel.grid_snap.options.quadtree",
        "panel.grid_snap.options.voronoi",
        "panel.grid_snap.options.chunks",
    ];
    let segs: Vec<Seg<'static>> = crate::paint_helpers::kind_option_ids_in_order()
        .into_iter()
        .zip(NOMES)
        .map(|((kind, id), k)| seg(id, k, kind == s.kind))
        .collect();
    segs.chunks(3).fold(y, |y, fila| escolha(c, None, fila, y))
}

fn alvo(c: &mut CardCtx<'_>, s: &GridSnapState, y: f32) -> f32 {
    let on = |t: SnapTarget| s.snap_target == t;
    let y = escolha(
        c,
        Some(tr("card.grid.snap_to")),
        &[
            seg(
                g::GS_SNAP_CENTER,
                "panel.grid_snap.options.center",
                on(SnapTarget::Center),
            ),
            seg(
                g::GS_SNAP_INTERSECTION,
                "panel.grid_snap.options.intersection",
                on(SnapTarget::Intersection),
            ),
            seg(
                g::GS_SNAP_TARGET_OPT_CORNER,
                "panel.grid_snap.options.corner",
                on(SnapTarget::Corner),
            ),
        ],
        y,
    );
    let y = escolha(
        c,
        None,
        &[seg(
            g::GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION,
            "panel.grid_snap.options.center_intersection",
            on(SnapTarget::CenterAndIntersection),
        )],
        y,
    );
    escolha(
        c,
        None,
        &[seg(
            g::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS,
            "panel.grid_snap.options.center_intersection_corners",
            on(SnapTarget::CenterIntersectionAndCorners),
        )],
        y,
    )
}

fn def<'a>(
    section: NodeId,
    title: &'a str,
    summary: Option<String>,
    body: Box<dyn Fn(&mut CardCtx<'_>, f32) -> f32 + 'a>,
) -> CardDef<'a> {
    CardDef {
        section,
        title,
        actions: Vec::new(),
        add: None,
        summary,
        body: Some(body),
    }
}

/// O que as duas sondas dizem da grelha, e as duas sondas (X e Y de uma grandeza: um rótulo).
fn inspecao(c: &mut CardCtx<'_>, s: &GridSnapState, y: f32) -> f32 {
    let i = ph2d_editor_core::grid_snap::inspect::snapshot(s);
    let leitura = |chave: &'static str, text| {
        Cell::new(Widget::Readout { text }).label(tr(chave))
    };
    let mut y = c.pair(
        leitura("chrome.grid_snap.probe_a", &i.probe_a_label),
        leitura("chrome.grid_snap.probe_b", &i.probe_b_label),
        y,
    );
    if !i.probe_a_extra.is_empty() {
        y = c.hint(&i.probe_a_extra, y);
    }
    let distancia = if i.distance == u32::MAX {
        tr("card.grid.not_available").to_string()
    } else {
        i.distance.to_string()
    };
    let linha = tr_with(
        "chrome.grid_snap.cells_neighbors",
        &[("cells", &i.line_length), ("neighbors", &i.neighbors)],
    );
    y = c.row(
        &[
            Cell::new(Widget::Readout { text: &distancia }).label(tr("chrome.grid_snap.distance")),
            Cell::new(Widget::Readout { text: &linha })
                .label(tr("chrome.grid_snap.line_neighbors")),
        ],
        y,
    );
    for (chave, xid, yid) in [
        ("chrome.grid_snap.probe_a", g::GS_PROBE_A_X, g::GS_PROBE_A_Y),
        ("chrome.grid_snap.probe_b", g::GS_PROBE_B_X, g::GS_PROBE_B_Y),
    ] {
        let rotulo = com_unidade(tr(chave));
        let x = Cell::new(Widget::Field {
            id: xid,
            glyph: Glyph::Text(tr("card.glyph.x")),
        })
        .label(&rotulo);
        let yc = Cell::new(Widget::Field {
            id: yid,
            glyph: Glyph::Text(tr("card.glyph.y")),
        });
        y = c.row(&[x, yc], y);
    }
    y
}

/// ⭐ Os cartões: o mestre *Snap* mora no alvo, junto do que ele governa; a mesma ordem de antes.
pub(crate) fn cards(s: &GridSnapState) -> Vec<CardDef<'_>> {
    vec![
        def(
            crate::ids::GS_SEC_KIND,
            tr("panel.grid_snap.sections.grid_kind"),
            None,
            Box::new(move |c, y| {
                let y = tipos(c, s, y);
                configuracao(c, s, y)
            }),
        ),
        def(
            crate::ids::GS_SEC_TARGET,
            tr("panel.grid_snap.sections.target"),
            Some(
                tr(if s.snap_enabled {
                    "panel.grid_snap.options.snap_on"
                } else {
                    "panel.grid_snap.options.snap_off"
                })
                .to_string(),
            ),
            Box::new(move |c, y| {
                let y = c.check(
                    g::GS_SNAP_ENABLED,
                    Rotulo::vista(TextKey::new("card.grid.snap")),
                    s.snap_enabled,
                    y,
                );
                let y = alvo(c, s, y);
                let raio = com_unidade(tr("panel.grid_snap.sections.magnetism_radius"));
                c.row(
                    &[
                        campo(
                            g::GS_CFG_SNAP_SUBDIVISIONS,
                            tr("panel.grid_snap.sections.subdivisions"),
                        ),
                        campo(g::GS_CFG_SNAP_MAGNETISM_RADIUS, &raio),
                    ],
                    y,
                )
            }),
        ),
        def(
            crate::ids::GS_SEC_DISPLAY,
            tr("panel.grid_snap.sections.display"),
            None,
            Box::new(move |c, y| {
                let y = c.check(
                    g::GS_SHOW_OVERLAY,
                    Rotulo::vista(TextKey::new("panel.grid_snap.rows.show_grid")),
                    s.show_overlay,
                    y,
                );
                let camada = [
                    seg(
                        g::GS_LAYER_IN_FRONT,
                        "panel.grid_snap.sections.in_front",
                        s.grid_in_front,
                    ),
                    seg(
                        g::GS_LAYER_BEHIND,
                        "panel.grid_snap.sections.behind",
                        !s.grid_in_front,
                    ),
                ];
                c.row(
                    &[
                        campo(chip_da_opacidade(), tr("panel.grid_snap.rows.opacity_pct")),
                        Cell::new(Widget::Choice(&camada))
                            .label(tr("panel.grid_snap.sections.layer")),
                    ],
                    y,
                )
            }),
        ),
        def(
            g::GS_INSPECT_HEADER,
            tr("chrome.grid_snap.inspect"),
            None,
            Box::new(move |c, y| inspecao(c, s, y)),
        ),
    ]
}

/// A janela flutuante: a faixa do título move e os cantos redimensionam, como antes. Registados DEPOIS
/// da lista (o último ganha), a faixa pára antes do «…» e do «×» dela.
pub(crate) fn chrome(ctx: &mut PaintCtx<'_>, lista: Rect) {
    let theme = ctx.host.theme();
    paint_panel_corner_dot(lista, ctx.scene, theme);
    paint_panel_corner_dot_bl(lista, ctx.scene, theme);
    let reserva = m::HEAD_ICON_BOX * 2.0 + m::LIST_PAD_X;
    let hit = ctx.host.hit_index_mut();
    hit.register(g::GS_RESIZE_HANDLE, panel_resize_handle_rect(lista));
    hit.register(g::GS_RESIZE_HANDLE_BL, panel_resize_handle_rect_bl(lista));
    hit.register(
        g::GS_DRAG_HANDLE,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
}
