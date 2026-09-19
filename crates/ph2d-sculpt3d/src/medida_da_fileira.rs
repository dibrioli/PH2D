//! ⭐⭐⭐⭐ **A RÉGUA QUE O OLHO USA — o COMPRIMENTO DA FILEIRA.**
//!
//! # ⛔⛔⛔⛔ Porque ela existe, e porque não existia
//!
//! Esta cena foi reprovada pelo dono **três vezes**, e de cada vez a régua que
//! a aprovava estava um nível abaixo do que ele vê:
//!
//! | report | a régua que estava verde | o que ela não via |
//! |---|---|---|
//! | *«pouca ou nenhuma diferença»* | `Q`, a **MÉDIA** de `cos 4α` | meia dúzia de arestas perfeitas no meio de milhares paradas |
//! | *«pior que o original»* | a **CONTAGEM** por baldes de `15°` | o RELEVO — nenhuma régua media a luz |
//! | *«não percebo nenhuma vantagem»* | a contagem outra vez, a `65 %` | **a CONTINUIDADE** |
//!
//! ⇒ *uma aresta alinhada não é uma fileira.* A contagem responde *«esta aresta
//! aponta para onde a mão andou?»* uma a uma, e o que o artista chama **edge
//! flow** é outra grandeza: **quantas arestas seguidas continuam a MESMA
//! linha**. Uma malha pode ter `65 %` das arestas alinhadas em **retalhos** de
//! meia dúzia, e a olho isso não é grade nenhuma — foi exactamente o que a
//! imagem mostrou em 2026-09-20.
//!
//! # Como
//!
//! 1. colhem-se as arestas da faixa que correm a menos de [`ALINHADA`] da
//!    direcção local do traço;
//! 2. encadeiam-se: de uma aresta passa-se à vizinha que **melhor continua** a
//!    linha, se o desvio for menor que [`CONTINUA`];
//! 3. devolve-se a distribuição dos comprimentos das cadeias, **em arestas**.
//!
//! ⚠️ **O percurso é GULOSO e cada aresta entra numa cadeia só.** Não é a
//! partição óptima — é a **determinística**, e é a que descreve o que o olho
//! segue: ele também não volta atrás.
//!
//! # ⚠️ O que «bom» significa aqui, e de onde sai
//!
//! O chão não é `0`: numa malha isotrópica há sempre pares de arestas que por
//! acaso se continuam. É por isso que toda leitura desta régua se lê **contra o
//! controlo** — a mesma peça sem pentear — e nunca sozinha.

use ph2d_mesh::Mesh;

/// Quanto uma aresta pode desviar da direcção do traço e ainda contar como
/// parte de uma fileira.
///
/// ⚠️ **É o mesmo `15°` do balde zero da [`crate::medida_do_pente::grade_da_faixa`]**,
/// e de propósito: as duas réguas têm de falar da mesma população, senão a
/// diferença entre elas deixa de ser *«contagem contra continuidade»* e passa a
/// ser *«duas definições de alinhado»*.
pub const ALINHADA: f64 = 15.0;

/// Quanto duas arestas seguidas podem divergir e ainda ser a mesma linha.
///
/// ⛔⛔⛔ **A primeira redacção pôs `30°` aqui, e ele era INERTE POR CONSTRUÇÃO
/// — uma mutação sobreviveu e a varredura provou-o.** Duas arestas que estão
/// **cada uma** a menos de [`ALINHADA`] da MESMA direcção local diferem no
/// máximo `2 × ALINHADA = 30°` ⇒ *o teste nunca podia recusar nada*. A
/// varredura lê exactamente o mesmo em `180°` e em `30°`:
///
/// | `CONTINUA` | sem pente (`p50`/`p90`) | com retícula (`p50`/`p90`) |
/// |---|---|---|
/// | `180°` | `2` / `9` | `10` / **`53`** |
/// | `30°` | `2` / `9` | `10` / **`53`** |
/// | **`20°`** | `2` / `8` | `10` / **`40`** |
/// | `15°` | `2` / `8` | `10` / `40` |
/// | `10°` | `1` / `7` | `6` / `37` |
/// | `5°` | `1` / `6` | `3` / `21` |
///
/// ⭐ **`20°` é o primeiro valor que MORDE**, e mantém a separação entre os dois
/// lados (`on/off` do `p50` fica em `5,0×`, como em `180°`). Abaixo dele começa
/// a cortar o sinal: a `10°` a mediana da retícula cai de `10` para `6`.
///
/// *Uma linha que a mutação não consegue matar não é lei, é comentário com
/// sintaxe de código* — e a fixtura que a mata é o zigue-zague do
/// `uma_fileira_em_ziguezague_nao_e_uma_linha`.
pub const CONTINUA: f64 = 20.0;

