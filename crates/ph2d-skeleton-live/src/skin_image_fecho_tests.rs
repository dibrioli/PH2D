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
            mascara: None,
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
    assert_eq!(segunda.aneis, aneis_da_borda(&a.mesh.tris));
    let b = malha(&[(1, 1)]);
    assert_eq!(bordas_da(&b).aneis, aneis_da_borda(&b.mesh.tris));
    assert_eq!(
        bordas_da(&b).aneis.len(),
        2,
        "a malha com buraco tem dois anéis"
    );
}

/// Membros rectangulares `[x0, y0, x1, y1]`, cada um preso só ao osso dado (de `3`, todos na pose
/// de repouso: o posado é o repouso) — a costura deles, sem tinta, à escala de `1` texel.
fn cose(membros: &[([f64; 4], usize)]) -> crate::skin_image_costura::Costura {
    use ph2d_skeleton::SkinBone;
    let (mut rest, mut tris, mut pesos) = (Vec::new(), Vec::new(), Vec::new());
    for &([x0, y0, x1, y1], osso) in membros {
        let k = u32::try_from(rest.len()).expect("poucos");
        rest.extend([[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);
        tris.extend([[k, k + 1, k + 2], [k, k + 2, k + 3]]);
        for _ in 0..4 {
            pesos.extend((0..3).map(|j| if j == osso { 1.0 } else { 0.0 }));
        }
    }
    let mesh = ph2d_poly2d::Mesh2d {
        rest,
        tris,
        size: [32, 16],
    };
    let bordas = crate::skin_image_arte::BordasDaMalha::da(&mesh, None);
    let osso = |x: f64, tendon: u32| SkinBone {
        rest_a: [x, 0.0],
        rest_b: [x + 10.0, 0.0],
        radius: 20.0,
        pose: Xform::IDENTITY,
        sub: (0, 1),
        tendon,
    };
    let pele = Skin::new(vec![osso(0.0, 0), osso(10.0, 1), osso(20.0, 2)]).expect("a pele");
    crate::skin_image_costura::costura(
        &mesh,
        &bordas,
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        &pele,
        &pesos,
        [[0.0, 0.0], [32.0, 16.0]],
        &[],
    )
}

/// A área dos triângulos cosidos.
fn area_cosida(c: &crate::skin_image_costura::Costura) -> f64 {
    let p = |i: u32| c.local[i as usize].map(f64::from);
    c.tris
        .iter()
        .map(|t| {
            let [a, b, q] = t.map(p);
            0.5 * ((b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1])).abs()
        })
        .sum()
}

/// ⭐⭐⭐ GATE (A5-a, sobrevivente da mutação) — **a costura cose UMA vez o vão mais fino que
/// `VAO_MAXIMO_EM_TEXELS`, e nada acima dele**: dois membros a `2` ossos, de frente, a `1,5` texel
/// cosem a faixa entre eles com área `1,5 × 10` (uma vez só: do lado do membro de índice menor), a
/// 1.ª metade de cada pedaço entra depois do triângulo DELE e a 2.ª depois do do OUTRO membro. A
/// `3` texels (entre `1×` e `2×` o limite) nada se cose.
#[test]
fn a_costura_cose_uma_vez_o_vao_fino_e_nada_acima_do_limite() {
    assert_eq!(
        VAO_MAXIMO_EM_TEXELS, 2.0,
        "as larguras abaixo contam com `2`"
    );
    let par = |g: f64| {
        cose(&[
            ([0.0, 0.0, 10.0, 10.0], 0),
            ([10.0 + g, 0.0, 20.0 + g, 10.0], 2),
        ])
    };
    let c = par(1.5);
    assert!(
        !c.tris.is_empty(),
        "controlo: o vão de 1,5 texel não se coseu"
    );
    let area = area_cosida(&c);
    assert!(
        (area - 15.0).abs() < 1e-3,
        "área cosida {area}, a faixa é 15"
    );
    let (pedacos, resto) = c.local.as_chunks::<8>();
    assert!(resto.is_empty(), "8 pontos por pedaço");
    for g in pedacos {
        assert_eq!(g[0][0], 10.0, "um pedaço cosido do lado do 2.º membro");
    }
    assert_eq!(c.slot.len(), c.tris.len());
    for s in c.slot.as_chunks::<4>().0 {
        assert!(
            s[0] == s[1] && s[2] == s[3] && s[0] < 2 && (2..4).contains(&s[2]),
            "slots {s:?}: a 2.ª metade não entra depois do triângulo do outro membro"
        );
    }
    assert!(par(3.0).tris.is_empty(), "coseu um vão de 3 texels");
}

/// ⭐⭐ GATE (A5-a, sobrevivente da mutação) — **o vão está à frente das DUAS beiras**: uma barra
/// fina que ENTRA no outro membro (a beira dele fica dentro da barra, à frente da beira de fora
/// dela) não se cose. Controlo: a mesma barra fora, a `0,5` texel, cose-se.
#[test]
fn um_membro_dentro_da_tinta_do_outro_nao_se_cose() {
    let membro = ([0.0, 0.0, 10.0, 10.0], 0);
    assert!(
        !cose(&[membro, ([10.5, 2.0, 12.0, 8.0], 2)]).tris.is_empty(),
        "controlo: a barra de fora não se coseu"
    );
    let dentro = cose(&[membro, ([9.5, 2.0, 11.0, 8.0], 2)]);
    assert!(
        dentro.tris.is_empty(),
        "coseu {} triângulos sobre a sobreposição",
        dentro.tris.len()
    );
}
