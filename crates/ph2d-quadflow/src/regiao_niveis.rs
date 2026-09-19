//! ⛔⛔⛔⛔ **A MANCHA POR NÍVEIS — o caminho CONSTRUÍDO, MEDIDO e RECUSADO.**
//!
//! Este módulo é o registo de uma recusa, não um caminho do produto. Ele nasceu
//! por ordem do dono (*«pagar o cálculo por níveis»*, 20/09), mediu **pior que
//! um nível em todas as colunas**, e fica aqui porque **a medição dele é a
//! recusa** — apagá-lo levava a medição junto, que é a lei que o `§5.0` deste
//! repo escreve sobre o que foi medido e rejeitado.
//!
//! O mecanismo, as três pernas da recusa e as duas explicações minhas que
//! caíram antes da certa estão no cabeçalho da [`campos_em_niveis`].
//!
//! ⚠️ **Ele saiu da [`crate::regiao`] por TECTO DE LOC** (`945` contra `700`),
//! e o corte é por responsabilidade: ali fica o domínio que o produto corre,
//! aqui fica a classe que ele não corre.

use core::cmp::Ordering;

use super::{Mancha, norm, orientacao_semeada, posicao_da_mancha_com};

/// ⭐⭐⭐ **A MANCHA COMO NÍVEL `0` DE UMA HIERARQUIA.**
///
/// ⚠️ **Ela não converte nada — ela RENOMEIA.** Um [`Mancha`] já tem as quatro
/// colunas que um [`crate::hierarchy::Level`] pede (posições, normais, áreas e a
/// adjacência PONDERADA), pela mesma razão que ele as tem: as duas foram
/// escritas para alimentar o mesmo par de núcleos. O `parent` fica vazio porque
/// é a pilha que o preenche.
#[must_use]
pub fn nivel_da_mancha(m: &Mancha) -> crate::hierarchy::Level {
    crate::hierarchy::Level {
        positions: m.pos.clone(),
        normals: m.nrm.clone(),
        areas: m.areas.clone(),
        adjacency: m.adj.clone(),
        parent: Vec::new(),
    }
}

