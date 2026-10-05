//! ⭐⭐ **AS PASSAGENS QUE O TRAÇO ENCHE CORTAM-SE PELA CORDA** (A12, ordem do dono 2026-10-05:
//! *«sim»* a fechar as reentrâncias mais estreitas que a linha). Num *Zig Zag* muito dobrado a UNIÃO
//! dos membros deixa, onde dois contornos se cruzam, passagens mais estreitas que o traço — uma
//! FENDA aberta para fora, ou a PONTA de um dente que entra num buraco: o traço dos dois lados
//! enche-as e elas leem-se como manchas escuras (FOTOGRAFADO na `=5` a `100°`, uma de cada). Cada
//! uma corta-se pela corda onde a largura chega à do traço, e o contorno passa recto por ali.
//!
//! ⚠️ **Só as NOVAS:** o vale e o bico de um dente também são estreitos, mas já estão na fonte (os
//! dois lados da corda estão no MESMO contorno da fonte, à mesma distância pelo contorno) — ficam.
//! ⛔ E um contorno que o traço engole inteiro (o buraco pequeno) fica (F59-b, recusado pelo dono).

use ph2d_vec_scene::{VecPath, VecVertex};

#[cfg(test)]
thread_local! {
    /// Os gates desligam a lei para o CONTROLO.
    pub(super) static SEM_FECHO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Amostras por segmento das polilinhas.
const AMOSTRAS: usize = 16;
/// Fendas fechadas por contorno no máximo (cada uma refaz as amostras).
const MAX_FENDAS: usize = 32;

/// A polilinha de um contorno fechado, `AMOSTRAS` por segmento, sem repetir o 1.º ponto.
fn polilinha(v: &[VecVertex]) -> Vec<[f64; 2]> {
    let n = v.len();
    (0..n)
        .flat_map(|k| {
            let c = [
                v[k].anchor,
                v[k].out_handle,
                v[(k + 1) % n].in_handle,
                v[(k + 1) % n].anchor,
            ];
            (0..AMOSTRAS).map(move |i| {
                #[expect(clippy::cast_precision_loss, reason = "amostra")]
                let t = i as f64 / AMOSTRAS as f64;
                super::super::frente::avalia(&c, t)
            })
        })
        .collect()
}

/// `p` está dentro dos anéis `aneis` (voltas diferentes de zero)?
fn dentro(aneis: &[Vec<[f64; 2]>], p: [f64; 2]) -> bool {
    let mut voltas = 0_i32;
    for a in aneis {
        for i in 0..a.len() {
            let (x, y) = (a[i], a[(i + 1) % a.len()]);
            if (x[1] > p[1]) != (y[1] > p[1])
                && (y[0] - x[0]) * (p[1] - x[1]) / (y[1] - x[1]) + x[0] > p[0]
            {
                voltas += if y[1] > x[1] { 1 } else { -1 };
            }
        }
    }
    voltas != 0
}

/// A distância de `p` ao troço `ab` e a fracção do troço onde ela cai.
fn ao_troco(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> (f64, f64) {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-30);
    let f = (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0);
    ((p[0] - a[0] - f * ab[0]).hypot(p[1] - a[1] - f * ab[1]), f)
}

/// Um contorno da FONTE como polilinha com o comprimento acumulado.
struct Anel {
    pts: Vec<[f64; 2]>,
    acc: Vec<f64>,
}

impl Anel {
    fn novo(pts: Vec<[f64; 2]>) -> Self {
        let mut acc = vec![0.0];
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            acc.push(acc[i] + (b[0] - a[0]).hypot(b[1] - a[1]));
        }
        Self { pts, acc }
    }

    /// As posições (comprimento ao longo do anel) onde `p` mora, a menos de `tol`.
    fn onde(&self, p: [f64; 2], tol: f64) -> Vec<f64> {
        let n = self.pts.len();
        (0..n)
            .filter_map(|i| {
                let (d, f) = ao_troco(p, self.pts[i], self.pts[(i + 1) % n]);
                (d <= tol).then(|| (self.acc[i + 1] - self.acc[i]).mul_add(f, self.acc[i]))
            })
            .collect()
    }

    /// A distância pelo anel entre duas posições, pelo caminho mais curto.
    fn entre(&self, a: f64, b: f64) -> f64 {
        let t = self.acc[self.pts.len()];
        let d = (a - b).abs();
        d.min(t - d)
    }
}

