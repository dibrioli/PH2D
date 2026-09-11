//! Seam da **CONTAGEM DE OPÇÕES** — a opção marcada é sempre uma que a row consegue oferecer.
//!
//! ⚠️ **O documento muda por quadro e o store não sabe disso.** Uma row de lista guarda a escolha
//! como um ÍNDICE (`Tabs.selected`, `Radio.selected_index`, `Dropdown.selected_index`), e as opções
//! são os FILHOS da forma na Hierarquia — o artista apaga um filho e o índice fica a apontar para
//! um que já não existe. Ninguém reconcilia os dois.
//!
//! ⚠️ **E as quatro variantes falham de maneiras DIFERENTES, o que é pior que falharem igual:**
//! `Tabs::selected` **CLAMPA** (`idx.min(len-1)`) e acende a ÚLTIMA aba; `RadioGroup::select` e
//! `Dropdown::select` **IGNORAM em silêncio** um valor que não está na lista e não acendem nada; e
//! o `event` devolve o índice **CRU** do store. Então o painel pinta uma coisa, o intent carrega
//! outra, e qual das duas está errada depende do tipo que a row veste.

use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::{TabItem, Tabs, WidgetKind};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_authored::state::{AuthoredIntent, AuthoredPanelState};
use ph2d_panel_authored::{AuthoredPanel, drain_intents, ids, rows};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn row(key: &str, kind: WidgetKind, options: &[&str]) -> rows::Row {
    rows::Row {
        kind,
        label: key.to_string(),
        key: key.to_string(),
        id: ids::authored_row_id(key),
        rgba: None,
        icon: None,
        icon_id: None,
        options: options.iter().map(|o| (*o).to_string()).collect(),
    }
}

/// Um clique de verdade no meio de `r`, com o painel a consumir o evento.
fn click(h: &mut MockPanelHost, st: &mut AuthoredPanelState, r: Rect, t: u128) {
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, t));
    for ev in h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, t + SEC / 100)) {
        AuthoredPanel::apply_event(st, h, ev);
    }
}

/// **O gesto REAL: três opções, o artista marca a terceira, e depois apaga um filho.**
///
/// ⚠️ A marca é posta por PONTEIRO, não escrita no store à mão — um `store.register(id, Tabs {
/// selected: 2 })` provaria a mesma coisa por um caminho que o produto não tem, e é justamente a
/// rota do clique que decide se o índice fica cru.
fn marked_third_then_shrunk(kind: WidgetKind) -> (MockPanelHost, AuthoredPanelState) {
    rows::set_live_rows(None);
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    let mut st = AuthoredPanelState;
    rows::set_live_rows(Some(vec![row("mode", kind, &["A", "B", "C"])]));
    // ⚠️ **Um dropdown esconde as opções, e o retângulo delas só nasce no passe DIFERIDO** — sem
    // este clique no chip a fixture procuraria uma opção que o painel (corretamente) não pintou.
    if kind.defers_a_popover() {
        let chip = h
            .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_row_id("mode"))
            .expect("o chip nao foi pintado");
        click(&mut h, &mut st, chip, SEC);
    }
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_option_id("mode", 2))
        .expect("a terceira opcao nao foi pintada");
    click(&mut h, &mut st, r, 2 * SEC);
    // A PREMISSA do gate, declarada: sem ela o resto mede um controle que nunca foi marcado.
    assert_eq!(
        rows::selected_of(h.store().get(ids::authored_row_id("mode"))),
        Some(2),
        "o clique na terceira opcao nao a marcou — a fixture nao contem o fenomeno"
    );
    // O artista apaga um filho na Hierarquia.
    rows::set_live_rows(Some(vec![row("mode", kind, &["A", "B"])]));
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    let _ = drain_intents();
    (h, st)
}

