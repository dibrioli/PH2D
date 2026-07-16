//! **A camada ACIMA da correspondência: qual contorno de A vira qual contorno de B.**
//!
//! O motor de [`crate::matching`] responde *"que PONTO de A vira que ponto de B"*, e responde bem —
//! mas ele responde sobre **um contorno**. Uma rosquinha tem dois, e a rosquinha é a saída típica
//! da booleana e do Shape Builder. Este módulo é o nível-1: parear os contornos entre si, para que
//! o nível-2 (a fase, as quinas, a virada) rode dentro de cada par, sem saber que existe um buraco.
//!
//! # O PAPEL de um contorno é a profundidade de aninhamento dele
//!
//! Parear o contorno de FORA de A com o BURACO de B é o desastre silencioso desta camada: no meio
//! do caminho os dois contornos se cruzam, o aninhamento inverte, e a forma vira do avesso. E não é
//! uma hipótese exótica — se o pareamento for por proximidade de centroide, é exatamente o que
//! acontece com duas rosquinhas concêntricas, onde os QUATRO contornos têm o MESMO centro.
//!
//! O papel de um contorno é quantos outros contornos da MESMA forma o contêm: `0` = o de fora,
//! `1` = buraco, `2` = ilha dentro do buraco. Sob [`FillRule::EvenOdd`] a paridade dessa contagem
//! é literalmente o que o renderer usa para decidir cheio/vazio — então parear profundidade com
//! profundidade não é uma preferência de estilo, é preservar o que a tela desenha.

use crate::{Outline, VecPath};
use ph2d_vec_scene::{FillRule, contains_point};

/// Um contorno de uma forma, com o papel dele.
pub(crate) struct Ring {
    pub(crate) outline: Outline,
    /// Quantos outros contornos DESTA forma contêm este. `0` = o de fora, `1` = buraco, `2` = ilha.
    pub(crate) depth: usize,
}

/// Os contornos de uma forma, cozidos, cada um com a profundidade dele.
///
/// A ordem é a do documento (o primário primeiro) — e ela é preservada de propósito: o passo
/// montado sai com `links[0]` como contorno primário, e o primário de A é o primário do passo.
///
/// Um contorno **degenerado** (menos de 2 vértices, ou comprimento nulo) é DESCARTADO, não
/// mantido como buraco vazio: ele não desenha nada na tela, e um contorno sem arco não tem
/// correspondência a oferecer. É a mesma decisão que [`Outline::of`] já toma para a forma inteira.
pub(crate) fn rings(path: &VecPath) -> Vec<Ring> {
    let cooked = path.cooked();
    // O índice do documento viaja junto: a profundidade é medida contra os contornos ORIGINAIS
    // (é `contains_point` quem coze, e ele é a única porta para "o ponto está dentro?").
    let live: Vec<(usize, Outline)> = (0..cooked.contour_count())
        .filter_map(|c| Outline::of_contour(&cooked, c).map(|o| (c, o)))
        .collect();

    let depths: Vec<usize> = live
        .iter()
        .map(|(c, o)| {
            live.iter()
                .filter(|(other, _)| other != c && contour_contains(path, *other, o))
                .count()
        })
        .collect();

    live.into_iter()
        .zip(depths)
        .map(|((_, outline), depth)| Ring { outline, depth })
        .collect()
}

/// O contorno `c` de `path` contém o contorno `inner`?
///
/// **Um ponto basta.** Num compound bem formado os contornos NÃO se cruzam (é o que faz o buraco
/// ser um buraco), então a resposta é a mesma para todo ponto de `inner` — e onde eles se cruzam,
/// "está contido" não tem resposta verdadeira para dar.
///
/// A pergunta vai para [`contains_point`], que é a porta que o hit-test e o renderer já usam. Um
/// teste de cruzamentos próprio aqui seria uma 2ª porta para "o ponto está dentro?", e duas portas
/// divergem — esta camada inteira existe por causa de uma delas.
fn contour_contains(path: &VecPath, c: usize, inner: &Outline) -> bool {
    let Some((verts, closed)) = path.contour(c) else {
        return false;
    };
    // O contorno `c` SOZINHO, como forma própria: é a pergunta "este anel contém o ponto?", e não
    // "a forma toda contém o ponto?" (que já embute os buracos, e é justamente o que se quer medir
    // aqui). Contorno único ⇒ as duas regras de preenchimento coincidem, então o default serve.
    let solo = VecPath {
        verts: verts.to_vec(),
        closed,
        ..VecPath::default()
    };
    let probe = inner.at(0.0);
    contains_point(&solo, [probe.x, probe.y])
}

