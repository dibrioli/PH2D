//! **As duas listas do MOTION INPUT** (GEF 08 onda 2): a acção do Input Map (fonte Action) e o campo lido
//! (fonte Property) — o molde da «Shape» do Ray Sensor: o chip guarda o rect num slot, e a lista pinta-se
//! no passe diferido, por cima de tudo. As opções re-derivam-se do instantâneo (a lei da rederivação).

use super::paint_open_popover;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::motion_input_edits::InspectorMotionInputInfo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Qual das duas listas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lista {
    Action,
    Property,
}

impl Lista {
    /// O id do chip.
    pub(crate) const fn id(self) -> ph2d_a11y::NodeId {
        match self {
            Self::Action => crate::ids::INSP_MI_ACTION,
            Self::Property => crate::ids::INSP_MI_PROPERTY,
        }
    }

    /// A tabela de ids das opções.
    pub(crate) const fn opcoes_ids(self) -> &'static crate::ids::OpcoesDeTag {
        match self {
            Self::Action => &crate::ids::INSP_MI_ACTION_OPT,
            Self::Property => &crate::ids::INSP_MI_PROPERTY_OPT,
        }
    }

    /// Quantas opções o instantâneo dá.
    pub(crate) fn len(self, i: &InspectorMotionInputInfo) -> usize {
        match self {
            Self::Action => i.acoes_do_mapa.len(),
            Self::Property => i.campos_do_alvo.len(),
        }
    }

    /// As opções (valor = posição) e a escolhida.
    pub(crate) fn opcoes(
        self,
        i: &InspectorMotionInputInfo,
    ) -> (Vec<DropdownOption<usize>>, Option<usize>) {
        let ids = self.opcoes_ids().ids(self.len(i));
        match self {
            Self::Action => (
                i.acoes_do_mapa
                    .iter()
                    .zip(ids)
                    .enumerate()
                    .map(|(n, (a, id))| DropdownOption::new(id, n, a.as_str()))
                    .collect(),
                i.acoes_do_mapa.iter().position(|a| *a == i.action),
            ),
            Self::Property => (
                i.campos_do_alvo
                    .iter()
                    .zip(ids)
                    .enumerate()
                    .map(|(n, ((rotulo, _), id))| DropdownOption::new(id, n, rotulo.as_str()))
                    .collect(),
                i.campos_do_alvo.iter().position(|(_, k)| *k == i.property),
            ),
        }
    }
}

thread_local! {
    static PENDING_MI_DD: std::cell::Cell<Option<(Lista, Rect)>> = const { std::cell::Cell::new(None) };
}

/// O chip aberto guarda-se aqui no passe normal.
pub(crate) fn set_pending(v: Option<(Lista, Rect)>) {
    PENDING_MI_DD.with(|c| c.set(v));
}

/// Pinta a lista aberta (se houver) no passe diferido.
pub(super) fn paint_deferred(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    region: Rect,
) {
    let Some((lista, chip)) = PENDING_MI_DD.with(std::cell::Cell::take) else {
        return;
    };
    let Some(info) = crate::state_components::current_inspector_motion_input() else {
        return;
    };
    let (opcoes, sel) = lista.opcoes(&info);
    let mut dd = Dropdown::new(lista.id(), "", opcoes).open(true);
    if let Some(n) = sel {
        dd.select(n);
    }
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
