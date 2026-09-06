//! Seam da **LISTA LONGA** — uma lista que não cabe na tela ROLA, em vez de sair dela.
//!
//! ⚠️ **O `popover_rect_clamped` fazia o trabalho dele e ninguém fazia o resto.** Ele encolhe o
//! PAINEL para o espaço que há (é o terceiro braço dele, quando nem abaixo nem acima cabe), mas as
//! LINHAS continuavam dispostas a `row_h * index` a partir do topo do painel — então as últimas
//! eram pintadas e ganhavam retângulo de hit **fora** dele. Medido, com 30 opções num viewport de
//! 900: as opções 26 a 29 pousam em `y = 900..1012`, ou seja **abaixo da borda de baixo da tela**.
//! Desenhadas, vivas para o dispatcher, e inalcançáveis.
//!
//! ⚠️ **E a resposta já existia no repo, três vezes** — `paint_filters_blend`, `font_dropdown` e o
//! `dropdown_popover` do painel de camadas usam `paint_dropdown_popover_scrolled` +
//! `option_rect_in_scrolled` e recortam o retângulo de hit ao painel. Este gate existe para que o
//! painel gerado seja o quarto, e não uma quarta resposta.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::{DropdownState, WidgetKind};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_authored::state::AuthoredPanelState;
use ph2d_panel_authored::{AuthoredPanel, ids, rows};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
/// ⚠️⚠️ **O número é DERIVADO da altura de linha, e antes era `30` escrito à mão.**
///
/// O `30` saíra de uma varredura (com `20` a lista ainda cabia abaixo do chip) — mas a varredura
/// mediu a árvore de um dia: quando o dono pediu linhas mais compactas e o `chrome.row-h` desceu
/// de `28` para `22` px (2026-09-06), a lista de trinta passou a caber **exactamente** na janela
/// (`664` de conteúdo em `664` visíveis) e o gate deixou de conter o fenómeno. ⭐ **Ele disse-o em
/// voz alta** — *«a fixture nao transborda»* — em vez de passar por vácuo, que é o desenho certo.
///
/// ⇒ conta-se: quantas linhas cabem na janela INTEIRA, mais folga. Assim a lista nunca cabe nem
/// abaixo nem acima do chip, que é o braço do `popover_rect_clamped` que este gate existe para
/// alcançar — e a conta sobrevive à próxima vez que a linha mudar de altura.
fn options() -> usize {
    (VIEWPORT.h / ph2d_tokens::ROW_H_PX).ceil() as usize + 8
}

fn long_list() -> (ph2d_a11y::NodeId, String) {
    let key = "mode".to_string();
    let id = ids::authored_row_id(&key);
    rows::set_live_rows(Some(vec![rows::Row {
        kind: WidgetKind::Dropdown,
        label: key.clone(),
        key: key.clone(),
        id,
        rgba: None,
        icon: None,
        icon_id: None,
        options: (0..options()).map(|i| format!("Op{i}")).collect(),
    }]));
    (id, key)
}

fn host_with_open_list(id: ph2d_a11y::NodeId) -> (MockPanelHost, AuthoredPanelState) {
    let mut h = MockPanelHost::with_panel::<AuthoredPanel>();
    h.set_panel_visible(AuthoredPanel::ID, true);
    h.store_mut().register(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: true,
            selected_index: Some(0),
        },
    );
    (h, AuthoredPanelState)
}

/// **Nenhuma opção é oferecida fora do painel que a contém.**
///
/// ⚠️ O oráculo é o PAINEL e não o viewport, e é o mais apertado dos dois: o painel já está dentro
/// da tela (o `popover_rect_clamped` garante-o), então uma linha que caiba nele cabe na tela — e
/// uma que saia dele está por cima do que estiver por baixo, que é o modo de falha que se sente.
#[test]
fn no_option_is_offered_outside_the_panel_that_holds_it() {
    let (id, key) = long_list();
    let (mut h, mut st) = host_with_open_list(id);
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    let panel = h
        .store()
        .dropdown_popover()
        .map(|(_, r)| r)
        .expect("o painel nao publicou onde a lista aberta esta' — a roda nunca a alcanca");

    let mut offered = 0;
    for i in 0..options() {
        let Some(r) =
            h.painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, ids::authored_option_id(&key, i))
        else {
            continue;
        };
        offered += 1;
        assert!(
            r.y >= panel.y - 0.5 && r.y + r.h <= panel.y + panel.h + 0.5,
            "a opcao {i} ({:.1}..{:.1}) foi oferecida FORA do painel ({:.1}..{:.1}) — ela esta' \
             desenhada por cima do que houver ali, e o clique nela nao e' o clique que parece",
            r.y,
            r.y + r.h,
            panel.y,
            panel.y + panel.h
        );
    }
    assert!(
        offered >= 2,
        "so {offered} opcoes foram oferecidas — a fixture nao contem uma LISTA"
    );
    rows::set_live_rows(None);
}

/// **A opção que não cabe é ALCANÇÁVEL rolando** — a metade que impede a cura preguiçosa.
///
/// ⚠️ Sem ela, um "conserto" que simplesmente deixasse de oferecer as linhas que transbordam
/// passaria no gate acima e deixaria o produto PIOR: as últimas opções deixariam de comer cliques
/// alheios e passariam a não existir — o artista veria uma lista que não chega ao fim dela.
///
/// ⚠️ **O rolamento é escrito pelo mesmo número que a roda escreve** (`panel_scroll` do id do
/// dropdown), e o alcance vem dos dois que o painel PUBLICA — é assim que o `dispatch_wheel`
/// decide quanto rolar, então usar outros aqui mediria uma lista que o rato não move.
#[test]
fn the_option_that_does_not_fit_is_reachable_by_scrolling() {
    let (id, key) = long_list();
    let (mut h, mut st) = host_with_open_list(id);
    let _ = h.paint::<AuthoredPanel>(&mut st, VIEWPORT);
    let content_h = h
        .store()
        .panel_content_h(id)
        .expect("a lista nao publicou a altura do conteudo");
    let visible_h = h
        .store()
        .panel_visible_h(id)
        .expect("a lista nao publicou a altura visivel");
    assert!(
        content_h > visible_h + 1.0,
        "a fixture nao transborda ({content_h:.1} de conteudo em {visible_h:.1} visiveis) — o gate \
         mediria uma lista que cabe"
    );

    h.store_mut().set_panel_scroll(id, content_h - visible_h);
    let last = ids::authored_option_id(&key, options() - 1);
    let panel = h
        .store()
        .dropdown_popover()
        .map(|(_, r)| r)
        .expect("sem painel");
    let r = h
        .painted_rect::<AuthoredPanel>(&mut st, VIEWPORT, last)
        .expect(
            "a ultima opcao nao existe nem com a lista rolada ate' o fim — ela e' inalcancavel",
        );
    assert!(
        r.y >= panel.y - 0.5 && r.y + r.h <= panel.y + panel.h + 0.5,
        "a ultima opcao ({:.1}..{:.1}) continua fora do painel ({:.1}..{:.1}) depois de rolar tudo",
        r.y,
        r.y + r.h,
        panel.y,
        panel.y + panel.h
    );
    rows::set_live_rows(None);
}
