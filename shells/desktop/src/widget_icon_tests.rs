//! Gates da **porta única do glifo** (plano UI/UX W8b — o `IconButton`).

use super::*;
use ph2d_vec_scene::{VecPath, rectangle, star};

/// Uma estrela deitada — larga e baixa de propósito, para a lei uniforme ter o que provar.
fn wide_star() -> VecPath {
    star([3.0, -7.0], 4.0, 1.0, 5, 0.45)
}

/// **O lado maior encosta na viewbox, e a figura fica CENTRADA nela.**
///
/// ⚠️ O `24` não é um número desta wave: é a viewbox que o `paint_icon_path` do catálogo divide
/// para encaixar o glifo no botão. Normalizar noutra escala desenharia o ícone do tamanho errado
/// dentro da moldura — e o defeito seria *o ícone parece pequeno*, que ninguém atribui a uma
/// constante.
#[test]
fn the_glyph_fills_the_icon_viewbox_and_is_centred() {
    let bb = icon_face(&wide_star())
        .expect("a estrela tem figura")
        .bounding_box();
    let (w, h) = (bb.width(), bb.height());
    assert!(
        (w.max(h) - 24.0).abs() < 1e-9,
        "o lado maior nao encostou na viewbox: {w} x {h}"
    );
    // Centrada: a folga do lado menor é igual dos dois lados.
    assert!(
        (bb.x0 + bb.x1 - 24.0).abs() < 1e-9,
        "descentrada em x: {bb:?}"
    );
    assert!(
        (bb.y0 + bb.y1 - 24.0).abs() < 1e-9,
        "descentrada em y: {bb:?}"
    );
}

/// **A ESCALA É UNIFORME** — um glifo é uma figura, e esticá-la para encher a caixa mudaria o
/// desenho que o artista fez.
///
/// ⚠️ Este gate é o que separa *normalizar* de *deformar*, e a fixture tem de ser larga: numa
/// figura quadrada as duas leis dão o mesmo resultado, e o gate ficaria verde por vácuo.
#[test]
fn the_normalisation_is_uniform_so_the_drawing_keeps_its_shape() {
    let src = wide_star();
    let before = ph2d_vec_render::build_bezpath(&src.cooked()).bounding_box();
    let after = icon_face(&src)
        .expect("a estrela tem figura")
        .bounding_box();
    let (ra, rb) = (
        after.width() / after.height(),
        before.width() / before.height(),
    );
    assert!(
        (ra - rb).abs() < 1e-9,
        "a razao de aspecto mudou: {rb} -> {ra} (o glifo foi esticado)"
    );
}

/// **Sem figura, sem glifo** — e o botão desenha a moldura, que é o neutro.
#[test]
fn a_path_without_a_figure_has_no_face() {
    assert!(
        icon_face(&VecPath::default()).is_none(),
        "um caminho vazio inventou um glifo"
    );
    assert!(
        icon_face(&rectangle([1.0, 1.0], [1.0, 1.0])).is_none(),
        "um ponto (bbox de lado zero nos DOIS eixos) inventou um glifo"
    );
}

/// **Uma RETA tem glifo**, e a distinção é o ponto: ela é degenerada num eixo só.
///
/// ⚠️ Recusá-la seria confundir *não há figura* com *a figura é fina*: o lado maior dá a escala, o
/// menor centra-se, e o desenho é uma barra — que é exactamente o que o artista desenhou.
#[test]
fn a_straight_line_still_has_a_face() {
    let bb = icon_face(&rectangle([0.0, 5.0], [8.0, 5.0]))
        .expect("uma reta e' uma figura")
        .bounding_box();
    assert!(
        (bb.width() - 24.0).abs() < 1e-9,
        "a reta nao escalou: {bb:?}"
    );
    assert!(bb.height() < 1e-9, "a reta ganhou altura: {bb:?}");
}

/// **O CANVAS E O PAINEL LEEM O MESMO GLIFO** — o gate que carrega a fatia.
///
/// As duas metades precisam do glifo por motivos diferentes (o canvas pinta uma curva, o codegen
/// escreve texto), e é exactamente aí que nasce a divergência que só uma screenshot revela. Elas
/// percorrem [`icon_face`]; o que muda depois é só a REPRESENTAÇÃO, e este gate afirma que a
/// viagem por ela é sem perda.
///
/// ⚠️ Ele afirma **igualdade de curva**, não semelhança: o `Display` de `f64` do Rust escreve a
/// string mais curta que re-lê ao bit, então um `to_svg`/`from_svg` que perdesse precisão faria
/// isto sangrar em vez de desenhar um ícone quase-certo que ninguém compara.
#[test]
fn the_glyph_survives_the_trip_through_text() {
    let face = icon_face(&wide_star()).expect("a estrela tem figura");
    let back = BezPath::from_svg(&face.to_svg()).expect("o SVG que nos escrevemos re-le");
    assert_eq!(
        face.to_svg(),
        back.to_svg(),
        "o glifo mudou na viagem por texto — o painel desenharia outro icone"
    );
    assert_eq!(face.elements().len(), back.elements().len());
}
