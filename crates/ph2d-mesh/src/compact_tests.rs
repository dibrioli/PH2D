//! Os gates da porta da compactação.

use super::compact_for_faces;
use crate::{Face, shapes};

/// ⭐ **A metade recortada não leva os vértices da outra.**
///
/// ⚠️ **A régua é a CONTAGEM, não «a malha constrói-se»** — sem compactar ela
/// constrói-se igualmente: nenhum índice fica fora de alcance, e é isso que
/// torna o defeito mudo.
#[test]
fn a_metade_recortada_nao_leva_os_vertices_da_outra() {
    let cheia = shapes::uv_sphere(32, 48, 1.0);
    let pos = cheia.positions().to_vec();
    let escolhidas: Vec<Face> = cheia
        .faces()
        .iter()
        .filter(|f| f.verts().iter().all(|&v| pos[v as usize][1] <= 0.0))
        .copied()
        .collect();
    let (novas, faces, origem) = compact_for_faces(&pos, &escolhidas);

    assert_eq!(faces.len(), escolhidas.len(), "nenhuma face se perde");
    // ⚠️ **Os dois números são MEDIDOS, e o que importa é a DIFERENÇA** — sem
    // a porta a malha nasce com `1 490` posições para `768` faces, e os `721`
    // que sobram são a nuvem em que a busca de âncora da cena `=42` aterrava.
    assert_eq!((pos.len(), novas.len()), (1490, 769));

    // ⚠️ Toda posição nova é a posição de onde ela diz ter vindo — é isto que
    // deixa um canal paralelo (cor, máscara) seguir pela terceira saída.
    assert_eq!(origem.len(), novas.len());
    for (nova, &o) in novas.iter().zip(&origem) {
        assert_eq!(*nova, pos[o as usize]);
    }
    // E toda posição guardada é citada por alguma face.
    let mut citado = vec![false; novas.len()];
    for f in &faces {
        for &v in f.verts() {
            citado[v as usize] = true;
        }
    }
    assert!(citado.iter().all(|c| *c), "nenhum órfão sobrevive");
}

/// ⛔ **Um quad continua quad e um triângulo continua triângulo** — a sentinela
/// do 4.º slot não é um índice.
#[test]
fn a_sentinela_do_triangulo_nao_e_remapeada() {
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        // ⚠️ o órfão, de propósito: ele é quem a porta tem de deixar cair
        [9.0, 9.0, 9.0],
    ];
    let faces = vec![Face::quad(0, 1, 2, 3), Face::tri(1, 4, 2)];
    let (novas, saida, _) = compact_for_faces(&pos, &faces);

    assert_eq!(novas.len(), 5, "o órfão fica de fora");
    assert!(saida[0].vert_count() == 4 && saida[1].is_tri());
    assert_eq!(saida[1].0[3], crate::face::TRI);
    // A ordem é a do primeiro uso: 0,1,2,3 e depois o 4.
    assert_eq!(saida[0].verts(), [0, 1, 2, 3]);
    assert_eq!(saida[1].verts(), [1, 4, 2]);
}
