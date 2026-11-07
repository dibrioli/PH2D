//! `PH2D_SFX_SMOKE=3`: a aba aberta em «Metal Hit»; `4`: com a janela do Use in Game. O roteador é um só, da família do jogo (`ph2d_sfx_scene::ROUTER`):
//! é ela que monta o mundo, e aqui só se pergunta o nível.

use bevy_ecs::world::World;
use ph2d_sfx_recipe::{SfxLibrary, factory};

use crate::docs::SfxLabDocs;

/// O som que a cena abre.
pub const SOM_DA_CENA: &str = "Metal Hit";
/// O som das cenas da exploração: quatro camadas, com a primeira travada, para o sorteio mostrar o Lock.
pub const SOM_DA_EXPLORACAO: &str = "Impact";

impl SfxLabDocs {
    /// Põe o som da cena no mundo, pela fábrica, e pede a aba. Chamado uma vez, na imagem em que a cena está armada.
    pub fn abre_a_cena(&mut self, world: &mut World) {
        if !world.contains_resource::<SfxLibrary>() {
            world.insert_resource(SfxLibrary::default());
        }
        let explora = ph2d_sfx_scene::nivel() >= Some(ph2d_sfx_scene::NIVEL_EXPLORA);
        let nome = if explora {
            SOM_DA_EXPLORACAO
        } else {
            SOM_DA_CENA
        };
        if let Some(mut lib) = world.get_resource_mut::<SfxLibrary>()
            && !lib.recipes.contains_key(nome)
            && let Some((_, mut r)) = factory::all().into_iter().find(|(n, _)| *n == nome)
        {
            if explora && let Some(l) = r.layers.first_mut() {
                l.travada = true;
            }
            lib.recipes.insert(nome.to_owned(), r);
        }
        self.pede_abrir(nome.to_owned());
    }

    /// `PH2D_SFX_SMOKE=6`: a aba da exploração com o Explore aberto, uma vez.
    pub(crate) fn abre_a_grade_da_cena(&mut self) {
        if !self.janela_da_cena
            && let Some(a) = self.abas.first_mut()
        {
            let (r, t) = (a.receita.clone(), a.takes.clone());
            a.ex.alterna(&r, &t);
            self.janela_da_cena = true;
        }
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
