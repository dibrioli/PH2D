//! **As lanes da água de UM traço guardado** (W16, [`crate::tool::paint::tracos`]): a sessão da água é
//! partilhada pelos traços abertos, a cadeia de dabs de cada um não.
//!
//! O índice de uma lane é o do rasto no MOTOR (`begin_direct_stroke(i, …)`), e o motor é um só: dois
//! traços com a lane `0` escreveriam no mesmo rasto (a 1.ª pincelada passava a pintar com a cor da 2.ª).
//! Cada troca iguala o comprimento dos dois vectores com lanes MORTAS (nunca casam: a posição é infinita),
//! e o vector que está na sessão é sempre o mais comprido — uma lane nasce no fim dele, num índice que
//! nenhum outro traço usa.

use super::session::Lane;
use crate::tool::PainterTool;

#[derive(Clone, Default)]
pub(in crate::tool::paint) struct LanesGuardadas(Vec<Lane>);

const MORTA: Lane = Lane {
    pos: [f32::INFINITY; 2],
    ink: [0.0; 3],
    arco: 0.0,
    copia: 0,
};

impl PainterTool {
    pub(in crate::tool::paint) fn wet_troca_as_lanes(&mut self, g: &mut LanesGuardadas) {
        let Some(s) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        let n = s.lanes.len().max(g.0.len());
        s.lanes.resize(n, MORTA);
        g.0.resize(n, MORTA);
        std::mem::swap(&mut s.lanes, &mut g.0);
    }

    /// O traço directo do motor da água está aberto (o gate do fecho no último pen-up).
    #[cfg(test)]
    pub(crate) fn wet_traco_directo_aberto(&self) -> bool {
        self.paint
            .wetpaint
            .session
            .as_ref()
            .is_some_and(|s| s.stroke_open)
    }
}
