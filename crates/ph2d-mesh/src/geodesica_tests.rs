//! Os gates da [`crate::geodesica`].
//!
//! ⭐ **O oráculo é EXACTO e não um golden:** numa chapa plana a distância pela
//! superfície **é** a distância euclidiana no plano, e calcula-se à mão. É a
//! fixtura mais dura que existe para esta lei, porque é exactamente onde o
//! passeio por arestas falha por `√2`.

use super::*;
use crate::{Face, Mesh};
use std::collections::BinaryHeap;

/// Lado da chapa de teste, em vértices.
const N: usize = 41;
/// O passo da grelha.
const H: f32 = 0.05;

/// ⭐ **Quanto a marcha pode inflar a distância, na faixa onde um pincel decide
/// — e o número sai de um VALE MEDIDO, não de conforto.**
///
/// | | pior razão na faixa `[tecto/2, tecto]` |
/// |---|---|
/// | marcha, chapa de **quads** | `1,051` |
/// | marcha, chapa de **triângulos** (leque) | `1,090` |
/// | **← o vale →** | |
/// | passeio por **arestas** (o que ela substitui) | `1,342`–`1,414` |
///
/// ⚠️ **O `1,090` do leque é a ANTI-DIAGONAL, e é da MALHA e não da lei:** ali
/// os dois vértices não partilham face nenhuma, logo a recta entre eles
/// atravessa **DOIS** triângulos e uma marcha de primeira ordem — que só sabe
/// desdobrar UM — não a pode ver. Um leque é o pior caso de anisotropia que se
/// consegue construir, o que faz dele a fixtura conservadora.
const BARRA_DA_RAZAO: f32 = 1.12;

