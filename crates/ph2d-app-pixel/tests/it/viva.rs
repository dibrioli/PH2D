//! **P6d pelo caminho real**: a forma VIVA — largar o arrasto não grava; as alças do Vector, a caixa
//! do gizmo do sprite e os campos do painel mudam-na na hora; `Enter`, outra ferramenta ou um
//! clique fora assentam-na como UM passo; `Esc` desfaz o arrasto e, depois, larga-a.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::{PixelApp, Tool, viva};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PixelRequest, PixelShape};
use ph2d_render::Camera2d;
use ph2d_tool_vector::shape_handles::handles_of;
use ph2d_tool_vector::shape_handles_face::{Glyph, glyph_of};
use ph2d_vector::Point;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, at, born, canvas, spawn_root, value};

const CLEAR: [u8; 4] = [0, 0, 0, 0];
const WHITE: [u8; 4] = [255, 255, 255, 255];

struct Mesa {
    app: PixelApp,
    sim: SimWorld,
    cam: Camera2d,
    bits: u64,
    key: u32,
}

impl Mesa {
    fn nova() -> Self {
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

    fn cena(n: u32) -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, n, spawn_root
        ));
        let (bits, key) = ph2d_app_pixel::mode::sprites(sim.world())[0];
        let mut cam = Camera2d::default();
        // Como na shell: o 1.º quadro ainda sem Draw, o seguinte já nele.
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

    fn celula(&self, c: (i32, i32)) -> (f32, f32) {
        at(&self.app, &self.sim, &self.cam, self.bits, c.0, c.1)
    }

    /// O ponto de ecrã do ponto `(x, y)` da IMAGEM (cantos das células).
    fn ponto(&self, x: f64, y: f64) -> (f32, f32) {
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

    fn press(&mut self, p: (f32, f32)) {
        let c = canvas(&self.sim, &self.cam, self.bits);
        let r = input::press(&mut self.app, &c, p, Button::Primary, Mods::default());
        assert_eq!(r, Press::Done);
    }

    fn arrasta(&mut self, de: (f32, f32), para: (f32, f32)) {
        self.press(de);
        let c = canvas(&self.sim, &self.cam, self.bits);
        assert!(input::moved(&mut self.app, &c, para, Mods::default()));
        assert!(input::release(&mut self.app));
    }

    fn desenha(&mut self, tool: Tool, a: (i32, i32), b: (i32, i32)) {
        self.app.tool = tool;
        let (pa, pb) = (self.celula(a), self.celula(b));
        self.arrasta(pa, pb);
    }

    fn tecla(&mut self, k: KeyCode) {
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

    fn v(&self, c: (i32, i32)) -> [u8; 4] {
        value(&self.app, &self.sim, self.bits, c.0, c.1)
    }

    /// A alça da forma viva com a cara `g` (a 1.ª), no ecrã.
    fn alca(&self, g: Glyph) -> (f32, f32) {
        let forma = self.app.live_shape().expect("viva").forma();
        let c = canvas(&self.sim, &self.cam, self.bits);
        let no_ecra = viva::handles_on_screen(&self.app, &c, self.key);
        handles_of(forma.kind())
            .iter()
            .zip(no_ecra)
            .find_map(|(h, p)| (glyph_of(h) == g).then_some(p).flatten())
            .expect("a alça desenha-se")
    }

    fn raio(&self) -> f64 {
        self.app.options.shapes.get(PixelShape::Rectangle).get(0)
    }
}

#[test]
fn releasing_a_shape_leaves_it_live_and_enter_settles_it_as_one_step() {
    let mut m = Mesa::nova();
    m.desenha(Tool::Rectangle, (4, 4), (20, 16));
    assert!(
        m.app.live_shape().is_some(),
        "largar não grava: a forma fica viva"
    );
    assert_eq!(m.v((4, 4)), CLEAR, "o documento só muda ao assentar");
    m.tecla(KeyCode::Enter);
    assert!(m.app.live_shape().is_none() && !m.app.stroking());
    for c in [(4, 4), (20, 4), (4, 16), (20, 16), (12, 4)] {
        assert_eq!(m.v(c), WHITE, "{c:?}");
    }
    assert_eq!(m.v((12, 10)), CLEAR, "oco");
    assert!(m.app.undo(m.key));
    assert_eq!(m.v((4, 4)), CLEAR);
    assert!(!m.app.undo(m.key), "um passo só");
}

#[test]
fn a_panel_field_recooks_the_live_shape_at_once() {
    let mut m = Mesa::nova();
    m.desenha(Tool::Rectangle, (4, 4), (20, 16));
    let mut o = m.app.options;
    o.shapes.set(
        PixelShape::Rectangle,
        o.shapes.get(PixelShape::Rectangle).with(0, 4.0),
    );
    o.shape_fill = true;
    m.app.aplica(PixelRequest::Options(Box::new(o)));
    assert!(m.app.live_shape().is_some(), "o campo não assenta a forma");
    m.tecla(KeyCode::Enter);
    assert_eq!(m.v((4, 4)), CLEAR, "o canto ficou redondo");
    assert_eq!(m.v((12, 4)), WHITE, "a borda de cima");
    assert_eq!(m.v((12, 10)), WHITE, "o Fill do painel encheu-a");
}

#[test]
fn the_corner_handle_rounds_the_rectangle_by_the_vector_law_and_a_double_click_resets_it() {
    let mut m = Mesa::nova();
    m.desenha(Tool::Rectangle, (4, 4), (24, 20));
    assert_eq!(m.raio(), 0.0);
    let alca = m.alca(Glyph::Corner);
    let dentro = m.celula((12, 12));
    m.arrasta(alca, dentro);
    let r = m.raio();
    assert!(r > 0.0 && r.fract() == 0.0, "células inteiras de raio: {r}");
    assert!(m.app.live_shape().is_some(), "largar a alça não assenta");
    // Duplo-clique na alça: o campo volta ao de fábrica (o Affinity).
    let alca = m.alca(Glyph::Corner);
    for _ in 0..2 {
        m.press(alca);
        assert!(input::release(&mut m.app));
    }
    assert_eq!(m.raio(), 0.0, "o duplo-clique repõe");
    m.arrasta(m.alca(Glyph::Corner), dentro);
    m.tecla(KeyCode::Enter);
    assert_eq!(m.v((4, 4)), CLEAR, "o canto assentou redondo");
    assert_eq!(m.v((14, 4)), WHITE);
}

#[test]
fn esc_mid_drag_puts_the_field_back_and_esc_again_drops_the_shape() {
    let mut m = Mesa::nova();
    m.desenha(Tool::Rectangle, (4, 4), (24, 20));
    let alca = m.alca(Glyph::Corner);
    m.press(alca);
    let c = canvas(&m.sim, &m.cam, m.bits);
    let dentro = m.celula((12, 12));
    assert!(input::moved(&mut m.app, &c, dentro, Mods::default()));
    assert!(m.raio() > 0.0);
    m.tecla(KeyCode::Escape);
    assert_eq!(m.raio(), 0.0, "o Esc a meio repõe o campo");
    assert!(m.app.live_shape().is_some(), "e a forma continua viva");
    m.tecla(KeyCode::Escape);
    assert!(!m.app.stroking(), "o 2.º Esc larga-a");
    assert_eq!(m.v((4, 4)), CLEAR);
    assert!(!m.app.undo(m.key), "nenhum passo");
}

#[test]
fn the_box_handle_grows_the_shape_without_stretching_its_corner() {
    let mut m = Mesa::nova();
    let r = m.app.options.shapes.get(PixelShape::Rectangle).with(0, 2.0);
    m.app.options.shapes.set(PixelShape::Rectangle, r);
    m.desenha(Tool::Rectangle, (4, 4), (13, 13));
    // O canto ↘ da caixa (a alça de escala do gizmo do sprite) até ao canto da célula (24, 24).
    let (de, para) = (m.ponto(14.0, 14.0), m.ponto(25.0, 25.0));
    m.arrasta(de, para);
    m.tecla(KeyCode::Enter);
    // O retângulo de raio 2 na caixa 4..=24 — nunca o de 4..=13 esticado.
    let (quer, _) =
        ph2d_app_pixel::formas::cells(PixelShape::Rectangle, &r, (4, 4), (24, 24), false);
    let quer: std::collections::BTreeSet<_> = quer.into_iter().collect();
    for y in 0..32 {
        for x in 0..32 {
            let cor = if quer.contains(&(x, y)) { WHITE } else { CLEAR };
            assert_eq!(m.v((x, y)), cor, "({x}, {y})");
        }
    }
}

#[test]
fn another_tool_or_a_click_outside_settles_the_live_shape() {
    let mut m = Mesa::nova();
    m.desenha(Tool::Rectangle, (4, 4), (12, 12));
    m.tecla(KeyCode::KeyB);
    assert!(m.app.live_shape().is_none());
    assert_eq!(m.v((4, 4)), WHITE, "trocar de ferramenta assenta");
    m.desenha(Tool::Rectangle, (16, 4), (24, 12));
    m.desenha(Tool::Rectangle, (4, 20), (12, 28));
    assert_eq!(m.v((16, 4)), WHITE, "o clique fora assentou a anterior");
    assert!(m.app.live_shape().is_some(), "e desenhou a próxima, viva");
    assert_eq!(m.v((4, 20)), CLEAR);
}

#[test]
fn scene_12_shows_a_live_rotated_star_with_its_handles() {
    use ph2d_app_pixel::smoke_viva::{SETTLED, STAR};
    let mut m = Mesa::cena(12);
    assert_eq!(m.app.tool, Tool::Star);
    let viva = m.app.live_shape().expect("a estrela viva");
    assert_eq!(viva.forma(), PixelShape::Star);
    let c = canvas(&m.sim, &m.cam, m.bits);
    let alcas = viva::handles_on_screen(&m.app, &c, m.key);
    assert!(
        alcas.iter().flatten().count() >= 3,
        "as alças da estrela: {alcas:?}"
    );
    let [(_, ra, _, rcor, _), (_, ea, eb, ecor, _)] = SETTLED;
    assert_eq!(m.v((ra.0 + 12, ra.1)), rcor, "o retângulo assentado");
    assert_eq!(
        m.v(((ea.0 + eb.0) / 2, (ea.1 + eb.1) / 2)),
        ecor,
        "a elipse cheia"
    );
    let ((a, b), cor) = ((STAR.0, STAR.1), STAR.2);
    let meio = ((a.0 + b.0) / 2, (a.1 + b.1) / 2);
    assert_eq!(m.v(meio), CLEAR, "a estrela ainda não é pixel");
    m.tecla(KeyCode::Enter);
    assert_eq!(m.v(meio), cor, "assentou cheia");
    // Girada no sentido do relógio: a ponta de cima foge para a DIREITA do meio.
    let topo = (a.1..=b.1)
        .find_map(|y| (a.0..=b.0).find(|&x| m.v((x, y)) == cor).map(|x| (x, y)))
        .expect("a estrela pintou");
    assert!(topo.0 > meio.0 + 3, "a ponta de cima girou: {topo:?}");
}
