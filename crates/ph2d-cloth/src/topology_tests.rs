//! **A REGIÃO** — as dobradiças que existem, e a coloração que ordena a varredura.

use crate::{ClothTopology, fixtures};

/// ⭐⭐⭐ **GATE — a grade acha exatamente as arestas INTERIORES.**
///
/// Numa grade `n×n` triangulada por diagonal: `n²` diagonais + `n(n−1)`
/// horizontais + `n(n−1)` verticais. As da borda ficam de fora **por não terem
/// duas faces**, que é a definição de dobradiça.
#[test]
fn a_grade_acha_as_arestas_interiores() {
    for n in [2usize, 3, 5] {
        let (x, t) = fixtures::grid(n);
        let topo = fixtures::region(&x, &t);
        let quer = n * n + 2 * n * (n - 1);
        assert_eq!(topo.hinge_count(), quer, "grade {n}x{n}");
    }
}

/// ⭐⭐⭐ **GATE — nenhuma cor liga dois vértices que um elemento acopla.**
///
/// ⛔ É a razão de existir da coloração: dentro de uma cor ninguém pode ler o que
/// o vizinho está a escrever. ⚠️ E a varredura inclui a **DOBRADIÇA**, cujos dois
/// ápices podem não partilhar triângulo nenhum — colorir só por aresta de
/// triângulo os poria na mesma cor, e os dois escreveriam sobre a mesma dobra.
#[test]
fn nenhuma_cor_liga_dois_vizinhos() {
    let (x, t) = fixtures::dome(6);
    let topo = fixtures::region(&x, &t);
    let mut cor = vec![u32::MAX; x.len()];
    for (c, bin) in topo.color_bins().iter().enumerate() {
        for v in bin {
            cor[*v as usize] = u32::try_from(c).unwrap_or(u32::MAX);
        }
    }
    let par = |a: u32, b: u32| {
        assert!(
            a == b || cor[a as usize] != cor[b as usize],
            "{a} e {b} partilham a cor {}",
            cor[a as usize]
        );
    };
    for tri in &t {
        for a in *tri {
            for b in *tri {
                par(a, b);
            }
        }
    }
    for h in 0..topo.hinge_count() {
        let vs = topo.hinges[h].verts();
        for a in vs {
            for b in vs {
                par(a, b);
            }
        }
    }
    // Controle: a coloração não é a trivial (uma cor por vértice), senão o gate
    // acima passaria sobre uma varredura sem paralelismo nenhum.
    assert!(
        topo.color_bins().len() < 12,
        "coloracao degenerou em {} cores",
        topo.color_bins().len()
    );
}

/// ⭐⭐⭐ **GATE — cada vértice tem UMA cor, e todos têm uma.**
#[test]
fn a_coloracao_particiona_os_vertices() {
    let (x, t) = fixtures::dome(4);
    let topo = fixtures::region(&x, &t);
    let mut n = 0;
    let mut visto = vec![false; x.len()];
    for bin in topo.color_bins() {
        for v in bin {
            assert!(!visto[*v as usize], "{v} aparece em duas cores");
            visto[*v as usize] = true;
            n += 1;
        }
    }
    assert_eq!(n, x.len());
}

/// ⭐⭐⭐ **GATE — uma aresta com TRÊS faces não vira dobradiça.**
///
/// ⛔ Uma escultura tem não-manifold (a `line/quadextract` passou jornadas a
/// medi-lo). Escolher duas das três faces seria **inventar** uma lei; ali o pano
/// fica sem resistência a dobrar, que é menos errado do que uma dobra arbitrária
/// — e a membrana continua valendo nas três faces.
#[test]
fn uma_aresta_com_tres_faces_nao_vira_dobradica() {
    let x = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, -1.0, 0.0],
        [0.5, 0.0, 1.0],
    ];
    let t = [[0, 1, 2], [1, 0, 3], [1, 0, 4]];
    let topo = ClothTopology::build(&t, x.len());
    assert_eq!(topo.hinge_count(), 0);
    // Controle: com DUAS faces a mesma aresta vira dobradiça.
    let topo2 = ClothTopology::build(&t[..2], x.len());
    assert_eq!(topo2.hinge_count(), 1);
}

/// ⭐⭐⭐ **GATE — a coloração é função da MALHA, não da ordem de um mapa.**
///
/// ⚠️ A ordem das cores é a ordem de Gauss-Seidel, logo ela muda os últimos bits
/// do resultado. Esta casa tem hash de replay a cobrar determinismo.
#[test]
fn a_coloracao_e_deterministica() {
    let (x, t) = fixtures::dome(5);
    let a = ClothTopology::build(&t, x.len());
    let b = ClothTopology::build(&t, x.len());
    assert_eq!(a.color_bins(), b.color_bins());
}
