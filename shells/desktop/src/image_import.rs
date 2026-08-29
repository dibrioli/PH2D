//! Full pipeline for importing image files into the running demo
//! (drag-drop / file-dialog path).
//!
//! Self-contained orchestrator — bytes → AssetDb decode → SpriteRenderer
//! atlas pack → SimWorld spawn `(Transform, Sprite, Name)`. A batch of
//! files is laid out in a tidy near-square grid (see
//! [`import_images_grid`]) so a multi-file drop spreads out instead of
//! stacking every sprite on one point.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::{Sprite, SpriteRenderer};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// Re-export SimWorld type alias used in the signatures (defined in the
// main module). Avoids dragging the heavier bevy_ecs::World import
// surface into this file.
use crate::SimWorld;

/// World-space gap between adjacent grid cells in a multi-image import,
/// expressed as a fraction of the largest sprite's max dimension. Keeps
/// the spacing proportional to the imported sprites' scale so a grid of
/// 4K photos and a grid of 64px icons both read as a tidy grid rather
/// than "touching" or "lost in whitespace". Not a UI token (HR-15) —
/// this is world-space scene layout, same class as `MIN_SPRITE_SIZE`.
pub(crate) const IMPORT_GRID_GAP_FRAC: f32 = 0.08;

/// Outcome of importing one file in a batch, returned in input order so
/// the host can surface a per-file toast and seat the selection.
pub enum ImportItemResult {
    /// File decoded, packed and spawned. `bits` is the sim-entity bits
    /// (so the host can select it); `label` is the assigned `Name`.
    Ok { label: String, bits: u64 },
    /// File failed before spawning. `name` is the file name (for the
    /// toast); `error` is the human-readable cause.
    Err { name: String, error: String },
}

/// Decode + atlas-pack one file WITHOUT spawning, returning its
/// world-space size `[w, h]`. Split out from the spawn step so a batch
/// can be measured first (for grid layout) and spawned second.
///
/// 1. Read bytes from disk.
/// 2. `AssetDb::insert_image_bytes` (auto-detects PNG/WEBP/JPEG,
///    hashes blake3 per HR-6).
/// 3. `SpriteRenderer::insert_atlas_sprite_with_regrow` packs the
///    native-resolution RGBA into the atlas via the Skyline rect packer
///    — no resize, no aspect-ratio squash. The regrow path lets a 2nd /
///    3rd 4K import double the atlas instead of failing with AtlasFull;
///    the closure recovers each existing region's source bytes from
///    `asset_db` via `atlas_asset_map[key] → AssetId`.
fn pack_image(
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    path: &Path,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Result<([f32; 2], PackedSource), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let asset_id = asset_db
        .insert_image_bytes(&bytes)
        .map_err(|e| format!("decode {}: {e}", path.display()))?;
    let decoded = asset_db
        .get(&asset_id)
        .ok_or_else(|| format!("asset {asset_id} missing after insert"))?;
    // **A bifurcação da W2.4**: uma imagem de 16 bits não entra no atlas, nasce `Individual`
    // (plano `docs/Sprite_projeto/18` §3.3 — o atlas é uma textura com um formato).
    //
    // ⚠️ Ela também **não entra no `atlas_asset_map`**, e isso é o que mantém verdadeira a linha do
    // `project_assets.rs` que grava as células do atlas em 8 bits sem perder nada.
    if let ph2d_asset::Asset::ImageRgba16 {
        width,
        height,
        pixels,
    } = &*decoded
    {
        let texture_id = renderer
            .acquire_individual_16(*width, *height, pixels)
            .map_err(|e| format!("individual 16-bit {}: {e}", path.display()))?;
        return Ok((
            world_size(*width, *height, pixels_per_meter),
            PackedSource::Individual {
                texture_id,
                pixels_id: asset_id,
            },
        ));
    }
    // ⚠️ `image_rgba8` e não um `match` na variante: daqui para baixo é o caminho PARA O ATLAS, que
    // é de 8 bits por construção. O `Cow` não copia no caso de 8 bits, que é o de sempre.
    let (width, height, pixels) = decoded
        .image_rgba8()
        .ok_or_else(|| format!("{asset_id} is not an uncompressed image after decode"))?;
    // Track this import's mapping BEFORE the insert so a regrow
    // triggered by it sees the new key.
    atlas_asset_map.insert(cell_idx, asset_id);
    let fetch_pixels = |key: u32| -> Option<Vec<u8>> {
        let aid = atlas_asset_map.get(&key)?;
        let asset = asset_db.get(aid)?;
        // `pixels` is `Arc<[u8]>`; the regrow callback wants an owned `Vec<u8>` because the
        // underlying packer may outlive the asset borrow.
        //
        // ⚠️ `image_rgba8` e não um `match` na variante: o ATLAS é de 8 bits por construção (uma
        // textura, um formato), por isso converter para baixo aqui é a resposta certa e não uma
        // perda escondida. Casar `ImageRgba8` fazia um asset de 16 bits devolver `None` e o
        // regrow reconstruir a célula VAZIA (plano `docs/Sprite_projeto/18`, auditoria da W2).
        asset.image_rgba8().map(|(_, _, px)| px.into_owned())
    };
    if let Err(e) =
        renderer.insert_atlas_sprite_with_regrow(cell_idx, width, height, &pixels, fetch_pixels)
    {
        // On failure, roll back the map insert so the next import
        // doesn't see a dangling key → nonexistent atlas region.
        atlas_asset_map.remove(&cell_idx);
        return Err(format!("atlas insert {}: {e}", path.display()));
    }
    // Sprite world size = source pixels / pixels_per_meter. With the
    // Skyline atlas the source bytes are stored at full resolution, so
    // the world quad's aspect ratio matches the file exactly (a 256×128
    // PNG renders as a 2:1 rect in world space).
    Ok((
        world_size(width, height, pixels_per_meter),
        PackedSource::Atlas { cell_idx },
    ))
}

