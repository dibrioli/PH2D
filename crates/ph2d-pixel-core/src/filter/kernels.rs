//! Os kernels da Convolution Matrix — DADOS NOSSOS com o nome medido: cada matriz foi
//! recuperada da resposta do Trial (`docs/PixelArt/ferramentas/fit_convolucao.py` sobre
//! `tests/fixtures/oracle_trial_convolution.json`, 0 células fora). Gerado; não editar à mão.

use super::convolution::Kernel;

#[rustfmt::skip]
pub const KERNELS: &[Kernel] = &[
    Kernel { name: "brightness", w: 1, h: 1, cx: 0, cy: 0, div: 1, bias: 8, weights: &[1] },
    Kernel { name: "contrast", w: 1, h: 1, cx: 0, cy: 0, div: 1, bias: -8, weights: &[1] },
    Kernel { name: "negative", w: 1, h: 1, cx: 0, cy: 0, div: 1, bias: 255, weights: &[-1] },
    Kernel { name: "blur-3x3", w: 3, h: 3, cx: 1, cy: 1, div: 16, bias: 0, weights: &[1, 2, 1, 2, 4, 2, 1, 2, 1] },
    Kernel { name: "blur-3x3-hard", w: 3, h: 3, cx: 1, cy: 1, div: 12, bias: 0, weights: &[0, 1, 0, 1, 8, 1, 0, 1, 0] },
    Kernel { name: "blur-5x5", w: 5, h: 5, cx: 2, cy: 2, div: 65, bias: 0, weights: &[1, 2, 3, 2, 1, 2, 3, 4, 3, 2, 3, 4, 5, 4, 3, 2, 3, 4, 3, 2, 1, 2, 3, 2, 1] },
    Kernel { name: "blur-7x7", w: 7, h: 7, cx: 3, cy: 3, div: 175, bias: 0, weights: &[1, 2, 3, 4, 3, 2, 1, 2, 3, 4, 5, 4, 3, 2, 3, 4, 5, 6, 5, 4, 3, 4, 5, 6, 7, 6, 5, 4, 3, 4, 5, 6, 5, 4, 3, 2, 3, 4, 5, 4, 3, 2, 1, 2, 3, 4, 3, 2, 1] },
    Kernel { name: "blur-9x9", w: 9, h: 9, cx: 4, cy: 4, div: 369, bias: 0, weights: &[1, 2, 3, 4, 5, 4, 3, 2, 1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 4, 5, 6, 7, 8, 7, 6, 5, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1, 2, 3, 4, 5, 4, 3, 2, 1] },
    Kernel { name: "blur-5x3-left", w: 5, h: 3, cx: 4, cy: 1, div: 32, bias: 0, weights: &[0, 1, 2, 3, 2, 1, 2, 3, 4, 6, 0, 1, 2, 3, 2] },
    Kernel { name: "blur-5x5-diagonal(\\)", w: 5, h: 5, cx: 2, cy: 2, div: 27, bias: 0, weights: &[1, 1, 1, 0, 0, 1, 2, 2, 1, 0, 1, 2, 3, 2, 1, 0, 1, 2, 2, 1, 0, 0, 1, 1, 1] },
    Kernel { name: "blur-5x5-diagonal(/)", w: 5, h: 5, cx: 2, cy: 2, div: 27, bias: 0, weights: &[0, 0, 1, 1, 1, 0, 1, 2, 2, 1, 1, 2, 3, 2, 1, 1, 2, 2, 1, 0, 1, 1, 1, 0, 0] },
    Kernel { name: "sharpen-3x3", w: 3, h: 3, cx: 1, cy: 1, div: 8, bias: 0, weights: &[-1, -1, -1, -1, 16, -1, -1, -1, -1] },
    Kernel { name: "sharpen-5x5", w: 5, h: 5, cx: 2, cy: 2, div: 8, bias: 0, weights: &[0, -1, -2, -1, 0, -1, -2, -4, -2, -1, -2, -4, 48, -4, -2, -1, -2, -4, -2, -1, 0, -1, -2, -1, 0] },
    Kernel { name: "sharpen-7x7", w: 7, h: 7, cx: 3, cy: 3, div: 16, bias: 0, weights: &[0, -1, -2, -4, -2, -1, 0, -1, -2, -4, -8, -4, -2, -1, -2, -4, -8, -16, -8, -4, -2, -4, -8, -16, 224, -16, -8, -4, -2, -4, -8, -16, -8, -4, -2, -1, -2, -4, -8, -4, -2, -1, 0, -1, -2, -4, -2, -1, 0] },
    Kernel { name: "misc-contour", w: 3, h: 3, cx: 1, cy: 1, div: 1, bias: 255, weights: &[1, 1, 1, 1, -8, 1, 1, 1, 1] },
    Kernel { name: "misc-marmolize", w: 3, h: 3, cx: 1, cy: 1, div: 2, bias: 128, weights: &[1, 1, -1, 1, 0, -1, 1, -1, -1] },
    Kernel { name: "misc-rock-edges", w: 3, h: 3, cx: 1, cy: 1, div: 1, bias: 128, weights: &[-1, -1, -1, -1, 8, -1, -1, -1, -1] },
    Kernel { name: "drunk-3x3_x", w: 3, h: 3, cx: 1, cy: 1, div: 5, bias: 0, weights: &[1, 0, 1, 0, 1, 0, 1, 0, 1] },
    Kernel { name: "drunk-3x3_+", w: 3, h: 3, cx: 1, cy: 1, div: 5, bias: 0, weights: &[0, 1, 0, 1, 1, 1, 0, 1, 0] },
    Kernel { name: "drunk-5x5_x", w: 5, h: 5, cx: 2, cy: 2, div: 5, bias: 0, weights: &[1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1] },
    Kernel { name: "drunk-5x5_+", w: 5, h: 5, cx: 2, cy: 2, div: 5, bias: 0, weights: &[0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0] },
    Kernel { name: "drunk-7x7_x", w: 7, h: 7, cx: 3, cy: 3, div: 5, bias: 0, weights: &[1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1] },
    Kernel { name: "drunk-7x7_+", w: 7, h: 7, cx: 3, cy: 3, div: 5, bias: 0, weights: &[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0] },
    Kernel { name: "drunk-9x9_x", w: 9, h: 9, cx: 4, cy: 4, div: 5, bias: 0, weights: &[1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1] },
    Kernel { name: "drunk-9x9_+", w: 9, h: 9, cx: 4, cy: 4, div: 5, bias: 0, weights: &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0] },
];
