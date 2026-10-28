//! ⭐ **A vista do jogo ENCAIXADA no canvas visível** (report do dono 25/10, *Play in the Editor*: «não
//! fecha a câmera na área visível do canvas, como se houvesse áreas ocultas por trás dos painéis»).
//!
//! A câmara do editor cobre a JANELA inteira (o centro dela no centro da janela, a altura dela na
//! altura da janela); o jogo tem de se ver no rectângulo do canvas entre os painéis. A câmara do jogo
//! é medida nesse rectângulo — a proporção dele — e escrita na da janela com a mesma escala e o centro
//! deslocado. Com o canvas a ocupar a janela (o palco) é a identidade.

/// ⭐ **O rectângulo do JOGO dentro do canvas** (plano GEF 17 D-F): o maior com a proporção do formato do jogo,
/// centrado. É nele que a vista do jogo se mede e se encaixa — o editor, o palco e a moldura pela MESMA conta.
#[must_use]
pub fn caixa_do_jogo(canvas: [f32; 4], formato: ph2d_editor_core::GameFormat) -> [f32; 4] {
    let a = formato.aspect();
    if canvas[2] <= 0.0 || canvas[3] <= 0.0 || !(a > 0.0) {
        return canvas;
    }
    let (w, h) = if canvas[2] / canvas[3] > a {
        (canvas[3] * a, canvas[3])
    } else {
        (canvas[2], canvas[2] / a)
    };
    [
        canvas[0] + (canvas[2] - w) * 0.5,
        canvas[1] + (canvas[3] - h) * 0.5,
        w,
        h,
    ]
}

/// O tamanho com que a janela do jogo abre (pontos lógicos): o formato ampliado até caber em [`CAIXA_DA_JANELA`];
/// uma consola antiga amplia por um inteiro (píxeis nítidos).
#[must_use]
pub fn tamanho_da_janela(formato: ph2d_editor_core::GameFormat) -> [u32; 2] {
    let [w, h] = formato.resolution().map(f64::from);
    let [cw, ch] = CAIXA_DA_JANELA.map(f64::from);
    let s = (cw / w).min(ch / h);
    let s = if formato.preset.is_retro() && s >= 1.0 {
        s.floor()
    } else {
        s
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    [(w * s).round() as u32, (h * s).round() as u32]
}

/// A caixa em que a janela do jogo cabe ao abrir: 1280 de largura (o 16:9 de sempre, 1280 × 720) e 900 de altura
/// (um ecrã de 1080 com a barra do sistema e a moldura da janela).
pub const CAIXA_DA_JANELA: [u32; 2] = [1280, 900];

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
        let meia_l = altura * canvas[2] / canvas[3] * 0.5;
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

    fn formato(p: ph2d_editor_core::GamePreset) -> ph2d_editor_core::GameFormat {
        ph2d_editor_core::GameFormat {
            preset: p,
            ..Default::default()
        }
    }

    /// ⭐ **A moldura é o rectângulo do jogo, em dois formatos** — a caixa tem a proporção do JOGO (nunca a do
    /// canvas), cabe inteira no canvas e toca-o em dois lados.
    #[test]
    fn a_caixa_tem_a_proporcao_do_jogo_e_nao_a_do_canvas() {
        use ph2d_editor_core::GamePreset as P;
        let canvas = [300.0, 60.0, 960.0, 700.0];
        for (p, esperado) in [
            (P::Wide16x9, [300.0, 140.0, 960.0, 540.0]),
            (P::GameBoy, [391.111_1, 60.0, 777.777_8, 700.0]),
            (P::Phone9x16, [583.125, 60.0, 393.75, 700.0]),
        ] {
            let c = caixa_do_jogo(canvas, formato(p));
            for i in 0..4 {
                assert!(
                    (c[i] - esperado[i]).abs() < 1e-2,
                    "{p:?}: caixa {c:?}, esperada {esperado:?}"
                );
            }
            assert!((c[2] / c[3] - formato(p).aspect()).abs() < 1e-4);
        }
    }

    /// ⭐ A janela do jogo abre com a proporção do formato; uma consola antiga por um inteiro.
    #[test]
    fn a_janela_do_jogo_abre_com_o_formato() {
        use ph2d_editor_core::GamePreset as P;
        assert_eq!(tamanho_da_janela(formato(P::Wide16x9)), [1280, 720]);
        assert_eq!(tamanho_da_janela(formato(P::GameBoy)), [960, 864]);
        assert_eq!(tamanho_da_janela(formato(P::Pico8)), [896, 896]);
        assert_eq!(tamanho_da_janela(formato(P::Phone9x16)), [506, 900]);
    }

    #[test]
    fn com_o_canvas_a_janela_inteira_e_a_identidade() {
        let c = na_janela([3.0, -1.0], 8.0, [1600.0, 900.0], [0.0, 0.0, 1600.0, 900.0]);
        assert_eq!(c, ([3.0, -1.0], 8.0));
    }
}
