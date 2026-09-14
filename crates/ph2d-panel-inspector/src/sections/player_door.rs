//! **A face VAZIA da §14** — um botão, e é ele que faz o comportamento existir.
//!
//! ⛔ **Ela MORREU na F3** (ADR-0166), quando a §14 passou a pintar-se só COM o componente e a rota
//! virou o `+` do cabeçalho. **Voltou em 2026-09-14 por ordem do dono:** *«um objeto de física
//! (Physics Body) e todas as opções aparecem com ele (inclusive Collision Shape e Platform
//! Player)»* — a paleta passa a ter UMA entrada de física, e esta é a porta do comportamento, na
//! secção que fala dele.
//!
//! ⚠️ **Ficheiro próprio pelo TETO DE FUNÇÃO** (200), e o corte estava à mão porque a linha que ele
//! segue é a mesma do `physics_doors.rs` um irmão acima: *o que este player É* × **o que CRIAR
//! aqui**. ⚠️ E o `player.rs` estava **exactamente** no teto de ficheiro (600), então o bloco não
//! podia crescer lá dentro de maneira nenhuma — *dois tectos diferentes a apontar para o mesmo
//! corte é o sinal de que ele é por responsabilidade, e não por aritmética*.

use super::*;
use ph2d_i18n::tr;

/// Pinta a porta e devolve o `y` final do corpo da seção.
///
/// ⚠️ **Quem fecha o escopo é o CHAMADOR** (`fold.finish`) — um `return` cru a partir daqui não
/// existe, e é isso que impede o recorte de ficar pendurado na cena (o `Drop` do `SectionFold`
/// grita em debug por isto).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_empty_face(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    h: f32,
) -> f32 {
    let rect = Rect::new(x, y, w, h);
    let btn = Button::new(
        ids::INSP_PLAYER_ADD,
        tr("panel.inspector.player.make_platform_player"),
    )
    .kind(ButtonKind::Default)
    .visual(store.button_visual(ids::INSP_PLAYER_ADD));
    paint_button(&btn, rect, scene, text_system, theme);
    hit_index.register(ids::INSP_PLAYER_ADD, rect);
    // ⚠️ **A cauda de um BLOCO tem UMA porta** (`control_gap_px`), e o gate
    // `the_tail_of_a_block_is_one_answer` apanhou este sítio no instante em que o corte o tornou uma
    // cauda de verdade: dentro do `player.rs` a mesma soma vivia como ARGUMENTO de uma chamada, e a
    // régua não a via. *Mover código não muda o que ele faz — muda o que as réguas conseguem ver.*
    //
    // ⚠️⚠️ **E o VALOR muda com ele: `6 px → 3 px`.** O `Spacing::Sm` é `6` e o `control_gap_px()` é
    // `Xs + 1 − 2 = 3` — o número que o dono pediu em 2026-09-07 (*«para ambos vamos colocar o padrão
    // de espaçamento de 3 px»*), e que esta face vazia nunca chegou a usar porque ela morreu na F3
    // **antes** daquela wave e voltou depois dela. ⛔ Não é regressão: é a face a entrar na escada
    // que os outros 78 sítios já entraram (`1 / 3 / 8`, os três degraus do `spacing.rs`).
    y + h + ph2d_tokens::control_gap_px()
}
