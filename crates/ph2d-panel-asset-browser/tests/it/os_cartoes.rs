//! ⭐ **Assets em cartões: nenhum controlo morreu.** A grade continua grade, a árvore continua árvore;
//! cada gesto é o do ponteiro real, nos rects que a pintura registou.

use ph2d_a11y::NodeId;
use ph2d_asset_index::{AssetEntry, AssetIndex, AssetKind, AssetRef, CatalogTree};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::asset_cell_id;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{dicas, dropdown, list_menu_of};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_asset_browser::state::AssetBrowserState;
use ph2d_panel_asset_browser::{
    AssetBrowserPanel, CatalogPick, PANEL_ID, cards, ids, set_current_catalogs, set_current_index,
};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1400.0,
};
const PAINEL: NodeId = ph2d_editor_core::ids::ASSET_PANEL;
const CASA: AssetRef = AssetRef::Component { stable_id: 10 };
const CASCA: AssetRef = AssetRef::Texture { asset: [1; 32] };

fn biblioteca(n: usize) {
    let mut ix = AssetIndex::new();
    ix.push(AssetEntry::new(CASCA, "bark"));
    ix.push(AssetEntry::new(CASA, "house"));
    for i in 0..n {
        ix.push(AssetEntry::new(
            AssetRef::Texture {
                asset: [(i + 2) as u8; 32],
            },
            &format!("image {i}"),
        ));
    }
    set_current_index(ix);
}

struct Mesa {
    host: MockPanelHost,
    st: AssetBrowserState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn com(st: AssetBrowserState, n: usize) -> Self {
        cards::force_cards_on_this_thread(Some(true));
        biblioteca(n);
        let mut tree = CatalogTree::default();
        tree.create("Heroes");
        set_current_catalogs(tree);
        let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
        host.set_panel_visible(PANEL_ID, true);
        host.set_collapsed(ids::ASSET_SEC_CATALOGS, false);
        let mut mesa = Self {
            host,
            st,
            rects: Vec::new(),
        };
        mesa.assenta();
        mesa
    }

