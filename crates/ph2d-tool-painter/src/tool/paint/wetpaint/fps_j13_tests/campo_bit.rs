//! A 2.ª medição (J13b): o `CampoAltura` ao BIT entre duas composições seguidas, e as células que a geometria da luz
//! (horizonte, rebote, luz directa; `Sectores::ate` não lê o campo) teria de refazer — as mudadas dilatadas pelo que
//! cada função lê à volta. Lê o campo só pelos acessores públicos (`luz_ceu` é `pub(super)`: fica de fora).

use ph2d_wet_paint::material::MaterialCelula;
use ph2d_wet_paint::oleo_real::{CampoAltura, OleoReal, albedo_em};
use std::cell::RefCell;

/// Cópia de `oleo_real::HORIZONTE` (privada) — as distâncias da marcha do horizonte e do rebote.
const HORIZONTE: [f64; 9] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0];
const FATIAS: usize = 4;
const PASSOS_SOMBRA: usize = 24;
const CAMPOS: [&str; 9] = [
    "v", "detalhe", "sombra", "albedo", "material", "borda", "brilho", "oleo", "sss",
];

#[derive(Clone, Copy, PartialEq)]
struct Cel {
    v: f64,
    det: f64,
    sombra: f64,
    albedo: [f64; 3],
    material: MaterialCelula,
    borda: f64,
    brilho: f64,
    oleo: f64,
    sss: [f64; 3],
}

impl Cel {
    fn de(c: &CampoAltura, x: usize, y: usize) -> Self {
        let (xi, yi) = (x as isize, y as isize);
        let v = c.at(xi, yi);
        Self {
            v,
            det: c.at_fina(xi, yi) - v,
            sombra: c.at_sombra(xi, yi),
            albedo: albedo_em(c, xi, yi),
            material: c.material_em(x, y),
            borda: c.cobertura_da_borda(x, y),
            brilho: c.brilho_em(x, y),
            oleo: c.oleo_em(x, y),
            sss: c.subsuperficie_em(x, y),
        }
    }

    /// Por campo: mudou um bit.
    fn muda(&self, o: &Self) -> [bool; 9] {
        let b = |a: f64, b: f64| a.to_bits() != b.to_bits();
        let b3 = |a: [f64; 3], c: [f64; 3]| (0..3).any(|k| b(a[k], c[k]));
        [
            b(self.v, o.v),
            b(self.det, o.det),
            b(self.sombra, o.sombra),
            b3(self.albedo, o.albedo),
            self.material != o.material,
            b(self.borda, o.borda),
            b(self.brilho, o.brilho),
            b(self.oleo, o.oleo),
            b3(self.sss, o.sss),
        ]
    }
}

#[derive(Default, Clone, Copy)]
struct Acum {
    n: f64,
    cel: f64,
    qualquer: f64,
    luz: f64,
    por_campo: [f64; 9],
    refaz: [f64; 4],
    blocos: f64,
    blocos_luz: f64,
    blocos_refaz: f64,
    sem_mudanca: f64,
}

#[derive(Default)]
struct Estado {
    grade: (usize, usize),
    antes: Vec<Option<Cel>>,
    acum: Acum,
    alcance: Option<(f64, f64, usize, usize, usize)>,
}

thread_local! {
    static E: RefCell<Option<Estado>> = const { RefCell::new(None) };
}

pub(in crate::tool::paint) fn liga() {
    E.with(|e| *e.borrow_mut() = Some(Estado::default()));
}

pub(in crate::tool::paint) fn desliga() {
    E.with(|e| *e.borrow_mut() = None);
}

/// Um bitset por linha da janela: `dst[x] |= src[x + ox]`.
fn ou_deslocado(dst: &mut [u64], src: &[u64], ox: isize) {
    let n = src.len() as isize;
    let palavra = |q: isize| if q < 0 || q >= n { 0 } else { src[q as usize] };
    for (wi, d) in dst.iter_mut().enumerate() {
        let s = 64 * wi as isize + ox;
        let (q, r) = (s.div_euclid(64), s.rem_euclid(64) as u32);
        let lo = palavra(q) >> r;
        let hi = if r == 0 {
            0
        } else {
            palavra(q + 1) << (64 - r)
        };
        *d |= lo | hi;
    }
}

