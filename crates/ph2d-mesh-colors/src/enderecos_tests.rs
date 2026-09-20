//! Os gates da LEI: a fronteira partilhada, a orientação e a leitura.

use super::amostragem::{leitura_tri, posicao_tri};
use super::*;

/// Dois triângulos que partilham a aresta `1–2`, e o segundo percorre-a **ao
/// contrário** — que é o caso normal numa malha orientada.
fn duas_faces() -> (Vec<[f32; 3]>, Vec<Vec<u32>>) {
    let p = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    (p, vec![vec![0, 1, 2], vec![3, 2, 1]])
}

/// ⭐⭐⭐⭐ **A FRONTEIRA É PARTILHADA — e isto é a diferença de espécie para o
/// Ptex, não um detalhe de contagem.**
///
/// As duas faces leem a aresta comum em sentidos opostos. Para cada ponto
/// **físico** dela, as duas têm de devolver **o mesmo índice** — senão há duas
/// células para o mesmo sítio da superfície, que é uma costura.
///
/// ⛔⛔ **E a metade que apanha a orientação esquecida:** sem virar o `t`, as
/// duas leem índices que existem e são DIFERENTES do certo — a tinta fica
/// espelhada ao longo da aresta, e nenhuma contagem o vê.
#[test]
fn as_duas_faces_leem_a_mesma_amostra_na_aresta_comum() {
    let (pos, faces) = duas_faces();
    let it = || faces.iter().map(|f| &f[..]);
    for nivel in 1..=4u8 {
        let t = Tinta::nova(4, it(), nivel);
        let l = t.lado();
        for passo in 0..=l {
            // face 0 = (0,1,2): a aresta 1–2 é o lado `1` (b→c), `i = 0`.
            let a0 = t.indice_tri(0, &faces[0], 0, l - passo, passo);
            // face 1 = (3,2,1): a aresta 2–1 é o lado `1` (b→c), e o mesmo
            // ponto físico está a `l - passo` de `2`.
            let a1 = t.indice_tri(1, &faces[1], 0, passo, l - passo);
            let p0 = posicao_tri(pos[0], pos[1], pos[2], l, (0, l - passo, passo));
            let p1 = posicao_tri(pos[3], pos[2], pos[1], l, (0, passo, l - passo));
            assert_eq!(p0, p1, "nível {nivel}, passo {passo}: não é o mesmo ponto");
            assert_eq!(
                a0, a1,
                "nível {nivel}, passo {passo}: o mesmo ponto da superfície tem DOIS endereços"
            );
        }
        // ⭐ O CONTROLO: o interior de uma face NÃO é partilhado com a outra.
        if l >= 3 {
            let dentro0 = t.indice_tri(0, &faces[0], 1, 1, l - 2);
            let dentro1 = t.indice_tri(1, &faces[1], 1, 1, l - 2);
            assert_ne!(
                dentro0, dentro1,
                "nível {nivel}: dois interiores distintos colidiram"
            );
        }
    }
}

/// ⭐⭐⭐ **A LEITURA REPRODUZ UM CAMPO LINEAR, EXACTAMENTE — e é isto que prova
/// os sub-triângulos INVERTIDOS.**
///
/// ⛔⛔ Se a [`leitura_tri`] escolher sempre o sub-triângulo direito, metade
/// dos pontos cai fora dos três vértices que ela devolve: os pesos ainda somam
/// `1` (a mutação não é apanhada por uma partição da unidade) e a resposta é
/// uma **extrapolação**. Um campo linear é reproduzido *se e só se* o ponto
/// estiver DENTRO do triângulo escolhido. *Uma régua de soma-dos-pesos passa
/// sobre o defeito; uma de reprodução não.*
#[test]
fn a_leitura_reproduz_um_campo_linear() {
    let (pos, faces) = duas_faces();
    let it = || faces.iter().map(|f| &f[..]);
    // O campo: `cor = (x, y, x + y)`, linear na posição ⇒ linear nas baricêntricas.
    for nivel in 1..=4u8 {
        let mut t = Tinta::nova(4, it(), nivel);
        let l = t.lado();
        let cantos = [pos[0], pos[1], pos[2]];
        let tri: Vec<u32> = faces[0].clone();
        let mut onde: Vec<(u32, (u32, u32, u32))> = Vec::new();
        t.para_cada_amostra_tri(0, &tri, |idx, ijk| onde.push((idx, ijk)));
        for (idx, ijk) in onde {
            let p = posicao_tri(cantos[0], cantos[1], cantos[2], l, ijk);
            t.amostras_mut()[idx as usize] = [p[0], p[1], p[0] + p[1]];
        }
        let mut pior = 0.0f32;
        for a in 0..=20 {
            for b in 0..=(20 - a) {
                let bar = [a as f32 / 20.0, b as f32 / 20.0, (20 - a - b) as f32 / 20.0];
                let p = [
                    bar[0] * cantos[0][0] + bar[1] * cantos[1][0] + bar[2] * cantos[2][0],
                    bar[0] * cantos[0][1] + bar[1] * cantos[1][1] + bar[2] * cantos[2][1],
                ];
                let esperado = [p[0], p[1], p[0] + p[1]];
                let lido = t.cor_tri(0, &tri, bar);
                for e in 0..3 {
                    pior = pior.max((lido[e] - esperado[e]).abs());
                }
            }
        }
        assert!(
            pior <= 2e-6,
            "nível {nivel}: a leitura desviou {pior:e} de um campo LINEAR — \
             ela está a extrapolar fora do sub-triângulo"
        );
    }
}