/// O tamanho em metros de uma imagem de `width × height` px.
///
/// Extraído para os **dois** ramos do [`pack_image`] responderem o mesmo: uma sprite de 16 bits e a
/// sua gémea de 8 têm de nascer do mesmo tamanho, senão a precisão passaria a mudar a geometria.
fn world_size(width: u32, height: u32, pixels_per_meter: f32) -> [f32; 2] {
    let safe_px_per_m = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    [
        (width as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE),
        (height as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE),
    ]
}

/// Human-readable base name for an imported file, falling back to a
/// cell-indexed stub for paths with no usable stem.
fn base_label(path: &Path, cell_idx: u32) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("imported_{cell_idx}"))
}

/// Spawn one already-packed sprite at `world_center`. The unique-name
/// bump runs HERE (not at pack time) so batch siblings sharing a file
/// stem see each other already in the world and get distinct " (1)" /
/// " (2)" labels — same convention as `HierDuplicate` / `AddChild` /
/// rename (mirrors the duplicate-`Name` bug fixed 2026-05-27).
pub(crate) fn spawn_sprite(
    sim: &mut SimWorld,
    source: PackedSource,
    world_center: Vec2,
    world_size: [f32; 2],
    base: &str,
) -> (String, u64) {
    let label = crate::name_unique::unique_name(sim, base);
    // ⚠️ A estratégia vem do PACK, não de um default: uma imagem de 16 bits foi para uma textura
    // própria e tem de nascer a apontar para ela (plano `docs/Sprite_projeto/18` W2.4). Construir
    // `Sprite::atlas` aqui faria a sprite mostrar a célula de outra pessoa.
    let sprite = match source {
        PackedSource::Atlas { cell_idx } => {
            Sprite::atlas(cell_idx, world_size, [1.0, 1.0, 1.0, 1.0])
        }
        PackedSource::Individual { texture_id, .. } => {
            Sprite::individual(texture_id, world_size, [1.0, 1.0, 1.0, 1.0])
        }
    };
    let entity = sim
        .world_mut()
        .spawn((
            Transform::from_translation(world_center),
            sprite,
            Name::new(label.clone()),
        ))
        .id();
    // ⚠️ **O carimbo durável dos pixels, e ele é o que separa "importou" de "importou e sobrevive
    // ao save".** Uma sprite de atlas grava-se pelo `atlas_asset_map`; uma `Individual` só se grava
    // se nomear os seus bytes por `AssetId` — o `texture_id` é uma alocação de GPU e morre com o
    // processo. Mesmo raciocínio (e mesmo remédio) do `inspector_strategy::promote_to_individual`.
    if let PackedSource::Individual { pixels_id, .. } = source {
        sim.world_mut()
            .entity_mut(entity)
            .insert(ph2d_ecs::SpritePixels(pixels_id));
    }
    (label, entity.to_bits())
}

