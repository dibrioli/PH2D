//! ⭐⭐ **As leis da porta da lista que ROLA** (`panel::paint_dropdown_list`), medidas sobre a porta
//! e o despacho reais, com o `HeroScreen` como hospedeiro.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, dispatch_pointer};
use ph2d_editor_core::panel::{PaintCtx, paint_dropdown_list};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, DropdownState, PaintedPopover, PopoverRow,
    ScrollbarState,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1200.0,
    h: 800.0,
};
/// Uma banda baixa: a lista de 40 não cabe, o clamp encolhe o painel e a lista rola.
const REGION: Rect = Rect {
    x: 0.0,
    y: 100.0,
    w: 1200.0,
    h: 300.0,
};
const CHIP: Rect = Rect {
    x: 40.0,
    y: 120.0,
    w: 200.0,
    h: 24.0,
};
const DD: NodeId = NodeId(9_100_000);
const N: usize = 40;
const APAGADA: usize = 1;
/// Um TÍTULO de categoria (`header`) — pinta-se como cabeçalho e não se oferece.
const TITULO: usize = 3;

fn opt(i: usize) -> NodeId {
    NodeId(9_100_001 + i as u64)
}

fn lista() -> Dropdown<usize> {
    let options = (0..N)
        .map(|i| {
            DropdownOption::new(opt(i), i, format!("{i}"))
                .disabled(i == APAGADA)
                .header(i == TITULO)
        })
        .collect();
    Dropdown::new(DD, "", options).selected(2).open(true)
}

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.register(
        DD,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: true,
            selected_index: Some(2),
        },
    );
    hero
}

/// Pinta pela PORTA e devolve o que ela publicou e as linhas que o pintor de linha recebeu.
fn pinta(hero: &mut HeroScreen) -> (PaintedPopover, Vec<PopoverRow>) {
    hero.hit_index.clear_for_frame();
    let layout = HeroLayout::for_viewport(VIEWPORT);
    let mut scene = VectorScene::new();
    let mut text_system = TextSystem::without_system_fonts();
    let mut vistas = Vec::new();
    let mut linha = |_: &mut VectorScene, _: &mut TextSystem, r: PopoverRow| {
        vistas.push(r);
    };
    let painted = {
        let mut ctx = PaintCtx {
            host: hero,
            layout: &layout,
            slot: VIEWPORT,
            viewport: VIEWPORT,
            scene: &mut scene,
            text_system: &mut text_system,
        };
        paint_dropdown_list(
            &mut ctx,
            &lista(),
            CHIP,
            REGION,
            Theme::Forge,
            Some(&mut linha),
        )
    };
    (painted, vistas)
}

fn ponteiro(hero: &mut HeroScreen, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
    let arena = bumpalo::Bump::new();
    let event = PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    dispatch_pointer(&mut hero.store, &hero.hit_index, event, &arena).to_vec()
}

fn hit_de(hero: &HeroScreen, id: NodeId) -> Option<Rect> {
    hero.hit_index
        .iter_registrations()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| r)
}

/// ⭐⭐ A porta PUBLICA o alcance, grava de volta a rolagem clampada, e oferece só o VISÍVEL —
/// como hit E como focável; a opção apagada nunca.
///
/// **Mutações que devem sangrar:** tirar o `publish_dropdown_popover` (a rolagem rançosa fica) ·
/// registar o `option_rect_in_scrolled` inteiro · registar como focável TODA a lista.
#[test]
fn a_porta_publica_clampa_e_oferece_so_o_visivel() {
    let mut hero = hero();
    hero.store.set_panel_scroll(DD, 1.0e6);
    let (p, vistas) = pinta(&mut hero);
    assert!(
        p.content_h > p.visible_h,
        "a fixtura não transborda: {} em {}",
        p.content_h,
        p.visible_h
    );
    assert_eq!(hero.store.dropdown_popover(), Some((DD, p.panel)));
    let max_scroll = p.content_h - p.visible_h;
    assert!(
        (hero.store.panel_scroll(DD) - max_scroll).abs() < 0.01,
        "a rolagem rançosa não foi clampada de volta no store: {}",
        hero.store.panel_scroll(DD)
    );
    assert!(
        hit_de(&hero, DROPDOWN_SCROLLBAR_ID).is_some(),
        "a barra não foi registada"
    );
    let mut vivas = 0;
    for i in 0..N {
        let hit = hit_de(&hero, opt(i));
        let focavel = matches!(
            hero.store.get(opt(i)),
            Some(InteractiveState::Button { .. })
        );
        assert_eq!(
            hit.is_some(),
            focavel,
            "a linha {i}: hit e focável têm de ser o MESMO conjunto"
        );
        if let Some(r) = hit {
            vivas += 1;
            assert!(
                r.y >= p.panel.y - 0.01 && r.y + r.h <= p.panel.y + p.panel.h + 0.01,
                "a linha {i} foi registada fora da lista"
            );
        }
    }
    assert!(vivas > 0 && vivas < N, "esperava-se uma FATIA, viu {vivas}");
    assert!(
        vistas.iter().all(|r| r.index > APAGADA),
        "rolada até ao fim, uma linha do topo ainda chegou ao pintor"
    );
    assert!(
        vistas.iter().any(|r| r.index == N - 1),
        "a última linha não chegou ao pintor de linha"
    );
}

