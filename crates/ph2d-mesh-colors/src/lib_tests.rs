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

fn faces_de(f: &[[u32; 3]]) -> impl Iterator<Item = &[u32]> + Clone {
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
    assert_eq!(t.lado_uniforme().expect("a fixtura e' uniforme"), 1);
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
        let l = t.lado_uniforme().expect("a fixtura e' uniforme");
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
    assert_eq!(
        t.lado_uniforme().expect("a fixtura e' uniforme"),
        1 << NIVEL_MAX
    );
}

/// ⚠️ **Um QUAD e um TRIÂNGULO convivem na mesma malha**, e o interior deles
/// conta-se por fórmulas diferentes.
#[test]
fn a_malha_mista_conta_cada_face_pela_forma_dela() {
    let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2], vec![1, 3, 4, 2]];
    let it = || faces.iter().map(|f| &f[..]);
    let t = Tinta::nova(5, it(), 1);
    let l = t.lado_uniforme().expect("a fixtura e' uniforme");
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

/// ⭐⭐⭐ **SUBIR O NÍVEL NÃO MEXE NA TINTA QUE JÁ LÁ ESTÁ.**
///
/// ⛔ É o gate que faltava quando a [`Tinta::nova`] foi armada sobre uma peça
/// já pintada: ela nasce BRANCA, e o efeito é *«o pincel apagou o meu
/// trabalho»*. O desvio medido foi `1,0` num canal — uma cor inteira, nunca um
/// arredondamento.
#[test]
fn semear_preserva_a_cor_dos_vertices_ao_bit() {
    let (_, f) = tetra();
    let cores = vec![
        [0.10, 0.20, 0.30],
        [0.40, 0.50, 0.60],
        [0.70, 0.80, 0.90],
        [0.11, 0.22, 0.33],
    ];
    for nivel in 0..=3u8 {
        let t = Tinta::semeada(&cores, faces_de(&f), nivel);
        for (v, c) in cores.iter().enumerate() {
            assert_eq!(
                t.de_vertice(v),
                *c,
                "nível {nivel}: a amostra do vértice {v} não é a cor dele"
            );
        }
        // ⭐ E as amostras NOVAS ficam dentro do envelope das cores de entrada —
        // uma mistura convexa não pode sair dele, e um endereço trocado sairia.
        for a in t.amostras() {
            for e in 0..3 {
                let lo = cores.iter().map(|c| c[e]).fold(f32::MAX, f32::min);
                let hi = cores.iter().map(|c| c[e]).fold(f32::MIN, f32::max);
                assert!(
                    a[e] >= lo - 1e-6 && a[e] <= hi + 1e-6,
                    "nível {nivel}: a amostra {a:?} saiu do envelope [{lo}, {hi}]"
                );
            }
        }
    }
}

/// ⭐ **A `lado = 1` semear é o mesmo que copiar, AO BIT** — o controlo que
/// amarra a porta nova ao caso base.
#[test]
fn a_lado_um_semear_e_copiar() {
    let (_, f) = tetra();
    let cores = vec![
        [0.10, 0.20, 0.30],
        [0.40, 0.50, 0.60],
        [0.70, 0.80, 0.90],
        [0.11, 0.22, 0.33],
    ];
    let a = Tinta::semeada(&cores, faces_de(&f), 0);
    let b = Tinta::do_plano_por_vertice(&cores, faces_de(&f));
    assert_eq!(a.amostras(), b.amostras());
}

