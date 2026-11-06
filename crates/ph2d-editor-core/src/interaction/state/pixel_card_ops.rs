//! **Os cartões dos Pixel Sprites** no `WidgetStore`: a família `ph2d-app-pixel` publica, o Inspector lê.

use super::super::WidgetStore;
use crate::pixel_panel::PixelSpriteCard;

impl WidgetStore {
    pub fn set_pixel_sprite_cards(&mut self, cards: Vec<PixelSpriteCard>) {
        self.pixel.sprite_cards = cards;
    }

    /// O cartão de `entity_bits`, se ele é um Pixel Sprite à vista no Inspector.
    #[must_use]
    pub fn pixel_sprite_card(&self, entity_bits: u64) -> Option<&PixelSpriteCard> {
        self.pixel
            .sprite_cards
            .iter()
            .find(|c| c.entity_bits == entity_bits)
    }
}
