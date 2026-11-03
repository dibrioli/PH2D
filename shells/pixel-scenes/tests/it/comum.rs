//! O mínimo dos palcos de `crates/ph2d-app-pixel/tests/it/{edit,selecao,viva}.rs` de que os testes
//! daqui precisam — a mesma montagem, sem mudar o que afirmam.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Canvas, KeyAnswer, Press};
use ph2d_app_pixel::{PixelApp, Tool, mode, object_add, panel, viva};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    PixelOptions, PixelPanelInfo, PixelRequest, PixelSelectAction,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_pixel_core::Sprite;
use ph2d_render::Camera2d;
use ph2d_tool_vector::shape_handles::handles_of;
use ph2d_tool_vector::shape_handles_face::{Glyph, glyph_of};
use ph2d_vector::Point;
use winit::keyboard::KeyCode;

pub(crate) const WINDOW: WindowSize = WindowSize {
    width: 800,
    height: 600,
};
pub(crate) const PPM: f32 = 100.0;

/// O vazio da casa (`spawn_empty_root`): nome e identidade estável.
pub(crate) fn spawn_root(sim: &mut SimWorld, name: &str) -> u64 {
    let bits = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    bits
}

/// Um Pixel Sprite nascido pelo Add, com a câmera já enquadrada pela entrada em Edit.
pub(crate) fn born() -> (PixelApp, SimWorld, Camera2d, u64) {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    let bits = object_add::add(object_add::PIXEL_SPRITE, &mut sim, &mut app, spawn_root)
        .expect("é desta família")
        .expect("nasce");
    let mut camera = Camera2d::default();
    app.frame(&mut sim, &mut camera, WINDOW, PPM, Some(bits), spawn_root);
    (app, sim, camera, bits)
}

pub(crate) fn canvas<'a>(sim: &'a SimWorld, camera: &'a Camera2d, bits: u64) -> Canvas<'a> {
    Canvas {
        world: sim.world(),
        camera,
        window: WINDOW,
        ppm: PPM,
        editing: Some(bits),
    }
}

/// O ponto de ecrã do CENTRO da célula `(x, y)`.
pub(crate) fn at(
    app: &PixelApp,
    sim: &SimWorld,
    camera: &Camera2d,
    bits: u64,
    x: i32,
    y: i32,
) -> (f32, f32) {
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let m = input::to_screen(
        sim.world(),
        bits,
        app.doc(key).expect("doc"),
        camera,
        WINDOW,
        PPM,
    )
    .expect("afim");
    let p = m * Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
    (p.x as f32, p.y as f32)
}

pub(crate) fn value(app: &PixelApp, sim: &SimWorld, bits: u64, x: i32, y: i32) -> [u8; 4] {
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let d = app.doc(key).expect("doc");
    d.sprite
        .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))
}

/// A cena 9 em Draw com o painel a correr cada quadro (o palco de `selecao.rs`).
pub(crate) struct Palco {
    pub(crate) app: PixelApp,
    pub(crate) sim: SimWorld,
    pub(crate) cam: Camera2d,
    pub(crate) hero: HeroScreen,
    pub(crate) toasts: ToastQueue,
    pub(crate) bits: u64,
    pub(crate) key: u32,
}

impl Palco {
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(
            ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 9, spawn_root),
            "a cena 9 monta-se"
        );
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let mut p = Self {
            app,
            sim,
            cam,
            hero,
            toasts: ToastQueue::new(),
            bits,
            key,
        };
        p.quadro();
        p
    }

    /// A cena 9 sem selecção nem história (um palco limpo para um gesto).
    pub(crate) fn limpo() -> Self {
        let mut p = Self::novo();
        p.pede(PixelRequest::Select(PixelSelectAction::Deselect));
        p
    }

    fn quadro(&mut self) -> PixelPanelInfo {
        panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        )
    }

    pub(crate) fn pede(&mut self, r: PixelRequest) -> PixelPanelInfo {
        self.app.pede(r);
        self.quadro()
    }

    fn tecla_com(&mut self, k: KeyCode, ctrl: bool, shift: bool) -> Option<KeyAnswer> {
        let a = input::key(
            &mut self.app,
            self.sim.world(),
            Some(self.bits),
            k,
            ctrl,
            shift,
        );
        self.quadro();
        a
    }

    pub(crate) fn tecla(&mut self, k: KeyCode, ctrl: bool, shift: bool) {
        assert!(
            self.tecla_com(k, ctrl, shift).is_some(),
            "{k:?} é da família"
        );
    }

    pub(crate) fn sprite(&self) -> &Sprite {
        &self.app.doc(self.key).expect("doc").sprite
    }

    pub(crate) fn rgba(&self, x: i32, y: i32) -> [u8; 4] {
        let d = self.app.doc(self.key).expect("doc");
        d.sprite
            .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))
    }

    pub(crate) fn press(&mut self, c: (i32, i32), m: Mods) {
        let p = at(&self.app, &self.sim, &self.cam, self.bits, c.0, c.1);
        let r = input::press(
            &mut self.app,
            &canvas(&self.sim, &self.cam, self.bits),
            p,
            Button::Primary,
            m,
        );
        assert_eq!(r, Press::Done);
    }

    pub(crate) fn release(&mut self) {
        // (um premir que acabou no próprio premir — a varinha, o clique que assenta — larga em vazio)
        let _ = input::release(&mut self.app);
        self.quadro();
    }

    pub(crate) fn ferramenta(&mut self, t: Tool) {
        self.pede(PixelRequest::Tool(t));
    }

    pub(crate) fn opcoes(&mut self, f: impl FnOnce(&mut PixelOptions)) {
        let mut o = self.app.options;
        f(&mut o);
        self.pede(PixelRequest::Options(Box::new(o)));
    }
}

