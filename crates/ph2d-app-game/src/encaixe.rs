//! ⭐ **A vista do jogo ENCAIXADA no canvas visível** (report do dono 25/10, *Play in the Editor*: «não
//! fecha a câmera na área visível do canvas, como se houvesse áreas ocultas por trás dos painéis»).
//!
//! A câmara do editor cobre a JANELA inteira (o centro dela no centro da janela, a altura dela na
//! altura da janela); o jogo tem de se ver no rectângulo do canvas entre os painéis. A câmara do jogo
//! é medida nesse rectângulo — a proporção dele — e escrita na da janela com a mesma escala e o centro
//! deslocado. Com o canvas a ocupar a janela (o palco) é a identidade.

/// A proporção (largura ÷ altura) com que a câmara do jogo mede a vista: a do canvas visível.
#[must_use]
pub fn aspecto(canvas: [f32; 4]) -> f32 {
    if canvas[3] > 0.0 {
        canvas[2] / canvas[3]
    } else {
        1.0
    }
}

/// A câmara da JANELA (`centro`, `altura` em mundo) que mostra a vista do jogo (`centro`, `altura`)
/// exactamente no `canvas` (`[x, y, largura, altura]` em pixéis, y para baixo) de uma `janela`.
#[must_use]
pub fn na_janela(
    centro: [f32; 2],
    altura: f32,
    janela: [f32; 2],
    canvas: [f32; 4],
) -> ([f32; 2], f32) {
    if canvas[3] <= 0.0 {
        return (centro, altura);
    }
    let s = altura / canvas[3];
    let (cx, cy) = (canvas[0] + canvas[2] * 0.5, canvas[1] + canvas[3] * 0.5);
    (
        [
            centro[0] + (janela[0] * 0.5 - cx) * s,
            centro[1] - (janela[1] * 0.5 - cy) * s,
        ],
        s * janela[1],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O mapa mundo→pixel de uma câmara que cobre a janela (a mesma conta do `Camera2d`).
    fn px(c: ([f32; 2], f32), janela: [f32; 2], p: [f32; 2]) -> [f32; 2] {
        let s = c.1 / janela[1];
        [
            janela[0] * 0.5 + (p[0] - c.0[0]) / s,
            janela[1] * 0.5 - (p[1] - c.0[1]) / s,
        ]
    }

    #[test]
    fn a_vista_do_jogo_cai_inteira_no_canvas_entre_os_paineis() {
        let janela = [1600.0, 900.0];
        let canvas = [300.0, 60.0, 960.0, 700.0];
        let (centro, altura) = ([5.0, 2.0], 10.0);
        let c = na_janela(centro, altura, janela, canvas);
        let meia_l = altura * aspecto(canvas) * 0.5;
        // O centro, o topo-esquerdo e o fundo-direito da vista do jogo nos cantos do canvas.
        let perto =
            |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
        assert!(perto(px(c, janela, centro), [780.0, 410.0]));
        assert!(perto(
            px(c, janela, [centro[0] - meia_l, centro[1] + altura * 0.5]),
            [300.0, 60.0]
        ));
        assert!(perto(
            px(c, janela, [centro[0] + meia_l, centro[1] - altura * 0.5]),
            [1260.0, 760.0]
        ));
    }

    #[test]
    fn com_o_canvas_a_janela_inteira_e_a_identidade() {
        let c = na_janela([3.0, -1.0], 8.0, [1600.0, 900.0], [0.0, 0.0, 1600.0, 900.0]);
        assert_eq!(c, ([3.0, -1.0], 8.0));
    }
}
