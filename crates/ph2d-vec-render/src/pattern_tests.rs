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
