//! Os gates da LEI: a fronteira partilhada, a orientação e a leitura.

use super::amostragem::{leitura_tri, posicao_tri};
use super::*;

/// Dois triângulos que partilham a aresta `1–2`, e o segundo percorre-a **ao
/// contrário** — que é o caso normal numa malha orientada.
/// ⭐ **Uma grelha `2×2` de quads sobre `[0,2]²`** — e o tamanho dela é
/// DERIVADO da pergunta: os quatro lados de um quad têm de ser partilhados
/// pelo menos uma vez, senão o ramo que não é cruzado fica sem régua (ver
/// [`dois_quads_leem_a_mesma_amostra_na_aresta_comum`]). ⭐ Cada célula é o
/// quadrado unitário, logo `(u, v)` de uma face **é** a posição no mundo menos
/// o canto dela — o que torna um campo bilinear exprimível sem conversão.
fn grelha_de_quads() -> (Vec<[f32; 3]>, Vec<Vec<u32>>) {
    let p: Vec<[f32; 3]> = (0..3)
        .flat_map(|y| (0..3).map(move |x| [x as f32, y as f32, 0.0]))
        .collect();
    let faces: Vec<Vec<u32>> = vec![
        vec![0, 1, 4, 3],
        vec![1, 2, 5, 4],
        vec![3, 4, 7, 6],
        vec![4, 5, 8, 7],
    ];
    (p, faces)
}

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

/// ⭐⭐⭐⭐ **A FRONTEIRA PARTILHADA DE DOIS QUADS** — a irmã da
/// [`as_duas_faces_leem_a_mesma_amostra_na_aresta_comum`], que é de
/// TRIÂNGULOS.
///
/// ⛔⛔ **Ela nasceu de uma MUTAÇÃO SOBREVIVENTE, e a razão é a fixtura:** o
/// [`sitio_quad`] tem DOIS lados que andam para trás (`c→d` no eixo `i`,
/// `d→a` no eixo `j`), e trocar o `t: lado - j` do `d→a` por `t: j` passava os
/// `19` gates desta crate. *Nenhuma fixtura tinha dois quads a tocarem-se* — a
/// irmã são dois triângulos, e o único quad do corpus está sozinho, onde
/// inverter o `t` de uma aresta é apenas uma PERMUTAÇÃO da própria aresta.
///
/// ⚠️⚠️ **E é por isso que a bijecção não o via:** ela conta índices
/// distintos, e uma permutação dentro do bloco de uma aresta deixa a contagem
/// **exactamente igual**. O que a apanha é a IGUALDADE por ponto FÍSICO.
///
/// ⭐⭐ **A malha é uma grelha `2×2` de quads, e o tamanho dela é DERIVADO da
/// pergunta:** os quatro lados de um quad têm de ser partilhados **pelo menos
/// uma vez**, senão o ramo que não é cruzado fica sem régua — que foi
/// exactamente como o `d→a` chegou aqui. Numa fita de dois só as arestas
/// `1`/`3` se tocam, e o gémeo `c→d` ficava de fora; na grelha, `3–4` é o lado
/// **`2`** (`c→d`, o outro que anda para trás) de um e o lado `0` de outro.
/// ⚠️ E o vértice do meio é partilhado por **quatro** faces, que é o único
/// sítio onde um canto é visitado mais do que duas vezes.
///
/// ⭐ As posições são comparadas **ao bit** e isso é exacto, não uma folga: a
/// bilinear dos dois lados reduz-se à MESMA expressão na aresta comum
/// (`u = 1` num, `u = 0` no outro), e as coordenadas são inteiros pequenos com
/// `lado` potência de dois.
#[test]
fn dois_quads_leem_a_mesma_amostra_na_aresta_comum() {
    use super::amostragem::posicao_quad;
    use std::collections::{BTreeMap, BTreeSet};

    let (p, faces) = grelha_de_quads();
    let it = || faces.iter().map(|f| &f[..]);

    for nivel in 1..=4u8 {
        let t = Tinta::nova(p.len(), it(), nivel);
        let l = t.lado();
        let mut visto: BTreeMap<[u32; 3], u32> = BTreeMap::new();
        let mut partilhadas = 0usize;

        for (fi, f) in faces.iter().enumerate() {
            let q = [
                p[f[0] as usize],
                p[f[1] as usize],
                p[f[2] as usize],
                p[f[3] as usize],
            ];
            for j in 0..=l {
                for i in 0..=l {
                    let pos = posicao_quad(q, l, (i, j));
                    let chave = [pos[0].to_bits(), pos[1].to_bits(), pos[2].to_bits()];
                    let idx = t.indice_quad(fi, f, i, j);
                    if let Some(antes) = visto.insert(chave, idx) {
                        partilhadas += 1;
                        assert_eq!(
                            antes, idx,
                            "nível {nivel}, face {fi}, ({i},{j}): o mesmo ponto \
                             da superfície tem DOIS endereços"
                        );
                    }
                }
            }
        }

        // ⭐ CONTROLO 1: a aresta comum TEM de ter sido visitada pelas duas —
        //   sem ele, um gate que nunca vê um ponto duas vezes passa por vácuo.
        //   `4(L+1)² − (2L+1)²`: os pares (face, ponto) menos os pontos
        //   DISTINTOS da grelha `2×2`.
        let esperadas = 4 * (l as usize + 1).pow(2) - (2 * l as usize + 1).pow(2);
        assert_eq!(
            partilhadas, esperadas,
            "nível {nivel}: as arestas comuns não foram percorridas pelas duas faces"
        );
        // ⭐ CONTROLO 2: e pontos DISTINTOS não podem colidir num índice.
        let distintos: BTreeSet<u32> = visto.values().copied().collect();
        assert_eq!(
            distintos.len(),
            visto.len(),
            "nível {nivel}: dois pontos distintos da superfície colidiram"
        );
    }
}

