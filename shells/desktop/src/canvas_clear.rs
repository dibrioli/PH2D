//! ⭐⭐ **A COR COM QUE A CAMADA DE SPRITES É LIMPA** — o fundo que o artista de facto vê atrás
//! dos objectos, e a razão de ele ser hoje **derivado** em vez de escrito à mão.
//!
//! # Porque isto é o fundo do canvas, e não o `paint_canvas_bg`
//!
//! Em modo vivo (com `grid_view` publicado, que é sempre no produto) o `HeroScreen` **salta** o
//! fill opaco do canvas: o compositor mostra o `game_rt` por baixo de onde o `vello_rt` tem α=0.
//! ⇒ *o que se vê no canvas é ESTE `clear`*, e o `Bg1` pintado pelo vello só aparece no modo
//! fixtura. Quem procurar a cor do fundo no painter vai encontrar código que o produto não corre.
//!
//! # ⛔ A conversão que NÃO se faz, e o mecanismo por trás disso
//!
//! O valor entra no `wgpu` como componente **linear** — e aqui divide-se o byte sRGB por 255 sem
//! linearizar, o que está errado em teoria e é **exactamente** o que o produto precisa. A nota
//! histórica no `render_loop` tem o mecanismo: antes da M14.5 este fundo era o `Bg1` pintado pelo
//! vello (byte ~12) e o blitter antigo amostrava-o como `12/255 ≈ 0,047` *tratado como linear* — a
//! confusão de gama documentada no `ph2d-render/src/vello_pass.rs`; as bordas anti-aliased do chrome,
//! em `ph2d-tokens`, estão calibradas contra esse fundo.
//! ⛔ Passar o `Bg1` pela conversão correcta (linear 0,012) é a regressão dos *"pixelated borders"*
//! da M14.5 ronda 2 — **medida e revertida**. A divisão por 255 reproduz o fundo legado byte a
//! byte, e é por isso que ela é a conversão certa neste sítio.
//!
//! ⚠️ **O que mudou em 2026-09-02 foi só a FONTE, não a lei.** O literal `(0,047, 0,047, 0,055)`
//! era uma cópia à mão do `Bg1` do Forge; hoje o valor vem da porta
//! ([`ph2d_editor::screens::hero::canvas_backdrop`]), a mesma que o cartão do navegador de
//! assets lê. Enquanto era cópia, *"mudar a cor do canvas"* (trocar de tema, ou autorar o token no
//! painel de Tokens) movia tudo à volta e deixava o canvas onde estava — e nenhum gate o dizia,
//! porque cada sítio estava certo sozinho.

use ph2d_tokens::Theme;

/// Quantos passos tem um canal de cor de 8 bits.
///
/// ⚠️ **Não é um número de UI, é o denominador de uma REPRESENTAÇÃO** — ver a nota de cabeçalho
/// sobre a conversão que deliberadamente não acontece.
const CHANNEL_STEPS: f64 = 255.0;

/// O `clear` da camada de sprites, derivado do fundo do canvas deste tema.
#[must_use]
pub(crate) fn canvas_clear_rgb(theme: Theme) -> (f64, f64, f64) {
    let c = ph2d_editor::screens::hero::canvas_backdrop(theme);
    (
        f64::from(c.r) / CHANNEL_STEPS,
        f64::from(c.g) / CHANNEL_STEPS,
        f64::from(c.b) / CHANNEL_STEPS,
    )
}

#[cfg(test)]
#[path = "canvas_clear_tests.rs"]
mod tests;
