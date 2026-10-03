use super::*;

#[test]
fn o_cruzamento_e_o_mesmo_dos_dois_lados_da_costura() {
    // Um quadrado rodado que atravessa a linha x = 100: o mosaico da esquerda e o da direita
    // cortam-no por lados opostos e têm de escrever os MESMOS pontos na linha.
    let poly: Vec<P> = vec![(37, -91), (163, 7), (71, 141), (-29, 33)];
    let esq = corta(&poly, (-200, -200), (100, 200));
    let dir = corta(&poly, (100, -200), (300, 200));
    let na_linha = |v: &[P]| {
        let mut l: Vec<P> = v.iter().copied().filter(|p| p.0 == 100).collect();
        l.sort_unstable();
        l
    };
    assert_eq!(na_linha(&esq).len(), 2);
    assert_eq!(na_linha(&esq), na_linha(&dir));
}

#[test]
fn o_cruzamento_sai_da_aresta_original_e_nao_do_pedaco() {
    // Uma aresta comprida cortada primeiro por x = 10 e depois por y = 50: o ponto em y = 50 tem
    // de ser o da aresta ORIGINAL — o mesmo que um mosaico que só a corta por y = 50 escreve.
    let tri: Vec<P> = vec![(0, 0), (97, 103), (-50, 80)];
    let a = corta(&tri, (10, -1000), (1000, 50));
    let b = corta(&tri, (-1000, -1000), (1000, 50));
    let em_y = |v: &[P]| {
        v.iter()
            .copied()
            .filter(|p| p.1 == 50 && p.0 > 10)
            .collect::<Vec<_>>()
    };
    assert_eq!(em_y(&a), em_y(&b));
    assert_eq!(em_y(&a).len(), 1);
}

/// ⭐ **O caso que a aresta ORIGINAL decide** (achado por busca, depois de a prova de mutação o
/// mostrar sem régua): dois mosaicos EMPILHADOS, e uma aresta que atravessa o de baixo de ponta a
/// ponta. No de baixo, o corte por `y = −10` vem ANTES do de `y = 60`; do pedaço, o ponto em `y = 60`
/// arredondava para `x = −3`, e o de cima (que só corta por `y = 60`) escreve `x = −2`.
#[test]
fn a_costura_de_mosaicos_empilhados_concorda_quando_a_aresta_atravessa_um_inteiro() {
    let tri: Vec<P> = vec![(-9, -51), (0, 102), (-40, 20)];
    let baixo = corta(&tri, (-100, -10), (100, 60));
    let cima = corta(&tri, (-100, 60), (100, 130));
    let em_60 = |v: &[P]| {
        let mut l: Vec<P> = v.iter().copied().filter(|p| p.1 == 60).collect();
        l.sort_unstable();
        l
    };
    assert_eq!(em_60(&baixo), em_60(&cima));
    assert!(
        em_60(&baixo).contains(&(-2, 60)),
        "o ponto é o da aresta ORIGINAL"
    );
}

#[test]
fn a_divisao_arredonda_ao_mais_perto() {
    assert_eq!(divide_ao_mais_perto(7, 2), 4);
    assert_eq!(divide_ao_mais_perto(-7, 2), -4);
    assert_eq!(divide_ao_mais_perto(5, 3), 2);
    assert_eq!(divide_ao_mais_perto(-5, 3), -2);
    assert_eq!(divide_ao_mais_perto(5, -3), -2);
}

#[test]
fn um_poligono_fora_do_rectangulo_some() {
    let q: Vec<P> = vec![(0, 0), (10, 0), (10, 10), (0, 10)];
    assert!(corta(&q, (20, 20), (30, 30)).is_empty());
    assert_eq!(corta(&q, (-5, -5), (50, 50)), q);
}

/// ⭐ (W9) **Onde a malha mudou**: tudo na 1.ª construção, nada quando nada muda, e só o mosaico de uma
/// pedra que se mexe — é o que deixa a fila do replaneio não percorrer os caminhos longe dela.
#[test]
fn a_area_que_mudou_e_o_mosaico_da_pedra() {
    let reg = vec![[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]];
    let pedra = |x: f64| Shape::Circle {
        center: [x, 22.0],
        radius: 0.5,
    };
    let mut t = TiledMesh::new(Params::default(), 10.0);
    t.update(&reg, &[pedra(25.0)]);
    assert_eq!(t.changed_area(), None, "a 1.ª construção muda tudo");
    t.update(&reg, &[pedra(25.0)]);
    assert_eq!(t.changed_area(), Some(Vec::new()), "nada mudou");
    t.update(&reg, &[pedra(25.5)]);
    let a = t.changed_area().expect("só os mosaicos refeitos");
    assert_eq!(a.len(), 1, "{a:?}");
    let (lo, hi) = a[0];
    assert!(
        lo[0] <= 25.5 && 25.5 <= hi[0] && lo[1] <= 22.0 && 22.0 <= hi[1],
        "{a:?}"
    );
    assert!(hi[0] - lo[0] <= 10.0 + 1e-9, "um mosaico de 10 m: {a:?}");
}
