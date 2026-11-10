//! ⭐ **A Timeline nos cartões arrasta-se pelo título, redimensiona-se pelos cantos e encaixa de
//! volta** (dono 10/11). Medido sobre o quadro real: pintado, gesto pelo despacho, pintado outra vez.

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_timeline::ids::{TIMELINE_CARD_DOCK, TIMELINE_CARD_DRAG, TIMELINE_RESIZE_BR};
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1366.0, 1024.0);
const PAINEL: ph2d_editor_core::NodeId = ph2d_editor_core::ids::TIMELINE_PANEL;

fn pinta(h: &mut HeroScreen) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..3 {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
    }
}

fn so_a_timeline(cartoes: bool) -> HeroScreen {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(cartoes));
    ph2d_editor_core::widget::lista_flutuante::force_cromo_on_this_thread(Some(cartoes));
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            let id = p.manifest.id;
            h.panel_visibility
                .insert(id, id == ph2d_editor_core::screens::hero::PANEL_TIMELINE);
        }
    });
    pinta(&mut h);
    h
}

fn solta_os_modos() {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    ph2d_editor_core::widget::lista_flutuante::force_cromo_on_this_thread(None);
}

fn evento(h: &mut HeroScreen, kind: PointerKind, (x, y): (f32, f32)) {
    let arena = bumpalo::Bump::new();
    let ev = PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let evs: Vec<_> =
        ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
            .to_vec();
    for e in evs {
        h.apply_event(e);
    }
}

/// Down em `de`, um Move até `ate`, Up, e o quadro seguinte.
fn arrasta(h: &mut HeroScreen, de: (f32, f32), ate: (f32, f32)) {
    evento(h, PointerKind::Down, de);
    evento(
        h,
        PointerKind::Move,
        ((de.0 + ate.0) * 0.5, (de.1 + ate.1) * 0.5),
    );
    evento(h, PointerKind::Move, ate);
    evento(h, PointerKind::Up, ate);
    pinta(h);
}

fn rect(h: &HeroScreen) -> Rect {
    h.store
        .panel_rect(PAINEL)
        .expect("a Timeline publicou o rect")
}

/// Um ponto da alça `id` que o índice resolve para ela (controlo: senão o gate mediria outro alvo).
fn ponto_de(h: &HeroScreen, id: ph2d_editor_core::NodeId) -> (f32, f32) {
    let r = h.hit_index.rect_for(id).expect("a alça foi registada");
    let p = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let p = if id == TIMELINE_CARD_DRAG {
        (r.x + 12.0, p.1)
    } else {
        p
    };
    assert_eq!(
        h.hit_index.hit(p.0, p.1),
        Some(id),
        "o ponto {p:?} não cai em {id:?}"
    );
    p
}

fn hud_y(h: &HeroScreen) -> f32 {
    h.last_layout.expect("layout").bottom_hud.y
}

#[test]
fn o_titulo_arrasta_o_cartao_preso_na_janela_e_o_encaixe_o_devolve() {
    let mut h = so_a_timeline(true);
    let origem = rect(&h);
    let hud_na_origem = hud_y(&h);
    assert!(
        hud_na_origem < origem.y,
        "encaixada, a pílula sobe acima dela"
    );
    assert!(
        !h.store.floating_card_away(PAINEL),
        "controlo: nasce na origem"
    );

    // O «encaixar» na origem não faz nada.
    let dock = ponto_de(&h, TIMELINE_CARD_DOCK);
    evento(&mut h, PointerKind::Down, dock);
    evento(&mut h, PointerKind::Up, dock);
    pinta(&mut h);
    assert_eq!(rect(&h), origem, "na origem o encaixar é inerte");

    // O título arrasta pelo delta.
    let p = ponto_de(&h, TIMELINE_CARD_DRAG);
    arrasta(&mut h, p, (p.0 + 4.0, p.1 - 300.0));
    let r = rect(&h);
    assert!(
        (r.x - (origem.x + 4.0)).abs() < 0.5,
        "x seguiu o delta: {r:?} × {origem:?}"
    );
    assert!(
        (r.y - (origem.y - 300.0)).abs() < 0.5,
        "y seguiu o delta: {r:?} × {origem:?}"
    );
    assert_eq!(
        (r.w, r.h),
        (origem.w, origem.h),
        "arrastar não muda o tamanho"
    );
    assert!(h.store.floating_card_away(PAINEL));
    assert!(
        hud_y(&h) > hud_na_origem,
        "solta, a pílula volta ao lugar dela"
    );

    // Nunca sai da janela.
    let p = ponto_de(&h, TIMELINE_CARD_DRAG);
    arrasta(&mut h, p, (p.0 - 5000.0, p.1 - 5000.0));
    let r = rect(&h);
    let area = h.last_layout.expect("layout").draw_area;
    assert!(
        (r.y - area.y).abs() < 0.5,
        "pára por baixo do cromo do topo, onde o «×» continua alcançável: {r:?} × {area:?}"
    );
    assert!(
        r.x >= VIEWPORT.x && r.y >= VIEWPORT.y,
        "preso na janela: {r:?}"
    );
    assert!(r.x + r.w <= VIEWPORT.x + VIEWPORT.w && r.y + r.h <= VIEWPORT.y + VIEWPORT.h);

    // O encaixar devolve a origem, exacta.
    let dock = ponto_de(&h, TIMELINE_CARD_DOCK);
    evento(&mut h, PointerKind::Down, dock);
    evento(&mut h, PointerKind::Up, dock);
    pinta(&mut h);
    assert_eq!(rect(&h), origem, "encaixada outra vez no lugar de origem");
    assert!(!h.store.floating_card_away(PAINEL));
    solta_os_modos();
}

#[test]
fn o_canto_redimensiona_o_cartao_sem_descer_do_piso() {
    let mut h = so_a_timeline(true);
    let origem = rect(&h);
    let p = ponto_de(&h, TIMELINE_RESIZE_BR);
    arrasta(&mut h, p, (p.0 - 200.0, p.1 - 60.0));
    let r = rect(&h);
    assert!(
        (r.w - (origem.w - 200.0)).abs() < 0.5,
        "a largura seguiu o canto: {r:?}"
    );
    assert!(
        (r.h - (origem.h - 60.0)).abs() < 0.5,
        "a altura seguiu o canto: {r:?}"
    );
    assert_eq!(
        (r.x, r.y),
        (origem.x, origem.y),
        "o canto oposto fica parado"
    );

    let p = ponto_de(&h, TIMELINE_RESIZE_BR);
    arrasta(&mut h, p, (p.0 - 5000.0, p.1 - 5000.0));
    let r = rect(&h);
    assert!(
        r.w >= 320.0 - 0.5 && r.h >= 120.0 - 0.5,
        "abaixo do piso: {r:?}"
    );
    assert_eq!(
        (r.x, r.y),
        (origem.x, origem.y),
        "no piso o canto oposto não anda"
    );
    solta_os_modos();
}

/// No clássico (`PH2D_CARDS=0`) a faixa docada não ganha título que arraste nem «encaixar».
#[test]
fn no_classico_a_timeline_continua_docada_e_sem_as_alcas_do_cartao() {
    let h = so_a_timeline(false);
    assert!(h.hit_index.rect_for(TIMELINE_CARD_DRAG).is_none());
    assert!(h.hit_index.rect_for(TIMELINE_CARD_DOCK).is_none());
    assert!(h.hit_index.rect_for(TIMELINE_RESIZE_BR).is_none());
    assert!(!h.store.is_floating_panel(PAINEL));
    solta_os_modos();
}
