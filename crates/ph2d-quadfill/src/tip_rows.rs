//! ⭐⭐⭐ **UMA LINHA POR PONTA — a régua que responde *«QUAL delas?»***.
//!
//! Irmã de [`super::tips`] por RESPONSABILIDADE: aquele módulo publica os **agregados**
//! (o pior `p50`, quantas passam da barra) e este é a **lei por ápice** de que eles saem.
//!
//! # ⛔⛔⛔ Por que ela teve de existir (report do dono, 2026-09-04)
//!
//! *«temos bons resultados em muitas pontas no mesmo mesh onde apenas uma tem resultado
//! ruim. Isso é muito estranho.»* — e nenhuma das três réguas da ponta sabia dizer **qual**.
//! A [`super::tip_deviation`] devolve o pior `p50` **entre** as pontas, a
//! [`super::tip_density`] a mediana e o pior, e a [`super::tip_survival`] um extremo global:
//! *três réguas que já mediam ponta a ponta e deitavam fora o índice antes de devolver*.
//!
//! ⭐ **É a quarta vez que esta linha paga a mesma forma** — o `edge_max` global era cego ao
//! quad de `0,02 × 0,30`, o `χ` era cego à almofada, a `ENTREGA` era cega à ponta que
//! engrossou. *Um extremo ou uma média sobre a peça inteira nunca vê UMA ponta* — e, quando
//! só uma está má, ele também não diz **qual**, que é a pergunta com que uma cura começa.
//!
//! ⚠️ **As duas agregadas passam a ser DOBRAS desta lei** ([`super::tip_deviation`],
//! [`super::tip_density`]), byte a byte: os números que o produto decide não mudam, e não
//! podem — *uma régua nova que muda o veredito da anterior não é a mesma régua.*

use ph2d_mesh::Mesh;

use super::apex::{adjacency, apices, cone_of, path_ball};
use super::local::dist;
use super::tips::point_triangle;

/// ⚠️ **A vizinhança da ponta, em quads** — os vértices da entrada a menos disto do ápice
/// entram no desvio, e a busca de faces da saída alcança o dobro. É a coroa que as duas
/// últimas voltas da grade podiam cobrir; ver [`super::tip_deviation`].
pub(crate) const DEV_RADIUS: f32 = 3.0;

/// ⚠️ **Quantos quads de caminho entram na grade do bico** — ver [`super::tip_density`].
pub(crate) const GRADE_RINGS: f32 = 3.0;

/// ⚠️ **A que distância do bico se procura o PÓLO** — `2` quads, a distância a que a malha
/// que o dono aprovou tem as quatro valência-`3` (plano §101).
pub(crate) const POLE_RINGS: f32 = 2.0;

/// ⭐⭐⭐ **O QUE A CADEIA FEZ A **UMA** PONTA** — uma linha da tabela de [`tip_rows`].
///
/// ⚠️ Todas as distâncias vêm **em unidades de `unit`** (adimensionais), logo duas
/// densidades comparam-se linha a linha.
#[derive(Debug, Clone, Copy)]
pub struct TipRow {
    /// O vértice da **entrada** que é o bico ([`apices`]).
    pub apex: usize,
    /// A distância do bico ao centroide dos vértices — a mesma ordenação que [`apices`] usa.
    /// ⛔ **Não é medida de forma**: para isso o centroide tem de ser o da área
    /// ([`super::reach`]).
    pub radius: f32,
    /// A forma do espinho ([`cone_of`]): `None` = *entrada sem amostra junto do bico*, que a
    /// lei do ápice lê como espinho.
    pub cone: Option<f32>,
    /// ⭐ **A AMPUTAÇÃO** — a distância do ápice da entrada à superfície da saída, contra
    /// [`super::TIP_GAP_MAX`].
    pub gap: f32,
    /// ⚠️ **Nenhuma face da saída dentro do raio da busca**: o `gap` é o **piso** do que se
    /// sabe (*«mais longe do que eu olhei»*), não a distância verdadeira. É a ponta comida
    /// por inteiro.
    pub blind: bool,
    /// O `p50`, o `p90` e o pior desvio dos vértices da entrada junto do bico. `None` = a
    /// **entrada** não tem vértice a menos de [`DEV_RADIUS`] quads do próprio ápice (malha
    /// mais grosseira que o alvo) — ⛔ e isso não é uma acusação à saída.
    pub dev: Option<[f32; 3]>,
    /// ⭐ **A GRADE NO BICO** — o tamanho médio do quad da saída junto da ponta, contra
    /// [`super::TIP_DENSITY_MAX`].
    pub grade: Option<f32>,
    /// ⭐⭐⭐ **O PÓLO — quantos vértices IRREGULARES da saída vivem a `≤ 2` quads de caminho
    /// do bico**, e a valência do próprio bico.
    ///
    /// ⛔ **É a coluna do mecanismo do plano §101:** na retopologia que o dono aprovou **todo**
    /// espinho fecha com um pólo `+1` — **quatro** valência-`3` a `≤ 2 h` —, e nas saídas que
    /// ele reprovou as singularidades estão a `9`–`15 h`. *Uma ponta com a grade certa e sem
    /// pólo é uma ponta que a grade atravessa em vez de fechar*, e nenhuma das outras colunas
    /// distingue as duas.
    pub pole: (usize, usize),
}

