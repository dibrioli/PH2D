//! ⭐⭐⭐ **SONDA — a correcção ARAP onde a lei DOBRA** (ordem do dono, 2026-09-29: *«a dobra forte
//! do cotovelo: tente o estado da arte diretamente»*).
//!
//! Protótipo de MEDIÇÃO, antes de qualquer linha de produto: sobre a malha do domínio da barra da
//! cena (o `CampoDoDominio` guardado no bind), a lei de hoje dá um alvo por vértice; onde o mapa
//! dela se esmaga (`det J < τ`) os vértices soltam-se e a energia *As-Rigid-As-Possible*
//! (Sorkine–Alexa 2007, a mesma família do Plastic do OpenToonz — BSD-3) decide onde eles vão.
//! Onde a lei está sã os vértices ficam PRESOS a ela ⇒ nada muda fora do cotovelo.

use super::ouro_reguas_tests::*;

/// A malha do campo, no espaço LOCAL da forma, com o alvo da lei por vértice.
pub(super) struct Palco {
    pub(super) rest: Vec<[f64; 2]>,
    pub(super) tris: Vec<[u32; 3]>,
    pub(super) lei: Vec<[f64; 2]>,
}

pub(super) fn palco(p: &BPalco) -> Palco {
    let pele = p.pele();
    let n = p.campo.malha.rest.len();
    let mut rest = Vec::with_capacity(n);
    let mut lei = Vec::with_capacity(n);
    for i in 0..n {
        let x = p.campo.local_do_vertice(i).expect("régua");
        let linha = p.campo.linha_do_vertice(i).expect("linha");
        let mut w = pele.scratch();
        pele.weights_corrected(x, Some(linha), &mut w, &p.correcoes);
        rest.push(x);
        lei.push(pele.blend(x, &w));
    }
    Palco {
        rest,
        tris: p.campo.malha.tris.clone(),
        lei,
    }
}

pub(super) fn cruz(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0].mul_add(b[1], -(a[1] * b[0]))
}
pub(super) fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

/// `det J` de cada triângulo: área deformada sobre área de repouso, COM sinal.
pub(super) fn dets(rest: &[[f64; 2]], def: &[[f64; 2]], tris: &[[u32; 3]]) -> Vec<f64> {
    tris.iter()
        .map(|t| {
            let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
            let r = cruz(sub(rest[b], rest[a]), sub(rest[c], rest[a]));
            let d = cruz(sub(def[b], def[a]), sub(def[c], def[a]));
            d / r
        })
        .collect()
}

/// As cotangentes `(c_bc, c_ca, c_ab)` — metade da cotangente do canto oposto a cada aresta.
pub(super) fn cots(rest: &[[f64; 2]], t: [u32; 3]) -> [f64; 3] {
    let p = [
        rest[t[0] as usize],
        rest[t[1] as usize],
        rest[t[2] as usize],
    ];
    let cot = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        let (e1, e2) = (sub(b, a), sub(c, a));
        e1[0].mul_add(e2[0], e1[1] * e2[1]) / cruz(e1, e2).abs()
    };
    [
        0.5 * cot(p[0], p[1], p[2]),
        0.5 * cot(p[1], p[2], p[0]),
        0.5 * cot(p[2], p[0], p[1]),
    ]
}

