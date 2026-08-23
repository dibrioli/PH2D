//! ⭐⭐⭐ **G3 — RESOLVER `(u, v)` ALINHADO AO CAMPO, com as costuras acopladas.**
//!
//! # A energia, deduzida
//!
//! Queremos que andar `h` na direcção `X` da cruz faça `u` subir **exactamente 1**, e
//! que `v` não mexa. Isso é uma condição sobre os **gradientes**:
//!
//! ```text
//!     grad u = X / h        grad v = Y / h        (Y = n x X)
//! ```
//!
//! ⇒ a energia é o desvio disso, pesado pela área:
//!
//! ```text
//!     E = Σ_t A_t · [ |grad u_t − X_t/h|² + |grad v_t − Y_t/h|² ]
//! ```
//!
//! ⭐ **Não há condição de fronteira nenhuma.** É a diferença de espécie contra tudo o
//! que o F5 tentou: os quatro achatamentos dele impõem *onde o bordo cai* e pagam-no no
//! interior; aqui o bordo vai para onde o campo o mandar, e o que prende os patches uns
//! aos outros são as **costuras**, não um polígono inventado.
//!
//! # ⭐⭐⭐ Como os dois lados de uma costura se relacionam — deduzido, não adivinhado
//!
//! O [`crate::comb`] mede `k` como *o número de quartos de volta que leva a moldura do
//! lado 1 até à do lado 0*: `R^k X_b = X_a`. Com `X_b = (1,0)`, `Y_b = (0,1)` e `k = 1`
//! (`R(x,y) = (−y,x)`), vem `X_a = (0,1)` e `Y_a = (−1,0)`. Um ponto `q = (qx, qy)` lê-se
//! então:
//!
//! ```text
//!     z_b = (q·X_b, q·Y_b) = ( qx,  qy)
//!     z_a = (q·X_a, q·Y_a) = ( qy, −qx) = R^{−1} z_b
//! ```
//!
//! ⇒ **`z_b = R^k z_a + t`**, com `t` a translação da costura. ⚠️ *O sinal está
//! deduzido aqui e **confirmado pelo controlo plano**, onde a resposta exacta é
//! conhecida de antemão — um sinal trocado dá resíduo de costura grande e o gate
//! reprova.*
//!
//! # O passo de Gauss–Seidel fica DIAGONAL, e isso não é sorte
//!
//! A penalização da costura é `w · |z_b − R^k z_a − t|²`. Derivando em ordem ao **meu**
//! `z`, a rotação cai sobre o valor do **outro** — a forma quadrática em `(u_k, v_k)` é
//! `w · I`. ⇒ `u` e `v` continuam a resolver-se com o mesmo denominador, e o acoplamento
//! entra só como mais um termo no numerador.
//!
//! ⚠️ *Se a rotação caísse do meu lado, o sistema local seria `2×2` cheio e este
//! ficheiro seria outro.*

use ph2d_mesh::Mesh;

use crate::comb::Combed;
use crate::cut::CutMesh;

/// ⭐⭐ **QUANTO PESA UMA COSTURA contra o alinhamento ao campo.**
///
/// O peso é **relativo ao denominador de Poisson do próprio vértice**, e não absoluto —
/// assim ele não depende da escala da peça nem da densidade da malha. ⚠️ *Um peso
/// absoluto teria de ser reafinado a cada fixtura, e seria afinado até o número parecer
/// bom.*
///
/// # ⭐⭐⭐ O valor sai de MEDIÇÃO, e a medição mostra um COMPROMISSO real
///
/// Esfera `24×36`, `h` = aresta mediana da malha, à convergência:
///
/// | peso | ⭐ ângulo `grad u` vs `X` | escala | costura p50 | costura **max** |
/// |---|---|---|---|---|
/// | `1` | ⭐ **`4,1°`** | `0,89` | `0,141` | ⛔ **`2,98`** |
/// | `8` | `8,2°` | `0,76` | `0,029` | `0,90` |
/// | `64` | `12,3°` | `0,71` | `0,004` | `0,146` |
/// | ⭐ **`512`** | `13,0°` | `0,61` | **`0,0006`** | ⭐ **`0,017`** |
///
/// ⚠️ **É compromisso e não solver lento**, e isso foi medido: de `40 000` a `640 000`
/// rondas o ângulo a `w = 64` vai de `11,9°` a `12,3°` e a `w = 512` de `25,0°` a
/// `13,0°` — *os dois assentam*. **Fechar as costuras custa o alinhamento.**
///
/// ⭐ **`512` porque o G4 lê onde as isolinhas INTEIRAS cruzam cada arco**, e aí quem
/// manda é o resíduo da costura: `0,15` de célula seria `15 %` de desacordo sobre onde
/// a marca cai. *O ângulo paga `9°`; a marca ganha uma ordem de grandeza.*
///
/// ⚠️ **A escala baixa NÃO bloqueia**, e é por desenho: o G4 usa o mapa só para decidir
/// *onde ao longo de cada arco* as marcas caem, mantendo as CONTAGENS do F4 — a mesma
/// disciplina do `regraduate`. ⇒ *um factor de escala global sai na normalização por
/// arco.*
pub const SEAM_WEIGHT: f32 = 512.0;

