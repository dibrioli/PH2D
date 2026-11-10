//! P13 U-Paleta pelo caminho real: o pedido que o painel faz (New Ramp ▸ Create, Cycle — o clique
//! prova-se em `ph2d-panel-pixel-tools/tests/it/seam_pixel_rampas.rs`) chega ao documento num
//! passo, e o New Palette from Sprite LEVA as rampas.

use ph2d_app_pixel::{PixelApp, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    PixelCycle, PixelRampAction, PixelRampConfig, PixelRequest, PixelSelect,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::Camera2d;

use crate::comum::{PPM, WINDOW, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    key: u32,
}

impl Palco {
    /// A cena 3 (Indexed, DB32) em Draw.
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, 3, spawn_root
        ));
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let mut p = Self {
            app,
            sim,
            hero,
            toasts: ToastQueue::new(),
            key,
        };
        p.pede(PixelRequest::Fg([200, 120, 60, 255]));
        p
    }

    fn pede(&mut self, r: PixelRequest) {
        self.app.pede(r);
        let _ = panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        );
    }

    fn sprite(&self) -> &ph2d_pixel_core::Sprite {
        &self.app.doc(self.key).expect("doc").sprite
    }

    fn passos(&self) -> usize {
        self.app.doc(self.key).expect("doc").history.undo_len()
    }
}

#[test]
fn create_do_new_ramp_poe_a_rampa_no_documento_num_passo() {
    let mut p = Palco::novo();
    let (n, passos) = (p.sprite().palette().len(), p.passos());
    p.pede(PixelRequest::Ramp(PixelRampAction::New(
        PixelRampConfig::default(),
    )));
    let s = p.sprite();
    assert_eq!(s.ramps().ramps.len(), 1, "a rampa entrou no documento");
    let quer: Vec<u8> = (n..n + 5).map(|i| u8::try_from(i).expect("u8")).collect();
    assert_eq!(s.ramps().ramps[0].indices, quer, "as 5 cores novas, no fim");
    assert_eq!(p.passos(), passos + 1, "um passo");
    assert!(p.app.undo(p.key));
    assert!(p.sprite().ramps().ramps.is_empty());
    assert_eq!(p.sprite().palette().len(), n);
}

#[test]
fn ligar_cycle_poe_o_ciclo_no_documento() {
    let mut p = Palco::novo();
    p.pede(PixelRequest::Ramp(PixelRampAction::Make(vec![20, 21, 22])));
    let id = p.sprite().ramps().ramps[0].id;
    let c = PixelCycle::default_for(3);
    p.pede(PixelRequest::Ramp(PixelRampAction::Cycle {
        ramp: id.0,
        cycle: Some(c),
    }));
    let s = p.sprite();
    assert_eq!(s.ramps().cycles.len(), 1, "o ciclo entrou no documento");
    assert_eq!(s.ramps().cycles[0].ramp, id);
    assert_eq!(s.ramps().cycles[0].periodo_ms, c.period_ms);
    let info = panel::frame(&mut p.app, p.sim.world(), &mut p.hero, &mut p.toasts);
    assert_eq!(info.ramps[0].cycle, Some(c), "o painel vê o ciclo");
}

#[test]
fn new_palette_from_sprite_mantem_as_rampas() {
    let mut p = Palco::novo();
    // A rampa nas cores 20..=24 da DB32 (a cena não as usa).
    let antes: Vec<[u8; 4]> = (20..25).map(|i| p.sprite().palette()[i]).collect();
    p.pede(PixelRequest::Ramp(PixelRampAction::Make(
        (20..25).collect(),
    )));
    let id = p.sprite().ramps().ramps[0].id;
    p.pede(PixelRequest::Ramp(PixelRampAction::Cycle {
        ramp: id.0,
        cycle: Some(PixelCycle::default_for(5)),
    }));
    let passos = p.passos();
    p.pede(PixelRequest::PaletteFromSprite);
    assert_eq!(p.passos(), passos + 1, "um passo");
    let s = p.sprite();
    assert!(
        s.palette().len() < 32,
        "a paleta encolheu às usadas ∪ rampa"
    );
    let r = &s.ramps().ramps;
    assert_eq!(r.len(), 1, "a rampa ficou");
    let cores: Vec<[u8; 4]> = r[0]
        .indices
        .iter()
        .map(|&i| s.palette()[usize::from(i)])
        .collect();
    assert_eq!(
        cores, antes,
        "a rampa aponta as MESMAS cores, na mesma ordem"
    );
    assert_eq!(s.ramps().cycles.len(), 1, "o ciclo foi junto");
    let _ = PixelSelect::Only;
}
