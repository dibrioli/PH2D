//! `PH2D_SFX_SMOKE=3`: a aba aberta em «Metal Hit»; `4`: com a janela do Use in Game. O roteador é um só, da família do jogo (`ph2d_sfx_scene::ROUTER`):
//! é ela que monta o mundo, e aqui só se pergunta o nível.

use bevy_ecs::world::World;
use ph2d_sfx_recipe::{SfxLibrary, factory};

use crate::docs::SfxLabDocs;

/// O som que a cena abre.
pub const SOM_DA_CENA: &str = "Metal Hit";

impl SfxLabDocs {
    /// Põe o som da cena no mundo, pela fábrica, e pede a aba. Chamado uma vez, na imagem em que a cena está armada.
    pub fn abre_a_cena(&mut self, world: &mut World) {
        if !world.contains_resource::<SfxLibrary>() {
            world.insert_resource(SfxLibrary::default());
        }
        if let Some(mut lib) = world.get_resource_mut::<SfxLibrary>()
            && !lib.recipes.contains_key(SOM_DA_CENA)
            && let Some((_, r)) = factory::all().into_iter().find(|(n, _)| *n == SOM_DA_CENA)
        {
            lib.recipes.insert(SOM_DA_CENA.to_owned(), r);
        }
        self.pede_abrir(SOM_DA_CENA.to_owned());
    }

    /// `PH2D_SFX_SMOKE=4`: a mesma aba, com a janela do Use in Game aberta uma vez.
    pub(crate) fn abre_a_janela_da_cena(&mut self) {
        if !self.janela_da_cena
            && let Some(a) = self.abas.first_mut()
        {
            a.quer_escolher = true;
            self.janela_da_cena = true;
        }
    }
}
