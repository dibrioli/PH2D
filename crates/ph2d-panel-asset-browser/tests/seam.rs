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
        pushed.contains(&EditorAction::AssetInstantiate {
            stable_id: 77,
            // ⚠️ `None` — um duplo-clique nao aponta para lado nenhum; a QUEDA e' que aponta.
            at: None
        }),
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

// ── ⭐⭐ O MENU DO CARTÃO (etapa C) ──────────────────────────────────────────────────────────────

/// Encena um menu de cartão já fechado, como o `pointer_down` faz: abrir estaciona o pedido, e
/// fechar move-o para o `last_context_menu`, que é de onde o `apply_event` o lê.
fn stage_card_menu(host: &mut MockPanelHost, cell: ph2d_a11y::NodeId) {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::AssetCard { cell },
    });
    host.store_mut().close_context_menu();
}

/// ⭐⭐ **Toda linha do menu de um cartão chega a um EFEITO.**
///
/// ⚠️ **A fonte é a TABELA** (`menu_rows(AssetCard)` — a mesma que o overlay pinta), nunca uma
/// lista aqui dentro: um verbo novo entra neste gate no dia em que é pintado, sem ninguém se
/// lembrar. É o gémeo exacto do `every_hierarchy_row_menu_entry_dispatches_something`, e existe
/// pela mesma doença medida: um item pintado e morto lê-se, do lado do artista, como um app
/// partido.
///
/// ⛔⛔ **O ORÁCULO DESTE GATE FOI ALARGADO EM 2026-09-02, e a 1.ª versão mandava a cura ERRADA.**
/// Ela perguntava *«empurrou para o barramento?»* — o único destino que existia quando foi escrita
/// —, e por isso acusou de mortos os dois itens de relação (D9), que estão **vivos e correctos**:
/// eles mudam a VISTA deste painel e não têm nada a dizer ao mundo. A mensagem chegava a nomear o
/// ficheiro do shell onde «drenar a acção». *Um gate que presume o destino de um efeito acusa de
/// morto quem tem outro — e a mensagem dele manda alguém construir a doença.*
/// ⇒ a pergunta passa a ser *«alguma coisa mudou?»*: o barramento **ou** o estado do painel.
///
/// ⚠️ A comparação é por `{:?}` de propósito, e não por um campo escolhido à mão: assim um campo
/// de vista **novo** entra neste oráculo sozinho, que é a mesma razão de a fonte ser a tabela.
///
/// **Mutação que deve sangrar:** apagar um braço do `card_verb_of` **ou** do `relation_of` no
/// `event.rs`.
#[test]
fn every_asset_card_menu_entry_dispatches_something() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_editor_core::screens::hero::menu_rows::menu_rows;

    let rows = menu_rows(ContextMenuKind::AssetCard {
        cell: ids::asset_cell_id(0),
    });
    assert!(
        !rows.is_empty(),
        "a tabela do menu do cartão está vazia — este gate mediria nada"
    );

    let mut dead: Vec<&str> = Vec::new();
    for (id, label, _) in rows {
        let (mut host, mut st) = open_host();
        ph2d_panel_asset_browser::state::probe_set_painted(vec![AssetRef::Component {
            stable_id: 77,
        }]);
        stage_card_menu(&mut host, ids::asset_cell_id(0));
        let before = format!("{st:?}");
        let out = host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(*id));
        let touched_the_world = !host.bus().is_empty();
        let touched_the_view = format!("{st:?}") != before;
        if out != EventOutcome::Consumed || !(touched_the_world || touched_the_view) {
            dead.push(label);
        }
    }
    assert!(
        dead.is_empty(),
        "linhas do menu do cartão que são PINTADAS e não chegam a efeito nenhum: {dead:?}.\n\
         Um item que toca no MUNDO liga-se no `card_verb_of` de \
         `crates/ph2d-panel-asset-browser/src/event.rs` e drena em \
         `shells/desktop/src/asset_card_verbs.rs`; um que só muda a VISTA da grade liga-se no \
         `relation_of` do mesmo ficheiro e escreve no `AssetBrowserState`."
    );
}

