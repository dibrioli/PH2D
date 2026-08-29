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

fn tile() -> StableImage {
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
        crate::draw_path_tiled(&path, Affine::IDENTITY, &mut s, tile, None);
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

// ── O PADRÃO NO TRAÇO — wave B (plano 35) ─────────────────────────────────────────────

use ph2d_vec_scene::{StrokePaint, StrokeSpec};

/// Uma forma **só com traço** (sem preenchimento), com a tinta que se pedir.
fn so_traco(paint: StrokePaint) -> ph2d_vec_scene::VecScene {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut s = StrokeSpec::new(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255), 1.0);
    s.paint = paint;
    scene.push_path(ph2d_vec_scene::VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(ph2d_vec_scene::VecVertex::corner)
            .to_vec(),
        closed: true,
        stroke: Some(s),
        ..ph2d_vec_scene::VecPath::default()
    });
    scene
}

fn pat_do_traco() -> StrokePaint {
    StrokePaint::Pattern(Box::new(ph2d_vec_scene::PatternFill::new(
        ph2d_vec_scene::PatternSource::Shape(1),
        [4.0, 4.0],
        ph2d_vec_scene::Rgba8::new(200, 30, 30, 255),
    )))
}

fn tile_de_teste() -> crate::PatternTile {
    crate::PatternTile {
        image: tile(),
        cells: [1, 1],
        tile_px: [1, 1],
        quality: ImageQuality::Medium,
    }
}

fn desenha(scene: &ph2d_vec_scene::VecScene, tiles: &crate::PatternTiles) -> VectorScene {
    let mut target = VectorScene::new();
    crate::dispatch(
        scene,
        &ph2d_vec_scene::VecViewState::default(),
        &ph2d_vec_scene::VecXforms::new(),
        &crate::LiveGeometry::new(),
        &crate::FxImages::new(),
        &crate::WidgetSkins::new(),
        tiles,
        Affine::IDENTITY,
        &mut target,
    );
    target
}

/// ⭐⭐ **UM TRAÇO COM PADRÃO DESENHA O LADRILHO** — e sem ladrilho pinta a `fallback`, byte a byte
/// como um traço sólido daquela cor.
///
/// As duas metades são desenho CERTO: a segunda é o que o artista vê enquanto a arte carrega.
/// ⛔ Desenhar NADA seria pior — uma linha invisível não se distingue de uma forma sem contorno.
#[test]
fn a_patterned_stroke_draws_the_tile_and_falls_back_to_the_colour() {
    let com_padrao = so_traco(pat_do_traco());
    let solido = so_traco(StrokePaint::Solid(ph2d_vec_scene::Rgba8::new(
        200, 30, 30, 255,
    )));
    let vazio = crate::PatternTiles::new();

    // 1. Sem ladrilho: BYTE-A-BYTE o encode de um traço sólido da cor de recurso.
    let a = desenha(&com_padrao, &vazio);
    let b = desenha(&solido, &vazio);
    assert!(a.inner().encoding().draw_tags == b.inner().encoding().draw_tags);
    assert_eq!(
        a.inner().encoding().draw_data,
        b.inner().encoding().draw_data
    );

    // 2. Com ladrilho: deixa de o ser.
    let mut tiles = crate::PatternTiles::new();
    let id = com_padrao.paths()[0].id;
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    let c = desenha(&com_padrao, &tiles);
    assert!(
        c.inner().encoding().draw_tags != b.inner().encoding().draw_tags,
        "com ladrilho o traco continuou a encodar um solido"
    );
}

/// ⚠️⚠️ **O ladrilho do TRAÇO não é o do PREENCHIMENTO** — a chave do mapa tem de os separar.
///
/// Uma chave só pela forma entregaria o ladrilho do preenchimento ao traço, e o desenho ficaria
/// certo **por acidente** enquanto os dois fossem iguais.
#[test]
fn the_fill_tile_is_not_handed_to_the_stroke() {
    let scene = so_traco(pat_do_traco());
    let id = scene.paths()[0].id;
    let vazio = crate::PatternTiles::new();
    let base = desenha(&scene, &vazio);

    // Um ladrilho no slot do PREENCHIMENTO não pode mudar o traço.
    let mut errado = crate::PatternTiles::new();
    errado.insert((id, crate::PatternSlot::Fill), tile_de_teste());
    let a = desenha(&scene, &errado);
    assert!(
        a.inner().encoding().draw_tags == base.inner().encoding().draw_tags,
        "o ladrilho do preenchimento vazou para o traco"
    );
    // CONTROLO: no slot certo, ele muda — senão este gate mediria um mapa que nunca é lido.
    let mut certo = crate::PatternTiles::new();
    certo.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    assert!(
        desenha(&scene, &certo).inner().encoding().draw_tags != base.inner().encoding().draw_tags
    );
}

/// ⭐ **O KILL-CRITERION do plano 35:** um traço com padrão custa o que um traço sólido custa —
/// **zero** camadas de clip.
///
/// ⚠️ O `n_clips` conta **duas** por camada (o `begin` e o `end`), e é por isso que a barra é a
/// IGUALDADE com o sólido, e não um número escrito à mão.
#[test]
fn a_patterned_stroke_pushes_no_clip_layer() {
    let scene = so_traco(pat_do_traco());
    let id = scene.paths()[0].id;
    let mut tiles = crate::PatternTiles::new();
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
    let com = desenha(&scene, &tiles);
    let solido = desenha(
        &so_traco(StrokePaint::Solid(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255))),
        &crate::PatternTiles::new(),
    );
    assert_eq!(
        com.inner().encoding().n_clips,
        solido.inner().encoding().n_clips,
        "o padrao no traco empurrou camada - o kill-criterion do plano 35 caiu"
    );
    assert_eq!(
        com.inner().encoding().n_paths,
        solido.inner().encoding().n_paths,
        "o padrao no traco custou mais um desenho que o solido"
    );
}

