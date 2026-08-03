//! Gates da porta de entrada, **sem janela**.
//!
//! O que roda aqui é a [`place`] — a metade que decide *onde cada peça de um
//! arquivo vai parar e com que origem local*. Montar as peças na cena exige um
//! `wgpu::Device`, e nenhuma das decisões que podem estar erradas tem a ver com
//! um device.

use super::*;
use ph2d_mesh::{Face, Mesh, import_obj};

/// Um triângulo com o canto inferior-esquerdo em `(x, y)` e lado `s`.
fn tri_at(x: f32, y: f32, s: f32) -> ImportedPiece {
    ImportedPiece {
        name: None,
        mesh: Mesh::from_parts(
            vec![[x, y, 0.0], [x + s, y, 0.0], [x, y + s, 0.0]],
            vec![Face::tri(0, 1, 2)],
        )
        .expect("malha"),
    }
}

/// **A origem local de cada peça vai para o centro dela** — e o oráculo é o
/// PLANO DO ESPELHO, que é o mecanismo que exige isto.
///
/// ⚠️ Sem este passo o gesto de simetria reflete em torno de um plano que não
/// passa pelo modelo: a cópia espelhada de uma peça modelada a dez unidades do
/// zero nasce a vinte unidades da original.
#[test]
fn every_piece_gets_its_local_origin_at_its_own_centre() {
    let mut pieces = vec![tri_at(10.0, 0.0, 2.0), tri_at(-30.0, 5.0, 4.0)];
    let _ = place(&mut pieces, [0.0, 0.0, 0.0]);

    for p in &pieces {
        let c = p.mesh.bounds().center();
        assert!(
            c[0].abs() < 1e-5 && c[1].abs() < 1e-5 && c[2].abs() < 1e-5,
            "a origem local tem de ficar no centro, e ficou em {c:?}"
        );
    }
}

/// **O ARRANJO do arquivo sobrevive** — quem estava à esquerda continua à
/// esquerda, e a distância entre as peças escala com elas.
///
/// ⚠️ É a metade que impede a cura de virar outro defeito: centrar cada peça na
/// própria origem, sem devolver o deslocamento à `Pose`, EMPILHARIA a cabeça
/// sobre o corpo — as duas com a origem no zero, no mesmo lugar.
#[test]
fn the_files_arrangement_survives_in_the_poses() {
    let mut pieces = vec![tri_at(0.0, 0.0, 1.0), tri_at(10.0, 0.0, 1.0)];
    let poses = place(&mut pieces, [0.0, 0.0, 0.0]);

    assert!(
        poses[0].translation[0] < poses[1].translation[0],
        "quem estava à esquerda continua à esquerda: {:?} contra {:?}",
        poses[0].translation,
        poses[1].translation
    );
    // A separação no MUNDO é a do arquivo vezes a escala — e é isso que faz o
    // arranjo ser o mesmo desenho, e não dois pontos quaisquer em ordem certa.
    let gap = poses[1].translation[0] - poses[0].translation[0];
    assert!(
        (gap - 10.0 * poses[0].scale()).abs() < 1e-4,
        "a separação tem de escalar com as peças: {gap}"
    );
}

/// **Os tamanhos RELATIVOS sobrevivem** — um fator por ARQUIVO, não por peça.
///
/// ⚠️ Normalizar cada peça sozinha faria um olho chegar do tamanho de um corpo,
/// que é a forma de "importou e ficou irreconhecível".
#[test]
fn one_scale_for_the_whole_file_keeps_the_pieces_relative_sizes() {
    let mut pieces = vec![tri_at(0.0, 0.0, 1.0), tri_at(20.0, 0.0, 10.0)];
    let poses = place(&mut pieces, [0.0, 0.0, 0.0]);

    assert!(
        (poses[0].scale() - poses[1].scale()).abs() < 1e-6,
        "as duas peças têm de chegar na MESMA escala"
    );
    // E a peça grande continua dez vezes a pequena, no mundo.
    let big = pieces[1].mesh.bounds().longest_edge() * poses[1].scale();
    let small = pieces[0].mesh.bounds().longest_edge() * poses[0].scale();
    assert!(
        (big / small - 10.0).abs() < 1e-3,
        "a razão de tamanhos tem de sobreviver: {}",
        big / small
    );
}

