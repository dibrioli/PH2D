//! ⭐⭐ **A MONTAGEM DO SISTEMA** — os triângulos preparados, os pares de costura e o
//! numerador de Poisson.
//!
//! ⚠️ **Ela vive num irmão do [`crate::solve`] por causa do tecto de LOC**, e o corte é
//! por RESPONSABILIDADE: aqui **monta-se** o sistema (o que cada triângulo pede, quem
//! está acoplado a quem), lá **resolve-se**. *Duas perguntas diferentes sobre o mesmo
//! mapa, e a segunda nunca precisa de saber como a primeira leu a malha.*

use ph2d_mesh::Mesh;

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{GridMap, Partner, SolveReport, Tri, cross, dot, sub, unit};

/// ⭐⭐⭐ **O PASSO DA GRADE — um número, ou um CAMPO por vértice.**
///
/// ⛔⛔ **Ele existe por um report do artista (2026-08-28): «as pontas finas, que deveriam
/// ser relativamente mais densas que as áreas lisas, têm menos densidade de faces e perdem
/// detalhes».** E a medição confirma-o com um número: na saída dele o expoente de
/// `aresta ∼ curvatura^n` é **`−0,003`** sobre uma faixa de curvatura de **`9,4×`** — *a
/// grade é rigorosamente uniforme, e onde a forma aperta nove vezes mais os quads têm
/// exactamente o mesmo tamanho.*
///
/// ⭐ **O passo entra no sistema num sítio só** — o gradiente alvo de cada triângulo, que
/// era `direcção / h`. Torná-lo `direcção / h(x)` é a *sizing field* clássica da família, e
/// é o que o *Adaptive Size* do ZBrush e a escala por curvatura do Instant Meshes fazem.
/// ⚠️ **A extracção é AGNÓSTICA a isto:** ela lê as isolinhas **inteiras** do mapa, e um
/// passo que varia deforma o mapa sem mexer no que é inteiro.
///
/// ⚠️ **A gradação tem de ser LIMITADA**, e a cerca já existia noutra crate com a razão
/// escrita: `ph2d_quadflow::MAX_ADAPTIVE_RATIO` (*«duas células cujas escalas diferem por
/// mais do que isto deixam de ter aresta comum — a grade rasga em vez de transitar»*).
#[derive(Clone, Copy, Debug)]
pub struct Step<'a> {
    /// O passo médio pedido — o valor quando não há campo, e o que as réguas usam.
    pub h: f32,
    /// O passo **por vértice da malha de trabalho**. Vazio = uniforme.
    pub per_vertex: &'a [f32],
}

impl Step<'static> {
    /// O passo constante — o que todo chamador de antes de 2026-08-28 pede.
    #[must_use]
    pub const fn uniform(h: f32) -> Self {
        Self { h, per_vertex: &[] }
    }
}

impl Step<'_> {
    /// ⭐ O passo **deste triângulo** — a média dos três vértices, ou o escalar.
    ///
    /// ⚠️ **A média e não o mínimo:** o mínimo faria um vértice apertado encolher o
    /// triângulo inteiro e a gradação deixaria de ser suave — que é exactamente o que a
    /// cerca da razão máxima existe para impedir.
    ///
    /// ⚠️ **Pública desde 2026-08-30**: a [`crate::injective_solve`] mede a energia num
    /// referencial de repouso que tem de estar **em unidades de célula**, e a conversão é
    /// exactamente este passo. *Reimplementá-la lá seria uma segunda média a divergir desta.*
    #[must_use]
    pub fn at(&self, v: &[u32]) -> f32 {
        if self.per_vertex.is_empty() {
            return self.h;
        }
        let mut sum = 0.0f32;
        let mut n = 0u32;
        for &i in v {
            if let Some(x) = self.per_vertex.get(i as usize)
                && x.is_finite()
                && *x > 0.0
            {
                sum += *x;
                n += 1;
            }
        }
        if n == 0 { self.h } else { sum / n as f32 }
    }
}

