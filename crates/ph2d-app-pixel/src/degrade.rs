//! **As ferramentas Gradient e Blur/Jumble** (P8, `P8_plano.md` §2.1 e §2.7): o gesto só liga o
//! núcleo ([`ph2d_pixel_core::gradient`], [`ph2d_pixel_core::filter::smudge`]). Os valores prontos
//! viram o [`Stroke::with_values`] do gesto (a pré-visualização) e o largar grava UM passo pelo
//! `finish` de sempre; `Esc` larga sem gravar.

use ph2d_pixel_core::brush::spray_seed;
use ph2d_pixel_core::filter::smudge::{self, Smudge};
use ph2d_pixel_core::gradient::{Gradient, Shape, palette_ramp};
use ph2d_pixel_core::raster::{Cell, line_cells};
use ph2d_pixel_core::{Paint, Sprite, Stroke};

use crate::gesture::{Kind, Mods};
use crate::input::Press;
use crate::state::Gesture;
use crate::{PixelApp, Tool};
pub use ph2d_pixel_ponte::cor::dither;
use ph2d_pixel_ponte::cor::entrada;

/// O Blur/Jumble em curso: o estado de trabalho, o pincel e o último ponto.
pub(crate) struct Mancha {
    smudge: Smudge,
    paint: Paint,
    last: Cell,
}

/// Os valores do gradiente de `from` a `to` com as opções da mão, sobre a região inteira
/// ([`ph2d_pixel_core::gradient::region`]: a selecção, ou o cel sem selecção).
fn gradiente(app: &PixelApp, s: &Sprite, from: Cell, to: Cell) -> Vec<(Cell, u32)> {
    let o = &app.options;
    let g = Gradient {
        shape: if o.gradient_radial {
            Shape::Radial
        } else {
            Shape::Linear
        },
        from,
        to,
        fg: app.fg,
        bg: app.bg,
        dither: dither(o.gradient_dither),
    };
    if o.gradient_ramp {
        let pal = s.palette();
        let ramp = palette_ramp(pal, entrada(pal, app.fg), entrada(pal, app.bg));
        g.ramp_values(s, &ramp)
    } else {
        g.values(s)
    }
}

/// As células de um carimbo em `at`: o MESMO funil do lápis (pincel, simetria, ladrilho, selecção).
fn carimbo(s: &Sprite, key: ph2d_pixel_core::CelKey, paint: &Paint, at: Cell) -> Vec<Cell> {
    let mut st = Stroke::new(key.0, key.1, 0, paint.clone());
    st.start(s, at.0, at.1);
    st.cells().collect()
}

/// O premir do Gradient, do Blur e do Jumble; `None` = a ferramenta é outra.
pub(crate) fn press(
    app: &mut PixelApp,
    tool: Tool,
    entity: u64,
    key: u32,
    at: Cell,
    mods: Mods,
) -> Option<Press> {
    if !matches!(
        tool,
        Tool::Gradient | Tool::Blur | Tool::Jumble | Tool::Push
    ) {
        return None;
    }
    app.mancha = None;
    let stamp = app.stamp();
    let pincel = app.pincel.clone().map(|p| (p, app.pincel_imagem));
    let doc = app.docs.get(&key)?;
    let (s, ck) = (&doc.sprite, (doc.layer, doc.frame));
    let (stroke, kind) = if tool == Tool::Gradient {
        let v = gradiente(app, s, at, at);
        (
            Stroke::with_values(ck.0, ck.1, v),
            Kind::Gradient { from: at },
        )
    } else {
        let paint = crate::gesture::paint(&app.options, doc, pincel.as_ref());
        let k = match tool {
            Tool::Blur => smudge::Kind::Blur,
            Tool::Jumble => smudge::Kind::Jumble,
            _ => smudge::Kind::Push {
                forca: app.options.push_strength,
            },
        };
        // A semente é do gesto: a célula do premir (HR-5, nada de relógio).
        let mut smudge = Smudge::new(
            s,
            ck,
            k,
            spray_seed(u32::from(tool == Tool::Jumble), at.0, at.1),
        );
        smudge.arrasta(s, at, &carimbo(s, ck, &paint, at));
        let st = Stroke::with_values(ck.0, ck.1, smudge.values(s));
        app.mancha = Some(Mancha {
            smudge,
            paint,
            last: at,
        });
        (st, Kind::Smudge)
    };
    if let Some(d) = app.docs.get_mut(&key) {
        d.stamp = stamp;
    }
    app.gesture = Some(Gesture {
        entity,
        key,
        stroke,
        kind,
        held: true,
        cursor: at,
        mods,
        viva: None,
    });
    Some(Press::Done)
}

/// O arrasto: o Gradient refaz-se de A ao cursor; o Blur/Jumble carimba cada célula do caminho.
/// `false` = o gesto não é deles.
pub(crate) fn moved(app: &mut PixelApp, key: u32, at: Cell) -> bool {
    let Some(kind) = app.gesture.as_ref().map(|g| g.kind.clone()) else {
        return false;
    };
    let stamp = app.stamp();
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let (s, ck) = (&doc.sprite, (doc.layer, doc.frame));
    let values = match kind {
        Kind::Gradient { from } => gradiente(app, s, from, at),
        Kind::Smudge => {
            let Some(m) = app.mancha.as_mut() else {
                return true;
            };
            if m.last == at {
                return true;
            }
            let mut caminho = Vec::new();
            line_cells(m.last.0, m.last.1, at.0, at.1, |x, y| caminho.push((x, y)));
            for p in caminho.into_iter().skip(1) {
                let cells = carimbo(s, ck, &m.paint, p);
                m.smudge.arrasta(s, p, &cells);
            }
            m.last = at;
            m.smudge.values(s)
        }
        _ => return false,
    };
    let stroke = Stroke::with_values(ck.0, ck.1, values);
    if let Some(g) = app.gesture.as_mut() {
        g.stroke = stroke;
        g.cursor = at;
    }
    if let Some(d) = app.docs.get_mut(&key) {
        d.stamp = stamp;
    }
    true
}