/// Um Pixel Sprite nascido pelo Add, para a forma VIVA (o palco de `viva.rs`).
pub(crate) struct Mesa {
    pub(crate) app: PixelApp,
    pub(crate) sim: SimWorld,
    pub(crate) cam: Camera2d,
    pub(crate) bits: u64,
    pub(crate) key: u32,
}

impl Mesa {
    pub(crate) fn nova() -> Self {
        let (app, sim, cam, bits) = born();
        let key = input::key_of(sim.world(), bits).expect("marcador");
        Self {
            app,
            sim,
            cam,
            bits,
            key,
        }
    }

    /// A cena `n` do smoke, montada como na shell (o 1.º quadro ainda sem Draw, o seguinte já nele).
    pub(crate) fn cena(n: u32) -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, n, spawn_root
        ));
        let (bits, key) = ph2d_app_pixel::mode::sprites(sim.world())[0];
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, None, spawn_root);
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        Self {
            app,
            sim,
            cam,
            bits,
            key,
        }
    }

    pub(crate) fn celula(&self, c: (i32, i32)) -> (f32, f32) {
        at(&self.app, &self.sim, &self.cam, self.bits, c.0, c.1)
    }

    /// O ponto de ecrã do ponto `(x, y)` da IMAGEM (cantos das células).
    pub(crate) fn ponto(&self, x: f64, y: f64) -> (f32, f32) {
        let m = input::to_screen(
            self.sim.world(),
            self.bits,
            self.app.doc(self.key).expect("doc"),
            &self.cam,
            WINDOW,
            PPM,
        )
        .expect("afim");
        let p = m * Point::new(x, y);
        (p.x as f32, p.y as f32)
    }

    pub(crate) fn press(&mut self, p: (f32, f32)) {
        let c = canvas(&self.sim, &self.cam, self.bits);
        let r = input::press(&mut self.app, &c, p, Button::Primary, Mods::default());
        assert_eq!(r, Press::Done);
    }

    pub(crate) fn arrasta(&mut self, de: (f32, f32), para: (f32, f32)) {
        self.press(de);
        let c = canvas(&self.sim, &self.cam, self.bits);
        assert!(input::moved(&mut self.app, &c, para, Mods::default()));
        assert!(input::release(&mut self.app));
    }

    pub(crate) fn desenha(&mut self, tool: Tool, a: (i32, i32), b: (i32, i32)) {
        self.app.tool = tool;
        let (pa, pb) = (self.celula(a), self.celula(b));
        self.arrasta(pa, pb);
    }

    pub(crate) fn tecla(&mut self, k: KeyCode) {
        let r = input::key(
            &mut self.app,
            self.sim.world(),
            Some(self.bits),
            k,
            false,
            false,
        );
        assert!(r.is_some(), "{k:?} é da família");
    }

    pub(crate) fn v(&self, c: (i32, i32)) -> [u8; 4] {
        value(&self.app, &self.sim, self.bits, c.0, c.1)
    }

    /// A alça da forma viva com a cara `g` (a 1.ª), no ecrã.
    pub(crate) fn alca(&self, g: Glyph) -> (f32, f32) {
        let forma = self.app.live_shape().expect("viva").forma();
        let c = canvas(&self.sim, &self.cam, self.bits);
        let no_ecra = viva::handles_on_screen(&self.app, &c, self.key);
        handles_of(forma.kind())
            .iter()
            .zip(no_ecra)
            .find_map(|(h, p)| (glyph_of(h) == g).then_some(p).flatten())
            .expect("a alça desenha-se")
    }
}
