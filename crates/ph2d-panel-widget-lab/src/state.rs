//! Estado retido do laboratório — **a aparência escolhida, e o que a bancada mostra**.
//!
//! ⚠️ **O que NÃO está aqui é o ponto:** o valor da caixa viva mora no `WidgetStore` como qualquer
//! slider do app, e a APARÊNCIA é um [`SliderStyle`](ph2d_tokens::SliderStyle) — o mesmo tipo que o
//! produto lê. *Guardar aqui uma cópia dos três eixos daria duas respostas a «qual é o desenho
//! actual?», e a que o artista vê seria a que envelhece.*

use ph2d_editor_core::zones::Rect;
use ph2d_tokens::SliderStyle;

/// O que a bancada retém entre quadros.
///
/// ⚠️ **O `Default` é escrito à mão de propósito** — ele tem de honrar as decisões já tomadas:
/// a aparência nasce no padrão do app (`Underline` · `4` · `22`), a coluna de animação nasce
/// LIGADA (*"em todas as propriedades que podem ser animadas"*) e a comparação com o widget antigo
/// nasce ligada (sem ela não se vê se melhorámos). *Um default que contradiz a decisão obriga a
/// re-decidir a cada abertura.*
#[derive(Clone, Debug)]
pub struct WidgetLabState {
    /// Geometria da janela flutuante, persistida entre quadros.
    pub rect: Option<Rect>,
    /// ⭐ **A aparência que o app inteiro vai usar** — publicada a cada quadro pelo `paint`.
    pub style: SliderStyle,
    /// Índice na tabela de acentos do [`crate::study`]. ⚠️ Índice, não `ColorToken`: a tabela é do
    /// estudo, e um token guardado aqui deixaria de existir se a tabela mudasse.
    pub accent: usize,
    /// Desenha a coluna de animação em todas as amostras.
    pub decorator: bool,
    /// Mostra a secção com o widget antigo, lado a lado.
    pub compare: bool,
}

impl Default for WidgetLabState {
    fn default() -> Self {
        Self {
            rect: None,
            style: SliderStyle::default(),
            accent: 0,
            decorator: true,
            compare: true,
        }
    }
}
