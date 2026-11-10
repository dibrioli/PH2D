//! A cor por acção (plano §3): `Categorical01..12` de `ph2d-tokens`, por `ActionId` (`id.0 % 12`), nunca pela
//! posição — reordenar ou apagar não troca a cor de quem fica.

use ph2d_input::ActionId;
use ph2d_tokens::ColorToken;

#[must_use]
pub const fn of_action(id: ActionId) -> ColorToken {
    ph2d_tokens::categorical(id.0)
}
