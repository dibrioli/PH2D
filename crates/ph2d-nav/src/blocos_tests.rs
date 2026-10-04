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
    type Caso<'a> = (&'a [V2], &'a [&'a [u32]], fn(&MeshError) -> bool);
    let casos: [Caso; 4] = [
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
        m.poe((1, 0), dois_triangulos((1, 0)));
        assert!(m.monta().is_ok() && !m.faixas_de_paredes().is_empty());
        m.poe((1, 0), peca((1, 0), verts, polys));
        let e = m.monta().expect_err("a peça má passou");
        assert!(esperado(&e), "caso {i}: {e:?}");
        // (W11) As faixas das paredes são as da malha montada: recusada, nenhuma.
        assert!(m.faixas_de_paredes().is_empty(), "caso {i}");
        // E a recusa NÃO fica: trocada a peça, a malha monta (o erro era do bloco, não da malha).
        m.poe((1, 0), dois_triangulos((1, 0)));
        let ok = m.monta().expect("a peça boa monta");
        assert_eq!(ok.poly_count(), 4);
        let f = m.faixas_de_paredes();
        assert_eq!(
            (f.len(), f.last().map(|f| f.paredes.end)),
            (2, Some(ok.walls().len()))
        );
        assert_eq!(
            ok.island_count(),
            1,
            "caso {i}: a costura não ligou os dois blocos"
        );
    }
}

#[test]
fn um_indice_fora_e_as_areas_que_nao_batem_sao_recusados() {
    let mut m = MalhaPorBlocos::new();
    m.poe(
        (0, 0),
        peca((0, 0), &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], &[&[0, 1, 7]]),
    );
    assert!(matches!(
        m.monta(),
        Err(MeshError::BadIndex { poly: 0, index: 7 })
    ));
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
    MalhaPorBlocos::new().poe(
        (0, 0),
        peca((0, 0), &[[0.0, 0.0], [1.5, 0.0], [1.0, 1.0]], &[&[0, 1, 2]]),
    );
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
        peca(
            (1, 0),
            &[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0]],
            &[&[0, 1, 2, 3]],
        ),
    );
    let por_blocos = m.monta().expect("monta");
    let inteira = NavMesh::from_polygons(
        vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [2.0, 0.0],
            [2.0, 1.0],
        ],
        vec![vec![0, 1, 2, 3], vec![1, 4, 5, 2]],
    )
    .expect("a inteira");
    assert_eq!(por_blocos.diferenca(&inteira), None);
}

/// Um bloco `[x, x + 1] × [0, 1]` com o quadrado inteiro.
fn quadrado(x: i64) -> Peca {
    let a = x as f64;
    peca(
        (x, 0),
        &[[a, 0.0], [a + 1.0, 0.0], [a + 1.0, 1.0], [a, 1.0]],
        &[&[0, 1, 2, 3]],
    )
}

/// A montagem do zero com estes blocos (a régua do incremental).
fn fresca(blocos: &[(Chave, Peca)]) -> NavMesh {
    let mut m = MalhaPorBlocos::new();
    for (k, p) in blocos {
        m.poe(*k, p.clone());
    }
    m.monta().expect("monta")
}

#[test]
fn tirar_um_bloco_desliga_o_vizinho() {
    let mut m = MalhaPorBlocos::new();
    m.poe((0, 0), quadrado(0));
    m.poe((1, 0), quadrado(1));
    assert_eq!(m.monta().expect("monta").island_count(), 1);
    m.tira((1, 0));
    let depois = m.monta().expect("monta");
    assert_eq!(depois.diferenca(&fresca(&[((0, 0), quadrado(0))])), None);
    assert_eq!(
        depois.walls().len(),
        4,
        "a aresta da direita voltou a ser parede"
    );
}

#[test]
fn um_vizinho_com_os_mesmos_pontos_de_lado_que_deixa_de_ligar_devolve_a_parede() {
    // O da direita é trocado por um triângulo que não toca o lado esquerdo — mas os pontos desse lado
    // continuam na peça (soltos): o da esquerda NÃO se recose, e a ligação velha tem de cair.
    let longe = peca(
        (1, 0),
        &[[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.5, 0.5]],
        &[&[1, 2, 4]],
    );
    let mut m = MalhaPorBlocos::new();
    m.poe((0, 0), quadrado(0));
    m.poe((1, 0), quadrado(1));
    m.monta().expect("monta");
    m.poe((1, 0), longe.clone());
    let depois = m.monta().expect("monta");
    assert_eq!(
        depois.diferenca(&fresca(&[((0, 0), quadrado(0)), ((1, 0), longe)])),
        None
    );
    assert_eq!(depois.island_count(), 2);
}

#[test]
fn dois_pontos_cosidos_numa_aresta_que_desce_vao_pela_ordem_dela() {
    // O da esquerda tem (1, 0,3) e (1, 0,6) no lado direito; a aresta esquerda do quadrado da direita
    // DESCE de (1, 1) a (1, 0) e tem de os receber por essa ordem: 0,6 e depois 0,3.
    let esq = peca(
        (0, 0),
        &[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.3],
            [1.0, 0.6],
            [1.0, 1.0],
            [0.0, 1.0],
        ],
        &[&[0, 1, 2, 3, 4, 5]],
    );
    let mut m = MalhaPorBlocos::new();
    m.poe((0, 0), esq);
    m.poe((1, 0), quadrado(1));
    let por_blocos = m.monta().expect("monta");
    let inteira = NavMesh::from_polygons(
        vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.3],
            [1.0, 0.6],
            [1.0, 1.0],
            [0.0, 1.0],
            [2.0, 0.0],
            [2.0, 1.0],
        ],
        vec![vec![0, 1, 2, 3, 4, 5], vec![1, 6, 7, 4, 3, 2]],
    )
    .expect("a inteira");
    assert_eq!(por_blocos.diferenca(&inteira), None);
}

#[test]
fn um_ponto_a_tolerancia_da_fronteira_ve_o_bloco_do_outro_lado() {
    // `p` está a EPS à esquerda da fronteira x = 1: dentro do quadrado da esquerda e, à tolerância, do da
    // direita. A sonda `p + EPS` cai EXACTAMENTE na fronteira, e a fronteira é da coluna da direita.
    let p = [1.0 - EPS, 0.5];
    assert_eq!(p[0] + EPS, 1.0, "a pré-condição da fixtura");
    let por_blocos = fresca(&[((0, 0), quadrado(0)), ((1, 0), quadrado(1))]);
    let mut v = Vec::new();
    por_blocos.locate_all(p, &mut v);
    assert_eq!(v, [0, 1]);
}