/// ⭐ **A correcção**: devolve as posições corrigidas e quantos vértices ficaram soltos.
///
/// `tau`: abaixo deste `det J` (mínimo dos triângulos à volta) o vértice solta-se.
/// `k0`: a força com que um vértice solto ainda é puxado para o alvo da lei, e ela cresce até
/// infinito quando o `det` sobe a `tau` — é isso que faz a correcção nascer do ZERO, sem salto.
pub(super) fn corrige(pc: &Palco, tau: f64, k0: f64, iters: usize) -> (Vec<[f64; 2]>, usize) {
    let n = pc.rest.len();
    let d = dets(&pc.rest, &pc.lei, &pc.tris);
    let mut dv = vec![f64::INFINITY; n];
    for (t, &dt) in pc.tris.iter().zip(&d) {
        for &v in t {
            dv[v as usize] = dv[v as usize].min(dt);
        }
    }
    let livre: Vec<bool> = dv.iter().map(|&x| x < tau).collect();
    let soltos = livre.iter().filter(|&&l| l).count();
    let mut x = pc.lei.clone();
    if soltos == 0 {
        return (x, 0);
    }
    let k: Vec<f64> = dv
        .iter()
        .map(|&x| k0 * x.max(0.0) / (tau - x).max(1e-12))
        .collect();
    let c: Vec<[f64; 3]> = pc.tris.iter().map(|&t| cots(&pc.rest, t)).collect();
    // O laplaciano em listas por linha (só as linhas livres importam).
    let mut diag = vec![0.0_f64; n];
    let mut viz: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for (t, cc) in pc.tris.iter().zip(&c) {
        let e = [(1, 2, cc[0]), (2, 0, cc[1]), (0, 1, cc[2])];
        for (a, b, w) in e {
            let (i, j) = (t[a] as usize, t[b] as usize);
            diag[i] += w;
            diag[j] += w;
            viz[i].push((j, w));
            viz[j].push((i, w));
        }
    }
    for i in 0..n {
        if livre[i] {
            diag[i] += k[i];
        }
    }
    let mut rot = vec![[1.0_f64, 0.0]; pc.tris.len()];
    let mut b = vec![[0.0_f64; 2]; n];
    for _ in 0..iters {
        // ── Local: a melhor rotação de cada triângulo ────────────────────────────────────────
        for (ti, (t, cc)) in pc.tris.iter().zip(&c).enumerate() {
            let mut s = [0.0_f64; 4];
            let e = [(1, 2, cc[0]), (2, 0, cc[1]), (0, 1, cc[2])];
            for (a, bb, w) in e {
                let (i, j) = (t[a] as usize, t[bb] as usize);
                let pr = sub(pc.rest[i], pc.rest[j]);
                let px = sub(x[i], x[j]);
                s[0] += w * px[0] * pr[0];
                s[1] += w * px[0] * pr[1];
                s[2] += w * px[1] * pr[0];
                s[3] += w * px[1] * pr[1];
            }
            let ang = (s[2] - s[1]).atan2(s[0] + s[3]);
            rot[ti] = [ang.cos(), ang.sin()];
        }
        // ── Global: o lado direito ───────────────────────────────────────────────────────────
        b.fill([0.0, 0.0]);
        for ((t, cc), r) in pc.tris.iter().zip(&c).zip(&rot) {
            let e = [(1, 2, cc[0]), (2, 0, cc[1]), (0, 1, cc[2])];
            for (a, bb, w) in e {
                let (i, j) = (t[a] as usize, t[bb] as usize);
                let pr = sub(pc.rest[i], pc.rest[j]);
                let rp = [r[0] * pr[0] - r[1] * pr[1], r[1] * pr[0] + r[0] * pr[1]];
                b[i][0] += w * rp[0];
                b[i][1] += w * rp[1];
                b[j][0] -= w * rp[0];
                b[j][1] -= w * rp[1];
            }
        }
        for i in 0..n {
            if livre[i] {
                b[i][0] += k[i] * pc.lei[i][0];
                b[i][1] += k[i] * pc.lei[i][1];
            }
        }
        // Gauss–Seidel sobre os livres (os presos são Dirichlet na lei).
        for _ in 0..40 {
            for i in 0..n {
                if !livre[i] {
                    continue;
                }
                let mut s = b[i];
                for &(j, w) in &viz[i] {
                    s[0] += w * x[j][0];
                    s[1] += w * x[j][1];
                }
                x[i] = [s[0] / diag[i], s[1] / diag[i]];
            }
        }
    }
    (x, soltos)
}

/// `χ(d, ε) = (d + √(ε² + d²)) / 2` — a regularização de Garanzha et al. 2021 (*Foldover-free maps
/// in 50 lines of code*): positiva em todo `d`, e `→ max(d, 0)` quando `ε → 0`.
pub(super) fn chi(d: f64, eps: f64) -> f64 {
    0.5 * (d + eps.hypot(d))
}
pub(super) fn dchi(d: f64, eps: f64) -> f64 {
    0.5 * (1.0 + d / eps.hypot(d))
}

/// A barreira sobre `c = χ(d)`: zero acima de `c0`, infinita em `0`. `(c0/c − 1)²`.
pub(super) fn barreira(c: f64, c0: f64) -> (f64, f64) {
    if c >= c0 {
        return (0.0, 0.0);
    }
    let u = c0 / c - 1.0;
    (u * u, -2.0 * u * c0 / (c * c))
}

