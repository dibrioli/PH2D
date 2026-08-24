//! ⭐⭐⭐ **G4 — AS MARCAÇÕES DE CADA ARCO, lidas do mapa global.**
//!
//! # O que muda de espécie, dito na régua certa
//!
//! Hoje o `τ` de um arco — *onde ao longo dele caem os pontos de subdivisão* — é
//! **comprimento de arco**. Seis curas locais tentaram substituí-lo por algo melhor e
//! nenhuma moveu o número (`PLAN.md` §4-octoetquadragies, §4-novemetquadragies,
//! §4-unetquinquagies), porque a marcação de um arco tem de servir **dois** pedidos que
//! não se satisfazem ao mesmo tempo localmente.
//!
//! ⭐ Aqui o `τ` passa a ser **a coordenada do mapa global** ao longo do arco. Os dois
//! patches que partilham o arco leem a **mesma função** — logo concordam **por
//! construção**, e não por uma média negociada depois.
//!
//! # ⚠️ O TOTAL de cada arco NÃO muda, e a restrição é a mesma do `regraduate`
//!
//! O último valor de `arc_tau` é o peso do arco perante os irmãos do mesmo lado **e**
//! perante o alvo da quantização. Mexer nele mudaria quantos segmentos o F4 dá a cada
//! arco, que é outra experiência. ⇒ *muda-se a FORMA de dentro do arco e mais nada*, e
//! um factor de escala global do mapa sai na normalização.
//!
//! # ⭐⭐⭐ E a régua desta fase é o DESACORDO ENTRE OS DOIS LADOS
//!
//! É a promessa inteira do mapa global, e é medível: derivar o `τ` a partir do lado `0`
//! e a partir do lado `1`, e comparar. ⛔ *Se os dois lados discordarem, esta fase não
//! entregou nada que as seis curas locais não tivessem entregue.*

/// ⭐⭐⭐ **QUÃO DIREITO um arco tem de ser NO MAPA para o mapa opinar sobre ele.**
///
/// A rectidão é `|z_fim − z_ini| / Σ|Δz|` — quanto o arco **anda** contra quanto ele
/// **percorre**. `1` = uma recta no mapa; perto de `0` = ele serpenteia e a direcção
/// dele é ruído.
///
/// ⛔ **O valor sai de MEDIÇÃO**, esfera lisa, os arcos ordenados por desacordo entre os
/// dois lados:
///
/// | arco | desacordo | ⭐ rectidão |
/// |---|---|---|
/// | `19` | ⛔ **`0,1515`** | **`0,066`** |
/// | `15` | `0,0193` | `0,365` |
/// | `38` | `0,0116` | `0,154` |
/// | `5` | `0,0100` | `0,691` |
/// | `14` | `0,0092` | `0,566` |
///
/// ⭐ **Os três piores desacordos são os três arcos que mais serpenteiam**, e há um
/// intervalo natural entre `0,365` e `0,423`. ⇒ `0,4`.
///
/// ⚠️ Um arco recusado **não é um erro**: ele fica com o `τ` de comprimento de arco de
/// sempre, e a recusa é contada em [`MarkReport::gave_up`]. *O mapa não tem opinião
/// sobre um arco que ele não consegue endireitar, e fingir que tem seria pior.*
pub const MIN_STRAIGHTNESS: f32 = 0.4;

use ph2d_trace::PatchLayout;

use crate::cut::CutMesh;
use crate::solve::GridMap;

/// O que a leitura das marcações mediu.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MarkReport {
    /// Arcos que o layout tem.
    pub arcs: usize,
    /// ⭐ Arcos cuja marcação veio do mapa global.
    pub marked: usize,
    /// **Por que desistiu**, por ordem: `0` sem costura · `1` sem cópia local ·
    /// `2` percurso nulo no mapa · `3` `τ` de origem degenerado · `4` o arco
    /// **serpenteia** no mapa (ver [`MIN_STRAIGHTNESS`]).
    ///
    /// ⛔ *Um numerador sem o motivo diz **que** ela desiste e não **onde*** — a lição
    /// que o `regraduate` pagou três vezes.
    pub gave_up: [usize; 5],
    /// ⭐⭐⭐ **O DESACORDO entre os dois lados**, em fracção do arco: mediana e pior.
    ///
    /// `0` = os dois lados marcam o arco no **mesmo sítio**, que é a promessa inteira
    /// desta fase.
    pub disagree_p50: f32,
    /// O pior desacordo entre os dois lados.
    pub disagree_max: f32,
    /// ⚠️ Quantos arcos saíram com o percurso **não monótono** no mapa e tiveram de ser
    /// forçados. *Um `τ` que recua devolve pontos fora de ordem.*
    pub forced_monotone: usize,
}

