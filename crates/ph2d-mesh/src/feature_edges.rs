//! ⭐⭐⭐ **DAS DIRECÇÕES POR VÉRTICE ÀS ARESTAS DE FEIÇÃO** — o degrau que falta
//! entre o que a [`crate::feature_dirs`] elege e o que a espec restringe.
//!
//! ⚠️ **A espec fala de ARESTAS, e a lei do *paper* elege DIRECÇÕES POR PONTO.**
//! (`SPEC_restricoes_por_eliminacao.md` §3: *«as arestas de feição entram como
//! restrições de orientação nos dois triângulos vizinhos»*.) Este ficheiro é a ponte,
//! e ela é uma conjunção de três condições — deliberadamente a mais conservadora que
//! ainda deixa uma linha de vinco passar:
//!
//! > uma aresta é de feição quando **as duas pontas a marcaram** *e* **a própria
//! > aresta segue as duas direcções**.
//!
//! ⭐ É isto que a torna esparsa **por construção**: um vértice isolado com uma leitura
//! forte não produz aresta nenhuma, porque não tem com quem concordar. *Uma feição é
//! uma LINHA; um ponto sozinho é uma leitura.*
//!
//! ⛔⛔ **O `min_cos` é um QUINTO coeficiente, e ele não vem do *paper*** — os quatro
//! da espec vivem na [`crate::FeatureOptions`]. Ele nasce desta ponte, então a medição
//! dele é dívida desta ponte: ver [`FEATURE_EDGE_MIN_COS`].

use std::collections::{BTreeMap, BTreeSet};

use crate::{FeatureDir, Mesh};

/// ⭐ **O quinto coeficiente: quão paralela a aresta tem de ser à direcção eleita.**
///
/// `cos 15° ≈ 0,966`. ⚠️ **Ele é um ÂNGULO DE EIXO** (mede-se por `|cos|`), então
/// `15°` aqui quer dizer *«a aresta não se afasta mais de 15° da direcção do vinco,
/// em qualquer dos dois sentidos»*.
///
/// ⭐⭐ **MEDIDO (2026-08-25, peça do artista, cadeia inteira):** a `30°` a lei marca
/// `1,06 %` das arestas e a peça sai com `30` arestas de bordo e `χ = −2`; a `15°` marca
/// `0,43 %`, o bordo cai a `6` e o `χ` fica em `1`. ⛔ *A versão permissiva não achava mais
/// vinco: ela aceitava a diagonal do triângulo ao lado dele.*
///
/// ⚠️ **A régua dele é a mesma da meia-janela: cobertura contra confiança.** Um valor
/// alto exige que a malha tenha por acaso uma aresta quase exactamente sobre o vinco
/// (o F1 remalha isotropicamente, ele **não** alinha arestas a nada); um valor baixo
/// aceita a diagonal do triângulo ao lado do vinco e a restrição passa a apontar para
/// onde a feição não está. ⇒ a sonda `feature_sweep` mede-o com a **contagem de
/// singularidades** ao lado, que é a régua do gate nº7.
pub const FEATURE_EDGE_MIN_COS: f32 = 0.966;

/// ⭐⭐⭐ **Uma aresta de feição, COM A DIRECÇÃO QUE ELA IMPÕE.**
///
/// ⛔⛔ **A direcção NÃO é a da aresta, e a 1.ª redacção desta ponte usava-a.** Medido na
/// peça do artista (2026-08-25): constranger a cruz à direcção da **aresta da malha** levava
/// o campo de `25` para `242` singularidades com `26`–`146` faces fixas. ⭐ *O F1 remalha
/// isotropicamente; ele **não** alinha arestas a vinco nenhum.* ⇒ uma linha de vinco é
/// aproximada por uma cadeia de arestas em **ziguezague**, e duas faces vizinhas recebiam
/// alvos até `60°` um do outro, que o campo tem de pagar em saltos de período.
///
/// ⚠️ **O sinal que nomeou a causa foi o `min_cos`:** apertar a prova de paralelismo de
/// `30°` para `15°` — que é apertar o ziguezague — **cortou as singularidades a metade**
/// (`242` para `107`) sem mudar mais nada.
///
/// ⇒ **A aresta ELEGE, a curvatura DECIDE.** O valor é a média (de EIXO) das duas direcções
/// de feição das pontas, que saem da segunda forma fundamental sobre uma vizinhança inteira
/// e por isso variam devagar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FeatureEdge {
    /// As duas pontas, `[menor, maior]`.
    pub verts: [u32; 2],
    /// A direcção do vinco ali, em mundo, normalizada. ⚠️ É um **eixo**.
    pub dir: [f32; 3],
}

