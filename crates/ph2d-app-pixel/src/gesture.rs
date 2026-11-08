//! **O gesto de cada ferramenta**: a lei pura mora em [`ph2d_pixel_ponte::gesto`]; aqui fica o que
//! vê o documento e o estado da família.

use ph2d_pixel_core::Paint;
use ph2d_pixel_core::raster::Cell;

pub use ph2d_pixel_ponte::gesto::Mods;
pub(crate) use ph2d_pixel_ponte::gesto::{Kind, boxed, bucket, ink, ink_out, reshape};

use crate::state::{Doc, Options};

/// O pincel, a simetria, o ladrilho e a tinta das opções, para o documento `doc`
/// ([`ph2d_pixel_ponte::gesto::paint`]).
pub(crate) fn paint(
    o: &Options,
    doc: &Doc,
    pincel: Option<&(std::sync::Arc<ph2d_pixel_core::pattern::PatternBrush>, bool)>,
) -> Paint {
    ph2d_pixel_ponte::gesto::paint(o, &doc.sprite, doc.axis2, pincel)
}

/// A simetria das opções, com os eixos que o artista arrastou ([`ph2d_pixel_ponte::gesto::symmetry`]).
pub(crate) fn symmetry(o: &Options, doc: &Doc) -> ph2d_pixel_core::symmetry::Symmetry {
    ph2d_pixel_ponte::gesto::symmetry(o, &doc.sprite, doc.axis2)
}

/// Um premir sobre um gesto que continua com o botão solto: a curva fixa o controlo (o largar
/// acaba-a); o polígono põe um vértice — sobre o PRIMEIRO, fecha. `true` = foi isso.
pub(crate) fn continue_pending(app: &mut crate::PixelApp, key: u32, at: Cell) -> bool {
    let (tool, options) = (app.tool, app.options);
    let stamp = app.stamp();
    let Some(g) = app.gesture.as_mut().filter(|g| g.key == key) else {
        return false;
    };
    match &mut g.kind {
        Kind::Curve { ctrl, .. } if ctrl.is_some() => {
            *ctrl = Some(at);
        }
        Kind::Polygon(pts) => {
            let first = pts[0];
            let len = pts.len();
            if at == first && len > 3 {
                pts.pop();
            } else {
                pts[len - 1] = at;
                pts.push(at);
            }
        }
        _ => return false,
    }
    g.held = true;
    g.cursor = at;
    let closed = matches!(&g.kind, Kind::Polygon(pts) if pts.last() != Some(&at));
    if let Some(doc) = app.docs.get_mut(&key) {
        let kind = g.kind.clone();
        reshape(
            &mut g.stroke,
            &doc.sprite,
            tool,
            &kind,
            &options,
            Mods::default(),
        );
        doc.stamp = stamp;
    }
    if closed {
        crate::input::finish(app);
    }
    true
}