/// ⭐ **A `lado = 1` a leitura É a interpolação baricêntrica dos três cantos** —
/// exactamente o que o renderizador faz hoje com atributos por-vértice.
#[test]
fn a_lado_um_a_leitura_e_a_interpolacao_de_hoje() {
    let (_, faces) = duas_faces();
    let cores = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.5, 0.5, 0.5],
    ];
    let t = Tinta::do_plano_por_vertice(&cores, faces.iter().map(|f| &f[..]));
    for (a, b) in [(0.2f32, 0.3f32), (0.0, 1.0), (0.5, 0.5), (1.0, 0.0)] {
        let bar = [a, b, 1.0 - a - b];
        let lido = t.cor_tri(0, &faces[0], bar);
        let esperado: Vec<f32> = (0..3)
            .map(|e| bar[0] * cores[0][e] + bar[1] * cores[1][e] + bar[2] * cores[2][e])
            .collect();
        for e in 0..3 {
            assert!(
                (lido[e] - esperado[e]).abs() <= 1e-6,
                "({a},{b}) canal {e}: {} contra {}",
                lido[e],
                esperado[e]
            );
        }
    }
}

/// ⚠️ **Os pesos somam `1`** — a metade FRACA, e ela está aqui nomeada como
/// fraca: ela passa sobre o defeito que o gate do campo linear apanha.
#[test]
fn os_pesos_somam_um() {
    for lado in [1u32, 2, 4, 8] {
        for a in 0..=13 {
            for b in 0..=(13 - a) {
                let bar = [
                    f64::from(a) as f32 / 13.0,
                    f64::from(b) as f32 / 13.0,
                    f64::from(13 - a - b) as f32 / 13.0,
                ];
                let s: f32 = leitura_tri(lado, bar).iter().map(|(_, w)| *w).sum();
                assert!((s - 1.0).abs() <= 1e-5, "lado {lado}: os pesos somam {s}");
            }
        }
    }
}

/// ⛔ **Um CANTO não tem endereço de aresta.** A ordem dos ramos da
/// [`sitio_tri`] é a lei; invertê-la dá a dois cantos da mesma aresta o mesmo
/// `t`, e eles passam a partilhar uma célula.
#[test]
fn um_canto_nunca_e_classificado_como_aresta() {
    for lado in [1u32, 2, 4, 8] {
        assert_eq!(sitio_tri(lado, lado, 0, 0), Sitio::Canto(0));
        assert_eq!(sitio_tri(lado, 0, lado, 0), Sitio::Canto(1));
        assert_eq!(sitio_tri(lado, 0, 0, lado), Sitio::Canto(2));
        assert_eq!(sitio_quad(lado, 0, 0), Sitio::Canto(0));
        assert_eq!(sitio_quad(lado, lado, 0), Sitio::Canto(1));
        assert_eq!(sitio_quad(lado, lado, lado), Sitio::Canto(2));
        assert_eq!(sitio_quad(lado, 0, lado), Sitio::Canto(3));
    }
}