/// **O arquivo inteiro chega do tamanho da mesa** — é a razão de existir da
/// escala, e ela é sobre CONVIVÊNCIA: desde a W8.1 a cena é uma lista, e uma
/// peça de 300 unidades ao lado de uma esfera de 1 torna a segunda invisível.
#[test]
fn a_huge_file_arrives_at_the_size_of_what_is_already_on_the_table() {
    let mut pieces = vec![tri_at(0.0, 0.0, 300.0)];
    let poses = place(&mut pieces, [0.0, 0.0, 0.0]);
    let world = pieces[0].mesh.bounds().longest_edge() * poses[0].scale();
    assert!(
        (world - IMPORT_SPAN).abs() < 1e-3,
        "o maior eixo tem de chegar em {IMPORT_SPAN}, e chegou em {world}"
    );
}

/// **Um arquivo degenerado não vira uma escala infinita.**
///
/// ⚠️ Sem o guard, `IMPORT_SPAN / 0` é `inf`, a `Pose::new` o recusa e clampa no
/// piso — e o objeto SOME sem nada dizendo por quê. Falhar alto seria melhor;
/// não falhar é melhor ainda.
#[test]
fn a_degenerate_file_does_not_produce_an_infinite_scale() {
    let mut pieces = vec![ImportedPiece {
        name: None,
        mesh: Mesh::from_parts(vec![[7.0; 3]; 3], vec![Face::tri(0, 1, 2)]).expect("malha"),
    }];
    let poses = place(&mut pieces, [1.0, 2.0, 3.0]);
    assert!(poses[0].scale().is_finite() && poses[0].scale() > 0.0);
    assert_eq!(poses[0].translation, [1.0, 2.0, 3.0], "ela pousa na âncora");
}

/// **O arquivo pousa na ÂNCORA** — ao lado do que já está na mesa, e não por
/// cima.
#[test]
fn the_file_lands_centred_on_the_anchor() {
    let mut pieces = vec![tri_at(0.0, 0.0, 1.0), tri_at(4.0, 0.0, 1.0)];
    let anchor = [9.0, -1.0, 0.5];
    let poses = place(&mut pieces, anchor);
    let mid: Vec<f32> = (0..3)
        .map(|k| (poses[0].translation[k] + poses[1].translation[k]) * 0.5)
        .collect();
    for k in 0..3 {
        assert!(
            (mid[k] - anchor[k]).abs() < 1e-4,
            "o centro do arquivo tem de cair na âncora: eixo {k} deu {}",
            mid[k]
        );
    }
}

/// **Um `.obj` é reconhecido; uma imagem não** — e o gate cobre as duas metades
/// porque a de ausência é a que decide o roteamento do drop.
#[test]
fn only_mesh_extensions_are_claimed() {
    for yes in ["a.obj", "A.OBJ", "/tmp/x/y.Obj"] {
        assert!(is_mesh_file(std::path::Path::new(yes)), "{yes}");
    }
    for no in ["a.png", "a.obj.png", "a", "obj"] {
        assert!(!is_mesh_file(std::path::Path::new(no)), "{no}");
    }
}

/// **A porta é a do arquivo REAL** — um OBJ multi-objeto atravessa o leitor e a
/// colocação, e sai como peças utilizáveis.
///
/// ⚠️ Este é o gate de ponta a ponta da wave: os dois de cima medem `place` com
/// malhas fabricadas, e é aqui que o `import_obj` novo e a colocação se
/// encontram — que é onde uma discordância entre eles apareceria.
#[test]
fn a_two_object_file_arrives_as_two_usable_pieces() {
    let src = "o cabeca\n\
               v 100 0 0\nv 102 0 0\nv 100 2 0\nf 1 2 3\n\
               o corpo\n\
               v 100 -9 0\nv 106 -9 0\nv 100 -3 0\nf 4 5 6\n";
    let mut pieces = import_obj(src).expect("carrega");
    assert_eq!(pieces.len(), 2);

    let poses = place(&mut pieces, [0.0, 0.0, 0.0]);
    for (p, pose) in pieces.iter().zip(&poses) {
        let c = p.mesh.bounds().center();
        assert!(c[0].abs() < 1e-4 && c[1].abs() < 1e-4, "centrada: {c:?}");
        assert!(pose.scale().is_finite() && pose.scale() > 0.0);
        assert!(pose.translation.iter().all(|v| v.is_finite()));
    }
    assert!(
        poses[0].translation[1] > poses[1].translation[1],
        "a cabeça estava ACIMA do corpo no arquivo, e continua"
    );
}
