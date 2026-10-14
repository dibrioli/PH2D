//! `PH2D_MACRO_SMOKE=1` — **o Splash** (`docs/Macros/01_plano_onda1.md` §6): uma tela branca 1024² com
//! um `Macro` que pinta nela quatro pinceladas de Wet Paint
//! ([`ph2d_tool_painter::macro_track::roteiro::grava_o_splash`]). A shell monta o clipe
//! (`add_macro_clip`, laço, Autoplay) e põe a cena a correr: as pinceladas pousam e a água escorre entre
//! elas, e cada volta do laço recua a macro à tela branca.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Macro, ModoDaMacro, SimWorld};
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// O lado da tela, em pixels (o plano §6).
pub const LADO: u32 = 1024;

/// **A cena**: a tela e o `Macro` gravado nela. Devolve os bits da sprite e a duração da macro, para a
/// shell montar o clipe.
pub fn monta(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, f64)> {
    let (_, bits) = ph2d_image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        LADO,
        2, // branca: o chão da aquarela, como na `PH2D_WETPAINT_SMOKE`
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    )
    .map_err(|e| eprintln!("[macro smoke =1] could not spawn the canvas: {e}"))
    .ok()?;
    let (doc, duracao) = ph2d_tool_painter::macro_track::roteiro::grava_o_splash(LADO, 0)?;
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(Macro {
            doc,
            modo: ModoDaMacro::Fiel,
        });
    eprintln!(
        "[macro smoke =1] 'Canvas' (white, {LADO}²) carries a recorded macro: four Wet Paint strokes \
         ({duracao:.1} s). The scene runs (top-bar Play): the strokes land and the water flows between \
         them; every loop goes back to the white canvas."
    );
    Some((bits, duracao))
}