/// ⚠️ **Uma IMAGEM também despacha os três** — a tabela é plana, e quem recusa (em voz alta) é o
/// shell. ⛔ Um painel que filtrasse aqui devolveria o silêncio que a tabela plana existe para
/// evitar, e este gate é o que impede alguém de «optimizar» isso.
#[test]
fn an_image_card_dispatches_every_verb_too_because_the_shell_is_who_refuses() {
    use ph2d_editor_core::action_bus::AssetCardAction;
    use ph2d_editor_core::interaction::drag_payload::DragPayload;

    for verb in [
        // ⭐ **Editar entrou em 2026-09-05** e o `match` abaixo é EXAUSTIVO de propósito: um verbo
        // novo que ninguém pusesse nesta lista deixaria de ser medido em silêncio, e o compilador
        // é quem o impede.
        AssetCardAction::EditPrefab,
        AssetCardAction::Instantiate,
        AssetCardAction::SelectUsers,
        AssetCardAction::RemoveFromLibrary,
    ] {
        let id = match verb {
            AssetCardAction::EditPrefab => ph2d_editor_core::ids::CTX_MENU_ASSET_EDIT,
            AssetCardAction::Instantiate => ph2d_editor_core::ids::CTX_MENU_ASSET_INSTANTIATE,
            AssetCardAction::SelectUsers => ph2d_editor_core::ids::CTX_MENU_ASSET_SELECT_USERS,
            AssetCardAction::RemoveFromLibrary => ph2d_editor_core::ids::CTX_MENU_ASSET_REMOVE,
        };
        let (mut host, mut st) = open_host();
        ph2d_panel_asset_browser::state::probe_set_painted(vec![AssetRef::Texture {
            asset: [9; 32],
        }]);
        stage_card_menu(&mut host, ids::asset_cell_id(0));
        let out = host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(id));
        assert_eq!(out, EventOutcome::Consumed, "{verb:?} morto numa imagem");
        let drained: Vec<_> = host.bus_mut().drain().collect();
        assert_eq!(
            drained,
            vec![EditorAction::AssetCardVerb {
                asset: DragPayload::Image { asset: [9; 32] },
                verb,
            }],
            "a imagem tem de chegar ao shell com o endereço dela"
        );
    }
}

/// ⚠️ **O menu que já não tem sujeito não age.** O menu abre no `Down` e é despachado num `Click`
/// posterior; se a grade mudou de conteúdo no meio, a célula já não desenha asset nenhum — e agir
/// sobre a célula seguinte seria apagar o prefab errado.
///
/// **Mutação que deve sangrar:** trocar o `None => Ignored` por um `unwrap_or` de índice 0.
#[test]
fn a_card_menu_whose_cell_no_longer_paints_anything_does_nothing() {
    let (mut host, mut st) = open_host();
    ph2d_panel_asset_browser::state::probe_set_painted(Vec::new());
    stage_card_menu(&mut host, ids::asset_cell_id(0));
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_ASSET_REMOVE),
    );
    assert_eq!(out, EventOutcome::Ignored);
    assert_eq!(host.bus().len(), 0);
}

// ── ⭐⭐ O MENU DA LINHA DE CATÁLOGO (etapa D) ───────────────────────────────────────────────────

/// O gémeo do [`stage_card_menu`] para uma linha da coluna.
fn stage_catalog_menu(host: &mut MockPanelHost, row: ph2d_a11y::NodeId) {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::CatalogRow { row },
    });
    host.store_mut().close_context_menu();
}

/// Uma taxonomia com um catálogo, publicada, e a coluna a dizer que pintou as três linhas.
fn stage_one_catalog() -> ph2d_asset_index::CatalogId {
    use ph2d_panel_asset_browser::CatalogPick;
    let mut tree = ph2d_asset_index::CatalogTree::default();
    let id = tree.create("Heroes");
    ph2d_panel_asset_browser::set_current_catalogs(tree);
    ph2d_panel_asset_browser::state::probe_set_painted_rows(vec![
        CatalogPick::All,
        CatalogPick::Unassigned,
        CatalogPick::One(id),
    ]);
    id
}

