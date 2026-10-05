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
    /// A última entrada da lei — `(união, fonte)` — para as sondas.
    pub(super) static ULTIMA: std::cell::RefCell<Option<(VecPath, VecPath)>> = const { std::cell::RefCell::new(None) };
    /// O que a [`fenda`] examinou: `(s, e, arco, corda, velha, raio)` — para as sondas.
    /// As sondas ligam o registo das [`EXAMINADAS`] (o raio de cada uma custa: fora do relógio).
    pub(super) static REGISTA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(super) static EXAMINADAS: std::cell::RefCell<Vec<(usize, usize, f64, f64, bool, f64)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Amostras por segmento das polilinhas.
const AMOSTRAS: usize = 16;

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
}

/// ⭐ Cabe em `p` um círculo de raio `r`? — algum ponto da grelha `24 × 24` da caixa está dentro e a
/// `r` ou mais da borda. Do CENTRO para fora, e pára no 1.º (o contorno de fora de uma barra tem
/// `1 700` amostras: o raio inteiro dele custava milissegundos por quadro, MEDIDO).
fn cabe(p: &[[f64; 2]], r: f64) -> bool {
    let (lo, hi) = p
        .iter()
        .fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), q| {
            (
                [lo[0].min(q[0]), lo[1].min(q[1])],
                [hi[0].max(q[0]), hi[1].max(q[1])],
            )
        });
    if hi[0] - lo[0] < 2.0 * r || hi[1] - lo[1] < 2.0 * r {
        return false;
    }
    let mut ordem: Vec<(i32, i32)> = (0..24).flat_map(|i| (0..24).map(move |j| (i, j))).collect();
    ordem.sort_by_key(|&(i, j)| (2 * i - 23).pow(2) + (2 * j - 23).pow(2));
    let anel = [p.to_vec()];
    ordem.into_iter().any(|(i, j)| {
        let q = [
            (hi[0] - lo[0]).mul_add((f64::from(i) + 0.5) / 24.0, lo[0]),
            (hi[1] - lo[1]).mul_add((f64::from(j) + 0.5) / 24.0, lo[1]),
        ];
        (0..p.len()).all(|k| ao_troco(q, p[k], p[(k + 1) % p.len()]).0 >= r) && dentro(&anel, q)
    })
}

/// ⭐ Que âncoras de `v` (um contorno da união) são CRUZAMENTOS: não são âncora da fonte e moram
/// em DOIS sítios dela (o ponto onde dois contornos, ou dois troços de um, se cortam).
fn cruzamentos(v: &[VecVertex], fonte: &[Anel], ancoras: &[[f64; 2]], tol: f64) -> Vec<bool> {
    v.iter()
        .map(|x| {
            let p = x.anchor;
            if ancoras
                .iter()
                .any(|a| (a[0] - p[0]).hypot(a[1] - p[1]) <= tol)
            {
                return false;
            }
            let mut sitios: Vec<(usize, f64)> = Vec::new();
            for (k, a) in fonte.iter().enumerate() {
                for s in a.onde(p, tol) {
                    if !sitios
                        .iter()
                        .any(|&(kk, ss)| kk == k && (ss - s).abs() <= 10.0 * tol)
                    {
                        sitios.push((k, s));
                    }
                }
            }
            sitios.len() >= 2
        })
        .collect()
}

/// O raio do maior círculo dentro do polígono `p` (grelha `24 × 24` na caixa) — as sondas e os gates.
#[cfg(test)]
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

