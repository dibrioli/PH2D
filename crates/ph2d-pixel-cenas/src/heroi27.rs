//! A cena 27 «O mesmo herói pelo Sprite Inspector» (`docs/PixelArt/P14_plano.md` §8): a cena 26 no
//! arranjo [`crate::heroi::CENA_27`], uma segunda parede, e o `.ase` que o Pixel grava do herói (a
//! marca do Add Collider tirada do MUNDO, como o Save) à espera do File ▸ Import respondido
//! «Sprite» — a shell importa-o por `ph2d_sprite_ase_import::import_ase`, e [`veste`] dá ao Sprite
//! do motor o corpo, o Blocks Light e a regra do herói do Pixel.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use ph2d_ecs::{Entity, Name, World};
use ph2d_pixel_core::Sprite;

use crate::heroi as h;

static PENDENTE: Mutex<Option<PathBuf>> = Mutex::new(None);
static VESTIDO: AtomicBool = AtomicBool::new(false);

/// Os bytes do `.ase` que o Pixel grava do documento `key` (`s`): as slices com collider NO MUNDO
/// marcadas (`ph2d.collider`).
#[must_use]
pub fn ase_do_documento(world: &mut World, s: &Sprite, key: u32) -> Option<Vec<u8>> {
    let marcadas = ph2d_pixel_cena::slices_com_collider(world, key);
    ph2d_pixel_io::ase::escrever(&ph2d_pixel_cena::com_colliders(s, &marcadas))
        .ok()
        .map(|(b, _)| b)
}

/// O resto da cena sobre a 26 já montada: a «Wall 2» (documento `parede`) e o `.ase` do herói (`s`,
/// documento `key`) gravado numa pasta temporária, à espera do import.
pub fn monta(world: &mut World, parede: u32, s: &Sprite, key: u32) -> Option<()> {
    let y = h::CENA_27.parede.2[1];
    h::parede(world, "Wall 2", parede, [h::PAREDE_27_X, y]);
    ph2d_ecs::assign_missing_stable_ids(world);
    ph2d_ecs::assign_missing_root_order(world);
    let bytes = ase_do_documento(world, s, key)?;
    let pasta = std::env::temp_dir().join("ph2d-pixel-cena27");
    std::fs::create_dir_all(&pasta).ok()?;
    let caminho = pasta.join(format!("{}.ase", h::NOME_DO_SPRITE));
    std::fs::write(&caminho, bytes).ok()?;
    *PENDENTE.lock().ok()? = Some(caminho);
    Some(())
}

/// O `.ase` à espera do import (uma vez).
#[must_use]
pub fn pendente() -> Option<PathBuf> {
    PENDENTE.lock().ok()?.take()
}

/// O Sprite do motor que o import nasceu (o nome do ficheiro).
#[must_use]
pub fn sprite_do_motor(world: &mut World) -> Option<Entity> {
    let mut q = world.query::<(Entity, &Name, &ph2d_render::Sprite)>();
    q.iter(world)
        .find(|(_, n, _)| n.0 == h::NOME_DO_SPRITE)
        .map(|(e, _, _)| e)
}

/// Com a cena 27 armada (`nivel`), uma vez, quando o Sprite do motor existe: vesti-lo
/// ([`h::veste_o_do_motor`]) à direita da lâmpada.
pub fn veste(world: &mut World, nivel: Option<u32>) {
    if nivel != Some(27) || VESTIDO.load(Ordering::Relaxed) {
        return;
    }
    if let Some(e) = sprite_do_motor(world) {
        h::veste_o_do_motor(world, e, [h::HEROI_27_X, h::HEROI[1]]);
        VESTIDO.store(true, Ordering::Relaxed);
    }
}