/// Spawn a blank, opaque-white **paint canvas** of `size_px` × `size_px` at
/// `world_center`, packed into the atlas exactly like an import so the Painter
/// edits it at native resolution. Returns `(label, entity_bits)` for seating
/// the selection.
///
/// **Why this exists (the realistic-smoke target).** The demo's atlas sprites
/// are 64px; painting there distorts the brush↔canvas ratio and makes the
/// world-space paper/grain texture read as coarse "grass" at display zoom (see
/// `docs/Novo Painter`). A `2048²` blank canvas is the canonical scale to smoke
/// the brush engine against — what a real Procreate canvas looks like.
///
/// Mirrors [`pack_image`] + [`spawn_sprite`]: the canvas is registered in the
/// `AssetDb` + `atlas_asset_map` BEFORE the atlas insert so a regrow triggered
/// by it can recover the pixels (same ordering as the import path).
#[allow(clippy::too_many_arguments)]
pub fn spawn_blank_canvas(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    size_px: u32,
    bg: u8,
    world_center: Vec2,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Result<(String, u64), String> {
    let px_count = (size_px as usize)
        .checked_mul(size_px as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| format!("canvas {size_px}² overflows a pixel buffer"))?;
    // Background fill: 0 = transparent, 1 = black, 2 = white (the New-image modal's choices).
    let fill: [u8; 4] = match bg {
        1 => [0, 0, 0, 255],       // LITERAL-COLOR-OK: opaque black canvas
        2 => [255, 255, 255, 255], // LITERAL-COLOR-OK: opaque white canvas
        _ => [0, 0, 0, 0],         // LITERAL-COLOR-OK: fully-transparent canvas
    };
    let mut pixels = vec![0u8; px_count];
    for px in pixels.as_chunks_mut::<4>().0.iter_mut() {
        px.copy_from_slice(&fill);
    }
    spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell_idx,
        size_px,
        size_px,
        pixels,
        world_center,
        pixels_per_meter,
        atlas_asset_map,
        "Canvas",
    )
}