    fn nova() -> Self {
        Self::com(
            AssetBrowserState {
                rect: Some(Rect::new(100.0, 40.0, 420.0, 1300.0)),
                ..AssetBrowserState::default()
            },
            0,
        )
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<AssetBrowserPanel>(st, JANELA),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Rect {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map_or_else(|| panic!("{id:?} não pintado"), |(_, r)| *r)
    }

    fn pintado(&self, id: NodeId) -> bool {
        self.rects.iter().any(|(n, r)| *n == id && r.w > 0.0)
    }

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<EditorAction> {
        for e in eventos {
            if let WidgetEvent::Click(id) = e
                && dropdown::on_click(id)
            {
                continue;
            }
            let _ = self
                .host
                .apply_panel_event::<AssetBrowserPanel>(&mut self.st, e);
        }
        self.host.drained_actions()
    }

    fn clica(&mut self, id: NodeId) -> Vec<EditorAction> {
        let r = self.rect(id);
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "{id:?} morto sob o dedo: {ev:?}"
        );
        self.aplica(ev)
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

/// Todos os ids do corpo de antes, em algum estado alcançável; a barra de tamanho nunca (é a pega do
/// campo), e a coluna deixou de ter barra própria.
#[test]
fn every_control_of_the_old_panel_is_painted_in_cards() {
    let mut mesa = Mesa::nova();
    let mut ids_fixos = vec![
        ids::ASSET_SEARCH,
        ids::ASSET_CATALOG_TOGGLE,
        ids::ASSET_CATALOG_NEW,
        ids::ASSET_KIND_BOX,
        ids::ASSET_SIZE_NUM,
        ids::ASSET_CLOSE,
        ids::ASSET_DRAG_HANDLE,
        ids::ASSET_RESIZE_HANDLE_BL,
        asset_cell_id(0),
        ph2d_editor_core::ids::catalog_row_id(0),
    ];
    ids_fixos.extend(ids::ASSET_SORT);
    for id in ids_fixos {
        assert!(mesa.pintado(id), "{id:?} morreu");
        assert!(mesa.host.store().get(id).is_some(), "{id:?} sem registo");
    }
    assert!(!mesa.pintado(ids::ASSET_SIZE), "a barra deixa de se pintar");
    let _ = mesa.clica(ids::ASSET_KIND_BOX);
    mesa.assenta();
    for id in ids::ASSET_KIND {
        assert!(mesa.pintado(id), "a opção {id:?} não abriu");
    }
}

/// Cada família pela lista suspensa e cada ordem pelo segmento escrevem o que o corpo de antes escrevia.
#[test]
fn the_kind_dropdown_and_the_sort_choice_answer_a_real_click() {
    let mut mesa = Mesa::nova();
    let _ = mesa.clica(ids::ASSET_KIND_BOX);
    mesa.assenta();
    let alvo = (0..ids::ASSET_KIND_FILTERS)
        .find(|i| AssetBrowserState::kind_for_chip(*i) == Some(AssetKind::Texture))
        .expect("a família das imagens");
    let _ = mesa.clica(ids::ASSET_KIND[alvo]);
    assert_eq!(mesa.st.kind, Some(AssetKind::Texture));
    mesa.assenta();
    assert!(mesa.pintado(asset_cell_id(0)));
    assert!(
        !mesa.pintado(asset_cell_id(1)),
        "o filtro deixou passar a casa"
    );
    let antes = mesa.st.sort;
    let _ = mesa.clica(ids::ASSET_SORT[1]);
    assert_ne!(mesa.st.sort, antes);
}

/// A busca filtra a grade: o texto vive no store, como antes.
#[test]
fn typing_in_the_search_filters_the_grid() {
    let mut mesa = Mesa::nova();
    assert!(mesa.pintado(asset_cell_id(1)));
    mesa.host.set_text(ids::ASSET_SEARCH, "bark");
    mesa.assenta();
    assert!(mesa.pintado(asset_cell_id(0)));
    assert!(!mesa.pintado(asset_cell_id(1)));
}

/// ⭐ O campo do tamanho pelo scrub real: avisa pela barra, a grade muda de colunas e o Esc devolve.
#[test]
fn scrubbing_the_size_writes_the_cell_and_escape_gives_it_back() {
    let mut mesa = Mesa::nova();
    let antes = mesa.st.cell_px;
    let largura_antes = mesa.rect(asset_cell_id(0)).w;
    let r = mesa.rect(ids::ASSET_SIZE_NUM);
    let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
    ev.extend(mesa.ponteiro(PointerKind::Move, x + 30.0, y));
    ev.extend(mesa.ponteiro(PointerKind::Move, x + 60.0, y));
    assert!(
        ev.contains(&WidgetEvent::ValueChanged(ids::ASSET_SIZE)),
        "o campo tem de avisar pela barra: {ev:?}"
    );
    let _ = mesa.aplica(ev);
    assert!(mesa.st.cell_px > antes + 1.0, "{}", mesa.st.cell_px);
    mesa.assenta();
    assert!(mesa.rect(asset_cell_id(0)).w > largura_antes);
    let arena = bumpalo::Bump::new();
    let esc = ph2d_host::KeyEvent {
        keycode: 0x1B,
        modifiers: ph2d_host::Modifiers::default(),
        kind: ph2d_host::KeyKind::Down,
        timestamp_ns: 0,
    };
    let ev = dispatch_key(mesa.host.store_mut(), esc, &arena).to_vec();
    let _ = mesa.aplica(ev);
    assert!(
        (mesa.st.cell_px - antes).abs() < 0.01,
        "{}",
        mesa.st.cell_px
    );
}

/// O catálogo: escolher filtra o escopo, o «+» pede o verbo, o interruptor esconde o cartão, o duplo
/// clique abre o campo por cima da linha e o foco vai para ele.
#[test]
fn the_catalog_card_keeps_every_gesture_of_the_column() {
    let mut mesa = Mesa::nova();
    let _ = mesa.clica(ph2d_editor_core::ids::catalog_row_id(1));
    assert_eq!(mesa.st.pick, CatalogPick::Unassigned);
    let out = mesa.clica(ids::ASSET_CATALOG_NEW);
    assert!(
        matches!(out.as_slice(), [EditorAction::AssetCatalogVerb(_)]),
        "{out:?}"
    );
    let CatalogPick::One(id) =
        ph2d_panel_asset_browser::catalog_row_pick(ph2d_editor_core::ids::catalog_row_id(2))
            .expect("Heroes")
    else {
        panic!("a 3.ª linha é um catálogo");
    };
    mesa.st.renaming = Some(ph2d_panel_asset_browser::CatalogRename { id, opened: false });
    mesa.assenta();
    assert_eq!(
        mesa.host.store().focus_id(),
        Some(ids::ASSET_CATALOG_RENAME)
    );
    assert_eq!(
        mesa.host.store().text(ids::ASSET_CATALOG_RENAME),
        Some("Heroes")
    );
    let linha = mesa.rect(ph2d_editor_core::ids::catalog_row_id(2));
    assert_eq!(mesa.rect(ids::ASSET_CATALOG_RENAME).y, linha.y);
    let _ = mesa.clica(ids::ASSET_CATALOG_TOGGLE);
    mesa.assenta();
    assert!(!mesa.st.show_catalogs);
    assert!(!mesa.pintado(ph2d_editor_core::ids::catalog_row_id(0)));
    assert!(
        mesa.pintado(ids::ASSET_CATALOG_TOGGLE),
        "o interruptor fica"
    );
    assert_ne!(
        mesa.host.store().focus_id(),
        Some(ids::ASSET_CATALOG_RENAME),
        "esconder não pode deixar um campo invisível a comer teclas"
    );
}

/// ⭐ A miniatura: duplo clique instancia, botão direito abre o menu, e o despachante sabe que é uma
/// célula (é o que arma o arrasto para o canvas).
#[test]
fn a_thumbnail_still_opens_its_menu_is_a_drag_source_and_instantiates() {
    let mut mesa = Mesa::nova();
    let casa = (0..2)
        .find(|i| ph2d_panel_asset_browser::probe_painted_at(*i) == Some(CASA))
        .expect("a casa");
    let id = asset_cell_id(casa);
    let r = mesa.rect(id);
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(mesa.host.hit_at(cx, cy), Some(id));
    let out = mesa.aplica(vec![WidgetEvent::DoubleClick(id)]);
    assert!(matches!(
        out.as_slice(),
        [EditorAction::AssetInstantiate { stable_id: 10, .. }]
    ));
    let ev = mesa.host.right_click_at(cx, cy);
    let _ = mesa.aplica(ev);
    assert!(matches!(
        mesa.host.store().context_menu().map(|c| c.kind),
        Some(ContextMenuKind::AssetCard { cell }) if cell == id
    ));
    mesa.host.store_mut().close_context_menu();
    // Quem arranca o arrasto é a shell, ao ver um `is_asset_cell` debaixo do Down.
    assert_eq!(mesa.host.store().asset_cell_index(id), Some(casa));
}

/// O seletor de imagem: a faixa nomeia o alvo, só há imagens, o clique escolhe e fecha; o «×» desiste.
#[test]
fn the_image_picker_band_chooses_and_cancels() {
    ph2d_panel_asset_browser::choose_image_for(55, "Hero".into());
    let mut mesa = Mesa::nova();
    assert!(mesa.st.choosing.is_some());
    let out = mesa.clica(asset_cell_id(0));
    assert!(
        matches!(
            out.as_slice(),
            [EditorAction::ChooseImage {
                entity_bits: 55,
                ..
            }]
        ),
        "{out:?}"
    );
    ph2d_panel_asset_browser::choose_image_for(55, "Hero".into());
    let mut mesa = Mesa::nova();
    let _ = mesa.clica(ids::ASSET_CHOOSE_CANCEL);
    assert!(mesa.st.choosing.is_none());
    assert!(!mesa.host.panel_visible(PANEL_ID));
}

/// A faixa do filtro de relação traz o próprio «×».
#[test]
fn the_relation_band_clears_its_filter() {
    let mut st = AssetBrowserState {
        rect: Some(Rect::new(100.0, 40.0, 420.0, 1300.0)),
        ..AssetBrowserState::default()
    };
    st.related = Some((CASA, ph2d_asset_index::Relation::Uses));
    let mut mesa = Mesa::com(st, 0);
    let _ = mesa.clica(ids::ASSET_RELATED_CLEAR);
    assert!(mesa.st.related.is_none());
}

/// O painel que flutua continua a mover-se e a redimensionar-se; arrasto e «×» não se tapam.
#[test]
fn the_floating_list_moves_and_resizes_by_its_old_handles() {
    let mut mesa = Mesa::nova();
    let faixa = mesa.rect(ids::ASSET_DRAG_HANDLE);
    let fecho = mesa.rect(ids::ASSET_CLOSE);
    assert!(faixa.x + faixa.w <= fecho.x, "{faixa:?} tapa {fecho:?}");
    let (x, y) = (faixa.x + 20.0, faixa.y + faixa.h * 0.5);
    assert_eq!(mesa.host.hit_at(x, y), Some(ids::ASSET_DRAG_HANDLE));
    let _ = mesa.ponteiro(PointerKind::Down, x, y);
    let _ = mesa.ponteiro(PointerKind::Move, x + 50.0, y + 30.0);
    let _ = mesa.ponteiro(PointerKind::Up, x + 50.0, y + 30.0);
    mesa.assenta();
    let nova = mesa.rect(ids::ASSET_DRAG_HANDLE);
    assert!(
        (nova.x - faixa.x - 50.0).abs() < 1.0,
        "{nova:?} × {faixa:?}"
    );
    let alca = mesa.rect(ids::ASSET_RESIZE_HANDLE_BL);
    let (x, y) = (alca.x + alca.w * 0.5, alca.y + alca.h * 0.5);
    assert_eq!(mesa.host.hit_at(x, y), Some(ids::ASSET_RESIZE_HANDLE_BL));
    let w = nova.w;
    let _ = mesa.ponteiro(PointerKind::Down, x, y);
    let _ = mesa.ponteiro(PointerKind::Move, x - 60.0, y);
    let _ = mesa.ponteiro(PointerKind::Up, x - 60.0, y);
    mesa.assenta();
    assert!(
        mesa.rect(ids::ASSET_DRAG_HANDLE).w > w + 30.0,
        "não alargou"
    );
}

/// A grade grande rola com a lista; uma célula de baixo responde; a de cima sai do dedo.
#[test]
fn a_big_library_scrolls_and_the_lower_thumbnail_answers() {
    let mut mesa = Mesa::com(
        AssetBrowserState {
            rect: Some(Rect::new(100.0, 40.0, 420.0, 500.0)),
            ..AssetBrowserState::default()
        },
        120,
    );
    let ultima = asset_cell_id(121);
    assert!(
        !mesa.pintado(ultima),
        "fora da janela a célula não paga nem regista"
    );
    let lista = mesa.host.store().panel_rect(PAINEL).expect("lista");
    let _ = mesa
        .host
        .wheel_at(lista.x + 50.0, lista.y + 100.0, -20000.0);
    mesa.assenta();
    let r = mesa.rect(ultima);
    assert!(
        r.y + r.h <= lista.y + lista.h + 0.5,
        "{r:?} fora de {lista:?}"
    );
    assert_eq!(mesa.host.hit_at(r.x + 5.0, r.y + 5.0), Some(ultima));
}

/// Dobra, reordena e menu do cabeçalho, como todo cartão.
#[test]
fn a_card_folds_reorders_and_opens_its_menu() {
    let mut mesa = Mesa::nova();
    let r = mesa.rect(ids::ASSET_SEC_FIND);
    let ev = mesa
        .host
        .right_click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.aplica(ev);
    assert!(matches!(
        mesa.host.store().context_menu().map(|c| c.kind),
        Some(ContextMenuKind::SectionOutline { section }) if section == ids::ASSET_SEC_FIND
    ));
    mesa.host.store_mut().close_context_menu();
    let _ = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    mesa.assenta();
    assert!(mesa.host.store().is_collapsed(ids::ASSET_SEC_FIND));
    assert!(!mesa.pintado(ids::ASSET_SEARCH));
}

/// Todo ícone sem texto tem dica.
#[test]
fn every_mute_control_has_a_tooltip() {
    let mesa = Mesa::nova();
    let store = mesa.host.store();
    let mudos: Vec<NodeId> = dicas::icon_only_ids(PAINEL)
        .into_iter()
        .chain([
            ids::ASSET_CATALOG_NEW,
            ids::ASSET_CATALOG_TOGGLE,
            ids::ASSET_CLOSE,
            list_menu_of(PAINEL),
        ])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
}

/// `PH2D_CARDS=0`: os chips e a barra de antes, sem cartões.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    biblioteca(0);
    let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    host.set_panel_visible(PANEL_ID, true);
    let mut st = AssetBrowserState::default();
    let rects = host.paint::<AssetBrowserPanel>(&mut st, JANELA);
    let tem = |id| rects.iter().any(|(n, _)| *n == id);
    assert!(tem(ids::ASSET_SIZE) && tem(ids::ASSET_KIND[0]));
    assert!(!tem(ids::ASSET_KIND_BOX) && !tem(ids::ASSET_SEC_FIND));
}
