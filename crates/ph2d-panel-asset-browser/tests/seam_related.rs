//! ⭐⭐ **A COSTURA DAS DUAS PERGUNTAS DE RELAÇÃO** (plano 07 D9) — *o que este asset usa* e *o que
//! o usa*, do item de menu até a grade mudar.
//!
//! ⚠️ Ficheiro próprio, e não mais um bloco do `seam.rs`: aquele já mede a etapa inteira e é o
//! sítio onde a próxima wave vai colar o dela. *Um assunto por endereço.*
//!
//! ⛔⛔ **O que estes gates existem para impedir tem nome, e este painel já o pagou:** um item de
//! menu pintado, registado e **morto sob o dedo**. Aqui há uma armadilha a mais — estes dois itens
//! saem do MESMO menu dos três verbos e **não vão ao barramento** —, e por isso o gate irmão
//! `every_asset_card_menu_entry_dispatches_something` teve de alargar a pergunta de *«empurrou
//! para o barramento?»* para *«mudou alguma coisa?»*.

use ph2d_asset_index::{AssetEntry, AssetIndex, AssetRef, Relation};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_asset_browser::state::AssetBrowserState;
use ph2d_panel_asset_browser::{AssetBrowserPanel, PANEL_ID, ids};
use ph2d_ui_testkit::MockPanelHost;

const HOUSE: AssetRef = AssetRef::Component { stable_id: 10 };
const BARK: AssetRef = AssetRef::Texture { asset: [1; 32] };

fn open_host() -> (MockPanelHost, AssetBrowserState) {
    let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    host.set_panel_visible(PANEL_ID, true);
    (host, AssetBrowserState::default())
}

/// A biblioteca do quadro: uma receita que desenha uma textura, e uma textura solta.
fn publish_library() {
    let mut ix = AssetIndex::new();
    ix.push(AssetEntry::new(BARK, "bark"));
    ix.push(AssetEntry::new(
        AssetRef::Texture { asset: [3; 32] },
        "unrelated",
    ));
    let mut house = AssetEntry::new(HOUSE, "house");
    house.deps = vec![BARK];
    ix.push(house);
    ph2d_panel_asset_browser::set_current_index(ix);
}

/// Os NOMES que a grade pintou neste quadro, em ordem de cartão.
///
/// ⚠️ Ela lê o índice publicado para converter endereço em nome — a mesma travessia que o cartão
/// faz —, e por isso mede o que está no ecrã, não o que o índice contém.
fn painted_names() -> Vec<String> {
    let mut out = Vec::new();
    for i in 0..8 {
        let Some(key) = ph2d_panel_asset_browser::probe_painted_at(i) else {
            break;
        };
        out.push(match key {
            HOUSE => "house".to_string(),
            BARK => "bark".to_string(),
            _ => "unrelated".to_string(),
        });
    }
    out.sort();
    out
}

/// Encena um menu de cartão já fechado, como o `pointer_down` faz.
fn stage_card_menu(host: &mut MockPanelHost, cell: ph2d_a11y::NodeId) {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::AssetCard { cell },
    });
    host.store_mut().close_context_menu();
}

/// ⭐⭐ **Os dois itens escrevem a pergunta no estado, com a ÂNCORA certa.**
///
/// **Mutação que deve sangrar:** apagar um braço do `relation_of` no `event.rs`.
#[test]
fn each_relation_item_writes_its_own_question_with_the_right_anchor() {
    for (id, want) in [
        (ph2d_editor_core::ids::CTX_MENU_ASSET_USES, Relation::Uses),
        (
            ph2d_editor_core::ids::CTX_MENU_ASSET_USED_BY,
            Relation::UsedBy,
        ),
    ] {
        let (mut host, mut st) = open_host();
        ph2d_panel_asset_browser::state::probe_set_painted(vec![HOUSE]);
        stage_card_menu(&mut host, ids::asset_cell_id(0));
        let out = host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(id));
        assert_eq!(out, EventOutcome::Consumed, "{want:?} morto sob o dedo");
        assert_eq!(
            st.related,
            Some((HOUSE, want)),
            "{want:?} nao escreveu a pergunta com a ancora do cartao"
        );
    }
}

