//! As linhas de corpo de cartão que um pintor de secção pede pela mão: campos num rect já arrumado,
//! barra com chip e amostras com código. Filho de `cards_corpo` pelo tecto de LOC.

use super::{Cell, Widget, ctx, glifo, letra_da_grandeza};
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::card_field::{CardFieldSpec, Glyph};
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::Eixos;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// O corpo de letra dos rótulos de um cartão, para o rótulo que um corpo pinta à mão.
pub const LABEL_PX: f32 = m::LABEL_PX;

/// Campos Figma dentro de um rect que o chamador já arrumou, sem rótulo nem marca (a linha dele
/// traz os dois). Com dois ids, as letras X e Y.
#[allow(clippy::too_many_arguments)]
pub fn fields_in(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    rotulo: &str,
    ids: &[NodeId],
    step: f64,
) {
    let spec = CardFieldSpec {
        step,
        scrub: step,
        default: None,
    };
    let n = ids.len();
    let cells: Vec<Cell<'_>> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            store.mark_card_field_at_paint(id, spec);
            Cell::new(Widget::Field {
                id,
                glyph: glifo(rotulo, Eixos::Xyzw, n, i),
            })
        })
        .collect();
    let _ = ctx(scene, text_system, store, hit_index, ink, r.x, r.w).row(&cells, r.y);
}

/// Uma amostra com o código num rect que o chamador já arrumou, sem rótulo nem marca.
#[allow(clippy::too_many_arguments)]
pub fn swatch_in(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    id: NodeId,
    rgba: [u8; 4],
) {
    let text = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
    let cell = Cell::new(Widget::Swatch {
        id,
        rgba: Some(rgba),
        text: &text,
        edit: None,
        detach: None,
    });
    let _ = ctx(scene, text_system, store, hit_index, ink, r.x, r.w).row(&[cell], r.y);
}

/// Uma barra com chip, no cartão: o campo é o chip, a barra não se pinta e a marca é a dela, que é o
/// canal (receita 11, «Slider + chip ligado»). Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn bar_field(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    chip: NodeId,
    bar: NodeId,
    spec: CardFieldSpec,
) -> f32 {
    store.mark_card_field_at_paint(chip, spec);
    let key = [bar];
    let cell = Cell::new(Widget::Field {
        id: chip,
        glyph: Glyph::Text(letra_da_grandeza(rotulo)),
    })
    .label(rotulo)
    .key(&key);
    ctx(scene, text_system, store, hit_index, ink, x, w).row(&[cell], y)
}

/// ⭐ **Amostras com o código, como no Fill**, em `colunas` por linha: o rótulo em cima e uma marca
/// para o grupo. Sem cor conhecida, a seleção diverge e o código diz «Mixed». A caixa inteira abre o
/// seletor. Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn swatch_grid(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    amostras: &[(NodeId, Option<[u8; 4]>)],
    colunas: usize,
) -> f32 {
    let ids: Vec<NodeId> = amostras.iter().map(|(id, _)| *id).collect();
    let codigos: Vec<String> = amostras
        .iter()
        .map(|(_, c)| {
            c.map_or_else(
                || ph2d_i18n::tr("panel.vector.mixed").to_owned(),
                |c| format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2]),
            )
        })
        .collect();
    let mut c = ctx(scene, text_system, store, hit_index, ink, x, w);
    let mut y = y;
    for (l, (fila, textos)) in amostras
        .chunks(colunas.max(1))
        .zip(codigos.chunks(colunas.max(1)))
        .enumerate()
    {
        let n = fila.len();
        let cells: Vec<Cell<'_>> = fila
            .iter()
            .zip(textos)
            .enumerate()
            .map(|(i, ((id, rgba), text))| {
                let cell = Cell::new(Widget::Swatch {
                    id: *id,
                    rgba: *rgba,
                    text,
                    edit: None,
                    detach: None,
                });
                let cell = if l == 0 && i == 0 {
                    cell.label(rotulo)
                } else {
                    cell
                };
                if l == 0 && i + 1 == n {
                    cell.key(&ids)
                } else {
                    cell
                }
            })
            .collect();
        y = c.row(&cells, y);
    }
    y
}
