//! Os pedidos da aba ao mapa (plano §3): a família aplica-os em `hero.input_map` no `frame`.

use ph2d_input::recipes::Recipe;
use ph2d_input::{ActionId, Binding, Key};

#[derive(Clone, Debug, PartialEq)]
pub enum InputMapEdit {
    /// Uma acção nova com este nome.
    Add(String),
    Remove(ActionId),
    AddBinding(ActionId, Binding),
    /// Tira a ligação `n` da acção.
    RemoveBinding(ActionId, usize),
    /// `(dead_zone, press_point)`.
    SetZone(ActionId, f32, f32),
    StartFrom(Recipe),
    SetPlayersCanChange(bool),
    SetShown(ActionId, bool),
    SetOpenKey(Key),
    ForgetPlayerKeys,
}
