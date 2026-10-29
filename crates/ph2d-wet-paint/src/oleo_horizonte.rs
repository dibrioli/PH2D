//! O HORIZONTE do Óleo Real: as amostras por fatia, a visibilidade GTAO e o céu do mapa, o reflexo (partido de
//! `oleo_real.rs`, 29/10).

use super::*;

/// As amostras do horizonte de uma [`escala_do_horizonte`]: por fatia a direcção `(cos φ, sin φ)` e, para cada
/// lado e cada raio do [`HORIZONTE`], o deslocamento arredondado e a distância dele (`d = 0` = amostra saltada).
/// São as MESMAS contas que cada célula fazia (ao bit), feitas uma vez por escala em vez de 72 vezes por célula.
pub(super) struct Amostras {
    pub(super) dir: [(f64, f64); FATIAS],
    pub(super) desloc: [[[(isize, isize, f64); HORIZONTE.len()]; 2]; FATIAS],
}

impl Amostras {
    pub(super) fn de(escala: f64) -> Self {
        use core::f64::consts::PI;
        let mut a = Self {
            dir: [(0.0, 0.0); FATIAS],
            desloc: [[[(0, 0, 0.0); HORIZONTE.len()]; 2]; FATIAS],
        };
        for k in 0..FATIAS {
            let phi = PI * k as f64 / FATIAS as f64;
            let (dx, dy) = (libm::cos(phi), libm::sin(phi));
            a.dir[k] = (dx, dy);
            for (j, sgn) in [1.0, -1.0].into_iter().enumerate() {
                for (i, r) in HORIZONTE.into_iter().enumerate() {
                    let r = r * escala;
                    let (ox, oy) = (exato::round(sgn * dx * r), exato::round(sgn * dy * r));
                    a.desloc[k][j][i] = (ox as isize, oy as isize, libm::hypot(ox, oy));
                }
            }
        }
        a
    }

    /// As da escala `escala`, guardadas por fio (a última escala pedida).
    pub(super) fn com<R>(escala: f64, f: impl FnOnce(&Self) -> R) -> R {
        thread_local! {
            static GUARDADAS: core::cell::RefCell<Option<(u64, Amostras)>> = const { core::cell::RefCell::new(None) };
        }
        GUARDADAS.with(|g| {
            let mut g = g.borrow_mut();
            if g.as_ref().is_none_or(|(b, _)| *b != escala.to_bits()) {
                *g = Some((escala.to_bits(), Self::de(escala)));
            }
            f(&g.as_ref().unwrap().1)
        })
    }
}

/// A visibilidade do céu pelo cosseno (GTAO, Jimenez 2016, vista ortográfica +z) sobre a altura FINA:
/// [`FATIAS`] fatias, cada uma com o horizonte dos dois lados nas distâncias [`HORIZONTE`], preso à
/// horizontal (a tela tapa o chão). Plano aberto = 1. Contra o Cycles (`oraculo/candidatas_ao.py`):
/// viés p50 +0,03 no Dabber, +0,11 no Oil Bristle (os sulcos dos fios tapam mais do que se vê aqui).
pub(super) fn visibilidade_e_ceu(
    campo: &CampoAltura,
    x: isize,
    y: isize,
    n: [f64; 3],
    escala: f64,
    ceu: Option<&CeuDoMapa<'_>>,
) -> (f64, Option<[f64; 3]>) {
    Amostras::com(escala, |am| visibilidade_com(campo, x, y, n, am, ceu))
}