/// ⭐⭐ **Toda linha do menu de um catálogo FAZ alguma coisa.**
///
/// ⚠️ **«Fazer» aqui não é «pôr no barramento»** — o *Rename…* abre um campo, que é estado do
/// painel, e um gate que exigisse uma acção declararia morto o item mais vivo dos dois. A pergunta
/// certa é *o clique deixou o mundo diferente?*, e as duas respostas legítimas são o barramento e
/// o campo aberto.
///
/// ⚠️ A fonte é a TABELA que o overlay pinta, como no menu do cartão: um item novo entra neste
/// gate no dia em que é pintado.
///
/// **Mutação que deve sangrar:** apagar o braço do `CTX_MENU_CATALOG_RENAME` no `event.rs`.
#[test]
fn every_catalog_row_menu_entry_does_something() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_editor_core::screens::hero::menu_rows::menu_rows;

    let rows = menu_rows(ContextMenuKind::CatalogRow {
        row: ids::catalog_row_id(2),
    });
    assert!(
        !rows.is_empty(),
        "a tabela do menu do catálogo está vazia — este gate mediria nada"
    );

    let mut dead: Vec<&str> = Vec::new();
    for (id, label, _) in rows {
        let (mut host, mut st) = open_host();
        stage_one_catalog();
        stage_catalog_menu(&mut host, ids::catalog_row_id(2));
        let out = host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(*id));
        if out != EventOutcome::Consumed || (host.bus().is_empty() && st.renaming.is_none()) {
            dead.push(label);
        }
    }
    assert!(
        dead.is_empty(),
        "linhas do menu do catálogo que são PINTADAS e não fazem nada: {dead:?}.\n\
         Ligue cada uma no braço `CTX_MENU_CATALOG_*` de \
         `crates/ph2d-panel-asset-browser/src/event.rs`."
    );
}

/// ⛔ **`All` e `Unassigned` não são catálogos.** Elas são linhas fixas da coluna, e um *Delete*
/// sobre elas teria de apagar o quê? O braço desiste, e é isso que o mantém sem um caso especial
/// no dreno do shell.
///
/// **Mutação que deve sangrar:** aceitar qualquer `CatalogPick` no braço em vez de só o `One`.
#[test]
fn the_menu_over_a_fixed_row_does_nothing() {
    // ⚠️ **O controlo POSITIVO da fixtura** (auditoria de 2026-08-30): o oráculo deste gate é
    // «nada aconteceu», e sem esta linha ele ficaria verde se o `probe_set_painted_rows` não
    // tivesse efeito nenhum — *uma fixtura sem o fenómeno passa em qualquer lei.*
    stage_one_catalog();
    assert_eq!(
        ph2d_panel_asset_browser::catalog_row_pick(ids::catalog_row_id(0)),
        Some(ph2d_panel_asset_browser::CatalogPick::All),
        "o censo do quadro não chegou — este gate mediria nada"
    );
    for row in [0usize, 1] {
        let (mut host, mut st) = open_host();
        stage_one_catalog();
        stage_catalog_menu(&mut host, ids::catalog_row_id(row));
        let out = host.apply_panel_event::<AssetBrowserPanel>(
            &mut st,
            WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_CATALOG_DELETE),
        );
        assert_eq!(out, EventOutcome::Ignored, "a linha fixa {row} agiu");
        assert_eq!(host.bus().len(), 0);
        assert!(st.renaming.is_none());
    }
}

