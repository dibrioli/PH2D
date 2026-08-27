//! Os gates do memo de padrões (plano 33, W4).

use super::*;
use ph2d_vec_pattern::TileKind;
use ph2d_vec_scene::{PatternFill, Rgba8, VecPath, VecVertex};

/// Arte 4x4 opaca com um texel por coordenada.
fn art_bytes() -> Vec<u8> {
    let mut v = Vec::new();
    for y in 0..4u8 {
        for x in 0..4u8 {
            v.extend_from_slice(&[x * 16, y * 16, 200, 255]);
        }
    }
    v
}

fn scene_with(fill: PatternFill) -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(fill))),
        ..VecPath::default()
    });
    (scene, id)
}

fn db_with_art() -> (AssetDb, ph2d_asset::AssetId) {
    let db = AssetDb::new();
    let id = db.insert_image_rgba8(4, 4, art_bytes());
    (db, id)
}

/// **A arte carregada vira ladrilho, e a grade encostada devolve-a ao tamanho.**
#[test]
fn a_loaded_image_becomes_a_tile() {
    let (db, asset) = db_with_art();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(&scene, &db, ImageQuality::Medium);
    let tile = live.tiles().get(&path).expect("o padrao assou");
    assert_eq!(tile.tile_px, [4, 4]);
    assert_eq!(tile.cells, [1, 1]);
}

/// ⚠️ **Uma fonte que não resolve NÃO produz ladrilho** — e é isso que faz a forma pintar a
/// `fallback` em vez de desaparecer.
#[test]
fn an_unresolved_source_produces_no_tile() {
    let db = AssetDb::new();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(ph2d_asset::AssetId::from_bytes(b"nunca importada")),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(&scene, &db, ImageQuality::Medium);
    assert!(live.tiles().get(&path).is_none());
    // E uma fonte-FORMA também não, enquanto a W7 não existir.
    let (scene2, path2) = scene_with(PatternFill::new(
        PatternSource::Shape(7),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    live.recook(&scene2, &db, ImageQuality::Medium);
    assert!(live.tiles().get(&path2).is_none());
}

/// ⭐ **O memo acerta: re-assar sem mudar nada devolve o MESMO handle** (o mesmo `Blob`, logo a
/// mesma textura no atlas do Vello).
///
/// ⚠️ Um handle novo por quadro faria o Vello re-enviar a textura **todo quadro** — é a razão de o
/// `StableImage` existir, e é invisível a qualquer gate que olhe só para os pixels.
#[test]
fn recooking_an_unchanged_pattern_keeps_the_same_handle() {
    let (db, asset) = db_with_art();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(&scene, &db, ImageQuality::Medium);
    let first = live.tiles()[&path].image.clone();
    live.recook(&scene, &db, ImageQuality::Medium);
    let second = &live.tiles()[&path].image;
    assert_eq!(
        first.width(),
        second.width(),
        "o memo re-assou o que nao mudou"
    );
    // Controlo: mudar a LEI tem de re-assar — senão este gate estaria a medir um mapa congelado.
    let mut f2 = PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f2.kind = TileKind::BrickRow;
    f2.offset_denom = 3;
    let (scene2, path2) = scene_with(f2);
    live.recook(&scene2, &db, ImageQuality::Medium);
    assert_eq!(
        live.tiles()[&path2].tile_px,
        [4, 12],
        "o tijolo de 1/3 pede tres linhas"
    );
}

/// ⚠️ **O filtro actualiza-se SEM re-assar** — ele escolhe a amostragem na GPU e não toca um byte
/// do ladrilho. Metê-lo na chave faria alternar o modo de imagem re-assar a cena inteira para
/// produzir os mesmos pixels.
#[test]
fn changing_the_filter_does_not_rebake() {
    let (db, asset) = db_with_art();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(&scene, &db, ImageQuality::Medium);
    live.recook(&scene, &db, ImageQuality::Low);
    let t = &live.tiles()[&path];
    assert_eq!(t.quality, ImageQuality::Low, "o filtro tem de acompanhar");
    assert_eq!(t.tile_px, [4, 4]);
}

/// ⚠️⚠️ **A varredura tem as DUAS metades.** Uma forma que deixou de ter padrão (ou que foi
/// apagada) não pode continuar a desenhar o ladrilho dela — nem a mantê-lo em memória.
#[test]
fn a_shape_that_loses_its_pattern_loses_its_tile() {
    let (db, asset) = db_with_art();
    let (mut scene, path) = scene_with(PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    let mut live = TexturePatternLive::default();
    live.recook(&scene, &db, ImageQuality::Medium);
    assert!(live.tiles().contains_key(&path));
    scene.path_mut(path).unwrap().fill = Some(Paint::solid(Rgba8::new(9, 9, 9, 255)));
    live.recook(&scene, &db, ImageQuality::Medium);
    assert!(
        live.tiles().is_empty(),
        "o ladrilho sobreviveu ao padrao que o pediu"
    );
}

/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"em column o pattern some"*.** Um ladrilho que não caberia
/// no atlas é **REDUZIDO até caber**, não recusado — e a forma continua a mostrar o padrão.
///
/// ⚠️ **Este gate afirmava o CONTRÁRIO até hoje** (`..._leaves_no_tile`), e foi ele que apanhou a
/// mudança de lei quando o assador passou a reduzir. *Um gate que se torna vermelho por causa de uma
/// cura é um gate a fazer o trabalho dele — ele obriga a dizer, por escrito, que a lei mudou.*
#[test]
fn a_tile_too_big_for_the_atlas_is_scaled_and_still_shows() {
    let db = AssetDb::new();
    let big = ph2d_vec_pattern::MAX_TILE_EDGE_PX / 2;
    let asset = db.insert_image_rgba8(big, 4, vec![0u8; (big as usize) * 4 * 4]);
    let mut f = PatternFill::new(
        PatternSource::Image(asset),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f.kind = TileKind::BrickCol;
    f.offset_denom = 3; // 3 x (MAX/2) passaria do tecto
    let (scene, path) = scene_with(f);
    let mut live = TexturePatternLive::default();
    live.recook(&scene, &db, ImageQuality::Medium);
    let tile = live
        .tiles()
        .get(&path)
        .expect("o ladrilho tem de ser REDUZIDO, nao recusado");
    assert!(
        tile.tile_px[0] <= ph2d_vec_pattern::MAX_TILE_EDGE_PX
            && tile.tile_px[1] <= ph2d_vec_pattern::MAX_TILE_EDGE_PX,
        "o reduzido nao coube: {:?}",
        tile.tile_px
    );
    assert_eq!(tile.cells, [3, 1], "a LEI nao muda com a reducao");
}
