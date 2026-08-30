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

/// ⭐⭐⭐ **O CHIP *Pattern* NÃO ESCOLHE A ARTE PELO ARTISTA** (report do Enio, 2026-08-30: *"ao
/// apertar pattern o usuário é obrigado a selecionar uma img no dialog. não tem a opção de usar
/// shape até que se use a img em pattern"*).
///
/// # A régua, e porque ela não é o valor devolvido
///
/// O defeito era um **efeito colateral**: a porta abria um diálogo de ficheiro. Um gate que só
/// olhasse o valor devolvido ficaria verde com o diálogo ainda lá — e num arnês sem ecrã o diálogo
/// **bloqueia ou devolve `None`**, o que se leria como "a função não fez nada".
///
/// ⇒ o que se afirma é a coisa que só é verdade **sem** diálogo: a porta é PURA. Ela deixou de
/// receber o `AssetDb` (não há o que descodificar nem inserir), e sem ele **não há forma de ela
/// produzir uma `PatternSource::Image`** — o tipo diz o que o comentário prometia. Este gate corre
/// num teste normal, o que por si só prova que ela não abre nada: um `rfd::FileDialog` num arnês
/// headless não voltaria daqui.
///
/// ⚠️ **E a segunda metade importa tanto como a primeira:** trocar de chip e voltar **não perde a
/// arte**. Sem ela, a cura seria "o chip apaga o que estava lá".
#[test]
fn choosing_pattern_on_a_bare_shape_does_not_pick_the_art_for_the_artist() {
    let mut scene = VecScene::default();
    let nua = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    assert_eq!(
        source_for(&scene, nua),
        Some(PatternSource::None),
        "o chip Pattern numa forma sem padrao tem de nascer SEM arte escolhida - se ele escolher, \
         escolhe sempre a mesma, e a outra arte fica atras dela"
    );
    // ⚠️ CONTROLO: numa forma que JÁ tem padrão, a porta devolve a arte que lá está.
    let vestida = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(ph2d_vec_scene::Paint::Pattern(Box::new(
            ph2d_vec_scene::PatternFill::new(
                PatternSource::Shape(nua),
                [2.0, 2.0],
                ph2d_vec_scene::Rgba8::new(1, 2, 3, 255),
            ),
        ))),
        ..VecPath::default()
    });
    assert_eq!(
        source_for(&scene, vestida),
        Some(PatternSource::Shape(nua)),
        "trocar de chip e voltar perdeu a arte"
    );
}