/// ⛔ **E o ramo dos QUADS de [`Tinta::semeada`] precisa de um quad na fixtura.**
///
/// ⚠️ A irmã corre sobre um tetraedro, que é **só de triângulos** — e uma
/// mutação que trocava a mistura bilinear pelo primeiro canto **SOBREVIVEU**
/// a ela. *Uma fixtura que não contém a forma não testa o ramo dela.*
#[test]
fn semear_preserva_a_cor_dos_vertices_tambem_num_quad() {
    let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2, 3]];
    let it = || faces.iter().map(|f| &f[..]);
    let cores = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
    ];
    for nivel in 0..=3u8 {
        let t = Tinta::semeada(&cores, it(), nivel);
        for (v, c) in cores.iter().enumerate() {
            assert_eq!(
                t.de_vertice(v),
                *c,
                "nível {nivel}: a amostra do vértice {v} do QUAD não é a cor dele"
            );
        }
        // ⭐ E o CENTRO é a média dos quatro — a bilinear em `(½, ½)`. Um
        // `c[0]` ali passa no teste dos cantos e falha aqui.
        if nivel >= 1 {
            let l = t.lado_uniforme().expect("a fixtura e' uniforme");
            let centro = t.indice_quad(0, &faces[0], l / 2, l / 2) as usize;
            let m = t.amostras()[centro];
            for e in 0..3 {
                let esperado = cores.iter().map(|c| c[e]).sum::<f32>() / 4.0;
                assert!(
                    (m[e] - esperado).abs() <= 1e-6,
                    "nível {nivel}: o centro do quad é {m:?} e devia ser a média"
                );
            }
        }
    }
}

/// ⭐⭐⭐⭐ **UM TRIÂNGULO MARCADO COM O SENTINELA É O MESMO TRIÂNGULO** — o
/// ramo de [`topo::cantos`] que lê `[a, b, c, TRI]` entrega **exactamente** o
/// que `[a, b, c]` entrega, ao bit.
///
/// ⛔⛔ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE, e o que ela expôs não
/// foi uma fixtura em falta — foi um ramo SEM CHAMADOR.** Trocar aquele `=> 3`
/// por `=> 4` passava os `18` gates desta crate, porque **todas** as fixturas
/// passam fatias de `3`, e no produto a [`ph2d_mesh::Face::verts`] **corta o
/// sentinela antes de sair** (ela devolve `&self.0[..self.vert_count()]`).
///
/// ⚠️ **E é por isso que o ramo FICA em vez de ser apagado:** a porta desta
/// crate aceita `&[u32]` cru, e o array de uma `Face` desta casa é um
/// `[u32; 4]` com `u32::MAX` no 4.º slot. Um chamador que passe `&face.0[..]`
/// em vez de `face.verts()` é o erro mais natural que existe aqui — e sem o
/// ramo ele lê um triângulo como quad com um canto `u32::MAX`, que é um
/// `index out of bounds` na [`Tinta::semeada`] ou endereços trocados em
/// silêncio. *Um ramo defensivo sem gate é indistinguível de um ramo morto, e
/// os dois leem-se igual numa mutação.*
///
/// ⭐ A régua é a IGUALDADE das duas `Tinta` inteiras (a disposição, a
/// topologia e as amostras), e não uma contagem: uma contagem igual com
/// endereços trocados é exactamente o defeito que isto existe para impedir.
#[test]
fn o_sentinela_do_triangulo_nao_muda_uma_amostra() {
    let (_, f3) = tetra();
    let f4: Vec<[u32; 4]> = f3.iter().map(|t| [t[0], t[1], t[2], topo::TRI]).collect();
    let cores = vec![
        [0.10, 0.20, 0.30],
        [0.40, 0.50, 0.60],
        [0.70, 0.80, 0.90],
        [0.11, 0.22, 0.33],
    ];
    for nivel in 0..=3u8 {
        let bare = || f3.iter().map(|t| &t[..]);
        let sent = || f4.iter().map(|t| &t[..]);

        // ⭐ O CONTROLO vem primeiro: o sentinela TEM de ser lido, senão este
        //   gate mede duas listas de três e não afirma nada.
        assert_eq!(topo::cantos(&f4[0][..]), 3, "o sentinela não foi lido");
        assert_eq!(f4[0].len(), 4, "a fatia marcada tem mesmo quatro entradas");

        assert_eq!(
            Tinta::nova(4, sent(), nivel),
            Tinta::nova(4, bare(), nivel),
            "nível {nivel}: o plano em branco de um triângulo marcado difere"
        );
        assert_eq!(
            Tinta::semeada(&cores, sent(), nivel),
            Tinta::semeada(&cores, bare(), nivel),
            "nível {nivel}: a semeadura de um triângulo marcado difere"
        );
    }
}
