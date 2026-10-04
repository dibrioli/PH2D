//! ⭐⭐⭐ **O PAINEL DE CAMADAS SOBRE A PEÇA 3D** (`docs/3D/30` §4 — a W3).
//!
//! Com a tela da vista presa ([`super::screen_canvas`]) a pilha desta ferramenta é a da TELA — o traço
//! em voo, uma camada só. A pilha que o artista quer ver é a da PEÇA, e quem a tem é a escultura: ela
//! espelha-a aqui a cada quadro ([`PainterTool::sync_piece_layers`]), o painel mostra o espelho
//! ([`PainterTool::panel_layers`]) e cada gesto do painel vira um [`PieceLayerOp`] que a escultura
//! drena ([`PainterTool::take_piece_layer_ops`]) e aplica pela porta da pilha dela — que muda pilha e
//! planos juntos e grava o desfazer.
//!
//! ⚠️ **O metadado é calculado AQUI, com as mesmas funções da pilha 2D** (a leitura do fio é a do
//! [`super::layer_edit`]; os parâmetros, as funções puras dos ajustes), e viaja como a pilha inteira
//! ([`PieceLayerOp::Metadata`]); a porta do outro lado recusa o que mudar a ESTRUTURA. O que cria ou
//! apaga camadas — que tem planos — viaja como pedido tipado.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::PanelRowDrop;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_effects::adjustments::AdjustmentKind;

use super::layer_edit::{LayerEdit, apply_param_edit, decode};
use super::*;

/// ⭐⭐ **Um pedido do painel sobre a pilha da peça.**
#[derive(Clone, Debug, PartialEq)]
pub enum PieceLayerOp {
    /// Uma camada de pintura nova, transparente, no topo.
    NewLayer,
    /// Uma máscara para a camada.
    NewMask(RtLayerId),
    /// Um ajuste novo no topo.
    NewAdjustment(AdjustmentKind),
    /// A cópia da camada, logo acima dela.
    Duplicate(RtLayerId),
    /// Apaga a camada.
    Delete(RtLayerId),
    /// ⭐ **O metadado novo** (modo, opacidade, visibilidade, recorte, máscara invertida, activa,
    /// ordem, parâmetros de um ajuste) — a pilha inteira, com a MESMA estrutura. `gesture` é o id do
    /// controlo que a produziu quando é um arrasto: os pedidos seguidos do mesmo arrasto são UM passo
    /// de desfazer.
    Metadata {
        stack: LayerStack,
        gesture: Option<NodeId>,
    },
}

/// Os ids dos editores de ajuste que mandam um ARRASTO (um pedido por quadro do gesto).
fn is_drag_editor(id: NodeId) -> bool {
    [
        crate::ids::PAINTER_CURVE_EDIT,
        crate::ids::PAINTER_MIXER_EDIT,
        crate::ids::PAINTER_GRADIENT_EDIT,
        crate::ids::PAINTER_GRADIENT_COLOR,
        crate::ids::PAINTER_SELCOLOR_EDIT,
    ]
    .contains(&id)
}

impl PainterTool {
    /// ⭐ **O painel de camadas mostra a pilha da PEÇA?** — a tela da vista 3D está presa.
    #[must_use]
    pub fn panel_shows_the_piece(&self) -> bool {
        self.on_screen_canvas()
    }

    /// ⭐⭐ **A escultura espelha a pilha da peça** (`None` = a peça não tem plano de tinta fina). Fora
    /// da tela da vista o painel não o lê ([`Self::panel_layers`]) e soltar a tela deita-o fora.
    pub fn sync_piece_layers(&mut self, stack: Option<&LayerStack>) {
        if self.piece_layers.as_ref() != stack {
            self.piece_layers = stack.cloned();
            self.layers_revision = self.layers_revision.wrapping_add(1);
        }
    }

    /// ⭐ **Onde vive o raio de um efeito de vizinhança na peça** — a escultura publica-o com o
    /// espelho: as unidades da peça e o tamanho dela (`docs/3D/30` §14).
    pub fn sync_piece_units(&mut self, units: ph2d_painter_effects::adjustments::SpatialUnits) {
        self.piece_units = units;
    }

