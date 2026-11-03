//! A MÃO dos testes de cartões no hero montado: pintar quadros, ponteiro real pelo dispatch,
//! arrastar, clicar e soltar um cartão no canvas. Genérica: nada aqui conhece um painel.

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

pub(super) fn pinta(h: &mut HeroScreen, n: usize) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..n {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

pub(super) fn mao(h: &mut HeroScreen, kind: PointerKind, (x, y): (f32, f32)) {
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

/// Agarra `de`, anda até `ate` em passos (pintando entre eles, como o app) e larga.
pub(super) fn arrasta(h: &mut HeroScreen, de: (f32, f32), ate: (f32, f32)) {
    mao(h, PointerKind::Down, de);
    pinta(h, 2);
    for i in 1..=8 {
        let t = i as f32 / 8.0;
        mao(
            h,
            PointerKind::Move,
            (de.0 + (ate.0 - de.0) * t, de.1 + (ate.1 - de.1) * t),
        );
        pinta(h, 2);
    }
    mao(h, PointerKind::Up, ate);
    pinta(h, 30);
}

pub(super) fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

pub(super) fn clica(h: &mut HeroScreen, id: ph2d_editor_core::NodeId) {
    let r = h.hit_index.rect_for(id).expect("o botão está pintado");
    mao(h, PointerKind::Down, centro(r));
    mao(h, PointerKind::Up, centro(r));
    pinta(h, 3);
}

/// Arrasta o cartão `secao` pelo título até ao canvas: ele fica SOLTO.
pub(super) fn solta(h: &mut HeroScreen, secao: ph2d_editor_core::NodeId) {
    let pega = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(secao))
        .expect("o título do cartão");
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    arrasta(
        h,
        (pega.x + 12.0, pega.y + pega.h * 0.5),
        (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.3),
    );
    assert!(
        h.store.loose_card(secao).is_some(),
        "o cartão não ficou solto"
    );
}
