//! ⭐ **A âmbar do REALCE do editor** — a cor de *«esta é a coisa que tens na mão»*: o realce e o hover de
//! selecção do Flip, a trajectória e as alças de tangente do Motion.
//!
//! ⚠️ **Fica FORA do tema de propósito:** os temas (`ph2d-tokens`, OKLCH por tema) pintam o chrome, e um
//! overlay sobre a arte tem de ler igual em qualquer tema — é por isso que as quatro constantes que esta
//! porta substitui carregavam `LITERAL-COLOR-OK`. ⛔ Até 2026-09-13 o literal vivia em QUATRO sítios de
//! DUAS famílias, unidos por um comentário («a mesma âmbar»): *uma lei escrita em dois sítios ainda não é
//! uma lei, só uma porta é*. O gate `the_editor_amber_has_one_door` impede-o de voltar a nascer fora
//! daqui. Entre os usos muda o ALFA (a selecção é um FACTO, o hover uma PROMESSA), nunca o matiz.

/// O matiz — as componentes 0..1 que o `Color::new` do Vello recebe.
pub const AMBER_RGB: [f32; 3] = [1.0, 0.72, 0.2]; // LITERAL-COLOR-OK: overlay do editor, invariante ao tema

/// A âmbar com o `alpha` do uso.
pub const fn amber(alpha: f32) -> [f32; 4] {
    [AMBER_RGB[0], AMBER_RGB[1], AMBER_RGB[2], alpha]
}