    /// ⭐ **As unidades dos raios que o painel mostra** — as da peça com a tela presa, px fora dela.
    #[must_use]
    pub fn panel_spatial_units(&self) -> ph2d_painter_effects::adjustments::SpatialUnits {
        if self.panel_shows_the_piece() {
            self.piece_units
        } else {
            ph2d_painter_effects::adjustments::SpatialUnits::Pixels
        }
    }

    /// ⭐ **A pilha que o painel de camadas mostra** — a da peça com a tela presa (`None` se ela não
    /// tem plano), a do documento fora dela.
    #[must_use]
    pub fn panel_layers(&self) -> Option<&LayerStack> {
        if self.panel_shows_the_piece() {
            self.piece_layers.as_ref()
        } else {
            Some(&self.layers)
        }
    }

    /// As linhas que o painel destaca — na peça, só a activa (ela não tem multi-selecção).
    #[must_use]
    pub fn panel_selection(&self) -> BTreeSet<RtLayerId> {
        if self.panel_shows_the_piece() {
            self.piece_layers
                .as_ref()
                .and_then(LayerStack::active)
                .into_iter()
                .collect()
        } else {
            self.selection()
        }
    }

    /// ⭐⭐ **Os pedidos do painel sobre a pilha da peça**, por ordem — drenados pela escultura.
    pub fn take_piece_layer_ops(&mut self) -> Vec<PieceLayerOp> {
        std::mem::take(&mut self.piece_ops)
    }

    /// ⭐ **A escultura diz porque recusou o último pedido** (`None` = aceitou) — o painel mostra a
    /// frase; nenhum botão do painel é mudo.
    pub fn set_piece_layer_refusal(&mut self, why: Option<String>) {
        self.piece_refusal = why;
    }

    /// A frase da última recusa da pilha da peça.
    #[must_use]
    pub fn piece_layer_refusal(&self) -> Option<&str> {
        self.piece_refusal
            .as_deref()
            .filter(|_| self.panel_shows_the_piece())
    }

    /// A tela soltou-se: o espelho e os pedidos que ninguém vai drenar saem com ela.
    pub(crate) fn forget_piece_layers(&mut self) {
        self.piece_refusal = None;
        if self.piece_layers.take().is_some() {
            self.layers_revision = self.layers_revision.wrapping_add(1);
        }
        self.piece_ops.clear();
    }

    /// ⭐⭐⭐ **Um evento do painel com a tela da vista presa** — `true` = era do painel de camadas, e
    /// foi para a pilha da peça (nunca para a da tela).
    pub(crate) fn route_piece_layer_event(&mut self, ev: &PanelEvent) -> bool {
        if !self.panel_shows_the_piece() {
            return false;
        }
        if let PanelEvent::Click(id) = ev
            && (*id == crate::ids::PAINTER_LAYERS_ADD_TEXTURE || *id == crate::ids::PAINTER_APPLY)
        {
            eprintln!("[painter] {id:?} is not offered on the 3D piece (the panel disables it)");
            return true;
        }
        let edit = match self.piece_layers.as_ref() {
            Some(m) => decode(ev, |id| layer_widget_in(m, id)),
            None => decode(ev, |_| None),
        };
        let Some(edit) = edit else {
            return false;
        };
        let gesture = match ev {
            PanelEvent::SetValue(id, _) => Some(*id),
            PanelEvent::SelectOption(id, _) if is_drag_editor(*id) => Some(*id),
            _ => None,
        };
        self.piece_edit(edit, gesture);
        true
    }

