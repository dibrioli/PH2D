//! **A cena 10 — as quatro correcções do smoke da P6 que SE VÊEM** (P6b, dono 26/10). Um Pixel
//! Sprite RGBA 128 × 80 em Draw, com a ESTRELA na mão (o painel mostra os campos dela):
//!
//! - na coluna: a **varinha** com o ícone de varinha, o **botão das selecções** com o triângulo no
//!   canto e as quatro abertas ao lado (a cara é a Elliptical, a última usada), a ferramenta
//!   **Transform** e as formas novas (polígono e estrela) com o contorno do Vector;
//! - em cima, da esquerda para a direita, as **formas do Vector** em células: um retângulo com
//!   fillet de 6 e o canto de cima à direita vivo, uma pizza (elipse com Sweep 270), um hexágono de
//!   cantos redondos e uma estrela de pontas redondas — cada uma UM passo do desfazer;
//! - em baixo: a figura LEVANTADA no **gizmo do sprite**, a 2× e girada 30° (as alças, o pivô e as zonas de
//!   giro da casa); `Enter` assenta.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PixelShape, PixelTransformField};
use ph2d_pixel_core::selection::{self, Mask, SelectMode};
use ph2d_pixel_core::{ColorMode, NewSprite, Rect, Stroke, edit};

use crate::gesture::{Kind, Mods, paint, reshape};
use crate::{PixelApp, Tool};

/// O tamanho da cena 10.
pub const SIZE: (u32, u32) = (128, 80);
/// Uma forma de cena: a ferramenta, a caixa (cantos inclusivos), a cor e a caixa «Fill».
pub type Forma = (Tool, (i32, i32), (i32, i32), [u8; 4], bool);

/// As quatro formas da cena 10.
pub const SHAPES: [Forma; 4] = [
    (
        Tool::RECTANGLE,
        (4, 4),
        (27, 23),
        [70, 110, 220, 255],
        false,
    ),
    (Tool::ELLIPSE, (34, 4), (55, 25), [240, 200, 60, 255], true),
    (
        Tool::Shape(PixelShape::POLYGON),
        (62, 4),
        (85, 25),
        [60, 170, 90, 255],
        true,
    ),
    (
        Tool::Shape(PixelShape::STAR),
        (92, 3),
        (119, 27),
        [220, 60, 60, 255],
        true,
    ),
];
/// Os valores de cada forma (os campos do catálogo do Vector, em células).
pub const RECT_FILLET: f64 = 6.0;
/// O desvio do canto de cima à direita: −6 = canto vivo.
pub const RECT_TR: f64 = -6.0;
pub const PIE_SWEEP: f64 = 270.0;
pub const PIE_START: f64 = 45.0;
pub const HEX_SIDES: f64 = 6.0;
pub const HEX_ROUND: f64 = 3.0;
pub const STAR_TIP: f64 = 2.0;
/// Onde a figura fica, o tamanho dela no gizmo (2×) e o giro (anti-horário, o campo Angle).
pub const FIGURE_AT: (i32, i32) = (50, 42);
pub const SCALED: i32 = 28;
pub const ANGLE: i32 = 30;

/// Os valores das formas da cena nas opções.
pub fn options(app: &mut PixelApp) {
    let sh = &mut app.options.shapes;
    let r = sh
        .get(PixelShape::RECTANGLE)
        .with(0, RECT_FILLET)
        .with(1, RECT_TR);
    sh.set(PixelShape::RECTANGLE, r);
    let e = sh
        .get(PixelShape::ELLIPSE)
        .with(0, PIE_SWEEP)
        .with(1, PIE_START);
    sh.set(PixelShape::ELLIPSE, e);
    let p = sh
        .get(PixelShape::POLYGON)
        .with(0, HEX_SIDES)
        .with(1, HEX_ROUND);
    sh.set(PixelShape::POLYGON, p);
    let s = sh.get(PixelShape::STAR).with(2, STAR_TIP);
    sh.set(PixelShape::STAR, s);
    app.options.shape_fill = true;
}

pub(crate) fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: ph2d_pixel_core::Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    options(app);
    for (tool, a, b, rgba, fill) in SHAPES {
        app.options.shape_fill = fill;
        forma(app, key, tool, a, b, rgba)?;
    }
    // A figura, seleccionada e levantada no gizmo, girada.
    let fig = crate::smoke_selecao::figure();
    let n = crate::smoke_selecao::FIGURE;
    let doc = app.docs.get(&key)?;
    let f = ph2d_pixel_core::transform::Floating {
        x: FIGURE_AT.0,
        y: FIGURE_AT.1,
        keep: vec![true; (n * n) as usize],
        image: fig,
    };
    let op = edit::paste(&doc.sprite, (doc.layer, doc.frame), &f, 0)?;
    app.commit(key, op).ok()?;
    let m = Mask::from_rect(w, h, Rect::new(FIGURE_AT.0, FIGURE_AT.1, n, n));
    let op = selection::select(&app.docs.get(&key)?.sprite, &m, SelectMode::Replace);
    if let Some(op) = op {
        app.commit(key, op).ok()?;
    }
    app.chave = Some(key);
    if !app.levanta(key) {
        return None;
    }
    app.campo_da_moldura(PixelTransformField::W, SCALED);
    app.campo_da_moldura(PixelTransformField::H, SCALED);
    app.campo_da_moldura(PixelTransformField::Angle, ANGLE);
    // A estrela na mão; o botão das selecções mostra a Elliptical e as quatro abertas.
    app.tool = Tool::Shape(PixelShape::STAR);
    app.groups.chose(Tool::EllipseMarquee);
    app.groups.chose(Tool::Shape(PixelShape::STAR));
    app.groups.open = Some(ph2d_editor_core::pixel_panel::PixelToolGroup::Select);
    Some(key)
}

/// Uma forma da cena pela MESMA lei do gesto, como UM passo.
pub(crate) fn forma(
    app: &mut PixelApp,
    key: u32,
    tool: Tool,
    a: (i32, i32),
    b: (i32, i32),
    rgba: [u8; 4],
) -> Option<()> {
    let doc = app.docs.get(&key)?;
    let s = &doc.sprite;
    let mut st = Stroke::new(
        doc.layer,
        doc.frame,
        s.value_of_rgba(rgba),
        paint(&app.options, doc),
    );
    let kind = Kind::Box { from: a, to: b };
    reshape(&mut st, s, tool, &kind, &app.options, Mods::default());
    let op = st.op(s)?;
    app.commit(key, op).ok()
}
