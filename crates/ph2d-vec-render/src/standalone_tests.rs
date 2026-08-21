//! **O desenho AVULSO de um caminho** — a porta que o bake de tile usa (Enio,
//! 2026-08-20: *"Glow não funciona com shape"* → *"tudo deve brilhar"* → *"não
//! funcionou"*).
//!
//! # O oráculo é o que foi ENCODADO
//!
//! `VectorScene::inner().encoding().n_paths` conta os caminhos que de facto
//! entraram na cena do Vello — a aparência, medida. Um gate que perguntasse
//! `path.fill.is_some()` estaria a testar o `if` que eu acabei de escrever.

use crate::{draw_path, draw_path_standalone};
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, VecPath, cook};
use ph2d_vector::{Affine, VectorScene};

/// Um pentágono NU: sem fill e sem stroke autorados — exactamente o que um
/// `source.shape` recém-largado no grafo produz.
fn bare_primitive() -> VecPath {
    let p = cook(ShapeKind::Polygon, [-1.0, -1.0], [1.0, 1.0], &[5.0]);
    assert!(p.fill.is_none() && p.stroke.is_none(), "a fixture é NUA");
    p
}

fn encoded_standalone(path: &VecPath) -> u32 {
    let mut scene = VectorScene::new();
    draw_path_standalone(path, Affine::IDENTITY, &mut scene);
    scene.inner().encoding().n_paths
}

fn encoded_document_route(path: &VecPath) -> u32 {
    let mut scene = VectorScene::new();
    draw_path(path, Affine::IDENTITY, &mut scene);
    scene.inner().encoding().n_paths
}

/// **UM PRIMITIVO NU DESENHA A SILHUETA** — o bug, ao contrário.
///
/// ⚠️ **Este gate nasceu de um "não funcionou" do Enio.** O bake de tile do
/// `source.shape` chamava a porta do DOCUMENTO ([`draw_path`]), que não desenha um
/// caminho sem fill nem stroke. O tile saía **totalmente transparente**, o quad do
/// glow desenhava nada, e o modo de falha era mudo: o `PH2D_GLOW_DIAG` mostrava a
/// camada CERTA (`tile_forma=1`, `camada=120`) e a tela não tinha halo nenhum.
#[test]
fn a_bare_primitive_encodes_its_silhouette() {
    assert_eq!(
        encoded_standalone(&bare_primitive()),
        1,
        "uma forma nua tem de encodar a silhueta — senão o tile sai vazio"
    );
}

/// **E É ISTO QUE AS DUAS PORTAS DISCORDAM** — o controle que dá sentido ao de cima.
///
/// ⚠️ Sem esta metade o gate acima ficaria verde num mundo em que as duas portas
/// fossem a mesma, e não diria por que a escolha da porta importa. A porta do
/// documento é a certa para um caminho do documento (que sempre tem paint); a de
/// instância é a certa para uma forma paramétrica.
#[test]
fn the_document_route_draws_nothing_for_the_same_bare_primitive() {
    assert_eq!(
        encoded_document_route(&bare_primitive()),
        0,
        "é esta a diferença que custou um smoke reprovado"
    );
}

/// **UM CAMINHO COM PAINT ENCODA IGUAL PELAS DUAS PORTAS** — a mudança não mexeu no
/// que já funcionava (os `source.object`, que trazem fill/stroke do documento).
#[test]
fn a_painted_path_encodes_the_same_through_either_door() {
    let mut p = bare_primitive();
    p.stroke = Some(StrokeSpec::new(Rgba8::new(255, 0, 0, 255), 0.1));
    assert_eq!(encoded_standalone(&p), encoded_document_route(&p));
    assert!(encoded_standalone(&p) >= 1, "e desenha alguma coisa");
}

/// **A CAIXA DE UM PRIMITIVO NU NÃO É VAZIA** — o irmão da lei acima, do lado dos
/// LIMITES: um bbox `None` faria o bake desistir antes de rasterizar, e o sintoma
/// seria o mesmo halo ausente por outro caminho.
#[test]
fn a_bare_primitive_has_measurable_bounds() {
    let b = crate::standalone_path_screen_bounds(&bare_primitive(), Affine::IDENTITY);
    let (x0, y0, x1, y1) = b.expect("uma forma nua tem caixa");
    assert!(x1 > x0 && y1 > y0, "e ela tem área: {x0},{y0} → {x1},{y1}");
}
