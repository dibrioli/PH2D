//! **O Óleo Real** (doc 49, [`README`](../../../docs/Painter/49_realshader/README.md) §4): a óptica da
//! tinta sobre o campo de ALTURA que a tinta já é — em luz linear, normal por Sobel, a fenda que satura no
//! tom da própria tinta (M2: `fenda = mediana · s · albedoᵏ`), a sombra macia por marcha na direcção da
//! luz e o brilho somado, separado do volume (M4). Referência em CPU; o passe da placa espelha-a.
//!
//! Lei de neutralidade: com a cavidade, a sombra e o brilho a zero, a tinta lisa sai com a cor que tinha
//! (`E_plano = 1`): a claridade continua a ser o pigmento.

use crate::colorops::transfer::{srgb255_of_linear, srgb255_to_linear};
use crate::grid::Grid;

/// Os knobs do Óleo Real. `1.0` em profundidade, sombra e brilho = o look das capturas `A_base`
/// (Rebelle: impasto 8, SoftShadows 10, Gloss 10, luz 42°).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OleoReal {
    /// Ganho da altura: `1` = a escala medida em W0(b) (≈ 15–17 px de relevo na faca).
    pub profundidade: f64,
    /// De ONDE vem a luz, em radianos, no plano da tela (x para a direita, y para BAIXO): `225°` = topo-esquerda.
    pub azimute: f64,
    /// Altura da luz sobre a tela, em radianos (o Rebelle não desce de 42°; nós sim).
    pub altitude: f64,
    /// Força da sombra projectada, `0..=1` (linear no slider — M6).
    pub sombra: f64,
    /// Força do brilho especular, `0..=1` (M4: só o termo somado; nunca o volume).
    pub brilho: f64,
    /// As constantes da lei, afinadas contra as capturas (não são knobs do artista).
    pub af: Afinacao,
}

impl Default for OleoReal {
    fn default() -> Self {
        Self {
            profundidade: 1.0,
            azimute: 225f64.to_radians(),
            altitude: 42f64.to_radians(),
            sombra: 1.0,
            brilho: 1.0,
            af: Afinacao::default(),
        }
    }
}

/// As constantes da óptica. Ponto de partida = README §2b (W0); cada uma afina-se pela régua contra as
/// capturas do Rebelle (`49_realshader/capturas/`), nunca a olho sem a régua ao lado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Afinacao {
    /// px de relevo por unidade de massa (`sett+susp`) a profundidade 1 (W0(b)/(c): massa p50 ≈ 2 600 na
    /// faca ↔ ≈ 15 px de relevo no Rebelle).
    pub px_por_massa: f64,
    /// Raio (células) das duas passagens de média que dão a FORMA da pincelada a partir da massa.
    pub raio_suave: usize,
    /// Fracção do grão de célula a célula da massa que fica no relevo (à escala cheia faz uma lixa).
    pub grao: f64,
    /// Ganho da altura dos fios (`Grid::altura`, px).
    pub fios: f64,
    /// A faixa da oclusão média (seno do horizonte acima do plano tangente) de «sem fenda» a «fundo».
    pub limiar_cavidade: f64,
    pub fundo_cavidade: f64,
    /// A lei da fenda (M2), em linear: `fenda/mediana = s_fenda · albedo^k_fenda` no fundo da cavidade.
    pub s_fenda: f64,
    pub k_fenda: f64,
    /// A luz do céu (não projecta sombra): o fundo até onde a sombra escurece.
    pub ceu: f64,
    /// Largura da penumbra, em tangente de elevação: a luz vai de cheia a nada quando o horizonte sobe
    /// `penumbra` acima da elevação dela. Maior = sombra mais macia.
    pub penumbra: f64,
    /// Rugosidade (GGX) e intensidade do brilho.
    pub rugosidade: f64,
    pub brilho_max: f64,
}

