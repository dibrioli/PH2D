//! **A animação de sprite** (GameEngineFuncional, [plano 07](../../../docs/GameEngineFuncional/07_animacao_num_cartao_plano.md)):
//! o tique que toca as animações ([`tick`]) e o snapshot e as edições do cartão «Animation» ([`inspector`]).

pub mod ase_import;
pub mod card;
pub mod editor;
pub mod image_choice;
pub mod image_use;
pub mod inspector;
pub mod normal_map;
pub use ph2d_anim_sheets::panel_frame;
pub mod previa;
pub mod region;
pub use ph2d_anim_sheets::sheet_cache as sheet;
pub mod sprite_card;
pub mod tick;
pub mod tiled;
pub use ph2d_anim_sheets::use_frame;

/// A declaração da família à shell. Sem roteador: as cenas dela (usos da imagem, Region, a animação
/// do Player) monta-as a `game` (`PH2D_GAME_SMOKE`) — quem possui a cena é quem a constrói.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "sprite-anim",
    routers: &[],
};
