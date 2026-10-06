//! ⭐⭐⭐ **A COSTURA** — o que se cose num quadro e onde entra na ordem das faces (irmão do
//! [`crate::skin_image_fecho`], cortado pelo tecto de LOC). O porquê da lei mora no cabeçalho dele.

use std::collections::BTreeMap;

use ph2d_poly2d::Mesh2d;
use ph2d_skeleton::{Correccao, Skin, Xform};

use crate::skin_image_arte::{BordasDaMalha, PontoDaArte};
use crate::skin_image_fecho::{OSSOS_DE_DISTANCIA, VAO_MAXIMO_EM_TEXELS};

/// Um encontro da borda com a outra parte: `(segmento, t nele, distância)`.
type Encontro = (usize, f64, f64);

/// O que a costura acrescenta: pontos POSADOS, a UV de cada um e os triângulos.
#[derive(Default)]
pub(crate) struct Costura {
    pub(crate) local: Vec<[f32; 2]>,
    pub(crate) uv: Vec<[f32; 2]>,
    pub(crate) tris: Vec<[u32; 3]>,
    /// Por triângulo: o triângulo da malha de onde vem a beira dele ([`junta`]).
    pub(crate) slot: Vec<u32>,
}

/// Um segmento da borda posada: as pontas (índices em `pos`/`uv`), o anel e o vizinho de cada lado.
#[derive(Clone, Copy)]
struct Seg {
    a: usize,
    b: usize,
    antes: usize,
    depois: usize,
    /// O triângulo da malha que tem este segmento (`u32::MAX`: nenhum).
    tri: u32,
}

