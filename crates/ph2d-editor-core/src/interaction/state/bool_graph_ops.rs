//! **O DIAGRAMA da booleana viva** no [`WidgetStore`] — abrir, fechar, arrastar, publicar a vista,
//! e a fila de intenções.
//!
//! Irmão de [`super::chrome_ops`] pelo teto de 700 LOC (HR-18), e o corte é por ASSUNTO: lá moram
//! os mutadores curtos de chave/valor da chrome; aqui mora UM card, com a sua máquina inteira.
//!
//! ⚠️ **A vista é uma FOTOGRAFIA, não estado.** A shell reescreve-a a cada frame a partir do ECS,
//! e o card nunca a muta — é o que impede duas respostas para *"quais são as ligações deste
//! grupo?"*. Pela mesma razão o card não escreve o documento: ele empilha INTENÇÕES que a shell
//! drena ([`WidgetStore::take_bool_graph_intents`]).

use super::WidgetStore;
use crate::interaction::InteractiveState;
use crate::widget::ButtonState;
use crate::zones::Rect;

impl WidgetStore {
    /// **Abre o DIAGRAMA da booleana viva** no canto `(x, y)` da tela, e regista o X e a alça.
    ///
    /// A vista chega separada ([`Self::set_bool_graph_view`]) porque ela é reescrita a CADA frame
    /// pela shell, e isto acontece uma vez só.
    pub fn open_bool_graph(&mut self, x: f32, y: f32) {
        self.bool_graph = Some((x, y));
        self.bool_graph_dragging = None;
        self.bool_graph_intents.clear();
        for id in [
            crate::ids::VECTOR_BOOL_GRAPH_CLOSE,
            crate::ids::VECTOR_BOOL_GRAPH_HANDLE,
        ] {
            self.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }

    /// Fecha o diagrama. ⚠️ Deixa as intenções pendentes em paz — a shell drena-as no mesmo frame
    /// em que as encaminha, e engoli-las aqui perderia o último gesto do artista.
    pub fn close_bool_graph(&mut self) {
        self.bool_graph = None;
        self.bool_graph_dragging = None;
        self.bool_graph_drawn = None;
    }

    /// O canto pedido do card, ou `None` quando fechado.
    #[must_use]
    pub fn bool_graph_pos(&self) -> Option<(f32, f32)> {
        self.bool_graph
    }

    /// Desloca o card (o arrasto da banda de título). ⚠️ **Não** prende ao viewport aqui: quem
    /// prende é o painter, na hora de desenhar — assim o delta acumulado continua a ser
    /// `cursor − pega` e o arrasto nunca cria zona morta. É a lei do `move_onion_modal`.
    pub fn move_bool_graph(&mut self, dx: f32, dy: f32) {
        if let Some((x, y)) = self.bool_graph.as_mut() {
            *x += dx;
            *y += dy;
        }
    }

    /// A shell publica **o que o diagrama mostra** neste frame.
    pub fn set_bool_graph_view(&mut self, view: crate::widget::BoolGraphView) {
        self.bool_graph_view = view;
    }

    /// O que o diagrama mostra — lido pelo painter e pelo acerto do clique.
    #[must_use]
    pub fn bool_graph_view(&self) -> &crate::widget::BoolGraphView {
        &self.bool_graph_view
    }

    /// O painter publica **o retângulo que de facto desenhou** (já preso ao viewport).
    pub fn set_bool_graph_drawn(&mut self, rect: Option<Rect>) {
        self.bool_graph_drawn = rect;
    }

    /// O retângulo desenhado neste frame — a porta ÚNICA do acerto do clique. Ver o campo.
    #[must_use]
    pub fn bool_graph_drawn(&self) -> Option<Rect> {
        self.bool_graph_drawn
    }

    /// De que forma partiu o arrasto de ligação em curso.
    #[must_use]
    pub fn bool_graph_dragging(&self) -> Option<u64> {
        self.bool_graph_dragging
    }

    /// Arma (ou desarma, com `None`) o arrasto de ligação.
    pub fn set_bool_graph_dragging(&mut self, from: Option<u64>) {
        self.bool_graph_dragging = from;
    }

    /// Empilha uma intenção do diagrama para a shell drenar.
    pub fn push_bool_graph_intent(&mut self, intent: crate::widget::BoolGraphIntent) {
        self.bool_graph_intents.push(intent);
    }

    /// **Drena** as intenções — a shell chama isto uma vez por frame e escreve o documento.
    pub fn take_bool_graph_intents(&mut self) -> Vec<crate::widget::BoolGraphIntent> {
        std::mem::take(&mut self.bool_graph_intents)
    }

    /// The Onion modal's top-left `(x, y)` in screen px, or `None` when closed. The painter gates the
    /// card's render + hit registration on this being `Some`; the shell gates the store→onion read-back
    /// on it too.
    #[must_use]
    pub fn onion_modal_pos(&self) -> Option<(f32, f32)> {
        self.onion_modal
    }

    /// Offset the Onion modal's position by `(dx, dy)` screen px (the title-band drag). No-op when
    /// closed. Not clamped here — the painter clamps to the viewport when it draws (so the accumulated
    /// delta always equals `cursor − grab_offset` and the drag never dead-zones), mirroring
    /// [`Self::move_fill_modal`].
    pub fn move_onion_modal(&mut self, dx: f32, dy: f32) {
        if let Some((x, y)) = self.onion_modal.as_mut() {
            *x += dx;
            *y += dy;
        }
    }
}
