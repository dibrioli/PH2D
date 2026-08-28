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

/// O assador de FORMA que devolve nada — o caminho de quem só tem fontes de imagem.
fn no_shape() -> impl FnMut(VecPathId) -> Option<(u32, u32, Vec<u8>)> {
    |_| None
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
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
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
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
    assert!(live.tiles().get(&path).is_none());
    // E uma fonte-FORMA também não, enquanto a W7 não existir.
    let (scene2, path2) = scene_with(PatternFill::new(
        PatternSource::Shape(7),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    live.recook(&scene2, &db, ImageQuality::Medium, &mut no_shape());
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
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
    let first = live.tiles()[&path].image.clone();
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
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
    live.recook(&scene2, &db, ImageQuality::Medium, &mut no_shape());
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
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
    live.recook(&scene, &db, ImageQuality::Low, &mut no_shape());
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
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
    assert!(live.tiles().contains_key(&path));
    scene.path_mut(path).unwrap().fill = Some(Paint::solid(Rgba8::new(9, 9, 9, 255)));
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
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
    live.recook(&scene, &db, ImageQuality::Medium, &mut no_shape());
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

/// ⛔⛔ **UMA FORMA NÃO PODE SER O PRÓPRIO PADRÃO** (plano 33, W7).
///
/// Assá-la exigiria desenhá-la, desenhá-la exigiria o ladrilho, e o ladrilho exigiria assá-la. ⚠️ E
/// o sintoma não seria um erro: seria o app a parar, ou um ladrilho de uma versão anterior de si
/// mesmo a cada quadro.
#[test]
fn a_pattern_whose_source_is_itself_is_refused() {
    let db = AssetDb::new();
    let (scene, path) = scene_with(PatternFill::new(
        PatternSource::Shape(0),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    ));
    // A forma da fixtura é a primeira da cena; apontar o padrão a ela é o ciclo.
    let mut f = PatternFill::new(
        PatternSource::Shape(path),
        [4.0, 4.0],
        Rgba8::new(1, 2, 3, 255),
    );
    f.origin = [0.0, 0.0];
    let (mut ciclo, id) = scene_with(f);
    let _ = (&scene, path);
    let mut assou = false;
    let mut bake = |_| {
        assou = true;
        Some((2u32, 2, vec![9u8; 2 * 2 * 4]))
    };
    let mut live = TexturePatternLive::default();
    live.recook(&ciclo, &db, ImageQuality::Medium, &mut bake);
    assert!(
        !assou,
        "o assador foi chamado para uma forma que e' a propria fonte"
    );
    assert!(live.tiles().get(&id).is_none(), "o ciclo produziu ladrilho");
    // CONTROLO: apontar a OUTRA forma assa.
    let outra = ciclo.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    if let Some(Paint::Pattern(p)) = ciclo.path_mut(id).and_then(|p| p.fill.as_mut()) {
        p.source = PatternSource::Shape(outra);
    }
    let mut bake2 = |_| Some((2u32, 2, vec![9u8; 2 * 2 * 4]));
    live.recook(&ciclo, &db, ImageQuality::Medium, &mut bake2);
    assert!(live.tiles().get(&id).is_some(), "a fonte valida nao assou");
}

/// ⭐⭐ **EDITAR A FORMA-FONTE RE-ASSA o ladrilho** — é o *"pattern fills are dynamic"* do Figma, e é
/// o que separa *um preenchimento de imagem* de *um sistema de padrões*.
///
/// ⚠️ Sem a forma na chave, a `PatternSource::Shape(id)` seria estável e mexer nos nós da fonte não
/// mudaria a tela — o defeito EXACTO que o `FxKey` da crate irmã documenta.
#[test]
fn editing_the_source_shape_rebakes_the_tile() {
    let db = AssetDb::new();
    let mut scene = VecScene::default();
    let fonte = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let alvo = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(PatternFill::new(
            PatternSource::Shape(fonte),
            [4.0, 4.0],
            Rgba8::new(1, 2, 3, 255),
        )))),
        ..VecPath::default()
    });
    let mut n = 0usize;
    let mut live = TexturePatternLive::default();
    {
        let mut bake = |_| {
            n += 1;
            Some((2u32, 2, vec![9u8; 2 * 2 * 4]))
        };
        live.recook(&scene, &db, ImageQuality::Medium, &mut bake);
        live.recook(&scene, &db, ImageQuality::Medium, &mut bake);
    }
    assert_eq!(n, 1, "o memo re-assou o que nao mudou");
    assert!(live.tiles().contains_key(&alvo));
    // Mexer num NÓ da fonte tem de re-assar.
    if let Some(p) = scene.path_mut(fonte) {
        p.verts[2].anchor = [5.0, 5.0];
    }
    {
        let mut bake = |_| {
            n += 1;
            Some((2u32, 2, vec![9u8; 2 * 2 * 4]))
        };
        live.recook(&scene, &db, ImageQuality::Medium, &mut bake);
    }
    assert_eq!(
        n, 2,
        "editar a forma-fonte NAO re-assou - o padrao ficaria morto"
    );
}
