//! ⭐⭐⭐ **AS DUAS RÉGUAS DA PONTA QUE NÃO SÃO O SUPORTE** — o alcance da FORMA e o
//! desvio LOCAL junto de cada ápice.
//!
//! Irmã de [`super::local`] por responsabilidade: aquele módulo mede **quanto** de cada
//! ponta sobreviveu ([`super::tip_survival`], a função de suporte); este responde às duas
//! perguntas que o suporte **não pode** responder.
//!
//! # ⛔⛔ 1. O alcance media a AMOSTRAGEM, não a forma
//!
//! O alcance «distância máxima ao centroide» que este repo usava desde sempre tirava o
//! centroide da **média dos vértices** — e essa média é uma propriedade de *onde estão os
//! vértices*, não de *que forma eles descrevem*. Uma retopologia redistribui vértices por
//! construção (a entrada amontoa-os onde o escultor trabalhou, a saída espalha-os por
//! igual), então o centroide **anda** e o alcance mente.
//!
//! ⭐ Medido em 2026-08-31 na escultura do dono (`_base_sculpt.obj`, `Detail 0,85`):
//!
//! | centroide | deslocamento entrada→saída | alcance lido |
//! |---|---|---|
//! | ⛔ média dos **vértices** | `0,2129` | **`−6,5 %`** |
//! | ⭐ pesado pela **área** | `0,0037` | `+0,0 %` |
//! | a verdade (referencial comum) | — | `−0,1 %` |
//!
//! ⛔⛔ **E isto estava no caminho do produto:** a chave de amputação do selector do botão
//! compara o alcance de duas candidatas, e a banda dela é [`super::TIP_CUT_PCT`] = `−2 %`.
//! Duas candidatas de densidades diferentes já diferem **`1,06 %`** só por isso — *metade
//! da banda, sem uma ponta se mexer.* ⚠️ E o sinal é o pior possível: uma candidata que
//! **corta** a ponta perde vértices longe do corpo, o centroide dela afasta-se da ponta, e
//! o alcance medido **sobe** — a régua defende exactamente a candidata que devia acusar.
//!
//! # ⛔⛔ 2. O suporte não vê o FUNIL
//!
//! A função de suporte é `max(v · d)`: ela diz *até onde a peça vai* naquela direcção, e
//! **nada** sobre a espessura com que lá chega. Medido na mesma peça a `Detail 0,50`: a
//! ponta 3 lê `−5,3 %` de suporte, e a superfície mais próxima do ápice está a `0,2133` —
//! **duas células** —, porque a saída fecha o espinho com um anel de raio `≈ 0,19` onde a
//! escultura tem `≈ 0,05`. *O bico é curto e GORDO, e só a metade curta tinha régua.*
//!
//! ⇒ [`tip_deviation`] mede a distância dos vértices da **entrada** que vivem junto de cada
//! ápice até à **superfície** da saída, em unidades do quad pedido.

use ph2d_mesh::Mesh;

use super::local::{apices, cross, dist, sub};

/// ⭐⭐⭐ **O ALCANCE DA FORMA** — a distância máxima ao centroide **pesado pela área**.
///
/// ⚠️ **O peso é a área e não o vértice**, e a diferença é a razão de esta função existir:
/// ver a tabela do módulo. Um centroide por vértice é uma média da *amostragem*; um
/// centroide por área é uma propriedade da *superfície*, e duas malhas da mesma forma
/// concordam nele qualquer que seja a densidade de cada uma.
///
/// ⚠️ **Degenera com honestidade:** uma malha sem faces (ou com área total nula — todas as
/// faces degeneradas) não tem centroide de área, e aí a média dos vértices é a única
/// resposta possível. *Uma régua que devolvesse `0` ali diria «esta forma não tem
/// tamanho», que é falso.*
#[must_use]
pub fn reach(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    if pos.is_empty() {
        return 0.0;
    }
    let c = area_centroid(mesh);
    pos.iter().fold(0.0f32, |acc, q| acc.max(dist(*q, c)))
}