/// ⭐⭐⭐ **O que a costura acrescenta nesta pose** — vazia quando nada encara nada.
pub(crate) fn costura(
    mesh: &Mesh2d,
    bordas: &BordasDaMalha,
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    [anchor, size]: [[f32; 2]; 2],
    correcoes: &[Correccao],
) -> Costura {
    let mut out = Costura::default();
    let aneis = &bordas.aneis;
    let ids: Vec<u32> = aneis.iter().flatten().copied().collect();
    if ids.is_empty() || mesh.rest.is_empty() {
        return out;
    }
    // ⭐ **A borda posa-se pela PORTA da malha** — uma malha só de borda, sem triângulos, com a
    // tabela das mesmas linhas: uma segunda conta da pose divergiria da que a arte desenha.
    let ossos = pesos.len() / mesh.rest.len();
    let linhas: Vec<f64> = ids
        .iter()
        .flat_map(|&v| {
            let v = v as usize;
            pesos
                .get(v * ossos..(v + 1) * ossos)
                .unwrap_or(&[])
                .iter()
                .copied()
        })
        .collect();
    let so_borda = Mesh2d {
        rest: ids.iter().map(|&v| mesh.rest[v as usize]).collect(),
        tris: Vec::new(),
        size: mesh.size,
    };
    let Some(borda) = crate::skin_image::posed_sprite_mesh_corrigida(
        so_borda, p2l, pele, &linhas, anchor, size, correcoes,
    ) else {
        return out;
    };
    let mut pos: Vec<[f64; 2]> = borda
        .local
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1])])
        .collect();
    let mut uv = borda.uv;
    let mut segs = Vec::with_capacity(pos.len());
    let mut k0 = 0;
    for (r, anel) in aneis.iter().enumerate() {
        let n = anel.len();
        for i in 0..n {
            // Um segmento por nó: o índice do segmento `i` do anel é o do nó `i`.
            segs.push(Seg {
                a: k0 + i,
                b: k0 + (i + 1) % n,
                antes: k0 + (i + n - 1) % n,
                depois: k0 + (i + 1) % n,
                tri: bordas
                    .faces
                    .get(r)
                    .and_then(|f| f.get(i))
                    .copied()
                    .unwrap_or(u32::MAX),
            });
        }
        k0 += n;
    }
    // O lado de FORA: o do anel de maior área (o de fora) diz o sentido de todos.
    let mut sinal = sinal_dos_aneis(&pos, aneis.iter().map(Vec::len));
    let [a, b, c, d, _, _] = p2l.0;
    let texel = (a * d - b * c).abs().sqrt();
    let vao = VAO_MAXIMO_EM_TEXELS * texel;
    if !(vao > 0.0 && vao.is_finite()) || sinal == 0.0 {
        return out;
    }
    // ⭐⭐ **Só se cose entre partes a mais de [`OSSOS_DE_DISTANCIA`]** — a chave de cada nó é a da
    // ordem das faces (`Σ wⱼ·j / Σ wⱼ`).
    if ossos < 2 {
        return out;
    }
    let mut chave: Vec<f64> = linhas.chunks_exact(ossos).map(chave_da_coluna).collect();
    // ⭐ **A saída rápida**: os segmentos por faixa de `1/4` de osso, a caixa de cada faixa, e só se
    // segue quando duas faixas que PODEM estar a mais de `OSSOS_DE_DISTANCIA` (os índices a `≥ 5`
    // faixas: a diferença entre membros delas chega a `(|i − j| + 1)/4`) têm as caixas a menos de
    // `2·vão`. Na pose recta e nas dobras sem contacto a costura custa a borda posada e isto.
    {
        let chave_do = |s: &Seg| 0.5 * (chave[s.a] + chave[s.b]);
        let mut faixas: BTreeMap<i64, [f64; 4]> = BTreeMap::new();
        for sg in &segs {
            #[expect(clippy::cast_possible_truncation, reason = "faixa de osso")]
            let f = (chave_do(sg) * 4.0).floor() as i64;
            let c = faixas
                .entry(f)
                .or_insert([f64::MAX, f64::MAX, f64::MIN, f64::MIN]);
            for q in [pos[sg.a], pos[sg.b]] {
                *c = [
                    c[0].min(q[0]),
                    c[1].min(q[1]),
                    c[2].max(q[0]),
                    c[3].max(q[1]),
                ];
            }
        }
        let perto = |a: &[f64; 4], b: &[f64; 4]| {
            a[0] - 2.0 * vao <= b[2]
                && b[0] - 2.0 * vao <= a[2]
                && a[1] - 2.0 * vao <= b[3]
                && b[1] - 2.0 * vao <= a[3]
        };
        let ha_par = faixas
            .iter()
            .any(|(i, a)| faixas.range(i + 5..).any(|(_, b)| perto(a, b)));
        if !ha_par {
            return out;
        }
    }
    // ⭐⭐⭐ A5-a: com a máscara da tinta, o vão mede-se e cose-se sobre o ANEL DA ARTE (a malha passa
    // da tinta; na cúspide de uma tampa ela dizia «sem vão»). A saída rápida acima fica na malha:
    // ela passa da arte, logo se a malha não encara nada a arte também não.
    if !bordas.arte.is_empty()
        && let Some(b) = borda_da_arte(
            mesh,
            &bordas.arte,
            p2l,
            pele,
            pesos,
            [anchor, size],
            correcoes,
            ossos,
        )
    {
        (pos, uv, chave, segs, sinal) = b;
    }
    let normal = |s: &Seg| {
        let (p, q) = (pos[s.a], pos[s.b]);
        let (tx, ty) = (q[0] - p[0], q[1] - p[1]);
        let l = tx.hypot(ty).max(f64::MIN_POSITIVE);
        [ty / l * sinal, -tx / l * sinal]
    };
    let chave_do = |s: &Seg| 0.5 * (chave[s.a] + chave[s.b]);
    let longe = |si: usize, sj: usize| {
        let (a, b) = (&segs[si], &segs[sj]);
        sj != si
            && sj != a.antes
            && sj != a.depois
            && (chave_do(a) - chave_do(b)).abs() > OSSOS_DE_DISTANCIA
    };
    // A grelha dos segmentos, com células de `2·vão`: um ponto encontra numa vizinhança 3×3 todo
    // segmento a menos de `2·vão` dele — o dobro do que se cose, para as pontas se interpolarem.
    // ⚠️ Um vector ORDENADO por célula e não um mapa: é refeita a cada quadro.
    let celula = 2.0 * vao;
    #[expect(clippy::cast_possible_truncation, reason = "células de uma sprite")]
    let cel = |x: f64| (x / celula).floor() as i64;
    let mut grelha: Vec<((i64, i64), usize)> = Vec::with_capacity(segs.len() * 2);
    for (si, s) in segs.iter().enumerate() {
        let (p, q) = (pos[s.a], pos[s.b]);
        for gx in cel(p[0].min(q[0]))..=cel(p[0].max(q[0])) {
            for gy in cel(p[1].min(q[1]))..=cel(p[1].max(q[1])) {
                grelha.push(((gx, gy), si));
            }
        }
    }
    grelha.sort_unstable();
    let na_celula = |c: (i64, i64)| {
        let ini = grelha.partition_point(|e| e.0 < c);
        grelha[ini..]
            .iter()
            .take_while(move |e| e.0 == c)
            .map(|e| e.1)
    };
    // O ponto mais perto da OUTRA parte que encara `p` (do segmento `si`): `(segmento, t, distância)`.
    let encara = |p: [f64; 2], si: usize| -> Option<Encontro> {
        let n = normal(&segs[si]);
        let mut melhor: Option<Encontro> = None;
        for gx in cel(p[0]) - 1..=cel(p[0]) + 1 {
            for gy in cel(p[1]) - 1..=cel(p[1]) + 1 {
                for sj in na_celula((gx, gy)) {
                    if !longe(si, sj) {
                        continue;
                    }
                    let o = segs[sj];
                    let (q0, q1) = (pos[o.a], pos[o.b]);
                    let dd = [q1[0] - q0[0], q1[1] - q0[1]];
                    let l2 = dd[0] * dd[0] + dd[1] * dd[1];
                    let t = if l2 > 0.0 {
                        (((p[0] - q0[0]) * dd[0] + (p[1] - q0[1]) * dd[1]) / l2).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let w = [q0[0] + t * dd[0] - p[0], q0[1] + t * dd[1] - p[1]];
                    let no = normal(&o);
                    // Os dois de FORA um do outro: o vão está à frente de cada borda.
                    if w[0] * n[0] + w[1] * n[1] <= 0.0 || w[0] * no[0] + w[1] * no[1] >= 0.0 {
                        continue;
                    }
                    let dist = w[0].hypot(w[1]);
                    if melhor.is_none_or(|m| dist < m.2) {
                        melhor = Some((sj, t, dist));
                    }
                }
            }
        }
        melhor
    };
    // Só se amostra o segmento que tem um candidato LONGE nas células à volta dele.
    let tem_candidato = |si: usize| {
        let (p, q) = (pos[segs[si].a], pos[segs[si].b]);
        (cel(p[0].min(q[0])) - 1..=cel(p[0].max(q[0])) + 1).any(|gx| {
            (cel(p[1].min(q[1])) - 1..=cel(p[1].max(q[1])) + 1)
                .any(|gy| na_celula((gx, gy)).any(|sj| longe(si, sj)))
        })
    };
    let uv_em = |s: &Seg, t: f64| {
        let (u0, u1) = (uv[s.a], uv[s.b]);
        #[expect(clippy::cast_possible_truncation, reason = "t em [0, 1]")]
        let t = t as f32;
        [u0[0] + t * (u1[0] - u0[0]), u0[1] + t * (u1[1] - u0[1])]
    };
    let ponto = |s: &Seg, t: f64| {
        let (p, q) = (pos[s.a], pos[s.b]);
        [p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])]
    };
    for (si, s) in segs.iter().enumerate() {
        if !tem_candidato(si) {
            continue;
        }
        let (p, q) = (pos[s.a], pos[s.b]);
        let comprimento = (q[0] - p[0]).hypot(q[1] - p[1]);
        // Uma amostra por texel, as duas pontas incluídas.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "amostras"
        )]
        let n = ((comprimento / texel).ceil() as usize).max(1);
        #[expect(clippy::cast_precision_loss, reason = "amostras")]
        let amostras: Vec<(f64, Option<Encontro>)> = (0..=n)
            .map(|k| {
                let t = k as f64 / n as f64;
                (t, encara(ponto(s, t), si))
            })
            .collect();
        // ⚠️ Cada vão cose-se de UM lado só — o do segmento de índice menor —, senão as duas metades
        // pintavam a mesma faixa duas vezes (uma beira translúcida sairia mais escura).
        let dentro = |m: &Option<Encontro>| m.is_some_and(|(sj, _, d)| sj > si && d < vao);
        for par in amostras.windows(2) {
            let ((t0, m0), (t1, m1)) = (par[0], par[1]);
            let (d0, d1) = (dentro(&m0), dentro(&m1));
            // ⭐ A ponta de um troço é onde o vão REAL passa `vao` — bissecção sobre a distância, e
            // não a amostra nem uma interpolação: é isto que faz a costura crescer e encolher
            // CONTÍNUA com a pose. ⛔ A 1.ª redacção interpolava a distância linearmente entre as
            // amostras, e quando o ponto mais perto muda de segmento entre elas a conta mente: medido,
            // pedaços a coser vãos de `3,9` texels (a lei é `2`).
            let fronteira = |mut t_in: f64, mut t_out: f64| {
                for _ in 0..12 {
                    let t = 0.5 * (t_in + t_out);
                    if dentro(&encara(ponto(s, t), si)) {
                        t_in = t;
                    } else {
                        t_out = t;
                    }
                }
                t_in
            };
            let (ta, tb) = match (d0, d1) {
                (true, true) => (t0, t1),
                (true, false) => (t0, fronteira(t0, t1)),
                (false, true) => (fronteira(t1, t0), t1),
                (false, false) => continue,
            };
            let (pa, pb) = (ponto(s, ta), ponto(s, tb));
            let (Some(fa), Some(fb)) = (encara(pa, si), encara(pb, si)) else {
                continue;
            };
            // ⭐⭐ A5-a: cada METADE do vão estica a cor da SUA beira até ao meio — um quadrilátero
            // só misturava a UV de um membro com a do outro e apanhava a textura entre as duas (uma
            // risca castanha das pintas na cúspide, FOTOGRAFADA).
            let (qa, qb) = (ponto(&segs[fa.0], fa.1), ponto(&segs[fb.0], fb.1));
            let meio = |a: [f64; 2], b: [f64; 2]| [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
            let (ma, mb) = (meio(pa, qa), meio(pb, qb));
            let (ua, ub) = (uv_em(s, ta), uv_em(s, tb));
            let (va, vb) = (uv_em(&segs[fa.0], fa.1), uv_em(&segs[fb.0], fb.1));
            let base = out.local.len() as u32;
            for (pt, uv) in [
                (pa, ua),
                (pb, ub),
                (mb, ub),
                (ma, ua),
                (ma, va),
                (mb, vb),
                (qb, vb),
                (qa, va),
            ] {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "metros locais de uma sprite"
                )]
                out.local.push([pt[0] as f32, pt[1] as f32]);
                out.uv.push(uv);
            }
            for (b, tri) in [(base, s.tri), (base + 4, segs[fa.0].tri)] {
                out.tris.push([b, b + 1, b + 2]);
                out.tris.push([b, b + 2, b + 3]);
                out.slot.extend([tri, tri]);
            }
        }
    }
    out
}

