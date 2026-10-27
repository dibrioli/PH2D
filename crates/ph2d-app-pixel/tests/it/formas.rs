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
    PixelToolGroup as G,
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
        Self::cena(10)
    }

    fn cena(n: u32) -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(
            ph2d_app_pixel::smoke::stage(&mut app, &mut sim, n, spawn_root),
            "a cena {n} monta-se"
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

/// P6c: a cena 11 mostra as três correcções — a elipse na mão com o Fill, a cara das selecções é o
/// laço poligonal, o flyout das formas aberto; oco × cheio lado a lado; a selecção poligonal.
#[test]
fn scene_11_shows_each_correction() {
    use ph2d_app_pixel::smoke_ferramentas as onze;
    let mut p = Palco::cena(11);
    let i = p.quadro();
    assert_eq!(i.tool, Tool::Ellipse);
    assert!(i.options.shape_fill, "a caixa Fill ligada no painel");
    assert_eq!(i.groups.face(G::Select), Tool::PolygonLasso);
    assert_eq!(i.groups.face(G::Shape), Tool::Ellipse);
    assert_eq!(i.groups.open, Some(G::Shape));
    assert!(i.has_selection && i.transform.is_none());
    assert!(!Tool::ALL.iter().any(|t| t.label_key() == "pixel.tool.move"));
    let d = p.app.doc(p.key).expect("o sprite");
    let rgba = |x, y| {
        d.sprite
            .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))
    };
    for (tool, a, b, cor, fill) in onze::SHAPES {
        let meio = ((a.0 + b.0) / 2, (a.1 + b.1) / 2);
        assert_eq!(rgba(meio.0, a.1), cor, "{tool:?}: a borda de cima");
        assert_eq!(rgba(meio.0, meio.1) == cor, fill, "{tool:?} fill={fill}");
    }
    // `V`: o Transform na mão, a figura levantada no gizmo.
    let i = p.tecla(KeyCode::KeyV, false, false);
    assert_eq!(i.tool, Tool::Transform);
    assert!(i.transform.is_some(), "o V levanta a selecção");
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
    assert_eq!(i.groups.face(G::Select), Tool::EllipseMarquee);
    assert_eq!(i.groups.open, Some(G::Select));
    let t = i.transform.expect("a figura levantada no gizmo do sprite");
    assert_eq!(t.angle, cena::ANGLE);
    let d = p.app.doc(p.key).expect("o sprite");
    let rgba = |x, y| {
        d.sprite
            .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))
    };
    for (tool, a, b, cor, _) in cena::SHAPES {
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

/// O botão de grupo: pede a última usada do grupo e abre/fecha o flyout; escolher uma (pelo
/// atalho) fecha-o e passa a ser a cara. P6c: as formas têm o seu, com a mesma lei, e só um flyout
/// se abre de cada vez.
#[test]
fn each_group_holds_its_last_tool_and_opens_its_four() {
    let mut p = Palco::novo();
    p.pede(PixelRequest::Tool(Tool::Pencil));
    let i = p.pede(PixelRequest::Group(G::Select));
    assert_eq!(
        (i.tool, i.groups.open),
        (Tool::EllipseMarquee, Some(G::Select))
    );
    let i = p.pede(PixelRequest::Group(G::Select));
    assert_eq!(i.groups.open, None, "o mesmo clique fecha");
    let i = p.tecla(KeyCode::KeyQ, false, true);
    assert_eq!(
        (i.tool, i.groups.face(G::Select), i.groups.open),
        (Tool::PolygonLasso, Tool::PolygonLasso, None)
    );
    let i = p.pede(PixelRequest::Tool(Tool::Pencil));
    assert_eq!(
        i.groups.face(G::Select),
        Tool::PolygonLasso,
        "a cara fica a última"
    );
    // As formas: a estrela da cena é a cara; o U troca-a pelo retângulo.
    assert_eq!(i.groups.face(G::Shape), Tool::Star);
    p.pede(PixelRequest::Group(G::Select));
    let i = p.pede(PixelRequest::Group(G::Shape));
    assert_eq!(
        (i.tool, i.groups.open),
        (Tool::Star, Some(G::Shape)),
        "abrir as formas fecha as selecções"
    );
    let i = p.tecla(KeyCode::KeyU, false, true);
    assert_eq!(
        (i.tool, i.groups.face(G::Shape), i.groups.open),
        (Tool::Ellipse, Tool::Ellipse, None)
    );
    assert_eq!(
        i.groups.face(G::Select),
        Tool::PolygonLasso,
        "cada grupo tem a sua"
    );
}

/// P6c: a caixa «Fill» vale para as quatro formas — o retângulo e a elipse enchem com ela (o que o
/// Retângulo cheio e a Elipse cheia faziam), e sem ela ficam ocas.
#[test]
fn the_fill_box_fills_the_rectangle_and_the_ellipse() {
    // Um canto livre da cena 10 (as formas estão em cima, a figura ao meio).
    for tool in [Tool::Rectangle, Tool::Ellipse] {
        for fill in [false, true] {
            let mut p = Palco::novo();
            p.tecla(KeyCode::Enter, false, false);
            p.pede(PixelRequest::Select(S::Deselect));
            let mut o = p.app.options;
            o.shape_fill = fill;
            o.shapes = Default::default();
            p.pede(PixelRequest::Options(Box::new(o)));
            p.pede(PixelRequest::Tool(tool));
            let c = canvas(&p.sim, &p.cam, p.bits);
            let a = at(&p.app, &p.sim, &p.cam, p.bits, 4, 50);
            let b = at(&p.app, &p.sim, &p.cam, p.bits, 16, 62);
            let _ = input::press(&mut p.app, &c, a, Button::Primary, Mods::default());
            let _ = input::moved(&mut p.app, &c, b, Mods::default());
            let _ = input::release(&mut p.app);
            let d = p.app.doc(p.key).expect("o sprite");
            let cor = |x, y| {
                d.sprite
                    .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))[3]
            };
            assert_ne!(cor(10, 50), 0, "{tool:?}: o contorno");
            assert_eq!(cor(10, 56) == 0, !fill, "{tool:?} fill={fill}");
        }
    }
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

/// A foto da cena 10 (26/10): o 1.º quadro corre ANTES do Draw e a regra «sair do Draw assenta»
/// assentava a figura que a cena levantou — o gizmo nunca aparecia. Só SAIR do Draw assenta.
#[test]
fn the_first_frame_outside_draw_does_not_settle_the_staged_float() {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 10, spawn_root
    ));
    let (bits, _) = mode::sprites(sim.world())[0];
    let mut cam = Camera2d::default();
    app.frame(&mut sim, &mut cam, WINDOW, PPM, None, spawn_root);
    assert!(app.floating(), "o quadro fora do Draw não saiu do Draw");
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    assert!(app.floating());
    app.frame(&mut sim, &mut cam, WINDOW, PPM, None, spawn_root);
    assert!(!app.floating(), "sair do Draw assenta");
}