/// ⭐⭐ **Apagar o catálogo ESCOLHIDO devolve a grade a `All`.**
///
/// ⚠️ Sem isto a grade continuaria a filtrar por uma gaveta que já não existe: zero cartões, e
/// nada na tela a explicar porquê. *Um filtro cujo sujeito morreu não é um filtro vazio, é um
/// painel partido.*
///
/// **Mutação que deve sangrar:** apagar o `state.pick = CatalogPick::All` do braço.
#[test]
fn deleting_the_chosen_catalog_returns_the_grid_to_all() {
    use ph2d_editor_core::action_bus::CatalogVerb;
    use ph2d_panel_asset_browser::CatalogPick;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    st.pick = CatalogPick::One(id);
    stage_catalog_menu(&mut host, ids::catalog_row_id(2));
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_CATALOG_DELETE),
    );
    assert_eq!(out, EventOutcome::Consumed);
    assert_eq!(
        st.pick,
        CatalogPick::All,
        "a grade ficou a filtrar por um fantasma"
    );
    let drained: Vec<_> = host.bus_mut().drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::AssetCatalogVerb(CatalogVerb::Delete {
            id: id.0
        })]
    );
}

/// ⚠️ **O `Rename…` NÃO manda nada — ele abre o campo.** O nome só atravessa o barramento no
/// `Submit`/`Blur`, e é aí que ele é comparado com o actual.
///
/// **Mutação que deve sangrar:** fazer o braço do rename empurrar um `CatalogVerb::Rename` logo.
#[test]
fn rename_opens_the_field_and_the_name_only_travels_on_submit() {
    use ph2d_editor_core::action_bus::CatalogVerb;
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::widget::TextInputState;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    stage_catalog_menu(&mut host, ids::catalog_row_id(2));
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_CATALOG_RENAME),
    );
    assert_eq!(st.renaming.map(|r| r.id), Some(id), "o campo não abriu");
    assert_eq!(
        host.bus().len(),
        0,
        "o rename mandou o nome antes de haver nome"
    );

    // O que o campo teria depois de o artista escrever.
    host.store_mut().register(
        ids::ASSET_CATALOG_RENAME,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: "Villains".into(),
            caret: 8,
            selection_anchor: None,
        },
    );
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Submit(ids::ASSET_CATALOG_RENAME),
    );
    assert_eq!(out, EventOutcome::Consumed);
    let drained: Vec<_> = host.bus_mut().drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::AssetCatalogVerb(CatalogVerb::Rename {
            id: id.0,
            name: "Villains".into(),
        })]
    );
    assert!(
        st.renaming.is_none(),
        "o campo ficou aberto depois de gravar"
    );

    // ⚠️ **O par Enter→(Submit, Blur) é idempotente** — o segundo evento não acha campo nenhum.
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Blur(ids::ASSET_CATALOG_RENAME),
    );
    assert_eq!(host.bus().len(), 0, "o Blur mandou o nome uma segunda vez");
}

/// ⚠️ **Um nome IGUAL ao actual não levanta acção** — ela marcaria o projecto sujo por nada, e o
/// dreno faria um `rename` que não muda um byte.
///
/// **Mutação que deve sangrar:** apagar o `if text == current` do `commit`.
#[test]
fn renaming_to_the_same_name_dispatches_nothing() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::widget::TextInputState;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    stage_catalog_menu(&mut host, ids::catalog_row_id(2));
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_CATALOG_RENAME),
    );
    assert_eq!(st.renaming.map(|r| r.id), Some(id));
    host.store_mut().register(
        ids::ASSET_CATALOG_RENAME,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: "  Heroes  ".into(),
            caret: 0,
            selection_anchor: None,
        },
    );
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Submit(ids::ASSET_CATALOG_RENAME),
    );
    assert_eq!(
        host.bus().len(),
        0,
        "renomear para o mesmo nome (com espaços) sujou o projecto"
    );

    // ⚠️ **A metade que separa as DUAS causas do `None`** (auditoria de 2026-08-30): o `commit`
    // devolve `None` tanto para «nome igual» como para «catálogo não encontrado», e sem este
    // controlo a cláusula `text == current` estava a ser creditada por uma ausência. Mesma
    // fixtura, texto diferente ⇒ tem de despachar.
    stage_catalog_menu(&mut host, ids::catalog_row_id(2));
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_CATALOG_RENAME),
    );
    host.store_mut().register(
        ids::ASSET_CATALOG_RENAME,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: "Villains".into(),
            caret: 0,
            selection_anchor: None,
        },
    );
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Submit(ids::ASSET_CATALOG_RENAME),
    );
    assert_eq!(
        host.bus().len(),
        1,
        "um nome DIFERENTE não despachou — o `None` do commit tem outra causa"
    );
}