/// A coordenada do mapa ao longo de uma cadeia, já normalizada a `[0, 1]`.
///
/// ⭐⭐⭐ **É a PROJECÇÃO na direcção do próprio arco**, e não uma das duas coordenadas.
///
/// ⛔⛔ **A primeira versão escolhia «o eixo que mais anda», e isso é uma moeda ao ar.**
/// Medido 2026-08-23 na esfera lisa: no arco `5` o lado `0` percorre `(0,592 · 0,580)` —
/// as duas coordenadas andam **quase igual** — e cada lado caía num eixo diferente, com
/// `62 %` de desacordo sobre onde as marcas caem.
///
/// ⚠️ E o diagnóstico mostrou mais: no arco `38` os dois lados escolhem eixos
/// **diferentes e ambos certos** (`0,394/0,914` contra `0,908/0,399`), porque as
/// molduras deles estão rodadas uma em relação à outra. *Não há «o eixo» de um arco: há
/// a direcção dele.*
///
/// ⭐ A projecção resolve os dois de uma vez, e é **invariante à rotação** — se um lado
/// vê `z` e o outro `R^k z`, a direcção do arco roda com eles e a projecção sai igual.
///
/// `None` quando o percurso é nulo — *e é uma resposta, não um zero*.
fn along(z: &[[f32; 2]]) -> Option<Vec<f32>> {
    let (first, last) = (z.first()?, z.last()?);
    let d = [last[0] - first[0], last[1] - first[1]];
    let len2 = d[0].mul_add(d[0], d[1] * d[1]);
    if len2 < 1.0e-18 {
        return None;
    }
    Some(
        z.iter()
            .map(|p| (p[0] - first[0]).mul_add(d[0], (p[1] - first[1]) * d[1]) / len2)
            .collect(),
    )
}