/// ⚠️⚠️ **O PADRÃO CAI NO MESMO SÍTIO nos dois caminhos do `stroke_uniform`** — e este gate existe
/// porque o segundo caminho é uma armadilha real.
///
/// O Vello compõe `transform * brush_transform`. No caminho rápido (afim conforme) a geometria é
/// local e o afim é o `transform` ⇒ a colocação local chega certa. No caminho não-conforme a
/// geometria **já foi levada à tela** e o afim que chega ao Vello é `IDENTITY` ⇒ sem pré-compor, o
/// padrão ficaria no espaço LOCAL sobre uma geometria de TELA: encolhido no canto do mundo.
///
/// ⭐ A régua é a IGUALDADE dos afins que chegam ao encoding — não uma imagem, não um relógio.
#[test]
fn the_stroke_pattern_lands_in_the_same_place_under_a_non_conformal_affine() {
    let scene = so_traco(pat_do_traco());
    let id = scene.paths()[0].id;
    let mut tiles = crate::PatternTiles::new();
    tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());

    let desenhar = |xf: Affine| {
        let mut target = VectorScene::new();
        crate::dispatch(
            &scene,
            &ph2d_vec_scene::VecViewState::default(),
            &ph2d_vec_scene::VecXforms::new(),
            &crate::LiveGeometry::new(),
            &crate::FxImages::new(),
            &crate::WidgetSkins::new(),
            &tiles,
            xf,
            &mut target,
        );
        target.inner().encoding().transforms.clone()
    };
    // ⚠️ A MESMA escala não-uniforme que parte a caneta (bug #27) — é ela que manda o traço pelo
    // caminho lento. O controlo é a versão uniforme do mesmo afim.
    let partido = Affine::scale_non_uniform(3.0, 1.0);
    let conforme = Affine::scale(3.0);
    assert!(
        !is_conformal(partido),
        "a fixtura deixou de conter o fenomeno: este afim ja' e' conforme"
    );
    assert!(is_conformal(conforme));

    // O afim do PINCEL que chega ao Vello tem de ser o mesmo nos dois caminhos, a menos do afim da
    // geometria — que é exactamente o que o caminho lento pré-compõe.
    let a = desenhar(partido);
    let b = desenhar(conforme);
    assert!(
        !a.is_empty() && !b.is_empty(),
        "nenhum dos dois encodou transform nenhum"
    );
    // ⭐ A afirmação forte: o caminho lento **não** deixa a colocação no espaço local. Se deixasse,
    // o afim do pincel seria idêntico ao do caso identidade — e não é.
    let identidade = desenhar(Affine::IDENTITY);
    assert!(
        a != identidade,
        "o caminho nao-conforme deixou a colocacao no espaco LOCAL - o padrao encolhe no canto"
    );
}

/// ⭐⭐ **O PADRÃO NÃO ESCALA COM A LARGURA DO TRAÇO** (gate nº 4 do plano 35 §4) — a queixa que o
/// Illustrator colhe há anos, do lado certo.
///
/// *A largura decide a **faixa**; o padrão decide **o que a preenche**.* São duas grandezas, e
/// juntá-las faria engrossar a linha mudar o motivo debaixo dela.
///
/// ⚠️ **A régua é o afim do PINCEL que chega ao encoding**, e não uma imagem: se a colocação
/// passasse a ler a largura, ele mudaria entre as duas corridas.
///
/// ⚠️⚠️ **O CONTROLE é a metade que importa** — as duas corridas têm de diferir em ALGUMA coisa,
/// senão este gate ficaria verde sobre um produto que ignora a largura por completo (e aí ele não
/// mediria nada).
#[test]
fn the_stroke_pattern_does_not_scale_with_the_stroke_width() {
    let desenhar = |w: f64| {
        let mut scene = ph2d_vec_scene::VecScene::default();
        let mut s = StrokeSpec::new(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255), w);
        s.paint = pat_do_traco();
        let id = scene.push_path(ph2d_vec_scene::VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                .map(ph2d_vec_scene::VecVertex::corner)
                .to_vec(),
            closed: true,
            stroke: Some(s),
            ..ph2d_vec_scene::VecPath::default()
        });
        let mut tiles = crate::PatternTiles::new();
        tiles.insert((id, crate::PatternSlot::Stroke), tile_de_teste());
        let alvo = desenha(&scene, &tiles);
        let e = alvo.inner().encoding();
        (e.transforms.clone(), e.styles.clone())
    };
    let (xf_fino, estilo_fino) = desenhar(0.5);
    let (xf_grosso, estilo_grosso) = desenhar(4.0);
    assert_eq!(
        xf_fino, xf_grosso,
        "o afim do PINCEL mudou com a largura - engrossar a linha mexe no motivo (a queixa do \
         Illustrator, do lado errado)"
    );
    // CONTROLE: a largura CHEGOU ao desenho. Sem esta metade, o gate acima ficaria verde sobre um
    // produto que nunca lê a largura — e aí ele não estaria a medir nada.
    assert_ne!(
        estilo_fino, estilo_grosso,
        "as duas larguras encodaram o MESMO estilo - a fixtura nao contem o fenomeno"
    );
}