/// Prepara os triângulos de um patch.
pub(crate) fn prepare(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    p: usize,
    step: Step<'_>,
) -> (Vec<Tri>, usize) {
    let pos = mesh.positions();
    let mut out = Vec::with_capacity(cut.tris[p].len());
    let mut skipped = 0usize;
    for (i, t) in cut.tris[p].iter().enumerate() {
        let f = cut.tri_face[p][i];
        let v = mesh.faces()[f as usize].verts();
        let (p0, p1, p2) = (pos[v[0] as usize], pos[v[1] as usize], pos[v[2] as usize]);
        let (Some(e1), Some(n)) = (unit(sub(p1, p0)), unit(cross(sub(p1, p0), sub(p2, p0)))) else {
            skipped += 1;
            continue;
        };
        let e2 = cross(n, e1);
        let q1 = [dot(sub(p1, p0), e1), dot(sub(p1, p0), e2)];
        let q2 = [dot(sub(p2, p0), e1), dot(sub(p2, p0), e2)];
        let two_a = q1[0].mul_add(q2[1], -(q1[1] * q2[0]));
        if two_a.abs() < 1.0e-20 {
            skipped += 1;
            continue;
        }
        let inv = 1.0 / two_a;
        // `q0` está na origem.
        let g = [
            [(q1[1] - q2[1]) * inv, (q2[0] - q1[0]) * inv],
            [q2[1] * inv, -q2[0] * inv],
            [-q1[1] * inv, q1[0] * inv],
        ];
        let Some(&d) = combed.dir[p].get(i) else {
            skipped += 1;
            continue;
        };
        let x = [dot(d, e1), dot(d, e2)];
        // ⭐ `Y = n × X` lê-se no plano como `R(x, y) = (−y, x)` — ver [`turn2`].
        let y = [-x[1], x[0]];
        // ⭐⭐⭐ **O passo é DESTE triângulo** — ver [`Step`]. ⚠️ Os índices são os da malha
        // de trabalho (`v`), e não os locais do patch: o campo é uma propriedade da
        // superfície, e o corte é uma re-indexação dela.
        let h = step.at(v).max(1.0e-9);
        out.push(Tri {
            v: *t,
            g,
            area: 0.5 * two_a.abs(),
            target: [[x[0] / h, x[1] / h], [y[0] / h, y[1] / h]],
        });
    }
    (out, skipped)
}

/// ⭐ **O SISTEMA MONTADO** — os triângulos preparados, os pares de costura e o
/// denominador de Poisson de cada vértice.
///
/// ⚠️ **Ele existe como PORTA e não como conveniência.** O arredondamento inteiro
/// ([`crate::round`]) relaxa o mesmo sistema, um vértice de cada vez, e montá-lo por
/// conta própria seria escrever a mesma lei duas vezes — com as duas a divergirem em
/// silêncio no dia em que uma delas mudasse.
pub(crate) struct Assembly {
    pub tris: Vec<Vec<Tri>>,
    /// Por patch, por vértice local, quem é o par dele do outro lado de cada costura.
    pub partners: Vec<Vec<Vec<Partner>>>,
    /// O denominador de Poisson de cada vértice, que não muda com o mapa.
    pub denom: Vec<Vec<f32>>,
    /// Por patch, por vértice local, os triângulos incidentes.
    ///
    /// ⚠️ **Vive aqui e não no relaxador** porque agora há TRÊS leitores (a varredura
    /// do [`crate::round::Relaxer`], a classe soldada e a translação): *a mesma
    /// incidência construída em três sítios divergiria no dia em que um deles
    /// mudasse.*
    pub by_vert: Vec<Vec<Vec<u32>>>,
}

/// Monta o sistema, somando ao relatório o que a montagem mede.
pub(crate) fn assemble(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    step: Step<'_>,
    rep: &mut SolveReport,
) -> Assembly {
    let np = cut.tris.len();
    let mut tris: Vec<Vec<Tri>> = Vec::with_capacity(np);
    for p in 0..np {
        let (t, s) = prepare(mesh, cut, combed, p, step);
        rep.triangles += t.len();
        rep.skipped += s;
        tris.push(t);
    }

    let mut partners: Vec<Vec<Vec<Partner>>> = cut
        .origin
        .iter()
        .map(|o| (0..o.len()).map(|_| Vec::new()).collect())
        .collect();
    for (s, seam) in cut.seams.iter().enumerate() {
        let Some(k) = combed.jump.get(s).copied().flatten() else {
            rep.loose_seams += 1;
            continue;
        };
        let Ok(sid) = u32::try_from(s) else {
            continue;
        };
        let (pa, pb) = (seam.side[0].patch, seam.side[1].patch);
        for (la, lb) in seam.side[0].local.iter().zip(&seam.side[1].local) {
            let (Some(la), Some(lb)) = (la, lb) else {
                continue;
            };
            partners[pa as usize][*la as usize].push(Partner {
                patch: pb,
                local: *lb,
                seam: sid,
                first: true,
                jump: k,
            });
            partners[pb as usize][*lb as usize].push(Partner {
                patch: pa,
                local: *la,
                seam: sid,
                first: false,
                jump: k,
            });
            rep.pairs += 1;
        }
    }

    let mut denom: Vec<Vec<f32>> = cut.origin.iter().map(|o| vec![0.0f32; o.len()]).collect();
    for (p, ts) in tris.iter().enumerate() {
        for t in ts {
            for k in 0..3 {
                let g = t.g[k];
                denom[p][t.v[k] as usize] += t.area * g[0].mul_add(g[0], g[1] * g[1]);
            }
        }
    }
    let mut by_vert: Vec<Vec<Vec<u32>>> = denom.iter().map(|d| vec![Vec::new(); d.len()]).collect();
    for (p, ts) in tris.iter().enumerate() {
        for (i, t) in ts.iter().enumerate() {
            for k in 0..3 {
                #[allow(clippy::cast_possible_truncation)]
                by_vert[p][t.v[k] as usize].push(i as u32);
            }
        }
    }
    Assembly {
        tris,
        partners,
        denom,
        by_vert,
    }
}

