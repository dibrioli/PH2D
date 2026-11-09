//! ⭐⭐ **As janelas acessórias do Pixel fecham pelo «×» e moram no menu *Window*** (smoke do dono
//! 08/11: *«Não consigo estreitar muito o painel Undo History. Os painéis novos não têm o X de
//! fechar, nem estão presentes no menu Window.»*). A tela inteira pintada, um ponteiro real.

use bumpalo::Bump;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::cards::borda;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_HISTORY, PANEL_PIXEL_KEYS, PANEL_PIXEL_PREFS,
    PANEL_PIXEL_SLICES, PANEL_PIXEL_TILES, paint_hero_screen,
};
use ph2d_editor_core::screens::layout::DOCK_SEAM_PX;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_history::canal::{HistoryView, Row, publish_view};
use ph2d_pixel_core::{SliceId, StepState};
use ph2d_tokens::PANEL_MIN_W_PX;

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

thread_local! {
    static RELOGIO: std::cell::Cell<u128> = const { std::cell::Cell::new(0) };
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

const PASSOS: [&str; 4] = [
    "pixel.history.initial",
    "pixel.history.pencil",
    "pixel.history.paint_bucket",
    "pixel.history.draw",
];

fn publica_conteudo() {
    publish_view(HistoryView {
        rows: PASSOS
            .iter()
            .map(|&label| Row {
                label,
                state: StepState::Done,
            })
            .collect(),
        current: PASSOS.len() - 1,
        bytes: 5400,
    });
    use ph2d_panel_pixel_slices::canal::{self, Contexto, Linha, Vista};
    canal::publish_contexto(Contexto {
        na_mao: true,
        linhas: vec![Linha {
            id: SliceId(1),
            nome: "button".to_owned(),
            selecionada: true,
        }],
        vista: Some(Vista {
            nome: Some("button".to_owned()),
            bounds: [Some(4), Some(6), Some(20), Some(10)],
            ..Vista::default()
        }),
        transform: false,
        user: None,
    });
}

fn hero(ts: &mut ph2d_text::TextSystem, aberta: &'static str) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    publica_conteudo();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(aberta, true);
    assenta(&mut hero, ts);
    hero
}

/// O «×» de cada janela nova está no cabeçalho e esconde-a.
#[test]
fn o_x_fecha_cada_janela() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for (painel, node) in [
        (PANEL_PIXEL_HISTORY, ids::PIXEL_HISTORY_PANEL),
        (PANEL_PIXEL_KEYS, ids::PIXEL_KEYS_PANEL),
        (PANEL_PIXEL_PREFS, ids::PIXEL_PREFS_PANEL),
        (PANEL_PIXEL_SLICES, ids::PIXEL_SLICES_PANEL),
    ] {
        let mut hero = hero(&mut ts, painel);
        let janela = hero.store.panel_rect(node).expect(painel);
        let x = hero
            .hit_index
            .rect_for(borda::fecho_de(node))
            .unwrap_or_else(|| panic!("{painel}: sem «×»"));
        assert!(
            x.y < janela.y + janela.h * 0.25 && x.x + x.w > janela.x + janela.w * 0.75,
            "{painel}: o «×» {x:?} não está no canto do cabeçalho de {janela:?}"
        );
        clica(&mut hero, borda::fecho_de(node), painel);
        assenta(&mut hero, &mut ts);
        assert!(!hero.is_panel_visible(painel), "{painel}: o «×» não fechou");
        assert!(
            hero.store.panel_rect(node).is_none(),
            "{painel}: ainda pinta"
        );
    }
}

/// A linha do menu *Window* abre e fecha cada janela, e a marca segue a verdade.
#[test]
fn a_linha_do_menu_window_alterna_cada_janela() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for (painel, linha) in [
        (PANEL_PIXEL_SLICES, ids::TOPBAR_PIXEL_SLICES),
        (PANEL_PIXEL_TILES, ids::TOPBAR_PIXEL_TILES),
        (PANEL_PIXEL_HISTORY, ids::TOPBAR_PIXEL_HISTORY),
        (PANEL_PIXEL_KEYS, ids::TOPBAR_PIXEL_KEYS),
        (PANEL_PIXEL_PREFS, ids::TOPBAR_PIXEL_PREFS),
    ] {
        let _ = ph2d_panel_registry_init::register_all_panels();
        publica_conteudo();
        let mut hero = HeroScreen::new(NodeId(1));
        assenta(&mut hero, &mut ts);
        for esperado in [true, false] {
            hero.store.open_context_menu(ContextMenuRequest {
                x: 40.0,
                y: 40.0,
                kind: ContextMenuKind::MenuBarWindow,
            });
            assenta(&mut hero, &mut ts);
            clica(&mut hero, linha, painel);
            assert_eq!(hero.is_panel_visible(painel), esperado, "{painel}");
            assenta(&mut hero, &mut ts);
            assert_eq!(hero.is_panel_visible(painel), esperado, "{painel}");
            assert_eq!(
                hero.store.button_state(linha)
                    == Some(ph2d_editor_core::widget::ButtonState::Pressed),
                esperado,
                "{painel}: a marca do menu não segue a janela"
            );
        }
    }
}

