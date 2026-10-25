//! **O diálogo New Sprite** no `WidgetStore` (ADR-0192) — filho do [`super`] pelo tecto de LOC: a
//! escolha lembrada na sessão, os widgets do diálogo e o pedido do OK que a shell drena.

use super::super::WidgetStore;
use crate::interaction::ContextMenuRequest;

impl WidgetStore {
    /// Abre o diálogo **New Sprite** (Add ▸ Pixel Sprite), semeado com a última escolha da sessão.
    /// Os widgets registam-se aqui: o diálogo só existe enquanto aberto.
    pub fn open_new_sprite_dialog(&mut self) {
        use crate::interaction::InteractiveState as S;
        let c = self.pixel.new_sprite.choice;
        for (id, v) in crate::ids::NEW_SPRITE_FIELDS
            .into_iter()
            .zip([c.width, c.height])
        {
            self.register(
                id,
                S::NumberInput {
                    state: crate::widget::TextInputState::Normal,
                    value: f64::from(v),
                    buffer: String::new(),
                    caret: 0,
                    last_committed: f64::from(v),
                    selection_anchor: None,
                },
            );
            let max = f64::from(crate::pixel_panel::NEW_SPRITE_MAX_SIDE);
            self.set_number_range(id, 1.0, max, 1.0);
            self.set_number_drag_rate(id, 0.25); // LITERAL-PX-OK: células por px de arrasto, domínio do documento
            self.set_number_value(id, f64::from(v));
        }
        let modes = crate::ids::NEW_SPRITE_MODES.iter().map(|(_, id, _)| *id);
        let bgs = crate::ids::NEW_SPRITE_BGS.iter().map(|(_, id, _)| *id);
        let ends = [crate::ids::NEW_SPRITE_OK, crate::ids::NEW_SPRITE_CANCEL];
        for id in modes.chain(bgs).chain(ends) {
            let state = crate::widget::ButtonState::Normal;
            self.register(id, S::Button { state });
        }
        self.context_menu = Some(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::NewSpriteDialog,
        });
    }

    /// A escolha do diálogo New Sprite (os campos numéricos vivem nos widgets até ao OK).
    #[must_use]
    pub fn new_sprite_choice(&self) -> crate::pixel_panel::NewSpriteChoice {
        self.pixel.new_sprite.choice
    }

    pub fn set_new_sprite_mode(&mut self, m: crate::pixel_panel::NewSpriteMode) {
        self.pixel.new_sprite.choice.mode = m;
    }

    pub fn set_new_sprite_background(&mut self, b: crate::pixel_panel::NewSpriteBackground) {
        self.pixel.new_sprite.choice.background = b;
    }

    /// O OK: os lados escritos (presos a `1..=65535`) entram na escolha, que vira o pedido; fecha.
    pub fn request_new_sprite(&mut self) {
        let side = |id| self.number_value(id).map(crate::pixel_panel::side_of);
        let (w, h) = (
            side(crate::ids::NEW_SPRITE_WIDTH),
            side(crate::ids::NEW_SPRITE_HEIGHT),
        );
        let choice = &mut self.pixel.new_sprite.choice;
        choice.width = w.unwrap_or(choice.width);
        choice.height = h.unwrap_or(choice.height);
        self.pixel.new_sprite.request = Some(self.pixel.new_sprite.choice);
        self.close_context_menu();
    }

    /// Toma (e limpa) o pedido do OK. Fechar o diálogo por outro caminho não arma nada.
    pub fn take_new_sprite_request(&mut self) -> Option<crate::pixel_panel::NewSpriteChoice> {
        self.pixel.new_sprite.request.take()
    }
}
