//! Os gates do preenchimento com PADRÃO (plano 33, W2) — e o mais importante deles não é sobre o
//! padrão: é sobre a **diferença** entre ele e a caneta.

use super::pattern::{extend_of, fill_pattern};
use super::stroke_uniform::{is_conformal, pen_for, uniform_scale};
use ph2d_vec_pattern::PatternMode;
use ph2d_vector::{Affine, BezPath, Extend, Fill, ImageQuality, StableImage, Stroke, VectorScene};
use std::sync::Arc;

fn square() -> BezPath {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0, 1.0));
    bp.line_to((0.0, 1.0));
    bp.close_path();
    bp
}

pub(super) fn tile() -> StableImage {
    StableImage::from_rgba(Arc::new(vec![255u8, 0, 0, 255]), 1, 1).expect("1x1 RGBA")
}

/// A colocação de referência: um ladrilho `1x1 px` que mede 4 unidades de mundo, posto em (5, 5).
const PLACEMENT: [f64; 6] = [4.0, 0.0, 0.0, 4.0, 5.0, 5.0];

fn encode(transform: Affine, placement: [f64; 6], mode: PatternMode) -> VectorScene {
    let mut s = VectorScene::new();
    fill_pattern(
        &mut s,
        &square(),
        Fill::NonZero,
        transform,
        &tile(),
        placement,
        mode,
        ImageQuality::Medium,
        1.0,
    );
    s
}

/// **Cada modo tem o SEU `Extend`, e a tradução é injectiva.**
///
/// ⚠️ Um mapeamento que colapse dois modos num só é o defeito silencioso desta wave: o painel
/// ofereceria três opções e duas delas fariam a mesma coisa. O gate exige os três **distintos**,
/// não só "cada um mapeia para algo".
#[test]
fn each_pattern_mode_maps_to_its_own_extend() {
    assert_eq!(extend_of(PatternMode::Tile), Extend::Repeat);
    assert_eq!(extend_of(PatternMode::Mirror), Extend::Reflect);
    assert_eq!(extend_of(PatternMode::Clamp), Extend::Pad);
    let all = [
        extend_of(PatternMode::Tile),
        extend_of(PatternMode::Mirror),
        extend_of(PatternMode::Clamp),
    ];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j], "dois modos colapsaram no mesmo Extend");
        }
    }
}

/// ⭐⭐⭐ **O PADRÃO ESMAGA COM A FORMA; A CANETA NÃO. E as duas leis estão CERTAS.**
///
/// Esta é a pergunta 3 da [folha 29](../../../docs/Vector%20Module/29_fila_morph_state_machine_e_texture_pattern.md),
/// e ela aponta o [bug #27](../../../docs/Vector%20Module/BUGS_vector.md) — onde a caneta virava
/// elipse sob escala não-uniforme e o Enio decidiu `√|det|`. ⛔ **A analogia não se aplica**, e
/// confundi-las seria construir a coisa errada:
///
/// - o **traço** é a FERRAMENTA que desenha a forma — a caneta do artista não muda de feitio porque
///   a forma foi esticada;
/// - o **preenchimento** está COLADO à forma. Um gradiente radial já vira elipse sob escala
///   não-uniforme, **hoje**, e ninguém chamou a isso um defeito.
///
/// O gate mede as duas com o MESMO predicado da casa (`is_conformal`) e o mesmo afim, para que a
/// próxima pessoa que ler o bug #27 encontre a diferença escrita em vez de a redescobrir.
#[test]
fn the_pattern_shears_with_the_shape_unlike_the_pen() {
    let m = Affine::scale_non_uniform(3.0, 1.0);
    assert!(!is_conformal(m), "a fixtura tem de conter o fenomeno");

    // O PADRÃO: o afim efectivo do pincel é `transform * placement` (o Vello compõe-os), e ele
    // herda a distorção da forma.
    let effective = m * Affine::new(PLACEMENT);
    assert!(
        !is_conformal(effective),
        "o padrao tem de esmagar com a forma"
    );

    // A CANETA: a porta única do traço devolve um afim CONFORME para o mesmo `m`.
    let (_, pen_xf) = pen_for(&Stroke::new(1.0), m);
    assert!(is_conformal(pen_xf), "a caneta continua redonda");

    // ⚠️ E o padrão não passa pela lei da caneta: uniformizar mudaria o encode.
    let uni = Affine::scale(uniform_scale(m));
    assert_ne!(
        encode(m, PLACEMENT, PatternMode::Tile)
            .inner()
            .encoding()
            .transforms,
        encode(uni, PLACEMENT, PatternMode::Tile)
            .inner()
            .encoding()
            .transforms,
        "o padrao foi uniformizado como se fosse a caneta"
    );
}

