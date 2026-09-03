//! ⭐⭐⭐ **Um painel rola arrastando o CORPO — o gesto que um tablet precisa.**
//!
//! # O buraco, medido
//!
//! Censo completo de quem escreve `panel_scroll` (2026-09-03): a **roda** e o **polegar da barra**.
//! Mais nada — não há `kinetic`, `fling` nem arrasto de corpo em lado nenhum do repo. E
//! `PointerSource::Touch` tem **zero** usos fora do `ph2d-host`, logo o app nem consegue distinguir
//! um dedo de um rato.
//!
//! ⇒ **num tablet, um painel só rolava se o dedo acertasse exactamente nos `10 px` da barra.** O
//! tablet é o pré-requisito declarado deste redesenho inteiro (Enio: *«pre-requisito já que
//! queremos trabalhar no ipad»*).
//!
//! ⛔ **E é por isto que a proposta de encolher a barra para `2 px` em repouso** (pesquisa `07`
//! §5.3, escrita por mim) **foi RECUSADA:** o `SCROLLBAR_W = 10` traz uma cerca com as palavras do
//! dono — *"comfortable drag target on iPad/tablet"* — e afiná-la teria tornado a **única** via de
//! rolagem táctil cinco vezes mais fina. *Os 8 px de largura pagavam-se com a rolagem.*
//!
//! # A guarda que torna isto seguro
//!
//! O arrasto só arma quando a pressão **não acertou em nada** (`hit.is_none()`). Nenhum painel
//! regista o próprio fundo no `HitIndex`, então *"não acertou em nada"* é exactamente *"espaço
//! vazio do painel"* — e um widget que reclame a pressão fica com ela. O terceiro teste é o
//! controlo dessa afirmação.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{
    HitIndex, InteractiveState, WidgetStore, dispatch_pointer, format_number,
};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerEvent, PointerKind, PointerSource};

const PANEL: NodeId = NodeId(700);
const FIELD: NodeId = NodeId(701);

const PANEL_RECT: Rect = Rect {
    x: 100.0,
    y: 50.0,
    w: 260.0,
    h: 400.0,
};
/// O corpo tem `400` visíveis para `1000` de conteúdo ⇒ `600` de curso.
const CONTENT_H: f32 = 1000.0;
const VISIBLE_H: f32 = 400.0;

fn ev(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: t,
    }
}

/// Um painel rolável publicado como o pintor o publica, com um campo numérico lá dentro.
fn scrollable_panel() -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(8);
    store.set_panel_rect(PANEL, PANEL_RECT);
    store.set_panel_content_h(PANEL, CONTENT_H);
    store.set_panel_visible_h(PANEL, VISIBLE_H);
    store.register(
        FIELD,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 1.0,
            buffer: format_number(1.0),
            caret: 0,
            last_committed: 1.0,
            selection_anchor: None,
        },
    );
    let mut hits = HitIndex::default();
    // Um widget ocupa uma faixa do painel; o resto é espaço vazio.
    hits.register(FIELD, Rect::new(110.0, 60.0, 100.0, 22.0));
    (store, hits)
}

/// **Arrastar o espaço vazio ROLA, e o conteúdo segue o dedo 1:1.**
///
/// **Mutação que deve sangrar:** reciclar a conta PROPORCIONAL da barra
/// (`scrollbar_delta_for_drag`) — a `1000/400` o conteúdo andaria `2,5×` o dedo e fugir-lhe-ia.
#[test]
fn dragging_the_empty_body_scrolls_it_one_to_one() {
    let (mut store, hits) = scrollable_panel();
    let arena = Bump::new();
    let (x, y0) = (300.0, 300.0);
    let _ = dispatch_pointer(&mut store, &hits, ev(PointerKind::Down, x, y0, 0), &arena);
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Move, x, y0 - 40.0, 1_000_000),
        &arena,
    );
    assert!(
        (store.panel_scroll(PANEL) - 40.0).abs() < 0.001,
        "arrastar 40 px para CIMA tinha de rolar 40 px, e rolou {}",
        store.panel_scroll(PANEL)
    );
    // E ao contrário, a partir de um ponto já rolado.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Move, x, y0 - 10.0, 2_000_000),
        &arena,
    );
    assert!(
        (store.panel_scroll(PANEL) - 10.0).abs() < 0.001,
        "o arrasto tem de seguir a POSIÇÃO do dedo, nao acumular deltas: {}",
        store.panel_scroll(PANEL)
    );
}

/// **O curso respeita o fim do conteúdo, e o `Up` larga.**
#[test]
fn the_drag_stops_at_the_end_and_the_release_ends_it() {
    let (mut store, hits) = scrollable_panel();
    let arena = Bump::new();
    let (x, y0) = (300.0, 300.0);
    let _ = dispatch_pointer(&mut store, &hits, ev(PointerKind::Down, x, y0, 0), &arena);
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Move, x, y0 - 5_000.0, 1_000_000),
        &arena,
    );
    let max = CONTENT_H - VISIBLE_H;
    assert!(
        (store.panel_scroll(PANEL) - max).abs() < 0.001,
        "o arrasto passou o fim do conteudo: {} (tecto {max})",
        store.panel_scroll(PANEL)
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Up, x, y0 - 5_000.0, 2_000_000),
        &arena,
    );
    let after_release = store.panel_scroll(PANEL);
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Move, x, y0 + 5_000.0, 3_000_000),
        &arena,
    );
    assert!(
        (store.panel_scroll(PANEL) - after_release).abs() < 0.001,
        "mover DEPOIS de largar continuou a rolar — o arrasto nao terminou"
    );
}

/// ⛔ **O CONTROLO: uma pressão que um WIDGET reclamou nunca vira rolagem.**
///
/// ⚠️ É esta a metade que torna o gesto seguro. Sem ela, arrastar dentro de um campo de texto — ou
/// sobre um slider — rolaria o painel por baixo, e o app inteiro passaria a ter um gesto fantasma
/// a competir com todos os outros.
#[test]
fn a_press_a_widget_claimed_never_becomes_a_scroll() {
    let (mut store, hits) = scrollable_panel();
    let arena = Bump::new();
    // (160, 70) cai DENTRO do campo numérico registado.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Down, 160.0, 70.0, 0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Move, 160.0, 30.0, 1_000_000),
        &arena,
    );
    assert!(
        store.panel_scroll(PANEL).abs() < 0.001,
        "arrastar DENTRO de um widget rolou o painel ({}): o gesto esta' a roubar pressoes",
        store.panel_scroll(PANEL)
    );
}

/// ⛔ **E um painel que CABE não rola** — arrastar o vazio dele não pode mexer em nada.
#[test]
fn a_panel_that_fits_does_not_scroll() {
    let (mut store, hits) = scrollable_panel();
    // O conteúdo passa a caber: `content_h <= visible_h`.
    store.set_panel_content_h(PANEL, VISIBLE_H);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Down, 300.0, 300.0, 0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        ev(PointerKind::Move, 300.0, 100.0, 1_000_000),
        &arena,
    );
    assert!(
        store.panel_scroll(PANEL).abs() < 0.001,
        "um painel sem curso rolou {}",
        store.panel_scroll(PANEL)
    );
}
