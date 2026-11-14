//! ⭐ **T61 — os estados do sujeito PRESO de um cartão solto** (`docs/CartoesUI/04_plano_de_construcao.md`,
//! bloco 10). O mesmo id mostra, na lista, a seleção e, no solto preso, o objecto dele. O painel
//! mede-os por ENSAIO (semeia as caixas com o retrato do preso e guarda só o que mudou); a pintura
//! do solto troca-os no store à volta do corpo, e a pressão sobre ele deixa-os no store para o gesto.

use super::{InteractiveState, NumberInputDragState, WidgetStore};
use ph2d_a11y::NodeId;
use std::collections::BTreeMap;

/// O que [`WidgetStore::abre_ensaio`] guardou para repor.
#[derive(Debug)]
pub struct Ensaio {
    estados: BTreeMap<NodeId, InteractiveState>,
    cores: BTreeMap<NodeId, [u8; 4]>,
    foco: Option<NodeId>,
    arrasto: Option<NumberInputDragState>,
    ativo: Option<NodeId>,
    seletor: Option<NodeId>,
}

/// Os estados de um sujeito preso que diferem dos da seleção (caixas, listas, cores).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EstadosPresos {
    estados: Vec<(NodeId, InteractiveState)>,
    cores: Vec<(NodeId, [u8; 4])>,
}

impl EstadosPresos {
    #[must_use]
    pub fn estado(&self, id: NodeId) -> Option<&InteractiveState> {
        self.estados.iter().find(|(i, _)| *i == id).map(|(_, s)| s)
    }
}

/// O que [`WidgetStore::troca_estados_presos`] tirou do store (`None` = o id não existia).
#[derive(Debug, Default)]
#[must_use]
pub struct Trocados {
    estados: Vec<(NodeId, Option<InteractiveState>)>,
    cores: Vec<(NodeId, Option<[u8; 4]>)>,
}

impl WidgetStore {
    /// Abre o ensaio: guarda estados e cores e larga foco, arrasto, botão premido e seletor de cor
    /// (nenhuma caixa se defende — a semente escreve-as todas).
    pub fn abre_ensaio(&mut self) -> Ensaio {
        let estados = (*self.states).clone();
        // Toda caixa de número fica sem valor: a que a semente escrever é do preso, mesmo que hoje dê o número da seleção.
        for s in self.states.values_mut() {
            if let InteractiveState::NumberInput { value, .. } = s {
                *value = f64::NAN;
            }
        }
        Ensaio {
            estados,
            cores: self.widget_colors.clone(),
            foco: self.focus_id.take(),
            arrasto: self.number_input_drag.take(),
            ativo: self.active_id.take(),
            seletor: self.picker_target.take(),
        }
    }

    /// Fecha o ensaio: o que mudou desde [`Self::abre_ensaio`], com tudo reposto como estava.
    pub fn fecha_ensaio(&mut self, e: Ensaio) -> EstadosPresos {
        let estados = self
            .states
            .iter()
            .filter(|(id, s)| match s {
                // ⛔ Sem diferença o cartão mostraria a caixa partilhada: o número que a mão arrasta noutro cartão.
                InteractiveState::NumberInput { value, .. } => !value.is_nan(),
                _ => e.estados.get(id) != Some(*s),
            })
            .map(|(id, s)| (*id, s.clone()))
            .collect();
        let cores = self
            .widget_colors
            .iter()
            .filter(|(id, c)| e.cores.get(id) != Some(*c))
            .map(|(id, c)| (*id, *c))
            .collect();
        self.states.substitui(e.estados);
        self.widget_colors = e.cores;
        self.focus_id = e.foco;
        self.number_input_drag = e.arrasto;
        self.active_id = e.ativo;
        self.repoe_picker_target(e.seletor);
        EstadosPresos { estados, cores }
    }

    /// Os estados do sujeito preso do solto `section`, medidos pelo painel neste quadro.
    pub fn set_pinned_states(&mut self, section: NodeId, e: EstadosPresos) {
        self.card_lists.pinned_states.insert(section, e);
    }

    #[must_use]
    pub fn pinned_states(&self, section: NodeId) -> Option<&EstadosPresos> {
        self.card_lists.pinned_states.get(&section)
    }

    /// Põe no store os estados do preso de `section` e devolve os que tirou ([`Self::repoe_estados`]
    /// desfaz). A caixa que o gesto sobre ESTE preso edita fica (é a edição viva dele).
    pub fn troca_estados_presos(&mut self, section: NodeId) -> Trocados {
        let Some(e) = self.card_lists.pinned_states.get(&section).cloned() else {
            return Trocados::default();
        };
        let pin = self.card_lists.loose.get(&section).and_then(|c| c.pin);
        let gesto = pin.is_some() && self.card_lists.press_subject == pin;
        let vivas = [
            self.focus_id,
            self.number_input_drag.map(|d| d.id),
            self.active_id,
        ];
        let mut t = Trocados::default();
        for (id, s) in e.estados {
            if !(gesto && vivas.contains(&Some(id))) {
                t.estados.push((id, self.states.insert(id, s)));
            }
        }
        for (id, c) in e.cores {
            t.cores.push((id, self.widget_colors.insert(id, c)));
        }
        t
    }

    /// Desfaz uma [`Self::troca_estados_presos`].
    pub fn repoe_estados(&mut self, t: Trocados) {
        for (id, s) in t.estados.into_iter().rev() {
            match s {
                Some(s) => self.states.insert(id, s),
                None => self.states.remove(&id),
            };
        }
        for (id, c) in t.cores.into_iter().rev() {
            match c {
                Some(c) => self.widget_colors.insert(id, c),
                None => self.widget_colors.remove(&id),
            };
        }
    }

    /// A pressão caiu no solto preso `section`: o gesto parte dos estados do preso. O campo em foco fica:
    /// o que nele se escreveu ainda vai ao commit de quem o abriu.
    pub(super) fn poe_estados_presos(&mut self, section: NodeId) {
        let Some(e) = self.card_lists.pinned_states.get(&section).cloned() else {
            return;
        };
        let foco = self.focus_id;
        self.states
            .extend(e.estados.into_iter().filter(|(id, _)| Some(*id) != foco));
        self.widget_colors.extend(e.cores);
    }
}
