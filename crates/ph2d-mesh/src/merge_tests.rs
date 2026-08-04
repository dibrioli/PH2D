//! Os gates da [`super::merge`].
//!
//! ⚠️ **As fixtures são malhas escritas à mão, e não primitivas**, porque o que
//! está sob teste é ARITMÉTICA DE ÍNDICE: com uma esfera de 600 vértices um
//! deslocamento errado ainda produz índices válidos, e o oráculo teria de ser um
//! desenho. Com dois triângulos de vértices distinguíveis, ele é uma igualdade.

use super::merge;
use crate::face::Face;
use crate::mesh::Mesh;
use crate::pose::Pose;

/// Um triângulo cujos vértices ficam num `x` conhecido — assim dá para dizer
/// **de que peça** um vértice do resultado veio.
fn tri_at(x: f32) -> Mesh {
    Mesh::from_parts(
        vec![[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("fixture")
}

fn quad_at(x: f32) -> Mesh {
    Mesh::from_parts(
        vec![
            [x, 0.0, 0.0],
            [x + 1.0, 0.0, 0.0],
            [x + 1.0, 1.0, 0.0],
            [x, 1.0, 0.0],
        ],
        vec![Face::quad(0, 1, 2, 3)],
    )
    .expect("fixture")
}

/// **CADA FACE APONTA PARA A PRÓPRIA PEÇA.**
///
/// ⚠️ É o gate central, e o defeito que ele pega é mudo: um deslocamento
/// esquecido faz a face da segunda peça referenciar os vértices da PRIMEIRA —
/// índices válidos, malha que carrega, e uma superfície que atravessa a cena.
#[test]
fn every_face_points_into_its_own_piece() {
    let a = tri_at(0.0);
    let b = tri_at(10.0);
    let out = merge(&[(&a, Pose::IDENTITY), (&b, Pose::IDENTITY)]).expect("funde");

    assert_eq!(out.vert_count(), 6, "os vértices das duas");
    assert_eq!(out.face_count(), 2, "as faces das duas");
    assert_eq!(
        out.faces()[0].verts(),
        &[0, 1, 2],
        "a primeira fica onde está"
    );
    assert_eq!(out.faces()[1].verts(), &[3, 4, 5], "a segunda é DESLOCADA");
    // E o deslocamento aponta para a geometria certa: o vértice 3 é o começo da
    // segunda peça, que nasceu em x = 10.
    assert!(
        (out.positions()[3][0] - 10.0).abs() < 1e-6,
        "o vértice 3 é da segunda peça, não da primeira"
    );
}

/// **A POSE É ASSADA, e sem isso a fusão empilha tudo na origem.**
///
/// O defeito espelho do que o `export` já documenta: a posição de uma peça vive
/// na [`Pose`] desde a W8.1, então concatenar geometria LOCAL junta duas peças
/// que o artista pôs a dez unidades de distância no mesmo lugar.
#[test]
fn the_pose_is_baked_into_the_vertices() {
    let a = tri_at(0.0);
    let b = tri_at(0.0);
    let out =
        merge(&[(&a, Pose::IDENTITY), (&b, Pose::new([10.0, 0.0, 0.0], 2.0))]).expect("funde");

    let bounds = out.bounds();
    assert!(
        (bounds.min[0] - 0.0).abs() < 1e-6,
        "a primeira peça fica onde estava"
    );
    // A segunda vai para x = 10 e mede o DOBRO: o vértice mais à direita dela é
    // `10 + 1*2`.
    assert!(
        (bounds.max[0] - 12.0).abs() < 1e-6,
        "a segunda foi para onde a pose a punha, no tamanho da pose -- x max {}",
        bounds.max[0]
    );
}

/// **UM TRIÂNGULO CONTINUA TRIÂNGULO.**
///
/// ⚠️ O quarto elemento de um `Face` é o sentinela [`crate::TRI`], não um
/// vértice. Somar o deslocamento nele produz um índice grande — e, numa fusão
/// grande o bastante, um índice **válido**: o triângulo vira um quad que aponta
/// para um vértice qualquer de outra peça.
#[test]
fn a_triangle_survives_the_offset_as_a_triangle() {
    let a = quad_at(0.0);
    let b = tri_at(10.0);
    let out = merge(&[(&a, Pose::IDENTITY), (&b, Pose::IDENTITY)]).expect("funde");

    assert!(!out.faces()[0].is_tri(), "o quad continua quad");
    assert!(out.faces()[1].is_tri(), "e o triângulo continua triângulo");
    assert_eq!(
        out.faces()[1].verts(),
        &[4, 5, 6],
        "deslocado pelos 4 do quad"
    );
}

/// **A MÁSCARA VIAJA, e quem não tinha fica no default.**
///
/// ⚠️ Uma máscara é o que o artista pintou para PROTEGER. Perdê-la numa fusão é
/// destruir autoria em silêncio: o gesto seguinte esculpe o que ele havia
/// blindado, e nada na tela diz por quê.
#[test]
fn the_mask_travels_and_the_unmasked_piece_lands_on_the_default() {
    let mut a = tri_at(0.0);
    a.masks_mut().fill(1.0);
    let b = tri_at(10.0);

    let out = merge(&[(&a, Pose::IDENTITY), (&b, Pose::IDENTITY)]).expect("funde");
    let masks = out.masks().expect("a fusão carrega o plano");
    assert_eq!(
        &masks[..3],
        &[1.0, 1.0, 1.0],
        "a peça mascarada chega inteira"
    );
    assert_eq!(
        &masks[3..],
        &[crate::mesh::DEFAULT_MASK; 3],
        "e a que não tinha plano fica ESCULPÍVEL, que é o que 'sem máscara' quer dizer"
    );
}

/// **DUAS MALHAS VIRGENS FUNDEM NUMA MALHA VIRGEM.**
///
/// ⚠️ O par do gate acima, e ele é o que impede a cura de virar custo: se a
/// fusão materializasse os planos sempre, toda cena blocada com primitivas
/// passaria a pagar 16 B/vértice por dois canais que ninguém pintou — e a
/// pergunta *"esta malha tem máscara?"* passaria a responder `sim` para todas.
#[test]
fn merging_virgin_meshes_leaves_the_result_virgin() {
    let a = tri_at(0.0);
    let b = tri_at(10.0);
    let out = merge(&[(&a, Pose::IDENTITY), (&b, Pose::IDENTITY)]).expect("funde");
    assert!(out.masks().is_none(), "nenhum plano de máscara nasceu");
    assert!(out.colors().is_none(), "nenhum plano de cor nasceu");
}

/// **A COR viaja pelo mesmo caminho** — o irmão do gate da máscara, e ele existe
/// porque os dois canais são planos diferentes: um `if` escrito para um só
/// deixaria o outro cair, com o gate do primeiro verde.
#[test]
fn the_colour_travels_too() {
    let a = tri_at(0.0);
    let mut b = tri_at(10.0);
    b.colors_mut().fill([0.25, 0.5, 0.75]);

    let out = merge(&[(&a, Pose::IDENTITY), (&b, Pose::IDENTITY)]).expect("funde");
    let colors = out.colors().expect("a fusão carrega o plano");
    assert_eq!(
        &colors[..3],
        &[crate::mesh::DEFAULT_COLOR; 3],
        "quem não pintou fica branco"
    );
    assert_eq!(&colors[3..], &[[0.25, 0.5, 0.75]; 3], "e a pintada chega");
}

/// **Uma peça só é assar a pose**, e nenhuma é a malha vazia — os dois totais de
/// propósito: a recusa é do chamador, que é quem sabe o que dizer ao artista.
#[test]
fn one_piece_is_a_pose_bake_and_none_is_the_empty_mesh() {
    let a = tri_at(0.0);
    let out = merge(&[(&a, Pose::new([5.0, 0.0, 0.0], 1.0))]).expect("funde");
    assert_eq!(out.vert_count(), 3);
    assert!(
        (out.bounds().min[0] - 5.0).abs() < 1e-6,
        "a pose foi assada"
    );

    let empty = merge(&[]).expect("funde");
    assert_eq!(empty.vert_count(), 0);
    assert_eq!(empty.face_count(), 0);
}