/// ⚠️ **O Esc abandona sem gravar.**
#[test]
fn escape_abandons_the_rename() {
    let (mut host, mut st) = open_host();
    stage_one_catalog();
    stage_catalog_menu(&mut host, ids::catalog_row_id(2));
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_CATALOG_RENAME),
    );
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Cancel(ids::ASSET_CATALOG_RENAME),
    );
    assert_eq!(out, EventOutcome::Consumed);
    assert!(st.renaming.is_none());
    assert_eq!(host.bus().len(), 0);
    // ⚠️ **O despachante emite o PAR `Cancel` + `Blur`** (`dispatch/key.rs`), e o gate tem de o
    // reproduzir: sem isto ele não prova que o segundo do par não grava o que o Esc abandonou.
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Blur(ids::ASSET_CATALOG_RENAME),
    );
    assert_eq!(
        host.bus().len(),
        0,
        "o Blur que acompanha o Esc gravou o nome abandonado"
    );
}

/// ⭐⭐ **O campo aberto é PINTADO e REGISTADO** — a metade positiva, sem a qual a seguinte
/// mediria nada.
///
/// ⚠️ Ela lê o que o PAINT registou, que é o que o artista pode agarrar. Um campo que existe só no
/// estado é um campo que ninguém alcança.
#[test]
fn the_open_rename_field_is_registered_by_the_paint() {
    use ph2d_editor_core::zones::Rect;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    st.renaming = Some(ph2d_panel_asset_browser::state::CatalogRename { id, opened: false });
    let rects = host.paint::<AssetBrowserPanel>(&mut st, Rect::new(0.0, 0.0, 1600.0, 900.0));
    assert!(
        rects.iter().any(|(i, _)| *i == ids::ASSET_CATALOG_RENAME),
        "o campo de renomear não foi registado — ele existe no estado e não sob o dedo"
    );
    assert_eq!(
        host.store().focus_id(),
        Some(ids::ASSET_CATALOG_RENAME),
        "o campo abriu sem tomar o foco — as teclas iriam para outro sítio"
    );
    // ⚠️ **E o ELO semente→rótulo**, que a auditoria de 2026-08-30 achou sem gate: a `seed_state`
    // tem gate em isolamento, e nada afirmava que o `paint` lhe passa o NOME DO CATÁLOGO. A
    // mutação `seed_state("")` sobrevivia a tudo — e é exactamente a metade que o smoke apanhou
    // em produto.
    let Some(ph2d_editor_core::interaction::InteractiveState::TextInput {
        text,
        selection_anchor,
        ..
    }) = host.store().get(ids::ASSET_CATALOG_RENAME)
    else {
        panic!("o campo não é um campo de texto no store");
    };
    assert_eq!(text, "Heroes", "o campo abriu com o nome errado");
    assert_eq!(*selection_anchor, Some(0), "o nome não abriu seleccionado");
}