impl Assembly {
    /// ⭐⭐⭐ **OS TRIÂNGULOS CUJA IMAGEM ESTÁ VIRADA** — `(patch, índice)`.
    ///
    /// ⚠️ **É a mesma lei da sonda independente do `chain_info`** (o sinal da área da
    /// imagem), e de propósito: as duas medem a mesma coisa por caminhos diferentes, e é
    /// isso que torna uma delas um controlo da outra.
    ///
    /// ⚠️ Dentro de um patch não há transformação de costura a aplicar — os três vértices
    /// são locais. *É por isso que a dobra se pode ler aqui sem reconstruir o mapa de
    /// cantos.*
    pub(crate) fn folded(&self, map: &GridMap) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for (p, ts) in self.tris.iter().enumerate() {
            for (i, t) in ts.iter().enumerate() {
                let z = [
                    map.uv[p][t.v[0] as usize],
                    map.uv[p][t.v[1] as usize],
                    map.uv[p][t.v[2] as usize],
                ];
                let d = (z[1][0] - z[0][0]).mul_add(
                    z[2][1] - z[0][1],
                    -((z[1][1] - z[0][1]) * (z[2][0] - z[0][0])),
                );
                if d < 0.0 {
                    out.push((p, i));
                }
            }
        }
        out
    }

    /// ⭐⭐⭐ **ENDURECE os triângulos dados** — multiplica o peso deles na energia e
    /// reconstrói o denominador de Poisson.
    ///
    /// ⛔ **O denominador TEM de ser refeito**, e não é detalhe: ele é `Σ area·|g|²` por
    /// vértice e é o divisor de toda relaxação. *Mudar o peso e deixar o denominador para
    /// trás é resolver um sistema com a matriz de um e o lado direito do outro.*
    pub(crate) fn stiffen(&mut self, who: &[(usize, usize)], factor: f32) {
        for &(p, i) in who {
            self.tris[p][i].area *= factor;
        }
        for d in &mut self.denom {
            d.fill(0.0);
        }
        for (p, ts) in self.tris.iter().enumerate() {
            for t in ts {
                for k in 0..3 {
                    let g = t.g[k];
                    self.denom[p][t.v[k] as usize] += t.area * g[0].mul_add(g[0], g[1] * g[1]);
                }
            }
        }
    }
}

/// ⭐⭐ **O NUMERADOR DE POISSON de um vértice** — a soma, sobre os triângulos
/// incidentes, do que a energia de orientação pede a este vértice.
///
/// ⚠️ **Porta única.** Ele era escrito à mão em dois sítios (a varredura por patch e o
/// relaxador); com a soldadura passariam a ser quatro. *A mesma equação em quatro
/// sítios é quatro equações à espera de divergirem.*
pub(crate) fn poisson_numerator(a: &Assembly, map: &GridMap, p: usize, l: usize) -> [f32; 2] {
    let (mut nu, mut nv) = (0.0f32, 0.0f32);
    for &ti in &a.by_vert[p][l] {
        let t = &a.tris[p][ti as usize];
        let Some(k) = (0..3).find(|&k| t.v[k] as usize == l) else {
            continue;
        };
        let zs = [
            map.uv[p][t.v[0] as usize],
            map.uv[p][t.v[1] as usize],
            map.uv[p][t.v[2] as usize],
        ];
        let (mut rest, mut rest_v) = ([0.0f32; 2], [0.0f32; 2]);
        for (j, (z, g)) in zs.iter().zip(&t.g).enumerate() {
            if j == k {
                continue;
            }
            rest[0] += z[0] * g[0];
            rest[1] += z[0] * g[1];
            rest_v[0] += z[1] * g[0];
            rest_v[1] += z[1] * g[1];
        }
        let gu = [t.target[0][0] - rest[0], t.target[0][1] - rest[1]];
        let gv = [t.target[1][0] - rest_v[0], t.target[1][1] - rest_v[1]];
        let g = t.g[k];
        nu += t.area * gu[0].mul_add(g[0], gu[1] * g[1]);
        nv += t.area * gv[0].mul_add(g[0], gv[1] * g[1]);
    }
    [nu, nv]
}
