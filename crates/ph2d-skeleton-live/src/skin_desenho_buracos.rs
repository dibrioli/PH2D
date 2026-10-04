//! ⭐⭐ **OS BURACOS QUE O TRAÇO ENGOLE FECHAM-SE** (A5-b, ordem do dono 2026-10-04: *«fechar os
//! buracos tão pequenos que a linha os cobre»*). Num *Zig Zag* muito dobrado os dentes dos dois
//! membros cruzam-se e a UNIÃO deixa buracos reais; os de raio inscrito menor que meia largura do
//! traço ficam todos cobertos pela linha e liam-se como manchas pretas (MEDIDO na `=5`: raios
//! `0,14`/`0,15` a `100°`, `0,26`/`0,43` a `110°`, em larguras). Só os buracos NOVOS — um buraco que
//! a fonte já tinha (desenhado pelo artista) fica, por pequeno que seja.

use ph2d_vec_scene::VecPath;

#[cfg(test)]
thread_local! {
    /// Os gates desligam a lei para o CONTROLO.
    pub(super) static SEM_FECHO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A polilinha de um contorno fechado (`16` amostras por segmento).
pub(super) fn polilinha(v: &[ph2d_vec_scene::VecVertex]) -> Vec<[f64; 2]> {
    let n = v.len();
    (0..n)
        .flat_map(|k| {
            let c = [
                v[k].anchor,
                v[k].out_handle,
                v[(k + 1) % n].in_handle,
                v[(k + 1) % n].anchor,
            ];
            (0..16).map(move |i| super::super::frente::avalia(&c, f64::from(i) / 16.0))
        })
        .collect()
}

/// A área com sinal de uma polilinha fechada.
pub(super) fn area(p: &[[f64; 2]]) -> f64 {
    0.5 * (0..p.len())
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
}

fn caixa(p: &[[f64; 2]]) -> [f64; 4] {
    p.iter()
        .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |c, q| {
            [
                c[0].min(q[0]),
                c[1].min(q[1]),
                c[2].max(q[0]),
                c[3].max(q[1]),
            ]
        })
}

/// O raio do maior círculo dentro da polilinha fechada `p` (uma grelha `24 × 24` na caixa; o erro é
/// meio passo, `1/48` da caixa).
pub(super) fn raio_inscrito(p: &[[f64; 2]]) -> f64 {
    let [x0, y0, x1, y1] = caixa(p);
    let dentro = |q: [f64; 2]| {
        let mut d = false;
        for i in 0..p.len() {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            if (a[1] > q[1]) != (b[1] > q[1])
                && q[0] < (b[0] - a[0]) * (q[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                d = !d;
            }
        }
        d
    };
    let borda = |q: [f64; 2]| {
        (0..p.len())
            .map(|i| {
                let (a, b) = (p[i], p[(i + 1) % p.len()]);
                let ab = [b[0] - a[0], b[1] - a[1]];
                let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-30);
                let t = (((q[0] - a[0]) * ab[0] + (q[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0);
                (q[0] - a[0] - t * ab[0]).hypot(q[1] - a[1] - t * ab[1])
            })
            .fold(f64::MAX, f64::min)
    };
    let mut r = 0.0_f64;
    for i in 0..24 {
        for j in 0..24 {
            let q = [
                (x1 - x0).mul_add((f64::from(i) + 0.5) / 24.0, x0),
                (y1 - y0).mul_add((f64::from(j) + 0.5) / 24.0, y0),
            ];
            if dentro(q) {
                r = r.max(borda(q));
            }
        }
    }
    r
}

/// ⭐⭐ Tira de `u` (a UNIÃO dos fechados de `d`) cada BURACO novo de raio inscrito menor que meia
/// largura do traço — o traço cobri-lo-ia todo. Um buraco é um contorno de sentido contrário ao
/// maior; é NOVO quando nenhum fechado de `d` tem a mesma área e a mesma caixa (`10⁻³`).
pub(super) fn fecha_os_buracos_que_o_traco_engole(u: &mut VecPath, d: &VecPath) {
    #[cfg(test)]
    if SEM_FECHO.with(std::cell::Cell::get) {
        return;
    }
    let meia = 0.5 * u.stroke.as_ref().map_or(0.0, |s| s.width);
    if meia <= 0.0 {
        return;
    }
    let contornos: Vec<ph2d_vec_scene::Contour> = (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .map(|(v, closed)| ph2d_vec_scene::Contour {
            verts: v.to_vec(),
            closed,
        })
        .collect();
    let pl: Vec<Option<Vec<[f64; 2]>>> = contornos
        .iter()
        .map(|c| (c.closed && c.verts.len() > 1).then(|| polilinha(&c.verts)))
        .collect();
    let areas: Vec<f64> = pl
        .iter()
        .map(|p| p.as_ref().map_or(0.0, |p| area(p)))
        .collect();
    let Some(maior) = areas
        .iter()
        .copied()
        .max_by(|a, b| a.abs().total_cmp(&b.abs()))
    else {
        return;
    };
    let da_fonte: Vec<(f64, [f64; 4])> = (0..d.contour_count())
        .filter_map(|c| d.contour(c))
        .filter(|(v, f)| *f && v.len() > 1)
        .map(|(v, _)| {
            let p = polilinha(v);
            (area(&p), caixa(&p))
        })
        .collect();
    let diag = {
        let c = caixa(
            pl.iter()
                .flatten()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        );
        (c[2] - c[0]).hypot(c[3] - c[1])
    };
    let sai = |i: usize| -> bool {
        let Some(p) = &pl[i] else { return false };
        if areas[i] * maior >= 0.0 {
            return false;
        }
        let (a, c) = (areas[i], caixa(p));
        let da_arte = da_fonte.iter().any(|(fa, fc)| {
            (fa.abs() - a.abs()).abs() <= 1e-3 * a.abs().max(1e-12)
                && (0..4).all(|k| (fc[k] - c[k]).abs() <= 1e-3 * diag)
        });
        !da_arte && raio_inscrito(p) < meia
    };
    if !(0..contornos.len()).any(sai) {
        return;
    }
    let mut fica = contornos
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !sai(*i))
        .map(|(_, c)| c);
    let Some(primeiro) = fica.next() else { return };
    u.verts = primeiro.verts;
    u.closed = primeiro.closed;
    u.subpaths = fica.collect();
}

#[cfg(test)]
#[path = "skin_desenho_buracos_tests.rs"]
mod tests;