/// O pintor de linha recebe as linhas visíveis com o realce e o apagado certos — a apagada pinta
/// mas não se oferece.
#[test]
fn o_pintor_de_linha_recebe_o_visivel_com_o_estado() {
    let mut hero = hero();
    let (_, vistas) = pinta(&mut hero);
    let r = |i: usize| vistas.iter().find(|r| r.index == i).copied();
    assert!(
        r(APAGADA).is_some_and(|r| r.disabled),
        "a apagada não pintou"
    );
    assert!(
        r(2).is_some_and(|r| r.selected),
        "a vigente não veio realçada"
    );
    assert!(r(0).is_some_and(|r| !r.selected && !r.disabled));
    assert!(
        hit_de(&hero, opt(APAGADA)).is_none(),
        "a apagada ofereceu-se"
    );
    assert!(
        r(N - 1).is_none(),
        "uma linha abaixo da dobra chegou ao pintor"
    );
}

/// ⭐ **Um título de categoria não é uma opção** (dono 22/10): não se regista (nem clique, nem
/// foco) e não chega ao pintor de linha — a porta pinta-o como cabeçalho, com o seu próprio aspecto.
#[test]
fn um_titulo_nao_se_oferece_nem_chega_ao_pintor_de_linha() {
    let mut hero = hero();
    let (_, vistas) = pinta(&mut hero);
    assert!(
        hit_de(&hero, opt(TITULO)).is_none(),
        "o título ofereceu-se ao clique"
    );
    assert!(
        hero.store.get(opt(TITULO)).is_none(),
        "o título ficou focável"
    );
    assert!(
        vistas.iter().all(|r| r.index != TITULO),
        "o título chegou ao pintor de linha como uma opção"
    );
    assert!(
        hit_de(&hero, opt(TITULO + 1)).is_some(),
        "controlo: a opção sob o título continua viva"
    );
}

/// ⭐⭐⭐ **O polegar AGARRADO lê-se `Dragging` também pela pergunta SEM dono.**
///
/// ⛔ O picker de ícone e o de fonte do Vector pediam `scrollbar_visual(DROPDOWN_SCROLLBAR_ID)`; o
/// dono dessa barra não está no mapa da porta das barras — é o DESPACHO que o resolve, pelo
/// dropdown aberto ((`WidgetStore::dropdown_popover`)). Sem a mesma resolução
/// aqui, o polegar arrastado pintava-se em repouso.
#[test]
fn o_polegar_agarrado_le_se_dragging_pela_pergunta_sem_dono() {
    let mut hero = hero();
    let _ = pinta(&mut hero);
    let barra = hit_de(&hero, DROPDOWN_SCROLLBAR_ID).expect("sem barra");
    let x = barra.x + barra.w * 0.5;
    ponteiro(&mut hero, PointerKind::Down, x, barra.y + 1.0); // LITERAL-PX-OK: um pixel abaixo do topo da trilha, dentro do polegar em rolagem 0
    ponteiro(&mut hero, PointerKind::Move, x, barra.y + barra.h * 0.5);
    assert!(
        hero.store.panel_scroll(DD) > 0.0,
        "o arrasto não rolou a lista"
    );
    assert_eq!(
        hero.store
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(DD))
            .0,
        ScrollbarState::Dragging,
        "controlo: chaveada pelo chip a barra lê-se agarrada"
    );
    assert_eq!(
        hero.store.scrollbar_visual(DROPDOWN_SCROLLBAR_ID).0,
        ScrollbarState::Dragging,
        "a pergunta sem dono pinta o polegar agarrado em repouso"
    );
    ponteiro(&mut hero, PointerKind::Up, x, barra.y + barra.h * 0.5);
}
