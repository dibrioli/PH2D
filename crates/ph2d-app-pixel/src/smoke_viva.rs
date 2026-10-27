//! **A cena 12 — a forma VIVA** (P6d, dono 27/10: *«use os mesmos gizmos do vector»*). Um Pixel
//! Sprite RGBA 128 × 80 em Draw, com a ESTRELA na mão:
//!
//! - à esquerda, duas formas JÁ assentadas (pixels): um retângulo de canto redondo e uma elipse
//!   cheia — cada uma UM passo do desfazer;
//! - à direita, uma estrela cheia AINDA VIVA, girada: a caixa do gizmo do sprite (escala, giro, pivô)
//!   e, por cima, as alças da estrela iguais às do Vector (o ⇕ das pontas, o ○ do raio de dentro, o
//!   ⊙ do arredondamento). Mexer numa alça, na caixa ou num campo do painel muda-a na hora; `Enter`
//!   (ou outra ferramenta, ou um clique fora) assenta-a em células.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::PixelShape;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Stroke};

use crate::gesture::{Kind, Mods, paint};
use crate::state::Gesture;
use crate::viva::Viva;
use crate::{PixelApp, Tool};

/// O tamanho da cena 12.
pub const SIZE: (u32, u32) = (128, 80);
/// As duas formas assentadas.
pub const SETTLED: [crate::smoke_formas::Forma; 2] = [
    (
        Tool::Rectangle,
        (4, 6),
        (35, 33),
        [70, 110, 220, 255],
        false,
    ),
    (Tool::Ellipse, (6, 42), (35, 73), [240, 200, 60, 255], true),
];
/// O raio do canto do retângulo assentado, em células.
pub const CORNER: f64 = 6.0;
/// A estrela viva: a caixa (cantos inclusivos), a cor e o giro (graus, horário na tela).
pub const STAR: ((i32, i32), (i32, i32), [u8; 4], f64) =
    ((58, 8), (117, 67), [220, 60, 60, 255], 18.0);

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
        background: Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let r = app
        .options
        .shapes
        .get(PixelShape::Rectangle)
        .with(0, CORNER);
    app.options.shapes.set(PixelShape::Rectangle, r);
    for (tool, a, b, rgba, fill) in SETTLED {
        app.options.shape_fill = fill;
        crate::smoke_formas::forma(app, key, tool, a, b, rgba)?;
    }
    // A estrela na mão, viva: o mesmo estado que largar o arrasto dela deixa.
    let (a, b, rgba, giro) = STAR;
    app.options.shape_fill = true;
    crate::input::set_tool(app, Tool::Star);
    let doc = app.docs.get(&key)?;
    let stroke = Stroke::new(
        doc.layer,
        doc.frame,
        doc.sprite.value_of_rgba(rgba),
        paint(&app.options, doc),
    );
    app.gesture = Some(Gesture {
        entity: bits,
        key,
        stroke,
        kind: Kind::Box { from: a, to: b },
        held: false,
        cursor: b,
        mods: Mods::default(),
        viva: Some(Viva::new(PixelShape::Star, a, b).girada(giro)),
    });
    app.reaviva();
    Some(key)
}
