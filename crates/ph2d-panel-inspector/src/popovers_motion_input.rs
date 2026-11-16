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
                    .map(|(n, ((rotulo, k), id))| {
                        // O campo que manda um PAR diz-o na linha («Transform · Position  (x, y)»).
                        let rotulo = if i.campos_par.contains(k) {
                            ph2d_i18n::tr_with(
                                "panel.inspector.motion_input.pair_field",
                                &[("c", rotulo)],
                            )
                        } else {
                            rotulo.clone()
                        };
                        DropdownOption::new(id, n, rotulo)
                    })
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

/// Quantas linhas a lista tem de mostrar abaixo do chip antes de preferir abrir para cima.
const LINHAS_ABAIXO: u8 = 5;

/// ⭐ **A lista abre ABAIXO do chip e rola** (a lista longa da Property cobria o topo do Inspector): a região
/// começa no chip enquanto couberem [`LINHAS_ABAIXO`] linhas por baixo; senão, a região inteira (abre para cima,
/// também com rolagem). A altura nunca passa do que a região tem.
pub(crate) fn abaixo_do_chip(chip: Rect, region: Rect) -> Rect {
    let fundo = region.y + region.h;
    let abaixo = fundo - (chip.y + chip.h);
    if abaixo >= chip.h * f32::from(LINHAS_ABAIXO) {
        Rect::new(region.x, chip.y, region.w, fundo - chip.y)
    } else {
        region
    }
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
        abaixo_do_chip(chip, region),
        store,
        scene,
        text_system,
        theme,
        hit_index,
    );
}