/// ⭐⭐⭐ **A LEITURA DE UM QUAD REPRODUZ UM CAMPO BILINEAR, EXACTAMENTE** — a
/// irmã da [`a_leitura_reproduz_um_campo_linear`], para a metade da superfície
/// que o produto de facto tem.
///
/// ⛔⛔ **Ela nasce com a própria lei ([`leitura_quad`]), que não existia:** a
/// única leitura desta crate era a de triângulos, e a malha de escultura desta
/// casa é **quase toda de quads** — a `uv_sphere` só tem triângulos nos dois
/// pólos. *Uma lei de leitura que só sabe ler um terço da superfície do
/// produto não é uma lei de leitura*, e a mesma ausência já tinha mordido do
/// lado da ESCRITA (o laço do dab só tratava triângulos, e o pincel não
/// pintava nada).
///
/// ⭐ **O campo é `(x, y, x·y)` e o `x·y` é a metade que decide:** os dois
/// primeiros canais são reproduzidos por qualquer interpolação que some `1`
/// (uma régua de partição-da-unidade passa sobre o defeito), e só o produto
/// distingue a **bilinear** de uma média dos quatro cantos ou de uma leitura
/// que caísse na célula vizinha.
///
/// ⚠️ E ele é lido nas QUATRO faces, logo atravessa as arestas partilhadas: um
/// campo contínuo com uma leitura descontínua na costura reprova aqui.
#[test]
fn a_leitura_de_um_quad_reproduz_um_campo_bilinear() {
    use super::amostragem::posicao_quad;

    let (pos, faces) = grelha_de_quads();
    let it = || faces.iter().map(|f| &f[..]);
    for nivel in 0..=4u8 {
        let mut t = Tinta::nova(pos.len(), it(), nivel);
        let l = t.lado();

        let canto = |f: &[u32]| {
            [
                pos[f[0] as usize],
                pos[f[1] as usize],
                pos[f[2] as usize],
                pos[f[3] as usize],
            ]
        };
        for (fi, f) in faces.iter().enumerate() {
            let q = canto(f);
            let mut onde: Vec<(u32, (u32, u32))> = Vec::new();
            t.para_cada_amostra_quad(fi, f, |idx, ij| onde.push((idx, ij)));
            for (idx, ij) in onde {
                let p = posicao_quad(q, l, ij);
                t.amostras_mut()[idx as usize] = [p[0], p[1], p[0] * p[1]];
            }
        }

        let mut pior = 0.0f32;
        for (fi, f) in faces.iter().enumerate() {
            let q = canto(f);
            for a in 0..=12 {
                for b in 0..=12 {
                    let uv = [a as f32 / 12.0, b as f32 / 12.0];
                    let p = posicao_quad(q, 12, (a, b));
                    let esperado = [p[0], p[1], p[0] * p[1]];
                    let lido = t.cor_quad(fi, f, uv);
                    for e in 0..3 {
                        pior = pior.max((lido[e] - esperado[e]).abs());
                    }
                }
            }
        }
        // ⚠️ A `lado = 1` a célula é a face inteira e o `x·y` de uma face é
        //   exactamente a bilinear dos cantos dela ⇒ a barra é a mesma.
        assert!(
            pior <= 2e-6,
            "nível {nivel}: a leitura de um quad desviou {pior:e} de um campo \
             BILINEAR — ela está a ler a célula errada, ou a média dos quatro"
        );
    }
}

