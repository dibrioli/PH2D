//! ⭐⭐ **O painel Grid Settings em cartões: nenhum controlo morreu.** Cada id é provado pelo gesto REAL
//! (ponteiro → despacho → `apply_event` → `GridSnapState`), nos rects pintados, nos nove tipos.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::grid_snap::ids as g;
use ph2d_editor_core::grid_snap::{GridKind, GridSnapState};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_grid::snap::SnapTarget;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_grid_snap::state::GridSnapPanelState;
use ph2d_panel_grid_snap::{GridSnapPanel, cards};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};

struct Mesa {
    host: MockPanelHost,
    st: GridSnapPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova() -> Self {
        cards::force_cards_on_this_thread(Some(true));
        let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
        host.set_panel_visible(GridSnapPanel::ID, true);
        // A janela de fábrica é baixa: a lista rola. Aqui todos os cartões à vista.
        host.set_grid_snap_panel_rect(Some(Rect::new(100.0, 40.0, 340.0, 2400.0)));
        let mut mesa = Self {
            host,
            st: GridSnapPanelState,
            rects: Vec::new(),
        };
        mesa.assenta();
        mesa
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<GridSnapPanel>(st, JANELA),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) {
        for e in eventos {
            let _ = self
                .host
                .apply_panel_event::<GridSnapPanel>(&mut self.st, e);
        }
    }

    fn clica(&mut self, id: NodeId) {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        self.aplica(ev);
        self.assenta();
    }

    fn estado(&self) -> GridSnapState {
        self.host.grid_snap_state_clone()
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        })
    }
}

const TIPOS: [(GridKind, NodeId); 9] = [
    (GridKind::Square, g::GS_KIND_OPT_SQUARE),
    (GridKind::Hex, g::GS_KIND_OPT_HEX),
    (GridKind::Iso, g::GS_KIND_OPT_ISO),
    (GridKind::StaggeredSquare, g::GS_KIND_OPT_STAGGERED_SQ),
    (GridKind::StaggeredHex, g::GS_KIND_OPT_STAGGERED_HEX),
    (GridKind::Tri, g::GS_KIND_OPT_TRI),
    (GridKind::Quadtree, g::GS_KIND_OPT_QUADTREE),
    (GridKind::Voronoi, g::GS_KIND_OPT_VORONOI),
    (GridKind::Chunks, g::GS_KIND_OPT_CHUNKS),
];

/// ⭐ Cada tipo responde ao clique, e todo controlo do corpo de antes pinta-se em algum tipo.
#[test]
fn every_kind_answers_a_click_and_every_id_is_painted_in_some_kind() {
    let mut mesa = Mesa::nova();
    let mut pintados = std::collections::BTreeSet::new();
    for (kind, id) in TIPOS {
        mesa.clica(id);
        assert_eq!(mesa.estado().kind, kind);
        pintados.extend(mesa.rects.iter().map(|(n, _)| *n));
    }
    let inventario = [
        g::GS_CLOSE,
        g::GS_SNAP_ENABLED,
        g::GS_SHOW_OVERLAY,
        ph2d_panel_grid_snap::cards::chip_da_opacidade(),
        g::GS_DRAG_HANDLE,
        g::GS_RESIZE_HANDLE,
        g::GS_RESIZE_HANDLE_BL,
        g::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS,
        g::GS_CFG_NEIGHBORHOOD_4,
        g::GS_CFG_NEIGHBORHOOD_8,
        g::GS_CFG_TRI_EDGE3,
        g::GS_CFG_TRI_VERTEX12,
        g::GS_CFG_HEX_POINTY,
        g::GS_CFG_HEX_OFFSET_EVENQ,
        g::GS_CFG_STAGGER_PARITY_EVEN,
        g::GS_CFG_VORONOI_RESEED,
        g::GS_LAYER_BEHIND,
        g::GS_CFG_CELL_SIZE,
        g::GS_CFG_ISO_TILE_H,
        g::GS_CFG_QT_MAX_DEPTH,
        g::GS_CFG_QT_DEMO_SEED,
        g::GS_CFG_QT_BOUNDS_MAX_Y,
        g::GS_CFG_VORONOI_LLOYD_ITERS,
        g::GS_CFG_VORONOI_BOUNDS_MAX_Y,
        g::GS_CFG_CHUNKS_SIZE,
        g::GS_CFG_ORIGIN_Y,
        g::GS_CFG_SPACING_MAJOR,
        g::GS_CFG_SNAP_SUBDIVISIONS,
        g::GS_CFG_SNAP_MAGNETISM_RADIUS,
        g::GS_PROBE_B_Y,
    ];
    let mortos: Vec<NodeId> = inventario
        .into_iter()
        .filter(|id| !pintados.contains(id))
        .collect();
    assert!(mortos.is_empty(), "nunca pintados: {mortos:?}");
}