/// ⭐⭐⭐ **A TABELA — uma linha por espinho da entrada, na ordem de [`apices`].**
///
/// Ela corre **uma vez** o que as duas agregadas corriam **duas** (o censo de ápices, com o
/// Dijkstra do cone por candidato), e por isso é mais barata que a soma delas.
///
/// ⚠️ **A `unit` é o que o chamador diz que é**: no produto, o **alvo do slider** (o censo
/// tem de ser o mesmo em todas as candidatas de um clique); na bancada, a aresta **mediana**
/// da saída ([`super::median_edge`]), porque a saída de outra ferramenta não tem alvo.
///
/// ⛔ **Lista vazia quando não há o que medir** (entrada ou saída sem vértices, `unit` não
/// positiva, nenhum ápice) — *«não medido» e «perfeito» são o mesmo byte em toda régua que
/// só devolve a média*, e aqui distinguem-se pelo comprimento da tabela.
#[must_use]
pub fn tip_rows(input: &Mesh, output: &Mesh, unit: f32) -> Vec<TipRow> {
    let pos = input.positions();
    let opos = output.positions();
    if pos.is_empty() || opos.is_empty() || !unit.is_finite() || unit <= 0.0 {
        return Vec::new();
    }
    let (mid, apex) = apices(input, unit);
    if apex.is_empty() {
        return Vec::new();
    }
    let inbr = adjacency(input);
    let onbr = adjacency(output);
    // A aresta incidente média de cada vértice da SAÍDA — a matéria-prima da grade.
    let mean_edge: Vec<f32> = (0..opos.len())
        .map(|i| {
            if onbr[i].is_empty() {
                return 0.0;
            }
            #[allow(clippy::cast_precision_loss)]
            let n = onbr[i].len() as f32;
            onbr[i]
                .iter()
                .map(|&j| dist(opos[i], opos[j as usize]))
                .sum::<f32>()
                / n
        })
        .collect();
    // Os triângulos da saída, uma vez só — ver [`fan_tris`].
    let tris = fan_tris(output);
    let radius = DEV_RADIUS * unit;
    let mut rows = Vec::with_capacity(apex.len());
    for &i in &apex {
        let a = pos[i];
        let cone = cone_of(pos, &inbr, i, unit);
        let near = near_tris(&tris, a, radius + DEV_RADIUS * unit);
        // ⛔⛔⛔ **NENHUMA FACE PERTO DO ÁPICE É O PIOR CASO, NÃO UM «SALTAR»** — ver
        // [`TipRow::blind`]. O valor registado é o RAIO da busca, que é maior que as duas
        // barras por construção, logo a ponta conta como partida nas duas.
        if near.is_empty() {
            rows.push(TipRow {
                apex: i,
                radius: dist(a, mid),
                cone,
                gap: DEV_RADIUS,
                blind: true,
                dev: Some([DEV_RADIUS; 3]),
                grade: grade_at(opos, &onbr, &mean_edge, a, unit),
                pole: pole_at(opos, &onbr, a, unit),
            });
            continue;
        }
        let gap = gap_to(&near, a) / unit;
        let mut ds: Vec<f32> = pos
            .iter()
            .filter(|p| dist(a, **p) <= radius)
            .map(|p| {
                near.iter()
                    .fold(f32::MAX, |acc, t| acc.min(point_triangle(*p, t)))
                    / unit
            })
            .collect();
        ds.sort_by(f32::total_cmp);
        let dev = if ds.is_empty() {
            None
        } else {
            Some([ds[ds.len() / 2], ds[ds.len() * 9 / 10], ds[ds.len() - 1]])
        };
        rows.push(TipRow {
            apex: i,
            radius: dist(a, mid),
            cone,
            gap,
            blind: false,
            dev,
            grade: grade_at(opos, &onbr, &mean_edge, a, unit),
            pole: pole_at(opos, &onbr, a, unit),
        });
    }
    rows
}