/// ⛔⛔⛔⛔ **OS DOIS CAMPOS DA MANCHA POR NÍVEIS — CONSTRUÍDO, MEDIDO e
/// RECUSADO. Não tem chamador de produto, e não deve ter.**
///
/// A alternativa hierárquica à dupla [`orientacao_semeada`] +
/// [`posicao_da_mancha_com`]: a mesma lei, os mesmos núcleos, resolvida do
/// nível mais GROSSO para o mais fino. Ela existe porque o dono a **ordenou**
/// (*«pagar o cálculo por níveis»*, 20/09) e fica aqui porque a medição dela é
/// o registo da recusa — *apagá-la levava a medição junto*.
///
/// # ⛔ A RECUSA, com as três pernas
///
/// **1 — o CONTROLO, que é o que torna as outras duas legíveis.** Com
/// `mais_grosso` acima do tamanho da pegada a pilha tem um nível só, e esta
/// porta devolve a lei que shipa **ao dígito em todas as colunas e nos quatro
/// rumos** (`grade 64,22` · `vinco 0,994`/`2,620` · `fil 20`/`43`/`70` ·
/// `8 025` vértices). ⇒ *a implementação está certa e a tabela mede o programa
/// certo* (gate `uma_pilha_de_um_nivel_e_a_lei_de_hoje`).
///
/// **2 — ela mede PIOR, e não por pouco** (média dos quatro rumos, pela porta
/// do produto):
///
/// | classe | grade | vinco p90 | **fil p50** | **fil p90** |
/// |---|---|---|---|---|
/// | **um nível** (shipa) | **`64,03 %`** | **`2,724°`** | **`17,5`** | **`47,0`** |
/// | níveis (`24`) | `52,73 %` | `3,877°` | `3,2` | `11,0` |
/// | níveis (`8`) | `53,6 %` | `3,25°` | `3,2` | `11,5` |
///
/// ⛔ E **a saída barata está fechada**: com `2`, `4`, `8`, `16` e `32`
/// varreduras a hierarquia fica **plana** em `52`–`55 %` de grade e `fil90`
/// `11`–`13`, enquanto um nível vai a `65,17 %` e `68,5`. *Não é afinação.*
///
/// **3 — o MECANISMO: ela propõe uma retícula que NÃO É a que a malha tem.**
/// Medido sobre a malha a meio de um traço já arrumado (`891` vértices), o
/// **pedido** — a distância de cada vértice ao nó que a lei lhe aponta — lê
/// `p50 0,022` passos com um nível e **`0,375`** com a hierarquia. E `0,375` é
/// o **máximo** que duas grades do mesmo passo podem discordar: meia célula é
/// `0,5`, e a distância de um ponto qualquer ao nó mais perto de uma grade
/// rígida tem `p50 ≈ 0,40` por simulação. ⇒ *a fase que a hierarquia propõe não
/// tem relação nenhuma com a que a malha tem.*
///
/// ⭐⭐⭐ **E é isso que parte as fileiras, porque um pincel aplica a lei DEZENAS
/// de vezes sobre uma pegada que ANDA.** O emparelhamento do `coarsen` é outro
/// a cada dab (a pegada mudou), logo cada dab propõe uma grade sem relação com a
/// do anterior e **nada se acumula**. Um nível semeia cada vértice consigo
/// próprio, logo propõe sempre a grade de que a malha está mais perto — e os
/// dabs **reforçam-se**. *As fileiras de um pincel são feitas de acumulação ao
/// longo do traço, não de coerência dentro de um dab.*
///
/// ⛔⛔ **DUAS explicações minhas caíram ANTES desta, e ficam escritas para não
/// voltarem:**
///
/// - a **saturação da FRANJA** — refutada: `11 %` no nível `0` e `52 %` no mais
///   grosso (`891/101 · 482/71 · 259/51 · 144/36 · 80/28 · 42/18 · 23/12`), os
///   níveis grossos resolvem. Sonda: [`retrato_da_pilha`];
/// - a **aritmética da fase rígida** como custo *por construção* — refutada
///   pela própria sonda: numa esfera VIRGEM as duas classes pedem `0,400` e
///   `0,413`. *O `0,022` de um nível não é uma propriedade da lei, é o que
///   sobra depois de o traço já a ter aplicado doze vezes.*
/// - e a **convergência por repetição** — refutada: a retícula sozinha, sobre
///   uma conectividade fixa, não converge em classe nenhuma (`0,400 → 0,367` em
///   oito passagens, porque mover os vértices re-deriva o consenso). Quem
///   converge é o **PAR** retícula + dyntopo.
///
/// ⭐⭐⭐ **A leitura que fica, e que vale para o que vier:** *um extractor
/// precisa de fase GLOBAL porque emite malha nova; um PINCEL precisa de uma fase
/// que ele possa REPETIR, porque empurra os mesmos vértices dab após dab* — e
/// uma fase decidida no topo de uma pilha construída sobre uma pegada que anda
/// não é repetível.
///
/// ⚠️ **O gate do mecanismo vive em `scenes_pente_grelha_tests.rs`**
/// (`a_hierarquia_propoe_outra_reticula`) e não ao lado desta função: a malha
/// que o revela só o TRAÇO a produz. Aqui fica o **controlo**
/// (`uma_pilha_de_um_nivel_e_a_lei_de_hoje`), que é o que prova que a tabela
/// acima mede esta lei e não um defeito meu.
///
/// # Porque a hierarquia existe na crate, e porque ela está certa LÁ
///
/// ⛔⛔ **Um Gauss-Seidel de UM nível não tem acoplamento de longo alcance**, e
/// isso está medido *nesta crate* desde que a hierarquia nasceu (o cabeçalho do
/// [`crate::hierarchy`]: *«o resíduo entre vizinhos fica em 0,205 célula, imóvel
/// entre 32 e 2 048 varreduras»*). Uma varredura propaga **uma aresta**, logo
/// uma pegada de `d` arestas de diâmetro precisaria de `d` varreduras só para a
/// informação atravessar — e a medição do plano diz que nem assim: a malha fina
/// é um ponto fixo da média. É **exactamente** por isto que a referência
/// resolve a posição com multigrid, e é a razão pela qual esta porta não é uma
/// optimização mas uma mudança de alcance.
///
/// # ⚠️ A FRANJA é uma condição de DIRICHLET, e ela vale em TODOS os níveis
///
/// A mancha é um recorte: o que lhe fica à volta não é resolvido, e a franja é
/// o que segura a solução contra ele ([`Mancha::fronteira`]). Num laço de um
/// nível isso é um `fixos` e acabou; aqui há duas metades e **as duas são
/// obrigatórias**:
///
/// 1. um vértice grosso é **fixo se QUALQUER filho seu for** — conservador de
///    propósito, porque metade-fixo não existe. ⚠️ **Esta metade é uma escolha
///    DECLARADA e não tem gate, e a prova de mutação diz porquê:** apagá-la
///    deixa os `12` gates da mancha verdes, porque quem segura a condição de
///    fronteira no nível que o produto lê é a metade (2). *Uma linha que a
///    mutação não mata é comentário com sintaxe de código* — ela fica porque
///    **remover código de um caminho recusado sem re-medir a tabela da recusa
///    seria mudar o sujeito dela**, não porque seja lei;
/// 2. **a PROLONGAÇÃO reescreve tudo**, franja incluída, logo os fixos são
///    **repostos ao valor de semente** a seguir a cada prolongação — senão a
///    condição de fronteira desaparece do nível fino, que é o único onde o
///    produto a lê.
///
/// ⚠️ **A ESCALA é a mesma em todos os níveis por construção.** A
/// [`solve_fields_with`] tem de fazer a média dos filhos porque lá a escala é um
/// campo; aqui o `passo` é um escalar da pegada inteira — *é a mesma grade vista
/// de longe*, que é a lei que aquele comentário já escreve.
///
/// ⛔ **A prolongação NÃO arredonda à retícula do filho**, e a razão está escrita
/// na [`solve_fields_with`]: arredondar ali congela o campo num nó de grade
/// antes de o nível se acomodar, e cada nível herda o degrau do anterior.
#[must_use]
pub fn campos_em_niveis(
    m: &Mancha,
    direccao: [f32; 3],
    passo: f32,
    varreduras: usize,
    mais_grosso: usize,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    if m.is_empty() || norm(direccao) <= 0.0 || passo.partial_cmp(&0.0) != Some(Ordering::Greater) {
        return (Vec::new(), Vec::new());
    }

    let h = crate::hierarchy::Hierarchy::from_level(nivel_da_mancha(m), mais_grosso.max(1));

    // Os fixos de cada nível: no `0` é a franja; acima, qualquer filho fixo.
    let mut fixos: Vec<Vec<bool>> = Vec::with_capacity(h.depth());
    fixos.push(m.fronteira.clone());
    for l in 0..h.depth().saturating_sub(1) {
        let parent = &h.level(l).parent;
        let mut acima = vec![false; h.level(l + 1).len()];
        for (v, &p) in parent.iter().enumerate() {
            acima[p as usize] |= fixos[l][v];
        }
        fixos.push(acima);
    }

    let mut dirs: Vec<[f32; 3]> = Vec::new();
    let mut pos: Vec<[f32; 3]> = Vec::new();

    for l in h.coarse_to_fine() {
        let lv = h.level(l);
        // ⚠️ O MESMO `passo` em todos os níveis — ver o cabeçalho. Um nível
        // grosso tem menos vértices, nunca uma grade mais grossa.
        let escalas = vec![passo; lv.len()];
        if l + 1 == h.depth() {
            dirs = lv
                .normals
                .iter()
                .map(|&n| crate::orientation::project_tangent(direccao, n))
                .collect();
            pos = lv.positions.clone();
        } else {
            let parent = &lv.parent;
            let (de_cima_d, de_cima_p) = (dirs, pos);
            dirs = (0..lv.len())
                .map(|v| {
                    crate::orientation::project_tangent(
                        de_cima_d[parent[v] as usize],
                        lv.normals[v],
                    )
                })
                .collect();
            pos = (0..lv.len())
                .map(|v| {
                    let o = de_cima_p[parent[v] as usize];
                    let (n, vp) = (lv.normals[v], lv.positions[v]);
                    let d = n[0].mul_add(
                        o[0] - vp[0],
                        n[1].mul_add(o[1] - vp[1], n[2] * (o[2] - vp[2])),
                    );
                    [
                        d.mul_add(-n[0], o[0]),
                        d.mul_add(-n[1], o[1]),
                        d.mul_add(-n[2], o[2]),
                    ]
                })
                .collect();
            // ⚠️ A metade (2) do cabeçalho: a prolongação passou por cima da
            // condição de fronteira, e ela repõe-se AQUI.
            for v in 0..lv.len() {
                if fixos[l][v] {
                    dirs[v] = crate::orientation::project_tangent(direccao, lv.normals[v]);
                    pos[v] = lv.positions[v];
                }
            }
        }

        crate::orientation::smooth_on_fixed(
            &mut dirs,
            &lv.normals,
            &lv.adjacency,
            &fixos[l],
            varreduras,
        );
        crate::position::smooth_on_fixed(
            &mut pos,
            &lv.positions,
            &lv.normals,
            &dirs,
            &escalas,
            &lv.adjacency,
            &fixos[l],
            varreduras,
        );
    }

    (dirs, pos)
}