/// ⚠️ **Um ponto FORA do quad é CORTADO, e nunca dá a volta** — a metade que a
/// irmã de triângulos escreve como *«um `floor` negativo satura a zero em
/// silêncio num `as u32` e devolve um endereço plausível e errado»*.
///
/// ⭐ E o controlo positivo é o que a torna honesta: dentro, `u = 1` exacto
/// **não** é um endereço fora da face — o `clamp` do piso em `L−1` põe a
/// célula na última e a fracção em `1`, que devolve o canto.
#[test]
fn um_ponto_fora_do_quad_e_cortado_e_nao_da_a_volta() {
    use super::amostragem::leitura_quad;

    for nivel in 0..=3u8 {
        let l = 1u32 << nivel;
        let borda = leitura_quad(l, [1.0, 1.0]);
        for (fora, nome) in [
            ([1.5f32, 0.5], "u"),
            ([0.5, 1.5], "v"),
            ([-0.5, -0.5], "os dois"),
        ] {
            for ((i, j), w) in leitura_quad(l, fora) {
                assert!(
                    i <= l && j <= l,
                    "lado {l}, fora em {nome}: o endereço ({i},{j}) saiu da face"
                );
                assert!(w.is_finite() && (0.0..=1.0).contains(&w));
            }
        }
        // ⭐ CONTROLO: na borda EXACTA o peso todo está no canto `(L, L)`.
        let (canto, peso) = borda
            .iter()
            .copied()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("quatro pesos");
        assert_eq!(canto, (l, l), "lado {l}: a borda não caiu no canto");
        assert!((peso - 1.0).abs() <= 1e-6, "lado {l}: o canto pesa {peso}");
        // ⭐ E os quatro pesos somam UM em toda parte.
        for uv in [[0.0, 0.0], [0.3, 0.7], [1.0, 0.0], [0.5, 0.5]] {
            let s: f32 = leitura_quad(l, uv).iter().map(|(_, w)| *w).sum();
            assert!(
                (s - 1.0).abs() <= 1e-6,
                "lado {l}, {uv:?}: os pesos somam {s}"
            );
        }
    }
}

