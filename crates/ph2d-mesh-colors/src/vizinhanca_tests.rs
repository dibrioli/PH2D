//! Os gates do GRAFO: cada par uma vez, e o grafo atravessa a malha.

use super::*;

fn tetra() -> Vec<[u32; 3]> {
    vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
}

fn faces_de(f: &[[u32; 3]]) -> impl Iterator<Item = &[u32]> {
    f.iter().map(|t| &t[..])
}

fn pares(t: &Tinta, f: &[[u32; 3]]) -> Vec<(u32, u32)> {
    let mut v = Vec::new();
    for (fi, tri) in f.iter().enumerate() {
        t.para_cada_par_tri(fi, &tri[..], |a, b| v.push((a.min(b), a.max(b))));
    }
    v
}

/// ⭐⭐⭐⭐ **CADA PAR UMA VEZ — e é a contagem que o prova.**
///
/// ⛔ Sem o dono da aresta, os pares que correm ao longo de uma aresta da malha
/// saem **duas** vezes, e uma média do anel pesa o dobro exactamente na
/// fronteira. ⚠️ **O sintoma é um fio mais escuro ao longo de metade das
/// arestas da peça** — que é precisamente o defeito que esta família existe
/// para não ter, e ele passaria por qualquer régua de «a tinta chegou».
#[test]
fn cada_par_e_emitido_uma_vez_so() {
    let f = tetra();
    for nivel in 0..=3u8 {
        let t = Tinta::nova(4, faces_de(&f), nivel);
        let mut p = pares(&t, &f);
        let antes = p.len();
        p.sort_unstable();
        p.dedup();
        assert_eq!(
            p.len(),
            antes,
            "nível {nivel}: {} pares repetidos — a fronteira está a contar duas vezes",
            antes - p.len()
        );
        for (a, b) in &p {
            assert_ne!(a, b, "nível {nivel}: um par com ele próprio");
        }
    }
}

/// ⭐⭐⭐ **O GRAFO É CONEXO SOBRE UMA MALHA FECHADA — a tinta é UMA superfície.**
///
/// É esta a frase que separa a família do atlas: ali um vizinho no espaço da
/// textura pode estar noutra ponta da peça, e a média de um `Blur` **pára** na
/// costura. Aqui uma inundação que parta de qualquer amostra alcança **todas**.
#[test]
fn o_grafo_atravessa_as_arestas_da_malha() {
    let f = tetra();
    for nivel in 0..=3u8 {
        let t = Tinta::nova(4, faces_de(&f), nivel);
        let n = t.amostras().len();
        let mut viz: Vec<Vec<u32>> = vec![Vec::new(); n];
        for (a, b) in pares(&t, &f) {
            viz[a as usize].push(b);
            viz[b as usize].push(a);
        }
        let mut visto = vec![false; n];
        let mut pilha = vec![0u32];
        visto[0] = true;
        while let Some(v) = pilha.pop() {
            for &w in &viz[v as usize] {
                if !visto[w as usize] {
                    visto[w as usize] = true;
                    pilha.push(w);
                }
            }
        }
        assert!(
            visto.iter().all(|&b| b),
            "nível {nivel}: {} amostras de {n} ficaram ILHADAS — o grafo não atravessa",
            visto.iter().filter(|b| !**b).count()
        );
    }
}

/// ⭐ **A `lado = 1` os pares são EXACTAMENTE as arestas da malha** — o controlo
/// que amarra a família nova ao que o `Blur` de hoje já percorre.
#[test]
fn a_lado_um_os_pares_sao_as_arestas_da_malha() {
    let f = tetra();
    let t = Tinta::nova(4, faces_de(&f), 0);
    let mut p = pares(&t, &f);
    p.sort_unstable();
    let esperado: Vec<(u32, u32)> = vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    assert_eq!(p, esperado, "um tetraedro tem SEIS arestas, e são estas");
}

/// ⚠️ **A contagem por face é `3·L(L+1)/2` quando ela é dona dos três lados** —
/// a dedução do cabeçalho, conferida.
#[test]
fn a_contagem_por_face_e_a_da_deducao() {
    // Uma face SOZINHA é dona de todos os lados dela.
    let f = vec![[0u32, 1, 2]];
    for nivel in 0..=4u8 {
        let t = Tinta::nova(3, faces_de(&f), nivel);
        let l = u64::from(t.lado_uniforme().expect("a fixtura e' uniforme"));
        let mut n = 0u64;
        t.para_cada_par_tri(0, &f[0][..], |_, _| n += 1);
        assert_eq!(n, 3 * l * (l + 1) / 2, "nível {nivel}");
    }
}

/// ⛔ **Um QUAD também emite cada par uma vez**, e a contagem dele é outra
/// (`2·L(L+1)`): as duas famílias de linhas de uma retícula bilinear.
#[test]
fn o_quad_emite_cada_par_uma_vez() {
    let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2, 3]];
    let it = || faces.iter().map(|f| &f[..]);
    for nivel in 0..=3u8 {
        let t = Tinta::nova(4, it(), nivel);
        let l = u64::from(t.lado_uniforme().expect("a fixtura e' uniforme"));
        let mut p = Vec::new();
        t.para_cada_par_quad(0, &faces[0], |a, b| p.push((a.min(b), a.max(b))));
        let antes = p.len() as u64;
        p.sort_unstable();
        p.dedup();
        assert_eq!(p.len() as u64, antes, "nível {nivel}: par repetido");
        assert_eq!(antes, 2 * l * (l + 1), "nível {nivel}: a contagem");
    }
}