/// ⚠️ **A COLOCAÇÃO viaja até ao encoding.** Sem isto o padrão nasceria sempre na origem com um
/// ladrilho do tamanho de um pixel — visível, mas de uma forma que se confunde com "a arte está
/// errada" em vez de "a colocação foi ignorada".
#[test]
fn the_placement_reaches_the_encoding() {
    let id = Affine::IDENTITY;
    let a = encode(id, PLACEMENT, PatternMode::Tile);
    let b = encode(id, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0], PatternMode::Tile);
    assert_ne!(
        a.inner().encoding().transforms,
        b.inner().encoding().transforms,
        "a colocacao foi descartada"
    );
}

/// **O modo viaja até ao encoding** — o espelho do gate da folha, mas do lado que de facto encoda.
#[test]
fn the_mode_reaches_the_encoding() {
    let id = Affine::IDENTITY;
    let of = |m| {
        encode(id, PLACEMENT, m)
            .inner()
            .encoding()
            .draw_data
            .clone()
    };
    assert_ne!(of(PatternMode::Tile), of(PatternMode::Clamp));
    assert_ne!(of(PatternMode::Mirror), of(PatternMode::Clamp));
    assert_ne!(of(PatternMode::Tile), of(PatternMode::Mirror));
}

/// ⭐ **O kill-criterion, do lado da crate que desenha o documento:** uma forma com padrão encoda
/// **um** caminho e **zero** camadas — o mesmo que uma cor chapada.
#[test]
fn a_pattern_costs_one_path_and_no_layer() {
    let s = encode(Affine::IDENTITY, PLACEMENT, PatternMode::Tile);
    assert_eq!(s.inner().encoding().n_clips, 0);
    assert_eq!(s.inner().encoding().n_paths, 1);
}

/// ⭐ **Um padrão sem ladrilho resolvido pinta a `fallback` — e isso é desenho CERTO.**
///
/// A arte pode ainda não ter carregado, a forma-fonte pode ter desaparecido, o assado pode ter
/// recusado por tamanho. ⚠️ **Desenhar NADA seria pior**: uma forma invisível lê-se como *"a
/// ferramenta está partida"* e não se distingue de um preenchimento vazio. Mesmo papel do
/// `fallback` do `ProceduralFill` (ADR-0056-amendment-3).
#[test]
fn a_pattern_without_a_tile_paints_its_fallback() {
    use ph2d_vec_scene::{Paint, PatternFill, PatternSource, Rgba8, VecPath, VecPathId, VecVertex};
    let cor = Rgba8::new(200, 30, 30, 255);
    let mut path = VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    path.fill = Some(Paint::Pattern(Box::new(PatternFill::new(
        PatternSource::Shape(7 as VecPathId),
        [4.0, 4.0],
        cor,
    ))));
    // A cor de swatch é a `fallback` — e o pincel de recurso pinta exactamente essa cor.
    assert_eq!(path.fill.as_ref().unwrap().primary_color(), cor);
    // Controlo: um sólido de outra cor dá outro pincel, senão este gate mediria uma constante.
    let mut outro = path.clone();
    outro.fill = Some(Paint::solid(Rgba8::new(1, 2, 3, 255)));
    assert_ne!(
        outro.fill.as_ref().unwrap().primary_color(),
        path.fill.as_ref().unwrap().primary_color()
    );
}

