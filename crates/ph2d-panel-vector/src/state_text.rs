//! **O estado publicado do TEXTO** — irmão de `state.rs` pelo teto de 600 LOC dos painéis (ele
//! estava em 607 quando este nasceu).
//!
//! O corte é por RESPONSABILIDADE: aqui mora o que a shell publica sobre TIPOGRAFIA (o texto, a
//! fonte, o alinhamento, os eixos variáveis, os previews); no irmão, tudo o mais.
//!
//! ⚠️ As células vieram junto com as funções que as leem. Uma célula num arquivo e o seu par de
//! acessores noutro é a forma exata de alguém acrescentar um segundo acessor sem ver o primeiro.

use super::{
    CURRENT_TEXT, CURRENT_TEXT_ALIGN, CURRENT_TEXT_AXES, CURRENT_TEXT_FONT,
    CURRENT_TEXT_HAS_WEIGHT, CURRENT_TEXT_VISIBLE, CURRENT_TEXT_WRAP, FONT_PREVIEWS, TEXT_SEED,
    WANT_FONT_PREVIEWS,
};
use super::{FontPreview, TextAxisSlot};
use ph2d_tool_vector::TextAlign;
use std::cell::Cell;

/// Publica se a seção Text deve aparecer (modo Text OU objeto de texto selecionado).
pub fn set_current_text_visible(v: bool) {
    CURRENT_TEXT_VISIBLE.with(|c| c.set(v));
}

pub(crate) fn text_visible() -> bool {
    CURRENT_TEXT_VISIBLE.with(Cell::get)
}

/// Publica a semente ONE-SHOT dos sliders de texto (só quando o alvo muda).
pub fn set_current_text_seed(seed: Option<[f64; 4]>) {
    TEXT_SEED.with(|c| c.set(seed));
}

/// Publica o texto da sessão de edição ativa (`None` = sem sessão de texto).
/// A shell chama a cada frame; o painel só o exibe (read-only na A2).
pub fn set_current_text(text: Option<String>) {
    CURRENT_TEXT.with(|c| *c.borrow_mut() = text);
}

/// O texto da sessão ativa este frame (`None` ⇒ nada a exibir).
pub(crate) fn current_text() -> Option<String> {
    CURRENT_TEXT.with(|c| c.borrow().clone())
}

/// Publica o rótulo da família de fonte corrente do texto (a shell resolve o nome,
/// inclusive o da embutida). `None` fora do modo Text.
pub fn set_current_text_font(name: Option<String>) {
    CURRENT_TEXT_FONT.with(|c| *c.borrow_mut() = name);
}

/// O rótulo da família de fonte corrente este frame (para o seletor `<` nome `>`).
pub(crate) fn current_text_font() -> Option<String> {
    CURRENT_TEXT_FONT.with(|c| c.borrow().clone())
}

/// Publica o alinhamento horizontal corrente do texto (`None` fora do modo Text).
pub fn set_current_text_align(align: Option<TextAlign>) {
    CURRENT_TEXT_ALIGN.with(|c| c.set(align));
}

/// O alinhamento corrente este frame (destaca o botão L/C/R ativo). `None` = default.
pub(crate) fn current_text_align() -> Option<TextAlign> {
    CURRENT_TEXT_ALIGN.with(Cell::get)
}

/// Publica a largura de refluxo corrente do texto. `None` = **Auto** (sem caixa) — e é o
/// mesmo `None` que o documento guarda, não um "não sei": a fileira Width tem sempre uma
/// resposta, e a seção inteira já é gateada por [`super::text_visible`].
pub fn set_current_text_wrap(wrap: Option<f64>) {
    CURRENT_TEXT_WRAP.with(|c| c.set(wrap));
}

/// A largura de refluxo corrente este frame. `None` = Auto ⇒ o slider **não é pintado**.
pub(crate) fn current_text_wrap() -> Option<f64> {
    CURRENT_TEXT_WRAP.with(Cell::get)
}

/// Publica os eixos de variação da fonte corrente (sem `wght`), na ordem em que a
/// shell os aplica. Vazio fora do modo Text ou para fontes não-variáveis.
pub fn set_current_text_axes(axes: Vec<TextAxisSlot>) {
    CURRENT_TEXT_AXES.with(|c| *c.borrow_mut() = axes);
}

/// **Publica se a fonte corrente expõe o eixo `wght`** — o facto que decide se a fileira *Weight*
/// é sequer pintada.
///
/// ⚠️ **Terceira pergunta, e as duas óbvias respondem outra coisa.** Sem `fvar` o `skrifa` ignora
/// a localização de eixo, então numa fonte ESTÁTICA o slider de Weight é inerte — pintado,
/// arrastável, sem efeito. Mas a regra que esconde a secção AXES (`axes.is_empty()`) **não serve**:
/// aquela lista exclui o `wght` por construção, então uma variável só-de-peso publica-a vazia e
/// teria o Weight escondido **estando vivo e correcto** (medido com a `Cantarell-VF`, cujo `fvar` é
/// exactamente `['wght']`).
///
/// ⚠️ Quem sabe a resposta é quem tem a fonte parseada — a shell (`vec_font::has_weight_axis`).
/// O painel **consome**, nunca deduz.
pub fn set_current_text_has_weight(v: bool) {
    CURRENT_TEXT_HAS_WEIGHT.with(|c| c.set(v));
}

/// A fonte corrente expõe `wght` este frame? `false` ⇒ a fileira Weight **não é pintada**.
pub(crate) fn text_has_weight_axis() -> bool {
    CURRENT_TEXT_HAS_WEIGHT.with(Cell::get)
}

/// Publica a lista de previews do dropdown de fonte (a shell constrói sob demanda).
pub fn set_current_text_font_previews(previews: Vec<FontPreview>) {
    FONT_PREVIEWS.with(|c| *c.borrow_mut() = previews);
}

/// A shell drena o pedido (one-shot): `true` ⇒ construa + publique as previews.
pub fn take_want_font_previews() -> bool {
    WANT_FONT_PREVIEWS.with(|c| c.replace(false))
}