/// **OS TRIÂNGULOS DE UMA MALHA**, um leque por face — a mesma decomposição que o centroide
/// de área usa, e exacta para qualquer polígono planar.
pub(crate) fn fan_tris(mesh: &Mesh) -> Vec<[[f32; 3]; 3]> {
    let pos = mesh.positions();
    let mut tris: Vec<[[f32; 3]; 3]> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len().saturating_sub(1) {
            tris.push([
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            ]);
        }
    }
    tris
}

/// ⚠️ **Só os triângulos que podem competir** — sem esta cerca a régua é
/// `O(ápices × amostras × faces)` e uma escultura de 17 k vértices leva minutos.
pub(crate) fn near_tris<'a>(
    tris: &'a [[[f32; 3]; 3]],
    p: [f32; 3],
    radius: f32,
) -> Vec<&'a [[f32; 3]; 3]> {
    tris.iter()
        .filter(|t| t.iter().any(|q| dist(p, *q) <= radius))
        .collect()
}

/// ⭐ **A DISTÂNCIA DE UM PONTO À SUPERFÍCIE**, em unidades de mundo, sobre as faces que
/// [`near_tris`] deixou passar. ⛔ **Lista vazia ⇒ [`f32::MAX`]**, que é o caso CEGO: *«mais
/// longe do que eu olhei»* — quem lê tem de o tratar como o pior caso, nunca como um zero.
pub(crate) fn gap_to(near: &[&[[f32; 3]; 3]], p: [f32; 3]) -> f32 {
    near.iter()
        .fold(f32::MAX, |acc, t| acc.min(point_triangle(p, t)))
}

/// ⭐⭐⭐ **O PÓLO de uma ponta** — quantos vértices irregulares da saída vivem a `≤ 2` quads
/// de caminho do bico, e a valência do próprio bico. Ver [`TipRow::pole`].
///
/// ⚠️ **`2 × unit` é a distância da tabela do plano §101** (a malha aprovada tem as quatro
/// valência-`3` a `0,7`–`1,9 h`), e a **valência** conta arestas distintas: a
/// [`adjacency`] repete o vizinho uma vez por face incidente.
fn pole_at(opos: &[[f32; 3]], onbr: &[Vec<u32>], p: [f32; 3], unit: f32) -> (usize, usize) {
    let Some(seed) = (0..opos.len()).min_by(|&i, &j| dist(p, opos[i]).total_cmp(&dist(p, opos[j])))
    else {
        return (0, 0);
    };
    let grau = |v: usize| -> usize {
        onbr[v]
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<u32>>()
            .len()
    };
    let seen = path_ball(opos, onbr, seed, POLE_RINGS * unit);
    let irregulares = seen.keys().filter(|&&v| grau(v) != 4).count();
    (irregulares, grau(seed))
}

/// **A grade da saída junto de um ponto da entrada** — o vértice mais próximo, e a média das
/// arestas incidentes na bola de CAMINHO de [`GRADE_RINGS`] quads à volta dele.
///
/// ⚠️ **De caminho e não em linha recta**: uma vizinhança esférica sobre um espinho fino
/// apanha o **outro lado** do corpo (medido: uma ponta de raio `1,32` leu `25,60`).
fn grade_at(
    opos: &[[f32; 3]],
    onbr: &[Vec<u32>],
    mean_edge: &[f32],
    p: [f32; 3],
    unit: f32,
) -> Option<f32> {
    let seed = (0..opos.len()).min_by(|&i, &j| dist(p, opos[i]).total_cmp(&dist(p, opos[j])))?;
    let seen = path_ball(opos, onbr, seed, GRADE_RINGS * unit);
    if seen.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let n = seen.len() as f32;
    Some(seen.keys().map(|&v| mean_edge[v]).sum::<f32>() / n / unit)
}

// ⚠️ **`pub(crate)` para o irmão** ([`crate::tip_snap`]): *o remate e a régua que o mede têm
// de falar da mesma fixtura*, e duplicá-la seria a segunda resposta a «o que é uma ponta».
#[cfg(test)]
#[path = "tip_rows_tests.rs"]
pub(crate) mod tests;
