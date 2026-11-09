//! O import do `.ase` com a normal ligada: [`ph2d_sprite_ase_import`] nasce a sprite e devolve a folha
//! da normal; aqui ela entra na IMAGEM pela porta [`crate::normal_map`] (P14e), sem passo manual.

pub use ph2d_sprite_ase_import::{ASE_EXTENSIONS, AseImportResult, is_ase_file};

/// [`ph2d_sprite_ase_import::import_ase`] + a normal da imagem (um aviso se a porta a recusar).
#[allow(clippy::too_many_arguments)]
pub fn import_ase(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    path: &std::path::Path,
    anchor_world: [f32; 2],
    pixels_per_meter: f32,
) -> AseImportResult {
    let mut r = ph2d_sprite_ase_import::import_ase(
        sim,
        renderer,
        asset_db,
        next_cell,
        atlas_asset_map,
        path,
        anchor_world,
        pixels_per_meter,
    );
    if let AseImportResult::Ok {
        bits,
        notes,
        normal,
        ..
    } = &mut r
        && let Some(n) = normal.take()
        && let Err(e) = liga_normal(sim.world(), asset_db, atlas_asset_map, *bits, n)
    {
        notes.push(e);
    }
    r
}

/// A folha da normal `(w, h, rgba)` vira a normal da imagem do sprite `bits`.
///
/// # Errors
/// O texto da porta `normal_map` (outro tamanho, sprite sem imagem).
pub fn liga_normal(
    world: &ph2d_ecs::World,
    db: &ph2d_asset::AssetDb,
    atlas: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    bits: u64,
    (w, h, px): (u32, u32, Vec<u8>),
) -> Result<(), String> {
    let id = db.insert_image_rgba8(w, h, px);
    let e = ph2d_ecs::Entity::from_bits(bits);
    crate::normal_map::escolher_normal(world, db, atlas, e, Some(id))
}
