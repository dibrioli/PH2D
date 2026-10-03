//! Os gates da lei sobre malhas escritas À MÃO (as construídas a partir de colisores, e a varredura
//! aleatória contra o oráculo exacto, vivem na `ph2d-navmesh`, que sabe construí-las).

use crate::geom::{EPS, dist};
use crate::mesh::{MeshError, NavMesh, orient_ok};
use crate::{NoPath, Polyanya, oracle};

/// Um quadrado `4 × 4` com um furo `2 × 2` no meio, em quatro trapézios.
fn anel() -> NavMesh {
    let v = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [0.0, 4.0],
        [1.0, 1.0],
        [3.0, 1.0],
        [3.0, 3.0],
        [1.0, 3.0],
    ];
    let p = vec![
        vec![0, 1, 5, 4],
        vec![1, 2, 6, 5],
        vec![2, 3, 7, 6],
        vec![3, 0, 4, 7],
    ];
    for r in &p {
        assert!(orient_ok(&v, r));
    }
    NavMesh::from_polygons(v, p).expect("o anel é uma malha válida")
}

#[test]
fn o_anel_liga_os_quatro_e_todos_os_vertices_sao_cantos() {
    let m = anel();
    assert_eq!(m.island_count(), 1);
    assert_eq!(m.walls().len(), 8);
    assert!((0..8).all(|v| m.is_corner(v)));
    assert!((m.area() - 12.0).abs() < 1e-12);
    for p in m.polys() {
        assert_eq!(
            p.nbrs.iter().flatten().count(),
            2,
            "cada trapézio tem dois vizinhos"
        );
    }
}

#[test]
fn um_poligono_ao_contrario_e_recusado_em_voz_alta() {
    let v = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]];
    assert!(matches!(
        NavMesh::from_polygons(v, vec![vec![0, 1, 2]]),
        Err(MeshError::NotCcw { poly: 0, .. })
    ));
}

#[test]
fn um_vertice_que_dobra_para_dentro_e_recusado_e_um_a_180_graus_e_aceite() {
    let reflexo = vec![[0.0, 0.0], [2.0, 0.0], [1.0, 0.5], [1.0, 1.0]];
    assert!(matches!(
        NavMesh::from_polygons(reflexo, vec![vec![0, 1, 2, 3]]),
        Err(MeshError::NotConvex { poly: 0, at: 2 })
    ));
    // A grelha inteira da construção produz quase-colineares legítimos: o 180° passa.
    let raso = vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [1.0, 1.0]];
    assert!(NavMesh::from_polygons(raso, vec![vec![0, 1, 2, 3]]).is_ok());
}

#[test]
fn a_localizacao_ve_os_dois_lados_de_uma_aresta_partilhada() {
    let m = anel();
    let mut out = Vec::new();
    m.locate_all([3.5, 0.5], &mut out);
    assert_eq!(
        out,
        vec![0, 1],
        "o ponto está na aresta de baixo-direita: dois polígonos"
    );
    m.locate_all([2.0, 2.0], &mut out);
    assert!(out.is_empty(), "o meio do furo está fora");
}

#[test]
fn o_caminho_contorna_o_furo_pelo_mais_curto() {
    let m = anel();
    let mut s = Polyanya::new();
    let p = s.find_path(&m, [2.0, 0.5], [2.0, 3.5]).expect("há caminho");
    let esperado = 2.0 * (1.0f64 + 0.25).sqrt() + 2.0;
    assert!(
        (p.length - esperado).abs() < 1e-12,
        "{} contra {}",
        p.length,
        esperado
    );
    assert_eq!(
        p.points.len(),
        4,
        "parte, dois cantos, chega: {:?}",
        p.points
    );
    let (_, exacto) = oracle::shortest(&m, [2.0, 0.5], [2.0, 3.5]).expect("o oráculo também acha");
    assert!((p.length - exacto).abs() < 1e-12);
}

#[test]
fn no_mesmo_poligono_o_caminho_e_a_recta() {
    let m = anel();
    let mut s = Polyanya::new();
    let p = s.find_path(&m, [0.5, 0.2], [3.5, 0.3]).expect("há caminho");
    assert_eq!(p.points.len(), 2);
    assert!((p.length - dist([0.5, 0.2], [3.5, 0.3])).abs() < EPS);
}

#[test]
fn fora_da_malha_a_recusa_diz_qual_ponta() {
    let m = anel();
    let mut s = Polyanya::new();
    assert_eq!(
        s.find_path(&m, [2.0, 2.0], [0.5, 0.5]),
        Err(NoPath::StartOff)
    );
    assert_eq!(
        s.find_path(&m, [0.5, 0.5], [9.0, 9.0]),
        Err(NoPath::TargetOff)
    );
}

#[test]
fn o_ponto_mais_perto_de_fora_cai_na_parede() {
    let m = anel();
    let (p, _) = m
        .nearest_point([2.0, 1.6], None)
        .expect("a malha não está vazia");
    assert!(
        (p[0] - 2.0).abs() < 1e-12 && (p[1] - 1.0).abs() < 1e-12,
        "{p:?}"
    );
}

#[test]
fn todo_par_do_anel_bate_o_oraculo() {
    let m = anel();
    let mut s = Polyanya::new();
    let pontos = [
        [0.3, 0.3],
        [3.7, 0.2],
        [3.9, 3.9],
        [0.1, 3.5],
        [2.0, 0.5],
        [3.5, 2.0],
        [2.0, 3.5],
        [0.5, 2.0],
        [1.0, 0.5],
        [4.0, 2.0],
        [1.0, 1.0],
        [3.0, 3.0],
    ];
    for &a in &pontos {
        for &b in &pontos {
            let p = s.find_path(&m, a, b).expect("o anel é uma ilha");
            let (_, ex) = oracle::shortest(&m, a, b).expect("o oráculo acha");
            assert!(
                (p.length - ex).abs() < 1e-9,
                "{a:?} → {b:?}: {} contra {ex}",
                p.length
            );
        }
    }
}

/// ⚠️ **A mesma aresta orientada em dois polígonos é recusada em voz alta** (a malha sobrepõe-se) — a
/// prova de mutação da W6 achou a recusa sem régua depois de a vizinhança ser reescrita.
#[test]
fn dois_poligonos_sobrepostos_sao_recusados() {
    let verts = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
    let r = NavMesh::from_polygons(verts, vec![vec![0, 1, 2], vec![0, 1, 2]]);
    assert!(
        matches!(r, Err(MeshError::NonManifold { .. })),
        "aceitou dois polígonos em cima um do outro: {r:?}"
    );
}
