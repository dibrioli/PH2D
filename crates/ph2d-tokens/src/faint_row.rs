//! O tom de uma linha ÍMPAR — irmão do [`crate::color`] pelo tecto de LOC; o endereço
//! `ph2d_tokens::color::faint_row_bg` não muda (re-exportado lá).

use crate::color::Color;

/// ⭐⭐⭐ **O tom de uma linha ÍMPAR — o que separa duas linhas que se encostam.**
///
/// Enio, 2026-09-06, com a foto do *Outliner* do Blender: *«linhas pares e ímpares têm tonalidade
/// discretamente diferente».* É o companheiro natural da wave 17: quando as linhas de uma lista
/// encostam (vão de 1 px), o que devolve a leitura de *onde acaba uma e começa a outra* deixa de
/// ser o vão e passa a ser o TOM.
///
/// # De onde vem o número
///
/// ⛔ **Não do Blender:** o código dele é GPL e esta linha **não o lê** (a lei da triagem —
/// `pesquisa/02`); e o manual dele (CC-BY-SA) documenta o painel de temas sem enumerar o valor.
/// O número vem do **egui** (Apache-2.0 OR MIT, vendorizado, `crates/egui/src/style.rs:1512` e
/// `:1576`), que tem exactamente este slot com exactamente este propósito:
///
/// ```text
/// faint_bg_color: Color32::from_additive_luminance(5), // visible, but barely so
/// ```
///
/// ⭐ **E ele é o MESMO nos dois modos dele** (claro e escuro) — o que se porta é a magnitude,
/// `5` em 255, não uma cor.
///
/// # A direcção é DERIVADA, e é isso que a faz servir os oito temas
///
/// Somar sempre seria a lei do egui à letra, e ela parte-se num tema claro: um fundo a `250` sobe
/// para `255` e encosta no tecto. ⇒ a régua é *afastar-se do extremo mais próximo*, medida pela
/// [`Color::relative_luminance`], que é a régua de claro-escuro que esta casa já usa nos testes de
/// contraste. Num tema escuro a linha ímpar **clareia**; num claro **escurece**.
///
/// ⭐ E isso resolve de graça os dois temas em que uma lei fixa falharia: o **Light** do Godot, em
/// que a elevação escurece por decisão dele, e o **OLED**, em que o contraste é `0` e não há para
/// onde escurecer — ali só há uma direcção possível, e a régua escolhe-a sozinha.
///
/// ⚠️ **Não é o degrau da ESCADA de cartões** (12 e 10 em 255): esse diz *«isto está dentro
/// daquilo»*. Um degrau desse tamanho entre duas linhas irmãs leria como aninhamento — que é
/// precisamente a confusão que o dono pediu para evitar ao dizer **discretamente**.
pub fn faint_row_bg(base: Color) -> Color {
    /// `5` em 255 — «visible, but barely so» (egui `style.rs:1512`).
    const FAINT_STEP: i16 = 5;
    let step = if base.relative_luminance() > 0.5 {
        -FAINT_STEP
    } else {
        FAINT_STEP
    };
    let shift = |c: u8| (c as i16 + step).clamp(0, 255) as u8;
    Color {
        r: shift(base.r),
        g: shift(base.g),
        b: shift(base.b),
        a: base.a,
    }
}