/// `refaz[c] = ∃ o ∈ offs: mudou[c + o]` sobre a janela `ww × wh` (bitsets por linha de `nw` palavras).
fn dilata(mudou: &[u64], (ww, wh, nw): (usize, usize, usize), offs: &[(isize, isize)]) -> Vec<u64> {
    let mut out = vec![0u64; nw * wh];
    for &(ox, oy) in offs {
        for y in 0..wh {
            let ys = y as isize + oy;
            if ys < 0 || ys >= wh as isize {
                continue;
            }
            let ys = ys as usize;
            ou_deslocado(
                &mut out[y * nw..(y + 1) * nw],
                &mudou[ys * nw..(ys + 1) * nw],
                ox,
            );
        }
    }
    // As palavras do fim da linha além de `ww` não contam.
    if ww % 64 != 0 {
        let m = (1u64 << (ww % 64)) - 1;
        for y in 0..wh {
            out[y * nw + nw - 1] &= m;
        }
    }
    out
}

/// Os deslocamentos que cada função lê (além de `(0,0)`): horizonte (+ o Sobel 3×3 da normal), rebote, luz directa.
fn alcances(o: &OleoReal) -> ([Vec<(isize, isize)>; 3], (f64, usize, usize)) {
    let escala = o.alcance_fenda.clamp(1.0, 24.0) / 24.0;
    let mut marcha = vec![(0, 0)];
    for k in 0..FATIAS {
        let phi = std::f64::consts::PI * k as f64 / FATIAS as f64;
        let (dx, dy) = (phi.cos(), phi.sin());
        for sgn in [1.0, -1.0] {
            for r in HORIZONTE {
                let r = r * escala;
                marcha.push((
                    (sgn * dx * r).round() as isize,
                    (sgn * dy * r).round() as isize,
                ));
            }
        }
    }
    let mut vis = marcha.clone();
    for dy in -1..=1 {
        for dx in -1..=1 {
            vis.push((dx, dy));
        }
    }
    let (lx, ly) = (o.azimute.cos(), o.azimute.sin());
    let mut luz = vec![(0, 0)];
    for t in 1..=PASSOS_SOMBRA {
        let (fx, fy) = (
            (lx * t as f64).floor() as isize,
            (ly * t as f64).floor() as isize,
        );
        for (a, b) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            luz.push((fx + a, fy + b));
        }
    }
    for v in [&mut vis, &mut marcha, &mut luz] {
        v.sort_unstable();
        v.dedup();
    }
    let cheb = |v: &[(isize, isize)]| {
        v.iter()
            .map(|&(a, b)| a.unsigned_abs().max(b.unsigned_abs()))
            .max()
            .unwrap_or(0)
    };
    let r = (escala, cheb(&marcha), cheb(&luz));
    ([vis, marcha, luz], r)
}

