//! **A cena 13 — o catálogo do Vector no Pixel** (P6e, dono 27/10: *«traga todas as outras shapes
//! presentes em vector para pixel mantendo os mesmos gizmos maravilhosos»*). Um Pixel Sprite RGBA
//! 160 × 110 em Draw, com a ENGRENAGEM na mão:
//!
//! - à esquerda, nove formas JÁ assentadas, uma ou duas por família do catálogo do Vector (as
//!   abertas — espiral, arco, chave — como traço de uma célula, sem interior);
//! - à direita, uma engrenagem cheia AINDA VIVA: a caixa do gizmo do sprite e as alças dela iguais
//!   às do Vector. `Enter` (ou outra ferramenta, ou um clique fora) assenta-a.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::PixelShape;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Stroke};
use ph2d_vec_scene::ShapeKind;

use crate::gesture::{Kind, Mods, paint};
use crate::state::Gesture;
use crate::viva::Viva;
use crate::{PixelApp, Tool};

/// O tamanho da cena 13.
pub const SIZE: (u32, u32) = (160, 110);
/// O lado de cada amostra assentada e o passo da grelha, em células.
const LADO: i32 = 30;
const PASSO: i32 = 36;
/// As amostras assentadas, por linha da grelha 3 × 3: a forma, a cor e se é cheia.
pub const SETTLED: [(ShapeKind, [u8; 4], bool); 9] = [
    (ShapeKind::Spiral, [70, 110, 220, 255], false),
    (ShapeKind::Arc, [70, 110, 220, 255], false),
    (ShapeKind::Pie, [240, 200, 60, 255], true),
    (ShapeKind::ArrowBent, [60, 170, 90, 255], true),
    (ShapeKind::Document, [150, 90, 200, 255], false),
    (ShapeKind::Thought, [230, 230, 230, 255], true),
    (ShapeKind::Heart, [220, 60, 90, 255], true),
    (ShapeKind::IsoCube, [90, 180, 200, 255], false),
    (ShapeKind::Brace, [240, 140, 40, 255], false),
];
/// A engrenagem viva: a caixa (cantos inclusivos) e a cor.
pub const GEAR: ((i32, i32), (i32, i32), [u8; 4]) = ((112, 30), (155, 73), [200, 70, 50, 255]);

/// A caixa da amostra `i` da grelha (cantos inclusivos).
#[must_use]
pub const fn caixa(i: usize) -> ((i32, i32), (i32, i32)) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "índice de uma grelha 3 × 3"
    )]
    let (c, l) = ((i % 3) as i32, (i / 3) as i32);
    let a = (4 + c * PASSO, 4 + l * PASSO);
    (a, (a.0 + LADO - 1, a.1 + LADO - 1))
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
        background: Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    for (i, (k, rgba, fill)) in SETTLED.into_iter().enumerate() {
        let (a, b) = caixa(i);
        app.options.shape_fill = fill;
        let tool = Tool::Shape(PixelShape::new(k));
        crate::smoke_formas::forma(app, key, tool, a, b, rgba)?;
    }
    // A engrenagem na mão, viva: o mesmo estado que largar o arrasto dela deixa.
    let (a, b, rgba) = GEAR;
    let gear = PixelShape::new(ShapeKind::Gear);
    app.options.shape_fill = true;
    crate::input::set_tool(app, Tool::Shape(gear));
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
        viva: Some(Viva::new(gear, a, b)),
    });
    app.reaviva();
    Some(key)
}
