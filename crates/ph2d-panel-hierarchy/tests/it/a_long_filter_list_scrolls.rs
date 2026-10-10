//! ⭐⭐ **A lista do filtro ▾ ROLA quando não cabe** (onda 4, item 9) — com a roda e o polegar
//! REAIS, pelo despacho do produto.
//!
//! ⚠️ A lista tem uma linha por componente PRESENTE na cena; numa cena que use o catálogo inteiro
//! ela passa da altura da janela, o clamp encolhe o popover e, sem rolagem, as linhas de baixo
//! ficavam desenhadas fora dele e nunca registadas — impossíveis de escolher. A porta é a mesma dos
//! outros painéis: `panel::paint_dropdown_list`, com o rect e as duas alturas publicados para o
//! `HIER_FILTER_DD` (é por eles que a roda e o arrasto o acham).

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentSet;
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::widget::DROPDOWN_SCROLLBAR_ID;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::ids::{HIER_FILTER_DD, hier_filter_opt};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const TUDO: NodeId = NodeId(200_001);
const POUCO: NodeId = NodeId(200_002);

fn linha(nome: &str, tem: impl IntoIterator<Item = u8>) -> HierarchyEntity {
    let mut e = ph2d_panel_hierarchy::scene_entry();
    e.name = nome.into();
    e.role = RowRole::Object;
    e.components = Vec::new();
    e.content = Vec::new();
    for b in tem {
        e.has.insert(b);
    }
    e
}

/// Uma cena com o catálogo inteiro (`muitos`) ou com um componente só.
fn host(muitos: bool) -> (MockPanelHost, HierarchyState, usize) {
    let bits: Vec<u8> = if muitos {
        ComponentSet::authored().map(|(b, _)| b).collect()
    } else {
        ComponentSet::authored().map(|(b, _)| b).take(1).collect()
    };
    let n = bits.len() + 1; // + «All»
    let (id, nome) = if muitos {
        (TUDO, "Tudo")
    } else {
        (POUCO, "Pouco")
    };
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    ph2d_panel_hierarchy::sync_from_hierarchy(
        host.store_mut(),
        &[id],
        BTreeMap::from([(id, linha(nome, bits))]),
    );
    (host, HierarchyState::default(), n)
}