/// Quantas rondas de Gauss–Seidel.
///
/// ⭐ **`160 000` é onde a tabela do [`SEAM_WEIGHT`] deixa de mudar** (`640 000` dá o
/// mesmo ângulo), não uma folga escolhida. *O sistema com costuras rígidas é mal
/// condicionado e Gauss–Seidel percorre-o devagar — a mesma lição do `lscm`: dois
/// solvers diferentes não partilham um teto de espera.*
pub const ROUNDS: usize = 160_000;

/// De quantas em quantas rondas a translação de cada costura é reajustada.
///
/// ⚠️ **Não é a cada ronda:** a translação é a média de um resíduo que ainda está a
/// assentar, e persegui-la a cada passo faz os dois oscilarem. *Deixa-se o campo
/// acomodar-se e só depois se move o alvo.*
pub const SHIFT_EVERY: usize = 50;

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

fn unit(a: [f32; 3]) -> Option<[f32; 3]> {
    let l = dot(a, a).sqrt();
    (l > 1.0e-12).then(|| [a[0] / l, a[1] / l, a[2] / l])
}

/// `z` rodado de `k` quartos de volta. `R(x, y) = (−y, x)`.
#[must_use]
pub fn turn2(z: [f32; 2], k: i32) -> [f32; 2] {
    match k.rem_euclid(4) {
        1 => [-z[1], z[0]],
        2 => [-z[0], -z[1]],
        3 => [z[1], -z[0]],
        _ => z,
    }
}

/// Um triângulo já preparado: gradientes de base, área e o alvo do campo.
struct Tri {
    v: [u32; 3],
    g: [[f32; 2]; 3],
    area: f32,
    /// `X/h` e `Y/h` no plano do triângulo.
    target: [[f32; 2]; 2],
}

/// Quem é o par de um vértice do outro lado de uma costura.
struct Partner {
    patch: u32,
    local: u32,
    seam: u32,
    /// `true` = eu sou o lado `0`, logo o alvo é `R^{−k}(z_par − t)`.
    /// `false` = eu sou o lado `1`, logo o alvo é `R^{k}(z_par) + t`.
    first: bool,
    jump: i32,
}

/// O mapa resolvido.
#[derive(Debug, Clone, Default)]
pub struct GridMap {
    /// Por patch, por vértice local, o `(u, v)` em **unidades de grade**.
    pub uv: Vec<Vec<[f32; 2]>>,
    /// Por costura, a translação ajustada.
    pub shift: Vec<[f32; 2]>,
}

/// O que o solver mediu de si próprio.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SolveReport {
    /// Rondas gastas.
    pub rounds: usize,
    /// Triângulos que entraram na energia.
    pub triangles: usize,
    /// ⚠️ Triângulos deixados de fora (degenerados, ou sem direcção penteada).
    pub skipped: usize,
    /// Pares de vértices acoplados por costuras.
    pub pairs: usize,
    /// ⛔ Costuras sem salto lido — **não acopladas**, e por isso contadas.
    pub loose_seams: usize,
    /// ⭐⭐⭐ **O DESVIO DO ALINHAMENTO**, em fracção do alvo (`0` = o gradiente é
    /// exactamente `X/h`). Mediana e `p95` sobre os triângulos.
    pub align_p50: f32,
    /// O `p95` do desvio do alinhamento.
    pub align_p95: f32,
    /// ⭐⭐⭐ **O ÂNGULO entre `grad u` e `X`, em graus** — *para que lado a grade
    /// aponta*.
    ///
    /// ⛔⛔ **Esta coluna e a irmã abaixo existem porque [`Self::align_p50`] mistura
    /// duas coisas muito diferentes:** a grade apontar para o lado errado (fatal) e a
    /// grade ter o tamanho errado (não fatal — as células ficam maiores ou menores, e a
    /// quantização já lida com isso). *Um número só não distingue as duas, e a primeira
    /// leitura de `0,33` seria «o solver não presta».*
    pub angle_p50: f32,
    /// O `p95` do ângulo.
    pub angle_p95: f32,
    /// ⭐ **A ESCALA:** `|grad u| · h`. `1` = a célula tem exactamente o passo pedido.
    pub scale_p50: f32,
    /// O `p95` da escala.
    pub scale_p95: f32,
    /// ⭐⭐⭐ **O RESÍDUO DAS COSTURAS**, em **unidades de grade** (`1` = uma célula
    /// inteira de desacordo). Mediana e pior.
    pub seam_p50: f32,
    /// O pior resíduo de costura.
    pub seam_max: f32,
}

