//! ⭐⭐⭐ **OS TRÊS POPOVERS DE TAG do Inspector** (TOP-20 #9) — o filtro da §11 Physics, a tag
//! ALVO de uma *Signal Action*, e a caixa de escolha da secção *Tags*.
//!
//! ⚠️ **Irmão de [`super`] por tecto de FUNÇÃO** (200): o terceiro levou o
//! `paint_deferred_popovers` a `210`. ⛔ *A cura de um tecto estourado é o corte; subir o número é
//! adiar com juros.*
//!
//! ⚠️ **E o corte é por ASSUNTO:** os quatro que ficam no pai escolhem um ENUM do objecto
//! (amostragem, camada de ordenação, âncora, verbo); estes três escolhem uma **tag da árvore do
//! projecto**, e as opções deles saem de outra porta — a do documento, não a do instantâneo.
//!
//! ⚠️ **A lei da REDERIVAÇÃO vale aqui tal e qual:** o slot guarda só o rect do chip, e as opções
//! re-derivam-se do instantâneo. *Guardá-las seria uma cópia do documento a envelhecer entre o
//! clique e o quadro.*

use super::paint_open_popover;
use crate::state_popovers;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::Dropdown;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Pinta os três, na ordem em que as secções deles aparecem. Nenhum deles devolve `y`: um popover
/// sai da coluna por construção.
pub(super) fn paint_deferred_tag_popovers(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    region: Rect,
) {
    // ⭐⭐⭐ **O FILTRO por tag da §11 PHYSICS** (TOP-20 #9, W3c) — terceiro slot, opções próprias
    // (com o `(any)` à frente, que LIMPA o filtro).
    if let Some(chip) = state_popovers::take_pending_phys_tag_dd() {
        let dd = Dropdown::new(
            crate::ids::INSP_PHYS_SIGNAL_TAG,
            "",
            crate::sections::physics_rows::phys_tag_options(),
        )
        .placeholder(ph2d_i18n::tr("panel.tags.only_for"))
        .open(true);
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    // ⭐⭐⭐ **A TAG ALVO de uma SIGNAL ACTION** (TOP-20 #9, W3b) — slot próprio, opções próprias.
    // ⚠️ Elas são a árvore INTEIRA (uma acção pode apontar a qualquer tag), ao contrário da secção
    // *Tags*, onde a lista tira as que o objecto já tem.
    if let Some(chip) = state_popovers::take_pending_action_tag_dd() {
        let dd = Dropdown::new(
            crate::ids::INSP_ACTION_TAG_PICK,
            "",
            crate::sections::actions::tag_options(),
        )
        .placeholder(ph2d_i18n::tr("panel.tags.pick"))
        .open(true);
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    // ⭐⭐⭐ **TAGS** (TOP-20 #9) — a mesma máquina, e a lei da rederivação levada até ao fim: aqui
    // **só o rect** viaja no slot. As opções saem do snapshot MAIS o texto da busca, que vive no
    // store — e este passe tem os dois. ⚠️ Se o texto fosse guardado no slot, a lista pintada podia
    // ficar um quadro atrás do que o artista está a escrever.
    if let Some(chip) = state_popovers::take_pending_tags_dd()
        && let Some(info) = crate::state::current_inspector_tags()
    {
        let escrito = match store.get(crate::ids::INSP_TAGS_NEW) {
            Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) => {
                text.clone()
            }
            _ => String::new(),
        };
        let dd = Dropdown::new(
            crate::ids::INSP_TAGS_PICK,
            "",
            crate::sections::tags::pick_options(
                &crate::state::current_tag_tree(),
                &info.on_object,
                &ph2d_label_fold::fold(&escrito),
            ),
        )
        .placeholder(ph2d_i18n::tr("panel.tags.pick"))
        .open(true);
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
}