/// ⭐ As passagens NOVAS do contorno `pl` da união, sem se tocarem: `(s, e)` em amostras, cada uma
/// vai de `s` para a frente até `e`. A boca é a corda mais larga (`< w`) de uma passagem que o traço
/// cobre toda.
/// `cruz[k]`: a âncora `k` do contorno é um CRUZAMENTO da união ([`cruzamentos`]). `buraco`: `pl` é
/// um buraco, e a cor fica do lado do sentido `cor` (o do maior contorno).
fn fendas(pl: &[[f64; 2]], cruz: &[bool], w: f64, buraco: bool, cor: f64) -> Vec<(usize, usize)> {
    let n = pl.len();
    let mut acc = vec![0.0];
    for i in 0..n {
        let (a, b) = (pl[i], pl[(i + 1) % n]);
        acc.push(acc[i] + (b[0] - a[0]).hypot(b[1] - a[1]));
    }
    let total = acc[n];
    // ⭐ NOVA = o arco tem um cruzamento: entre dois cruzamentos a união segue UM pedaço contínuo
    // de um contorno da fonte (o mesmo caminho: o vale de um dente fica), e através de um ela salta
    // de um pedaço para outro (a fenda entre dois membros, a ponta que entra num bolso).
    let mut pre = vec![0_usize; n + 1];
    for i in 0..n {
        pre[i + 1] =
            pre[i] + usize::from(i % AMOSTRAS == 0 && cruz.get(i / AMOSTRAS) == Some(&true));
    }
    let tem_cruzamento = |s: usize, e: usize| {
        if s <= e {
            pre[e + 1] > pre[s]
        } else {
            pre[n] > pre[s] || pre[e + 1] > 0
        }
    };
    // Os pares de amostras a menos de `w` que estão longe pelo contorno — grelha plana de lado `w`.
    let (lo, hi) = pl
        .iter()
        .fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), q| {
            (
                [lo[0].min(q[0]), lo[1].min(q[1])],
                [hi[0].max(q[0]), hi[1].max(q[1])],
            )
        });
    let lado = w.max((hi[0] - lo[0]).max(hi[1] - lo[1]) / 256.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "célula da grelha, presa a 256"
    )]
    let celula =
        |p: [f64; 2]| [0, 1].map(|k| (((p[k] - lo[k]) / lado).floor().max(0.0) as usize).min(256));
    let dim = celula(hi).map(|c| c + 1);
    let mut inicio = vec![0_usize; dim[0] * dim[1] + 1];
    for p in pl {
        let c = celula(*p);
        inicio[c[1] * dim[0] + c[0] + 1] += 1;
    }
    for k in 1..inicio.len() {
        inicio[k] += inicio[k - 1];
    }
    let mut cheio = inicio.clone();
    let mut ordem = vec![0_usize; n];
    for (i, p) in pl.iter().enumerate() {
        let c = celula(*p);
        let x = &mut cheio[c[1] * dim[0] + c[0]];
        ordem[*x] = i;
        *x += 1;
    }
    let mut pares: Vec<(f64, usize, usize)> = Vec::new();
    for (i, p) in pl.iter().enumerate() {
        let c = celula(*p);
        for y in c[1].saturating_sub(1)..=(c[1] + 1).min(dim[1] - 1) {
            for x in c[0].saturating_sub(1)..=(c[0] + 1).min(dim[0] - 1) {
                let k = y * dim[0] + x;
                for &j in &ordem[inicio[k]..inicio[k + 1]] {
                    if j <= i {
                        continue;
                    }
                    let q = pl[j];
                    if (q[0] - p[0]).hypot(q[1] - p[1]) >= w {
                        continue;
                    }
                    // A passagem é o arco mais curto entre os dois.
                    let ida = acc[j] - acc[i];
                    let (s, e, arco) = if ida <= total - ida {
                        (i, j, ida)
                    } else {
                        (j, i, total - ida)
                    };
                    // Sem cruzamento no arco é da fonte — e tudo o que está dentro dele também.
                    if arco <= 1.5 * w || !tem_cruzamento(s, e) {
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
    let area = |p: &[[f64; 2]]| -> f64 {
        (0..p.len())
            .map(|i| {
                let (a, b) = (p[i], p[(i + 1) % p.len()]);
                a[0] * b[1] - b[0] * a[1]
            })
            .sum()
    };
    // Larga: só os pares junto à MESMA boca saem (os de dentro podem ser uma passagem estreita
    // dentro de uma larga — a ponta que entra num bolso, `=5`).
    let mut largas: Vec<(usize, usize)> = Vec::new();
    let mut achadas: Vec<(usize, usize)> = Vec::new();
    let perto = |a: usize, b: usize| a.abs_diff(b).min(n - a.abs_diff(b)) <= AMOSTRAS;
    for (arco, s, e) in pares {
        let toca = |&(a, b): &(usize, usize)| {
            dentro_do_arco(a, b, s) || dentro_do_arco(a, b, e) || dentro_do_arco(s, e, a)
        };
        if achadas.iter().any(toca) || largas.iter().any(|&(a, b)| perto(a, s) && perto(b, e)) {
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
        // ⛔ Num BURACO só sai a ponta de cor que entra nele (a região do lado da cor: o sentido do
        // maior contorno): um canto estreito do próprio buraco é fechá-lo aos bocados (F59-b).
        if buraco && area(&regiao).signum() != cor {
            continue;
        }
        #[cfg(test)]
        if REGISTA.with(std::cell::Cell::get) {
            let corda = (pl[s][0] - pl[e][0]).hypot(pl[s][1] - pl[e][1]);
            EXAMINADAS.with(|x| {
                x.borrow_mut()
                    .push((s, e, arco, corda, false, raio_inscrito(&regiao)));
            });
        }
        if cabe(&regiao, 0.5 * w) {
            largas.push((s, e));
        } else {
            achadas.push((s, e));
        }
    }
    achadas
}

/// ⭐⭐ Tira de `u` (a UNIÃO dos fechados de `fonte`) cada fenda NOVA que o traço enche — o contorno
/// passa recto pela boca dela.
pub(super) fn fecha_as_fendas_que_o_traco_enche(u: &mut VecPath, fonte: &VecPath) {
    #[cfg(test)]
    ULTIMA.with(|c| *c.borrow_mut() = Some((u.clone(), fonte.clone())));
    #[cfg(test)]
    if SEM_FECHO.with(std::cell::Cell::get) {
        return;
    }
    let w = u.stroke.as_ref().map_or(0.0, |s| s.width);
    if w <= 0.0 {
        return;
    }
    let fechados: Vec<&[VecVertex]> = (0..fonte.contour_count())
        .filter_map(|c| fonte.contour(c))
        .filter(|(v, f)| *f && v.len() > 1)
        .map(|(v, _)| v)
        .collect();
    let ancoras: Vec<[f64; 2]> = fechados
        .iter()
        .flat_map(|v| v.iter().map(|x| x.anchor))
        .collect();
    let fonte: Vec<Anel> = fechados.iter().map(|v| Anel::novo(polilinha(v))).collect();
    let tol = 1e-3 * w;
    let mut contornos: Vec<ph2d_vec_scene::Contour> = (0..u.contour_count())
        .filter_map(|c| u.contour(c))
        .map(|(v, closed)| ph2d_vec_scene::Contour {
            verts: v.to_vec(),
            closed,
        })
        .collect();
    // Um BURACO tem o sentido contrário ao do maior contorno.
    let areas: Vec<f64> = contornos
        .iter()
        .map(|c| {
            let p = polilinha(&c.verts);
            (0..p.len())
                .map(|i| {
                    let (a, b) = (p[i], p[(i + 1) % p.len()]);
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f64>()
        })
        .collect();
    let maior = areas
        .iter()
        .copied()
        .fold(0.0_f64, |m, a| if a.abs() > m.abs() { a } else { m });
    let buraco: Vec<bool> = areas.iter().map(|a| a * maior < 0.0).collect();
    let cruz: Vec<Vec<bool>> = contornos
        .iter()
        .map(|c| cruzamentos(&c.verts, &fonte, &ancoras, tol))
        .collect();
    let mut mexeu = false;
    for ci in 0..contornos.len() {
        let c = &contornos[ci];
        if !c.closed || c.verts.len() < 2 {
            continue;
        }
        let pl = polilinha(&c.verts);
        // ⛔ Um contorno que o traço engole INTEIRO (o buraco pequeno, a ilhota) fica como está:
        // fechá-lo foi recusado pelo dono (F59-b).
        if !cabe(&pl, 0.5 * w) {
            continue;
        }
        let achadas = fendas(&pl, &cruz[ci], w, buraco[ci], maior.signum());
        if !achadas.is_empty() {
            contornos[ci].verts = sem_as_fendas(&contornos[ci].verts, &achadas);
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

/// O contorno fechado `v` sem as passagens `fendas` (cada uma da amostra `s` para a frente até `e`,
/// sem se tocarem): ficam os pedaços entre elas, e a recta de cada `s` ao seu `e` liga-os.
fn sem_as_fendas(v: &[VecVertex], fendas: &[(usize, usize)]) -> Vec<VecVertex> {
    let m = v.len();
    #[expect(clippy::cast_precision_loss, reason = "parâmetro do contorno")]
    let u = |k: usize| k as f64 / AMOSTRAS as f64;
    let mut ordem: Vec<(f64, f64)> = fendas.iter().map(|&(s, e)| (u(s), u(e))).collect();
    ordem.sort_by(|a, b| a.0.total_cmp(&b.0));
    // O contorno aberto `v v v[0]` — duas voltas para um pedaço poder passar pela emenda.
    let mut dupla: Vec<VecVertex> = v.iter().chain(v).copied().collect();
    dupla.push(v[0]);
    let mut saida: Vec<VecVertex> = Vec::new();
    for (i, &(_, ue)) in ordem.iter().enumerate() {
        // Do fim desta ao início da seguinte (a última volta até à 1.ª).
        let us = ordem[(i + 1) % ordem.len()].0;
        #[expect(clippy::cast_precision_loss, reason = "parâmetro do contorno")]
        let fim = if us > ue { us } else { us + m as f64 };
        let mut pedaco: Vec<VecVertex> = super::super::frente::recorta(&dupla, ue, fim)
            .into_iter()
            .map(|(x, _)| x)
            .collect();
        if let Some(p) = pedaco.first_mut() {
            p.in_handle = p.anchor;
        }
        if let Some(p) = pedaco.last_mut() {
            p.out_handle = p.anchor;
        }
        saida.extend(pedaco);
    }
    saida
}

#[cfg(test)]
#[path = "skin_desenho_fendas_tests.rs"]
mod tests;