impl Default for Afinacao {
    fn default() -> Self {
        Self {
            // Régua M6 (`regua_sombra.py`, faca vertical H3, 42°): 0,0015 + penumbra 0,6 dá 53·35·6 → com
            // raio 3: 42·18·0,8 contra o Rebelle 51·12·3; 0,002 já salta para 66·55·32 (a parede passa de
            // 4,5 a 6 px). O 0,006 de W0 (sombra ÷ tan) era a altura da CÚPULA do Rebelle, não do planalto.
            px_por_massa: 0.0015,
            raio_suave: 3,
            grao: 0.15,
            fios: 1.0,
            // A oclusão da FORMA na folha `tracos.py` (Dabber/Filbert/Oil Bristle): p50 0,08 · p90 0,19 ·
            // p98 0,28 — a fenda começa no p90 e chega ao fundo além do p98 (o Rebelle: 2 % mais escuros, M1).
            limiar_cavidade: 0.15,
            fundo_cavidade: 0.35,
            s_fenda: 0.25,
            k_fenda: 2.1,
            ceu: 0.6,
            penumbra: 0.6,
            rugosidade: 0.28,
            brilho_max: 1.6,
        }
    }
}

/// Passos da marcha (sombra) e da cavidade, em células; direcções da cavidade. Custo medido em W0(a)
/// (README §2b): K=24·N=8 cabe na RTX a 4K; na CPU paga-se só na região suja.
pub const PASSOS_SOMBRA: usize = 24;
pub const PASSOS_CAVIDADE: usize = 8;
pub const DIRECOES_CAVIDADE: usize = 8;
/// A margem (células) que o render de uma região precisa à volta dela.
pub const MARGEM: usize = PASSOS_SOMBRA + 1;
/// O ALCANCE (células) de uma mudança de tinta: a célula `c` muda a forma até `2·raio_suave` à volta, e a
/// cavidade/sombra de quem a amostra até [`MARGEM`] (+1 do Sobel) — quanto a região suja tem de crescer.
#[must_use]
pub fn alcance(o: &OleoReal) -> usize {
    MARGEM + 2 * o.af.raio_suave + 1
}

/// `F0` do ligante (dieléctrico).
const F0: f64 = 0.04;

/// A altura da tinta (px de relevo ÷ px por célula) numa janela de células, de TODAS as camadas
/// visíveis, em duas escalas: a FORMA (massa suavizada) — que faz a cavidade e a sombra — e o DETALHE
/// (o grão de célula a célula e os fios) — que só inclina a normal. Com o detalhe na marcha a tinta
/// inteira lia-se como fenda (folha de 26/10: oclusão em «sal e pimenta», mediana 0,24).
pub struct CampoAltura {
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    v: Vec<f32>,
    detalhe: Vec<f32>,
}

impl CampoAltura {
    /// A janela `[x0..=x1]×[y0..=y1]` (células, `1..=W`) alargada pela margem e presa à grade.
    pub fn de(camadas: &[(&Grid, bool)], o: &OleoReal, ratio: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> Self {
        let g = camadas[0].0;
        let af = &o.af;
        // A janela leva a margem da marcha e as duas passagens do desfoque.
        let m = MARGEM + 2 * af.raio_suave;
        let (ax0, ay0) = (x0.saturating_sub(m).max(1), y0.saturating_sub(m).max(1));
        let (ax1, ay1) = ((x1 + m).min(g.w), (y1 + m).min(g.h));
        let (w, h) = (ax1 + 1 - ax0, ay1 + 1 - ay0);
        let escala = af.px_por_massa * o.profundidade / ratio.max(1) as f64;
        let mut massa = vec![0f64; w * h];
        let mut fios = vec![0f64; w * h];
        for y in 0..h {
            for x in 0..w {
                let i = (ax0 + x) + (ay0 + y) * g.s;
                for &(lg, visivel) in camadas {
                    if !visivel {
                        continue;
                    }
                    massa[x + y * w] += f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                    if !lg.altura.is_empty() {
                        fios[x + y * w] += f64::from(lg.altura[i]);
                    }
                }
            }
        }
        let suave = desfoca(&desfoca(&massa, w, h, af.raio_suave), w, h, af.raio_suave);
        let k_fios = af.fios * o.profundidade;
        let v = suave.iter().map(|&s| (s * escala) as f32).collect();
        let detalhe = (0..w * h)
            .map(|k| (af.grao * (massa[k] - suave[k]) * escala + fios[k] * k_fios) as f32)
            .collect();
        Self { x0: ax0, y0: ay0, w, h, v, detalhe }
    }

    #[inline]
    fn idx(&self, x: isize, y: isize) -> usize {
        let x = (x - self.x0 as isize).clamp(0, self.w as isize - 1) as usize;
        let y = (y - self.y0 as isize).clamp(0, self.h as isize - 1) as usize;
        x + y * self.w
    }

    /// A altura da FORMA na célula `(x, y)` (presa à janela).
    #[inline]
    pub fn at(&self, x: isize, y: isize) -> f64 {
        f64::from(self.v[self.idx(x, y)])
    }

    /// A altura com o DETALHE (a que dá a normal).
    #[inline]
    pub fn at_fina(&self, x: isize, y: isize) -> f64 {
        let i = self.idx(x, y);
        f64::from(self.v[i]) + f64::from(self.detalhe[i])
    }
}

/// Média móvel separável de raio `r` (caixa `2r+1`), presa às bordas.
fn desfoca(v: &[f64], w: usize, h: usize, r: usize) -> Vec<f64> {
    let n = (2 * r + 1) as f64;
    let mut tmp = vec![0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for k in 0..=2 * r {
                s += v[(x + k).saturating_sub(r).min(w - 1) + y * w];
            }
            tmp[x + y * w] = s / n;
        }
    }
    let mut out = vec![0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for k in 0..=2 * r {
                s += tmp[x + (y + k).saturating_sub(r).min(h - 1) * w];
            }
            out[x + y * w] = s / n;
        }
    }
    out
}

