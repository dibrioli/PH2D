//! **A LEGENDA DE UMA CENA DE SMOKE, NO CANVAS** — o rótulo pousa em cima da coisa que ele
//! explica, e não num terminal atrás da janela.
//!
//! Pedido do Enio (2026-08-23): *"melhore as explicações do smoke"*.
//!
//! ## Por que a explicação estava no sítio errado
//!
//! Uma cena de conferência é uma GRELHA de casos — três, seis, oito linhas, metade esquerda
//! contra metade direita — e até aqui o que dizia qual era qual era um bloco de `eprintln!` que
//! sai **antes de a janela abrir**. O Enio corre o comando, a janela cobre o terminal, e a partir
//! daí ele tem de contar linhas de cima para baixo e casá-las de memória com um texto que já não
//! está à vista. *A explicação existia; o que faltava era ela estar onde os olhos estão.*
//!
//! ⚠️ **O anúncio no terminal FICA**, e não é redundância: ele é o único sítio onde cabe o *"deu
//! errado se…"*, que é uma frase e não um rótulo. A ficha diz **o que é aquilo**; o terminal diz
//! **como saber que falhou**. Duas perguntas, dois sítios.
//!
//! ## A costura: uma função PURA por cena, e um só salto global
//!
//! Cada cena expõe uma `captions()` **pura** — âncora de mundo + texto —, e é essa função que os
//! gates medem. O roteador publica-a aqui, e o passe de pintura lê-a.
//!
//! ⚠️ **O global é um SALTO, não um estado.** Ele existe porque `build_level` é uma função livre
//! (documento + registry, sem `App`) chamada de dentro do construtor do `MotionState`: devolver
//! as legendas obrigaria a mudar a assinatura dos ~80 braços do `match`. Ele é escrito uma vez, na
//! construção da cena, e nunca mais. ⛔ **Nenhum gate lê daqui** — eles chamam a `captions()` da
//! cena, senão dois testes em paralelo disputariam a mesma célula.

use std::sync::Mutex;

/// Um rótulo e o ponto de MUNDO sobre o qual ele pousa.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Caption {
    /// Onde, em coordenadas de mundo — o passe de pintura leva-o à tela pelo afim da câmara, e a
    /// ficha é desenhada em pixels de TELA (ela é uma legenda, não uma medida: tem de continuar
    /// legível com o canvas afastado).
    pub(crate) world: [f32; 2],
    /// O que se lê. **Curto** — é uma ficha, não um parágrafo; a frase longa vive no terminal.
    pub(crate) text: String,
}

impl Caption {
    pub(crate) fn new(world: [f32; 2], text: impl Into<String>) -> Self {
        Self {
            world,
            text: text.into(),
        }
    }
}

static LEGEND: Mutex<Vec<Caption>> = Mutex::new(Vec::new());

/// A cena publica a legenda dela. Substitui, nunca acumula.
pub(crate) fn publish(captions: Vec<Caption>) {
    if let Ok(mut slot) = LEGEND.lock() {
        *slot = captions;
    }
}

/// O que o passe de pintura desenha neste quadro (vazio quando nenhuma cena publicou).
pub(crate) fn captions() -> Vec<Caption> {
    LEGEND.lock().map(|s| s.clone()).unwrap_or_default()
}

#[cfg(test)]
#[path = "motion_demo_legend_tests.rs"]
mod tests;