/// ⭐⭐⭐ **E a pergunta CHEGA À GRADE — medida no que o PAINT de facto desenhou.**
///
/// ⛔⛔ **A 1.ª versão deste gate media outra coisa, e dizia esta frase.** Ela montava uma `Query`
/// à mão e passava-a ao `probe_query` — o que prova que o ÍNDICE filtra (a `index_law` já o prova)
/// e **não** que o painel liga o `state.related` à consulta que ele constrói. A costura real é
/// `AssetBrowserState` → a `Query` de dentro do `paint_grid`, e uma `Query` escrita no teste passa
/// por cima dela. *Um gate que fabrica o alcance mede o seu próprio arnês.*
///
/// ⇒ a régua é o `payload_at`, que devolve o que a grade **pintou** neste quadro.
///
/// **Mutação que deve sangrar:** apagar `related: state.related` da `Query` do `paint_grid`.
#[test]
fn the_question_reaches_what_the_grid_actually_paints() {
    use ph2d_editor_core::interaction::drag_payload::DragPayload;
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);

    // Sem filtro: a grade desenha os três.
    publish_library();
    let (mut host, mut st) = open_host();
    host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert_eq!(
        painted_names(),
        vec!["bark", "house", "unrelated"],
        "a fixtura tem de trazer os tres"
    );

    // *O que a casa usa* ⇒ só a casca.
    publish_library();
    let (mut host, mut st) = open_host();
    st.related = Some((HOUSE, Relation::Uses));
    host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert_eq!(painted_names(), vec!["bark"], "a casa usa a casca, e so'");
    assert_eq!(
        ph2d_panel_asset_browser::payload_at(0),
        Some(DragPayload::Image { asset: [1; 32] }),
        "o cartao que sobra tem de ser arrastavel como a casca"
    );

    // *O que usa a casca* ⇒ só a casa.
    publish_library();
    let (mut host, mut st) = open_host();
    st.related = Some((BARK, Relation::UsedBy));
    host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert_eq!(painted_names(), vec!["house"], "so' a casa usa a casca");
}

/// ⭐⭐ **A faixa é PINTADA e o `✕` fica sob o dedo** — e só quando há filtro.
///
/// ⛔ É a metade que separa *«o estado mudou»* de *«o artista tem como voltar»*. Um filtro ligado
/// por menu, sem interruptor pintado, é um beco: nada no ecrã o desliga.
#[test]
fn the_band_registers_its_clear_button_only_while_the_filter_is_on() {
    publish_library();
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);

    let (mut host, mut st) = open_host();
    let rects = host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert!(
        !rects.iter().any(|(i, _)| *i == ids::ASSET_RELATED_CLEAR),
        "sem filtro a faixa nao existe, logo o `x` dela nao pode estar registado"
    );

    let (mut host, mut st) = open_host();
    st.related = Some((HOUSE, Relation::Uses));
    let rects = host.paint::<AssetBrowserPanel>(&mut st, viewport);
    assert!(
        rects.iter().any(|(i, _)| *i == ids::ASSET_RELATED_CLEAR),
        "com filtro ligado o `x` da faixa tem de estar sob o dedo"
    );

    // ⛔⛔ **E a OUTRA metade, que este gate não tinha e o censo da workspace apanhou.** Um
    // rectângulo no `HitIndex` sem `InteractiveState` no store é `is_focusable == false`: o `Down`
    // não arma e o `Click` **nunca nasce**. O botão fica pintado, hit-indexado e morto — e os
    // gates de `apply_event` não o vêem, porque um `Click` sintético não passa pelo store.
    //
    // **Mutação que deve sangrar:** apagar o `register` do `ASSET_RELATED_CLEAR` no `populate.rs`.
    assert!(
        matches!(
            host.store().get(ids::ASSET_RELATED_CLEAR),
            Some(ph2d_editor_core::interaction::InteractiveState::Button { .. })
        ),
        "o `x` da faixa nao tem estado no store — ele e' hit-indexado e morto sob o dedo"
    );
}

/// ⭐ **E o `✕` LARGA o filtro.**
///
/// **Mutação que deve sangrar:** apagar o braço do `ASSET_RELATED_CLEAR` no `event.rs`.
#[test]
fn the_clear_button_drops_the_filter() {
    let (mut host, mut st) = open_host();
    st.related = Some((HOUSE, Relation::UsedBy));
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ids::ASSET_RELATED_CLEAR),
    );
    assert_eq!(out, EventOutcome::Consumed);
    assert_eq!(st.related, None, "o `x` nao largou o filtro");
}

/// ⛔ **Um menu sobre uma célula que a grade não pintou não ancora nada.** O menu abre no `Down` e
/// o `Click` é posterior; ancorar na célula seguinte responderia sobre o asset errado.
#[test]
fn a_menu_without_a_subject_sets_no_question() {
    let (mut host, mut st) = open_host();
    ph2d_panel_asset_browser::state::probe_set_painted(Vec::new());
    stage_card_menu(&mut host, ids::asset_cell_id(0));
    let out = host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_ASSET_USES),
    );
    assert_eq!(out, EventOutcome::Ignored);
    assert_eq!(st.related, None);
}
