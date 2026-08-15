//! **Onde um MENU cabe.**
//!
//! Irmão do [`super`] pelo tecto de LOC, cortado por assunto: o pai desenha a MOLDURA de um
//! painel (cabeçalho, corpo, punhos); este responde *onde é que uma superfície flutuante pousa*.

/// Folga entre um menu e a borda do viewport.
const MENU_EDGE_INSET: f32 = 4.0; // LITERAL-PX-OK: menu-to-viewport edge inset (chrome)

/// **Onde um MENU cabe** — desliza o rect ancorado em `(anchor_x, anchor_y)` para dentro do
/// `viewport`, com uma folga da borda.
///
/// ⚠️ **Não confundir com a [`crate::widget::Dropdown::popover_rect_clamped`], que é outra lei
/// para outra coisa.** Aquela é de um popover preso a um CHIP: ele tem de continuar agarrado, então
/// ou fica abaixo, ou VIRA para cima, e encolhe se nenhum lado servir. Um menu não está preso a
/// nada — ele só tem de estar na tela —, e por isso a resposta dele é DESLIZAR. Um menu que só
/// virasse ficaria de fora sempre que nem acima nem abaixo coubesse, que é exactamente o estado em
/// que a lista do "+ Track" foi encontrada (13 linhas = 372 px, com ~235 px abaixo e ~239 acima).
///
/// ⚠️ **Ela vive aqui e não no `context_menu_overlay` de onde saiu** porque ganhou um consumidor
/// numa crate de PAINEL, e o `screens::hero` não é para onde um painel deve ter de estender a mão —
/// é o mesmo motivo, escrito no mesmo ficheiro, que trouxe o `HIGHLIGHTER_RGBA` para cá.
///
/// ⚠️ **Ela move, não encolhe.** Num viewport mais BAIXO que o menu não há resposta boa, e a que
/// ela dá é encostar ao topo — o resto fica de fora, e quem pinta tem de recusar as linhas que
/// caem lá (uma linha registada onde o rato não chega é pior que uma linha ausente).
#[must_use]
pub fn clamp_menu_to_viewport(
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
    viewport: crate::zones::Rect,
) -> crate::zones::Rect {
    let max_x = (viewport.x + viewport.w - w - MENU_EDGE_INSET).max(viewport.x + MENU_EDGE_INSET);
    let max_y = (viewport.y + viewport.h - h - MENU_EDGE_INSET).max(viewport.y + MENU_EDGE_INSET);
    let x = anchor_x.min(max_x).max(viewport.x + MENU_EDGE_INSET);
    let y = anchor_y.min(max_y).max(viewport.y + MENU_EDGE_INSET);
    crate::zones::Rect::new(x, y, w, h)
}
