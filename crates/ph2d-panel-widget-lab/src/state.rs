//! Estado retido do laboratório — **os eixos do estudo, e mais nada**.
//!
//! ⚠️ **O que NÃO está aqui é o ponto:** o valor da caixa viva mora no `WidgetStore` como qualquer
//! slider do app. Guardá-lo aqui daria duas respostas à pergunta *"em quanto está a caixa?"*, e o
//! defeito que isso produz já foi pago duas vezes neste repo (a caixa «Playing» do Sprite, o
//! `hover_live` do vector).

use ph2d_editor_core::zones::Rect;

use crate::design::BoxDesign;

/// Os eixos que o Enio pediu para poder variar: desenho, cor, raio, densidade, coluna de animação
/// e a comparação com o de hoje.
///
/// ⚠️ **O `Default` é escrito à mão de propósito.** O derivado poria `decorator: false` e
/// `compare: false`, e as duas contradizem decisões já tomadas — a coluna de animação vale para
/// toda propriedade animável, e a bancada sem o «hoje» ao lado não deixa ver se melhorámos.
/// *Um default que contradiz a decisão obriga a re-decidir a cada abertura.* E tem de ser o
/// `Default`, não um construtor: quem cria o estado é o `ErasedPanel`, que só sabe `Default`.
#[derive(Clone, Debug)]
pub struct WidgetLabState {
    /// Geometria da janela flutuante, persistida entre quadros.
    pub rect: Option<Rect>,
    /// Qual dos seis desenhos está escolhido.
    pub design: BoxDesign,
    /// Índice na tabela de acentos do [`crate::study`]. ⚠️ **Índice, não `ColorToken`** — a tabela
    /// é do estudo, e um token guardado aqui deixaria de existir se a tabela mudasse.
    pub accent: usize,
    /// Índice na tabela de raios.
    pub radius: usize,
    /// Índice na escada de densidade (`Compact` · `Cozy` · `Comfortable`).
    pub density: usize,
    /// Desenha a coluna de animação em todas as amostras.
    ///
    /// ⭐ Nasce **ligada**: o Enio decidiu que ela vale para toda propriedade animável, e um
    /// laboratório cujo default contradiz a decisão faz-nos re-decidir a cada abertura.
    pub decorator: bool,
    /// Mostra a §5 — o widget de hoje lado a lado.
    pub compare: bool,
}

impl Default for WidgetLabState {
    fn default() -> Self {
        Self {
            rect: None,
            design: BoxDesign::default(),
            accent: 0,
            radius: 0,
            density: 0,
            decorator: true,
            compare: true,
        }
    }
}
