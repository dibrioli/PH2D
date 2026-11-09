//! ⭐⭐ **A HIERARCHY e o INSPECTOR fecham pelo «×» do cabeçalho** (dono 09/11), como todos os
//! outros painéis, pelo MESMO caminho da linha do menu *View* (é lá que as duas moram): o «×» esconde, a marca do menu apaga-se e a
//! linha do menu reabre. A tela inteira pintada, um ponteiro real.

use bumpalo::Bump;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

thread_local! {
    static RELOGIO: std::cell::Cell<u128> = const { std::cell::Cell::new(0) };
}

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        // Um segundo por gesto: dois cliques do mesmo teste nunca viram duplo clique.
        timestamp_ns: RELOGIO.with(|r| {
            r.set(r.get() + 1_000_000_000);
            r.get()
        }),
    }
}

fn assenta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    for _ in 0..30 {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || paint_hero_screen(hero, vp, &mut scene, ts),
        );
    }
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert!(
        ev.contains(&WidgetEvent::Click(id)),
        "{nome}: o ponteiro deu {ev:?}; em ({x},{y}) acerta {:?}",
        hero.hit_index.hit_with_rect(x, y)
    );
    for e in ev {
        hero.apply_event(e);
    }
}

fn abre_o_menu_view(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    hero.store.open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 40.0,
        kind: ContextMenuKind::MenuBarView,
    });
    assenta(hero, ts);
}

/// O «×» no canto do cabeçalho esconde o painel; o menu *View* diz-o e reabre-o, com o «×».
#[test]
fn o_x_fecha_a_hierarchy_e_o_inspector_e_o_menu_view_reabre() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for (painel, node, linha) in [
        ("hierarchy", ids::HIER_PANEL, ids::RAIL_SHOW_HIERARCHY),
        ("inspector", ids::INSP_PANEL, ids::RAIL_SHOW_INSPECTOR),
    ] {
        let _ = ph2d_panel_registry_init::register_all_panels();
        // O app em cartões (o de hoje); `PH2D_CARDS=0` é a moldura antiga de referência.
        ph2d_panel_inspector::set_cards_on(true);
        ph2d_panel_hierarchy::cards::force_cards_on_this_thread(Some(true));
        let mut hero = HeroScreen::new(NodeId(1));
        assenta(&mut hero, &mut ts);
        assert!(hero.is_panel_visible(painel), "{painel}: nasce aberto");
        let janela = hero.store.panel_rect(node).expect(painel);
        let fecho = hero
            .store
            .card_list_close(node)
            .unwrap_or_else(|| panic!("{painel}: o cabeçalho não declara «×»"));
        let x = hero
            .hit_index
            .rect_for(fecho)
            .unwrap_or_else(|| panic!("{painel}: o «×» não foi pintado"));
        assert!(
            x.y < janela.y + janela.h * 0.25 && x.x + x.w > janela.x + janela.w * 0.75,
            "{painel}: o «×» {x:?} não está no canto do cabeçalho de {janela:?}"
        );
        clica(&mut hero, fecho, painel);
        assenta(&mut hero, &mut ts);
        assert!(!hero.is_panel_visible(painel), "{painel}: o «×» não fechou");
        assert!(
            hero.store.panel_rect(node).is_none(),
            "{painel}: ainda pinta"
        );
        abre_o_menu_view(&mut hero, &mut ts);
        assert_ne!(
            hero.store.button_state(linha),
            Some(ButtonState::Pressed),
            "{painel}: a marca do menu View ficou acesa com o painel fechado"
        );
        clica(&mut hero, linha, painel);
        assenta(&mut hero, &mut ts);
        assert!(
            hero.is_panel_visible(painel),
            "{painel}: o menu não reabriu"
        );
        assert!(
            hero.store.panel_rect(node).is_some(),
            "{painel}: reaberto e sem janela"
        );
        assert!(
            hero.hit_index.rect_for(fecho).is_some(),
            "{painel}: reaberto sem «×»"
        );
        abre_o_menu_view(&mut hero, &mut ts);
        assert_eq!(
            hero.store.button_state(linha),
            Some(ButtonState::Pressed),
            "{painel}: reaberto e a marca do menu apagada"
        );
    }
}