/// As duas caixas viram o estado; o alvo e a camada escrevem o seu campo; o «×» fecha.
#[test]
fn checks_choices_and_close_answer_a_real_click() {
    let mut mesa = Mesa::nova();
    let snap = mesa.estado().snap_enabled;
    mesa.clica(g::GS_SNAP_ENABLED);
    assert_eq!(mesa.estado().snap_enabled, !snap, "Snap");
    let vis = mesa.estado().show_overlay;
    mesa.clica(g::GS_SHOW_OVERLAY);
    assert_eq!(mesa.estado().show_overlay, !vis, "Show grid");
    mesa.clica(g::GS_SNAP_TARGET_OPT_CORNER);
    assert_eq!(mesa.estado().snap_target, SnapTarget::Corner);
    mesa.clica(g::GS_LAYER_BEHIND);
    assert!(!mesa.estado().grid_in_front);
    // Sem pintar a seguir: o `paint` do mock liga o painel que pinta.
    let r = mesa.rect(g::GS_CLOSE).expect("«×»");
    let ev = mesa.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    mesa.aplica(ev);
    assert!(!mesa.host.panel_visible(GridSnapPanel::ID), "o «×» fecha");
}

/// O scrub real do tamanho de célula escreve o estado; escrever também.
#[test]
fn scrubbing_and_typing_a_number_write_the_state() {
    let mut mesa = Mesa::nova();
    let antes = mesa.estado().square_cfg.cell_size;
    let r = mesa.rect(g::GS_CFG_CELL_SIZE).expect("campo");
    let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
    ev.extend(mesa.ponteiro(PointerKind::Move, x + 30.0, y));
    ev.extend(mesa.ponteiro(PointerKind::Move, x + 60.0, y));
    mesa.aplica(ev);
    let depois = mesa.estado().square_cfg.cell_size;
    assert!((depois - antes).abs() > 1e-4, "{antes} → {depois}");
    let _ = mesa.ponteiro(PointerKind::Up, x + 60.0, y);
    mesa.assenta();
    let ev = mesa.host.type_into_number(g::GS_CFG_SNAP_SUBDIVISIONS, "3");
    mesa.aplica(ev);
    assert_eq!(mesa.estado().snap_subdivisions, 3);
}

/// ⭐ A Opacity é um campo em %, não a barra antiga: o scrub real escreve o estado pela barra de sempre, o Esc devolve, e
/// escrever vale. *Mutações: sem `link_slider`; sem `mark_card_field`; a barra de volta no corpo.*
#[test]
fn opacity_is_a_percent_field_that_writes_the_state() {
    let chip = ph2d_panel_grid_snap::cards::chip_da_opacidade();
    let mut mesa = Mesa::nova();
    assert!(
        mesa.rect(g::GS_OPACITY_SLIDER).is_none(),
        "a barra antiga no corpo"
    );
    let antes = mesa.estado().opacity;
    assert_eq!(
        mesa.host.store().number_value(chip),
        Some(f64::from((antes * 100.0).round()))
    );
    let r = mesa.rect(chip).expect("campo");
    assert_eq!(r.h, m::FIELD_H);
    let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
    ev.extend(mesa.ponteiro(PointerKind::Move, x - 5.0, y));
    ev.extend(mesa.ponteiro(PointerKind::Move, x - 10.0, y));
    mesa.aplica(ev);
    let depois = mesa.estado().opacity;
    assert!(depois < antes - 0.01, "{antes} → {depois}");
    let _ = mesa.ponteiro(PointerKind::Up, x - 10.0, y);
    mesa.assenta();
    let ev = mesa.host.type_into_number(chip, "40");
    mesa.aplica(ev);
    assert!(
        (mesa.estado().opacity - 0.4).abs() < 1e-3,
        "{}",
        mesa.estado().opacity
    );
}

