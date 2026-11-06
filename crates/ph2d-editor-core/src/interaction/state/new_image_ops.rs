//! **O diálogo New Image** no `WidgetStore` — filho do `chrome_ops` pelo tecto de LOC: a escolha
//! lembrada na sessão, os campos do Custom e o pedido do OK que a shell drena.

use super::super::WidgetStore;
use crate::interaction::ContextMenuRequest;

impl WidgetStore {
    /// Open the New-image modal (Cmd/Ctrl+N) — centred (its painter ignores `x`/`y`), seeded with the
    /// remembered size/background selection.
    pub fn open_new_image_dialog(&mut self) {
        // Os campos do Custom registam-se aqui, como os do New Sprite: só existem com o diálogo aberto.
        use crate::interaction::InteractiveState as S;
        let [w, h] = self
            .new_image_custom
            .unwrap_or([self.new_image_size, self.new_image_size]);
        let max = f64::from(self.new_image_max_side);
        for (id, v) in crate::ids::NEW_IMAGE_FIELDS.into_iter().zip([w, h]) {
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
            self.set_number_range(id, 1.0, max, 1.0);
            self.set_number_value(id, f64::from(v));
        }
        self.context_menu = Some(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::NewImageDialog,
        });
    }

    /// Selected square size (px) for the New-image modal.
    pub fn new_image_size(&self) -> u32 {
        self.new_image_size
    }

    /// Set the New-image modal's selected square size (px). A preset leaves Custom.
    pub fn set_new_image_size(&mut self, px: u32) {
        self.new_image_size = px;
        self.new_image_custom = None;
    }

    /// O *Custom* do New Image está escolhido?
    #[must_use]
    pub fn new_image_is_custom(&self) -> bool {
        self.new_image_custom.is_some()
    }

    /// Carregar em *Custom*: revela Width e Height, a partir do tamanho que estava escolhido.
    pub fn set_new_image_custom(&mut self) {
        if self.new_image_custom.is_none() {
            let s = self.new_image_size;
            self.new_image_custom = Some([s, s]);
            for id in crate::ids::NEW_IMAGE_FIELDS {
                self.set_number_value(id, f64::from(s));
            }
        }
    }

    /// O lado máximo de uma imagem nova: o da textura do dispositivo. A shell publica-o a cada quadro.
    pub fn set_new_image_max_side(&mut self, max: u32) {
        if self.new_image_max_side != max {
            self.new_image_max_side = max;
            for id in crate::ids::NEW_IMAGE_FIELDS {
                self.set_number_range(id, 1.0, f64::from(max), 1.0);
            }
        }
    }

    /// Selected background for the New-image modal (`0` transparent / `1` black / `2` white).
    pub fn new_image_bg(&self) -> u8 {
        self.new_image_bg
    }

    /// Set the New-image modal's selected background (`0` transparent / `1` black / `2` white).
    pub fn set_new_image_bg(&mut self, bg: u8) {
        self.new_image_bg = bg;
    }

    /// Commit the modal's current `(w, h, bg)` as a pending new-image request (the shell polls + clears
    /// it via [`Self::take_new_image_request`]) and close the modal.
    pub fn request_new_image(&mut self) {
        let (mut w, mut h) = (self.new_image_size, self.new_image_size);
        if self.new_image_custom.is_some() {
            let max = self.new_image_max_side;
            let lado = |s: &Self, id, antes| {
                s.number_value(id)
                    .map_or(antes, |v| crate::pixel_panel::side_of(v).min(max))
            };
            w = lado(self, crate::ids::NEW_IMAGE_WIDTH, w);
            h = lado(self, crate::ids::NEW_IMAGE_HEIGHT, h);
            self.new_image_custom = Some([w, h]);
        }
        self.new_image_request = Some((w, h, self.new_image_bg));
        self.close_context_menu();
    }

    /// Take (and clear) the pending new-image request. The shell polls this each frame; on `Some` it
    /// spawns a blank canvas of that `(w, h, bg)`.
    pub fn take_new_image_request(&mut self) -> Option<(u32, u32, u8)> {
        self.new_image_request.take()
    }
}
