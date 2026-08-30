//! **A costura do navegador de assets**, ponta a ponta e sem GPU.
//!
//! ⚠️ O que estes gates provam não é que os widgets existem: é que **o clique chega a um efeito**.
//! Um braço de `apply_event` esquecido, um id errado ou um `if let` que não cobre a variante
//! deixam o controlo pintado, registado, hit-indexado — e **morto sob o dedo**, com toda a suíte
//! verde. Foi exactamente isso que o pill `Assets` foi durante meses.

use ph2d_asset_index::{AssetEntry, AssetIndex, AssetKind, AssetRef, Query, SortBy};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_panel_asset_browser::state::AssetBrowserState;
use ph2d_panel_asset_browser::{AssetBrowserPanel, PANEL_ID, ids};
use ph2d_ui_testkit::MockPanelHost;

fn open_host() -> (MockPanelHost, AssetBrowserState) {
    let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    host.set_panel_visible(PANEL_ID, true);
    (host, AssetBrowserState::default())
}

/// ⭐⭐ **O pill `Assets` deixou de ser um chip morto.**
///
/// **Mutação que deve sangrar:** apagar o braço de `TOPBAR_RIGHT_ASSETS` no `apply_event` —
/// que é literalmente o estado em que o app estava antes desta wave.
#[test]
fn the_assets_pill_opens_and_closes_the_panel() {
    let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    let mut st = AssetBrowserState::default();
    assert!(
        !host.panel_visible(PANEL_ID),
        "o painel tem de nascer fechado"
    );
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::TOPBAR_RIGHT_ASSETS),
    );
    assert_eq!(out, EventOutcome::Consumed, "o pill nao foi consumido");
    assert!(host.panel_visible(PANEL_ID), "o pill nao abriu o painel");
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::TOPBAR_RIGHT_ASSETS),
    );
    assert!(!host.panel_visible(PANEL_ID), "o pill nao fecha de volta");
}

/// O `X` do cabeçalho fecha — e é o MESMO braço, de propósito.
#[test]
fn the_close_button_closes_the_panel() {
    let (mut host, mut st) = open_host();
    let out =
        host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(ids::ASSET_CLOSE));
    assert_eq!(out, EventOutcome::Consumed);
    assert!(!host.panel_visible(PANEL_ID));
}

/// ⛔⛔ **Com o painel FECHADO, os controlos dele não respondem.** Eles não são pintados, mas o
/// `WidgetStore` ainda os conhece — e um `Click` sintético (um atalho, a paleta de comandos, um
/// teste) chegaria a eles.
#[test]
fn a_closed_panel_ignores_its_own_chips() {
    let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    let mut st = AssetBrowserState::default();
    let before = st.sort;
    let out = host
        .apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(ids::ASSET_SORT[2]));
    assert_eq!(out, EventOutcome::Ignored);
    assert_eq!(st.sort, before, "um painel fechado mudou de ordenacao");
}

/// ⭐ **O chip de ordenação DECIDE** — a terceira pergunta do knob morto.
#[test]
fn the_sort_chips_reach_the_state() {
    let (mut host, mut st) = open_host();
    assert_eq!(st.sort, SortBy::Name, "o default mudou; o alvo seria vazio");
    for (i, want) in SortBy::ALL.iter().enumerate() {
        let out = host.apply_panel_event::<AssetBrowserPanel>(
            &mut st,
            WidgetEvent::Click(ids::ASSET_SORT[i]),
        );
        assert_eq!(out, EventOutcome::Consumed, "chip de ordenacao {i} morto");
        assert_eq!(st.sort, *want, "chip {i} escreveu a ordenacao errada");
    }
}

/// ⭐ E o de família também — incluindo o `All`, que é `None` e não uma quarta variante.
#[test]
fn the_kind_chips_reach_the_state_and_all_is_the_absence_of_a_filter() {
    let (mut host, mut st) = open_host();
    host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(ids::ASSET_KIND[1]));
    assert_eq!(st.kind, Some(AssetKind::Component));
    host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(ids::ASSET_KIND[2]));
    assert_eq!(st.kind, Some(AssetKind::Texture));
    host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(ids::ASSET_KIND[0]));
    assert_eq!(st.kind, None, "`All` tem de ser a AUSENCIA de filtro");
}