/// ⭐⭐ **O LADRILHO SUBSTITUI A `fallback` — e sem ladrilho o encode é EXACTAMENTE o de uma cor
/// chapada.**
///
/// Esta é a afirmação forte da W4, e ela é byte-a-byte: uma forma com `Paint::Pattern` sem ladrilho
/// resolvido encoda o **mesmo** que a mesma forma com `Paint::Solid(fallback)`. Isso é o que faz a
/// pré-visualização ser *desenho certo* e não um estado degradado com aparência própria.
#[test]
fn a_tile_replaces_the_fallback_and_without_one_the_encode_is_the_solids() {
    use crate::{VecViewState, VecXforms};
    use ph2d_vec_scene::{Paint, PatternFill, PatternSource, Rgba8, VecPath, VecScene, VecVertex};
    let cor = Rgba8::new(200, 30, 30, 255);
    let shape = |fill: Paint| {
        let mut scene = VecScene::default();
        scene.push_path(VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            fill: Some(fill),
            ..VecPath::default()
        });
        scene
    };
    let run = |scene: &VecScene, tiles: &crate::PatternTiles| {
        let mut target = VectorScene::new();
        crate::dispatch(
            scene,
            &VecViewState::default(),
            &VecXforms::new(),
            &crate::LiveGeometry::new(),
            &crate::FxImages::new(),
            &crate::WidgetSkins::new(),
            tiles,
            &crate::BrushArts::new(),
            Affine::IDENTITY,
            &mut target,
        );
        target
    };
    let pat = shape(Paint::Pattern(Box::new(PatternFill::new(
        PatternSource::Shape(1),
        [4.0, 4.0],
        cor,
    ))));
    let solid = shape(Paint::solid(cor));
    let empty = crate::PatternTiles::new();

    // 1. Sem ladrilho: BYTE-A-BYTE o encode de um sólido da cor de recurso.
    let a = run(&pat, &empty);
    let b = run(&solid, &empty);
    // ⚠️ `DrawTag` não implementa `Debug`, então a comparação é `assert!` e não `assert_eq!`.
    assert!(a.inner().encoding().draw_tags == b.inner().encoding().draw_tags);
    assert_eq!(
        a.inner().encoding().draw_data,
        b.inner().encoding().draw_data
    );

    // 2. Com ladrilho: deixa de o ser, e continua a NÃO empurrar camada.
    let mut tiles = crate::PatternTiles::new();
    let id = pat.paths()[0].id;
    tiles.insert(
        (id, crate::PatternSlot::Fill),
        crate::PatternTile {
            image: tile(),
            cells: [1, 1],
            tile_px: [1, 1],
            quality: ImageQuality::Medium,
        },
    );
    let c = run(&pat, &tiles);
    assert!(
        c.inner().encoding().draw_tags != b.inner().encoding().draw_tags,
        "o ladrilho nao substituiu a cor de recurso"
    );
    assert_eq!(c.inner().encoding().n_clips, 0);
    assert_eq!(c.inner().encoding().n_paths, 1);
}

/// ⚠️⚠️ **UMA INSTÂNCIA DE MOTION DE UMA FORMA COM PADRÃO PINTA A `fallback`, e é DECLARADO.**
///
/// A rota de instância é alimentada pelo cozimento do Motion — outro oleoduto, que não tem o mapa
/// de ladrilhos do quadro em mãos. ⛔ Não é *"não deu"*: é a fronteira desta wave, e este gate
/// prende-a para que o dia em que alguém a mudar seja um acto deliberado e não um efeito colateral.
#[test]
fn a_motion_instance_of_a_patterned_shape_paints_the_fallback() {
    use ph2d_vec_scene::{Paint, PatternFill, PatternSource, Rgba8, VecPath, VecVertex};
    let cor = Rgba8::new(200, 30, 30, 255);
    let mk = |fill: Paint| VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(fill),
        ..VecPath::default()
    };
    let draw = |p: &VecPath| {
        let mut s = VectorScene::new();
        crate::standalone::draw_path_standalone(p, Affine::IDENTITY, &mut s);
        s
    };
    let pat = draw(&mk(Paint::Pattern(Box::new(PatternFill::new(
        PatternSource::Shape(1),
        [4.0, 4.0],
        cor,
    )))));
    let solid = draw(&mk(Paint::solid(cor)));
    assert_eq!(
        pat.inner().encoding().draw_data,
        solid.inner().encoding().draw_data,
        "a rota de instancia deixou de pintar a fallback - se foi de proposito, actualize esta lei"
    );
}