/// ⭐⭐⭐⭐ **O PAYLOAD RESOLVE O MESMO ENDEREÇO QUE A LEI, amostra a amostra** —
/// e este teste é, de propósito, **a implementação de referência do gémeo em
/// WGSL**: ele resolve os endereços usando *só* o registo achatado e os três
/// globais, que é exactamente o que um shader tem na mão.
///
/// ⛔⛔ Sem ele, o payload é uma segunda redacção da disposição — e a forma
/// como isso falha não é um erro, é **tinta no sítio errado**: uma face lê o
/// bloco de outra, ou uma aresta lê-se ao contrário. *Uma contagem igual com
/// endereços trocados passa em toda régua que conte.*
///
/// ⚠️ A régua percorre as DUAS retículas (tri e quad) nas quatro faces da
/// grelha e nas quatro do tetraedro, em todos os níveis — e compara contra a
/// [`Tinta::indice_de`], que é a porta do produto.
#[test]
fn o_payload_resolve_o_mesmo_endereco_que_a_lei() {
    use super::topo::{PAYLOAD_STRIDE, TRI};

    /// O que um shader faz: do registo da face e dos três globais, ao índice.
    fn le(reg: &[u32], lado: u32, verts: u32, arestas: u32, sitio: Sitio) -> u32 {
        match sitio {
            Sitio::Canto(c) => reg[c],
            Sitio::Aresta { lado_da_face, t } => {
                let w = reg[4 + lado_da_face];
                let (id, virada) = (w >> 1, w & 1 == 1);
                let t = if virada { lado - t } else { t };
                verts + id * (lado - 1) + (t - 1)
            }
            Sitio::Interior(n) => verts + arestas * (lado - 1) + reg[8] + n,
        }
    }

    let (_, quads) = grelha_de_quads();
    let tetra: Vec<Vec<u32>> = vec![vec![0, 2, 1], vec![0, 1, 3], vec![0, 3, 2], vec![1, 2, 3]];
    for (nome, verts_n, faces) in [("grelha de quads", 9usize, quads), ("tetraedro", 4, tetra)] {
        let it = || faces.iter().map(|f| &f[..]);
        for nivel in 0..=3u8 {
            let t = Tinta::nova(verts_n, it(), nivel);
            let l = t.lado();
            let topo = t.topologia();
            let mut pay = Vec::new();
            assert!(
                topo.payload(it(), &mut pay),
                "{nome}: a porta recusou as faces dela"
            );
            assert_eq!(
                pay.len(),
                faces.len() * PAYLOAD_STRIDE,
                "{nome}: o payload não tem um registo por face"
            );
            let (verts, arestas) = (topo.verts() as u32, topo.arestas() as u32);

            let mut conferidas = 0usize;
            for (fi, f) in faces.iter().enumerate() {
                let reg = &pay[fi * PAYLOAD_STRIDE..(fi + 1) * PAYLOAD_STRIDE];
                // ⭐ CONTROLO: o slot que um triângulo não usa leva o sentinela.
                if f.len() == 3 {
                    assert_eq!(
                        reg[3], TRI,
                        "{nome}, face {fi}: o canto 3 não é o sentinela"
                    );
                    assert_eq!(reg[7], TRI, "{nome}, face {fi}: o lado 3 não é o sentinela");
                }
                assert_eq!(
                    reg[9] as usize,
                    f.len(),
                    "{nome}, face {fi}: contagem de cantos"
                );

                let mut ver = |sitio: Sitio| {
                    let esperado = t.indice_de(fi, f, sitio);
                    let lido = le(reg, l, verts, arestas, sitio);
                    assert_eq!(
                        lido, esperado,
                        "{nome}, nível {nivel}, face {fi}, {sitio:?}: o payload \
                         resolve {lido} e a lei resolve {esperado}"
                    );
                    conferidas += 1;
                };
                if f.len() == 3 {
                    for i in 0..=l {
                        for j in 0..=(l - i) {
                            ver(sitio_tri(l, i, j, l - i - j));
                        }
                    }
                } else {
                    for j in 0..=l {
                        for i in 0..=l {
                            ver(sitio_quad(l, i, j));
                        }
                    }
                }
            }
            // ⭐ CONTROLO: uma régua que não visita nada passa por vácuo.
            assert!(
                conferidas >= faces.len() * (l as usize + 1),
                "{nome}, nível {nivel}: só {conferidas} amostras conferidas"
            );
        }
    }
}
