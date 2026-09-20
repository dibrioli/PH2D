//! Os gates da DISPOSIÇÃO: o caso base, a bijecção e a contagem.

use super::*;

/// Um tetraedro: `4` vértices, `6` arestas, `4` triângulos — a malha fechada
/// mais pequena que existe, e por isso a que expõe a bijecção sem ruído.
fn tetra() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let p = vec![
        [1.0, 1.0, 1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
    ];
    let f = vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]];
    (p, f)
}

fn faces_de(f: &[[u32; 3]]) -> impl Iterator<Item = &[u32]> {
    f.iter().map(|t| &t[..])
}

/// ⭐⭐⭐ **O NÍVEL ZERO É A COR POR-VÉRTICE, AO BIT.**
///
/// ⛔ Sem esta metade a família nova precisaria de um degrau de formato. Com
/// ela, a `Tinta` de nível `0` **é** o `colors()` de hoje — o mesmo vector, na
/// mesma ordem, com os mesmos bits.
#[test]
fn o_nivel_zero_e_a_cor_por_vertice_ao_bit() {
    let (_, f) = tetra();
    let cores = vec![
        [0.10, 0.20, 0.30],
        [0.40, 0.50, 0.60],
        [0.70, 0.80, 0.90],
        [0.11, 0.22, 0.33],
    ];
    let t = Tinta::do_plano_por_vertice(&cores, faces_de(&f));
    assert_eq!(t.nivel(), 0);
    assert_eq!(t.lado(), 1);
    assert_eq!(t.amostras(), &cores[..], "o plano É o vector de entrada");
    assert_eq!(t.plano_por_vertice(), &cores[..]);
    for (v, c) in cores.iter().enumerate() {
        assert_eq!(t.de_vertice(v), *c, "a amostra de um vértice é o vértice");
    }
    // ⭐ O CONTROLO: uma `Tinta::nova` no mesmo nível concorda na DISPOSIÇÃO.
    let n = Tinta::nova(cores.len(), faces_de(&f), 0);
    assert_eq!(n.amostras().len(), cores.len());
}

/// ⭐⭐⭐⭐ **A DISPOSIÇÃO É UMA BIJECÇÃO: percorrer todas as faces visita cada
/// amostra pelo menos uma vez e nunca um índice de fora.**
///
/// ⛔⛔ É este gate que prova as três coisas que uma leitura do código não
/// mostra: que o bloco das arestas não se sobrepõe ao dos vértices, que duas
/// faces vizinhas **partilham** a amostra de aresta (senão haveria mais
/// índices distintos do que o total), e que nenhuma face escreve fora da fatia
/// dela. *Uma falha em qualquer das três dá exactamente o mesmo sintoma no
/// ecrã — tinta no sítio errado — e só a contagem as separa.*
#[test]
fn a_disposicao_e_uma_bijeccao_em_todos_os_niveis() {
    let (_, f) = tetra();
    for nivel in 0..=4u8 {
        let t = Tinta::nova(4, faces_de(&f), nivel);
        let l = t.lado();
        let n = t.amostras().len();
        let mut visto = vec![false; n];
        for (fi, tri) in f.iter().enumerate() {
            t.para_cada_amostra_tri(fi, &tri[..], |idx, _| {
                assert!(
                    (idx as usize) < n,
                    "nível {nivel}: a face {fi} escreveu fora do plano ({idx} de {n})"
                );
                visto[idx as usize] = true;
            });
        }
        assert!(
            visto.iter().all(|&b| b),
            "nível {nivel}: {} amostras de {n} nunca foram visitadas — há um buraco na disposição",
            visto.iter().filter(|b| !**b).count()
        );
        // A contagem exacta, escrita à mão a partir da geometria do tetraedro.
        let esperado = 4 + 6 * (l as usize - 1) + 4 * interior_por_face(3, l) as usize;
        assert_eq!(n, esperado, "nível {nivel}: a contagem não é a da fórmula");
    }
}

/// ⚠️ **A contagem de uma malha grande é `≈ V · lado²`**, e é dela que sai a
/// tabela de memória do [`NIVEL_MAX`].
///
/// Numa malha de triângulos fechada e grande, `E ≈ 3V` e `F ≈ 2V`, logo
/// `V + 3V(L−1) + 2V(L−1)(L−2)/2 = V·L²` — **exactamente**, e a álgebra é
/// conferida aqui em vez de acreditada.
#[test]
fn a_contagem_de_uma_malha_grande_e_v_vezes_lado_ao_quadrado() {
    for lado in [1u64, 2, 4, 8, 16, 32] {
        let v = 1_000_000u64;
        let (e, fa) = (3 * v, 2 * v);
        let n = v + e * (lado - 1) + fa * (lado - 1) * (lado.saturating_sub(2)) / 2;
        assert_eq!(n, v * lado * lado, "lado {lado}");
    }
}

/// ⛔ **Um nível acima do tecto é CORTADO, não aceite nem recusado.**
#[test]
fn o_nivel_e_cortado_no_tecto() {
    let (_, f) = tetra();
    let t = Tinta::nova(4, faces_de(&f), 9);
    assert_eq!(t.nivel(), NIVEL_MAX);
    assert_eq!(t.lado(), 1 << NIVEL_MAX);
}

/// ⚠️ **Um QUAD e um TRIÂNGULO convivem na mesma malha**, e o interior deles
/// conta-se por fórmulas diferentes.
#[test]
fn a_malha_mista_conta_cada_face_pela_forma_dela() {
    let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2], vec![1, 3, 4, 2]];
    let it = || faces.iter().map(|f| &f[..]);
    let t = Tinta::nova(5, it(), 1);
    let l = t.lado();
    assert_eq!(l, 2, "nível 1 é lado 2 — o `k` não é o `lado`");
    assert_eq!(
        interior_por_face(3, l),
        0,
        "um triângulo a lado 2 não tem interior"
    );
    assert_eq!(interior_por_face(4, l), 1, "um quad a lado 2 tem o centro");
    let mut visto = vec![false; t.amostras().len()];
    t.para_cada_amostra_tri(0, &faces[0], |i, _| visto[i as usize] = true);
    t.para_cada_amostra_quad(1, &faces[1], |i, _| visto[i as usize] = true);
    assert!(visto.iter().all(|&b| b), "a malha mista deixou um buraco");
}
