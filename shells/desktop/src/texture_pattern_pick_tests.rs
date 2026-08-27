//! Os gates do NASCIMENTO de um padrão (plano 33; report do Enio de 2026-08-27).

use super::*;
use ph2d_asset::AssetDb;
use ph2d_vec_scene::{VecPath, VecVertex};

/// ⛔⛔ **UM PADRÃO NASCE SOBRE A FORMA, NÃO NA ORIGEM DO MUNDO.**
///
/// Com `Tile`/`Mirror` a diferença é invisível (o padrão repete-se por toda a parte); com `Clamp` é
/// catastrófica: o `Extend::Pad` devolve a **borda** da arte esticada, e o artista vê um borrão
/// chapado. Medido na cena de smoke antes da cura: as seis formas caíam em `uv.x` de **−331 a +331**
/// com o ladrilho a cobrir `0..32`.
///
/// ⚠️ É a metade que faltava da lei que o plano se gabava de honrar: eu evitei a ancoragem à origem
/// da régua do Illustrator na metade do TRANSFORM (o padrão anda com a forma) e reproduzi-a na
/// metade do NASCIMENTO.
#[test]
fn a_new_pattern_is_born_over_the_shape_not_at_the_world_origin() {
    let db = AssetDb::new();
    let art = db.insert_image_rgba8(4, 2, vec![9u8; 4 * 2 * 4]);
    let source = PatternSource::Image(art);
    let mut scene = VecScene::default();
    // Uma forma LONGE da origem — é aí que o defeito se vê.
    let id = scene.push_path(VecPath {
        verts: [[100.0, 50.0], [106.0, 50.0], [106.0, 56.0], [100.0, 56.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let (size, origin) = default_placement(&db, &scene, id, &source);
    assert_eq!(origin, [100.0, 50.0], "o padrao nasceu na origem do MUNDO");
    // E o tamanho continua a preservar o aspecto 2:1 da arte.
    assert!(
        (size[0] / size[1] - 2.0).abs() < 1e-9,
        "o aspecto 2:1 da arte nao sobreviveu: {size:?}"
    );
    // ⚠️ Controlo: uma forma NOUTRO sítio nasce noutro canto — senão este gate estaria a medir uma
    // constante.
    let id2 = scene.push_path(VecPath {
        verts: [[-7.0, -3.0], [-5.0, -3.0], [-5.0, -1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let (_, o2) = default_placement(&db, &scene, id2, &source);
    assert_eq!(o2, [-7.0, -3.0]);
}