/// O centroide **pesado pela área**, com a média dos vértices como último recurso.
#[must_use]
pub fn area_centroid(mesh: &Mesh) -> [f32; 3] {
    let pos = mesh.positions();
    let mut total = 0.0f64;
    let mut acc = [0.0f64; 3];
    for f in mesh.faces() {
        let v = f.verts();
        // ⚠️ **Leque a partir do vértice 0** — é a mesma decomposição que o resto desta
        // crate usa para um polígono, e para o centroide ela é exacta em qualquer polígono
        // planar (a soma dos momentos não depende de como se corta).
        for k in 1..v.len().saturating_sub(1) {
            let (a, b, d) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            let area = 0.5 * f64::from(norm(cross(sub(b, a), sub(d, a))));
            total += area;
            for j in 0..3 {
                acc[j] += area * f64::from((a[j] + b[j] + d[j]) / 3.0);
            }
        }
    }
    if total > 1.0e-12 {
        #[allow(clippy::cast_possible_truncation)]
        return [
            (acc[0] / total) as f32,
            (acc[1] / total) as f32,
            (acc[2] / total) as f32,
        ];
    }
    let mut c = [0.0f64; 3];
    for p in pos {
        for k in 0..3 {
            c[k] += f64::from(p[k]);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n = pos.len().max(1) as f64;
    #[allow(clippy::cast_possible_truncation)]
    [(c[0] / n) as f32, (c[1] / n) as f32, (c[2] / n) as f32]
}

/// ⚠️ **A BARRA, e ela é o CHÃO DA DISCRETIZAÇÃO — não um número escolhido.** Uma grade de
/// passo `h` não pode seguir uma superfície melhor que `h`; abaixo de uma célula o desvio
/// **é** a discretização, e acima dela falta superfície.
///
/// ⭐ Medido na escultura do dono (`_base_sculpt.obj`) com esta régua, nas duas densidades:
/// as pontas **sãs** medem `p50 0,08`–`0,30` e `máximo 0,45`; a ponta **partida** mede
/// `p50 1,15` e `p90 2,02`. *Há um vazio de `2,6×` entre as duas populações, e a barra
/// vive nele.*
pub const TIP_DEVIATION_MAX: f32 = 1.0;

/// **QUANTO A SAÍDA SE AFASTA DA ESCULTURA JUNTO DE CADA PONTA** — ver [`tip_deviation`].
#[derive(Debug, Clone, Copy, Default)]
pub struct TipDeviation {
    /// O pior `p50` entre as pontas medidas, em unidades do quad pedido.
    pub p50: f32,
    /// O pior `p90` entre as pontas medidas, em unidades do quad pedido.
    pub p90: f32,
    /// O pior desvio de todos, em unidades do quad pedido.
    pub max: f32,
    /// Quantas pontas foram medidas.
    pub tips: usize,
    /// Quantas delas passam de [`TIP_DEVIATION_MAX`].
    pub over: usize,
}

/// ⭐⭐⭐ **O DESVIO LOCAL JUNTO DE CADA PONTA, em unidades do quad pedido.**
///
/// Para cada ápice da **entrada** (achados por [`apices`], a mesma lei que
/// [`super::tip_survival`] usa — ⛔ *uma lei escrita em dois sítios ainda não é uma lei*),
/// tomam-se os vértices da entrada a menos de `3 × target` do ápice e mede-se a distância
/// de cada um à **superfície** da saída. O resultado vai dividido por `target`, logo é
/// **adimensional** e comparável entre densidades.
///
/// ⚠️ **É ponto→FACE e não ponto→vértice.** Medido: com vértices a população sã lê
/// `p50 0,28`–`0,35` (o erro é dominado por metade de uma aresta da saída) e com faces lê
/// `0,08`–`0,30`. *Uma régua cujo valor «são» é feito do artefacto da própria régua não
/// tem onde pôr uma barra.*
///
/// ⚠️ **O raio de `3 × target` é a vizinhança que as duas últimas coroas da grade podiam
/// cobrir.** Menor que isso e uma ponta são com poucas amostras fica sem população; muito
/// maior e a medida dilui-se no corpo, que é justamente onde a cadeia acerta sempre.
///
/// ⛔ **Devolve o zero-de-`Default` quando não há o que medir** (entrada ou saída vazia,
/// `target` não positivo, nenhum ápice) — e o `tips` fica a `0` **de propósito**, para que
/// quem lê distinga *«não medido»* de *«perfeito»*: são o mesmo byte em toda régua que só
/// devolve a média.
#[must_use]
pub fn tip_deviation(input: &Mesh, output: &Mesh, target: f32) -> TipDeviation {
    let mut out = TipDeviation::default();
    let pos = input.positions();
    // ⚠️ **`is_finite` ANTES da comparação, e não `!(target > 0.0)`**: um `NaN` faz toda
    // comparação ser falsa, então a forma negada esconde a intenção — e o clippy recusa-a.
    if pos.is_empty() || output.positions().is_empty() || !target.is_finite() || target <= 0.0 {
        return out;
    }
    let (_mid, apex) = apices(input);
    // ⚠️ **Os triângulos da saída, uma vez só** — um leque por face, como no centroide.
    let opos = output.positions();
    let mut tris: Vec<[[f32; 3]; 3]> = Vec::new();
    for f in output.faces() {
        let v = f.verts();
        for k in 1..v.len().saturating_sub(1) {
            tris.push([
                opos[v[0] as usize],
                opos[v[k] as usize],
                opos[v[k + 1] as usize],
            ]);
        }
    }
    if tris.is_empty() {
        return out;
    }
    const RADIUS: f32 = 3.0;
    let radius = RADIUS * target;
    for &i in &apex {
        let a = pos[i];
        // ⚠️ **Só os triângulos que podem competir** — sem esta cerca a régua é
        // `O(ápices × amostras × faces)` e uma escultura de 17 k vértices leva minutos.
        let near: Vec<&[[f32; 3]; 3]> = tris
            .iter()
            .filter(|t| t.iter().any(|q| dist(a, *q) <= radius + RADIUS * target))
            .collect();
        if near.is_empty() {
            continue;
        }
        let mut ds: Vec<f32> = pos
            .iter()
            .filter(|p| dist(a, **p) <= radius)
            .map(|p| {
                near.iter()
                    .fold(f32::MAX, |acc, t| acc.min(point_triangle(*p, t)))
                    / target
            })
            .collect();
        if ds.is_empty() {
            continue;
        }
        ds.sort_by(f32::total_cmp);
        let p50 = ds[ds.len() / 2];
        let p90 = ds[ds.len() * 9 / 10];
        let worst = ds[ds.len() - 1];
        out.tips += 1;
        out.p50 = out.p50.max(p50);
        out.p90 = out.p90.max(p90);
        out.max = out.max.max(worst);
        if p50 > TIP_DEVIATION_MAX {
            out.over += 1;
        }
    }
    out
}

#[cfg(test)]
#[path = "tips_tests.rs"]
mod tests;

fn norm(v: [f32; 3]) -> f32 {
    v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2])).sqrt()
}

/// A distância de um ponto a um triângulo, pelas sete regiões de Voronoi (Ericson).
fn point_triangle(p: [f32; 3], t: &[[f32; 3]; 3]) -> f32 {
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return dist(p, a);
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot3(ab, bp), dot3(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return dist(p, b);
    }
    let vc = d1.mul_add(d4, -(d3 * d2));
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return dist(p, axpy(a, ab, v));
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot3(ab, cp), dot3(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return dist(p, c);
    }
    let vb = d5.mul_add(d2, -(d1 * d6));
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return dist(p, axpy(a, ac, w));
    }
    let va = d3.mul_add(d6, -(d5 * d4));
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return dist(p, axpy(b, sub(c, b), w));
    }
    let den = 1.0 / (va + vb + vc);
    let (v, w) = (vb * den, vc * den);
    dist(p, axpy(axpy(a, ab, v), ac, w))
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn axpy(a: [f32; 3], d: [f32; 3], t: f32) -> [f32; 3] {
    [
        d[0].mul_add(t, a[0]),
        d[1].mul_add(t, a[1]),
        d[2].mul_add(t, a[2]),
    ]
}