/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"pattern anula stroke"*.**
///
/// Uma forma com padrão E traço tem de encodar **os dois** — o preenchimento e o contorno. Se o
/// padrão comesse o traço, a única saída do artista seria desenhar o contorno como segunda forma.
#[test]
fn a_patterned_shape_still_draws_its_stroke() {
    use ph2d_vec_scene::{
        Paint, PatternFill, PatternSource, Rgba8, StrokeSpec, VecPath, VecVertex,
    };
    let verts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec();
    let stroke = Some(StrokeSpec::new(Rgba8::new(10, 10, 10, 255), 0.5));
    let draw = |fill: Paint, tile: Option<&crate::PatternTile>| {
        let path = VecPath {
            verts: verts.clone(),
            closed: true,
            fill: Some(fill),
            stroke: stroke.clone(),
            ..VecPath::default()
        };
        let mut s = VectorScene::new();
        crate::draw_path_tiled(&path, Affine::IDENTITY, &mut s, tile, None, None);
        s.inner().encoding().n_paths
    };
    let cor = Rgba8::new(200, 30, 30, 255);
    // CONTROLO: um sólido com traço encoda DOIS caminhos (preenchimento + contorno).
    let solid = draw(Paint::solid(cor), None);
    assert_eq!(
        solid, 2,
        "o controlo mudou: um solido com traco nao encoda 2"
    );
    let pat = || {
        Paint::Pattern(Box::new(PatternFill::new(
            PatternSource::Shape(1),
            [4.0, 4.0],
            cor,
        )))
    };
    assert_eq!(draw(pat(), None), 2, "sem ladrilho, o traco sumiu");
    let t = crate::PatternTile {
        image: tile(),
        cells: [1, 1],
        tile_px: [1, 1],
        quality: ImageQuality::Medium,
    };
    assert_eq!(draw(pat(), Some(&t)), 2, "COM ladrilho, o traco sumiu");
}

/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"filters anula pattern"*.**
///
/// No [`crate::dispatch`] a imagem de FX **toma o lugar** do desenho da forma. A rasterização
/// isolada que a produz chamava a `draw_path`, que passa `None` de ladrilho — então uma forma com
/// padrão era rasterizada com a **cor de recurso**, e ligar um filtro apagava o padrão.
///
/// ⚠️ O doc-comment da própria função já declarava a lei que isso partia: *"passa pela MESMA
/// `draw_path` do `dispatch` — desenhar por uma 2ª porta faria o FX divergir do que a forma parece
/// de verdade"*. A segunda porta apareceu **dentro** da primeira, quando o ladrilho virou um
/// parâmetro que só o `dispatch` sabia preencher.
#[test]
fn the_isolated_rasterisation_honours_the_pattern_tile() {
    use crate::{VecViewState, VecXforms};
    use ph2d_vec_scene::{Paint, PatternFill, PatternSource, Rgba8, VecPath, VecScene, VecVertex};
    let _ = VecViewState::default();
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(PatternFill::new(
            PatternSource::Shape(1),
            [4.0, 4.0],
            Rgba8::new(200, 30, 30, 255),
        )))),
        ..VecPath::default()
    });
    let run = |tiles: &crate::PatternTiles| {
        let mut s = VectorScene::new();
        crate::standalone::draw_path_isolated(
            &scene,
            &VecXforms::new(),
            &crate::LiveGeometry::new(),
            tiles,
            &crate::BrushArts::new(),
            id,
            Affine::IDENTITY,
            Affine::IDENTITY,
            &mut s,
        );
        s.inner().encoding().draw_data.clone()
    };
    let sem = run(&crate::PatternTiles::new());
    let mut tiles = crate::PatternTiles::new();
    tiles.insert(
        (id, crate::PatternSlot::Fill),
        crate::PatternTile {
            image: tile(),
            cells: [1, 1],
            tile_px: [1, 1],
            quality: ImageQuality::Medium,
        },
    );
    assert_ne!(
        sem,
        run(&tiles),
        "a rasterizacao isolada ignorou o ladrilho - um filtro apagaria o padrao"
    );
}
