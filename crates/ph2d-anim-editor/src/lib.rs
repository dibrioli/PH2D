//! **O documento do editor de animações** — uma aba por sprite (plano GameEngineFuncional 07 e 26): o estado
//! ([`doc::AnimDoc`]), a disposição ([`layout`]), os gestos e a pintura. Não toca no mundo: cada gesto vira uma edição
//! que a família (`ph2d_app_sprite_anim::editor`) aplica pela porta do Inspector.

pub mod align;
pub mod align_pivot;
pub mod bands;
pub mod doc;
pub mod layout;
mod marks;
pub mod paint;
mod paint_align;
mod paint_pivot;
pub mod paint_sheet;
mod silhueta;
pub mod strip_bar;
pub use ph2d_anim_sheets::split;

/// Maior lado de uma grelha que o cartão aceita (o `SpriteGrid` guarda `u32`; isto é o que cabe a clicar numa
/// folha: 256 × 256 células).
pub const GRID_SIDE_MAX: u32 = 256;
