//! ⭐⭐⭐ **AS DIRECÇÕES DE FEIÇÃO** — onde a superfície tem um vinco com orientação
//! bem definida, e a que escala isso é verdade.
//!
//! Irmão do [`super::curvature_dirs`], e o corte é de **escala**: lá a segunda forma é
//! ajustada sobre **um triângulo** (a vizinhança é uma face de largura); aqui sobre uma
//! **vizinhança de raio `r`**, e sobre uma **faixa** de raios.
//!
//! # ⛔ Por que ela nasceu, e a razão é um report com foto
//!
//! O artista (2026-08-25) reportou pontas, chifres e vincos com resultado mau, com
//! buracos. Medido na peça dele: **25 singularidades** do campo contra as **8** que a
//! topologia exige, **14 triângulos dobrados** no mapa e **14 arestas de bordo** —
//! contra `8 / 0 / 0` numa esfera. ⇒ *o campo não sabe que o vinco existe, planta
//! singularidades a mais, o mapa dobra, a célula colapsa, e o buraco aparece.*
//!
//! # A LEI, e ela tem TRÊS degraus que são três perguntas diferentes
//!
//! 1. **A FAIXA.** Estima-se a segunda forma em **vários** raios ao longo de
//!    `[r₀, r₁]`. ⇒ cada ponto tem um **conjunto de candidatos**, um por raio.
//! 2. ⭐⭐⭐ **A JANELA, e é ela que decide a validade.** À volta de cada raio candidato
//!    `r` há uma janela `[r − w, r + w]`. ⛔ **Um candidato só é válido se os dois
//!    limiares — a anisotropia e o piso de curvatura média — valerem em TODA a janela
//!    dele.** ⚠️ *É esta condição que separa uma feição real de um pico de ruído:* uma
//!    leitura que só passa num raio e falha ao lado dele não é uma feição, é uma
//!    coincidência de escala.
//! 3. **A ELEIÇÃO.** Havendo vários candidatos válidos, ganha o de direcção mais
//!    estável — o de **menor variação de direcção DENTRO DA JANELA DELE**. ⛔ Não sobre
//!    a faixa inteira: *a faixa é onde se procura, a janela é onde se julga.*
//!
//! Um ponto **sem** candidato válido **não gera restrição nenhuma** — é assim que a lei
//! fica esparsa **por construção**, e não por um corte posterior.
//!
//! ⚠️ **Os quatro coeficientes são MEDIDOS neste corpus, não copiados** — ver
//! [`FeatureOptions`] e a sonda `the_feature_law_sweeps_its_four_coefficients`.
//!
//! ⛔ **A vizinhança é EUCLIDIANA e não geodésica**, e isso é uma divergência
//! declarada: numa peça fina os dois lados de uma parede caem na mesma bola. O preço
//! está medido ao lado do resultado, e a cura (se for precisa) é caminhar a malha em
//! vez de consultar a octree.

use crate::curvature_dirs::{anisotropy_of, second_form};
use crate::{Mesh, QueryScratch};

/// **Uma direcção de feição eleita, e a escala em que ela é verdade.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FeatureDir {
    /// O vértice.
    pub vert: u32,
    /// A direcção do vinco, em mundo, normalizada. ⚠️ É um **eixo**: `d` e `−d` dizem
    /// a mesma coisa.
    pub dir: [f32; 3],
    /// ⭐ O raio em que ela foi eleita — a **escala** da feição.
    pub radius: f32,
    /// A anisotropia nesse raio.
    pub anisotropy: f32,
}

/// ⭐ **OS QUATRO COEFICIENTES DA LEI.**
///
/// ⚠️ **Três deles são RELATIVOS a grandezas que a peça já dá** — o passo alvo da
/// grade, a aresta média e o raio da caixa — e é isso que os torna transportáveis entre
/// peças de tamanhos diferentes. *Um limiar absoluto teria de ser reafinado a cada
/// fixtura, e seria afinado até o número parecer bom.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FeatureOptions {
    /// O início da faixa de raios, **em múltiplos da aresta média**.
    pub r0_in_edges: f32,
    /// O fim da faixa, **em múltiplos do passo alvo da grade `h`**.
    pub r1_in_h: f32,
    /// Quantos raios se amostram na faixa.
    pub samples: usize,
    /// ⭐ A **meia-largura da janela**, em múltiplos de `h`.
    ///
    /// ⚠️ **Ela troca COBERTURA por CONFIANÇA:** grande de mais e nenhum ponto é
    /// válido; pequena de mais e a condição do degrau 2 deixa de filtrar o que existe.
    pub half_window_in_h: f32,
    /// O piso de anisotropia, adimensional em `[0, 1]`.
    pub min_anisotropy: f32,
    /// O piso de curvatura média, **em múltiplos de `1 / raio_da_caixa`**.
    pub min_curvature_in_bbox: f32,
}