/// O estado muda por fora (o menu Grid, o Desfazer): a caixa diz o número novo e os dois marcar dizem o
/// que o estado diz, e o clique seguinte parte daí.
/// *Mutações: sem `set_number_value`; sem a sincronia dos `Toggle`.*
#[test]
fn a_change_from_outside_reaches_the_boxes_and_the_next_click_starts_from_it() {
    let mut mesa = Mesa::nova();
    let celula0 = mesa.estado().square_cfg.cell_size;
    let celula = celula0 * 2.0;
    let caixa0 = mesa
        .host
        .store()
        .number_value(g::GS_CFG_CELL_SIZE)
        .expect("caixa");
    let (snap, vis) = (!mesa.estado().snap_enabled, !mesa.estado().show_overlay);
    {
        let s = mesa.host.grid_snap_state_mut();
        s.square_cfg.cell_size = celula;
        s.snap_subdivisions = 7;
        s.snap_enabled = snap;
        s.show_overlay = vis;
    }
    mesa.assenta();
    let caixa = mesa
        .host
        .store()
        .number_value(g::GS_CFG_CELL_SIZE)
        .expect("caixa");
    assert!((caixa - caixa0 * 2.0).abs() < 1e-3, "{caixa} × {caixa0}");
    // O que não é comprimento só tem esta porta (o de metros tem também a da unidade).
    assert_eq!(
        mesa.host.store().number_value(g::GS_CFG_SNAP_SUBDIVISIONS),
        Some(7.0)
    );
    mesa.clica(g::GS_SNAP_ENABLED);
    assert_eq!(mesa.estado().snap_enabled, !snap, "partiu do estado");
    mesa.clica(g::GS_SHOW_OVERLAY);
    assert_eq!(mesa.estado().show_overlay, !vis, "partiu do estado");
}

/// A faixa do título move a janela; a faixa não tapa o «×».
#[test]
fn the_floating_list_moves_by_its_title() {
    let mut mesa = Mesa::nova();
    let faixa = mesa.rect(g::GS_DRAG_HANDLE).expect("faixa");
    let (x, y) = (faixa.x + 10.0, faixa.y + faixa.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y);
    let _ = mesa.ponteiro(PointerKind::Move, x + 120.0, y + 60.0);
    let _ = mesa.ponteiro(PointerKind::Up, x + 120.0, y + 60.0);
    mesa.assenta();
    let movida = mesa.rect(g::GS_DRAG_HANDLE).expect("faixa");
    assert!(
        (movida.x - faixa.x - 120.0).abs() < 1.0,
        "{faixa:?} → {movida:?}"
    );
    let fecho = mesa.rect(g::GS_CLOSE).expect("«×»");
    assert_eq!(
        mesa.host
            .hit_at(fecho.x + fecho.w * 0.5, fecho.y + fecho.h * 0.5),
        Some(g::GS_CLOSE)
    );
}

/// `PH2D_CARDS=0`: o corpo de antes, sem campos de cartão.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
    host.set_panel_visible(GridSnapPanel::ID, true);
    let mut st = GridSnapPanelState;
    let _ = host.paint::<GridSnapPanel>(&mut st, JANELA);
    assert!(host.store().card_field(g::GS_CFG_CELL_SIZE).is_none());
}