/// ⛔⛔ **Fechar a coluna LEVA o campo com ela.**
///
/// ⚠️ O `paint` da coluna sai cedo quando a largura é zero (o botão *só-grade*, ou um painel
/// estreitado até o cartão já não caber). Sem a limpeza simétrica o campo deixa de ser pintado e
/// registado, **mas o `WidgetStore` continua com o foco nele** — e a partir daí escrever no app não
/// faz nada em lado nenhum. *Quem esconde uma região limpa o que ela publicou.*
///
/// **Mutação que deve sangrar:** apagar o `catalog_rename::abandon` do ramo `w <= 0.0`.
#[test]
fn collapsing_the_column_takes_the_rename_field_with_it() {
    use ph2d_editor_core::zones::Rect;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    st.renaming = Some(ph2d_panel_asset_browser::state::CatalogRename { id, opened: false });
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert_eq!(host.store().focus_id(), Some(ids::ASSET_CATALOG_RENAME));

    // O botão *só-grade*.
    st.show_catalogs = false;
    let rects = host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert!(
        !rects.iter().any(|(i, _)| *i == ids::ASSET_CATALOG_RENAME),
        "a coluna fechou e o campo continuou registado"
    );
    assert!(
        st.renaming.is_none(),
        "o campo ficou aberto com a coluna fechada"
    );
    assert_eq!(
        host.store().focus_id(),
        None,
        "o campo invisível continuou a comer as teclas"
    );
}

/// ⚠️ **O `abandon` só larga o foco se ele for NOSSO** — pisar o foco de outro widget seria trocar
/// um defeito por outro.
///
/// ⛔⛔ **A 1.ª versão deste «controlo» não controlava nada** (auditoria de 2026-08-30): ela punha
/// `renaming = None`, e aí o `.take()` curto-circuita **antes** de a cláusula do foco ser avaliada
/// — a mutação que a apaga sobrevivia. A fixtura tem de ter o campo **ABERTO** e o foco **noutro
/// widget** ao mesmo tempo; é o único estado em que a cláusula decide alguma coisa.
///
/// **Mutação que deve sangrar:** `if state.renaming.take().is_some() { store.set_focus(None); }`.
#[test]
fn abandoning_the_rename_does_not_steal_someone_elses_focus() {
    use ph2d_editor_core::zones::Rect;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    // O campo está ABERTO e o foco é de outro widget — o estado em que a cláusula decide.
    st.renaming = Some(ph2d_panel_asset_browser::CatalogRename { id, opened: true });
    host.store_mut().set_focus(Some(ids::ASSET_SEARCH));
    st.show_catalogs = false;
    host.paint::<AssetBrowserPanel>(&mut st, Rect::new(0.0, 0.0, 1600.0, 900.0));
    assert!(st.renaming.is_none(), "o campo tem de fechar na mesma");
    assert_eq!(
        host.store().focus_id(),
        Some(ids::ASSET_SEARCH),
        "fechar a coluna roubou o foco de um campo alheio"
    );
}

/// ⛔⛔ **A TERCEIRA porta: o catálogo desapareceu debaixo do campo.**
///
/// ⚠️ As outras duas (coluna colapsada · painel fechado) já chamavam o `abandon`; este ramo
/// escrevia `renaming = None` à mão e ficava a meio da lei — o campo deixava de ser pintado e o
/// foco ficava preso nele, o que trava **todos os atalhos do app** (`text_entry_focused`).
///
/// **Mutação que deve sangrar:** trocar o `abandon` daquele `else` por `state.renaming = None`.
#[test]
fn a_rename_whose_catalog_vanished_releases_the_focus_too() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_panel_asset_browser::CatalogPick;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    st.renaming = Some(ph2d_panel_asset_browser::CatalogRename { id, opened: false });
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert_eq!(host.store().focus_id(), Some(ids::ASSET_CATALOG_RENAME));

    // A taxonomia perde o catálogo — apagado aqui, ou por um undo.
    ph2d_panel_asset_browser::set_current_catalogs(ph2d_asset_index::CatalogTree::default());
    ph2d_panel_asset_browser::state::probe_set_painted_rows(vec![
        CatalogPick::All,
        CatalogPick::Unassigned,
    ]);
    host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert!(st.renaming.is_none(), "o campo sobreviveu ao catálogo");
    assert_eq!(
        host.store().focus_id(),
        None,
        "o catálogo morreu e o foco ficou preso no campo — os atalhos do app ficam mudos"
    );
}

