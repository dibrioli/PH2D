//! Read-only convenience accessors on [`WidgetStore`] for each
//! widget kind.
//!
//! Extracted from [`super`] (Track D7). All these methods are pure
//! reads — they match against the [`InteractiveState`] variant and
//! return the relevant fields. Painters lean on these heavily to
//! avoid open-coding the same match arms everywhere.

use super::WidgetStore;
use crate::interaction::InteractiveState;
use crate::widget::{
    ButtonState, CheckboxState, CheckboxValue, SliderState, TextInputState, ToggleState,
};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// **A PORTA ÚNICA de *«como este botão se pinta AGORA?»*** — o estado DISCRETO e quanto do
    /// hover está presente, numa pergunta só e num argumento só.
    ///
    /// ⚠️ **Ela nasceu porque o app já a tinha inventado QUATRO vezes em privado — e as quatro não
    /// eram a mesma.** Três (`context_menu_dialogs`, `fill_modal`, `onion_modal`) derivavam de
    /// `hot_id`/`active_id`; a quarta (o `paint_helpers` do grid-snap) repetia a lei do store. Esta
    /// porta **subsume as duas**, e a ordem entre elas é a correcção: o estado PRÓPRIO de um botão
    /// registado vence, e só quando não há registo é que o ponteiro responde.
    ///
    /// ⚠️ **A fallback não é cortesia — é o que torna um botão de MODAL vivo.** Um botão que o
    /// `populate` não registou não tem `InteractiveState::Button`, então a lei do store devolveria
    /// `Normal` para sempre: um botão pintado que nunca acende.
    ///
    /// ⚠️ **E o `t` sai do store, não de um segundo argumento** — ver [`Self::hover_live`]. O
    /// neutro é `1.0` = *assente no estado*: um id que o relógio nunca viu pinta exactamente o que
    /// pintava antes de a UI viva existir, que é o que mantém todo gate de painel a medir o que
    /// media.
    #[must_use]
    pub fn button_visual(&self, id: NodeId) -> (ButtonState, f32) {
        let state = self.button_state(id).unwrap_or_else(|| {
            if self.active_id() == Some(id) {
                ButtonState::Pressed
            } else if self.hot_id() == Some(id) {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            }
        });
        (state, self.hover_live(id))
    }

    /// Quanto do hover está PRESENTE neste id (`0.0`..=`1.0`); [`crate::motion::SETTLED`] para quem
    /// o relógio nunca viu.
    ///
    /// ⚠️ O neutro é `1.0` e não `0.0`: ele significa *assente no estado que o widget diz ter*, e é
    /// por isso que um id sem track pinta o mundo pré-UI-viva byte a byte. O nome mora no substrato
    /// para um sítio que FORÇA um estado duro poder dizer o mesmo neutro sem repetir o literal.
    #[must_use]
    pub fn hover_live(&self, id: NodeId) -> f32 {
        self.hover_live
            .get(&id)
            .copied()
            .unwrap_or(crate::motion::SETTLED)
    }

    /// **O par que a PELE consome** — o estado inteiro deste id e quanto da transição já passou.
    ///
    /// ⚠️ **Ela existe pela razão do [`Self::button_visual`], uma camada acima:** a
    /// [`crate::widget::paint_widget_skin_with`] tomava só o `&InteractiveState`, então cada arm
    /// copiava o estado DISCRETO e deixava o `hover_t` no default — o painel gerado **reagia e
    /// SALTAVA** enquanto os vinte e três painéis escritos à mão amaciavam. O par sai daqui
    /// **inteiro, de um id só**, e um par descasado deixa de ser exprimível no chamador.
    ///
    /// ⚠️ **`None` é a PRÉVIA, e é por isso que o tipo é `Option` e não um par com neutro:** um
    /// widget que o `populate` nunca registou não tem estado nenhum a mostrar, e a pele de canvas
    /// desenha os valores de prévia. Um `(Normal, SETTLED)` fabricado aqui apagaria essa distinção.
    #[must_use]
    pub fn skin_live(&self, id: NodeId) -> Option<(&InteractiveState, f32)> {
        self.states.get(&id).map(|st| (st, self.hover_live(id)))
    }

    /// **O tique publica aqui.** Único escritor, uma vez por quadro, imediatamente antes de todos os
    /// leitores — o gêmeo exacto do [`WidgetStore::set_panel_scroll_live`].
    pub fn set_hover_live(&mut self, id: NodeId, t: f32) {
        self.hover_live.insert(id, t);
    }

    /// Convenience: read button state.
    pub fn button_state(&self, id: NodeId) -> Option<ButtonState> {
        match self.states.get(&id) {
            Some(InteractiveState::Button { state }) => Some(*state),
            _ => None,
        }
    }

    /// **O par visual de um POLEGAR DE SCROLLBAR** — o irmão dos outros `*_visual`, e o único que
    /// não pergunta ao `states`.
    ///
    /// ⚠️ **Um polegar não tem estado GUARDADO, e é por isso que ele estava mudo.** O arrasto vive
    /// no `scrollbar_drag()` (chaveado pelo PAINEL) e o hover no `hot_id()` (chaveado pelo
    /// POLEGAR) — duas chaves diferentes para o mesmo widget. Os 24 sítios que pintam uma barra
    /// respondiam só à primeira, cada um com a sua linha de `matches!(..d.panel == X)`, e nenhum à
    /// segunda: **nenhuma barra do app acendia sob o ponteiro**.
    ///
    /// A ponte entre as duas chaves já existia e tinha um dono — o `scrollbar_panel_for_id`, que o
    /// despachante mantém —, então perguntar aqui **remove** uma linha de cada sítio em vez de
    /// acrescentar uma: o chamador passa o id do polegar e mais nada.
    #[must_use]
    pub fn scrollbar_visual(&self, thumb: NodeId) -> (crate::widget::ScrollbarState, f32) {
        let panel = crate::interaction::dispatch::scroll::scrollbar_panel_for_id(thumb);
        self.scrollbar_visual_for(thumb, panel)
    }

    /// O mesmo, para uma barra cujo PAINEL não está no mapa do despachante.
    ///
    /// ⚠️ **A do popover de um dropdown é chaveada pelo CHIP**, não por um id de painel — só há um
    /// dropdown aberto de cada vez, e é o chip que diz qual. Ela é a razão de a primitiva receber
    /// o painel: sem esta porta, ou o popover perdia a metade do arrasto, ou o mapa do despachante
    /// ganhava uma entrada que descreve uma coisa que não é um painel.
    #[must_use]
    pub fn scrollbar_visual_for(
        &self,
        thumb: NodeId,
        panel: Option<NodeId>,
    ) -> (crate::widget::ScrollbarState, f32) {
        use crate::widget::ScrollbarState as S;
        let dragging =
            panel.is_some_and(|p| matches!(self.scrollbar_drag(), Some(d) if d.panel == p));
        let state = if dragging {
            S::Dragging
        } else if self.hot_id() == Some(thumb) {
            S::Hovered
        } else {
            S::Normal
        };
        (state, self.hover_live(thumb))
    }

    /// **O par visual de um CHECKBOX**: o estado + quanto do hover está presente.
    ///
    /// ⚠️ **Ele nasceu porque a caixa deste app nunca reagiu ao rato.** Medido: `Checkbox::new`
    /// nasce em `CheckboxState::Normal` e **nenhum sítio de produção chamava `.state(..)`** — as
    /// ~20 caixas do editor pintavam o estado de repouso para sempre, hovered ou não, enquanto o
    /// store já sabia quem estava sob o ponteiro (o `hover_targets` sempre as incluiu). O `t` era a
    /// metade que faltava a seguir; a que faltava primeiro era o próprio ESTADO.
    ///
    /// ⚠️ **O `value` NÃO sai daqui, de propósito.** Quem sabe se a caixa está marcada é o MODELO
    /// que o painel desenha, não o store — perguntá-lo aqui poria o mesmo facto em dois sítios, e
    /// o do store fica um quadro atrás no quadro em que o artista clica.
    #[must_use]
    pub fn checkbox_visual(&self, id: NodeId) -> (CheckboxState, f32) {
        let state = self
            .checkbox(id)
            .map(|(s, _)| s)
            .unwrap_or_else(|| match () {
                () if self.active_id() == Some(id) => CheckboxState::Pressed,
                () if self.hot_id() == Some(id) => CheckboxState::Hovered,
                () => CheckboxState::Normal,
            });
        (state, self.hover_live(id))
    }

    /// **O par visual de um TOGGLE** — o irmão exacto do [`Self::checkbox_visual`], e pelo mesmo
    /// motivo: o `on` é do modelo, o estado e o `t` são do store.
    #[must_use]
    pub fn toggle_visual(&self, id: NodeId) -> (ToggleState, f32) {
        let state = self.toggle(id).map(|(s, _)| s).unwrap_or_else(|| match () {
            () if self.active_id() == Some(id) => ToggleState::Pressed,
            () if self.hot_id() == Some(id) => ToggleState::Hovered,
            () => ToggleState::Normal,
        });
        (state, self.hover_live(id))
    }

    /// Convenience: read toggle on/off + state.
    pub fn toggle(&self, id: NodeId) -> Option<(ToggleState, bool)> {
        match self.states.get(&id) {
            Some(InteractiveState::Toggle { state, on }) => Some((*state, *on)),
            _ => None,
        }
    }

    /// **O par visual de um SLIDER** — o quarto irmão de [`Self::button_visual`], e o primeiro
    /// para uma superfície que se ARRASTA em vez de se clicar.
    ///
    /// ⚠️ **Ele nasceu porque o pintor deitava fora o estado que o despachante já punha aqui.**
    /// Medido pela tinta, e não por leitura: `paint_slider(Hovered)` era **byte-idêntico** a
    /// `paint_slider(Normal)`, e `Dragging` também — só `Focused` e `Disabled` chegavam ao
    /// desenho. Não é a doença do `Checkbox` (que nunca RECEBIA o estado): aqui o `hover.rs` põe
    /// `Hovered`/`Dragging` no store desde sempre, a struct carrega-o, e o `paint` descarta-o —
    /// uma camada mais fundo, e por isso invisível a qualquer gate que olhasse o store.
    ///
    /// ⚠️ **O `value` NÃO sai daqui**, pela razão exacta do [`Self::checkbox_visual`]: quem sabe
    /// onde o slider está é o MODELO que o painel desenha. Um slider que lesse a posição daqui
    /// ficaria um quadro atrás no quadro em que o artista o arrasta — precisamente o gesto.
    #[must_use]
    pub fn slider_visual(&self, id: NodeId) -> (SliderState, f32) {
        let state = self.slider(id).map(|(s, _)| s).unwrap_or_else(|| match () {
            () if self.active_id() == Some(id) => SliderState::Dragging,
            () if self.hot_id() == Some(id) => SliderState::Hovered,
            () => SliderState::Normal,
        });
        (state, self.hover_live(id))
    }

    /// Convenience: read slider value + state.
    pub fn slider(&self, id: NodeId) -> Option<(SliderState, f32)> {
        match self.states.get(&id) {
            Some(InteractiveState::Slider { state, value, .. }) => Some((*state, *value)),
            _ => None,
        }
    }

    /// Convenience: read checkbox value + state.
    pub fn checkbox(&self, id: NodeId) -> Option<(CheckboxState, CheckboxValue)> {
        match self.states.get(&id) {
            Some(InteractiveState::Checkbox { state, value }) => Some((*state, *value)),
            _ => None,
        }
    }

    /// **O par visual de um CHIP DE DROPDOWN** — irmão do [`Self::text_input_visual`].
    ///
    /// ⚠️ **Ele SUBSUME uma derivação que estava escrita à mão em três painéis** (`if open {
    /// Focused } else { Normal }`, em color-equalization · ordering · transport_clips) e
    /// **cravada em `Normal`** nos outros doze: nenhum chip do app acendia sob o ponteiro, e os
    /// três que sabiam algo sabiam só metade.
    #[must_use]
    pub fn dropdown_visual(&self, id: NodeId) -> (crate::widget::DropdownState, f32) {
        use crate::widget::DropdownState as D;
        let (stored, open) = match self.states.get(&id) {
            Some(InteractiveState::Dropdown { state, open, .. }) => (Some(*state), *open),
            _ => (None, false),
        };
        // ⚠️ **`Hovered` NÃO conta como estado guardado, e um gate pagou por isto.** Desde que o
        // `hover.rs` passou a promover o chip no STORE, um dropdown ABERTO sob o rato lia
        // `Hovered` e perdia o anel de `Accent` — as duas metades discordavam sobre qual estado é
        // TRANSIENTE. Semântico aqui é `Focused` e `Disabled`; o resto é o ponteiro a falar.
        let state = match stored {
            Some(s @ (D::Focused | D::Disabled)) => s,
            _ if open => D::Focused,
            _ if stored == Some(D::Hovered) || self.hot_id() == Some(id) => D::Hovered,
            _ => D::Normal,
        };
        (state, self.hover_live(id))
    }

    /// Convenience: read text input contents.
    pub fn text(&self, id: NodeId) -> Option<&str> {
        match self.states.get(&id) {
            Some(InteractiveState::TextInput { text, .. }) => Some(text.as_str()),
            Some(InteractiveState::Combobox { query, .. }) => Some(query.as_str()),
            Some(InteractiveState::NumberInput { buffer, .. }) => Some(buffer.as_str()),
            _ => None,
        }
    }

    /// Convenience: read number-input full state (state + value +
    /// editing buffer + caret + selection anchor). Returns `None`
    /// for non-number widgets.
    #[allow(clippy::type_complexity)]
    pub fn number_input(
        &self,
        id: NodeId,
    ) -> Option<(TextInputState, f64, &str, usize, Option<usize>)> {
        match self.states.get(&id) {
            Some(InteractiveState::NumberInput {
                state,
                value,
                buffer,
                caret,
                selection_anchor,
                ..
            }) => Some((*state, *value, buffer.as_str(), *caret, *selection_anchor)),
            _ => None,
        }
    }

    /// Read just the current numeric value (committed). Useful for
    /// linked sliders that don't care about the in-progress buffer.
    pub fn number_value(&self, id: NodeId) -> Option<f64> {
        match self.states.get(&id) {
            Some(InteractiveState::NumberInput { value, .. }) => Some(*value),
            _ => None,
        }
    }
}