    /// Um pedido lido, sobre o espelho.
    fn piece_edit(&mut self, e: LayerEdit, gesture: Option<NodeId>) {
        use LayerEdit as E;
        let Some(mut m) = self.piece_layers.clone() else {
            eprintln!("[painter] a layer edit on a 3D piece without a fine-paint plane");
            return;
        };
        let active = m.active();
        let typed = match e {
            E::AddRaster => Some(PieceLayerOp::NewLayer),
            E::DuplicateActive => active.map(PieceLayerOp::Duplicate),
            E::DeleteActive => active.map(PieceLayerOp::Delete),
            E::MaskActive => active.map(PieceLayerOp::NewMask),
            E::AddAdjustment(k) => Some(PieceLayerOp::NewAdjustment(k)),
            E::AddGroup
            | E::ToggleAlphaLockActive
            | E::ToggleReferenceActive
            | E::ImpastoLevel(_)
            | E::ImpastoDepth(..)
            | E::MaskApply(_)
            | E::MaskView(_) => {
                eprintln!("[painter] {e:?} is not offered on the 3D piece (the panel disables it)");
                return;
            }
            E::Row(l, row) => {
                let _ = take_pending_select_mods(row);
                m.set_active(l);
                None
            }
            E::ToggleClipActive => {
                if let Some(a) = active {
                    let now = m.get(a).is_some_and(|l| l.clipping);
                    m.set_clipping(a, !now);
                }
                None
            }
            E::Visibility(l) => {
                let now = m.get(l).is_none_or(|l| l.visible);
                m.set_visible(l, !now);
                None
            }
            E::MoveUp(l) => {
                m.move_up(l);
                None
            }
            E::MoveDown(l) => {
                m.move_down(l);
                None
            }
            E::MaskInvert(l) => {
                if let Some(LayerKind::Mask(mk)) = m.get(l).map(|x| &x.kind) {
                    let inv = mk.inverted;
                    m.set_mask_inverted(l, !inv);
                }
                None
            }
            E::Opacity(l, v) => {
                m.set_opacity(l, v);
                None
            }
            E::Blend(l, mode) => {
                m.set_blend_mode(l, mode);
                None
            }
            E::Param(l, p) => {
                if let Some(adj) = m.adjustment_mut(l) {
                    apply_param_edit(&mut adj.params, &p, self.piece_units);
                }
                None
            }
        };
        match typed {
            Some(op) => self.piece_ops.push(op),
            None => self.piece_metadata(m, gesture),
        }
    }

    /// O metadado novo: o espelho passa a ele já (o pedido seguinte do mesmo quadro parte dele) e o
    /// pedido segue. Igual ao de antes = nada a pedir.
    fn piece_metadata(&mut self, m: LayerStack, gesture: Option<NodeId>) {
        if self.piece_layers.as_ref() == Some(&m) {
            return;
        }
        self.piece_layers = Some(m.clone());
        self.layers_revision = self.layers_revision.wrapping_add(1);
        self.piece_ops
            .push(PieceLayerOp::Metadata { stack: m, gesture });
    }

    /// ⭐ **Arrastar uma linha com a pilha da peça no painel** — a lei do 2D ([`reparent_in`]) sobre
    /// o espelho. `true` = era da peça.
    pub(crate) fn piece_reparent(&mut self, dragged: NodeId, drop: PanelRowDrop) -> bool {
        if !self.panel_shows_the_piece() {
            return false;
        }
        if let Some(mut m) = self.piece_layers.clone()
            && super::layers::reparent_in(&mut m, dragged, drop)
        {
            self.piece_metadata(m, None);
        }
        true
    }
}

/// `(camada, widget)` de um id por camada, sobre a pilha `stack`.
pub(crate) fn layer_widget_in(
    stack: &LayerStack,
    id: NodeId,
) -> Option<(RtLayerId, crate::ids::PainterLayerWidget)> {
    use crate::ids::{PainterLayerWidget, painter_layer_widget_id};
    stack.all_ids().find_map(|layer| {
        PainterLayerWidget::ALL
            .into_iter()
            .find(|&kind| painter_layer_widget_id(layer.0, kind) == id)
            .map(|kind| (layer, kind))
    })
}

#[cfg(test)]
#[path = "piece_layers_tests.rs"]
mod tests;