pub(super) fn visibilidade_com(
    campo: &CampoAltura,
    x: isize,
    y: isize,
    n: [f64; 3],
    am: &Amostras,
    ceu: Option<&CeuDoMapa<'_>>,
) -> (f64, Option<[f64; 3]>) {
    use core::f64::consts::{FRAC_PI_2, PI, TAU};
    let h0 = campo.at_fina(x, y);
    let mut acc = 0.0;
    let mut e = [0.0f64; 4];
    for k in 0..FATIAS {
        let phi = PI * k as f64 / FATIAS as f64;
        let (dx, dy) = am.dir[k];
        let mut s = [0.0f64; 2];
        for (j, lado) in am.desloc[k].iter().enumerate() {
            for &(ox, oy, d) in lado {
                if d == 0.0 {
                    continue;
                }
                let dh = campo.at_fina(x + ox, y + oy) - h0;
                s[j] = s[j].max(dh / exato::sqrt(dh * dh + d * d));
            }
        }
        // A normal projectada no plano da fatia: o comprimento pesa a fatia, o ângulo `g` mede-se do zénite.
        let no = -n[0] * dy + n[1] * dx;
        let np = [n[0] + no * dy, n[1] - no * dx, n[2]];
        let len = exato::sqrt(np[0] * np[0] + np[1] * np[1] + np[2] * np[2]);
        if len < 1e-9 {
            continue;
        }
        let lado = np[0] * dx + np[1] * dy;
        let g = libm::acos((np[2] / len).clamp(-1.0, 1.0)).copysign(lado);
        let (t1, t2) = (FRAC_PI_2 - libm::asin(s[0]), FRAC_PI_2 - libm::asin(s[1]));
        let a1 = g + t1.min(FRAC_PI_2 - g);
        let a2 = g - t2.min(FRAC_PI_2 + g);
        let (cg, sg) = (libm::cos(g), libm::sin(g));
        let v = 0.25 * (-libm::cos(2.0 * a1 - g) + cg + 2.0 * a1 * sg)
            + 0.25 * (-libm::cos(2.0 * a2 - g) + cg + 2.0 * a2 * sg);
        acc += len * v;
        // O GANHO do HDRI no arco FÍSICO de cada lado — do zénite até ao horizonte medido, preso ao hemisfério
        // da normal: `∫ L·cos(θ−γ)·sinθ dθ` em forma fechada pelas somas, ÷ o mesmo arco sob céu uniforme
        // (o canal 3). O arco da `v` acima (o de Jimenez, deslocado de `γ` e calibrado contra o Cycles no céu
        // uniforme) deixava a encosta virada para a janela vê-la através da parede: difuso da fenda 1,76 × 0,66.
        if let Some(cm) = ceu {
            let u = cm.u0 + phi / TAU;
            for (a, ud, sinal) in [
                (t1.min(FRAC_PI_2 + g), u, 1.0),
                (t2.min(FRAC_PI_2 - g), u + 0.5, -1.0),
            ] {
                let [c1, c2] = cm.s.ate(ud, a);
                for ch in 0..4 {
                    e[ch] += len * (cg * c1[ch] + sinal * sg * c2[ch]);
                }
            }
        }
    }
    let vis = (acc / FATIAS as f64).clamp(0.0, 1.0);
    let dir = ceu.map(|cm| {
        if e[3] > 1e-9 {
            [0, 1, 2].map(|ch| e[ch] / e[3] / cm.cima[ch].max(1e-9))
        } else {
            [1.0; 3]
        }
    });
    (vis, dir)
}

/// O céu do HDRI visto pelas fatias (W2 E2): as [`Sectores`] do mapa, o azimute do mapa da fatia `φ = 0`
/// e a irradiância do plano ABERTO pelas MESMAS 8 direcções (a tinta lisa sai exactamente com a cor dela).
pub(super) struct CeuDoMapa<'a> {
    s: &'a Sectores,
    u0: f64,
    pub(super) cima: [f64; 3],
}

impl<'a> CeuDoMapa<'a> {
    pub(super) fn novo(o: &OleoReal, claro: f64, s: &'a Sectores) -> Self {
        let u0 = u_do_mapa(o, claro);
        let mut cima = [0.0; 3];
        let lados = (1.0 / CUNHA) as usize;
        for k in 0..lados {
            let [c1, _] = s.ate(u0 + k as f64 * CUNHA, core::f64::consts::FRAC_PI_2);
            for ch in 0..3 {
                cima[ch] += c1[ch] / c1[3] / (2 * FATIAS) as f64;
            }
        }
        Self { s, u0, cima }
    }

    pub(super) fn de(o: &OleoReal) -> Option<CeuDoMapa<'static>> {
        let m = o.ambiente.mapa()?;
        Some(CeuDoMapa::novo(o, m.claro, m.sectores.as_ref()?))
    }
}

/// O `u` do mapa (fracção da volta, a de `Mapa::amostra`) da direcção da tela de ângulo `0` (x para a
/// direita); a de ângulo `φ` (y para BAIXO) é `u + φ/2π` — o espelho de y e a rotação de [`para_o_mapa`].
pub(super) fn u_do_mapa(o: &OleoReal, claro: f64) -> f64 {
    let l = para_a_luz(o);
    let rot = claro - libm::atan2(-l[1], l[0]);
    1.0 - rot / core::f64::consts::TAU
}

/// A fracção LIVRE do reflexo do ambiente: o horizonte da altura fina na direcção reflectida `r`
/// (distâncias [`HORIZONTE`]); o lobo de largura `rug` passa de tapado a livre à volta dele. Com a
/// visibilidade média no lugar disto a fenda reflectia o estúdio: luminância 74 × 47 do Cycles.
pub(super) fn reflexo_livre(
    campo: &CampoAltura,
    x: isize,
    y: isize,
    r: [f64; 3],
    rug: f64,
    escala: f64,
) -> f64 {
    let rxy = libm::hypot(r[0], r[1]);
    if rxy < 1e-6 {
        return 1.0;
    }
    let (dx, dy) = (r[0] / rxy, r[1] / rxy);
    let h0 = campo.at_fina(x, y);
    let mut sen = 0.0f64;
    for d in HORIZONTE {
        let d = d * escala;
        let (ox, oy) = (exato::round(dx * d), exato::round(dy * d));
        let dist = libm::hypot(ox, oy);
        if dist == 0.0 {
            continue;
        }
        let dh = campo.at_fina(x + ox as isize, y + oy as isize) - h0;
        sen = sen.max(dh / exato::sqrt(dh * dh + dist * dist));
    }
    ((r[2] - sen) / rug.max(0.05) + 0.5).clamp(0.0, 1.0)
}
