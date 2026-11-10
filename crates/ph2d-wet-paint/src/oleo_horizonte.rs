//! O HORIZONTE do Óleo Real: as amostras por fatia, a visibilidade GTAO e o céu do mapa, o reflexo (partido de
//! `oleo_real.rs`, 29/10).

use super::*;

/// As amostras do horizonte de uma [`escala_do_horizonte`]: por fatia a direcção `(cos φ, sin φ)` e, para cada
/// lado e cada raio do [`HORIZONTE`], o deslocamento arredondado e a distância dele (`d = 0` = amostra saltada).
/// São as MESMAS contas que cada célula fazia (ao bit), feitas uma vez por escala em vez de 72 vezes por célula.
pub(super) struct Amostras {
    /// As fatias em uso ([`QualidadeDaLuz::fatias`]); as de cima ficam por usar e as distâncias além das da
    /// qualidade com `d = 0` (saltadas).
    pub(super) nf: usize,
    pub(super) dir: [(f64, f64); FATIAS],
    pub(super) desloc: [[[(isize, isize, f64); HORIZONTE.len()]; 2]; FATIAS],
    /// Com a normal do PAPEL ([`PAPEL_PLANO`]), por fatia: o comprimento da normal projectada, o ângulo `g` do zénite
    /// e `(cos g, sin g)` — `None` onde a projecção se anula. As MESMAS expressões da [`visibilidade_com`] e do
    /// [`super::oleo_sombra::rebote_no_papel`] com essa normal (R3: eram refeitas em cada célula do papel).
    pub(super) papel: [Option<PlanoDaFatia>; FATIAS],
    /// Os dois lados de cada fatia do papel no [`super::oleo_sombra::rebote_no_papel`]: `gs = ±g` e o que só dele
    /// depende — o tecto `b` do arco, `cos gs`, `sin gs` e o integral até `b`.
    pub(super) papel_lados: [[LadoDoPapel; 2]; FATIAS],
}

/// Um lado de uma fatia do papel no rebote: as MESMAS expressões que cada célula fazia.
#[derive(Clone, Copy, Default)]
pub(super) struct LadoDoPapel {
    pub(super) gs: f64,
    pub(super) b: f64,
    pub(super) cgs: f64,
    pub(super) sgs: f64,
    pub(super) int_b: f64,
}

impl LadoDoPapel {
    fn de(gs: f64) -> Self {
        use core::f64::consts::FRAC_PI_2;
        let b = (FRAC_PI_2 + gs).min(FRAC_PI_2);
        let (cgs, sgs) = (libm::cos(gs), libm::sin(gs));
        Self {
            gs,
            b,
            cgs,
            sgs,
            int_b: Self::int_com(b, gs, cgs, sgs),
        }
    }

    /// `∫₀^a cos(θ − g)·sin θ dθ` com `cos g`, `sin g` dados — a `int` do rebote, verbatim.
    pub(super) fn int_com(a: f64, g: f64, cg: f64, sg: f64) -> f64 {
        0.25 * (-libm::cos(2.0 * a - g) + cg + 2.0 * a * sg)
    }
}

/// As peças de uma fatia que só dependem da normal (a do papel) e da direcção da fatia.
#[derive(Clone, Copy)]
pub(super) struct PlanoDaFatia {
    pub(super) len: f64,
    pub(super) g: f64,
    pub(super) cg: f64,
    pub(super) sg: f64,
}

impl PlanoDaFatia {
    /// A normal `n` projectada no plano da fatia `(dx, dy)` — a conta de cada célula, verbatim.
    pub(super) fn de(n: [f64; 3], dx: f64, dy: f64) -> Option<Self> {
        let no = -n[0] * dy + n[1] * dx;
        let np = [n[0] + no * dy, n[1] - no * dx, n[2]];
        let len = exato::sqrt(np[0] * np[0] + np[1] * np[1] + np[2] * np[2]);
        if len < 1e-9 {
            return None;
        }
        let lado = np[0] * dx + np[1] * dy;
        let g = libm::acos((np[2] / len).clamp(-1.0, 1.0)).copysign(lado);
        Some(Self {
            len,
            g,
            cg: libm::cos(g),
            sg: libm::sin(g),
        })
    }
}

/// `n` é a normal do papel, bit a bit (os zeros com sinal contam: o `copysign` lê-os).
pub(super) fn e_o_papel(n: [f64; 3]) -> bool {
    n.map(f64::to_bits) == PAPEL_PLANO.map(f64::to_bits)
}

impl Amostras {
    pub(super) fn de(escala: f64, q: QualidadeDaLuz) -> Self {
        use core::f64::consts::PI;
        let nf = q.fatias();
        let mut a = Self {
            nf,
            dir: [(0.0, 0.0); FATIAS],
            desloc: [[[(0, 0, 0.0); HORIZONTE.len()]; 2]; FATIAS],
            papel: [None; FATIAS],
            papel_lados: [[LadoDoPapel::default(); 2]; FATIAS],
        };
        for k in 0..nf {
            let phi = PI * k as f64 / nf as f64;
            let (dx, dy) = (libm::cos(phi), libm::sin(phi));
            a.dir[k] = (dx, dy);
            a.papel[k] = PlanoDaFatia::de(PAPEL_PLANO, dx, dy);
            if let Some(p) = a.papel[k] {
                a.papel_lados[k] = [1.0, -1.0].map(|sgn| LadoDoPapel::de(sgn * p.g));
            }
            for (j, sgn) in [1.0, -1.0].into_iter().enumerate() {
                for (i, &r) in q.horizonte().iter().enumerate() {
                    let r = r * escala;
                    let (ox, oy) = (exato::round(sgn * dx * r), exato::round(sgn * dy * r));
                    a.desloc[k][j][i] = (ox as isize, oy as isize, libm::hypot(ox, oy));
                }
            }
        }
        a
    }