/// ⭐⭐⭐ **A MENOR correcção à lei que não vira triângulo nenhum.**
///
/// `E(x) = Σ_v m_v |x_v − Y_v|² + κ Σ_t A_t · φ(χ(det J_t, ε))`, minimizada por L-BFGS, com `ε` a
/// descer em rondas até toda a malha estar orientada. Onde a lei está sã (`det ≥ c0`) a barreira é
/// zero e o mínimo é `x = Y` **exactamente**.
pub(super) fn desdobra(pc: &Palco, c0: f64, kappa: f64) -> (Vec<[f64; 2]>, usize) {
    desdobra_com(pc, c0, kappa, 0.0)
}

/// A soma assinada dos ângulos à volta de cada vértice e o gradiente dela, só nos vértices da borda.
pub(super) fn borda_de(pc: &Palco) -> Vec<Vec<(usize, usize)>> {
    use std::collections::BTreeMap;
    let mut arestas: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for t in &pc.tris {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            *arestas.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let mut borda = vec![false; pc.rest.len()];
    for (&(a, b), &c) in &arestas {
        if c == 1 {
            borda[a as usize] = true;
            borda[b as usize] = true;
        }
    }
    let mut cantos: Vec<Vec<(usize, usize)>> = vec![Vec::new(); pc.rest.len()];
    for (ti, t) in pc.tris.iter().enumerate() {
        for k in 0..3 {
            if borda[t[k] as usize] {
                cantos[t[k] as usize].push((ti, k));
            }
        }
    }
    cantos
}

/// ⭐⭐ **A mesma correcção, com a SEGUNDA barreira:** num vértice da BORDA a soma dos ângulos
/// dos triângulos à volta dele não pode passar de `360° − g0` (senão a borda dá a volta sobre si
/// mesma e o contorno cruza-se no canto). `g0 = 0` desliga-a.
pub(super) fn desdobra_com(pc: &Palco, c0: f64, kappa: f64, g0: f64) -> (Vec<[f64; 2]>, usize) {
    let n = pc.rest.len();
    let cantos = if g0 > 0.0 {
        borda_de(pc)
    } else {
        vec![Vec::new(); n]
    };
    let soma_ang = |x: &[f64], i: usize, g: Option<(&mut [f64], f64)>| -> f64 {
        let mut soma = 0.0;
        let mut acc: Vec<(usize, [f64; 2])> = Vec::new();
        for &(ti, k) in &cantos[i] {
            let t = pc.tris[ti];
            let (o, u, v) = (
                t[k] as usize,
                t[(k + 1) % 3] as usize,
                t[(k + 2) % 3] as usize,
            );
            let e1 = [x[2 * u] - x[2 * o], x[2 * u + 1] - x[2 * o + 1]];
            let e2 = [x[2 * v] - x[2 * o], x[2 * v + 1] - x[2 * o + 1]];
            soma += cruz(e1, e2).atan2(e1[0].mul_add(e2[0], e1[1] * e2[1]));
            if g.is_some() {
                let (l1, l2) = (
                    e1[0].mul_add(e1[0], e1[1] * e1[1]),
                    e2[0].mul_add(e2[0], e2[1] * e2[1]),
                );
                let gv = [-e2[1] / l2, e2[0] / l2];
                let gu = [e1[1] / l1, -e1[0] / l1];
                acc.push((v, gv));
                acc.push((u, gu));
                acc.push((o, [-(gv[0] + gu[0]), -(gv[1] + gu[1])]));
            }
        }
        if let Some((g, esc)) = g {
            for (j, d) in acc {
                g[2 * j] += esc * d[0];
                g[2 * j + 1] += esc * d[1];
            }
        }
        soma
    };
    let ra: Vec<f64> = pc
        .tris
        .iter()
        .map(|t| {
            let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
            cruz(sub(pc.rest[b], pc.rest[a]), sub(pc.rest[c], pc.rest[a]))
        })
        .collect();
    let mut m = vec![0.0_f64; n];
    for (t, &r) in pc.tris.iter().zip(&ra) {
        for &v in t {
            m[v as usize] += r.abs() / 6.0;
        }
    }
    let energia = |x: &[f64], eps: f64, g: &mut [f64]| -> f64 {
        g.iter_mut().for_each(|v| *v = 0.0);
        let mut e = 0.0;
        for i in 0..n {
            let (dx, dy) = (x[2 * i] - pc.lei[i][0], x[2 * i + 1] - pc.lei[i][1]);
            e += m[i] * dx.mul_add(dx, dy * dy);
            g[2 * i] += 2.0 * m[i] * dx;
            g[2 * i + 1] += 2.0 * m[i] * dy;
        }
        for (t, &r) in pc.tris.iter().zip(&ra) {
            let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
            let u = [x[2 * b] - x[2 * a], x[2 * b + 1] - x[2 * a + 1]];
            let v = [x[2 * c] - x[2 * a], x[2 * c + 1] - x[2 * a + 1]];
            let d = cruz(u, v) / r;
            let cc = chi(d, eps);
            let (f, df) = barreira(cc, c0);
            if f == 0.0 {
                continue;
            }
            let w = kappa * r.abs() * 0.5;
            e += w * f;
            let s = w * df * dchi(d, eps) / r;
            // ∂d/∂xb = (v1, −v0), ∂d/∂xc = (−u1, u0), ∂d/∂xa = −(soma).
            let gb = [v[1] * s, -v[0] * s];
            let gc = [-u[1] * s, u[0] * s];
            g[2 * b] += gb[0];
            g[2 * b + 1] += gb[1];
            g[2 * c] += gc[0];
            g[2 * c + 1] += gc[1];
            g[2 * a] -= gb[0] + gc[0];
            g[2 * a + 1] -= gb[1] + gc[1];
        }
        for i in 0..n {
            if cantos[i].is_empty() {
                continue;
            }
            let folga = std::f64::consts::TAU - soma_ang(x, i, None);
            let cc = chi(folga, eps);
            let (f, df) = barreira(cc, g0);
            if f == 0.0 {
                continue;
            }
            let w = kappa * m[i];
            e += w * f;
            // d(folga)/dx = −d(soma)/dx
            let esc = -w * df * dchi(folga, eps);
            soma_ang(x, i, Some((&mut *g, esc)));
        }
        e
    };
    let pior_folga = |x: &[f64]| {
        (0..n)
            .filter(|&i| !cantos[i].is_empty())
            .map(|i| std::f64::consts::TAU - soma_ang(x, i, None))
            .fold(f64::INFINITY, f64::min)
    };
    let mut x: Vec<f64> = pc.lei.iter().flat_map(|p| [p[0], p[1]]).collect();
    let min_det = |x: &[f64]| {
        pc.tris
            .iter()
            .zip(&ra)
            .map(|(t, &r)| {
                let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
                let u = [x[2 * b] - x[2 * a], x[2 * b + 1] - x[2 * a + 1]];
                let v = [x[2 * c] - x[2 * a], x[2 * c + 1] - x[2 * a + 1]];
                cruz(u, v) / r
            })
            .fold(f64::INFINITY, f64::min)
    };
    let d0 = min_det(&x);
    if d0 >= c0 && (g0 == 0.0 || pior_folga(&x) >= g0) {
        return (pc.lei.clone(), 0);
    }
    let mut eps = (2.0 * (-d0).max(0.0)).max(1e-3);
    let mut iters = 0usize;
    let mut g = vec![0.0_f64; 2 * n];
    let mut g2 = vec![0.0_f64; 2 * n];
    for _ronda in 0..12 {
        // L-BFGS
        let mem = 8;
        let mut ss: Vec<Vec<f64>> = Vec::new();
        let mut ys: Vec<Vec<f64>> = Vec::new();
        let mut e = energia(&x, eps, &mut g);
        for _ in 0..200 {
            iters += 1;
            let gn = g.iter().map(|v| v * v).sum::<f64>().sqrt();
            if gn < 1e-10 {
                break;
            }
            let mut q = g.clone();
            let k = ss.len();
            let mut al = vec![0.0; k];
            for i in (0..k).rev() {
                let rho = 1.0 / ys[i].iter().zip(&ss[i]).map(|(a, b)| a * b).sum::<f64>();
                al[i] = rho * ss[i].iter().zip(&q).map(|(a, b)| a * b).sum::<f64>();
                for (qq, yy) in q.iter_mut().zip(&ys[i]) {
                    *qq -= al[i] * yy;
                }
            }
            if k > 0 {
                let sy: f64 = ys[k - 1].iter().zip(&ss[k - 1]).map(|(a, b)| a * b).sum();
                let yy: f64 = ys[k - 1].iter().map(|a| a * a).sum();
                q.iter_mut().for_each(|v| *v *= sy / yy);
            } else {
                let s = 1e-2 / gn;
                q.iter_mut().for_each(|v| *v *= s);
            }
            for i in 0..k {
                let rho = 1.0 / ys[i].iter().zip(&ss[i]).map(|(a, b)| a * b).sum::<f64>();
                let be = rho * ys[i].iter().zip(&q).map(|(a, b)| a * b).sum::<f64>();
                for (qq, s) in q.iter_mut().zip(&ss[i]) {
                    *qq += s * (al[i] - be);
                }
            }
            // direcção = −q; Armijo com recuo
            let gd: f64 = -g.iter().zip(&q).map(|(a, b)| a * b).sum::<f64>();
            if gd >= 0.0 {
                ss.clear();
                ys.clear();
                continue;
            }
            let mut passo = 1.0;
            let mut xn = x.clone();
            let mut en = e;
            let mut ok = false;
            for _ in 0..40 {
                for (i, v) in xn.iter_mut().enumerate() {
                    *v = x[i] - passo * q[i];
                }
                en = energia(&xn, eps, &mut g2);
                if en.is_finite() && en <= e + 1e-4 * passo * gd {
                    ok = true;
                    break;
                }
                passo *= 0.5;
            }
            if !ok {
                break;
            }
            let s: Vec<f64> = xn.iter().zip(&x).map(|(a, b)| a - b).collect();
            let y: Vec<f64> = g2.iter().zip(&g).map(|(a, b)| a - b).collect();
            if s.iter().zip(&y).map(|(a, b)| a * b).sum::<f64>() > 1e-20 {
                ss.push(s);
                ys.push(y);
                if ss.len() > mem {
                    ss.remove(0);
                    ys.remove(0);
                }
            }
            let de = e - en;
            x = xn;
            e = en;
            std::mem::swap(&mut g, &mut g2);
            if de.abs() < 1e-14 * e.abs().max(1e-30) {
                break;
            }
        }
        let dm = min_det(&x);
        let fo = if g0 > 0.0 { pior_folga(&x) } else { 1.0 };
        if dm > 0.0 && fo > 0.0 && eps < 1e-6 {
            break;
        }
        eps = if dm > 0.0 && fo > 0.0 {
            eps * 0.1
        } else {
            eps * 0.5
        };
    }
    (x.chunks(2).map(|c| [c[0], c[1]]).collect(), iters)
}

/// Triângulo e baricêntricas de um ponto de repouso (força bruta: é uma sonda).
pub(super) fn localiza(pc: &Palco, p: [f64; 2]) -> Option<(usize, [f64; 3])> {
    let mut melhor: Option<(f64, usize, [f64; 3])> = None;
    for (ti, t) in pc.tris.iter().enumerate() {
        let (a, b, c) = (
            pc.rest[t[0] as usize],
            pc.rest[t[1] as usize],
            pc.rest[t[2] as usize],
        );
        let area = cruz(sub(b, a), sub(c, a));
        let l0 = cruz(sub(b, p), sub(c, p)) / area;
        let l1 = cruz(sub(c, p), sub(a, p)) / area;
        let l2 = 1.0 - l0 - l1;
        let fora = (-l0).max(-l1).max(-l2);
        if melhor.is_none_or(|m| fora < m.0) {
            melhor = Some((fora, ti, [l0, l1, l2]));
        }
    }
    melhor.map(|(_, t, l)| (t, l))
}

/// O contorno deformado: a lei no ponto MAIS a correcção interpolada.
pub(super) fn contorno(p: &BPalco, pc: &Palco, x: &[[f64; 2]], rest: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let pele = p.pele();
    // `s_v = 0` num vértice de um triângulo que a correcção tocou (algum canto mexeu).
    let mut s = vec![1.0_f64; pc.rest.len()];
    for t in &pc.tris {
        if t.iter().any(|&v| x[v as usize] != pc.lei[v as usize]) {
            for &v in t {
                s[v as usize] = 0.0;
            }
        }
    }
    rest.iter()
        .map(|&q| {
            let base = b_ouro_pt(&pele, &p.campo, &p.correcoes, q).0;
            let Some((t, l)) = localiza(pc, q) else {
                return base;
            };
            let tr = pc.tris[t];
            // ⭐ `p' = PL(x) + s·(lei(p) − PL(Y))`: o detalhe da lei ENTRE os vértices só entra onde
            // a malha não foi corrigida — onde foi, manda a malha, que não vira por construção.
            let (mut plx, mut ply, mut sv) = ([0.0_f64; 2], [0.0_f64; 2], 0.0_f64);
            for (k, &v) in tr.iter().enumerate() {
                let v = v as usize;
                plx[0] += l[k] * x[v][0];
                plx[1] += l[k] * x[v][1];
                ply[0] += l[k] * pc.lei[v][0];
                ply[1] += l[k] * pc.lei[v][1];
                sv += l[k] * s[v];
            }
            [
                sv.mul_add(base[0] - ply[0], plx[0]),
                sv.mul_add(base[1] - ply[1], plx[1]),
            ]
        })
        .collect()
}

#[test]
#[ignore = "sonda: imprime a tabela da correcção ARAP contra a lei de hoje"]
fn diag_arap_onde_a_lei_dobra() {
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    let rest = b_amostra_com(&p.fonte, 64);
    for (tau, k0) in [(0.5, 1.0), (0.7, 1.0), (0.9, 1.0), (0.7, 0.2)] {
        println!("── τ = {tau}  k0 = {k0}");
        for (nome, s) in [("C", false), ("S", true)] {
            for g in [60.0_f32, 90.0, 110.0, 130.0, 150.0] {
                if s {
                    p.dobra_em_s(g);
                } else {
                    p.dobra(g);
                }
                let pc = palco(&p);
                let d0 = dets(&pc.rest, &pc.lei, &pc.tris);
                let t = std::time::Instant::now();
                let (x, soltos) = corrige(&pc, tau, k0, 30);
                let ms = t.elapsed().as_secs_f64() * 1e3;
                let d1 = dets(&pc.rest, &x, &pc.tris);
                let inv0 = d0.iter().filter(|&&v| v <= 0.0).count();
                let inv1 = d1.iter().filter(|&&v| v <= 0.0).count();
                let min0 = d0.iter().copied().fold(f64::INFINITY, f64::min);
                let min1 = d1.iter().copied().fold(f64::INFINITY, f64::min);
                let c0 = contorno(&p, &pc, &pc.lei, &rest);
                let c1 = contorno(&p, &pc, &x, &rest);
                let mov = c0
                    .iter()
                    .zip(&c1)
                    .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                    .fold(0.0_f64, f64::max);
                println!(
                    "  {nome} {g:>4}°: invertidos {inv0:>3} → {inv1:>3} · min det {min0:>7.3} → \
                     {min1:>7.3} · auto-cruzes {} → {} · soltos {soltos:>3}/{} · contorno move \
                     {mov:.4} · {ms:.1} ms",
                    b_auto(&c0, 1e-7),
                    b_auto(&c1, 1e-7),
                    pc.rest.len(),
                );
            }
        }
    }
}

#[test]
#[ignore = "sonda: a menor correcção sem inversão, contra a lei de hoje"]
fn diag_barreira_onde_a_lei_dobra() {
    let mut p = b_palco(false);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    let rest = b_amostra_com(&p.fonte, 64);
    for (c0, kappa, g0) in [
        (0.1, 0.01, 0.0),
        (0.1, 0.01, 0.2),
        (0.1, 0.01, 0.5),
        (0.3, 0.01, 0.5),
    ] {
        println!("── c0 = {c0}  κ = {kappa}  g0 = {g0}");
        for (nome, s) in [("C", false), ("S", true)] {
            for g in [60.0_f32, 90.0, 100.0, 110.0, 130.0, 150.0] {
                if s {
                    p.dobra_em_s(g);
                } else {
                    p.dobra(g);
                }
                let pc = palco(&p);
                let d0 = dets(&pc.rest, &pc.lei, &pc.tris);
                let t = std::time::Instant::now();
                let (x, it) = desdobra_com(&pc, c0, kappa, g0);
                let ms = t.elapsed().as_secs_f64() * 1e3;
                let d1 = dets(&pc.rest, &x, &pc.tris);
                let inv0 = d0.iter().filter(|&&v| v <= 0.0).count();
                let inv1 = d1.iter().filter(|&&v| v <= 0.0).count();
                let min1 = d1.iter().copied().fold(f64::INFINITY, f64::min);
                let c_0 = contorno(&p, &pc, &pc.lei, &rest);
                let c_1 = contorno(&p, &pc, &x, &rest);
                let mov = c_0
                    .iter()
                    .zip(&c_1)
                    .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                    .fold(0.0_f64, f64::max);
                let mexidos = pc.lei.iter().zip(&x).filter(|(a, b)| a != b).count();
                println!(
                    "  {nome} {g:>4}°: invertidos {inv0:>3} → {inv1:>3} · min det → {min1:>9.2e} · \
                     auto-cruzes {} → {} · mexidos {mexidos:>3}/{} · contorno move {mov:.4} · \
                     {it} it · {ms:.1} ms",
                    b_auto(&c_0, 1e-7),
                    b_auto(&c_1, 1e-7),
                    pc.rest.len(),
                );
            }
        }
    }
}
