//! **O diálogo Color Mode** do Pixel Art no `WidgetStore` (Sprite ▸ Color Mode ▸ More Options do
//! Aseprite) — irmão do `new_sprite_ops`: a escolha lembrada na sessão, o modo do sprite ao
//! abrir e o pedido do OK que a família drena.

use super::super::WidgetStore;
use crate::interaction::ContextMenuRequest;
use crate::pixel_panel::{ColorModeChoice, NewSpriteMode, PixelDither, PixelToGray};

impl WidgetStore {
    /// Abre o diálogo **Color Mode** sobre um sprite no modo `current`, com o próprio modo
    /// ESCOLHIDO (smoke do dono 25/10: o modo actual apagado confundia). O dither e o método de
    /// cinza ficam os da última vez.
    pub fn open_color_mode_dialog(&mut self, current: NewSpriteMode) {
        use crate::interaction::InteractiveState as S;
        let d = &mut self.pixel.color_mode;
        d.current = current;
        d.choice.target = current;
        let targets = crate::ids::COLOR_MODE_TARGETS.iter().map(|(_, id, _)| *id);
        let dithers = crate::ids::COLOR_MODE_DITHERS.iter().map(|(_, id, _)| *id);
        let grays = crate::ids::COLOR_MODE_GRAYS.iter().map(|(_, id, _)| *id);
        let ends = [crate::ids::COLOR_MODE_OK, crate::ids::COLOR_MODE_CANCEL];
        for id in targets.chain(dithers).chain(grays).chain(ends) {
            let state = crate::widget::ButtonState::Normal;
            self.register(id, S::Button { state });
        }
        self.context_menu = Some(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::PixelColorModeDialog,
        });
    }

    #[must_use]
    pub fn color_mode_choice(&self) -> ColorModeChoice {
        self.pixel.color_mode.choice
    }

    /// O modo do sprite quando o diálogo abriu.
    #[must_use]
    pub fn color_mode_current(&self) -> NewSpriteMode {
        self.pixel.color_mode.current
    }

    /// Escolhe o alvo (o modo actual também: OK sobre ele não muda nada).
    pub fn set_color_mode_target(&mut self, m: NewSpriteMode) {
        self.pixel.color_mode.choice.target = m;
    }

    pub fn set_color_mode_dither(&mut self, d: PixelDither) {
        self.pixel.color_mode.choice.dither = d;
    }

    pub fn set_color_mode_gray(&mut self, g: PixelToGray) {
        self.pixel.color_mode.choice.gray = g;
    }

    /// O OK: a escolha vira o pedido — nenhum quando o alvo é o modo actual; fecha.
    pub fn request_color_mode(&mut self) {
        let d = &mut self.pixel.color_mode;
        d.request = (d.choice.target != d.current).then_some(d.choice);
        self.close_context_menu();
    }

    /// Toma (e limpa) o pedido do OK.
    pub fn take_color_mode_request(&mut self) -> Option<ColorModeChoice> {
        self.pixel.color_mode.request.take()
    }
}