/// O vector unitário PARA a luz (x direita, y baixo, z para fora da tela).
fn para_a_luz(o: &OleoReal) -> [f64; 3] {
    let c = libm::cos(o.altitude);
    [c * libm::cos(o.azimute), c * libm::sin(o.azimute), libm::sin(o.altitude)]
}

/// Os termos da óptica numa célula, antes da cor: a cavidade `0..1`, a luz directa que chega `0..1`
/// (1 − sombra), o difuso relativo `e` (plano e iluminado = 1), o brilho somado (linear) e a oclusão
/// bruta (antes da faixa da cavidade).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Termos {
    pub cav: f64,
    pub luz: f64,
    pub e: f64,
    pub espec: f64,
    pub ocl: f64,
}

/// Ilumina a cor `rgb` (sRGB 0..255, sem pré-multiplicar) da célula `(cx, cy)`. Devolve sRGB 0..255.
pub fn ilumina(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize, rgb: [f64; 3]) -> [f64; 3] {
    let t = termos(o, campo, cx, cy);
    let mut out = [0.0; 3];
    for c in 0..3 {
        let a = srgb255_to_linear(rgb[c]);
        let fenda = 1.0 + (o.af.s_fenda * libm::pow(a, o.af.k_fenda) - 1.0) * t.cav;
        out[c] = srgb255_of_linear((a * t.e * fenda + t.espec).clamp(0.0, 1.0));
    }
    out
}

/// A luz directa que chega à célula `(x, y)` de altura `h0`, `0..=1`: marcha para a luz, o raio sobe
/// `tan(alt)` por célula, penumbra tipo SDF; `sombra` a escala linearmente (M6).
fn luz_directa(o: &OleoReal, campo: &CampoAltura, x: isize, y: isize, h0: f64) -> f64 {
    if o.sombra <= 0.0 {
        return 1.0;
    }
    let l = para_a_luz(o);
    let lxy = libm::hypot(l[0], l[1]).max(1e-9);
    let (dx, dy) = (l[0] / lxy, l[1] / lxy);
    // O horizonte na direcção da luz (tangente da elevação do obstáculo mais alto visto daqui); a luz
    // apaga-se ao longo de `penumbra` (em tangente) abaixo da elevação dela. Em papel livre o horizonte
    // é 0 e a luz é cheia — a forma `k·folga/t` limitava-a a `k·tan(alt)` também no papel livre.
    let sobe = libm::tan(o.altitude);
    let mut horizonte = f64::NEG_INFINITY;
    for t in 1..=PASSOS_SOMBRA {
        let t = t as f64;
        let q = campo.at(x + libm::round(dx * t) as isize, y + libm::round(dy * t) as isize);
        horizonte = horizonte.max((q - h0) / t);
    }
    let luz = ((sobe - horizonte) / o.af.penumbra.max(1e-6)).clamp(0.0, 1.0);
    1.0 - o.sombra * (1.0 - luz)
}

