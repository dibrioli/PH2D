//! A ferramenta de osso: o estado que o painel Bones autora e que o arrasto lê.

use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};

use crate::ids;
use crate::params::{
    BoneAction, WEIGHT_AMOUNT_DEFAULT, WEIGHT_RADIUS_DEFAULT, WEIGHT_RADIUS_MIN, WeightDirection,
    WeightMode,
};

/// O id da ferramenta no registo.
pub const BONE: &str = "bone";

/// ⭐ **O que a shell espelha por quadro** — quem o lê é o despacho do ponteiro e o overlay, e
/// alcançar a ferramenta por downcast num handler de press seria trabalho por evento.
///
/// ⛔ Não é estado do documento: é o pincel. Um `.ph2dproj` não o guarda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoneConfig {
    /// O verbo do arrasto (Create · Transform · Weight).
    pub action: BoneAction,
    /// O raio do pincel de peso, em PÍXEIS DE ECRÃ ([`WEIGHT_RADIUS_DEFAULT`]).
    pub weight_radius: f64,
    /// QUANTO cada pincelada empurra — uma magnitude em `0..1`.
    pub weight_amount: f64,
    /// PARA QUE LADO ela empurra.
    pub weight_direction: WeightDirection,
    /// COMO ela atribui o peso.
    pub weight_mode: WeightMode,
}

impl Default for BoneConfig {
    fn default() -> Self {
        Self {
            action: BoneAction::default(),
            weight_radius: WEIGHT_RADIUS_DEFAULT,
            weight_amount: WEIGHT_AMOUNT_DEFAULT,
            weight_direction: WeightDirection::default(),
            weight_mode: WeightMode::default(),
        }
    }
}

/// ⭐ **A ferramenta.** O estado é o [`BoneConfig`]; o painel escreve-o por [`PanelEvent`].
#[derive(Debug, Default)]
pub struct BoneTool {
    config: BoneConfig,
}

impl BoneTool {
    /// O que a shell espelha.
    #[must_use]
    pub fn config(&self) -> BoneConfig {
        self.config
    }

    /// ⭐ O verbo armado pela shell (a aresta do foco, o modo) — a MESMA porta do clique no
    /// segmento: um segundo caminho divergiria no dia em que um deles ganhasse um efeito.
    pub fn set_action(&mut self, action: BoneAction) {
        self.config.action = action;
    }
}

impl Tool for BoneTool {
    fn id(&self) -> ToolId {
        ToolId::new(BONE)
    }

    fn label(&self) -> &str {
        "Bone"
    }

    fn icon_slug(&self) -> &str {
        "bone"
    }

    fn build_panel(&self) -> FloatingPanel {
        FloatingPanel::new(self.id(), "Bone")
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        let c = &mut self.config;
        match event {
            PanelEvent::Click(id) if let Some(a) = BoneAction::of_segment(id) => c.action = a,
            // ⛔ Escolher um lado ou um modo NÃO arma o verbo `Weight`: a secção destes só é pintada
            // com ele já na mão, e armá-lo arrancaria o artista do que estava a fazer.
            PanelEvent::Click(id) if id == ids::VECTOR_BONE_WEIGHT_ADD => {
                c.weight_direction = WeightDirection::Add;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_BONE_WEIGHT_SUB => {
                c.weight_direction = WeightDirection::Subtract;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_BONE_WEIGHT_CUMUL => {
                c.weight_mode = WeightMode::Cumulative;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_BONE_WEIGHT_ABS => {
                c.weight_mode = WeightMode::Absolute;
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_BONE_WEIGHT_RADIUS => {
                c.weight_radius = v.max(WEIGHT_RADIUS_MIN);
            }
            // ⚠️ Um negativo escrito à mão entra em valor ABSOLUTO (cortá-lo a zero deixaria o pincel
            // inerte e calado); o tecto é `1`, o peso vive em `0..1`.
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_BONE_WEIGHT_AMOUNT => {
                c.weight_amount = v.abs().clamp(0.0, 1.0);
            }
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "tool_tests.rs"]
mod tests;