fn aberto(host: &MockPanelHost) -> bool {
    matches!(
        host.store().get(HIER_FILTER_DD),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Abre o ▾ pelo clique real no chip e devolve a pintura com a lista aberta.
fn abre(host: &mut MockPanelHost, state: &mut HierarchyState) -> Vec<(NodeId, Rect)> {
    let painted = host.paint::<HierarchyPanel>(state, VIEWPORT);
    let (_, r) = painted
        .iter()
        .find(|(i, _)| *i == HIER_FILTER_DD)
        .copied()
        .expect("o ▾ do filtro não foi pintado");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<HierarchyPanel>(state, ev);
    }
    assert!(aberto(host), "o ▾ não abriu");
    host.paint::<HierarchyPanel>(state, VIEWPORT)
}

fn rect_of(painted: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    painted.iter().find(|(i, _)| *i == id).map(|(_, r)| *r)
}

fn popover(host: &MockPanelHost) -> Rect {
    let (dono, r) = host
        .store()
        .dropdown_popover()
        .expect("o rect da lista não foi publicado — a roda não a acha");
    assert_eq!(dono, HIER_FILTER_DD, "o popover publicado é de outro chip");
    r
}

/// ⭐⭐ A lista longa publica o alcance da rolagem, tem a barra, e só regista o que está DENTRO.
///
/// **Mutações que devem sangrar:** apagar o `set_panel_content_h` da publicação · registar o
/// `option_rect_in_scrolled` inteiro em vez da faixa recortada ao painel.
#[test]
fn a_lista_longa_publica_o_alcance_e_regista_so_o_visivel() {
    let (mut host, mut state, n) = host(true);
    let painted = abre(&mut host, &mut state);
    let panel = popover(&host);
    let content_h = host
        .store()
        .panel_content_h(HIER_FILTER_DD)
        .expect("a altura do CONTEÚDO da lista não foi publicada");
    let visible_h = host
        .store()
        .panel_visible_h(HIER_FILTER_DD)
        .expect("a altura VISÍVEL da lista não foi publicada");
    assert!(
        content_h > visible_h,
        "a fixtura não produz o fenómeno: {n} linhas cabem ({content_h:.1} em {visible_h:.1})"
    );
    assert!(
        rect_of(&painted, DROPDOWN_SCROLLBAR_ID).is_some_and(|r| r.h > 0.0),
        "a barra da lista longa não foi registada — o polegar não se agarraria"
    );
    let mut vistas = 0;
    for i in 0..n {
        let Some(r) = rect_of(&painted, hier_filter_opt(i)) else {
            continue;
        };
        vistas += 1;
        assert!(
            r.y >= panel.y - 0.5 && r.y + r.h <= panel.y + panel.h + 0.5, // LITERAL-PX-OK: meio pixel de tolerância entre duas medidas do mesmo rect
            "a linha {i} ({:.1}..{:.1}) foi registada fora da lista ({:.1}..{:.1})",
            r.y,
            r.y + r.h,
            panel.y,
            panel.y + panel.h
        );
    }
    assert!(
        vistas > 0 && vistas < n,
        "esperava-se uma FATIA da lista registada, viu {vistas} de {n}"
    );
}

/// ⭐⭐⭐ A RODA sobre a lista rola-a, e a última linha — abaixo da dobra — fica clicável e aplica o
/// filtro.
#[test]
fn a_roda_rola_a_lista_e_a_ultima_linha_aplica_o_filtro() {
    let (mut host, mut state, n) = host(true);
    let painted = abre(&mut host, &mut state);
    let ultima = hier_filter_opt(n - 1);
    assert!(
        rect_of(&painted, ultima).is_none(),
        "pré-condição: a última linha começa abaixo da dobra"
    );
    let panel = popover(&host);
    host.wheel_at(panel.x + panel.w * 0.5, panel.y + panel.h * 0.5, -100_000.0);
    assert!(
        host.store().panel_scroll(HIER_FILTER_DD) > 0.0,
        "a roda sobre a lista não a rolou"
    );
    let painted = host.paint::<HierarchyPanel>(&mut state, VIEWPORT);
    let r = rect_of(&painted, ultima).expect("rolada até ao fim, a última linha não aparece");
    let events = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        events.contains(&WidgetEvent::Click(ultima)),
        "o clique na última linha não virou o Click dela: {events:?}"
    );
    for ev in events {
        host.apply_panel_event::<HierarchyPanel>(&mut state, ev);
    }
    let esperado = ComponentSet::authored().map(|(b, _)| b).collect::<Vec<_>>();
    assert!(
        state
            .component_filter
            .is_some_and(|b| esperado.contains(&b)),
        "a última linha não aplicou o filtro: {:?}",
        state.component_filter
    );
    assert!(!aberto(&host), "escolher uma linha fecha a lista");
}

/// ⭐⭐ O POLEGAR arrasta: pegar no topo da barra e levá-lo ao fundo rola a lista.
#[test]
fn o_polegar_da_lista_arrasta() {
    let (mut host, mut state, _) = host(true);
    let painted = abre(&mut host, &mut state);
    let barra = rect_of(&painted, DROPDOWN_SCROLLBAR_ID).expect("sem barra");
    let x = barra.x + barra.w * 0.5;
    host.drag_at(x, barra.y + 1.0, x, barra.y + barra.h); // LITERAL-PX-OK: um pixel abaixo do topo da trilha, dentro do polegar em rolagem 0
    assert!(
        host.store().panel_scroll(HIER_FILTER_DD) > 0.0,
        "arrastar o polegar não rolou a lista"
    );
    assert!(aberto(&host), "arrastar a barra não pode fechar a lista");
}

/// CONTROLO: com poucas linhas nada muda — sem barra, e todas registadas.
#[test]
fn com_poucas_linhas_nao_ha_barra() {
    let (mut host, mut state, n) = host(false);
    let painted = abre(&mut host, &mut state);
    assert!(
        rect_of(&painted, DROPDOWN_SCROLLBAR_ID).is_none(),
        "uma lista que cabe não tem barra"
    );
    for i in 0..n {
        assert!(
            rect_of(&painted, hier_filter_opt(i)).is_some(),
            "a linha {i} de uma lista que cabe não foi registada"
        );
    }
}