/// **A sombra no PAPEL** (M6: cai também fora da tinta): a fracção `0..1` de escurecimento de uma
/// célula SEM tinta — a luz directa que a tinta à volta lhe tira, `(1 − céu)·(1 − luz)`. Só este termo
/// chega ao papel (o difuso, a fenda e o brilho são da tinta).
pub fn sombra_no_papel(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> f64 {
    let (x, y) = (cx as isize, cy as isize);
    (1.0 - o.af.ceu) * (1.0 - luz_directa(o, campo, x, y, campo.at(x, y)))
}

/// Os [`Termos`] da célula `(cx, cy)`.
pub fn termos(o: &OleoReal, campo: &CampoAltura, cx: usize, cy: usize) -> Termos {
    let af = &o.af;
    let (x, y) = (cx as isize, cy as isize);
    let h0 = campo.at(x, y);
    // Sobel 3×3: o da FORMA dá o plano tangente da cavidade; o da altura FINA dá a normal.
    let sobel = |h: &dyn Fn(isize, isize) -> f64| {
        let gx = (h(1, -1) + 2.0 * h(1, 0) + h(1, 1) - h(-1, -1) - 2.0 * h(-1, 0) - h(-1, 1)) / 8.0;
        let gy = (h(-1, 1) + 2.0 * h(0, 1) + h(1, 1) - h(-1, -1) - 2.0 * h(0, -1) - h(1, -1)) / 8.0;
        (gx, gy)
    };
    let (gx, gy) = sobel(&|dx, dy| campo.at(x + dx, y + dy));
    let (fx, fy) = sobel(&|dx, dy| campo.at_fina(x + dx, y + dy));
    let inv = 1.0 / libm::sqrt(fx * fx + fy * fy + 1.0);
    let n = [-fx * inv, -fy * inv, inv];
    let l = para_a_luz(o);

    let luz = luz_directa(o, campo, x, y, h0);

    // A cavidade: oclusão do horizonte ACIMA DO PLANO TANGENTE (uma encosta não é uma fenda), média de
    // N direcções × K passos; depois a faixa `limiar..fundo` → 0..1.
    let sen = |v: f64| v / libm::sqrt(1.0 + v * v);
    let mut ocl = 0.0f64;
    for d in 0..DIRECOES_CAVIDADE {
        let a = core::f64::consts::TAU * d as f64 / DIRECOES_CAVIDADE as f64;
        let (dx, dy) = (libm::cos(a), libm::sin(a));
        let tan_t = gx * dx + gy * dy;
        let mut tan_max = tan_t;
        for t in 1..=PASSOS_CAVIDADE {
            let t = t as f64;
            let q = campo.at(x + libm::round(dx * t) as isize, y + libm::round(dy * t) as isize);
            tan_max = tan_max.max((q - h0) / t);
        }
        ocl += sen(tan_max) - sen(tan_t);
    }
    let ocl = ocl / DIRECOES_CAVIDADE as f64;
    let cav = ((ocl - af.limiar_cavidade) / (af.fundo_cavidade - af.limiar_cavidade)).clamp(0.0, 1.0);

    // Difuso relativo: plano e iluminado = 1.
    let ndl = (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]).max(0.0);
    let ceu = af.ceu * (0.5 + 0.5 * n[2]);
    let e = ceu + (1.0 - af.ceu) * (ndl / l[2]) * luz;

    // Brilho: GGX + Schlick, vista ortográfica (0,0,1), só da luz directa e só onde ela chega.
    let hv = [l[0], l[1], l[2] + 1.0];
    let hn = libm::sqrt(hv[0] * hv[0] + hv[1] * hv[1] + hv[2] * hv[2]);
    let ndh = ((n[0] * hv[0] + n[1] * hv[1] + n[2] * hv[2]) / hn).max(0.0);
    let a2 = af.rugosidade * af.rugosidade * af.rugosidade * af.rugosidade;
    let den = ndh * ndh * (a2 - 1.0) + 1.0;
    let d_ggx = a2 / (core::f64::consts::PI * den * den);
    let vdh = hv[2] / hn;
    let fres = F0 + (1.0 - F0) * libm::pow(1.0 - vdh, 5.0);
    let espec = o.brilho * af.brilho_max * fres * d_ggx * ndl * luz * (1.0 - cav);
    Termos { cav, luz, e, espec, ocl }
}

#[cfg(test)]
#[path = "oleo_real_tests.rs"]
mod oleo_real_tests;
