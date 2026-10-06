//! O gate da ordem das faces da imagem presa. O comportamento na cena (nada puxado de um membro
//! para o outro, o osso de fora por cima) mede-se onde a cena mora:
//! `ph2d_app_vec::smoke_bone_par::membros_tests`.

use super::*;

/// ⭐⭐ A ordem pelo osso: cada face sai depois das de um osso ANTERIOR, e a ordenação é estável.
#[test]
fn as_faces_saem_pela_ordem_dos_ossos_e_estaveis() {
    // 4 vértices × 2 ossos: 0 e 1 no osso 0, 2 e 3 no osso 1.
    let pesos = [1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0];
    let mut tris = vec![[2, 3, 2], [0, 1, 0], [0, 1, 2], [1, 0, 1]];
    ordena_pelo_osso(&mut tris, &pesos, 4, &[]);
    assert_eq!(tris, vec![[0, 1, 0], [1, 0, 1], [0, 1, 2], [2, 3, 2]]);
    // Sem tabela (a lei derivada) a ordem fica.
    let mut iguais = vec![[2, 3, 2], [0, 1, 0]];
    ordena_pelo_osso(&mut iguais, &[], 4, &[]);
    assert_eq!(iguais, vec![[2, 3, 2], [0, 1, 0]]);
}