/// A chave de uma linha de pesos pela COLUNA (`Σ wⱼ·j / Σ wⱼ`) — a da costura.
fn chave_da_coluna(w: &[f64]) -> f64 {
    let soma: f64 = w.iter().sum();
    #[expect(clippy::cast_precision_loss, reason = "índice de osso")]
    let pos: f64 = w.iter().enumerate().map(|(j, p)| p * j as f64).sum();
    if soma > 0.0 { pos / soma } else { 0.0 }
}

/// O sentido de fora: o do anel de maior área diz o de todos (`pos` são os anéis em sequência).
fn sinal_dos_aneis(pos: &[[f64; 2]], tamanhos: impl Iterator<Item = usize>) -> f64 {
    let mut k = 0;
    let mut maior = 0.0_f64;
    for n in tamanhos {
        let area: f64 = (0..n)
            .map(|i| {
                let (p, q) = (pos[k + i], pos[k + (i + 1) % n]);
                p[0] * q[1] - q[0] * p[1]
            })
            .sum();
        if area.abs() > maior.abs() {
            maior = area;
        }
        k += n;
    }
    maior.signum()
}

/// A borda que a costura mede: posições POSADAS, UV, chave e segmentos, e o sentido de fora.
type Borda = (Vec<[f64; 2]>, Vec<[f32; 2]>, Vec<f64>, Vec<Seg>, f64);

