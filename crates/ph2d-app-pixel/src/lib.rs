//! **A família Pixel Art** (ADR-0196, `docs/PixelArt/02_plano_modulo.md`).
//!
//! O objecto **Pixel Sprite** nasce no menu Add (o marcador [`ph2d_ecs::PixelSprite`] guarda a chave
//! do documento) e edita-se no modo Draw, no viewport, com as ferramentas e os atalhos do Aseprite:
//! `B` lápis, `E` borracha, `I` conta-gotas (`Alt` segurado = conta-gotas), `X` troca frente/fundo,
//! `Ctrl+Z`/`Ctrl+Shift+Z` no desfazer PRÓPRIO do sprite, a roda em zoom inteiro.
//!
//! O documento ([`ph2d_pixel_core::Sprite`]) vive em [`PixelApp`], fora do mundo e do undo do mundo,
//! e grava-se como blob com versão própria no projecto ([`persist`]). A shell só compõe: chama
//! [`frame`](PixelApp::frame) uma vez por quadro, encaminha o ponteiro/roda/teclado ([`input`]) e
//! pede o desenho ([`draw`]).

pub mod camadas;
mod cartao;
pub mod comando;
pub mod compose;
pub mod conta_gotas;
pub mod degrade;
pub mod draw;
pub mod edicao;
pub mod exportar;
mod filtros;
pub use ph2d_pixel_formas as formas;
mod frame;
mod gesture;
pub mod input;
mod input_teclas;
mod intervalo;
pub mod luz;
pub mod mao;
pub mod mode;
pub mod object_add;
pub mod paleta;
pub mod panel;
pub mod persist;
pub mod pinceis;
mod pincel;
pub mod quadros;
pub mod selecao;
pub mod slices;
pub mod smoke;
pub mod smoke_ajustes;
pub mod smoke_camadas;
pub mod smoke_catalogo;
pub mod smoke_efeitos;
pub mod smoke_exporta;
pub mod smoke_ferramentas;
pub mod smoke_formas;
pub mod smoke_intervalo;
pub mod smoke_pincel;
pub mod smoke_pontas;
pub mod smoke_quadros;
pub mod smoke_selecao;
pub mod smoke_slices;
pub mod smoke_tiles;
pub mod smoke_tocar;
pub mod smoke_viva;
pub mod solta;
mod som;
mod sprite_props;
mod state;
pub mod texto;
mod tiles;
pub mod viva;

pub use ph2d_pixel_cena::registar;
pub use state::{Doc, Options, PixelApp, Tool};

/// Os derivados do Pixel Sprite na cena (P14 §4 A e C): os clips e a forma que segue a slice; uma
/// vez por quadro, ANTES do tique da animação e da física. A régua (píxeis por metro) é a do
/// projecto do `hero`; sem ele, a de omissão.
pub fn deriva(
    app: &PixelApp,
    world: &mut ph2d_ecs::World,
    hero: Option<&ph2d_editor_core::screens::hero::HeroScreen>,
) {
    ph2d_pixel_cena::deriva(world, |k| app.docs.get(&k).map(|d| (&d.sprite, d.stamp)));
    let ppm = hero.map_or(ph2d_editor_core::project::DEFAULT_PIXELS_PER_METER, |h| {
        h.project.pixels_per_meter
    });
    let m_por_px = 1.0 / f64::from(ppm.max(f32::MIN_POSITIVE));
    app.m_por_px.set(m_por_px);
    ph2d_pixel_cena::deriva_formas(
        world,
        |k| app.docs.get(&k).map(|d| (&d.sprite, d.frame)),
        m_por_px,
    );
    let marcas = app
        .docs
        .keys()
        .map(|&k| (k, ph2d_pixel_cena::slices_com_collider(world, k)))
        .filter(|(_, s)| !s.is_empty())
        .collect();
    *app.colliders_no_mundo.borrow_mut() = marcas;
}

impl PixelApp {
    /// A cópia do documento `k` que se grava: `ph2d.collider` nas slices com collider no mundo.
    pub(crate) fn a_gravar(&self, k: u32, s: &ph2d_pixel_core::Sprite) -> ph2d_pixel_core::Sprite {
        match self.colliders_no_mundo.borrow().get(&k) {
            Some(m) => ph2d_pixel_cena::com_colliders(s, m),
            None => s.clone(),
        }
    }
}

/// A família declarada à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "pixel",
    routers: smoke::ROUTERS,
};