impl Default for FeatureOptions {
    /// ⛔⛔ **Estes números são um PONTO DE PARTIDA da varredura, não um veredito.**
    /// Quem os fixa é a sonda, e a tabela dela tem de estar escrita ao lado do valor
    /// que shipar (`CLAUDE.md` §0.0).
    /// ⭐ **A TABELA QUE OS FIXOU** — varredura de 2026-08-25 sobre a peça do artista
    /// (`sculpt_t001`, 2 327 vértices depois do F1), com a esfera lisa como controlo:
    ///
    /// | `r₁/h` | janela | marcados na peça | ⭐ recusados **pela janela** | esfera |
    /// |---|---|---|---|---|
    /// | ⭐ **`2,0`** | ⭐ **`1,0`** | **`7,1 %`** | **`267`** | **`0,00 %`** |
    /// | `4,0` | `0,25` | `22,1 %` | ⛔ **`0`** | `0,00 %` |
    /// | `8,0` | `0,25` | `28,6 %` | ⛔ **`0`** | `0,00 %` |
    ///
    /// ⛔⛔ **A coluna que decide é a das recusas PELA JANELA, e ela mede se o degrau 2
    /// existe.** Com a janela pequena em relação à faixa ela recusa **zero** — a
    /// condição de estabilidade deixa de ser aplicada e a lei degenera na de um raio
    /// só, que é precisamente o que faz o ruído passar por feição.
    ///
    /// ⭐⭐ **E o controlo é o que torna isto uma medição:** a esfera lisa marca
    /// `0,00 %` em **todas** as 54 combinações, com os 2 525 vértices recusados pelo
    /// piso. *Uma detecção que marca a esfera está a ler ruído; nenhuma coluna sozinha
    /// distingue os dois erros.*
    ///
    /// ⚠️ **`7,1 %` ainda não é «esparso»** para a régua da espec (cada restrição força
    /// uma singularidade) — a varredura seguinte tem de subir os dois pisos e medir a
    /// contagem de singularidades ao lado, que é o gate nº7.
    fn default() -> Self {
        Self {
            r0_in_edges: 1.0,
            r1_in_h: 2.0,
            samples: 6,
            half_window_in_h: 1.0,
            min_anisotropy: 0.85,
            min_curvature_in_bbox: 0.05,
        }
    }
}

/// O que a detecção mediu de si própria.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FeatureReport {
    /// Vértices examinados.
    pub points: usize,
    /// ⭐ Vértices que geraram restrição.
    pub marked: usize,
    /// ⛔ Recusados porque **nenhum** raio passou os dois limiares.
    pub rejected_flat: usize,
    /// ⛔⛔ Recusados porque passaram nalgum raio mas **não em toda a janela** — é a
    /// contagem que mede o que o degrau 2 de facto filtra.
    ///
    /// ⚠️ *Se ela for zero, a janela não está a fazer nada e o `half_window` é pequeno
    /// de mais; se ela engolir tudo, é grande de mais.*
    pub rejected_window: usize,
    /// Vértices sem vizinhança utilizável em raio nenhum.
    pub rejected_degenerate: usize,
    /// A mediana do raio eleito, em unidades da peça.
    pub radius_p50: f32,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt();
    if l > 1.0e-20 {
        [a[0] / l, a[1] / l, a[2] / l]
    } else {
        [1.0, 0.0, 0.0]
    }
}

/// Um candidato: o que a segunda forma disse num raio.
#[derive(Clone, Copy)]
struct Candidate {
    radius: f32,
    anisotropy: f32,
    /// `|k₁ + k₂| / 2`, em `1/comprimento`.
    mean: f32,
    /// A direcção na moldura tangente.
    dir2: [f32; 2],
    ok: bool,
}