/// O gancho logo depois do campo no composite (só sob `cfg(test)`; inerte sem a sonda ligada).
pub(in crate::tool::paint) fn gancho(
    campo: Option<(&OleoReal, &CampoAltura)>,
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    (gw, gh): (usize, usize),
) {
    E.with(|e| {
        let mut e = e.borrow_mut();
        let Some(e) = e.as_mut() else {
            return;
        };
        let Some((o, c)) = campo else {
            return;
        };
        if e.grade != (gw, gh) {
            e.grade = (gw, gh);
            e.antes = vec![None; gw * gh];
        }
        let (offs, (escala, rh, rl)) = alcances(o);
        e.alcance = Some((escala, o.azimute, rh, rl, offs[0].len()));
        let (ww, wh) = (cx1 + 1 - cx0, cy1 + 1 - cy0);
        let nw = ww.div_ceil(64);
        let mut luz = vec![0u64; nw * wh];
        let mut por_campo = [0usize; 9];
        let (mut qualquer, mut novas) = (0usize, 0usize);
        for y in cy0..=cy1 {
            for x in cx0..=cx1 {
                let agora = Cel::de(c, x, y);
                let k = (x - 1) + (y - 1) * gw;
                let m = match &e.antes[k] {
                    Some(a) => agora.muda(a),
                    None => {
                        novas += 1;
                        [true; 9]
                    }
                };
                e.antes[k] = Some(agora);
                for (p, &b) in por_campo.iter_mut().zip(&m) {
                    *p += usize::from(b);
                }
                qualquer += usize::from(m.iter().any(|&b| b));
                // O que as quatro leem: `at_fina` (v + detalhe), `at_sombra`, o albedo e a Transparency do material.
                if m[0] || m[1] || m[2] || m[3] || m[4] {
                    let (wx, wy) = (x - cx0, y - cy0);
                    luz[wy * nw + wx / 64] |= 1u64 << (wx % 64);
                }
            }
        }
        if novas == ww * wh {
            return; // a primeira vez que a janela se vê: só inicializa
        }
        let conta = |v: &[u64]| v.iter().map(|w| w.count_ones() as f64).sum::<f64>();
        let jan = (ww, wh, nw);
        let [vis, reb, lz] = &offs;
        let d_vis = dilata(&luz, jan, vis);
        let d_reb = dilata(&luz, jan, reb);
        let d_luz = dilata(&luz, jan, lz);
        let uniao: Vec<u64> = (0..d_vis.len())
            .map(|i| d_vis[i] | d_reb[i] | d_luz[i])
            .collect();
        // Blocos 32×32 da grade (índice de célula − 1).
        let nbx = gw.div_ceil(32);
        let nby = gh.div_ceil(32);
        let (mut toca, mut bl, mut br) = (
            vec![false; nbx * nby],
            vec![false; nbx * nby],
            vec![false; nbx * nby],
        );
        let bit = |v: &[u64], wx: usize, wy: usize| v[wy * nw + wx / 64] >> (wx % 64) & 1 == 1;
        for y in cy0..=cy1 {
            for x in cx0..=cx1 {
                let (wx, wy) = (x - cx0, y - cy0);
                let b = ((y - 1) / 32) * nbx + (x - 1) / 32;
                toca[b] = true;
                bl[b] |= bit(&luz, wx, wy);
                br[b] |= bit(&uniao, wx, wy);
            }
        }
        let cb = |v: &[bool]| v.iter().filter(|&&b| b).count() as f64;
        let a = &mut e.acum;
        a.n += 1.0;
        a.cel += (ww * wh) as f64;
        a.qualquer += qualquer as f64;
        a.luz += conta(&luz);
        for (s, p) in a.por_campo.iter_mut().zip(por_campo) {
            *s += p as f64;
        }
        for (s, v) in a.refaz.iter_mut().zip([&d_vis, &d_reb, &d_luz, &uniao]) {
            *s += conta(v);
        }
        a.blocos += cb(&toca);
        a.blocos_luz += cb(&bl);
        a.blocos_refaz += cb(&br);
        a.sem_mudanca += f64::from(u8::from(qualquer == 0));
    });
}

pub(in crate::tool::paint) fn relato_da_fase(r: usize, nome: &str) -> String {
    E.with(|e| {
        let mut e = e.borrow_mut();
        let Some(e) = e.as_mut() else {
            return String::new();
        };
        let a = std::mem::take(&mut e.acum);
        let n = a.n.max(1.0);
        let pc = |v: f64| 100.0 * v / a.cel.max(1.0);
        let campos: Vec<String> = CAMPOS
            .iter()
            .zip(a.por_campo)
            .map(|(c, v)| format!("{c} {:.0} ({:.1}%)", v / n, pc(v)))
            .collect();
        let (escala, az, rh, rl, nvis) = e.alcance.unwrap_or_default();
        format!(
            "[j13b] r{r} {nome:<8}| n={:.0} (sem mudança {:.0}) janela {:.0} | campo ≠ ao bit: qualquer {:.0} ({:.1}%) \
             lido-pela-luz {:.0} ({:.1}%) | por campo: {} | refazer: horizonte {:.0} ({:.1}%) rebote {:.0} ({:.1}%) \
             luz-directa {:.0} ({:.1}%) união {:.0} ({:.1}%) | blocos32 {:.1}: c/campo-luz {:.1} ({:.1}%) a refazer {:.1} \
             ({:.1}%) | alcance: escala {escala:.3} azimute {az:.3} horizonte/rebote cheb {rh} luz cheb {rl} ({nvis} offs)",
            a.n,
            a.sem_mudanca,
            a.cel / n,
            a.qualquer / n,
            pc(a.qualquer),
            a.luz / n,
            pc(a.luz),
            campos.join(" · "),
            a.refaz[0] / n,
            pc(a.refaz[0]),
            a.refaz[1] / n,
            pc(a.refaz[1]),
            a.refaz[2] / n,
            pc(a.refaz[2]),
            a.refaz[3] / n,
            pc(a.refaz[3]),
            a.blocos / n,
            a.blocos_luz / n,
            100.0 * a.blocos_luz / a.blocos.max(1.0),
            a.blocos_refaz / n,
            100.0 * a.blocos_refaz / a.blocos.max(1.0),
        )
    })
}
