//! **A preferência da vista** (P13 U-CRT, `docs/PixelArt/P13_plano.md` §11–§12): ligada e os cinco
//! números, da SESSÃO — nunca o documento, nunca o ficheiro. Um só sítio para os dois leitores: a
//! Preview (`ph2d-panel-pixel-preview`) e a janela do jogo (a shell, por [`crate::palco`]). Mora na
//! folha para que a shell não dependa da crate do painel.

use crate::CrtParams;

/// A preferência da vista: ligada e os cinco números.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VistaCrt {
    pub ligada: bool,
    pub params: CrtParams,
}

/// Os params ao ligar (Scanlines, Mask, Curvature, Brightness, Bloom) — dado da lei, não UI.
#[rustfmt::skip]
const FABRICA: [f32; 5] = [0.5, 0.25, 0.1, 1.2, 0.15]; // LITERAL-PX-OK: params da lei CRT, não UI

impl Default for VistaCrt {
    /// Desligada; ao ligar, um ecrã de TV que se lê.
    fn default() -> Self {
        Self {
            ligada: false,
            params: CrtParams {
                scanline: FABRICA[0],
                mascara: FABRICA[1],
                curvatura: FABRICA[2],
                brilho: FABRICA[3],
                bloom: FABRICA[4],
            },
        }
    }
}

thread_local! {
    static VISTA: std::cell::Cell<VistaCrt> = std::cell::Cell::new(VistaCrt::default());
}

/// A CRT da vista agora (a Preview e a janela do jogo).
#[must_use]
pub fn vista() -> VistaCrt {
    VISTA.with(std::cell::Cell::get)
}

/// Os params que um leitor aplica: `None` = desligada (o leitor salta a chamada).
#[must_use]
pub fn params_ligados() -> Option<CrtParams> {
    let v = vista();
    v.ligada.then_some(v.params)
}

pub fn define(v: VistaCrt) {
    VISTA.with(|c| c.set(v));
}