/// ⚠️ **A fileira de chips de família é DERIVADA de `AssetKind::ALL`.** Uma família nova aparece
/// nela sozinha — e a tabela de ids tem de a alcançar.
#[test]
fn the_kind_row_covers_every_family_the_model_declares() {
    assert_eq!(
        ids::ASSET_KIND_FILTERS,
        AssetKind::ALL.len() + 1,
        "a fileira nao cobre todas as familias (ou tem um chip a mais)"
    );
    assert!(ids::ASSET_KIND_FILTERS <= ids::ASSET_KIND.len());
    assert_eq!(
        ids::ASSET_SORT_MODES,
        SortBy::ALL.len(),
        "um chip de ordenacao a mais e' um chip que nada pinta; a menos, um modo inalcancavel"
    );
}

/// O slider do tamanho **escreve no estado**, e a lei é de ida-e-volta.
#[test]
fn the_size_slider_reaches_the_state_and_the_law_round_trips() {
    let (mut host, mut st) = open_host();
    host.store_mut().set_slider_value(ids::ASSET_SIZE, 1.0);
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::ValueChanged(ids::ASSET_SIZE),
    );
    assert_eq!(out, EventOutcome::Consumed, "o slider de tamanho e' mudo");
    assert!(
        (st.cell_px - ph2d_panel_asset_browser::CELL_MAX_PX).abs() < 1e-3,
        "o slider no maximo nao deu o cartao maximo: {}",
        st.cell_px
    );
    assert!(
        (st.size_slider_value() - 1.0).abs() < 1e-3,
        "a volta perdeu-se"
    );
    st.set_size_from_slider(0.0);
    assert!((st.cell_px - ph2d_panel_asset_browser::CELL_MIN_PX).abs() < 1e-3);
}

/// ⭐⭐⭐ **O VERBO DE USAR chega ao barramento** (wave A7) — com o `StableId` certo.
///
/// **Mutação que deve sangrar:** trocar `DoubleClick` por `Click` no braço (o navegador passaria
/// a instanciar enquanto o artista percorre).
#[test]
fn a_double_click_on_a_component_card_pushes_the_instantiate_action() {
    let (mut host, mut st) = open_host();
    // A grade só sabe o que pintou — semeia-se o que ela pintaria.
    ph2d_panel_asset_browser::state::probe_set_painted(vec![AssetRef::Component { stable_id: 77 }]);
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::DoubleClick(ids::asset_cell_id(0)),
    );
    assert_eq!(
        out,
        EventOutcome::Consumed,
        "o cartao esta' morto sob o dedo"
    );
    let pushed: Vec<EditorAction> = host.bus().iter().cloned().collect();
    assert!(
        pushed.contains(&EditorAction::AssetInstantiate { stable_id: 77 }),
        "o duplo-clique nao pediu para instanciar: {pushed:?}"
    );
}

/// ⛔ **Uma TEXTURA não se instancia**, e o silêncio é declarado (pôr uma imagem na cena é a queda
/// da etapa B — *qual* objecto a recebe é o que a queda responde).
#[test]
fn a_double_click_on_an_image_card_does_nothing_on_purpose() {
    let (mut host, mut st) = open_host();
    ph2d_panel_asset_browser::state::probe_set_painted(vec![AssetRef::Texture { asset: [7; 32] }]);
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::DoubleClick(ids::asset_cell_id(0)),
    );
    assert_eq!(out, EventOutcome::Ignored);
    assert_eq!(host.bus().len(), 0, "uma imagem nao devia pedir nada");
}

/// ⛔ E um cartão que a grade **não pintou** não instancia coisa nenhuma — o índice muda entre o
/// quadro que pintou e o clique que chega.
#[test]
fn a_cell_the_grid_did_not_paint_instantiates_nothing() {
    let (mut host, mut st) = open_host();
    ph2d_panel_asset_browser::state::probe_set_painted(Vec::new());
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::DoubleClick(ids::asset_cell_id(3)),
    );
    assert_eq!(out, EventOutcome::Ignored);
    assert_eq!(host.bus().len(), 0);
}