/// ⭐⭐⭐ **A borda posada sobre o ANEL DA ARTE** (A5-a) — cada ponto é a mistura baricêntrica dos
/// vértices POSADOS do seu triângulo, posados pela MESMA porta da malha: ele está onde a imagem
/// desenha aquele texel, com a UV dele.
#[expect(clippy::too_many_arguments, reason = "a porta da malha mais o anel")]
fn borda_da_arte(
    mesh: &Mesh2d,
    arte: &[Vec<PontoDaArte>],
    p2l: Xform,
    pele: &Skin,
    pesos: &[f64],
    [anchor, size]: [[f32; 2]; 2],
    correcoes: &[Correccao],
    ossos: usize,
) -> Option<Borda> {
    let mut ids: Vec<u32> = arte
        .iter()
        .flatten()
        .flat_map(|p| mesh.tris[p.tri as usize])
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let linha = |v: u32| {
        pesos
            .get(v as usize * ossos..(v as usize + 1) * ossos)
            .unwrap_or(&[])
    };
    let linhas: Vec<f64> = ids.iter().flat_map(|&v| linha(v).iter().copied()).collect();
    let so = Mesh2d {
        rest: ids.iter().map(|&v| mesh.rest[v as usize]).collect(),
        tris: Vec::new(),
        size: mesh.size,
    };
    let posada = crate::skin_image::posed_sprite_mesh_corrigida(
        so, p2l, pele, &linhas, anchor, size, correcoes,
    )?;
    let (mut pos, mut uv, mut chave, mut segs) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for anel in arte {
        let (base, n) = (pos.len(), anel.len());
        for (i, p) in anel.iter().enumerate() {
            let t = mesh.tris[p.tri as usize];
            let w = [1.0 - p.uv[0] - p.uv[1], p.uv[0], p.uv[1]];
            let k = [0, 1, 2].map(|j| ids.binary_search(&t[j]).ok());
            let [Some(k0), Some(k1), Some(k2)] = k else {
                return None;
            };
            let ks = [k0, k1, k2];
            pos.push([0, 1].map(|c| {
                (0..3)
                    .map(|j| w[j] * f64::from(posada.local[ks[j]][c]))
                    .sum()
            }));
            // A COR: o ponto mais para dentro da tinta ([`PontoDaArte::cor`]).
            #[expect(clippy::cast_possible_truncation, reason = "UV em f32 como a da malha")]
            uv.push([0, 1].map(|c| (p.cor[c] / f64::from(mesh.size[c])) as f32));
            let row: Vec<f64> = (0..ossos)
                .map(|o| {
                    (0..3)
                        .map(|j| w[j] * linha(t[j]).get(o).copied().unwrap_or(0.0))
                        .sum()
                })
                .collect();
            chave.push(chave_da_coluna(&row));
            segs.push(Seg {
                a: base + i,
                b: base + (i + 1) % n,
                antes: base + (i + n - 1) % n,
                depois: base + (i + 1) % n,
                tri: p.tri,
            });
        }
    }
    let sinal = sinal_dos_aneis(&pos, arte.iter().map(Vec::len));
    Some((pos, uv, chave, segs, sinal))
}