/// ⭐⭐ **A marca do menu *Window* é a janela no ecrã** (P12 U4, smoke do dono 08/11): sem a
/// ferramenta Slice na mão, sem passos e sem camada tilemap, a linha abre a janela com a linha de
/// vazio — nunca uma marca sem janela.
#[test]
fn a_marca_do_menu_e_a_janela_no_ecra_mesmo_sem_nada_a_mostrar() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for (painel, linha, node) in [
        (
            PANEL_PIXEL_SLICES,
            ids::TOPBAR_PIXEL_SLICES,
            ids::PIXEL_SLICES_PANEL,
        ),
        (
            PANEL_PIXEL_TILES,
            ids::TOPBAR_PIXEL_TILES,
            ids::PIXEL_TILES_PANEL,
        ),
        (
            PANEL_PIXEL_HISTORY,
            ids::TOPBAR_PIXEL_HISTORY,
            ids::PIXEL_HISTORY_PANEL,
        ),
    ] {
        let _ = ph2d_panel_registry_init::register_all_panels();
        publish_view(HistoryView::default());
        // O Lápis na mão: a família publica as slices, mas não a mão.
        ph2d_panel_pixel_slices::canal::publish_contexto(
            ph2d_panel_pixel_slices::canal::Contexto::default(),
        );
        ph2d_panel_pixel_tiles::canal::publish_contexto(
            ph2d_panel_pixel_tiles::canal::Contexto::default(),
        );
        let mut hero = HeroScreen::new(NodeId(1));
        assenta(&mut hero, &mut ts);
        hero.store.open_context_menu(ContextMenuRequest {
            x: 40.0,
            y: 40.0,
            kind: ContextMenuKind::MenuBarWindow,
        });
        assenta(&mut hero, &mut ts);
        clica(&mut hero, linha, painel);
        assenta(&mut hero, &mut ts);
        assert!(
            hero.is_panel_visible(painel),
            "{painel}: a linha não a abriu"
        );
        assert_eq!(
            hero.store.button_state(linha),
            Some(ph2d_editor_core::widget::ButtonState::Pressed),
            "{painel}: sem marca"
        );
        assert!(
            hero.store.panel_rect(node).is_some(),
            "{painel}: a marca acesa e nenhuma janela no ecrã"
        );
    }
}

/// ⭐ A borda de fora da UNDO HISTORY estreita-a até ao piso do CONTEÚDO (o passo mais largo
/// inteiro), bem abaixo do mínimo dos painéis, e ele segura.
#[test]
fn a_undo_history_estreita_ate_ao_piso_do_conteudo() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero(&mut ts, PANEL_PIXEL_HISTORY);
    let node = ids::PIXEL_HISTORY_PANEL;
    let antes = hero.store.panel_rect(node).expect("pintou");
    let folga = PANEL_MIN_W_PX - antes.w;
    let nomes: Vec<String> = PASSOS.iter().map(|k| ph2d_i18n::tr(k).to_owned()).collect();
    let piso = borda::piso_das_linhas(&mut ts, nomes.iter().map(String::as_str));
    assert!(
        piso < PANEL_MIN_W_PX * 0.75,
        "o piso {piso} não estreita nada ({PANEL_MIN_W_PX})"
    );
    let (x, y) = (antes.x + DOCK_SEAM_PX * 0.5, antes.y + antes.h * 0.5);
    let arena = Bump::new();
    let _ = hero.handle_pointer(ptr(PointerKind::Down, x, y), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, x + 600.0, y), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Up, x + 600.0, y), &arena);
    assenta(&mut hero, &mut ts);
    let depois = hero.store.panel_rect(node).expect("continua");
    assert_eq!(depois.w + folga, piso, "o piso é o do conteúdo");
    assert_eq!(
        depois.x + depois.w,
        antes.x + antes.w,
        "a direita fica presa"
    );
    assenta(&mut hero, &mut ts);
    let ainda = hero.store.panel_rect(node).expect("continua");
    assert_eq!(ainda.w, depois.w, "o piso segura quadro a quadro");
    assert!(
        hero.hit_index.rect_for(borda::fecho_de(node)).is_some(),
        "no piso o «×» continua"
    );
}
