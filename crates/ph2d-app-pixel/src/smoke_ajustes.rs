//! **A cena 14 — a forma viva como no Vector, até ao fim** (P6f, os abertos da P6d §6). Um Pixel
//! Sprite RGBA 128 × 80 em Draw, com a ESTRELA na mão:
//!
//! - à esquerda, um retângulo azul JÁ assentado (o passo anterior do documento);
//! - à direita, uma estrela cheia VIVA girada 15° (a barra da moldura no painel diz `Angle 15`), com
//!   a alça das PONTAS quente e a etiqueta do Vector ao lado (`Points: 7`). Ela nasceu com 5 pontas
//!   e um ajuste levou-a a 7: o 1.º `Ctrl+Z` volta às 5, o 2.º larga-a, `Ctrl+Shift+Z` refaz.
//! - Gravar (`Ctrl+S`) assenta-a antes: reabrir o projecto mostra-a em células.
//!
//! [`ENV_AGARRA`] (só para a FOTO, que não manda cliques): a alça das pontas fica agarrada, como a
//! meio de um arrasto — a caixa só com o contorno.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::PixelShape;
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Stroke};
use ph2d_tool_vector::shape_handles::handles_of;
use ph2d_tool_vector::shape_handles_face::{Glyph, glyph_of};

use crate::gesture::{Kind, Mods, paint};
use crate::state::Gesture;
use crate::viva::Viva;
use crate::{PixelApp, Tool};

/// O tamanho da cena 14.
pub const SIZE: (u32, u32) = (128, 80);
/// O retângulo assentado: a caixa (cantos inclusivos) e a cor.
pub const SETTLED: ((i32, i32), (i32, i32), [u8; 4]) = ((6, 10), (39, 69), [70, 110, 220, 255]);
/// A estrela viva: a caixa, a cor e o giro (graus, horário na tela: a barra mostra o oposto).
pub const STAR: ((i32, i32), (i32, i32), [u8; 4], f64) =
    ((58, 8), (117, 67), [240, 200, 60, 255], -15.0);
/// As pontas da estrela ao nascer e depois do ajuste guardado.
pub const POINTS: (f64, f64) = (5.0, 7.0);
/// Só para a foto: a alça das pontas agarrada, como a meio de um arrasto.
pub const ENV_AGARRA: &str = "PH2D_PIXEL_SMOKE_AGARRA";

/// A alça das PONTAS da estrela (a da cara de contagem).
#[must_use]
pub fn points_handle() -> Option<usize> {
    handles_of(PixelShape::STAR.kind())
        .iter()
        .position(|h| glyph_of(h) == Glyph::Count)
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
    let (a, b, rgba) = SETTLED;
    app.options.shape_fill = true;
    crate::smoke_formas::forma(app, key, Tool::RECTANGLE, a, b, rgba)?;
    // A estrela na mão, viva, com 5 pontas: o mesmo estado que largar o arrasto dela deixa.
    let (a, b, rgba, giro) = STAR;
    let star = PixelShape::STAR;
    let k = points_handle()?;
    let campo = *handles_of(star.kind()).get(k)?.fields.first()?;
    let com = |app: &PixelApp, n: f64| {
        let mut o = app.options;
        o.shapes.set(star, o.shapes.get(star).with(campo, n));
        o
    };
    app.options = com(app, POINTS.0);
    crate::input::set_tool(app, Tool::Shape(star));
    let doc = app.docs.get(&key)?;
    let stroke = Stroke::new(
        doc.layer,
        doc.frame,
        doc.sprite.value_of_rgba(rgba),
        paint(&app.options, doc),
    );
    let mut viva = Viva::new(star, a, b).girada(giro);
    viva.mostra = Some(k);
    app.gesture = Some(Gesture {
        entity: bits,
        key,
        stroke,
        kind: Kind::Box { from: a, to: b },
        held: false,
        cursor: b,
        mods: Mods::default(),
        viva: Some(viva),
    });
    app.reaviva();
    // O ajuste guardado (o 1.º Ctrl+Z desfá-lo): as 7 pontas.
    let o = com(app, POINTS.1);
    crate::viva::ajuste(app, o);
    if std::env::var_os(ENV_AGARRA).is_some() {
        crate::viva::agarra_para_a_foto(app, k)?;
    }
    Some(key)
}