/// O raio do maior círculo dentro do polígono `p` (grelha `24 × 24` na caixa).
fn raio_inscrito(p: &[[f64; 2]]) -> f64 {
    let (lo, hi) = p
        .iter()
        .fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), q| {
            (
                [lo[0].min(q[0]), lo[1].min(q[1])],
                [hi[0].max(q[0]), hi[1].max(q[1])],
            )
        });
    let anel = [p.to_vec()];
    let mut r = 0.0_f64;
    for i in 0..24 {
        for j in 0..24 {
            let q = [
                (hi[0] - lo[0]).mul_add((f64::from(i) + 0.5) / 24.0, lo[0]),
                (hi[1] - lo[1]).mul_add((f64::from(j) + 0.5) / 24.0, lo[1]),
            ];
            if dentro(&anel, q) {
                let borda = (0..p.len())
                    .map(|k| ao_troco(q, p[k], p[(k + 1) % p.len()]).0)
                    .fold(f64::MAX, f64::min);
                r = r.max(borda);
            }
        }
    }
    r
}

/// ⭐ A melhor passagem NOVA do contorno `pl` da união: `(s, e)` em amostras, ela vai de `s` para
/// a frente até `e`. A boca é a corda mais larga (`< w`) de uma passagem que o traço cobre toda.
fn fenda(pl: &[[f64; 2]], fonte: &[Anel], w: f64) -> Option<(usize, usize)> {
    let n = pl.len();
    let mut acc = vec![0.0];
    for i in 0..n {
        let (a, b) = (pl[i], pl[(i + 1) % n]);
        acc.push(acc[i] + (b[0] - a[0]).hypot(b[1] - a[1]));
    }
    let total = acc[n];
    // Os pares de amostras a menos de `w` que estão longe pelo contorno, com a corda por FORA.
    let lado = w;
    let celula = |p: [f64; 2]| {
        #[expect(clippy::cast_possible_truncation, reason = "célula da grelha")]
        [(p[0] / lado).floor() as i64, (p[1] / lado).floor() as i64]
    };
    let mut grelha: std::collections::BTreeMap<[i64; 2], Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, p) in pl.iter().enumerate() {
        grelha.entry(celula(*p)).or_default().push(i);
    }
    let mut pares: Vec<(f64, usize, usize)> = Vec::new();
    for (i, p) in pl.iter().enumerate() {
        let c = celula(*p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for &j in grelha.get(&[c[0] + dx, c[1] + dy]).into_iter().flatten() {
                    if j <= i {
                        continue;
                    }
                    let q = pl[j];
                    if (q[0] - p[0]).hypot(q[1] - p[1]) >= w {
                        continue;
                    }
                    // A fenda é o arco mais curto entre os dois.
                    let ida = acc[j] - acc[i];
                    let (s, e, arco) = if ida <= total - ida {
                        (i, j, ida)
                    } else {
                        (j, i, total - ida)
                    };
                    if arco <= 1.5 * w {
                        continue;
                    }
                    pares.push((arco, s, e));
                }
            }
        }
    }
    pares.sort_by(|a, b| b.0.total_cmp(&a.0));
    let dentro_do_arco = |s: usize, e: usize, k: usize| {
        if s <= e {
            (s..=e).contains(&k)
        } else {
            k >= s || k <= e
        }
    };
    let mut vistos: Vec<(usize, usize)> = Vec::new();
    for (arco, s, e) in pares {
        if vistos
            .iter()
            .any(|&(a, b)| dentro_do_arco(a, b, s) && dentro_do_arco(a, b, e))
        {
            continue;
        }
        vistos.push((s, e));
        // ⚠️ NOVA: nenhum contorno da fonte liga os dois lados da boca pelo MESMO caminho (o mesmo
        // comprimento). ⛔ «Um caminho não mais longo» não serve: a borda de um buraco liga a base
        // de uma ponta que entra nele por um caminho CURTO, e a ponta lia-se como da fonte.
        let tol = 1e-3 * w;
        let velha = fonte.iter().any(|a| {
            let (xs, ys) = (a.onde(pl[s], tol), a.onde(pl[e], tol));
            xs.iter().any(|&x| {
                ys.iter()
                    .any(|&y| (a.entre(x, y) - arco).abs() <= 1e-2 * arco + tol)
            })
        });
        if velha {
            continue;
        }
        let mut regiao = Vec::new();
        let mut k = s;
        loop {
            regiao.push(pl[k]);
            if k == e {
                break;
            }
            k = (k + 1) % n;
        }
        if raio_inscrito(&regiao) < 0.5 * w {
            return Some((s, e));
        }
    }
    None
}