/// O **centroide** do contorno, amostrado por ARCO.
///
/// Por arco, e não pela média das âncoras: âncora é parametrização, e picar uma aresta em 20
/// pedaços (geometria idêntica!) mudaria a resposta. É a mesma lição que o `normalized` do
/// [`crate::matching`] já pagou.
fn centroid(o: &Outline) -> [f64; 2] {
    let (mut x, mut y) = (0.0, 0.0);
    for k in 0..CENTROID_SAMPLES {
        let p = o.at(k as f64 / CENTROID_SAMPLES as f64);
        x += p.x;
        y += p.y;
    }
    let n = CENTROID_SAMPLES as f64;
    [x / n, y / n]
}

/// Amostras de arco do centroide. É a mesma resolução do varrimento de fase do
/// [`crate::matching`] — a pergunta ("onde está este contorno?") tem a mesma escala.
const CENTROID_SAMPLES: usize = 256;

/// O quanto um contorno teria de VIAJAR para virar o outro — o custo de nível-1.
///
/// É a distância entre centroides **ao quadrado**: o `squaredDistance` do flubber (`src/order.js`),
/// que é a única referência publicada a pontuar um pareamento de peças. Sem peso mágico: uma
/// distância, em unidades de mundo.
///
/// # [DECLARADO] O quadrado não tem gate, e a razão importa
///
/// Trocá-lo por `Σd` (a distância crua) **não derruba nenhum gate** — os dois custos escolhem o
/// mesmo pareamento em toda forma que sabemos desenhar. Eles só divergem numa configuração
/// construída (`Σd` prefere `0 + 5`; `Σd²` prefere `2,4 + 2,7`), e ali os dois têm defesa: o
/// quadrado espalha o movimento entre os buracos — a ponto de fazê-los **cruzar** —, e o linear
/// deixa um parado e manda o outro atravessar a forma.
///
/// Qual das duas o olho prefere é uma pergunta sem resposta publicada: a pesquisa que precedeu
/// este módulo achou **zero** implementações e **zero** artigos que pareiem contornos com buraco
/// (o Sederberg & Greenwood 1992 põe exatamente este problema no §6 *Future Work*). Sem
/// verdade-fundamental, a regra é portar a referência em vez de inventar a nossa versão, e o
/// flubber é a referência. Fabricar um gate aqui seria fabricar a resposta — o gate afirmaria a
/// minha intuição estética com cara de medição. Um mutante sobrevive a isto, e **está certo**.
fn travel(a: &Ring, b: &Ring) -> f64 {
    let (ca, cb) = (centroid(&a.outline), centroid(&b.outline));
    let (dx, dy) = (ca[0] - cb[0], ca[1] - cb[1]);
    dx * dx + dy * dy
}

