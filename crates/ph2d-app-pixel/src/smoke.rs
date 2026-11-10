//! **As cenas de smoke** (`PH2D_PIXEL_SMOKE=n`): a lista está em `docs/PixelArt/README.md`.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::{NewSprite, Paint, Stroke};

use crate::PixelApp;
use ph2d_editor_core::pixel_panel::{
    ColorModeChoice, NewSpriteMode, PixelDither, PixelShape, PixelToGray,
};
pub use ph2d_pixel_cenas::basicas::*;
use ph2d_pixel_cenas::heroi::{CENA_26, CENA_27};

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_PIXEL_SMOKE",
    max_level: 36,
}];

/// O nível armado em `PH2D_PIXEL_SMOKE`.
#[must_use]
pub fn armed_level() -> Option<u32> {
    std::env::var(ROUTERS[0].env)
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
}

/// Um Pixel Sprite novo com o rosto ([`FACE`]) pintado; devolve os bits (a cena `=1` e a da ordem,
/// `PH2D_ORDEM_SMOKE`).
pub fn rosto(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    let bits = crate::object_add::spawn(sim, app, NewSprite::default(), spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    FACE.iter()
        .all(|(path, color)| draw_path(app, key, *color, path))
        .then_some(bits)
}

/// O rosto como peça de OUTRA cena (GEF 19, `PH2D_ORDEM_SMOKE`): sem o pedido de Draw com que o
/// objecto nasce, que no quadro seguinte ganharia ao Object que a cena pede.
pub fn rosto_em_object(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    let bits = rosto(app, sim, spawn_root);
    app.wants_edit = None;
    bits
}

/// A cena 23 selecciona o objecto, uma vez: o artista chega ao cartão com um clique, e a cena poupa-o.
pub(crate) fn conduz_o_cartao(
    world: &ph2d_ecs::World,
    hero: &mut ph2d_editor_core::screens::hero::HeroScreen,
) {
    static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if armed_level() != Some(23) || FEITO.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    if let Some((bits, _)) = crate::mode::sprites(world).first() {
        hero.gizmo.replace_selection(Some(*bits));
        FEITO.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Encena o nível `level`. `false` = nível que este roteador não conhece.
pub fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    level: u32,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    match level {
        1 => {
            let feito = rosto(app, sim, spawn_root).is_some();
            app.fg = [0, 0, 0, 255];
            app.bg = [255, 255, 255, 255];
            feito
        }
        2 => stage_house(app, sim, spawn_root),
        3 => stage_ball(app, sim, spawn_root),
        4 => stage_gradient(app, sim, spawn_root).is_some(),
        5 => stage_gradient(app, sim, spawn_root).is_some_and(|_| {
            app.abre_color_mode = Some(PHOTO_DIALOG);
            true
        }),
        6 => stage_gradient(app, sim, spawn_root).is_some_and(|key| {
            let c = ColorModeChoice {
                dither: PixelDither::Ordered4,
                ..PHOTO_DIALOG
            };
            app.converte(key, c);
            app.docs[&key].sprite.mode() == ph2d_pixel_core::ColorMode::Indexed
        }),
        7 => crate::smoke_camadas::stage(app, sim, spawn_root).is_some(),
        8 => crate::smoke_quadros::stage(app, sim, spawn_root).is_some(),
        9 => crate::smoke_selecao::stage(app, sim, spawn_root).is_some(),
        10 => crate::smoke_formas::stage(app, sim, spawn_root).is_some(),
        11 => crate::smoke_ferramentas::stage(app, sim, spawn_root).is_some(),
        12 => crate::smoke_viva::stage(app, sim, spawn_root).is_some(),
        13 => crate::smoke_catalogo::stage(app, sim, spawn_root).is_some(),
        14 => crate::smoke_ajustes::stage(app, sim, spawn_root).is_some(),
        15 => crate::smoke_pontas::stage(app, sim, spawn_root).is_some(),
        16 => crate::smoke_intervalo::stage(app, sim, spawn_root).is_some(),
        17 => crate::smoke_tocar::stage(app, sim, spawn_root).is_some(),
        18 => crate::smoke_pincel::stage(app, sim, spawn_root).is_some(),
        19 => crate::smoke_efeitos::stage(app, sim, spawn_root).is_some(),
        20 => crate::smoke_quadros::stage(app, sim, spawn_root).is_some(),
        21 => crate::smoke_tiles::stage(app, sim, spawn_root).is_some(),
        22 => crate::smoke_tiles::stage_terreno(app, sim, spawn_root).is_some(),
        23 => stage_gradient(app, sim, spawn_root).is_some_and(|_| {
            app.wants_edit = None;
            true
        }),
        24 => crate::smoke_slices::stage(app, sim, spawn_root).is_some(),
        25 => crate::smoke_slices::stage_historia(app, sim, spawn_root).is_some(),
        26 => crate::smoke_slices::stage_heroi(app, sim, spawn_root, &CENA_26).is_some(),
        27 => crate::smoke_slices::stage_heroi(app, sim, spawn_root, &CENA_27).is_some(),
        n @ 28..=36 => p13(app, sim, n, spawn_root).is_some(),
        _ => false,
    }
}

trait Raiz: FnOnce(&mut SimWorld, &str) -> u64 {}
impl<F: FnOnce(&mut SimWorld, &str) -> u64> Raiz for F {}
fn p13(app: &mut PixelApp, sim: &mut SimWorld, n: u32, r: impl Raiz) -> Option<()> {
    let c = ph2d_pixel_cenas::p13::cena(n)?;
    stage(app, sim, c.base, r).then_some(())?;
    let d = app
        .docs
        .get_mut(&crate::mode::sprites(sim.world()).first()?.1)?;
    let pedidos = (c.monta)(&mut d.sprite)?;
    (d.frame, d.layer) = (d.sprite.frames()[0].id, d.sprite.layers()[0].id);
    pedidos.into_iter().for_each(|r| app.pede(r));
    Some(())
}

/// As formas da casa da cena 2: a ferramenta, as células que a definem, a cor (índice da DB32) e
/// a caixa «Fill».
type Peca = (crate::Tool, &'static [(i32, i32)], usize, bool);

pub const HOUSE: [Peca; 6] = [
    (crate::Tool::Line, &[(0, 40), (63, 40)], 12, false),
    (crate::Tool::RECTANGLE, &[(14, 22), (40, 40)], 4, false),
    (
        crate::Tool::Polygon,
        &[(11, 22), (27, 9), (43, 22)],
        27,
        false,
    ),
    (crate::Tool::RECTANGLE, &[(24, 30), (30, 40)], 3, true),
    (crate::Tool::ELLIPSE, &[(50, 3), (59, 12)], 8, true),
    (crate::Tool::Curve, &[(34, 13), (28, 1), (44, 2)], 2, false),
];

fn stage_house(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    let spec = NewSprite {
        width: 64,
        height: 48,
        ..NewSprite::default()
    };
    let Ok(bits) = crate::object_add::spawn(sim, app, spec, spawn_root) else {
        return false;
    };
    let Some(key) = crate::input::key_of(sim.world(), bits) else {
        return false;
    };
    app.options.shapes.set(
        PixelShape::RECTANGLE,
        app.options.shapes.get(PixelShape::RECTANGLE).with(0, 2.0),
    );
    for (tool, pts, color, fill) in HOUSE {
        app.options.shape_fill = fill;
        if !draw_shape(app, key, tool, pts, color) {
            return false;
        }
    }
    app.options.shape_fill = false;
    app.options.shapes.set(
        PixelShape::RECTANGLE,
        app.options.shapes.get(PixelShape::RECTANGLE).with(0, 0.0),
    );
    app.tool = crate::Tool::RECTANGLE;
    app.fg = [0, 0, 0, 255];
    app.bg = [255, 255, 255, 255];
    true
}

/// Uma forma da cena pela MESMA lei do gesto ([`crate::gesture::reshape`]), como UM passo.
fn draw_shape(
    app: &mut PixelApp,
    key: u32,
    tool: crate::Tool,
    pts: &[(i32, i32)],
    color: usize,
) -> bool {
    use crate::gesture::{Kind, Mods, paint, reshape};
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let Some(&rgba) = s.palette().get(color) else {
        return false;
    };
    let kind = match (tool, pts) {
        (crate::Tool::Polygon, _) => Kind::Polygon(pts.to_vec()),
        (crate::Tool::Curve, [a, c, b]) => Kind::Curve {
            from: *a,
            to: *b,
            ctrl: Some(*c),
        },
        (_, [a, b]) => Kind::Box { from: *a, to: *b },
        _ => return false,
    };
    let mut st = Stroke::new(
        doc.layer,
        doc.frame,
        s.value_of_rgba(rgba),
        paint(&app.options, doc, None),
    );
    reshape(&mut st, s, tool, &kind, &app.options, Mods::default());
    let Some(op) = st.op(s) else {
        return false;
    };
    app.commit(key, op).is_ok()
}

/// Um traço pela `path` com a cor `color` da paleta, como UM passo do desfazer.
fn draw_path(app: &mut PixelApp, key: u32, color: usize, path: &[(i32, i32)]) -> bool {
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let Some(&rgba) = s.palette().get(color) else {
        return false;
    };
    let Some(&(x0, y0)) = path.first() else {
        return false;
    };
    let mut st = Stroke::begin(
        s,
        doc.layer,
        doc.frame,
        s.value_of_rgba(rgba),
        Paint::default(),
        (x0, y0),
    );
    for &(x, y) in &path[1..] {
        st.to(s, x, y);
    }
    let Some(op) = st.op(s) else {
        return false;
    };
    app.commit(key, op).is_ok()
}

fn stage_ball(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    let spec = NewSprite {
        width: 64,
        height: 48,
        mode: ph2d_pixel_core::ColorMode::Indexed,
        ..NewSprite::default()
    };
    let Ok(bits) = crate::object_add::spawn(sim, app, spec, spawn_root) else {
        return false;
    };
    let Some(key) = crate::input::key_of(sim.world(), bits) else {
        return false;
    };
    let (w, h) = (64, 48);
    let banda = 36 / SKY.len() as i32;
    let mut pecas: Vec<Vec<((i32, i32), usize)>> = vec![Vec::new(); 3];
    for y in 0..h {
        for x in 0..w {
            if y < 36 {
                pecas[0].push(((x, y), SKY[(y / banda) as usize]));
            } else {
                pecas[1].push(((x, y), GROUND));
            }
            if let Some(k) = ball_step(x, y) {
                pecas[2].push(((x, y), BALL_RAMP[k]));
            }
        }
    }
    for p in &pecas {
        if !draw_cells(app, key, p) {
            return false;
        }
    }
    app.select(BALL_RAMP);
    app.options.ink = ph2d_editor_core::pixel_panel::PixelInk::Shading;
    app.tool = crate::Tool::Pencil;
    app.options.size = 3;
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    app.fg = doc.sprite.palette()[BALL_RAMP[1]];
    app.bg = doc.sprite.palette()[BALL_RAMP[0]];
    true
}

/// Células com os índices dados (um sprite Indexed), como UM passo do desfazer.
fn draw_cells(app: &mut PixelApp, key: u32, cells: &[((i32, i32), usize)]) -> bool {
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let rect = cells
        .iter()
        .fold(ph2d_pixel_core::Rect::default(), |r, &((x, y), _)| {
            r.union(ph2d_pixel_core::Rect::new(x, y, 1, 1))
        });
    let mut pixels = s.read_rect((doc.layer, doc.frame), rect);
    for &((x, y), i) in cells {
        let Ok(v) = u8::try_from(i) else {
            return false;
        };
        pixels[(y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize] = v;
    }
    let bbox = s
        .cel((doc.layer, doc.frame))
        .map_or(rect, |c| c.bbox.union(rect));
    let op = ph2d_pixel_core::Op::Patch(ph2d_pixel_core::Patch {
        layer: doc.layer,
        frame: doc.frame,
        bbox: Some(bbox),
        rect,
        pixels,
    });
    app.commit(key, op).is_ok()
}

/// A escolha com que a cena 5 abre o diálogo.
const PHOTO_DIALOG: ColorModeChoice = ColorModeChoice {
    target: NewSpriteMode::Indexed,
    dither: PixelDither::FloydSteinberg,
    gray: PixelToGray::Luminance,
    metric: ph2d_editor_core::pixel_panel::PixelColorMetric::ExactRgb,
};

pub use ph2d_pixel_cenas::gradiente::{GRADIENT_SIZE, gradient_rgba};

/// A cena 4: a imagem como UM passo, sobre a paleta de fábrica (DB32). Devolve a chave.
fn stage_gradient(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = GRADIENT_SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ph2d_pixel_core::ColorMode::Rgba,
        ..NewSprite::default()
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let doc = app.docs.get(&key)?;
    let rect = ph2d_pixel_core::Rect::new(0, 0, w, h);
    let pixels: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).flat_map(move |x| gradient_rgba(x, y)))
        .collect();
    let op = ph2d_pixel_core::Op::Patch(ph2d_pixel_core::Patch {
        layer: doc.layer,
        frame: doc.frame,
        bbox: Some(rect),
        rect,
        pixels,
    });
    app.commit(key, op).ok()?;
    app.tool = crate::Tool::Pencil;
    app.fg = [255, 255, 255, 255];
    app.bg = [0, 0, 0, 255];
    Some(key)
}
