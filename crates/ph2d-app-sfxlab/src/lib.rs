//! **A família do SFX Lab** (W3, `docs/SfxLab/04_W3_plano.md`): o som numa aba, com camadas sobre uma régua.
//!
//! A receita de trabalho mora na aba e é escrita na `SfxLibrary` do mundo por [`service`]. O render corre numa thread
//! por aba ([`RenderJob`]); o limitador só existe na escuta ([`Escuta`]). Os cartões são do painel `ph2d-panel-sfxlab`, na vaga do Inspector ([`ponte`]).

mod aba;
mod docs;
mod escuta;
mod gestos;
mod historia;
mod metrics;
mod novo;
mod paint;
mod ponte;
mod smoke;
mod surface;

pub use aba::Alvo;
pub use docs::SfxLabDocs;
pub use escuta::Escuta;
pub use historia::MAX_PASSOS;
pub use novo::{Fonte, semente_do_nome};
pub use ph2d_sfx_lab::job::{COLUNAS_DA_ONDA, Feito, Onda, Pedido, RenderJob};
pub use smoke::SOM_DA_CENA;
pub use surface::{CMD_NEW, cmd_abrir};

use bevy_ecs::world::World;
use ph2d_editor_core::screens::hero::HeroScreen;

/// O arranque da família.
pub fn install(hero: &mut HeroScreen) {
    hero.documents
        .install_surface(Box::new(SfxLabDocs::default()));
}

/// O serviço (a shell chama-o uma vez por imagem): o mundo, as edições, o render que chegou e o som.
pub fn service(hero: &mut HeroScreen, world: &mut World, escuta: Option<&mut dyn Escuta>) {
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<SfxLabDocs>() else {
        return;
    };
    if ph2d_sfx_scene::nivel() == Some(ph2d_sfx_scene::NIVEL_ABA) && docs.cena_por_abrir() {
        docs.abre_a_cena(world);
    }
    let ativa = ponte::aba_ativa(hero);
    ponte::vaga(hero, ativa.is_some());
    ponte::drena(hero, ativa);
    if let Some(docs) = hero.documents.surfaces_mut().find_mut::<SfxLabDocs>() {
        docs.servir(world, escuta);
        ponte::publica_o_instantaneo(docs, ativa);
    }
}

/// A declaração da família ao registo. O `PH2D_SFX_SMOKE` é declarado pelo jogo, que monta o mundo da cena.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "sfxlab",
    routers: &[],
};

pub(crate) const A11Y_SALT: u64 = 0x5346_584C_0000_0000;
