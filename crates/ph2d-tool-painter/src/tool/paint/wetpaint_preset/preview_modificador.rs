//! ⭐ **O preview das ferramentas que MEXEM na tinta** (dono 23/10: *«um traço de pintura na vertical e o
//! traço da tool modificadora na horizontal»*). A Borracha, o Blur, o Smear, a Faca e o Clone não depositam:
//! sobre a tela vazia o preview delas saía em branco. Agora pinta-se primeiro uma FAIXA VERTICAL de tinta de
//! referência (o mesmo pincel e a mesma cor em todas as ferramentas, para se compararem) e depois a
//! ferramenta passa na HORIZONTAL, cruzando-a — pelo mesmo `PainterTool` e o mesmo gesto da tela.

use super::super::PaintMode;
use crate::{PaintMedia, PainterTool};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

/// A ferramenta que mexe na tinta que já lá está — a que o preview mostra a cruzar a faixa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modificador {
    Borracha,
    Blur,
    Smear,
    /// A espátula do Impasto (o Smear que leva o corpo).
    Faca,
    Clone,
}

impl Modificador {
    /// A ferramenta na mão, se ela mexe na tinta. O Wet Paint tem as suas (a cena delas é a da miniatura);
    /// a Máscara escreve noutro canal e o Sculpt no relevo — nenhuma das três mexe na cor que lá está.
    #[must_use]
    pub fn de(t: &PainterTool) -> Option<Self> {
        match t.paint.paint_mode {
            PaintMode::Paint if t.paint.eraser => Some(Self::Borracha),
            PaintMode::Blur => Some(Self::Blur),
            PaintMode::Smear => Some(Self::Smear),
            PaintMode::Knife => Some(Self::Faca),
            PaintMode::Clone => Some(Self::Clone),
            _ => None,
        }
    }

    /// O fio do trilho (`set_paint_tool_mode`) — a mesma porta que o clique no botão.
    #[must_use]
    pub const fn fio(self) -> &'static str {
        match self {
            Self::Borracha => "eraser",
            Self::Blur => "blur",
            Self::Smear => "smear",
            Self::Faca => "knife",
            Self::Clone => "clone",
        }
    }
}

/// A cor da tinta de referência — o vermelho da faixa das miniaturas das ferramentas.
pub const COR_DA_REFERENCIA: [u8; 3] = [191, 0, 18];

/// A faixa de referência na tela `lw × lh`: o centro em `x` e a meia-largura. Fixa pela TELA (que cresce
/// com o pincel), nunca pelo pincel da ferramenta: é a mesma em todas.
#[must_use]
pub fn faixa(lw: u32, lh: u32) -> (f32, f32) {
    (lw as f32 * 0.5, lh as f32 * 0.12)
}

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Pinta a faixa vertical de referência em `t` (que ainda está no pincel de fábrica): Digital, redondo,
/// força e fluxo cheios, sem acumular, de cima a baixo.
pub(super) fn pinta_a_referencia(t: &mut PainterTool, lw: u32, lh: u32) {
    t.set_paint_media(PaintMedia::Digital);
    let (x, meia) = faixa(lw, lh);
    let b = &mut t.paint.brush;
    b.radius_px = meia;
    b.strength = 1.0;
    (b.flow, b.accumulate) = (1.0, false);
    b.blend = ph2d_painter_brush::BrushBlend::Mix;
    t.set_brush_color_srgb8(COR_DA_REFERENCIA);
    let (y0, y1) = (-meia, lh as f32 + meia);
    let n = ((y1 - y0) / 3.0).ceil() as u32;
    t.on_canvas_pointer(ponto([x, y0], PointerPhase::Down));
    for k in 1..=n {
        t.on_canvas_pointer(ponto(
            [x, y0 + (y1 - y0) * k as f32 / n as f32],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(ponto([x, y1], PointerPhase::Up));
}

/// `px` (RGBA, alfa direto) composto sobre o branco opaco.
#[must_use]
pub(super) fn sobre_o_branco(px: &[u8]) -> Vec<u8> {
    px.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| {
            let sobre = |c: u8| {
                ((u32::from(c) * u32::from(a) + 255 * (255 - u32::from(a)) + 127) / 255) as u8
            };
            [sobre(r), sobre(g), sobre(b), 255]
        })
        .collect()
}

/// O Clone copia de um quarto da tela à direita do início do gesto: ao passar pela faixa traz o papel, e a
/// faixa reaparece um quarto à esquerda — o que ele faz, à vista.
pub(super) fn fonte_do_clone(t: &mut PainterTool, inicio: [f32; 2], lw: u32) {
    t.set_clone_source([inicio[0] + lw as f32 * 0.25, inicio[1]]);
}

#[cfg(test)]
#[path = "preview_modificador_tests.rs"]
mod tests;
