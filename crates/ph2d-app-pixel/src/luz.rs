//! ⭐ **O Pixel Sprite na luz 2D** (P14c-pixel 2) — a família entrega cada objecto a
//! [`ph2d_pixel_luz`] pela porta imagem → mundo dela ([`crate::input::to_view`], o pivô do quadro
//! mostrado): a banda normal de cada faixa Pixel e a camada de oclusão.

use ph2d_ecs::Entity;
use ph2d_pixel_luz::{ObjetoPixel, OclusaoDoPixel};
use ph2d_vector::Affine;

use crate::PixelApp;
use crate::input::{key_of, to_view};

/// Os Pixel Sprites visíveis de `ids` como a luz os vê (`sem_luz` = os `NaoRecebeLuz`).
#[must_use]
pub fn objetos<'a>(
    app: &'a PixelApp,
    world: &ph2d_ecs::World,
    ppm: f32,
    ids: &[u64],
    sem_luz: &[Entity],
) -> Vec<ObjetoPixel<'a>> {
    ids.iter()
        .filter_map(|&bits| {
            let e = Entity::try_from_bits(bits)?;
            if ph2d_ecs::visibility::is_hidden_in_tree(world, e) {
                return None;
            }
            let doc = app.docs.get(&key_of(world, bits)?)?;
            Some(ObjetoPixel {
                bits,
                sprite: &doc.sprite,
                quadro: ph2d_pixel_cena::quadro_mostrado(world, e, &doc.sprite, doc.frame),
                stamp: doc.stamp,
                img_mundo: to_view(world, bits, doc, Affine::IDENTITY, ppm)?,
                lin: ph2d_pixel_luz::base_da_instancia(world, e)?,
                sem_luz: sem_luz.contains(&e),
            })
        })
        .collect()
}

/// ⭐ A banda normal da próxima faixa, ao lado da cena dela (UMA chamada por faixa, na ordem das
/// cenas): `ids` = os Pixel Sprites de uma faixa Pixel (`None` noutra família); `sem_luz` = `None`
/// com a luz apagada ([`ph2d_app_light2d::LuzDoQuadro::normais_do_quadro`]).
pub fn faixa_normal(
    app: &PixelApp,
    world: &ph2d_ecs::World,
    (cam, ppm): (Affine, f32),
    ids: Option<&[u64]>,
    (normais, sem_luz): (&mut ph2d_app_light2d::NormaisDasFaixas, Option<&[Entity]>),
) {
    let banda = ids.zip(sem_luz).and_then(|(ids, sem)| {
        ph2d_pixel_luz::banda_normal(&objetos(app, world, ppm, ids, sem), cam)
    });
    normais.cenas.push(banda);
}

/// A camada de oclusão de TODOS os Pixel Sprites (a luz pede os que bloqueiam, por `quem`).
#[must_use]
pub fn oclusao<'a>(app: &'a PixelApp, world: &ph2d_ecs::World, ppm: f32) -> OclusaoDoPixel<'a> {
    let ids: Vec<u64> = world
        .try_query::<(Entity, &ph2d_ecs::PixelSprite)>()
        .map(|mut q| q.iter(world).map(|(e, _)| e.to_bits()).collect())
        .unwrap_or_default();
    OclusaoDoPixel {
        objetos: objetos(app, world, ppm, &ids, &[]),
    }
}
