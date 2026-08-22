//! **A rota de INSTÂNCIA de Motion** — o que um `source.shape` de facto encoda.
//!
//! # O oráculo é o que foi ENCODADO
//!
//! `VectorScene::inner().encoding()` conta os caminhos que entraram na cena do Vello e
//! guarda os bytes de tinta (`draw_data`). Um gate que perguntasse `path.fill.is_some()`
//! estaria a testar o `if` que acabei de escrever; estes perguntam ao Vello.

use crate::draw_shape_instance;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_trim::TrimSpec;
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPath, cook};
use ph2d_vector::{Affine, VectorScene};

/// Um pentágono NU — o que um `source.shape` recém-largado no grafo produz: sem fill e
/// sem stroke autorados, porque a cor dele vem do `tint` da INSTÂNCIA.
fn bare() -> VecPath {
    let p = cook(ShapeKind::Polygon, [-1.0, -1.0], [1.0, 1.0], &[5.0]);
    assert!(p.fill.is_none() && p.stroke.is_none(), "a fixture e' NUA");
    p
}

fn stroked() -> VecPath {
    let mut p = bare();
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 0, 0, 255), 0.1));
    p
}

fn drawn(path: &VecPath, tint: [f32; 4]) -> (u32, Vec<u32>) {
    let mut scene = VectorScene::new();
    draw_shape_instance(path, Affine::IDENTITY, tint, &mut scene);
    let e = scene.inner().encoding();
    (e.n_paths, e.draw_data.clone())
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

/// **CONTROLE — uma forma nua desenha a silhueta com o `tint` da instância.** Sem esta
/// metade os gates abaixo ficariam verdes num mundo em que nada é desenhado.
#[test]
fn a_bare_primitive_fills_with_the_instance_tint() {
    let (n, white) = drawn(&bare(), WHITE);
    assert_eq!(n, 1, "a silhueta");
    let (_, green) = drawn(&bare(), GREEN);
    assert_ne!(white, green, "e o `tint` da instancia e' o que a pinta");
}

/// **O TRAÇO NÃO APAGA O PREENCHIMENTO** — o defeito, ao contrário.
///
/// ⚠️ **Medido em 2026-08-21, ao ligar o TRIM:** pôr `stroke_width > 0` num `source.shape`
/// mandava o caminho pela rota do DOCUMENTO (`path_tess` + `draw_path_with`), que só
/// preenche quando `path.fill.is_some()` — e um primitivo tem `fill: None` de propósito,
/// porque a cor dele é o `tint` da instância. Resultado: a forma ficava **oca** no instante
/// em que o artista mexia na largura do traço, e o `motion.tint` a jusante deixava de ter
/// efeito. O doc-comment do próprio `ShapeParams::stroke` já dizia o contrário (*"o
/// preenchimento de uma forma vem do `tint` da instância … é o controle que separa forma de
/// silhueta"*), então isto era a intenção escrita contra o desenho.
#[test]
fn a_stroke_does_not_erase_the_instance_fill() {
    let (n, _) = drawn(&stroked(), WHITE);
    assert_eq!(n, 2, "preenchimento (tint) + traco (cor propria)");
}

/// **E o `tint` continua vivo com traço** — a metade que prova que o segundo caminho é o
/// PREENCHIMENTO, e não dois traços.
#[test]
fn the_instance_tint_survives_a_stroke() {
    let (_, white) = drawn(&stroked(), WHITE);
    let (_, green) = drawn(&stroked(), GREEN);
    assert_ne!(white, green, "trocar o tint tem de mudar o que se encoda");
}

/// **UM CONTORNO ABERTO NÃO É PREENCHIDO** — a lei que faz o TRIM funcionar sem um `if`
/// dedicado a ele.
///
/// Um trim revela um trecho, e um trecho de contorno **não tem interior**: fechá-lo
/// implicitamente desenharia a corda. O `build_contours(_, Some(true))` já descarta os
/// contornos abertos, então a forma aparada desenha só o traço — sem que uma linha de
/// código pergunte *"isto foi aparado?"*.
#[test]
fn a_trimmed_contour_draws_only_the_stroke() {
    let mut p = stroked();
    p.effects.push(FxEntry::new(PathEffect::Trim(TrimSpec {
        start: 0.0,
        end: 0.4,
        offset: 0.0,
    })));
    let (n, _) = drawn(&p, WHITE);
    assert_eq!(n, 1, "so' o traco: um trecho aberto nao tem interior");
}

/// **E o TRIM NEUTRO não move um bit** — o invariante de que a pilha inteira depende
/// (ADR-0132): um efeito no ponto neutro é saltado, então o default do nó é a forma que
/// sempre shipou.
#[test]
fn a_neutral_trim_encodes_exactly_what_no_trim_encodes() {
    let mut p = stroked();
    p.effects
        .push(FxEntry::new(PathEffect::Trim(TrimSpec::default())));
    assert_eq!(drawn(&p, WHITE), drawn(&stroked(), WHITE));
}