/// **O pareamento de nível-1.** Uma entrada por link: `(contorno de A, contorno de B)`.
///
/// `None` de um lado = o contorno não tem par e nasce de / colapsa num PONTO (o buraco abre ou
/// fecha). É uma mudança de TOPOLOGIA, e é o comportamento certo: uma rosquinha virando disco tem
/// de fechar o buraco em algum lugar.
///
/// A ordem é a de A (o primário de A é o primário do passo), com os contornos órfãos de B no fim.
///
/// # A profundidade primeiro, a distância depois
///
/// O par só pode sair da **mesma profundidade** (§ o cabeçalho do módulo): é o que impede o
/// contorno de fora de A de casar com o buraco de B e virar a forma do avesso. Dentro de uma
/// profundidade, o pareamento é o de menor viagem TOTAL — [`best_order`], o branch-and-bound
/// exato do flubber.
///
/// No caso que importa não há o que escolher: uma rosquinha tem UM contorno em cada profundidade,
/// e o par é forçado. A busca só decide entre buracos irmãos.
pub(crate) fn pair(ra: &[Ring], rb: &[Ring]) -> Vec<(Option<usize>, Option<usize>)> {
    let mut mate: Vec<Option<usize>> = vec![None; ra.len()];
    let mut taken = vec![false; rb.len()];

    // Uma profundidade de cada vez: o pareamento nunca cruza papéis, então cada classe é um
    // problema de atribuição independente — e pequeno (os buracos de uma forma são poucos).
    let depths: Vec<usize> = {
        let mut d: Vec<usize> = ra.iter().map(|r| r.depth).collect();
        d.sort_unstable();
        d.dedup();
        d
    };
    for depth in depths {
        let ia: Vec<usize> = idx_at(ra, depth);
        let ib: Vec<usize> = idx_at(rb, depth);
        if ia.is_empty() || ib.is_empty() {
            continue;
        }
        let cost: Vec<Vec<f64>> = ia
            .iter()
            .map(|&i| ib.iter().map(|&j| travel(&ra[i], &rb[j])).collect())
            .collect();
        for (row, j) in best_order(&cost).into_iter().enumerate() {
            if let Some(j) = j {
                mate[ia[row]] = Some(ib[j]);
                taken[ib[j]] = true;
            }
        }
    }

    let mut out: Vec<(Option<usize>, Option<usize>)> =
        (0..ra.len()).map(|i| (Some(i), mate[i])).collect();
    out.extend(
        (0..rb.len())
            .filter(|j| !taken[*j])
            .map(|j| (None, Some(j))),
    );
    out
}

/// Os índices dos anéis de `rings` na profundidade `depth`.
fn idx_at(rings: &[Ring], depth: usize) -> Vec<usize> {
    (0..rings.len())
        .filter(|&i| rings[i].depth == depth)
        .collect()
}

/// **O pareamento de menor custo TOTAL** — o `bestOrder` do flubber (`src/order.js`), porte direto.
///
/// Devolve, para cada linha (contorno de A), a coluna (contorno de B) com que ela casa; `None`
/// quando não há coluna para ela (mais linhas que colunas — o buraco que fecha).
///
/// # Por que exato, e não guloso
///
/// A 1ª versão era gulosa na ordem do documento, e o guloso erra o clássico: dois buracos quase
/// equidistantes de dois alvos, onde a melhor escolha LOCAL do primeiro força uma péssima para o
/// segundo. O flubber — a única referência publicada que pontua um pareamento de peças — faz
/// branch-and-bound **exato**, e dentro de uma profundidade os números são minúsculos, então o
/// exato é essencialmente de graça. Portá-lo é estritamente melhor que a minha versão, e por isso
/// ele está aqui.
///
/// A poda (`custo_parcial >= melhor` ⇒ corta) é **admissível**: todo custo é ≥ 0, então um
/// prefixo nunca fica mais barato ao crescer — a resposta é o ótimo, não uma aproximação.
///
/// # A degradação do flubber é um bug, e ela NÃO foi portada
///
/// Acima de 8 peças o flubber devolve a **ordem identidade** — isto é, desiste do pareamento e não
/// conta a ninguém. Aqui, acima de [`EXACT_MAX`], o caminho é o guloso: uma resposta pior que o
/// ótimo, mas ainda uma resposta *sobre a geometria*. Uma forma com tantos buracos na MESMA
/// profundidade não existe na prática (uma rosquinha tem um); o limite existe porque `n!` existe.
fn best_order(cost: &[Vec<f64>]) -> Vec<Option<usize>> {
    let (n, m) = (cost.len(), cost.first().map_or(0, Vec::len));
    if n > EXACT_MAX || m > EXACT_MAX {
        return greedy_order(cost);
    }
    let mut used = vec![false; m];
    let mut cur: Vec<Option<usize>> = vec![None; n];
    let mut best: Option<(f64, Vec<Option<usize>>)> = None;
    walk(cost, 0, 0.0, &mut used, &mut cur, &mut best);
    best.map_or_else(|| greedy_order(cost), |(_, order)| order)
}

