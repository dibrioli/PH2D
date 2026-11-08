//! **P12a: as teclas pela tabela** (`docs/PixelArt/P12_teclas.md`): nenhuma tecla do app é do
//! Pixel (dono 08/11, §9.7), e cada linha de fábrica em `Normal` faz a acção dela.

use ph2d_app_pixel::PixelApp;
use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input;
use ph2d_ecs::SimWorld;
use ph2d_pixel_keys::{Chord, Context, FACTORY, PixelAction as A, RESERVED_GLOBAL};

use crate::edit::born;

fn prime(app: &mut PixelApp, sim: &SimWorld, bits: u64, c: Chord) -> bool {
    input::space(app, sim.world(), Some(bits), c.space);
    let m = Mods {
        alt: c.alt,
        ctrl: c.ctrl,
        shift: c.shift,
    };
    input::key_with(app, sim.world(), Some(bits), c.key, m).is_some()
}

#[test]
fn no_app_wide_key_is_taken_by_the_pixel() {
    for &(c, dono) in RESERVED_GLOBAL {
        let (mut app, sim, _cam, bits) = born();
        assert!(!prime(&mut app, &sim, bits, c), "{c} é do app ({dono})");
    }
}

/// Sem efeito no Pixel ainda: a grade do app (`Ctrl+'`) não tem porta.
const SEM_PORTA: &[A] = &[A::ShowGrid];

#[test]
fn every_factory_key_in_normal_does_its_action() {
    let normal = FACTORY
        .iter()
        .filter(|(a, _, x)| *x == Context::Normal && !SEM_PORTA.contains(a));
    for &(a, c, _) in normal {
        let (mut app, sim, _cam, bits) = born();
        let antes = (app.fg, app.bg);
        assert!(
            prime(&mut app, &sim, bits, c),
            "{c} ({a:?}) não foi consumida"
        );
        let feito = (a == A::SwapColors && (app.fg, app.bg) == (antes.1, antes.0))
            || a.tool().is_some_and(|t| app.tool == t)
            || a.request().is_none_or(|r| app.pending().contains(&r));
        assert!(feito, "{c} não fez {a:?}");
    }
}

/// A tecla pela porta REAL (`compose::keyboard`, a que a shell chama), com o sprite em Draw.
fn pela_porta(code: winit::keyboard::KeyCode, m: Mods) -> bool {
    let (mut app, sim, mut cam, bits) = born();
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    let mut v = ph2d_app_pixel::compose::View {
        world: sim.world(),
        camera: &mut cam,
        toasts: &mut toasts,
        window: crate::comum::WINDOW,
        ppm: crate::comum::PPM,
        editing: Some(bits),
        typing: false,
        conta_gotas: false,
    };
    let key = winit::keyboard::PhysicalKey::Code(code);
    app.keyboard(&mut v, key, true, m, None)
}

/// §9.7: o `Ctrl+Space` é o zen do app; o Pixel em Draw não o consome pela porta real.
#[test]
fn ctrl_space_in_draw_is_not_consumed_by_the_pixel() {
    let m = Mods {
        alt: false,
        ctrl: true,
        shift: false,
    };
    assert!(!pela_porta(winit::keyboard::KeyCode::Space, m));
}

/// `Ctrl+Alt+Shift+K` abre a janela KEYBOARD SHORTCUTS (consumida pela porta real).
#[test]
fn ctrl_alt_shift_k_opens_the_shortcuts_window() {
    let m = Mods {
        alt: true,
        ctrl: true,
        shift: true,
    };
    assert!(pela_porta(winit::keyboard::KeyCode::KeyK, m));
}
