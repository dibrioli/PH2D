//! ⭐⭐ **O QUE FICA POR BAIXO DA MINIATURA DE UM CARTÃO** — a lei que o [`super::paint`] aplica,
//! separada dele para poder ser **medida** (e porque aquele ficheiro está a uma linha do tecto).
//!
//! # A pergunta que isto responde
//!
//! Uma miniatura tem alfa: um objecto recortado deixa ver o que está atrás dele. *O quê?*
//!
//! Até 2026-09-02 a resposta era **a cor dominante do próprio asset** — a média de um pixel que a
//! wave A2 calcula. Ela tinha uma razão boa (reconhecer um asset **antes** de a miniatura existir)
//! e uma consequência má, que o dono do produto viu no smoke: o mesmo objecto lia-se de uma cor no
//! cartão e de outra na tela, e nada as ligava.
//!
//! ⇒ A lei passa a ser **a cor do fundo do canvas**, pela porta única
//! ([`ph2d_editor_core::screens::hero::canvas_backdrop`]) — o cartão mostra o objecto sobre o mesmo
//! fundo em que ele vai pousar, e re-vestir o canvas re-veste os cartões no mesmo quadro.
//!
//! ⛔ **A cor dominante NÃO morre — ela deixa de ser fundo e volta a ser o que era: a CARA de um
//! cartão que ainda não tem miniatura.** O orçamento de miniaturas é por quadro, então «sem
//! miniatura» é um estado normal e transitório, não uma falha; apagar a cor ali daria uma grade de
//! quadrados todos iguais enquanto as imagens não chegassem.
//!
//! ⚠️ **Um xadrez seria a terceira resposta, e é recusada de propósito.** Ele diz *"aqui há
//! transparência"*, que é informação sobre o FICHEIRO; o que se pediu é ver o objecto **como ele
//! vai aparecer**, que é informação sobre a CENA.

use ph2d_tokens::{Color, Theme};

/// A cor que o quadrado do cartão leva por baixo do que quer que se desenhe em cima.
///
/// ⚠️ **O `has_thumb` é o sujeito da decisão, e não o tipo do asset.** Um componente e uma imagem
/// caem os dois no mesmo braço quando têm miniatura — porque a pergunta não é *"o que és?"*, é
/// *"há uma forma a desenhar por cima?"*.
#[must_use]
pub fn card_backdrop(theme: Theme, swatch: [u8; 4], has_thumb: bool) -> Color {
    if has_thumb {
        ph2d_editor_core::screens::hero::canvas_backdrop(theme)
    } else {
        Color {
            r: swatch[0],
            g: swatch[1],
            b: swatch[2],
            a: swatch[3],
        }
    }
}
