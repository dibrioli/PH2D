//! Registo dos widgets FIXOS (1× na instalação do painel).
//!
//! ⚠️ **As células NÃO se registam aqui**, e isso é deliberado: quantos cartões existem só se sabe
//! em runtime, e o paint regista o que pinta (é o mesmo idioma da tira do Flip e das linhas do
//! painel de camadas). ⛔ Um `for i in 0..MAX_ASSET_CELLS` aqui registaria 512 ids de que 20 são
//! pintados — os outros 492 seriam ids **órfãos**, que é a terceira espécie do §5.0 e cuja cura é
//! oposta à do knob morto.

use crate::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub(crate) fn populate(store: &mut WidgetStore) {
    // ⛔⛔ **A faixa de arrasto e a alça NÃO são botões, e a AUDITORIA apanhou-me a registá-las
    // como tal.** O despacho de um painel flutuante não passa pelo `Click`: ele lê
    // `InteractiveState::BlenderHit { parent, kind }` no `pointer_down`, e é daí que saem o
    // `begin_blender_drag` e o `begin_panel_resize_bl`. Registadas como `Button` elas ficavam
    // **pintadas, hit-indexadas e mortas** — o painel abria e **não se podia mover nem
    // redimensionar**, com toda a suíte verde. É a terceira pergunta do §5.0 outra vez: *o leitor
    // DECIDE?*
    store.register(
        ids::ASSET_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::ASSET_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::ASSET_RESIZE_HANDLE_BL,
        InteractiveState::BlenderHit {
            parent: ids::ASSET_PANEL,
            kind: BlenderHitKind::ResizeHandleBl,
        },
    );
    // ⚠️ **O pill do topo NÃO entra aqui.** Ele já está registado pela `topbar::populate` (é um dos
    // três chips do grupo da direita), e o que faltava nunca foi o registo: era o DESPACHO, que
    // este painel passa a ter.
    store.register(
        ids::ASSET_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // ⛔⛔ **O `✕` da faixa de relação entra AQUI, e o censo `hit_indexed_ids_are_registered` foi
    // quem o apanhou** (2026-09-02): ele era pintado e hit-indexado pelo `paint_related` e **não
    // tinha `InteractiveState`** ⇒ `is_focusable` falso, o `Down` não arma, e o `Click` **nunca
    // nasce**. Ele estava morto sob o dedo com os meus cinco gates verdes — porque um `Click`
    // sintético num teste não passa pelo store.
    //
    // ⚠️ **É FIXO ainda que a faixa seja condicional.** O registo é *«este id existe e é um
    // botão»*, não *«ele está no ecrã agora»*; quem decide se ele é alcançável é o `HitIndex`, que
    // o paint só enche quando a faixa existe. O gémeo — registar no paint — reintroduziria o
    // `register` a substituir o `Pressed` do quadro anterior, que é o defeito que os cartões já
    // pagaram.
    store.register(
        ids::ASSET_RELATED_CLEAR,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    for id in ids::ASSET_KIND.iter().take(ids::ASSET_KIND_FILTERS) {
        store.register(
            *id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for id in ids::ASSET_SORT.iter().take(ids::ASSET_SORT_MODES) {
        store.register(
            *id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.register(
        ids::ASSET_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.register(
        ids::ASSET_SIZE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: crate::state::AssetBrowserState::default().size_slider_value(),
            orientation: SliderOrientation::Horizontal,
        },
    );
    // ⛔⛔ **O polegar da barra REGISTA-SE como `Plain`, e a auditoria apanhou-me a omiti-lo.**
    // Ele não tem estado próprio (o par visual sai do `scrollbar_visual_for`), mas sem uma entrada
    // no store o `is_focusable` do despachante é **falso** e o `pointer_down` nunca semeia o
    // arrasto: a barra desenhava, acendia, e **não se podia agarrar**.
    store.register(
        ph2d_editor_core::widget::ASSET_BROWSER_SCROLLBAR_ID,
        InteractiveState::Plain,
    );
    // ⭐⭐ A coluna de catálogos (wave A3): o polegar dela e os dois botões. ⛔ Sem o registo o
    // `is_focusable` é falso, o `pointer_down` não semeia e o polegar fica **inagarrável** — o
    // defeito nº 2 da auditoria da etapa A, e ele não se anuncia.
    store.register(
        ph2d_editor_core::widget::ASSET_CATALOG_SCROLLBAR_ID,
        InteractiveState::Plain,
    );
    for id in [ids::ASSET_CATALOG_TOGGLE, ids::ASSET_CATALOG_NEW] {
        store.register(
            id,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
    }
    // ⭐⭐ **O campo de renomear nasce aqui, VAZIO e sem foco** — e o `paint` do
    // [`crate::paint_catalog_rename`] semeia-o com o nome quando ele abre.
    //
    // ⛔ Sem esta entrada o `is_focusable` do despachante é **falso**, e um campo que se pinta e se
    // hit-testa sem ser focável está morto sob o dedo — a classe dos *vector pills*. ⚠️ A auditoria
    // de 2026-08-30 mostrou que **o gate de paridade nem lia o ficheiro dele**: o
    // `read_paint_sources` só varre `paint*`/`sections/`, e o módulo chamava-se `catalog_rename.rs`
    // ⇒ ele foi **renomeado para entrar no alcance da regra**, ⛔ nunca a regra alargada para o
    // apanhar (alargá-la arrastaria quatro ids de painéis de OUTRAS linhas, e essa decisão é dos
    // donos deles — o doc do gate já o diz).
    store.register(
        ids::ASSET_CATALOG_RENAME,
        InteractiveState::TextInput {
            state: ph2d_editor_core::widget::TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
}
