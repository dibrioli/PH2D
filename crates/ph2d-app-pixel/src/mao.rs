//! A mão do traço livre: a lei está em [`ph2d_pixel_ponte::mao`]; aqui, o gesto em curso.

use ph2d_pixel_core::dynamics::Dab;
pub use ph2d_pixel_ponte::mao::*;

/// Mover com o traço livre e a mão ligada: o ponto que pinta e o carimbo; senão, `p` tal e qual.
pub(crate) fn move_hand(app: &mut crate::PixelApp, p: (f32, f32)) -> ((f32, f32), Option<Dab>) {
    let livre = app
        .gesture
        .as_ref()
        .is_some_and(|g| matches!(g.kind, crate::gesture::Kind::Free));
    match app.mao.as_mut() {
        Some(m) if livre => {
            let (pressure, ns) = tinta();
            let (q, d) = m.ponto(p, pressure, ns);
            (q, Some(d))
        }
        _ => (p, None),
    }
}
