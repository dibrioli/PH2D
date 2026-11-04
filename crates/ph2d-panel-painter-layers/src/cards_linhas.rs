//! As peças comuns das linhas de cartão do painel Painter (`docs/CartoesUI/13_PAINTER_PLANO.md`): as listas de caixa do
//! cartão, a linha esmaecida da lei, e os pares.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::widget::DropdownState;
use ph2d_vector::{Affine, Compose, Mix, RoundedRect, VelloBlend};

/// As listas destes cartões: a caixa é a do cartão, e a lista que abre é a de sempre (a do Preset arrasta linhas).
pub(crate) fn registra_lista(store: &mut WidgetStore, id: NodeId, atual: usize) {
    store.register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(atual),
        },
    );
}

pub(crate) fn aberta(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Os itens de dois em dois, na mesma linha, cada um na sua metade; o que sobra fica com a linha inteira.
pub(crate) fn em_pares<T>(
    c: &mut CardCtx<'_>,
    itens: &[T],
    y: f32,
    mut pinta: impl FnMut(&mut CardCtx<'_>, &T, f32) -> f32,
) -> f32 {
    let (x, w) = (c.x, c.w);
    let meia = ((w - m::FIELD_GAP) * 0.5).max(0.0);
    let mut topo = y;
    for par in itens.chunks(2) {
        let mut fundo = topo;
        for (i, item) in par.iter().enumerate() {
            if par.len() == 2 {
                c.x = x + i as f32 * (meia + m::FIELD_GAP);
                c.w = meia;
            }
            fundo = fundo.max(pinta(c, item, topo));
        }
        topo = fundo;
    }
    c.x = x;
    c.w = w;
    topo
}

/// A altura de uma linha com rótulo, de sobra: a camada só recorta, nunca empurra.
fn altura_de_linha() -> f32 {
    m::LABEL_TOP + 2.0 * (m::LABEL_PX + m::LABEL_BOTTOM) + m::FIELD_ROW_GAP + m::FIELD_H
}

/// Esmaecida como a linha de antes: continua editável, salvo a do Color inativo, que o chamador tira do índice.
pub(crate) fn esmaecida(
    c: &mut CardCtx<'_>,
    y: f32,
    pintar: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    let forma = RoundedRect::new(
        f64::from(c.x),
        f64::from(y),
        f64::from(c.x + c.w),
        f64::from(y + altura_de_linha()),
        0.0,
    );
    c.scene.push_layer_shape(
        VelloBlend::new(Mix::Normal, Compose::SrcOver),
        ph2d_editor_core::property_row::inativa::alfa_inativo(c.ink.theme),
        Affine::IDENTITY,
        &forma,
    );
    let fim = pintar(c, y);
    c.scene.pop_layer();
    fim
}

/// A linha do id `id`, esmaecida se a ferramenta o diz inerte.
pub(crate) fn com_lei(
    c: &mut CardCtx<'_>,
    id: NodeId,
    y: f32,
    pintar: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    if state::current_inercias().dica_de(id).is_some() {
        esmaecida(c, y, pintar)
    } else {
        pintar(c, y)
    }
}