/// ⭐⭐⭐ **AS MARCAÇÕES NOVAS**, um `arc_tau` de substituição.
///
/// ⚠️ **`None` num arco é uma resposta:** ele fica com o `τ` de sempre, e o motivo
/// aparece em [`MarkReport::gave_up`].
#[must_use]
pub fn arc_marks(
    layout: &PatchLayout,
    cut: &CutMesh,
    map: &GridMap,
) -> (Vec<Vec<f32>>, MarkReport) {
    let mut rep = MarkReport {
        arcs: layout.arc_tau.len(),
        ..MarkReport::default()
    };
    let mut out: Vec<Vec<f32>> = layout.arc_tau.clone();

    // Por arco, a costura que o representa.
    let mut by_arc: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for (s, seam) in cut.seams.iter().enumerate() {
        if let Some(a) = seam.arc {
            by_arc.insert(a, s);
        }
    }

    let mut disagree: Vec<f32> = Vec::with_capacity(rep.arcs);
    for (a, old) in layout.arc_tau.iter().enumerate() {
        let total = old.last().copied().unwrap_or(0.0);
        if total <= 0.0 || old.len() < 2 {
            rep.gave_up[3] += 1;
            continue;
        }
        let Ok(aid) = u32::try_from(a) else {
            continue;
        };
        let Some(&s) = by_arc.get(&aid) else {
            rep.gave_up[0] += 1;
            continue;
        };
        let seam = &cut.seams[s];
        // ⭐ Os dois lados, cada um lido do SEU patch — é a comparação que vale.
        let mut per_side: [Option<Vec<f32>>; 2] = [None, None];
        for (which, side) in seam.side.iter().enumerate() {
            let p = side.patch as usize;
            let mut z: Vec<[f32; 2]> = Vec::with_capacity(side.local.len());
            for l in &side.local {
                let Some(l) = l else {
                    break;
                };
                let Some(&v) = map.uv.get(p).and_then(|u| u.get(*l as usize)) else {
                    break;
                };
                z.push(v);
            }
            if z.len() == side.local.len() && z.len() == old.len() {
                per_side[which] = along(&z);
            }
        }
        // ⭐ A rectidão dos dois lados: o mapa só opina sobre um arco que ele
        // endireita. ⚠️ *Um arco que serpenteia tem direcção de ruído, e a projecção
        // amplifica-o — medido: o pior desacordo do corpus tem rectidão `0,066`.*
        if straightness_of(seam, map, 0).min(straightness_of(seam, map, 1)) < MIN_STRAIGHTNESS {
            rep.gave_up[4] += 1;
            continue;
        }
        let (Some(f0), Some(f1)) = (&per_side[0], &per_side[1]) else {
            if per_side[0].is_none() && per_side[1].is_none() {
                rep.gave_up[1] += 1;
            } else {
                rep.gave_up[2] += 1;
            }
            continue;
        };
        // ⭐⭐⭐ **A RÉGUA:** os dois lados marcam o arco no mesmo sítio?
        let worst = f0
            .iter()
            .zip(f1)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        disagree.push(worst);

        // A marcação que shipa é a média dos dois — ⚠️ e com o mapa global ela é uma
        // média de dois números que já concordam, não uma negociação entre dois pedidos
        // em conflito. *É essa a diferença, e o `worst` acima é quem a prova.*
        let mut t: Vec<f32> = f0
            .iter()
            .zip(f1)
            .map(|(x, y)| 0.5 * (x + y) * total)
            .collect();
        // ⛔⛔ **AS PONTAS PREGAM-SE ANTES DE FORÇAR A MONOTONIA, e a ordem custou um
        // gate vermelho.** A primeira versão pregava depois: um `t[0]` negativo virava
        // `0` e passava a ser MAIOR que o `t[1]` já forçado, ⇒ o `τ` saía a recuar
        // apesar da passagem monótona ter corrido. *Uma correcção aplicada depois da
        // rede desfaz a rede.*
        t[0] = 0.0;
        if let Some(last) = t.last_mut() {
            *last = total;
        }
        let n = t.len();
        for x in t.iter_mut().take(n - 1).skip(1) {
            *x = x.clamp(0.0, total);
        }
        let mut forced = false;
        for k in 1..n {
            if t[k] < t[k - 1] {
                forced = true;
                t[k] = t[k - 1];
            }
        }
        rep.forced_monotone += usize::from(forced);
        out[a] = t;
        rep.marked += 1;
    }

    disagree.sort_by(f32::total_cmp);
    if !disagree.is_empty() {
        rep.disagree_p50 = disagree[disagree.len() / 2];
        rep.disagree_max = disagree.last().copied().unwrap_or(0.0);
    }
    (out, rep)
}

/// `|z_fim − z_ini| / Σ|Δz|` de um lado de uma costura — ver [`MIN_STRAIGHTNESS`].
fn straightness_of(seam: &crate::cut::Seam, map: &GridMap, which: usize) -> f32 {
    let Some(side) = seam.side.get(which) else {
        return 0.0;
    };
    let z: Vec<[f32; 2]> = side
        .local
        .iter()
        .filter_map(|l| {
            l.and_then(|l| {
                map.uv
                    .get(side.patch as usize)
                    .and_then(|u| u.get(l as usize))
            })
        })
        .copied()
        .collect();
    if z.len() < 2 {
        return 0.0;
    }
    let d = [z[z.len() - 1][0] - z[0][0], z[z.len() - 1][1] - z[0][1]];
    let disp = d[0].mul_add(d[0], d[1] * d[1]).sqrt();
    let walk: f32 = z
        .windows(2)
        .map(|w| {
            let e = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
            e[0].mul_add(e[0], e[1] * e[1]).sqrt()
        })
        .sum();
    if walk < 1.0e-12 { 0.0 } else { disp / walk }
}

#[cfg(test)]
#[path = "marks_tests.rs"]
mod tests;