/// **O COMPRIMENTO DAS FILEIRAS da faixa**, em arestas.
///
/// Devolve `(p50, p90, máximo, quantas cadeias)`. ⚠️ **A contagem não é
/// decoração:** uma faixa sem arestas alinhadas devolveria zeros, que é o que
/// uma malha perfeitamente isotrópica também lê — *um zero de «não medido» e um
/// de «sem fileira» são o mesmo byte*.
#[must_use]
pub fn fileira_da_faixa(
    malha: &Mesh,
    percurso: &[[f32; 3]],
    raio: f32,
) -> (f64, f64, usize, usize) {
    let arestas = alinhadas(malha, percurso, raio);
    if arestas.is_empty() {
        return (0.0, 0.0, 0, 0);
    }

    // vértice → índices das arestas alinhadas que o tocam.
    let mut por_vertice: std::collections::BTreeMap<u32, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, a) in arestas.iter().enumerate() {
        por_vertice.entry(a.0).or_default().push(i);
        por_vertice.entry(a.1).or_default().push(i);
    }

    let mut gasta = vec![false; arestas.len()];
    let mut comprimentos: Vec<usize> = Vec::new();
    for semente in 0..arestas.len() {
        if gasta[semente] {
            continue;
        }
        gasta[semente] = true;
        let mut n = 1usize;
        // ⚠️ **Os DOIS sentidos a partir da semente**, senão uma cadeia
        // apanhada a meio conta metade do que mede.
        for para in [arestas[semente].1, arestas[semente].0] {
            let (mut atual, mut ponta) = (semente, para);
            while let Some(seguinte) = continua(&arestas, &por_vertice, &gasta, atual, ponta) {
                gasta[seguinte] = true;
                n += 1;
                let (a, b) = (arestas[seguinte].0, arestas[seguinte].1);
                ponta = if a == ponta { b } else { a };
                atual = seguinte;
            }
        }
        comprimentos.push(n);
    }

    comprimentos.sort_unstable();
    let quantil = |q: f64| -> f64 {
        let i = ((comprimentos.len() as f64 - 1.0) * q).round() as usize;
        comprimentos[i.min(comprimentos.len() - 1)] as f64
    };
    (
        quantil(0.50),
        quantil(0.90),
        comprimentos.last().copied().unwrap_or(0),
        comprimentos.len(),
    )
}

/// **As arestas que as fileiras encadeiam**, como pares `(min, max)` — para
/// quem quiser DESENHAR a grandeza em vez de a resumir num número.
///
/// ⚠️ Ela devolve a MESMA população que a [`fileira_da_faixa`] conta; escrever
/// um segundo percurso para o desenho seria desenhar outra coisa.
#[must_use]
pub fn arestas_das_fileiras(malha: &Mesh, percurso: &[[f32; 3]], raio: f32) -> Vec<(u32, u32)> {
    alinhadas(malha, percurso, raio)
        .into_iter()
        .map(|(a, b, _)| (a.min(b), a.max(b)))
        .collect()
}

/// A aresta não gasta que melhor CONTINUA `atual` em `ponta`.
fn continua(
    arestas: &[(u32, u32, [f32; 3])],
    por_vertice: &std::collections::BTreeMap<u32, Vec<usize>>,
    gasta: &[bool],
    atual: usize,
    ponta: u32,
) -> Option<usize> {
    let d0 = arestas[atual].2;
    let mut melhor: Option<(usize, f64)> = None;
    for &j in por_vertice.get(&ponta)? {
        if gasta[j] || j == atual {
            continue;
        }
        let c = f64::from(d0[0].mul_add(
            arestas[j].2[0],
            d0[1].mul_add(arestas[j].2[1], d0[2] * arestas[j].2[2]),
        ))
        .abs()
        .clamp(0.0, 1.0);
        if c.acos().to_degrees() > CONTINUA {
            continue;
        }
        if melhor.is_none_or(|(_, m)| c > m) {
            melhor = Some((j, c));
        }
    }
    melhor.map(|(j, _)| j)
}

/// As arestas da faixa que correm a menos de [`ALINHADA`] do traço — com a
/// direcção já **unitária**, que é o que o encadeamento compara.
fn alinhadas(malha: &Mesh, percurso: &[[f32; 3]], raio: f32) -> Vec<(u32, u32, [f32; 3])> {
    let pos = malha.positions();
    let mut vistas = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for f in malha.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            if !vistas.insert((a.min(b), a.max(b))) {
                continue;
            }
            let (pa, pb) = (pos[a as usize], pos[b as usize]);
            let meio = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let Some(rumo) = crate::medida_do_pente::direccao_do_troco(percurso, meio, raio) else {
                continue;
            };
            let aresta = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let (la, lr) = (norma(aresta), norma(rumo));
            if la <= 0.0 || lr <= 0.0 {
                continue;
            }
            let c = f64::from(
                aresta[0].mul_add(rumo[0], aresta[1].mul_add(rumo[1], aresta[2] * rumo[2])),
            ) / f64::from(la * lr);
            if c.abs().clamp(0.0, 1.0).acos().to_degrees() > ALINHADA
                && (180.0 - c.abs().clamp(0.0, 1.0).acos().to_degrees()) > ALINHADA
            {
                continue;
            }
            out.push((a, b, [aresta[0] / la, aresta[1] / la, aresta[2] / la]));
        }
    }
    out
}

fn norma(v: [f32; 3]) -> f32 {
    v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2])).sqrt()
}

#[cfg(test)]
#[path = "medida_da_fileira_tests.rs"]
mod tests;
