//! Gates da topologia do atlas.

use super::topo::{Elo, elos, faces_dos_triangulos, triangulos, vertices_dos_cantos};
use ph2d_mesh::{Face, Mesh};

/// Duas faces que partilham a aresta `(0, 1)`, com o plano escrito à mão por CANTO.
///
/// ```text
///        2                 cantos: face A = 0,1,2   face B = 1,0,3
///       / \
///      0---1
///       \ /
///        3
/// ```
fn duas_faces(plano_b: [[f32; 2]; 3]) -> (Mesh, Vec<[f32; 2]>) {
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, -1.0, 0.0],
    ];
    let mesh = Mesh::from_parts(pos, vec![Face::tri(0, 1, 2), Face::tri(1, 0, 3)])
        .expect("a fixtura e' valida");
    let mut plano = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
    plano.extend_from_slice(&plano_b);
    (mesh, plano)
}

/// ⭐⭐⭐ **Um elo só cola quando as DUAS pontas encostam.**
///
/// ⛔⛔ Esta lei nasceu de uma **mutação SOBREVIVENTE**: com a verificação de uma ponta
/// só, todos os gates do corte ficavam verdes, porque as fixturas deles deslocam a
/// costura inteira e nenhuma a deslocava **numa ponta só**. *Uma costura que casa numa
/// ponta e falha na outra é uma costura em LEQUE, e ela existe em toda peça onde o mapa
/// roda* — pô-la na mesma peça deixa um triângulo esticado a atravessar a ilha.
#[test]
fn um_elo_so_cola_quando_as_duas_pontas_encostam() {
    // (a) As duas pontas no mesmo sítio: colado.
    let (m, p) = duas_faces([[1.0, 0.0], [0.0, 0.0], [0.5, -1.0]]);
    let e = elos(&m, &p);
    assert_eq!(e.len(), 1, "as duas faces partilham uma aresta");
    assert_eq!(
        e[0].2,
        Elo {
            na_peca: true,
            no_atlas: true
        }
    );

    // (b) ⭐ UMA ponta fora do sítio — o vértice `1`, que é a ponta ALTA da chave. Com
    // uma verificação de uma ponta só, isto lê-se colado.
    let (m, p) = duas_faces([[1.0, 0.7], [0.0, 0.0], [0.5, -1.0]]);
    let e = elos(&m, &p);
    assert_eq!(
        e[0].2,
        Elo {
            na_peca: true,
            no_atlas: false
        },
        "uma ponta fora do sitio ja' e' um corte"
    );

    // (c) E a OUTRA ponta, porque a chave da aresta ordena os vértices e só um dos dois
    // lados é o «primeiro» — *sem este caso, metade da lei fica por medir*.
    let (m, p) = duas_faces([[1.0, 0.0], [0.0, 0.7], [0.5, -1.0]]);
    let e = elos(&m, &p);
    assert_eq!(
        e[0].2,
        Elo {
            na_peca: true,
            no_atlas: false
        }
    );
}

/// ⭐ **A tolerância é RELATIVA ao comprimento da aresta**, logo a mesma fixtura mil
/// vezes mais pequena dá a mesma resposta.
#[test]
fn a_tolerancia_do_elo_nao_depende_do_tamanho_da_peca() {
    let peq = |z: [f32; 2]| [z[0] * 1.0e-3, z[1] * 1.0e-3];
    let (m, p) = duas_faces([[1.0, 0.7], [0.0, 0.0], [0.5, -1.0]]);
    let pp: Vec<[f32; 2]> = p.iter().map(|&z| peq(z)).collect();
    assert!(
        !elos(&m, &pp)[0].2.no_atlas,
        "o corte continua um corte em ponto pequeno"
    );
    let (m2, p2) = duas_faces([[1.0, 0.0], [0.0, 0.0], [0.5, -1.0]]);
    let pp2: Vec<[f32; 2]> = p2.iter().map(|&z| peq(z)).collect();
    assert!(elos(&m2, &pp2)[0].2.no_atlas, "e a cola continua cola");
}

/// ⭐⭐ **O leque é UMA definição** — cantos, triângulos e faces contam a mesma história.
///
/// ⚠️ A fixtura tem um QUAD de propósito: com triângulos só, um leque errado é
/// indistinguível do certo.
#[test]
fn o_leque_e_a_mesma_definicao_para_os_tres_leitores() {
    let pos = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [2.0, 0.0, 0.0],
    ];
    let mesh = Mesh::from_parts(pos, vec![Face::quad(0, 1, 2, 3), Face::tri(1, 4, 2)])
        .expect("a fixtura e' valida");
    let (base, n) = super::bases_dos_cantos(&mesh);
    assert_eq!((base.as_slice(), n), ([0u32, 4].as_slice(), 7));
    let t = triangulos(&mesh);
    assert_eq!(
        t,
        vec![[0, 1, 2], [0, 2, 3], [4, 5, 6]],
        "dois do quad, um do tri"
    );
    assert_eq!(faces_dos_triangulos(&mesh), vec![0, 0, 1]);
    assert_eq!(vertices_dos_cantos(&mesh), vec![0, 1, 2, 3, 1, 4, 2]);
}