/// ⛔⛔ **SONDA — o retrato da pilha de uma mancha: quantos níveis, que tamanho,
/// que fracção PREGADA, e quanto a retícula pede a cada vértice.**
///
/// Ela existe porque a hierarquia mediu **pior** que um nível (20/09) com o
/// controlo de um nível a bater ao dígito, e *«é pior» sem mecanismo é um
/// número*. As duas explicações que ela separa:
///
/// 1. **saturação da FRANJA** — se «fixo se qualquer filho for» pregar quase
///    tudo lá em cima, o nível grosso não resolve nada e a prolongação só
///    espalha a semente;
/// 2. **a FASE vem de LONGE** — o alvo de cada vértice é o ponto de retícula
///    mais perto DELE, e uma origem herdada de um vértice grosso a várias
///    arestas de distância pede um deslocamento grande, que a cerca da forma e
///    a queda do pincel depois recusam ou atenuam.
///
/// Devolve `(por nível: (vértices, fixos), pedido p50 em passos, pedido p90)`.
#[must_use]
pub fn retrato_da_pilha(
    m: &Mancha,
    direccao: [f32; 3],
    passo: f32,
    varreduras: usize,
    mais_grosso: usize,
) -> (Vec<(usize, usize)>, f64, f64) {
    let h = crate::hierarchy::Hierarchy::from_level(nivel_da_mancha(m), mais_grosso.max(1));
    let mut fixos: Vec<Vec<bool>> = vec![m.fronteira.clone()];
    for l in 0..h.depth().saturating_sub(1) {
        let parent = &h.level(l).parent;
        let mut acima = vec![false; h.level(l + 1).len()];
        for (v, &p) in parent.iter().enumerate() {
            acima[p as usize] |= fixos[l][v];
        }
        fixos.push(acima);
    }
    let niveis: Vec<(usize, usize)> = (0..h.depth())
        .map(|l| (h.level(l).len(), fixos[l].iter().filter(|f| **f).count()))
        .collect();

    let (_d, pos) = if mais_grosso >= m.len() {
        let d = orientacao_semeada(m, direccao, varreduras);
        let g = posicao_da_mancha_com(m, &d, passo, varreduras, false);
        (d, g)
    } else {
        campos_em_niveis(m, direccao, passo, varreduras, mais_grosso)
    };
    let mut pedidos: Vec<f64> = Vec::new();
    for i in 0..m.len() {
        if m.fronteira[i] || i >= pos.len() {
            continue;
        }
        let d = [
            pos[i][0] - m.pos[i][0],
            pos[i][1] - m.pos[i][1],
            pos[i][2] - m.pos[i][2],
        ];
        pedidos.push(f64::from(norm(d) / passo));
    }
    pedidos.sort_by(f64::total_cmp);
    let q = |f: f64| -> f64 {
        if pedidos.is_empty() {
            return 0.0;
        }
        pedidos[(((pedidos.len() - 1) as f64) * f).round() as usize]
    };
    (niveis, q(0.50), q(0.90))
}