/// Uma chapa plana de **quads** — a forma da malha que o botão de retopologia
/// entrega, e a que faz a marcha degenerar se os cantos vierem da adjacência.
fn chapa_de_quads() -> Mesh {
    let mut pos = Vec::with_capacity(N * N);
    for i in 0..N {
        for j in 0..N {
            pos.push([i as f32 * H, j as f32 * H, 0.0]);
        }
    }
    let at = |i: usize, j: usize| (i * N + j) as u32;
    let mut faces = Vec::with_capacity((N - 1) * (N - 1));
    for i in 0..N - 1 {
        for j in 0..N - 1 {
            faces.push(Face::quad(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("a chapa é construída aqui e é válida")
}

/// A mesma chapa, triangulada por leque — o outro lado da mesma pergunta.
fn chapa_de_triangulos() -> Mesh {
    let mut pos = Vec::with_capacity(N * N);
    for i in 0..N {
        for j in 0..N {
            pos.push([i as f32 * H, j as f32 * H, 0.0]);
        }
    }
    let at = |i: usize, j: usize| (i * N + j) as u32;
    let mut faces = Vec::with_capacity(2 * (N - 1) * (N - 1));
    for i in 0..N - 1 {
        for j in 0..N - 1 {
            faces.push(Face::tri(at(i, j), at(i + 1, j), at(i + 1, j + 1)));
            faces.push(Face::tri(at(i, j), at(i + 1, j + 1), at(i, j + 1)));
        }
    }
    Mesh::from_parts(pos, faces).expect("a chapa é construída aqui e é válida")
}

/// ⚠️ **O CONTROLO da lei** — o passeio por ARESTAS, que é o que a marcha
/// substitui. Sem ele o gate não distingue *«a marcha é boa»* de *«esta malha é
/// fácil»*.
fn passeio_por_arestas(mesh: &Mesh, semente: u32) -> Vec<f32> {
    let pos = mesh.positions();
    let mut d = vec![f32::INFINITY; pos.len()];
    d[semente as usize] = 0.0;
    let mut fila = BinaryHeap::new();
    fila.push((std::cmp::Reverse(Ordenavel(0.0)), semente));
    let viz = &mesh.adjacency().vert_verts;
    while let Some((std::cmp::Reverse(Ordenavel(dv)), v)) = fila.pop() {
        if dv > d[v as usize] {
            continue;
        }
        for &u in viz.neighbours(v as usize) {
            let nd = dv + comprimento(pos[v as usize], pos[u as usize]);
            if nd < d[u as usize] {
                d[u as usize] = nd;
                fila.push((std::cmp::Reverse(Ordenavel(nd)), u));
            }
        }
    }
    d
}

/// A pior razão `medido / exacto` e o pior erro ABSOLUTO em arestas, sobre a
/// **faixa onde um pincel decide**.
///
/// ⚠️⚠️ **A população é `[tecto/2, tecto]` e isso é a metade que torna o gate
/// honesto.** Junto da semente a razão é dominada pelo tamanho da CÉLULA e não
/// pela lei (a um passo de distância, meia aresta de erro é `50 %`), e ali
/// **nada é nunca cortado** — quem decide um corte está à beira do tecto. *Uma
/// régua que varre onde a decisão não acontece mede o tamanho da malha.*
fn na_faixa_da_decisao(
    mesh: &Mesh,
    medido: &dyn Fn(u32) -> f32,
    semente: u32,
    tecto: f32,
) -> (f32, f32, f32, usize) {
    let pos = mesh.positions();
    let s = pos[semente as usize];
    let (mut pior, mut melhor, mut abs, mut n) = (0.0f32, f32::INFINITY, 0.0f32, 0usize);
    for (v, p) in pos.iter().enumerate() {
        let exacto = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2)).sqrt();
        if exacto < tecto * 0.5 || exacto > tecto {
            continue;
        }
        let m = medido(v as u32);
        if !m.is_finite() {
            continue;
        }
        n += 1;
        pior = pior.max(m / exacto);
        melhor = melhor.min(m / exacto);
        abs = abs.max((m - exacto).abs() / H);
    }
    (melhor, pior, abs, n)
}

// ---------------------------------------------------------------------------

/// ⭐⭐⭐ **A marcha lê a distância euclidiana numa chapa — e o passeio por
/// arestas não.**
///
/// As duas metades são obrigatórias: sozinha, a primeira não distingue uma lei
/// boa de uma malha fácil.
///
/// ⚠️ **As barras saem da tabela medida** (`diag_o_erro_decai_com_a_distancia`),
/// e as duas grandezas dizem coisas diferentes:
/// * a **razão** é o que decide um corte, e ela DECAI com a distância
///   (`1,00` na 1.ª coroa, `1,05` a 16 arestas) porque o erro da marcha é de
///   **primeira ordem**;
/// * o erro **absoluto** SATURA abaixo de uma aresta (`0,96 h` a 20 arestas), e
///   é isso que garante que ele não cresce com o tamanho do pincel.
///
/// ⛔ O passeio por arestas não tem nem uma nem outra: ele fica em **`1,41` a
/// TODA distância** — um erro relativo que nunca decai.
#[test]
fn numa_chapa_de_quads_a_marcha_e_redonda_e_o_passeio_por_arestas_nao() {
    let mesh = chapa_de_quads();
    let semente = (N / 2 * N + N / 2) as u32;
    let tecto = 0.7;

    let mut g = Geodesica::default();
    g.marcha(&mesh, semente, tecto);
    let (min_m, max_m, abs_m, n) = na_faixa_da_decisao(&mesh, &|v| g.distancia(v), semente, tecto);

    let arestas = passeio_por_arestas(&mesh, semente);
    let (_, max_a, _, _) = na_faixa_da_decisao(&mesh, &|v| arestas[v as usize], semente, tecto);

    assert!(n > 200, "a faixa da decisao tem de ter populacao ({n})");
    assert!(
        max_m <= BARRA_DA_RAZAO && min_m >= 0.97,
        "a MARCHA devia ler a distancia euclidiana: razao em [{min_m:.4}, {max_m:.4}]"
    );
    assert!(
        abs_m < 1.0,
        "o erro ABSOLUTO da marcha devia saturar abaixo de uma aresta: {abs_m:.3} h"
    );
    // ⚠️ O controlo: numa grelha de quads a diagonal NÃO é aresta, logo o
    // passeio tem de ir por dois lados e infla `√2`. Sem esta metade, um gate
    // verde nao afirma que a marcha CUROU alguma coisa.
    assert!(
        max_a >= 1.30,
        "o CONTROLO devia inflar (o passeio por arestas nao tem a diagonal): {max_a:.4} \
         -- ou a fixtura deixou de conter o fenomeno"
    );
}

/// ⭐⭐ **E numa malha de TRIÂNGULOS também.**
///
/// ⚠️ **A malha triangulada por LEQUE é ANISOTRÓPICA, e a medição diz onde:** a
/// diagonal do leque é aresta (`1,00`) e a ANTI-diagonal não é nem aresta nem
/// canto de face nenhuma — para lá chegar a frente atravessa DOIS triângulos e
/// lê `1,207` na 1.ª coroa. *Isso é uma propriedade daquela malha, não da lei*,
/// e por isso o gate mede a faixa da decisão, onde ela já decaiu.
#[test]
fn numa_chapa_de_triangulos_a_marcha_continua_redonda() {
    let mesh = chapa_de_triangulos();
    let semente = (N / 2 * N + N / 2) as u32;
    let tecto = 0.7;
    let mut g = Geodesica::default();
    g.marcha(&mesh, semente, tecto);
    let (min_m, max_m, abs_m, n) = na_faixa_da_decisao(&mesh, &|v| g.distancia(v), semente, tecto);
    let arestas = passeio_por_arestas(&mesh, semente);
    let (_, max_a, _, _) = na_faixa_da_decisao(&mesh, &|v| arestas[v as usize], semente, tecto);
    assert!(n > 200, "a faixa da decisao tem de ter populacao ({n})");
    assert!(
        max_m <= BARRA_DA_RAZAO && min_m >= 0.97,
        "a marcha devia ler a distancia euclidiana: razao em [{min_m:.4}, {max_m:.4}]"
    );
    assert!(
        abs_m < 1.0,
        "o erro ABSOLUTO da marcha devia saturar abaixo de uma aresta: {abs_m:.3} h"
    );
    // ⚠️ **Cada gate carrega o PRÓPRIO controlo.** Sem ele, uma marcha que
    // degenerasse em Dijkstra ficaria verde aqui no dia em que alguém afrouxasse
    // a barra — e o vale que a escolheu deixaria de estar à vista.
    assert!(
        max_a >= 1.30,
        "o CONTROLO devia inflar: {max_a:.4} -- ou a fixtura deixou de conter o fenomeno"
    );
}

/// ⭐⭐ **O LEQUE DA SEMENTE: a primeira coroa entra pela CORDA.**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Desligar o leque
/// passava em todos os outros, porque eles medem a **faixa da decisão**
/// (`[tecto/2, tecto]`) e ali o leque já não importa — o erro dele é de
/// primeira coroa e decai. *Uma régua posta onde a decisão acontece é cega ao
/// que acontece onde ela não acontece.*
///
/// Sem o leque, a diagonal de um quad é resolvida por DOIS lados e lê
/// `1,207 × h√2`; com ele, a corda entra directa e lê `1,000`. Num pincel
/// pequeno sobre malha grossa — `R` de duas ou três arestas — isso é o campo
/// INTEIRO.
#[test]
fn a_primeira_coroa_entra_pela_corda_e_nao_por_dois_lados() {
    let mesh = chapa_de_quads();
    let semente = (N / 2 * N + N / 2) as u32;
    let mut g = Geodesica::default();
    g.marcha(&mesh, semente, 4.0 * H);
    let pos = mesh.positions();
    let s = pos[semente as usize];
    // A diagonal do quad: o vértice a `h√2` que NÃO é vizinho de aresta.
    let diagonal = ((N / 2 + 1) * N + N / 2 + 1) as u32;
    let exacto = (pos[diagonal as usize][0] - s[0]).hypot(pos[diagonal as usize][1] - s[1]);
    let medido = g.distancia(diagonal);
    assert!(
        g.alcanca(diagonal),
        "a marcha nem alcancou a diagonal da primeira coroa"
    );
    let razao = medido / exacto;
    assert!(
        razao <= 1.001,
        "a diagonal da 1.a coroa devia entrar pela CORDA e leu {razao:.4} x o exacto          ({medido:.5} contra {exacto:.5}) -- o leque da semente nao esta' a armar"
    );
}

/// ⛔⛔ **A ARMADILHA DOS CANTOS, medida em vez de acreditada.**
///
/// Se o terceiro ponto viesse da lista de ADJACÊNCIA em vez da FACE, numa malha
/// de quads **nenhuma** actualização de triângulo aconteceria (dois vizinhos de
/// um vértice nunca são vizinhos entre si) e a marcha degeneraria em Dijkstra
/// **em silêncio**. Este gate mede a diferença entre as duas saídas: se ela
/// desaparecer, é porque a travessia deixou de correr.
#[test]
fn a_travessia_de_face_de_facto_corre_numa_malha_de_quads() {
    let mesh = chapa_de_quads();
    let semente = (N / 2 * N + N / 2) as u32;
    let tecto = 0.7;
    let mut g = Geodesica::default();
    g.marcha(&mesh, semente, tecto);
    let arestas = passeio_por_arestas(&mesh, semente);
    let mut melhorados = 0usize;
    for v in 0..mesh.vert_count() as u32 {
        if g.alcanca(v) && g.distancia(v) < arestas[v as usize] - 1e-4 {
            melhorados += 1;
        }
    }
    assert!(
        melhorados > 200,
        "so' {melhorados} vertices ficaram melhores que o passeio por arestas -- \
         a travessia de FACE nao esta' a correr (os cantos vieram da adjacencia?)"
    );
}

/// ⭐ **O tecto corta, e o que ficou de fora lê `∞`.**
#[test]
fn a_marcha_para_no_tecto() {
    let mesh = chapa_de_quads();
    let semente = (N / 2 * N + N / 2) as u32;
    let tecto = 0.30;
    let mut g = Geodesica::default();
    g.marcha(&mesh, semente, tecto);
    let pos = mesh.positions();
    let s = pos[semente as usize];
    let (mut dentro, mut fora) = (0usize, 0usize);
    for (v, p) in pos.iter().enumerate() {
        let exacto = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2)).sqrt();
        if exacto <= tecto * 0.9 {
            dentro += 1;
            assert!(
                g.alcanca(v as u32),
                "o vertice {v} esta' a {exacto:.3} e o tecto e' {tecto:.3} -- a marcha \
                 devia te-lo alcancado"
            );
        } else if exacto > tecto {
            // ⚠️⚠️ **A barra é o TECTO e não `1,3 × tecto`, e a diferença é uma
            // MUTAÇÃO SOBREVIVENTE.** Numa chapa a marcha nunca lê MENOS que o
            // exacto (a corda do leque é exacta no plano, e a travessia
            // sobrestima), logo *alcançado ⇒ exacto ≤ tecto* é uma implicação
            // exacta, não uma folga. Com a barra em `1,3` havia uma banda cega
            // de um anel — e é exactamente um anel que vaza quando o tecto
            // deixa de cortar na ENTRADA e passa a cortar só na saída da fila.
            fora += 1;
            assert!(
                !g.alcanca(v as u32),
                "o vertice {v} esta' a {exacto:.4}, ALEM do tecto {tecto:.4}, e a marcha \
                 alcancou-o ({:.4}) -- o tecto deixou de cortar na ENTRADA",
                g.distancia(v as u32)
            );
        }
    }
    assert!(
        dentro > 50 && fora > 50,
        "a fixtura tem de conter os dois lados do tecto ({dentro} dentro, {fora} fora)"
    );
}