/// O que a ponte mediu de si própria.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FeatureEdgeReport {
    /// ⭐ **Quantas arestas DISTINTAS a peça tem** — o denominador do gate nº7.
    pub total_edges: usize,
    /// Arestas cujas **duas** pontas foram marcadas.
    pub candidates: usize,
    /// ⭐ Arestas que ficaram — as que geram restrição.
    pub kept: usize,
    /// ⛔ Candidatas recusadas porque a aresta **não segue** as duas direcções.
    pub rejected_angle: usize,
}

impl FeatureEdgeReport {
    /// ⭐ **A ESPARSIDADE, em percentagem das arestas da peça** — é o número que o
    /// gate nº7 lê. *A espec não pede «muitas feições»; pede poucas e certas.*
    #[must_use]
    pub fn sparsity_pct(&self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        {
            100.0 * self.kept as f64 / self.total_edges.max(1) as f64
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

/// A média de dois **EIXOS** — `d` e `−d` dizem a mesma coisa, então o segundo é virado
/// para o lado do primeiro antes de somar.
///
/// ⛔ Sem a viragem, dois eixos quase iguais mas gravados com sinais opostos somam **zero**,
/// e a direcção resultante seria a do `fallback`. *Uma média de eixos feita como média de
/// vectores cancela exactamente onde os dois mais concordam.*
fn axis_mean(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    let s = if dot(a, b) < 0.0 { -1.0f32 } else { 1.0 };
    let m = [
        s.mul_add(b[0], a[0]),
        s.mul_add(b[1], a[1]),
        s.mul_add(b[2], a[2]),
    ];
    let l = dot(m, m).sqrt();
    if l > 1.0e-12 {
        [m[0] / l, m[1] / l, m[2] / l]
    } else {
        a
    }
}

/// ⭐⭐⭐ **AS ARESTAS DE FEIÇÃO**, ordenadas e sem repetição (`[menor, maior]`).
///
/// ⚠️ **`BTreeSet`, nunca `HashSet`:** a ordem desta lista entra na ordem em que as
/// faces são restringidas, e um empate resolvido por ordem de hash tornaria a saída
/// dependente da semente do processo (HR-5).
#[must_use]
pub fn feature_edges(
    mesh: &Mesh,
    dirs: &[FeatureDir],
    min_cos: f32,
) -> (Vec<FeatureEdge>, FeatureEdgeReport) {
    let pos = mesh.positions();
    let dir_of: BTreeMap<u32, [f32; 3]> = dirs.iter().map(|d| (d.vert, d.dir)).collect();

    let mut all: BTreeSet<[u32; 2]> = BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            all.insert(if a < b { [a, b] } else { [b, a] });
        }
    }

    let mut rep = FeatureEdgeReport {
        total_edges: all.len(),
        ..FeatureEdgeReport::default()
    };
    let mut out: Vec<FeatureEdge> = Vec::new();
    for e in all {
        let (Some(&da), Some(&db)) = (dir_of.get(&e[0]), dir_of.get(&e[1])) else {
            continue;
        };
        rep.candidates += 1;
        let d = sub(pos[e[1] as usize], pos[e[0] as usize]);
        let l = dot(d, d).sqrt();
        if l <= 1.0e-20 {
            rep.rejected_angle += 1;
            continue;
        }
        let u = [d[0] / l, d[1] / l, d[2] / l];
        if dot(u, da).abs() >= min_cos && dot(u, db).abs() >= min_cos {
            out.push(FeatureEdge {
                verts: e,
                dir: axis_mean(da, db),
            });
        } else {
            rep.rejected_angle += 1;
        }
    }
    rep.kept = out.len();
    (out, rep)
}

#[cfg(test)]
#[path = "feature_edges_tests.rs"]
mod tests;

/// ⭐⭐⭐ **O BORDO É UMA LINHA DE FEIÇÃO — e é a única EXACTA.**
///
/// # ⛔⛔ Por que ela existe
///
/// A [`feature_edges`] infere feição da **curvatura**, com cinco coeficientes e um limiar;
/// ela pode marcar a mais ou a menos. Uma aresta de **bordo** não precisa de nada disso: ela
/// é feição **por definição** — a superfície acaba ali, e nenhuma grade a pode atravessar.
/// *É o único sítio onde a lei da feição não tem parâmetro nenhum.*
///
/// Medido 2026-08-26 nas peças do artista: a `t002` entra com um buraco (`1` laço, perímetro
/// `0,60`) e o mapa dela sai com **38 dobras no contínuo**, contra **`0`** na `t001` (fechada,
/// depois de curada). A peça que ele chamou *«melhor resultado até agora»* — a `t003` — é a
/// **única das três que chega sem bordo e sem aresta ambígua**.
///
/// # ⚠️ A direcção é a TANGENTE DO LAÇO, não a da aresta
///
/// Numa triangulação, a poligonal do bordo **serrilha** em torno da curva verdadeira. ⇒ a
/// tangente de cada vértice de bordo é a média-de-eixo das suas duas arestas de bordo, e a
/// da aresta é a média-de-eixo das duas pontas — **a mesma lei** que a [`feature_edges`]
/// aplica às direcções de curvatura. *Uma lei escrita duas vezes de maneiras diferentes é
/// duas leis.*
///
/// ⚠️ **Um vértice de bordo com mais de duas arestas de bordo** (o pinçamento onde dois
/// buracos se tocam) recebe a média de todas — não há laço para seguir ali, e recusar seria
/// deixar sem restrição precisamente o sítio mais frágil.
///
/// A saída é ordenada e sem repetição, como a da [`feature_edges`], e pelo mesmo motivo
/// (HR-5: a ordem entra na ordem de restrição das faces).
#[must_use]
pub fn boundary_feature_edges(mesh: &Mesh) -> (Vec<FeatureEdge>, usize) {
    let pos = mesh.positions();
    let mut count: BTreeMap<[u32; 2], usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *count
                .entry(if a < b { [a, b] } else { [b, a] })
                .or_default() += 1;
        }
    }
    let border: Vec<[u32; 2]> = count
        .into_iter()
        .filter(|(_, n)| *n == 1)
        .map(|(e, _)| e)
        .collect();
    if border.is_empty() {
        return (Vec::new(), 0);
    }

    let unit_of = |e: &[u32; 2]| {
        let d = sub(pos[e[1] as usize], pos[e[0] as usize]);
        let l = dot(d, d).sqrt();
        (l > 1.0e-20).then(|| [d[0] / l, d[1] / l, d[2] / l])
    };

    // A tangente de cada vértice do bordo — média-de-eixo das arestas de bordo que o tocam.
    let mut tangent: BTreeMap<u32, [f32; 3]> = BTreeMap::new();
    for e in &border {
        let Some(u) = unit_of(e) else { continue };
        for &v in e {
            tangent
                .entry(v)
                .and_modify(|t| *t = axis_mean(*t, u))
                .or_insert(u);
        }
    }

    let mut loops = 0usize;
    {
        // Quantos laços de bordo há — a régua que diz se o remalhe preservou o buraco.
        let mut nxt: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for e in &border {
            nxt.entry(e[0]).or_default().push(e[1]);
            nxt.entry(e[1]).or_default().push(e[0]);
        }
        let mut seen: BTreeSet<u32> = BTreeSet::new();
        for &v in nxt.keys() {
            if !seen.insert(v) {
                continue;
            }
            loops += 1;
            let mut stack = vec![v];
            while let Some(u) = stack.pop() {
                for &w in nxt.get(&u).into_iter().flatten() {
                    if seen.insert(w) {
                        stack.push(w);
                    }
                }
            }
        }
    }

    let out: Vec<FeatureEdge> = border
        .iter()
        .filter_map(|e| {
            let (ta, tb) = (tangent.get(&e[0])?, tangent.get(&e[1])?);
            Some(FeatureEdge {
                verts: *e,
                dir: axis_mean(*ta, *tb),
            })
        })
        .collect();
    (out, loops)
}
