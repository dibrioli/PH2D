//! **P6b pelo caminho real** (smoke do dono 26/10): a cena 10, o botão de grupo das selecções, a
//! ferramenta Transform e as formas do Vector em células.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::formas::cells;
use ph2d_app_pixel::input::{self, Button};
use ph2d_app_pixel::smoke_formas as cena;
use ph2d_app_pixel::{PixelApp, Tool, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    PixelPanelInfo, PixelRequest, PixelSelectAction as S, PixelShape, PixelShapeValues,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_core::raster::{Cell, ellipse_cells, ellipse_filled_cells, rounded_rect_cells};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, at, canvas, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    cam: Camera2d,
    hero: HeroScreen,
    toasts: ToastQueue,
    bits: u64,
    key: u32,
}

impl Palco {
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(
            ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 10, spawn_root),
            "a cena 10 monta-se"
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

    fn quadro(&mut self) -> PixelPanelInfo {
        panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        )
    }

    fn pede(&mut self, r: PixelRequest) -> PixelPanelInfo {
        self.app.pede(r);
        self.quadro()
    }

    fn tecla(&mut self, k: KeyCode, ctrl: bool, shift: bool) -> PixelPanelInfo {
        let a = input::key(
            &mut self.app,
            self.sim.world(),
            Some(self.bits),
            k,
            ctrl,
            shift,
        );
        assert!(a.is_some(), "{k:?} é da família");
        self.quadro()
    }
}

fn col(f: impl FnOnce(&mut dyn FnMut(i32, i32))) -> Vec<Cell> {
    let mut v = Vec::new();
    f(&mut |x, y| v.push((x, y)));
    v
}

fn ordena(mut v: Vec<Cell>) -> Vec<Cell> {
    v.sort_unstable();
    v.dedup();
    v
}

#[test]
fn scene_10_shows_each_correction() {
    let mut p = Palco::novo();
    let i = p.quadro();
    assert_eq!(
        i.tool,
        Tool::Star,
        "a estrela na mão: o painel mostra os campos dela"
    );
    assert_eq!((i.marquee, i.marquee_open), (Tool::EllipseMarquee, true));
    let t = i.transform.expect("a figura levantada no gizmo do sprite");
    assert_eq!(t.angle, cena::ANGLE);
    let d = p.app.doc(p.key).expect("o sprite");
    let rgba = |x, y| {
        d.sprite
            .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))
    };
    for (tool, a, b, cor) in cena::SHAPES {
        let pintadas = (a.1..=b.1)
            .flat_map(|y| (a.0..=b.0).map(move |x| (x, y)))
            .filter(|&(x, y)| rgba(x, y) == cor)
            .count();
        assert!(pintadas > 20, "{tool:?}: {pintadas} células na caixa");
    }
}

/// Com os campos neutros, o retângulo e a elipse são as leis medidas do Pixel (o desenho aprovado
/// não muda).
#[test]
fn neutral_fields_keep_the_measured_pixel_laws() {
    let (a, b) = ((2, 3), (17, 12));
    for r in [0_u32, 2, 5] {
        let v = PixelShapeValues::defaults(PixelShape::Rectangle).with(0, f64::from(r));
        for cheia in [false, true] {
            let (o, i) = cells(PixelShape::Rectangle, &v, a, b, cheia);
            assert_eq!(
                ordena(o),
                ordena(col(|cb| rounded_rect_cells(
                    a.0, a.1, b.0, b.1, r, false, cb
                )))
            );
            let quer = if cheia {
                ordena(col(|cb| {
                    rounded_rect_cells(a.0, a.1, b.0, b.1, r, true, cb)
                }))
            } else {
                Vec::new()
            };
            assert_eq!(ordena(i), quer, "r={r} cheia={cheia}");
        }
    }
    let v = PixelShapeValues::defaults(PixelShape::Ellipse);
    let (o, i) = cells(PixelShape::Ellipse, &v, a, b, true);
    assert_eq!(
        ordena(o),
        ordena(col(|cb| ellipse_cells(a.0, a.1, b.0, b.1, cb)))
    );
    assert_eq!(
        ordena(i),
        ordena(col(|cb| ellipse_filled_cells(a.0, a.1, b.0, b.1, cb)))
    );
}

