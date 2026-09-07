//! ⭐⭐⭐ **AS PORTAS DE GRUPO deste painel** — o stepper, o mostrador e o bloco de botões.
//!
//! ⚠️ **Saiu do `paint.rs` porque o teto de 600 LOC o mandou, e a linha do corte é a pergunta:** o
//! pai responde *como se pinta UMA peça* (um botão, um interruptor, um ícone, a lei de cor deles) e
//! este responde *como N peças formam um CORPO*. É o mesmo corte que o `panel_chrome` → `segmented`
//! fez na wave 11 — ficheiro irmão sob o mesmo módulo, nenhum caminho de chamada muda.
//!
//! Report do dono que os obrigou (2026-09-07): *«ficaria mais pro se as setas ficassem no mesmo
//! grupo dos botões Apply, Save e Load»*.

use crate::paint::{ClippedHits, button_in_group};
use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{paint_text_centered, resolve};
use ph2d_editor_core::widget::GroupCell;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// **A largura de uma seta de selector** (`◀` / `▶`).
///
/// ⚠️ **Ela estava declarada TRÊS vezes** — uma em cada ficheiro que desenha um selector
/// (`paint_fx`, `paint_variation`, `paint_delivery`), com o mesmo `26.0` escrito à mão. Nenhum
/// teste podia ver isso: cada cópia estava certa sozinha. *É a mesma espécie do `Bg1` com doze
/// cópias e do vão de linha atrás de um campo.*
pub(crate) const ARROW_W: f32 = 26.0; // LITERAL-PX-OK: selector arrow button width (chrome)

/// **Um selector `◀ nome ▶` SOZINHO** — o mesmo corpo, sem fileira por baixo.
///
/// ⚠️ Existe porque a casa tem **três** destes (preset, estratégia de variação, formato de
/// entrega) e só um tem ordens por baixo. Deixá-lo escrever a disposição à mão devolvia o defeito
/// que o dono apontou, num sítio de cada vez.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stepper_row(
    rect: Rect,
    name: &str,
    enabled: bool,
    prev: NodeId,
    next: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let mid = ph2d_editor_core::widget::stepper_middle_w(rect.w, ARROW_W);
    let cells = ph2d_editor_core::widget::block_cells_of(
        Rect::new(rect.x, rect.y, rect.w, 0.0),
        &[&[ARROW_W, mid, ARROW_W]],
        rect.h,
    );
    let cells = &cells[0];
    button_in_group(
        cells[0].0,
        "\u{25c0}",
        enabled,
        prev,
        cells[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    display_in_group(
        cells[1].0,
        name,
        enabled,
        cells[1].1,
        scene,
        text_system,
        theme,
    );
    button_in_group(
        cells[2].0,
        "\u{25b6}",
        enabled,
        next,
        cells[2].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    rect.y + rect.h
}

/// ⭐⭐⭐ **N botões em várias fileiras, e o conjunto é UM CORPO** (wave 20, report do dono:
/// *«tudo o que puder ser ajuntado, ajunte; apenas quando o grupo for nitidamente de função
/// diferente é que deve permanecer afastado»*).
///
/// ⚠️ **Este painel já agrupava na HORIZONTAL e não na VERTICAL:** o `segment_rects` juntava
/// *Add Marker | Delete*, e a seguir um `y += row_h + gap` escrito à mão punha *Split at Markers*
/// **fora** do corpo — três botões do mesmo assunto lidos como dois controlos. A lei do grupo tem
/// duas dimensões desde a wave 11 (`block_cells`); o que faltava era um sítio por onde este painel
/// a alcançasse.
///
/// Devolve o `y` **imediatamente abaixo** do bloco — sem vão: quem chama decide se o que vem a
/// seguir é o mesmo assunto (nada) ou outro (`control_gap_px`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn buttons_block(
    rect: Rect,
    cols_per_row: &[usize],
    items: &[(&str, bool, NodeId)],
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    debug_assert_eq!(
        items.len(),
        cols_per_row.iter().sum::<usize>(),
        "a grelha declarada nao cobre os botoes"
    );
    let block = ph2d_editor_core::widget::block_cells(
        Rect::new(rect.x, rect.y, rect.w, 0.0),
        cols_per_row,
        rect.h,
    );
    let mut i = 0usize;
    for (r, count) in cols_per_row.iter().enumerate() {
        for k in 0..*count {
            let (label, enabled, id) = items[i + k];
            let (seg, cell) = block[r][k];
            button_in_group(
                seg,
                label,
                enabled,
                id,
                cell,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
        i += count;
    }
    rect.y + ph2d_editor_core::widget::grid_height(cols_per_row.len(), rect.h)
}

/// **O MOSTRADOR de um stepper** — a peça do meio de `◀ nome ▶`, que diz o que as setas escolhem.
///
/// ⚠️ **Ela tem superfície e NÃO é clicável, e as duas metades são deliberadas:** sem superfície o
/// corpo do stepper tem um buraco no meio e lê-se como duas setas soltas; com alvo de clique
/// prometeria um gesto que não existe (o nome não abre lista nenhuma — quem escolhe são as setas).
/// ⛔ Por isso ela não passa pelo `hit_index`: *uma affordance que o painel não pode honrar é pior
/// que nenhuma.*
///
/// O tom é o do repouso de um botão (`Bg3`) — a peça pertence ao corpo —, e o texto segue o
/// `enabled` das setas, que é o que diz se há alguma coisa para escolher.
pub(crate) fn display_in_group(
    rect: Rect,
    label: &str,
    enabled: bool,
    pos: GroupCell,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    ph2d_editor_core::paint::fill_rounded_rect_radii(
        scene,
        rect,
        pos.radii(ph2d_editor_core::paint::frame_radius(
            theme,
            Radius::Sm.px(),
        )),
        resolve(ColorToken::Bg3, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(
            if enabled {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            },
            theme,
        ),
    );
}