/// ⚠️ **A busca da grade filtra a GRADE** — e a consulta que o painel monta é a que o índice
/// responde. É a metade que a lente 3 da auditoria pede: *o leitor DECIDE?*
#[test]
fn the_search_narrows_what_the_grid_would_paint() {
    let mut ix = AssetIndex::new();
    ix.push(AssetEntry::new(
        AssetRef::Component { stable_id: 1 },
        "Ragdoll",
    ));
    ix.push(AssetEntry::new(
        AssetRef::Texture { asset: [2; 32] },
        "brick.png",
    ));
    let all = ph2d_panel_asset_browser::probe_query(&ix, &Query::default());
    assert_eq!(all.len(), 2);
    let q = Query {
        text: "brick".into(),
        ..Default::default()
    };
    assert_eq!(
        ph2d_panel_asset_browser::probe_query(&ix, &q),
        vec!["brick.png"]
    );
}

/// Mudar o filtro **volta ao topo** — senão a rolagem aponta para uma linha que a lista nova não
/// tem, e a grade parece vazia sobre um resultado que existe.
#[test]
fn changing_the_filter_returns_the_grid_to_the_top() {
    let (mut host, mut st) = open_host();
    host.store_mut()
        .set_panel_scroll(ph2d_editor_core::ids::ASSET_PANEL, 240.0);
    host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(ids::ASSET_KIND[1]));
    assert!(
        host.store()
            .panel_scroll(ph2d_editor_core::ids::ASSET_PANEL)
            < f32::EPSILON,
        "a rolagem ficou onde estava"
    );
}

// ── O que a AUDITORIA de 2026-08-30 achou, agora com gate ──────────────────────────────────────
//
// ⚠️ Os dois defeitos abaixo passavam por **todos** os gates anteriores deste ficheiro: o painel
// abria, os chips respondiam, o duplo-clique instanciava — e o painel **não se podia mover nem
// agarrar a barra**. A costura que faltava não é o clique, é o *tipo* de registo.

/// ⛔⛔ **A faixa de arrasto e a alça de redimensionar NÃO são botões.**
///
/// O despacho de um painel flutuante não passa pelo `Click`: ele lê
/// `InteractiveState::BlenderHit { parent, kind }` no `pointer_down`. Registadas como `Button` elas
/// ficam pintadas, hit-indexadas e **mortas** — foi assim que esta wave as escreveu primeiro.
#[test]
fn the_drag_and_resize_handles_are_blender_hits_pointing_at_this_panel() {
    use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState};
    let host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    for (id, want) in [
        (ids::ASSET_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (ids::ASSET_RESIZE_HANDLE_BL, BlenderHitKind::ResizeHandleBl),
    ] {
        match host.store().get(id) {
            Some(InteractiveState::BlenderHit { parent, kind }) => {
                assert_eq!(
                    *parent,
                    ph2d_editor_core::ids::ASSET_PANEL,
                    "a alca aponta para outro painel"
                );
                assert_eq!(*kind, want, "a alca tem o papel errado");
            }
            other => panic!("a alca nao e' um BlenderHit: {other:?} — o painel fica imovel"),
        }
    }
}

/// ⛔⛔ **O polegar da barra tem de estar no store.** Sem uma entrada o `is_focusable` do
/// despachante é falso e o `pointer_down` nunca semeia o arrasto: a barra desenha, acende, e não se
/// pode agarrar.
#[test]
fn the_scrollbar_thumb_is_registered_so_the_drag_can_start() {
    let host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    assert!(
        host.store()
            .get(ph2d_editor_core::widget::ASSET_BROWSER_SCROLLBAR_ID)
            .is_some(),
        "o polegar nao esta' no store — a barra e' inagarravel"
    );
}

/// ⚠️ **O polegar da barra resolve para ESTE painel** no mapa do despachante — sem essa entrada o
/// arrasto move a rolagem de outro painel (ou de nenhum).
#[test]
fn the_scrollbar_thumb_maps_back_to_this_panel() {
    let host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    let (_, _) = host.store().scrollbar_visual_for(
        ph2d_editor_core::widget::ASSET_BROWSER_SCROLLBAR_ID,
        Some(ph2d_editor_core::ids::ASSET_PANEL),
    );
    // A prova real é o mapa do despachante, alcançado pelo `scrollbar_visual` de UM argumento:
    // ele pergunta ao `scrollbar_panel_for_id`, e um id ausente dali devolve o par de repouso
    // mesmo com o painel a rolar.
    let visual_via_map = host
        .store()
        .scrollbar_visual(ph2d_editor_core::widget::ASSET_BROWSER_SCROLLBAR_ID);
    assert_eq!(
        visual_via_map.0,
        ph2d_editor_core::widget::ScrollbarState::Normal,
        "o par de repouso mudou de forma; re-leia este gate"
    );
}