/// **A opção marcada é uma que a row consegue oferecer** — a lei, para as quatro variantes.
///
/// ⚠️ Ela é enunciada sobre o RESULTADO (*o índice cabe na lista*) e não sobre o mecanismo, porque
/// é ela que faz o `paint` e o `event` concordarem por construção: os dois lêem a MESMA porta
/// (`selected_of`), então um índice válido não pode ser desenhado num sítio e devolvido noutro.
#[test]
fn the_marked_option_is_one_the_row_can_offer() {
    for kind in [
        WidgetKind::Tabs,
        WidgetKind::SegmentedAdaptive,
        WidgetKind::RadioGroup,
        WidgetKind::Dropdown,
    ] {
        let (h, _st) = marked_third_then_shrunk(kind);
        let marked = rows::selected_of(h.store().get(ids::authored_row_id("mode")))
            .expect("uma row de lista tem de reportar uma escolha");
        assert!(
            marked < 2,
            "{kind:?}: o store marca a opcao {marked} e a row so oferece 2 — o indice sobreviveu ao \
             filho que o artista apagou"
        );
        rows::set_live_rows(None);
    }
}

/// **Apagar TODOS os filhos não derruba o painel, e não inventa uma escolha.**
///
/// ⚠️ É o gesto que leva `count` a zero, e ali **nenhum índice é válido**: `count - 1` estoura em
/// debug (o perfil do `ship.sh` liga `overflow-checks`), e clampar para 0 afirmaria que o artista
/// escolheu o primeiro de nada. A guarda é load-bearing nas duas metades, e este é o único gate que
/// a vê.
#[test]
fn a_list_with_no_children_neither_panics_nor_invents_a_choice() {
    let (mut h, mut st) = marked_third_then_shrunk(WidgetKind::Tabs);
    rows::set_live_rows(Some(vec![row("mode", WidgetKind::Tabs, &[])]));
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    assert_eq!(
        rows::selected_of(h.store().get(ids::authored_row_id("mode"))),
        Some(1),
        "com zero opcoes a marca tem de ficar onde estava — nao ha indice valido a escolher"
    );
    rows::set_live_rows(None);
}

/// **O que o painel PINTA e o que o intent CARREGA são o mesmo número.**
///
/// ⚠️ A metade *pintada* sai da porta que o `skin.rs` usa (`Tabs::selected`), e não de uma
/// re-derivação: é ela que clampa, e é por isso que uma faixa de abas acende a última enquanto o
/// store aponta para uma que já não está lá. Sem esta metade o gate acima poderia ser satisfeito
/// por uma lei que deixasse os dois consistentes e ERRADOS.
#[test]
fn what_the_strip_lights_is_what_the_click_reports() {
    let (mut h, mut st) = marked_third_then_shrunk(WidgetKind::Tabs);
    let stored = rows::selected_of(h.store().get(ids::authored_row_id("mode"))).unwrap_or(0);
    let items: Vec<TabItem> = ["A", "B"]
        .iter()
        .map(|o| TabItem::new(ids::AUTHORED_PANEL, (*o).to_string()))
        .collect();
    let lit = Tabs::new(ids::authored_row_id("mode"), "mode", items)
        .selected(stored)
        .selected;
    assert_eq!(
        lit, stored,
        "a faixa acende a aba {lit} e o painel reporta a {stored} — o clamp do widget e o indice \
         cru do store discordam"
    );

    // E o intent que sai de um gesto carrega um índice que a lista consegue honrar.
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_option_id("mode", 1))
        .expect("a segunda opcao nao foi pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    for ev in h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)) {
        AuthoredPanel::apply_event(&mut st, &mut h, ev);
    }
    let intents = drain_intents();
    let chosen = intents
        .iter()
        .find_map(|i| match i {
            AuthoredIntent::Choice { index, .. } => Some(*index),
            _ => None,
        })
        .expect("o clique numa opcao nao virou `Choice`");
    assert!(
        chosen < 2,
        "o intent carrega a opcao {chosen} e a lista so tem 2"
    );
    rows::set_live_rows(None);
}