/// ⛔⛔ **Fechar o PAINEL leva o campo com ele** — a segunda porta pela qual a coluna desaparece.
///
/// ⚠️ Ela usa o `paint_hidden` do testkit, que existe precisamente porque *nenhum gate deste repo
/// conseguia exercitar o ramo escondido de um painel* — e é lá que ele larga os rects velhos, os
/// gestos a meio e as flags publicadas. Fechar o painel com o campo aberto deixava-o focado e
/// invisível **a comer as teclas do app inteiro**, não só as deste painel.
///
/// **Mutação que deve sangrar:** apagar o `catalog_rename::abandon` do ramo `!panel_visible`.
#[test]
fn closing_the_panel_takes_the_rename_field_with_it() {
    use ph2d_editor_core::zones::Rect;

    let (mut host, mut st) = open_host();
    let id = stage_one_catalog();
    st.renaming = Some(ph2d_panel_asset_browser::state::CatalogRename { id, opened: false });
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert_eq!(host.store().focus_id(), Some(ids::ASSET_CATALOG_RENAME));

    host.paint_hidden::<AssetBrowserPanel>(&mut st, viewport);
    assert!(
        st.renaming.is_none(),
        "o campo sobreviveu ao painel fechado"
    );
    assert_eq!(
        host.store().focus_id(),
        None,
        "o campo de um painel fechado continuou a comer as teclas"
    );
}

/// ⛔⛔ **O polegar da barra da coluna é agarrável ONDE ELE ESTÁ DESENHADO.**
///
/// ⚠️ O defeito medido pela auditoria de 2026-08-30: o pintor recebia o rect da **coluna** e a
/// geometria do hit recebia o rect da **lista**, 30 px abaixo (o cabeçalho do `+ Catalog`) — o
/// polegar desenhava-se num sítio e só respondia noutro. A cura é o pintor **devolver** o rect que
/// desenhou, e o chamador registar esse.
///
/// ⚠️ **A régua é uma PROPRIEDADE, não uma re-derivação**: o polegar tem de cair dentro do corpo
/// da lista (abaixo do cabeçalho), que é o que um `col` passado ao pintor viola. Re-calcular aqui
/// o `thumb_rect` seria um oráculo feito com a função sob teste.
///
/// **Mutação que deve sangrar:** passar `col` ao `paint_scrollbar` em vez de `list_rect`.
#[test]
fn the_catalog_scrollbar_thumb_is_grabbable_where_it_is_drawn() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_panel_asset_browser::CatalogPick;

    // Catálogos que cheguem para a lista transbordar o corpo.
    let mut tree = ph2d_asset_index::CatalogTree::default();
    let mut rows = vec![CatalogPick::All, CatalogPick::Unassigned];
    for i in 0..60 {
        rows.push(CatalogPick::One(tree.create(&format!("C{i}"))));
    }
    ph2d_panel_asset_browser::set_current_catalogs(tree);
    ph2d_panel_asset_browser::state::probe_set_painted_rows(rows);

    let (mut host, mut st) = open_host();
    let rects = host.paint::<AssetBrowserPanel>(&mut st, Rect::new(0.0, 0.0, 1600.0, 900.0));
    let thumb = rects
        .iter()
        .find(|(i, _)| *i == ph2d_editor_core::widget::ASSET_CATALOG_SCROLLBAR_ID)
        .map(|(_, r)| *r)
        .expect("a barra da coluna não foi registada com 60 catálogos");
    let first_row = rects
        .iter()
        .find(|(i, _)| *i == ids::catalog_row_id(0))
        .map(|(_, r)| *r)
        .expect("a coluna não pintou linha nenhuma");
    assert!(
        thumb.y >= first_row.y - 0.5,
        "o polegar está registado ACIMA da 1.ª linha da lista (y={}, linha={}) — ele foi desenhado \
         a partir da coluna e agarrado a partir do corpo",
        thumb.y,
        first_row.y
    );
}
