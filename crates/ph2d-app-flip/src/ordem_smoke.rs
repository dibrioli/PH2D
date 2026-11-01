//! O traço do Flip da cena da ordem (`PH2D_ORDEM_SMOKE`, GEF 19): uma onda grossa vermelho-alaranjada
//! no 1.º quadro da 1.ª camada do objecto — larga o bastante para se ver por cima e por baixo das
//! outras peças.

use ph2d_core::Vec2;
use ph2d_flip::{FlipDoc, FlipObjectId, FlipStroke, Hold, KeyKind, Point, Rgba};

/// A cor do traço (linear).
const COR: [f32; 4] = [0.9, 0.18, 0.05, 1.0];

/// Põe a onda no objecto `oid`; `false` se o objecto não tem camada ou o quadro 0 já existe.
pub fn traco(doc: &mut FlipDoc, oid: FlipObjectId) -> bool {
    let Some(obj) = doc.object_mut(oid) else {
        return false;
    };
    let Some(camada) = obj.layers().first().map(|l| l.id) else {
        return false;
    };
    let Some(desenho) = obj.insert_frame(camada, 0, Hold::Implicit, KeyKind::Keyframe) else {
        return false;
    };
    let Some(d) = obj.drawing_mut(desenho) else {
        return false;
    };
    let mut s = FlipStroke::new();
    const N: usize = 40;
    for i in 0..N {
        let t = i as f32 / (N - 1) as f32;
        s.push_point(Point {
            pos: Vec2::new(-1.4 + 2.8 * t, 0.45 * (std::f32::consts::TAU * t).sin()),
            width: 0.28,
            opacity: 1.0,
            color: Rgba(COR),
        });
    }
    s.hardness = 1.0;
    d.strokes.push(s);
    true
}