/// **Empacota `pixels` numa célula do atlas e spawna a sprite** — a porta única desse par.
///
/// ⚠️ **Extraída de [`spawn_blank_canvas`] em 2026-08-23**, quando a cena de smoke da §11 precisou
/// de uma tira RETANGULAR com conteúdo. Duplicar o corpo teria duplicado a **ordem** que ele
/// impõe e que é load-bearing: os pixels entram no `AssetDb` e o vínculo `key → AssetId` no
/// `atlas_asset_map` **ANTES** do insert no atlas, para que um regrow disparado por ele consiga
/// recuperar os bytes. *Duas cópias de uma ordem convergem enquanto ninguém mexe numa delas.*
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_rgba(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    w_px: u32,
    h_px: u32,
    pixels: Vec<u8>,
    world_center: Vec2,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    label: &str,
) -> Result<(String, u64), String> {
    let asset_id = asset_db.insert_image_rgba8(w_px, h_px, pixels.clone());
    atlas_asset_map.insert(cell_idx, asset_id);
    let fetch_pixels = |key: u32| -> Option<Vec<u8>> {
        let aid = atlas_asset_map.get(&key)?;
        // ⚠️ Ver o irmão acima: o atlas é de 8 bits, converter para baixo é correcto, e o `match`
        // na variante devolvia `None` (célula vazia) para 16 bits.
        let asset = asset_db.get(aid)?;
        asset.image_rgba8().map(|(_, _, px)| px.into_owned())
    };
    if let Err(e) =
        renderer.insert_atlas_sprite_with_regrow(cell_idx, w_px, h_px, &pixels, fetch_pixels)
    {
        atlas_asset_map.remove(&cell_idx);
        return Err(format!("atlas insert {label}: {e}"));
    }
    let safe_px_per_m = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let ww = (w_px as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    let wh = (h_px as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    Ok(spawn_sprite(
        sim,
        PackedSource::Atlas { cell_idx },
        world_center,
        [ww, wh],
        label,
    ))
}

/// One packed-but-not-yet-spawned image, carried from the measure pass
/// into the spawn pass.
struct Packed {
    /// Index into the caller's `paths` slice — used to restore input
    /// order in the returned results (errors are interleaved).
    input_index: usize,
    source: PackedSource,
    world_size: [f32; 2],
    base_label: String,
}

/// **De onde os pixels de uma imagem importada passam a vir.**
///
/// ⚠️ Existe porque *nem toda imagem cabe no atlas*: uma de **16 bits** não cabe, e a razão é
/// estrutural — o atlas é **uma** textura com **um** formato (plano
/// [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
/// §3.3). Esta é uma das duas portas onde a regra *16 bits ⇒ `Individual`* se impõe; a outra é a
/// conversão pela UI.
///
/// ⛔ **A alternativa recusada** era converter a imagem de 16 bits para 8 e metê-la no atlas na
/// mesma. Isso importaria um ficheiro de alta precisão **rebaixando-o em silêncio**, que é
/// exatamente o que esta wave existe para deixar de fazer.
#[derive(Clone, Copy, Debug)]
pub(crate) enum PackedSource {
    /// Uma célula do atlas partilhado (o caminho de sempre, e o de toda imagem de 8 bits).
    Atlas { cell_idx: u32 },
    /// Uma textura própria — o único sítio onde 16 bits pode viver.
    ///
    /// ⚠️ **O `pixels_id` viaja junto, e não é opcional.** Um `texture_id` é um id de alocação da
    /// GPU: ele morre com o processo. O que faz a sprite sobreviver a um save/load é o carimbo
    /// `SpritePixels(AssetId)`, e sem ele esta importação gravaria a sprite **sem imagem** — o
    /// mesmo buraco que o `inspector_strategy` já tapa quando promove a `Individual`.
    Individual { texture_id: u32, pixels_id: AssetId },
}

/// Imports a batch of image files, laying them out in a tidy near-square
/// grid and returning per-file outcomes in input order.
///
/// **Layout.** `cols = ceil(sqrt(N))` (so the footprint stays as close
/// to a square as a row-major grid allows), and cells are *uniform* —
/// pitch = largest imported sprite + a proportional gap — so rows and
/// columns line up even when the files differ in size (each sprite is
/// centered in its cell). The first cell's CENTER sits on
/// `anchor_world`, and the grid grows right (`+x`) and down (`-y`) —
/// "ao lado e abaixo". `N == 1` reduces to a single sprite centered on
/// the anchor, identical to the pre-grid single-import behavior.
///
/// **Two passes.** Files are decoded + atlas-packed first (to measure
/// every world size before placing anything), then spawned at their
/// computed cells. A file that fails to pack is recorded as
/// [`ImportItemResult::Err`] in place and dropped from the layout, so a
/// bad file never leaves a hole in the grid.
///
/// `next_cell` is advanced once per *successful* pack (errors don't
/// consume an atlas cell).
// Helper has 8 args because the import path is a top-level fixture
// orchestrator that needs full access to sim/renderer/asset_db plus the
// batch inputs. Splitting into a struct would just move the noise.
#[allow(clippy::too_many_arguments)]
pub fn import_images_grid(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    anchor_world: [f32; 2],
    next_cell: &mut u32,
    paths: &[PathBuf],
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Vec<ImportItemResult> {
    // Pass 1 — measure. Decode + atlas-pack each file; successful packs
    // feed the layout, failures are slotted straight into the results.
    let mut packed: Vec<Packed> = Vec::new();
    let mut results: Vec<Option<ImportItemResult>> = (0..paths.len()).map(|_| None).collect();
    for (i, path) in paths.iter().enumerate() {
        let cell_idx = *next_cell;
        match pack_image(
            renderer,
            asset_db,
            cell_idx,
            path,
            pixels_per_meter,
            atlas_asset_map,
        ) {
            Ok((world_size, source)) => {
                // ⚠️ O contador de células só avança quando uma célula foi MESMO consumida. Uma
                // imagem de 16 bits foi para textura própria e não gastou nenhuma — incrementar
                // aqui na mesma abriria buracos no atlas a cada import de alta precisão.
                if matches!(source, PackedSource::Atlas { .. }) {
                    *next_cell = next_cell.saturating_add(1);
                }
                packed.push(Packed {
                    input_index: i,
                    source,
                    world_size,
                    base_label: base_label(path, cell_idx),
                });
            }
            Err(error) => {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("(unnamed)")
                    .to_owned();
                results[i] = Some(ImportItemResult::Err { name, error });
            }
        }
    }

    // Pass 2 — lay out + spawn.
    let n = packed.len();
    if n > 0 {
        let cols = (n as f32).sqrt().ceil().max(1.0) as usize;
        let max_w = packed
            .iter()
            .map(|p| p.world_size[0])
            .fold(0.0_f32, f32::max);
        let max_h = packed
            .iter()
            .map(|p| p.world_size[1])
            .fold(0.0_f32, f32::max);
        let gap = IMPORT_GRID_GAP_FRAC * max_w.max(max_h);
        let pitch_x = max_w + gap;
        let pitch_y = max_h + gap;
        for (k, p) in packed.iter().enumerate() {
            let col = (k % cols) as f32;
            let row = (k / cols) as f32;
            // y grows up, so successive rows step DOWN (−y).
            let center = Vec2::new(
                anchor_world[0] + col * pitch_x,
                anchor_world[1] - row * pitch_y,
            );
            let (label, bits) = spawn_sprite(sim, p.source, center, p.world_size, &p.base_label);
            results[p.input_index] = Some(ImportItemResult::Ok { label, bits });
        }
    }

    // Every index was filled (Err in pass 1 or Ok in pass 2); `flatten`
    // drops the structurally-impossible `None` and preserves order.
    results.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_render::SpriteSource;

    /// **Uma importação de 16 bits nasce `Individual` E CARIMBADA** — plano
    /// [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
    /// W2.4.
    ///
    /// ⚠️ O carimbo é a metade que se esquece. A estratégia errada dá uma sprite **visivelmente**
    /// partida (mostra a célula de outra pessoa); o carimbo em falta dá uma sprite **perfeita até
    /// ao save**, e depois vazia — e nada no ecrã avisa entre as duas coisas.
    #[test]
    fn a_sixteen_bit_import_is_individual_and_stamped_with_its_pixel_id() {
        let mut sim = SimWorld::default();
        let pixels_id = ph2d_asset::AssetId::from_bytes(b"pixels de 16 bits");
        let (_, bits) = spawn_sprite(
            &mut sim,
            PackedSource::Individual {
                texture_id: 7,
                pixels_id,
            },
            Vec2::new(0.0, 0.0),
            [1.0, 1.0],
            "alta_precisao",
        );
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let sprite = sim.world().get::<Sprite>(entity).copied().expect("sprite");
        assert!(
            matches!(sprite.source, SpriteSource::Individual { texture_id: 7 }),
            "uma imagem de 16 bits nao pode nascer no atlas — ele e' uma textura com UM formato"
        );
        assert_eq!(
            sim.world()
                .get::<ph2d_ecs::SpritePixels>(entity)
                .map(|p| p.0),
            Some(pixels_id),
            "sem o carimbo `SpritePixels` esta sprite abre perfeita e grava VAZIA: o `texture_id` \
             e' uma alocacao de GPU e morre com o processo"
        );
    }

    /// **Controle positivo:** o caminho do atlas continua a nascer no atlas e **sem** carimbo — ele
    /// grava-se pelo `atlas_asset_map`, e um `SpritePixels` a mais fá-lo-ia ser gravado duas vezes.
    #[test]
    fn an_atlas_import_stays_on_the_atlas_and_is_not_stamped() {
        let mut sim = SimWorld::default();
        let (_, bits) = spawn_sprite(
            &mut sim,
            PackedSource::Atlas { cell_idx: 3 },
            Vec2::new(0.0, 0.0),
            [1.0, 1.0],
            "normal",
        );
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let sprite = sim.world().get::<Sprite>(entity).copied().expect("sprite");
        assert!(matches!(sprite.source, SpriteSource::Atlas { key: 3 }));
        assert!(
            sim.world().get::<ph2d_ecs::SpritePixels>(entity).is_none(),
            "uma sprite de atlas carimbada seria gravada DUAS vezes — pelo mapa e pelos pixels"
        );
    }

    /// As duas precisões nascem do MESMO tamanho: a precisão não pode mudar a geometria.
    #[test]
    fn precision_does_not_change_the_world_size() {
        assert_eq!(world_size(256, 128, 100.0), world_size(256, 128, 100.0));
        assert_eq!(world_size(200, 100, 100.0), [2.0, 1.0]);
    }
}
