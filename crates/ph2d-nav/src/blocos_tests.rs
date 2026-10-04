use super::*;

/// A peça de um bloco `[x, x + 1] × [y, y + 1]`.
fn peca((x, y): Chave, verts: &[V2], polys: &[&[u32]]) -> Peca {
    let mut ring_off = vec![0];
    let mut ring = Vec::new();
    for p in polys {
        ring.extend_from_slice(p);
        ring_off.push(ring.len() as u32);
    }
    Peca {
        lo: [x as f64, y as f64],
        hi: [x as f64 + 1.0, y as f64 + 1.0],
        verts: verts.to_vec(),
        ring_off,
        ring,
        area: vec![0; polys.len()],
    }
}

/// O quadrado do bloco em dois triângulos.
fn dois_triangulos((x, y): Chave) -> Peca {
    let (a, b) = (x as f64, y as f64);
    peca(
        (x, y),
        &[[a, b], [a + 1.0, b], [a + 1.0, b + 1.0], [a, b + 1.0]],
        &[&[0, 1, 2], &[0, 2, 3]],
    )
}

#[test]
fn as_recusas_sao_as_da_porta_inteira_com_o_indice_da_malha_montada() {
    // O 2.º bloco recusa no seu 2.º polígono: o índice é o da malha montada (2 + 1).
    let casos: [(&[V2], &[&[u32]], fn(&MeshError) -> bool); 4] = [
        (
            &[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0]],
            &[&[0, 1, 2], &[0, 3, 2]],
            |e| matches!(e, MeshError::NotCcw { poly: 3, .. }),
        ),
        (
            &[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.5, 0.5]],
            &[&[0, 1, 2], &[0, 1, 4, 2, 3]],
            |e| matches!(e, MeshError::NotConvex { poly: 3, .. }),
        ),
        (
            &[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0]],
            &[&[0, 1, 2], &[0, 2]],
            |e| matches!(e, MeshError::Degenerate { poly: 3 }),
        ),
        (
            &[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0]],
            &[&[0, 1, 2], &[0, 1, 3]],
            |e| matches!(e, MeshError::NonManifold { .. }),
        ),
    ];
    for (i, (verts, polys, esperado)) in casos.into_iter().enumerate() {
        let mut m = MalhaPorBlocos::new();
        m.poe((0, 0), dois_triangulos((0, 0)));
        m.poe((1, 0), peca((1, 0), verts, polys));
        let e = m.monta().expect_err("a peça má passou");
        assert!(esperado(&e), "caso {i}: {e:?}");
        // E a recusa NÃO fica: trocada a peça, a malha monta (o erro era do bloco, não da malha).
        m.poe((1, 0), dois_triangulos((1, 0)));
        let ok = m.monta().expect("a peça boa monta");
        assert_eq!(ok.poly_count(), 4);
        assert_eq!(ok.island_count(), 1, "caso {i}: a costura não ligou os dois blocos");
    }
}

#[test]
fn um_indice_fora_e_as_areas_que_nao_batem_sao_recusados() {
    let mut m = MalhaPorBlocos::new();
    m.poe((0, 0), peca((0, 0), &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], &[&[0, 1, 7]]));
    assert!(matches!(m.monta(), Err(MeshError::BadIndex { poly: 0, index: 7 })));
    let mut p = dois_triangulos((0, 0));
    p.area.pop();
    m.poe((0, 0), p);
    assert!(matches!(
        m.monta(),
        Err(MeshError::AreaCount { polys: 2, areas: 1 })
    ));
}

#[test]
#[should_panic(expected = "fora do rectângulo")]
fn uma_peca_fora_do_rectangulo_e_um_erro_de_quem_chama() {
    MalhaPorBlocos::new().poe((0, 0), peca((0, 0), &[[0.0, 0.0], [1.5, 0.0], [1.0, 1.0]], &[&[0, 1, 2]]));
}

#[test]
fn um_ponto_de_costura_repetido_na_peca_e_um_vertice_so() {
    // O bloco da esquerda tem o ponto (1, 0) DUAS vezes e o anel usa a 2.ª; a montagem inteira de antes
    // fundia-os (o índice das costuras é por posição): o mesmo vértice, e a costura liga.
    let mut m = MalhaPorBlocos::new();
    m.poe(
        (0, 0),
        peca(
            (0, 0),
            &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [1.0, 0.0]],
            &[&[0, 4, 2, 3]],
        ),
    );
    m.poe(
        (1, 0),
        peca((1, 0), &[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0]], &[&[0, 1, 2, 3]]),
    );
    let por_blocos = m.monta().expect("monta");
    let inteira = NavMesh::from_polygons(
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [2.0, 0.0], [2.0, 1.0]],
        vec![vec![0, 1, 2, 3], vec![1, 4, 5, 2]],
    )
    .expect("a inteira");
    assert_eq!(por_blocos.diferenca(&inteira), None);
}