/// ⛔ **A volta da época não devolve a malha inteira como alcançada.**
///
/// Uma vez a cada quatro bilhões de dabs, e impossível de reproduzir sem esta
/// porta. É a mesma cerca do [`crate::QueryScratch`].
#[test]
fn a_volta_da_epoca_nao_devolve_a_malha_inteira() {
    let mesh = chapa_de_quads();
    let semente = (N / 2 * N + N / 2) as u32;
    let mut g = Geodesica::default();
    g.marcha(&mesh, semente, 0.10);
    g.forcar_epoca_para_teste(u32::MAX);
    g.marcha(&mesh, semente, 0.10);
    let alcancados = (0..mesh.vert_count() as u32)
        .filter(|&v| g.alcanca(v))
        .count();
    assert!(
        alcancados > 4 && alcancados < mesh.vert_count() / 4,
        "depois do wrap a marcha alcancou {alcancados} de {} vertices",
        mesh.vert_count()
    );
}

/// ⚠️ **Uma semente fora de alcance, um tecto absurdo ou uma malha vazia não
/// estoiram** — os três caminhos de recusa, porque um pincel chega aqui com o
/// que o gesto lhe deu.
#[test]
fn as_recusas_nao_estoiram() {
    let mesh = chapa_de_quads();
    let mut g = Geodesica::default();
    g.marcha(&mesh, u32::MAX, 0.2);
    assert!(!g.alcanca(0));
    g.marcha(&mesh, 0, 0.0);
    assert!(!g.alcanca(0));
    g.marcha(&mesh, 0, f32::NAN);
    assert!(!g.alcanca(0));
}