/// ⭐⭐⭐ **AS DIRECÇÕES DE FEIÇÃO, uma por vértice que a mereça.**
///
/// `h` é o passo alvo da grade, na unidade da peça — a mesma que o mapa usa.
#[must_use]
pub fn feature_dirs(mesh: &Mesh, h: f32, opts: FeatureOptions) -> (Vec<FeatureDir>, FeatureReport) {
    let pos = mesh.positions();
    let vn = mesh.normals();
    let b = mesh.bounds();
    let bbox_radius = 0.5 * dot(sub(b.max, b.min), sub(b.max, b.min)).sqrt();
    let mut edge_sum = 0.0f64;
    let mut edge_n = 0usize;
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let d = sub(pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            edge_sum += f64::from(dot(d, d).sqrt());
            edge_n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    let edge_mean = if edge_n > 0 {
        (edge_sum / edge_n as f64) as f32
    } else {
        h
    };

    let r0 = opts.r0_in_edges * edge_mean;
    let r1 = (opts.r1_in_h * h).max(r0 * 1.0001);
    let win = opts.half_window_in_h * h;
    let kmin = if bbox_radius > 0.0 {
        opts.min_curvature_in_bbox / bbox_radius
    } else {
        0.0
    };

    let mut rep = FeatureReport::default();
    let mut out: Vec<FeatureDir> = Vec::new();
    let mut radii: Vec<f32> = Vec::new();
    let mut scratch = QueryScratch::default();
    let mut hits: Vec<u32> = Vec::new();
    let mut pairs: Vec<([f32; 2], [f32; 2])> = Vec::new();
    let mut cands: Vec<Candidate> = Vec::with_capacity(opts.samples);

    for v in 0..pos.len() {
        rep.points += 1;
        let p = pos[v];
        let n = normalize(vn[v]);
        // A moldura tangente: `u` qualquer perpendicular a `n`, `w = n × u`.
        let seed = if n[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = normalize(cross(n, seed));
        let w = cross(n, u);

        // ── 1. A FAIXA.
        cands.clear();
        for i in 0..opts.samples.max(2) {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / (opts.samples.max(2) - 1) as f32;
            let r = r0 + t * (r1 - r0);
            mesh.verts_in_sphere(p, r, &mut scratch, &mut hits);
            pairs.clear();
            for &q in &hits {
                if q as usize == v {
                    continue;
                }
                let e = sub(pos[q as usize], p);
                let dn = sub(normalize(vn[q as usize]), n);
                pairs.push((
                    [dot(e, u), dot(e, w)],
                    [dot(dn, u), dot(dn, w)],
                ));
            }
            // ⚠️ Três pares é o mínimo para os três coeficientes; abaixo disso o
            // sistema é subdeterminado e a resposta honesta é «não há direcção».
            let Some((k1, k2, dir2)) = (pairs.len() >= 3)
                .then(|| second_form(&pairs))
                .flatten()
            else {
                cands.push(Candidate {
                    radius: r,
                    anisotropy: 0.0,
                    mean: 0.0,
                    dir2: [1.0, 0.0],
                    ok: false,
                });
                continue;
            };
            let a = anisotropy_of(k1, k2);
            let mean = ((k1 + k2) * 0.5).abs();
            cands.push(Candidate {
                radius: r,
                anisotropy: a,
                mean,
                dir2,
                ok: a >= opts.min_anisotropy && mean >= kmin,
            });
        }

        if cands.iter().all(|c| c.anisotropy == 0.0 && c.mean == 0.0) {
            rep.rejected_degenerate += 1;
            continue;
        }
        let any_ok = cands.iter().any(|c| c.ok);
        if !any_ok {
            rep.rejected_flat += 1;
            continue;
        }

        // ── 2. A JANELA decide a validade, e ── 3. a ELEIÇÃO é dentro dela.
        let mut best: Option<(f32, &Candidate)> = None;
        for c in &cands {
            if !c.ok {
                continue;
            }
            let window: Vec<&Candidate> = cands
                .iter()
                .filter(|o| (o.radius - c.radius).abs() <= win)
                .collect();
            if !window.iter().all(|o| o.ok) {
                continue;
            }
            // ⭐ A variação de direcção DENTRO DA JANELA — e o ângulo é de EIXO, então
            // `d` e `−d` são o mesmo: mede-se por `|cos|`.
            let spread = window
                .iter()
                .map(|o| {
                    let c2 = c.dir2[0].mul_add(o.dir2[0], c.dir2[1] * o.dir2[1]).abs();
                    c2.clamp(0.0, 1.0).acos()
                })
                .fold(0.0f32, f32::max);
            if best.is_none_or(|(s, _)| spread < s) {
                best = Some((spread, c));
            }
        }
        let Some((_, c)) = best else {
            rep.rejected_window += 1;
            continue;
        };
        radii.push(c.radius);
        #[allow(clippy::cast_possible_truncation)]
        out.push(FeatureDir {
            vert: v as u32,
            dir: normalize([
                c.dir2[0].mul_add(u[0], c.dir2[1] * w[0]),
                c.dir2[0].mul_add(u[1], c.dir2[1] * w[1]),
                c.dir2[0].mul_add(u[2], c.dir2[1] * w[2]),
            ]),
            radius: c.radius,
            anisotropy: c.anisotropy,
        });
    }
    rep.marked = out.len();
    radii.sort_by(f32::total_cmp);
    rep.radius_p50 = radii.get(radii.len() / 2).copied().unwrap_or(0.0);
    (out, rep)
}

#[cfg(test)]
#[path = "feature_dirs_tests.rs"]
mod tests;
