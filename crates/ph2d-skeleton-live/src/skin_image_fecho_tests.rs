//! Os gates da borda da malha — o contorno que o fecho da imagem lê.
//!
//! ⚠️ O comportamento na cena (o vão fechado, a placa intacta sem vão) mede-se onde a cena mora:
//! `ph2d_app_vec::smoke_bone_par::fresta_tests`.

use super::*;

/// Uma grelha `n × m` de células, cada uma em dois triângulos no MESMO sentido; `fora` lista as
/// células que não existem.
fn grelha(n: u32, m: u32, fora: &[(u32, u32)]) -> (Vec<[f64; 2]>, Vec<[u32; 3]>) {
    let id = |i: u32, j: u32| j * (n + 1) + i;
    let rest = (0..=m)
        .flat_map(|j| (0..=n).map(move |i| [f64::from(i), f64::from(j)]))
        .collect();
    let tris = (0..m)
        .flat_map(|j| (0..n).map(move |i| (i, j)))
        .filter(|c| !fora.contains(c))
        .flat_map(|(i, j)| {
            [
                [id(i, j), id(i + 1, j), id(i + 1, j + 1)],
                [id(i, j), id(i + 1, j + 1), id(i, j + 1)],
            ]
        })
        .collect();
    (rest, tris)
}

fn area(rest: &[[f64; 2]], anel: &[u32]) -> f64 {
    let pts: Vec<[f64; 2]> = anel.iter().map(|&v| rest[v as usize]).collect();
    ph2d_poly2d::signed_area(&pts)
}

/// ⭐ Uma grelha cheia tem UMA borda, com os nós do perímetro e a área da grelha, no sentido dos
/// triângulos.
#[test]
fn a_borda_de_uma_grelha_cheia_e_um_anel_so() {
    let (rest, tris) = grelha(3, 2, &[]);
    let aneis = aneis_da_borda(&tris);
    assert_eq!(aneis.len(), 1, "{aneis:?}");
    assert_eq!(aneis[0].len(), 10, "o perímetro de 3 × 2 tem 10 nós");
    assert!((area(&rest, &aneis[0]) - 6.0).abs() < 1e-12);
}

/// ⭐⭐ Uma grelha com uma célula a menos no meio tem DOIS anéis, em sentidos opostos — a regra
/// não-zero lê o buraco como buraco sem mais nada.
#[test]
fn um_buraco_e_um_anel_no_sentido_oposto() {
    let (rest, tris) = grelha(3, 3, &[(1, 1)]);
    let mut aneis = aneis_da_borda(&tris);
    aneis.sort_by_key(Vec::len);
    assert_eq!(aneis.len(), 2, "{aneis:?}");
    assert_eq!(aneis[0].len(), 4);
    assert!(
        (area(&rest, &aneis[0]) + 1.0).abs() < 1e-12,
        "o buraco gira ao contrário"
    );
    assert!((area(&rest, &aneis[1]) - 9.0).abs() < 1e-12);
}

/// ⚠️ Dois pedaços que se tocam por UM canto não são um contorno — o anel que lá passa sai, e o
/// resto não rebenta.
#[test]
fn um_canto_partilhado_nao_rebenta() {
    let (_, tris) = grelha(2, 2, &[(1, 0), (0, 1)]);
    for anel in aneis_da_borda(&tris) {
        assert!(anel.len() >= 3);
    }
}

/// ⭐ A costura nasce LIGADA — só `"0"` a desliga (para bissecar um report).
#[test]
fn a_costura_nasce_ligada_e_o_zero_bissecta() {
    assert!(costura_de(None));
    assert!(costura_de(Some("1")));
    assert!(costura_de(Some("")));
    assert!(!costura_de(Some("0")));
}

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

/// ⭐⭐ Os anéis guardados são os CALCULADOS, e a gaveta devolve o mesmo `Rc` enquanto a malha vive;
/// outra malha (outro `Rc`) recebe os seus. ⛔ Uma mutação que devolvia vazio no acerto sobrevivia:
/// os gates da cena calculam os anéis à mão.
#[test]
fn os_aneis_guardados_sao_os_calculados_e_ficam_por_malha() {
    let malha = |fora: &[(u32, u32)]| {
        let (rest, tris) = grelha(3, 3, fora);
        std::rc::Rc::new(crate::skinned_mesh::SkinnedMesh {
            mesh: ph2d_poly2d::Mesh2d {
                rest,
                tris,
                size: [3, 3],
            },
            pesos: Vec::new(),
        })
    };
    let a = malha(&[]);
    let primeira = bordas_da(&a);
    let segunda = bordas_da(&a);
    assert!(
        std::rc::Rc::ptr_eq(&primeira, &segunda),
        "o acerto não veio da gaveta"
    );
    assert_eq!(*segunda, aneis_da_borda(&a.mesh.tris));
    let b = malha(&[(1, 1)]);
    assert_eq!(*bordas_da(&b), aneis_da_borda(&b.mesh.tris));
    assert_eq!(bordas_da(&b).len(), 2, "a malha com buraco tem dois anéis");
}
