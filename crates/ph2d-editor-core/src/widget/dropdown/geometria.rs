//! ⭐⭐⭐ **A GEOMETRIA de um chip de escolha — o que ele GASTA, e o caminho inverso.**
//!
//! ⚠️ **Saiu do [`super`] por TETO DE LOC** (`509 > 500`, 2026-09-19) e o corte é de **assunto**,
//! o mesmo do [`crate::paint_label_box`] e do irmão `tag_geometria`: o ficheiro-mãe responde *que
//! aspecto tem um chip*; este responde *quanto dele é do RÓTULO*. As três funções são uma lei só,
//! com ida, volta e o tamanho do chevron de que as duas dependem.

use crate::zones::Rect;
use ph2d_tokens::Spacing;

/// ⭐⭐ **O tamanho do chevron de um chip** — 60 % da altura, entre `14` e `20`.
///
/// Sai para uma porta porque a LARGURA do chip depende dele: quem dimensiona precisa da mesma
/// resposta que quem pinta.
#[must_use]
pub fn dropdown_chevron_size(h: f32) -> f32 {
    (h * 0.6).clamp(14.0, 20.0) // LITERAL-PX-OK: chevron sized 60% of host height with min/max
}

/// ⭐⭐⭐ **O que sobra de um chip de dropdown para o RÓTULO** — depois dos dois recuos, do vão e do
/// chevron.
///
/// ⛔⛔ **Ela é pública por um defeito medido** (2026-09-18, varredura das elisões): a barra da tira
/// do Flip reserva `84 px` para o chip do ciclo, e desses **`46` não são texto** ⇒ o rótulo tem
/// `38`, e `No Cycle` mede `56,4`. Saía **`No…`**.
///
/// ⚠️⚠️ **E o censo não vê metade do defeito:** ele mede o rótulo que está PINTADO, que é a opção
/// escolhida — `Ping-Pong` (`65,5`) e `Ease In-Out` (`72,9`) vivem na mesma lista e nunca foram
/// medidos por ninguém. *Quem dimensiona um chip de escolha tem de o fazer pela LISTA, nunca pelo
/// item em mãos.*
#[must_use]
pub fn dropdown_label_budget(rect: Rect) -> f32 {
    let pad_x = Spacing::Lg.px();
    (rect.w - pad_x * 2.0 - Spacing::Md.px() - dropdown_chevron_size(rect.h)).max(0.0)
}

/// ⭐⭐⭐ **O caminho INVERSO: que largura de chip um rótulo de `text_w` precisa.**
///
/// ⚠️ Ela e a [`dropdown_label_budget`] são uma lei só, com gate de ida-e-volta — derivar a conta à
/// mão no painel seria a segunda cópia do recuo, que divergiria no dia em que o `Spacing::Lg`
/// mudasse. *É o mesmo par que o `label_budget`/`rect_for_label` já é para uma caixa de rótulo.*
#[must_use]
pub fn dropdown_chip_width_for(text_w: f32, h: f32) -> f32 {
    let w = text_w + Spacing::Lg.px() * 2.0 + Spacing::Md.px() + dropdown_chevron_size(h);
    // ⛔⛔⛔ **E ela confere-se contra a LEI, nunca contra a álgebra que a escreveu.**
    //
    // Medido em 2026-09-19 e apanhado pelo PRODUTO: a coluna da família do editor de variantes
    // passou a pedir exactamente o que `Dictionary` mede (`63,62`), o chip recebeu exactamente
    // esse número de volta… e o rótulo saiu **`Dictiona…`**. Em `f32` `(t + c) − c` fica **abaixo**
    // de `t` na maioria do domínio, e a elisão compara `<=`: *um défice de um ULP corta a palavra
    // inteira*. ⚠️ É o **terceiro** par ida/volta desta casa com o mesmo furo (o
    // [`crate::paint::rect_for_label`] e a [`crate::widget::Tag::width_for`] foram curados no
    // mesmo dia) — **a álgebra fecha e a aritmética de máquina não**.
    if dropdown_label_budget(Rect::new(0.0, 0.0, w, h)) < text_w {
        w.next_up()
    } else {
        w
    }
}