    /// As da escala `escala` e da qualidade `q`, guardadas por fio (as últimas pedidas).
    pub(super) fn com<R>(escala: f64, q: QualidadeDaLuz, f: impl FnOnce(&Self) -> R) -> R {
        type Chave = (u64, QualidadeDaLuz);
        thread_local! {
            static GUARDADAS: core::cell::RefCell<Option<(Chave, Amostras)>> = const { core::cell::RefCell::new(None) };
        }
        GUARDADAS.with(|g| {
            let mut g = g.borrow_mut();
            let chave = (escala.to_bits(), q);
            if g.as_ref().is_none_or(|(b, _)| *b != chave) {
                *g = Some((chave, Self::de(escala, q)));
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
    (escala, q): (f64, QualidadeDaLuz),
    ceu: Option<&CeuDoMapa<'_>>,
) -> (f64, Option<[f64; 3]>) {
    Amostras::com(escala, q, |am| visibilidade_com(campo, x, y, n, am, ceu))
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
    let papel = e_o_papel(n);
    for k in 0..am.nf {
        let phi = PI * k as f64 / am.nf as f64;
        let (dx, dy) = am.dir[k];
        let mut s = [0.0f64; 2];
        for (j, lado) in am.desloc[k].iter().enumerate() {
            for &(ox, oy, d) in lado {
                if d == 0.0 {
                    continue;
                }
                let dh = campo.at_fina(x + ox, y + oy) - h0;
                // O horizonte começa no plano (`0`): uma amostra que não sobe não o move (ao bit, também `NaN`).
                if dh <= 0.0 || dh.is_nan() {
                    continue;
                }
                s[j] = s[j].max(dh / exato::sqrt(dh * dh + d * d));
            }
        }
        // A normal projectada no plano da fatia: o comprimento pesa a fatia, o ângulo `g` mede-se do zénite.
        let plano = if papel {
            am.papel[k]
        } else {
            PlanoDaFatia::de(n, dx, dy)
        };
        let Some(PlanoDaFatia { len, g, cg, sg }) = plano else {
            continue;
        };
        let (t1, t2) = (FRAC_PI_2 - libm::asin(s[0]), FRAC_PI_2 - libm::asin(s[1]));
        let a1 = g + t1.min(FRAC_PI_2 - g);
        let a2 = g - t2.min(FRAC_PI_2 + g);
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
    let vis = (acc / am.nf as f64).clamp(0.0, 1.0);
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
        // As direcções das fatias em uso (`φ = π·k/nf` e o lado oposto), nem mais nem menos: a tinta lisa sai com a cor.
        let nf = o.qualidade.fatias();
        let salto = FATIAS / nf;
        let lados = (1.0 / CUNHA) as usize / salto;
        for k in 0..lados {
            let [c1, _] = s.ate(
                u0 + (k * salto) as f64 * CUNHA,
                core::f64::consts::FRAC_PI_2,
            );
            for ch in 0..3 {
                cima[ch] += c1[ch] / c1[3] / (2 * nf) as f64;
            }
        }
        Self { s, u0, cima }
    }

    /// O céu do mapa de `o`, guardado por fio pela chave de que ele depende (o ambiente e a direcção da luz): as
    /// MESMAS contas, ao bit — R3: era refeito em cada célula (8 leituras do mapa, a luz e um `atan2`).
    pub(super) fn de(o: &OleoReal) -> Option<CeuDoMapa<'static>> {
        type Guardado = Option<((u8, u64, u64, QualidadeDaLuz), (f64, [f64; 3]))>;
        thread_local! {
            static GUARDADO: core::cell::Cell<Guardado> = const { core::cell::Cell::new(None) };
        }
        let m = o.ambiente.mapa()?;
        let s = m.sectores.as_ref()?;
        let chave = (
            o.ambiente.indice(),
            o.altitude.to_bits(),
            o.azimute.to_bits(),
            o.qualidade,
        );
        let (u0, cima) = GUARDADO.with(|g| match g.get() {
            Some((k, v)) if k == chave => v,
            _ => {
                let c = CeuDoMapa::novo(o, m.claro, s);
                g.set(Some((chave, (c.u0, c.cima))));
                (c.u0, c.cima)
            }
        });
        Some(CeuDoMapa { s, u0, cima })
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
    (escala, q): (f64, QualidadeDaLuz),
) -> f64 {
    let rxy = libm::hypot(r[0], r[1]);
    if rxy < 1e-6 {
        return 1.0;
    }
    let (dx, dy) = (r[0] / rxy, r[1] / rxy);
    let h0 = campo.at_fina(x, y);
    let mut sen = 0.0f64;
    for &d in q.horizonte() {
        let d = d * escala;
        let (ox, oy) = (exato::round(dx * d), exato::round(dy * d));
        let dist = libm::hypot(ox, oy);
        if dist == 0.0 {
            continue;
        }
        let dh = campo.at_fina(x + ox as isize, y + oy as isize) - h0;
        if dh <= 0.0 || dh.is_nan() {
            continue;
        }
        sen = sen.max(dh / exato::sqrt(dh * dh + dist * dist));
    }
    ((r[2] - sen) / rug.max(0.05) + 0.5).clamp(0.0, 1.0)
}
