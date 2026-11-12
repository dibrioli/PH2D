//! ⭐ **A vista do jogo ENCAIXADA no canvas visível** (report do dono 25/10, *Play in the Editor*: «não
//! fecha a câmera na área visível do canvas, como se houvesse áreas ocultas por trás dos painéis»).
//!
//! A câmara do editor cobre a JANELA inteira (o centro dela no centro da janela, a altura dela na
//! altura da janela); o jogo tem de se ver no rectângulo do canvas entre os painéis. A câmara do jogo
//! é medida nesse rectângulo — a proporção dele — e escrita na da janela com a mesma escala e o centro
//! deslocado. Com o canvas a ocupar a janela (o palco) é a identidade.

// A caixa do jogo e o palco que alarga moram com a moldura e as tarjas (Input Map, 11/11).
pub use ph2d_camera_frame::palco::{area_do_palco, caixa_do_jogo, centro_no_palco};

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

/// ⭐ **A vista do editor que SEGUE a câmera** (onda 18, «Editor View: Follows»): o centro da câmara da janela que põe
/// `centro` (o do jogo) no meio do `canvas`, com a altura `altura_da_janela` do ARTISTA — o zoom é dele.
#[must_use]
pub fn centro_que_segue(
    centro: [f32; 2],
    altura_da_janela: f32,
    janela: [f32; 2],
    canvas: [f32; 4],
) -> [f32; 2] {
    if janela[1] <= 0.0 {
        return centro;
    }
    let s = altura_da_janela / janela[1];
    let (cx, cy) = (canvas[0] + canvas[2] * 0.5, canvas[1] + canvas[3] * 0.5);
    [
        centro[0] + (janela[0] * 0.5 - cx) * s,
        centro[1] - (janela[1] * 0.5 - cy) * s,
    ]
}