/// ⭐⭐ Tira de `u` (a UNIÃO dos fechados de `fonte`) cada fenda NOVA que o traço enche — o contorno
/// passa recto pela boca dela.
pub(super) fn fecha_as_fendas_que_o_traco_enche(u: &mut VecPath, fonte: &VecPath) {
    #[cfg(test)]
    if SEM_FECHO.with(std::cell::Cell::get) {
        return;
    }
    let w = u.stroke.as_ref().map_or(0.0, |s| s.width);
    if w <= 0.0 {
        return;
    }
    let fonte: Vec<Anel> = (0..fonte.contour_count())
        .filter_map(|c| fonte.contour(c))
        .filter(|(v, f)| *f && v.len() > 1)
        .map(|(v, _)| Anel::novo(polilinha(v)))
        .collect();
    let mut contornos: Vec<ph2d_vec_scene::Contour> = (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .map(|(v, closed)| ph2d_vec_scene::Contour {
            verts: v.to_vec(),
            closed,
        })
        .collect();
    let mut mexeu = false;
    for ci in 0..contornos.len() {
        for _ in 0..MAX_FENDAS {
            let c = &contornos[ci];
            if !c.closed || c.verts.len() < 2 {
                break;
            }
            let pl = polilinha(&c.verts);
            // ⛔ Um contorno que o traço engole INTEIRO (o buraco pequeno, a ilhota) fica como
            // está: fechá-lo foi recusado pelo dono (F59-b).
            if raio_inscrito(&pl) < 0.5 * w {
                break;
            }
            let Some((s, e)) = fenda(&pl, &fonte, w) else {
                break;
            };
            contornos[ci].verts = sem_a_fenda(&contornos[ci].verts, s, e);
            mexeu = true;
        }
    }
    if !mexeu {
        return;
    }
    let mut it = contornos.into_iter();
    let Some(primeiro) = it.next() else { return };
    u.verts = primeiro.verts;
    u.closed = primeiro.closed;
    u.subpaths = it.collect();
}

/// O contorno fechado `v` sem a fenda que vai da amostra `s` à `e` (para a frente): fica o resto,
/// de `e` à volta até `s`, e a recta de `s` a `e` fecha-o.
fn sem_a_fenda(v: &[VecVertex], s: usize, e: usize) -> Vec<VecVertex> {
    let m = v.len();
    #[expect(clippy::cast_precision_loss, reason = "parâmetro do contorno")]
    let (us, ue) = (s as f64 / AMOSTRAS as f64, e as f64 / AMOSTRAS as f64);
    #[expect(clippy::cast_precision_loss, reason = "parâmetro do contorno")]
    let fim = if us > ue { us } else { us + m as f64 };
    // O contorno aberto `v v v[0]` — duas voltas para o resto poder passar pela emenda.
    let mut dupla: Vec<VecVertex> = v.iter().chain(v).copied().collect();
    dupla.push(v[0]);
    let mut resto: Vec<VecVertex> = super::super::frente::recorta(&dupla, ue, fim)
        .into_iter()
        .map(|(x, _)| x)
        .collect();
    if let Some(p) = resto.first_mut() {
        p.in_handle = p.anchor;
    }
    if let Some(u) = resto.last_mut() {
        u.out_handle = u.anchor;
    }
    resto
}

#[cfg(test)]
#[path = "skin_desenho_fendas_tests.rs"]
mod tests;