/// Os campos que a lei do Pixel não tem vão pela geometria do Vector: um canto vivo, a pizza, o
/// furo, o hexágono e a estrela — tudo dentro da caixa, a estrela de ponta para CIMA.
#[test]
fn the_vector_fields_draw_in_cells() {
    let (a, b) = ((0, 0), (23, 23));
    let dentro = |v: &[Cell]| {
        v.iter()
            .all(|&(x, y)| (a.0..=b.0).contains(&x) && (a.1..=b.1).contains(&y))
    };
    // Fillet 6 com o canto de cima à direita vivo: o canto (23, 0) é pintado, o (0, 0) não.
    let v = PixelShapeValues::defaults(PixelShape::Rectangle)
        .with(0, 6.0)
        .with(1, -6.0);
    let (o, _) = cells(PixelShape::Rectangle, &v, a, b, false);
    assert!(
        dentro(&o) && o.contains(&(23, 0)) && !o.contains(&(0, 0)),
        "{o:?}"
    );
    // A pizza de 270° abre à direita (o corte vai de −45° a 45°): o miolo à direita do centro fica
    // de fora, o da esquerda dentro.
    let v = PixelShapeValues::defaults(PixelShape::Ellipse)
        .with(0, 270.0)
        .with(1, 45.0);
    let (o, i) = cells(PixelShape::Ellipse, &v, a, b, true);
    assert!(dentro(&o) && dentro(&i));
    assert!(i.contains(&(5, 12)) && !i.contains(&(19, 12)), "{i:?}");
    // O furo: o centro sai.
    let v = PixelShapeValues::defaults(PixelShape::Ellipse).with(2, 0.5);
    let (_, i) = cells(PixelShape::Ellipse, &v, a, b, true);
    assert!(!i.contains(&(12, 12)) && i.contains(&(3, 12)), "{i:?}");
    // O hexágono e a estrela cheios, dentro da caixa; a estrela com uma ponta na linha de cima.
    let v = PixelShapeValues::defaults(PixelShape::Polygon).with(0, 6.0);
    let (o, i) = cells(PixelShape::Polygon, &v, a, b, true);
    assert!(dentro(&o) && dentro(&i) && !i.is_empty());
    let v = PixelShapeValues::defaults(PixelShape::Star);
    let (o, i) = cells(PixelShape::Star, &v, a, b, true);
    assert!(dentro(&o) && !i.is_empty());
    assert!(
        o.iter().any(|&(x, y)| y == 0 && (10..=13).contains(&x)),
        "{o:?}"
    );
    assert!(
        !o.iter().any(|&(x, y)| y == 23 && (10..=13).contains(&x)),
        "{o:?}"
    );
}

/// O botão de grupo: pede a última selecção usada e abre/fecha as quatro; escolher uma (pelo
/// atalho) fecha-as e passa a ser a cara.
#[test]
fn the_selection_group_holds_the_last_marquee_and_opens_the_four() {
    let mut p = Palco::novo();
    p.pede(PixelRequest::Tool(Tool::Pencil));
    let i = p.pede(PixelRequest::MarqueeGroup);
    assert_eq!((i.tool, i.marquee_open), (Tool::EllipseMarquee, true));
    let i = p.pede(PixelRequest::MarqueeGroup);
    assert!(!i.marquee_open, "o mesmo clique fecha");
    let i = p.tecla(KeyCode::KeyQ, false, true);
    assert_eq!(
        (i.tool, i.marquee, i.marquee_open),
        (Tool::PolygonLasso, Tool::PolygonLasso, false)
    );
    let i = p.pede(PixelRequest::Tool(Tool::Pencil));
    assert_eq!(i.marquee, Tool::PolygonLasso, "a cara fica a última");
}

/// A ferramenta Transform: o `Ctrl+T` põe-na na mão; com ela, premir sobre o sprite levanta (sem
/// selecção, o cel inteiro) e o premir segue para o gizmo; o Enter assenta.
#[test]
fn the_transform_tool_lifts_on_press_and_ctrl_t_picks_it() {
    let mut p = Palco::novo();
    p.tecla(KeyCode::Enter, false, false);
    assert!(p.quadro().transform.is_none());
    p.pede(PixelRequest::Select(S::Deselect));
    let i = p.pede(PixelRequest::Tool(Tool::Transform));
    assert!(i.transform.is_some(), "escolher a ferramenta levanta");
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(p.quadro().tool, Tool::Transform);
    assert!(p.quadro().transform.is_none());
    let c = canvas(&p.sim, &p.cam, p.bits);
    let pt = at(&p.app, &p.sim, &p.cam, p.bits, 60, 44);
    let _ = input::press(&mut p.app, &c, pt, Button::Primary, Mods::default());
    let _ = input::release(&mut p.app);
    assert!(
        p.quadro().transform.is_some(),
        "premir com o Transform levanta"
    );
    p.tecla(KeyCode::Enter, false, false);
    p.pede(PixelRequest::Tool(Tool::Pencil));
    let i = p.tecla(KeyCode::KeyT, true, false);
    assert_eq!(i.tool, Tool::Transform);
    assert!(i.transform.is_some());
}