/// O triângulo que tem cada segmento de cada anel (a aresta `anel[i] → anel[i + 1]`).
pub(crate) fn faces_dos_aneis(tris: &[[u32; 3]], aneis: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut arestas: Vec<((u32, u32), u32)> = (0_u32..)
        .zip(tris)
        .flat_map(|(k, t)| [((t[0], t[1]), k), ((t[1], t[2]), k), ((t[2], t[0]), k)])
        .collect();
    arestas.sort_unstable();
    aneis
        .iter()
        .map(|anel| {
            let n = anel.len();
            (0..n)
                .map(|i| {
                    let e = (anel[i], anel[(i + 1) % n]);
                    arestas
                        .binary_search_by_key(&e, |x| x.0)
                        .map_or(u32::MAX, |k| arestas[k].1)
                })
                .collect()
        })
        .collect()
}

/// ⭐⭐⭐ **Cada triângulo cosido entra logo DEPOIS do triângulo de onde vem a sua beira** — à
/// profundidade do SEU membro ([`crate::skin_image_fecho::ordena_pelo_osso`]): a metade do membro
/// de trás fica por baixo da tinta do da frente e só se vê no vão. `novos` já vêm deslocados para
/// depois dos pontos da malha; um `slot` sem triângulo vai para o fim.
pub(crate) fn junta(tris: &mut Vec<[u32; 3]>, novos: &[[u32; 3]], slot: &[u32]) {
    let n = tris.len();
    let mut por_slot: Vec<(usize, [u32; 3])> = slot
        .iter()
        .map(|&k| (k as usize).min(n))
        .zip(novos.iter().copied())
        .collect();
    por_slot.sort_by_key(|e| e.0);
    let mut out = Vec::with_capacity(n + por_slot.len());
    let mut j = 0;
    for (i, t) in tris.iter().enumerate() {
        out.push(*t);
        while por_slot.get(j).is_some_and(|e| e.0 == i) {
            out.push(por_slot[j].1);
            j += 1;
        }
    }
    out.extend(por_slot[j..].iter().map(|e| e.1));
    *tris = out;
}
