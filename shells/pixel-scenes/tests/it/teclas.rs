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

/// Sem efeito no Pixel ainda: a janela de atalhos e a grade do app (`Ctrl+'`) não têm porta.
const SEM_PORTA: &[A] = &[A::KeyboardShortcuts, A::ShowGrid];

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
