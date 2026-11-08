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

/// Sem pedido para conferir aqui: as novas da P12a têm prova própria abaixo (`as_teclas_novas_*`).
const SEM_PORTA: &[A] = &[];

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

const NADA: Mods = Mods {
    alt: false,
    ctrl: false,
    shift: false,
};
const CTRL: Mods = Mods {
    alt: false,
    ctrl: true,
    shift: false,
};
const SHIFT: Mods = Mods {
    alt: false,
    ctrl: false,
    shift: true,
};

/// A cena 9 em Draw (o painel corre cada quadro) e a porta real ([`PixelApp::keyboard`]).
struct Porta(crate::comum::Palco);

impl std::ops::Deref for Porta {
    type Target = crate::comum::Palco;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Porta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Porta {
    fn nova() -> Self {
        Self(crate::comum::Palco::limpo())
    }

    fn tecla(&mut self, code: winit::keyboard::KeyCode, m: Mods) {
        let p = &mut self.0;
        let mut v = ph2d_app_pixel::compose::View {
            world: p.sim.world(),
            camera: &mut p.cam,
            toasts: &mut p.toasts,
            window: crate::comum::WINDOW,
            ppm: crate::comum::PPM,
            editing: Some(p.bits),
            typing: false,
            conta_gotas: false,
        };
        let key = winit::keyboard::PhysicalKey::Code(code);
        assert!(p.app.keyboard(&mut v, key, true, m, None), "{code:?} {m:?}");
        p.quadro();
    }

    fn doc(&self) -> &ph2d_app_pixel::Doc {
        self.app.doc(self.key).expect("doc")
    }
}

/// `1`..`6` = 100…3200 %, `` ` `` cai no 100 % (o Draw não desce de uma célula por píxel),
/// `Ctrl+=` / `Ctrl+Shift+=` aproximam e `Ctrl+-` afasta, um degrau da escada.
#[test]
fn as_teclas_novas_do_zoom() {
    use winit::keyboard::KeyCode as K;
    let mut p = Porta::nova();
    let h = |p: &mut Porta, k, m| {
        p.tecla(k, m);
        p.cam.height_world
    };
    let h1 = h(&mut p, K::Digit1, NADA);
    for (k, x) in [
        (K::Digit2, 2.0),
        (K::Digit3, 4.0),
        (K::Digit4, 8.0),
        (K::Digit5, 16.0),
    ] {
        let r = h1 / h(&mut p, k, NADA);
        assert!((r - x).abs() < 0.01, "{k:?}: {r}");
    }
    let r = h1 / h(&mut p, K::Digit6, NADA);
    assert!((r - 32.0).abs() < 0.05, "6: {r}");
    let r = h1 / h(&mut p, K::Backquote, NADA);
    assert!((r - 1.0).abs() < 0.01, "`: {r}");
    let r = h1 / h(&mut p, K::Equal, CTRL);
    assert!((r - 2.0).abs() < 0.01, "Ctrl+=: {r}");
    let ctrl_shift = Mods {
        shift: true,
        ..CTRL
    };
    let r = h1 / h(&mut p, K::Equal, ctrl_shift);
    assert!((r - 3.0).abs() < 0.01, "Ctrl+Shift+=: {r}");
    let r = h1 / h(&mut p, K::Minus, CTRL);
    assert!((r - 2.0).abs() < 0.01, "Ctrl+-: {r}");
}

/// `-`/`+` (e os do teclado numérico) mudam o tamanho do pincel em 1.
#[test]
fn as_teclas_novas_do_pincel() {
    use winit::keyboard::KeyCode as K;
    let mut p = Porta::nova();
    let s0 = p.app.options.size;
    for (k, m) in [(K::Equal, SHIFT), (K::NumpadAdd, NADA)] {
        p.tecla(k, m);
    }
    assert_eq!(p.app.options.size, s0 + 2);
    for k in [K::Minus, K::NumpadSubtract] {
        p.tecla(k, NADA);
    }
    assert_eq!(p.app.options.size, s0);
}

/// `]`/`0` = a cor seguinte da paleta, `[`/`9` = a anterior (dá a volta).
#[test]
fn as_teclas_novas_da_paleta() {
    use winit::keyboard::KeyCode as K;
    let mut p = Porta::nova();
    let pal = p.doc().sprite.palette().to_vec();
    assert!(pal.len() >= 3, "{}", pal.len());
    p.app.fg = pal[1];
    for (k, i) in [
        (K::BracketRight, 2),
        (K::Digit0, 3 % pal.len()),
        (K::BracketLeft, 2),
        (K::Digit9, 1),
    ] {
        p.tecla(k, NADA);
        assert_eq!(p.app.fg, pal[i], "{k:?}");
    }
}

/// `↑`/`↓` = a camada de cima / de baixo.
#[test]
fn as_teclas_novas_da_camada() {
    use ph2d_editor_core::pixel_panel::{PixelLayerAction, PixelRequest};
    use winit::keyboard::KeyCode as K;
    let mut p = Porta::nova();
    let baixo = p.doc().layer;
    p.pede(PixelRequest::LayerAction(PixelLayerAction::NewLayer));
    let cima = p.doc().layer;
    assert_ne!(baixo, cima);
    p.tecla(K::ArrowDown, NADA);
    assert_eq!(p.doc().layer, baixo);
    p.tecla(K::ArrowUp, NADA);
    assert_eq!(p.doc().layer, cima);
}

/// `,`/`.` andam os quadros e dão a volta (sem tag, a linha inteira).
#[test]
fn as_teclas_novas_dos_quadros() {
    use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelRequest};
    use winit::keyboard::KeyCode as K;
    let mut p = Porta::nova();
    for _ in 0..2 {
        p.pede(PixelRequest::FrameAction(PixelFrameAction::NewFrame));
    }
    let i = |p: &Porta| p.doc().sprite.frame_index(p.doc().frame).expect("quadro");
    let n = p.doc().sprite.frames().len();
    assert!(n >= 3, "{n}");
    let a = i(&p);
    p.tecla(K::Period, NADA);
    assert_eq!(i(&p), (a + 1) % n);
    p.tecla(K::Comma, NADA);
    p.tecla(K::Comma, NADA);
    assert_eq!(i(&p), (a + n - 1) % n);
}

/// `Ctrl+'` = a grade de píxeis do canvas (a mesma do `Ctrl+Shift+'`).
#[test]
fn as_teclas_novas_da_grade() {
    use winit::keyboard::KeyCode as K;
    let mut p = Porta::nova();
    let antes = p.app.grid;
    p.tecla(K::Quote, CTRL);
    assert_eq!(p.app.grid, !antes);
}

/// `F2` com a Timeline fechada abre-a (o campo do nome da camada é dela, o que o artista vê).
#[test]
fn f2_opens_the_timeline_to_rename_the_layer() {
    use ph2d_editor_core::screens::hero::PANEL_PIXEL_TIMELINE as T;
    let mut p = Porta::nova();
    p.hero.panel_visibility.insert(T, false);
    p.tecla(winit::keyboard::KeyCode::F2, NADA);
    assert_eq!(p.hero.panel_visibility.get(T).copied(), Some(true));
}