/// Sonda: como é que o erro da marcha decai com a distância à semente.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_o_erro_decai_com_a_distancia() {
    for (nome, mesh) in [("quads", chapa_de_quads()), ("tris", chapa_de_triangulos())] {
        let semente = (N / 2 * N + N / 2) as u32;
        let tecto = 1.0;
        let mut g = Geodesica::default();
        g.marcha(&mesh, semente, tecto);
        let arestas = passeio_por_arestas(&mesh, semente);
        let pos = mesh.positions();
        let s = pos[semente as usize];
        println!("\n== {nome} (passo {H}) ==");
        println!(
            "{:>8} {:>10} {:>10} {:>10}",
            "arestas", "marcha", "|erro|/h", "arestas_dij"
        );
        for k in [1usize, 2, 4, 8, 12, 16, 20] {
            let alvo = k as f32 * H;
            let (mut pior_m, mut pior_abs, mut pior_a) = (0.0f32, 0.0f32, 0.0f32);
            for (v, p) in pos.iter().enumerate() {
                let e = ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2)).sqrt();
                if !(e > alvo - H * 0.51 && e < alvo + H * 0.51) {
                    continue;
                }
                let m = g.distancia(v as u32);
                if m.is_finite() {
                    pior_m = pior_m.max(m / e);
                    pior_abs = pior_abs.max((m - e).abs() / H);
                }
                pior_a = pior_a.max(arestas[v] / e);
            }
            println!("{k:>8} {pior_m:>10.4} {pior_abs:>10.3} {pior_a:>10.4}");
        }
    }
}
