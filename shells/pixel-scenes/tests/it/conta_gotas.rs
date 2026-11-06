//! **O conta-gotas do seletor partilhado em Draw**, pela porta da shell (`PixelApp::pointer` com a
//! `View` lida do `hero`). Smoke do dono 05/11: armado, o clique pintava e o seletor ficava armado.

use ph2d_app_pixel::PixelApp;
use ph2d_app_pixel::compose::{Mods, Pointer, View};
use ph2d_app_pixel::conta_gotas::entrega;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::ids::INSP_BLENDER_PICKER;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::{ChannelMode, Harmony, InterpolationMode};
use ph2d_host::{PointerButton, PointerKind};
use ph2d_render::Camera2d;
use ph2d_tokens::ColorValue;

use crate::edit::{WINDOW, at, born, canvas, value};

const TINTA: [u8; 4] = [10, 200, 30, 255];
const OUTRA: [u8; 4] = [90, 90, 240, 255];
const SEMENTE: [u8; 4] = [1, 2, 3, 255];
const VAZIO: [u8; 4] = [0, 0, 0, 0];

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    cam: Camera2d,
    bits: u64,
    hero: HeroScreen,
    toasts: ToastQueue,
}

impl Palco {
    /// Um sprite com a célula `(3, 3)` pintada de [`TINTA`], em Draw, com o seletor aberto.
    fn novo() -> Self {
        let (mut app, sim, cam, bits) = born();
        app.fg = TINTA;
        let p = at(&app, &sim, &cam, bits, 3, 3);
        let c = canvas(&sim, &cam, bits);
        let premiu = input::press(&mut app, &c, p, Button::Primary, Mods::default());
        assert_eq!(premiu, Press::Done);
        input::release(&mut app);
        app.fg = OUTRA;
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let [r, g, b, a] = SEMENTE;
        hero.store.register(
            INSP_BLENDER_PICKER,
            InteractiveState::BlenderPicker {
                value: ColorValue::from_rgba8(r, g, b, a),
                channel_mode: ChannelMode::Rgb,
                interpolation: InterpolationMode::Perceptual,
                active_palette: 0,
                hsv_h: 0.0,
                hsv_s: 0.0,
                harmony: Harmony::None,
                picks: 0,
            },
        );
        Self {
            app,
            sim,
            cam,
            bits,
            hero,
            toasts: ToastQueue::new(),
        }
    }

    fn arma(&mut self) {
        (self.hero.store).set_eyedropper_pending(Some(INSP_BLENDER_PICKER));
    }

    fn premir(&mut self, p: (f32, f32)) -> Pointer {
        let Self {
            app,
            sim,
            cam,
            hero,
            toasts,
            ..
        } = self;
        let mut v = View::of(sim.world(), hero, cam, toasts, WINDOW);
        app.pointer(
            &mut v,
            PointerKind::Down,
            PointerButton::Primary,
            p,
            Mods::default(),
            true,
        )
    }

    fn celula(&self, x: i32, y: i32) -> (f32, f32) {
        at(&self.app, &self.sim, &self.cam, self.bits, x, y)
    }

    fn seletor(&self) -> [u8; 4] {
        let (v, ..) = (self.hero.store)
            .blender_picker(INSP_BLENDER_PICKER)
            .expect("o seletor");
        v.rgba
    }
}

/// Armado, o clique lê a cor exacta da célula, não pinta, e o seletor recebe-a e desarma.
#[test]
fn the_armed_click_reads_the_cell_and_does_not_paint() {
    let mut p = Palco::novo();
    p.arma();
    let sobre_a_tinta = p.celula(3, 3);
    let Pointer::Amostra(cor) = p.premir(sobre_a_tinta) else {
        panic!("o clique armado é do conta-gotas");
    };
    assert_eq!(cor, TINTA);
    entrega(&mut p.hero.store, cor);
    assert_eq!(p.seletor(), TINTA);
    assert_eq!(p.hero.store.eyedropper_pending(), None, "desarma");
    assert_eq!(value(&p.app, &p.sim, p.bits, 3, 3), TINTA, "nada pintado");
    assert_eq!(p.app.fg, OUTRA, "a cor da frente só muda pelo seletor");

    // Numa célula vazia nada é pintado, e a cor do seletor fica.
    let mut p = Palco::novo();
    p.arma();
    let vazia = p.celula(5, 5);
    let Pointer::Amostra(cor) = p.premir(vazia) else {
        panic!("dentro do sprite o clique armado é do conta-gotas");
    };
    input::release(&mut p.app);
    assert_eq!(value(&p.app, &p.sim, p.bits, 5, 5), VAZIO, "não pintou");
    entrega(&mut p.hero.store, cor);
    assert_eq!(p.seletor(), SEMENTE, "alfa zero não vira a cor");
    assert_eq!(p.hero.store.eyedropper_pending(), None, "e desarma");

    // Controlo: desarmado, o mesmo clique pinta.
    let mut p = Palco::novo();
    let vazia = p.celula(5, 5);
    assert_eq!(p.premir(vazia), Pointer::Done);
    input::release(&mut p.app);
    assert_eq!(
        value(&p.app, &p.sim, p.bits, 5, 5),
        OUTRA,
        "controlo: pintou"
    );
}

/// Fora do sprite o clique armado segue para o caminho da casa, que lê o ecrã.
#[test]
fn the_armed_click_outside_the_sprite_is_the_screen_pick() {
    let mut p = Palco::novo();
    p.arma();
    let fora = p.celula(-3, -3);
    assert_eq!(p.premir(fora), Pointer::NotMine);
    assert!(
        p.hero.store.eyedropper_pending().is_some(),
        "o da casa desarma"
    );
}