/// Prepara os triângulos de um patch.
fn prepare(mesh: &Mesh, cut: &CutMesh, combed: &Combed, p: usize, h: f32) -> (Vec<Tri>, usize) {
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
        out.push(Tri {
            v: *t,
            g,
            area: 0.5 * two_a.abs(),
            target: [[x[0] / h, x[1] / h], [y[0] / h, y[1] / h]],
        });
    }
    (out, skipped)
}

/// ⭐⭐⭐ **RESOLVE O MAPA.** `h` é o passo alvo da grade, na unidade da peça.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn solve(mesh: &Mesh, cut: &CutMesh, combed: &Combed, h: f32) -> (GridMap, SolveReport) {
    solve_with(mesh, cut, combed, h, SEAM_WEIGHT, ROUNDS)
}

/// ⭐ **O MESMO, com o peso e as rondas explícitos.**
///
/// ⚠️ Existe porque as duas constantes **têm de ser mediveis** (`CLAUDE.md` §0.0): uma
/// sonda que não as pode varrer não as pode justificar, e elas ficariam a ser o palpite
/// de quem as escreveu.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn solve_with(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    weight: f32,
    rounds: usize,
) -> (GridMap, SolveReport) {
    let mut rep = SolveReport::default();
    let np = cut.tris.len();
    let mut tris: Vec<Vec<Tri>> = Vec::with_capacity(np);
    for p in 0..np {
        let (t, s) = prepare(mesh, cut, combed, p, h);
        rep.triangles += t.len();
        rep.skipped += s;
        tris.push(t);
    }

    // ── Os pares de costura, por patch e por vértice local.
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

    let mut map = GridMap {
        uv: cut
            .origin
            .iter()
            .map(|o| vec![[0.0f32; 2]; o.len()])
            .collect(),
        shift: vec![[0.0; 2]; cut.seams.len()],
    };

    // ── O denominador de Poisson de cada vértice, que não muda.
    let mut denom: Vec<Vec<f32>> = map.uv.iter().map(|u| vec![0.0f32; u.len()]).collect();
    for (p, ts) in tris.iter().enumerate() {
        for t in ts {
            for k in 0..3 {
                let g = t.g[k];
                denom[p][t.v[k] as usize] += t.area * g[0].mul_add(g[0], g[1] * g[1]);
            }
        }
    }

    for round in 0..rounds {
        for p in 0..np {
            // Numeradores de Poisson, acumulados sobre os triângulos incidentes.
            let mut num = vec![[0.0f32; 2]; map.uv[p].len()];
            for t in &tris[p] {
                let (z0, z1, z2) = (
                    map.uv[p][t.v[0] as usize],
                    map.uv[p][t.v[1] as usize],
                    map.uv[p][t.v[2] as usize],
                );
                let zs = [z0, z1, z2];
                for k in 0..3 {
                    // `Σ_{j≠k} z_j g_j`, os dois eixos numa passagem só.
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
                    num[t.v[k] as usize][0] += t.area * gu[0].mul_add(g[0], gu[1] * g[1]);
                    num[t.v[k] as usize][1] += t.area * gv[0].mul_add(g[0], gv[1] * g[1]);
                }
            }
            for l in 0..map.uv[p].len() {
                let base = denom[p][l];
                if base <= 0.0 {
                    continue;
                }
                let w = weight * base;
                let (mut nu, mut nv) = (num[l][0], num[l][1]);
                let mut den = base;
                for q in &partners[p][l] {
                    let other = map.uv[q.patch as usize][q.local as usize];
                    let t = map.shift[q.seam as usize];
                    // ⭐ `z_b = R^k z_a + t` — ver o doc deste módulo.
                    let want = if q.first {
                        turn2([other[0] - t[0], other[1] - t[1]], -q.jump)
                    } else {
                        let r = turn2(other, q.jump);
                        [r[0] + t[0], r[1] + t[1]]
                    };
                    nu += w * want[0];
                    nv += w * want[1];
                    den += w;
                }
                map.uv[p][l] = [nu / den, nv / den];
            }
        }

        // ── A translação de cada costura: a média do resíduo.
        if round % SHIFT_EVERY == SHIFT_EVERY - 1 {
            for (s, seam) in cut.seams.iter().enumerate() {
                if combed.jump.get(s).copied().flatten().is_none() {
                    continue;
                }
                let k = combed.jump[s].unwrap_or(0);
                let (pa, pb) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
                let (mut acc, mut n) = ([0.0f32; 2], 0.0f32);
                for (la, lb) in seam.side[0].local.iter().zip(&seam.side[1].local) {
                    let (Some(la), Some(lb)) = (la, lb) else {
                        continue;
                    };
                    let za = turn2(map.uv[pa][*la as usize], k);
                    let zb = map.uv[pb][*lb as usize];
                    acc[0] += zb[0] - za[0];
                    acc[1] += zb[1] - za[1];
                    n += 1.0;
                }
                if n > 0.0 {
                    map.shift[s] = [acc[0] / n, acc[1] / n];
                }
            }
        }
        rep.rounds = round + 1;
    }

    // ── ⭐ AS DUAS RÉGUAS.
    let mut align: Vec<f32> = Vec::with_capacity(rep.triangles);
    let mut angle: Vec<f32> = Vec::with_capacity(rep.triangles);
    let mut scale: Vec<f32> = Vec::with_capacity(rep.triangles);
    for (p, ts) in tris.iter().enumerate() {
        for t in ts {
            let mut gu = [0.0f32; 2];
            let mut gv = [0.0f32; 2];
            for k in 0..3 {
                let z = map.uv[p][t.v[k] as usize];
                gu[0] += z[0] * t.g[k][0];
                gu[1] += z[0] * t.g[k][1];
                gv[0] += z[1] * t.g[k][0];
                gv[1] += z[1] * t.g[k][1];
            }
            let du = [gu[0] - t.target[0][0], gu[1] - t.target[0][1]];
            let dv = [gv[0] - t.target[1][0], gv[1] - t.target[1][1]];
            let err =
                (du[0].mul_add(du[0], du[1] * du[1]) + dv[0].mul_add(dv[0], dv[1] * dv[1])).sqrt();
            // ⭐ **Relativo ao alvo**, senão o número depende da escala da peça.
            align.push(err * h / std::f32::consts::SQRT_2);
            // ⭐⭐⭐ E a decomposição: para que lado, e de que tamanho.
            let lu = gu[0].mul_add(gu[0], gu[1] * gu[1]).sqrt();
            let lt = t.target[0][0]
                .mul_add(t.target[0][0], t.target[0][1] * t.target[0][1])
                .sqrt();
            if lu > 1.0e-12 && lt > 1.0e-12 {
                let c = (gu[0].mul_add(t.target[0][0], gu[1] * t.target[0][1]) / (lu * lt))
                    .clamp(-1.0, 1.0);
                angle.push(c.acos().to_degrees());
                scale.push(lu / lt);
            }
        }
    }
    let mut seam: Vec<f32> = Vec::with_capacity(rep.pairs);
    for (s, sm) in cut.seams.iter().enumerate() {
        let Some(k) = combed.jump.get(s).copied().flatten() else {
            continue;
        };
        let (pa, pb) = (sm.side[0].patch as usize, sm.side[1].patch as usize);
        let t = map.shift[s];
        for (la, lb) in sm.side[0].local.iter().zip(&sm.side[1].local) {
            let (Some(la), Some(lb)) = (la, lb) else {
                continue;
            };
            let za = turn2(map.uv[pa][*la as usize], k);
            let zb = map.uv[pb][*lb as usize];
            let d = [zb[0] - za[0] - t[0], zb[1] - za[1] - t[1]];
            seam.push(d[0].mul_add(d[0], d[1] * d[1]).sqrt());
        }
    }
    let pct = |v: &mut Vec<f32>, q: f32| -> f32 {
        if v.is_empty() {
            return 0.0;
        }
        v.sort_by(f32::total_cmp);
        #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
        let i = ((v.len() - 1) as f32 * q).round() as usize;
        v[i.min(v.len() - 1)]
    };
    rep.align_p50 = pct(&mut align, 0.50);
    rep.align_p95 = pct(&mut align, 0.95);
    rep.angle_p50 = pct(&mut angle, 0.50);
    rep.angle_p95 = pct(&mut angle, 0.95);
    rep.scale_p50 = pct(&mut scale, 0.50);
    rep.scale_p95 = pct(&mut scale, 0.95);
    rep.seam_p50 = pct(&mut seam, 0.50);
    rep.seam_max = seam.last().copied().unwrap_or(0.0);

    (map, rep)
}

#[cfg(test)]
#[path = "solve_tests.rs"]
mod tests;
