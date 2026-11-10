//! ⭐ **A faixa do onion responde a um PONTEIRO real** (P12 R4): com o onion ligado, os colchetes
//! `[` `]` ficam sobre os números dos quadros, à volta do actual (1 antes, 1 depois de fábrica);
//! arrastar o `]` duas colunas para a direita põe `next = 3` nas preferências do documento. O
//! botão DIREITO no botão do onion abre o cartão Onion Skin.

use bumpalo::Bump;
use ph2d_editor_core::pixel_panel::{PixelFrameCol, PixelLayerRow, PixelPanelInfo};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_PREFS, PANEL_PIXEL_TIMELINE, PANEL_TIMELINE,
    paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_prefs::canal;
use ph2d_panel_pixel_timeline::ids;
use ph2d_pixel_core::PixelPrefs;
use ph2d_pixel_render::prefs_canal as fio;

fn ptr(kind: PointerKind, x: f32, y: f32, button: PointerButton) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button,
        timestamp_ns: 0,
    }
}

fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, n: usize) {
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    for _ in 0..n {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || paint_hero_screen(hero, vp, &mut scene, ts),
        );
    }
}

/// Oito quadros (ids 1..=8), o actual é o 3.º, o onion ligado.
fn palco(ts: &mut ph2d_text::TextSystem) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let _ = fio::troca_em(Some(PixelPrefs::default()), None);
    ph2d_panel_pixel_timeline::set_current_pixel(PixelPanelInfo {
        editing: true,
        onion: true,
        layers: vec![PixelLayerRow {
            id: 4,
            name: "Ink".to_owned(),
            depth: 0,
            group: false,
            collapsed: false,
            visible: true,
            locked: false,
            continuous: false,
            normal: false,
            reference: false,
            background: false,
            opacity: 255,
            blend: Default::default(),
            outline: None,
        }],
        active_layer: Some(4),
        frames: (1..=8)
            .map(|id| PixelFrameCol {
                id,
                duration_ms: 100,
            })
            .collect(),
        active_frame: Some(3),
        ..PixelPanelInfo::default()
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL_TIMELINE, true);
    // A banda de baixo é de um painel de cada vez: a linha do tempo da casa cede-a (como em Draw).
    hero.panel_visibility.insert(PANEL_TIMELINE, false);
    pinta(&mut hero, ts, 30);
    hero
}

fn rect(hero: &HeroScreen, id: NodeId, nome: &str) -> Rect {
    hero.hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"))
}

#[test]
fn dragging_the_right_onion_bracket_two_frames_right_sets_next_to_three() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = palco(&mut ts);
    let (h2, h4, h5) = (
        rect(&hero, ids::header_id(2), "o cabeçalho 2"),
        rect(&hero, ids::header_id(4), "o cabeçalho 4"),
        rect(&hero, ids::header_id(5), "o cabeçalho 5"),
    );
    let abre = rect(&hero, ids::ONION_EDGES[0], "o `[`");
    let fecha = rect(&hero, ids::ONION_EDGES[1], "o `]`");
    assert!(
        (abre.x - h2.x).abs() <= abre.w && fecha.x + fecha.w <= h4.x + h4.w + 0.5,
        "de fábrica a faixa é [2 … 4]: {abre:?} {fecha:?} {h2:?} {h4:?}"
    );
    let passo = h5.x - h4.x;
    let (x, y) = (fecha.x + fecha.w * 0.5, fecha.y + fecha.h * 0.5);
    assert_eq!(
        hero.hit_index.hit_with_rect(x, y).map(|(id, _)| id),
        Some(ids::ONION_EDGES[1]),
        "o `]` está por cima do número"
    );
    let arena = Bump::new();
    let b = PointerButton::Primary;
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y, b), &arena)
        .to_vec();
    for k in 1..=4 {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let dx = passo * 2.0 * k as f32 / 4.0;
        ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Move, x + dx, y, b), &arena));
    }
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x + passo * 2.0, y, b), &arena));
    for e in ev {
        hero.apply_event(e);
    }
    pinta(&mut hero, &mut ts, 1);
    let p = fio::troca_em(Some(PixelPrefs::default()), None).expect("o alcance mudou");
    assert_eq!((p.onion.prev, p.onion.next), (1, 3));
    assert_eq!(
        PixelPrefs {
            onion: PixelPrefs::default().onion,
            ..p
        },
        PixelPrefs::default(),
        "só o onion mudou"
    );
    pinta(&mut hero, &mut ts, 1);
    let fecha = rect(&hero, ids::ONION_EDGES[1], "o `]`");
    let h6 = rect(&hero, ids::header_id(6), "o cabeçalho 6");
    assert!(
        fecha.x >= h6.x && fecha.x + fecha.w <= h6.x + h6.w + 0.5,
        "o `]` foi para o 6"
    );
}

#[test]
fn right_clicking_the_onion_button_opens_the_onion_skin_card() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = palco(&mut ts);
    assert!(!hero.is_panel_visible(PANEL_PIXEL_PREFS));
    let r = rect(&hero, ids::TOGGLES[0].1, "o botão do onion");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = Bump::new();
    let b = PointerButton::Secondary;
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y, b), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y, b), &arena));
    for e in ev {
        hero.apply_event(e);
    }
    pinta(&mut hero, &mut ts, 30);
    assert!(hero.is_panel_visible(PANEL_PIXEL_PREFS), "a janela abriu");
    assert!(
        hero.hit_index
            .rect_for(ph2d_panel_pixel_prefs::ids::caixa(canal::Caixa::OnionFront))
            .is_some(),
        "no cartão Onion Skin"
    );
}