/// Um passo do branch-and-bound: escolhe a coluna da linha `row` e desce.
fn walk(
    cost: &[Vec<f64>],
    row: usize,
    acc: f64,
    used: &mut [bool],
    cur: &mut Vec<Option<usize>>,
    best: &mut Option<(f64, Vec<Option<usize>>)>,
) {
    if best.as_ref().is_some_and(|(b, _)| acc >= *b) {
        return; // a poda: este prefixo já custa o que a melhor solução INTEIRA custa
    }
    if row == cost.len() {
        *best = Some((acc, cur.clone()));
        return;
    }
    let free: Vec<usize> = (0..used.len()).filter(|&j| !used[j]).collect();
    if free.is_empty() {
        // Mais linhas que colunas: as que sobram fecham (ficam `None`) e o ramo termina.
        for slot in cur.iter_mut().skip(row) {
            *slot = None;
        }
        if best.as_ref().is_none_or(|(b, _)| acc < *b) {
            *best = Some((acc, cur.clone()));
        }
        return;
    }
    for j in free {
        used[j] = true;
        cur[row] = Some(j);
        walk(cost, row + 1, acc + cost[row][j], used, cur, best);
        used[j] = false;
        cur[row] = None;
    }
}

/// O caminho de escape acima de [`EXACT_MAX`]: cada linha fica com a coluna livre mais barata.
fn greedy_order(cost: &[Vec<f64>]) -> Vec<Option<usize>> {
    let m = cost.first().map_or(0, Vec::len);
    let mut used = vec![false; m];
    cost.iter()
        .map(|row| {
            let pick = (0..m).filter(|&j| !used[j]).min_by(|&x, &y| {
                row[x]
                    .partial_cmp(&row[y])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if let Some(j) = pick {
                used[j] = true;
            }
            pick
        })
        .collect()
}

/// Até quantos contornos na MESMA profundidade a busca é exaustiva.
///
/// É o mesmo limite do flubber, pela mesma razão: `n!` cresce, e a poda não muda isso no pior
/// caso. Oito buracos na mesma profundidade duma forma já é uma forma que ninguém desenhou.
const EXACT_MAX: usize = 8;

/// **Para onde colapsa um contorno sem par** — o ponto de onde o buraco nasce, ou onde ele morre.
///
/// É o centroide do contorno de FORA da forma do OUTRO lado, e não o do próprio contorno órfão.
/// A diferença é visível: o ponto do próprio contorno fica parado **onde a forma estava**, e a
/// forma vai embora — o buraco encolheria saindo pela borda, em vez de fechar dentro dela. Com o
/// centroide do outro lado, o buraco viaja junto com a forma e fecha no meio dela.
///
/// `None` se a outra forma não tem contorno externo (não acontece: toda forma viva tem um de
/// profundidade 0 — mas quem depende disso não adivinha).
pub(crate) fn collapse_point(other: &[Ring]) -> Option<[f64; 2]> {
    other
        .iter()
        .find(|r| r.depth == 0)
        .map(|r| centroid(&r.outline))
}

/// A regra de preenchimento de um passo com `n` contornos.
///
/// # Multi-contorno é SEMPRE `EvenOdd`, e isto não é preferência
///
/// A correspondência tem liberdade de **inverter o sentido de percurso** de um contorno (o
/// `search` testa os dois e fica com o de menor custo — é o que faz uma forma importada ao
/// contrário blendar sem colapsar). Sob [`FillRule::NonZero`] o sentido É o significado: inverter
/// o percurso de um buraco o **preenche**. Ou seja, o motor não tem como honrar `NonZero` num
/// compound sem abrir mão da correspondência — e é exatamente por essa robustez que a booleana e o
/// `make_compound` já escrevem `EvenOdd` em tudo que produzem.
///
/// Contorno **único**: as duas regras coincidem, e o default histórico (`NonZero`) fica — o passo
/// de um blend simples continua byte-idêntico ao de antes deste módulo existir.
pub(crate) fn fill_rule_for(n: usize) -> FillRule {
    if n > 1 {
        FillRule::EvenOdd
    } else {
        FillRule::NonZero
    }
}
