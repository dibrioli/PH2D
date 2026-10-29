//! A LUZ do Óleo Real: a direcção, os [`super::Termos`] de uma célula, o difuso, a fenda, o multi-ricochete e a cor
//! (partido de `oleo_real.rs`, 29/10).

use super::*;

/// O vector unitário PARA a luz (x direita, y baixo, z para fora da tela).
pub(super) fn para_a_luz(o: &OleoReal) -> [f64; 3] {
    let c = libm::cos(o.altitude);
    [
        c * libm::cos(o.azimute),
        c * libm::sin(o.azimute),
        libm::sin(o.altitude),
    ]
}

/// Os termos da óptica numa célula, antes da cor: a visibilidade do céu `0..1` (GTAO pelo cosseno; plano
/// aberto = 1), a luz directa que chega `0..1` (1 − sombra), o difuso da lâmpada (plano iluminado =
/// `1 − céu`) e o brilho somado por canal (linear; o do AMBIENTE tem a cor dele).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Termos {
    pub vis: f64,
    pub luz: f64,
    pub sol: f64,
    pub espec: [f64; 3],
    /// O GANHO do céu visível por canal (W2 E2): a irradiância do HDRI no sector que cada fatia vê,
    /// ÷ a visibilidade (céu uniforme = 1), ÷ a do plano aberto (a tinta lisa = a cor dela). `None` no
    /// Clássico (céu uniforme).
    pub dir: Option<[f64; 3]>,
    /// W2 E3-B: o peso do METAL (o difuso vai com `1 − metal`), o que o VERNIZ deixa passar à camada de
    /// baixo (`1` sem verniz).
    pub metal: f64,
    pub sob_verniz: f64,
    /// W3 M1: o difuso que a SUBSURFACE soma a cada canal (já com o peso; `0` sem ela).
    pub sss: [f64; 3],
    /// W3b: o peso da TRANSMISSION ([`crate::material::transmite`]; `0` sem ela).
    pub transmissao: f64,
    /// O modo dela ([`crate::material::filtro`]: `0` Blender · `1` Clear glass).
    pub vidro_claro: f64,
}

impl Termos {
    /// O difuso relativo de uma tinta cinza `0,5` (o mapa `t_e` da sonda, comparável ao render cinza do
    /// Cycles; plano iluminado = 1), média dos canais.
    #[must_use]
    pub fn e(&self, o: &OleoReal) -> f64 {
        (0..3).map(|c| difuso(o, self, 0.5, c)).sum::<f64>() / 3.0
    }
}

/// O difuso relativo (plano iluminado = 1) do canal `c` de albedo linear `a`: o ambiente pelo sector
/// visível com o multi-ricochete de Jimenez, mais a lâmpada. No Clássico é a lei de antes, bit a bit.
#[must_use]
pub fn difuso(o: &OleoReal, t: &Termos, a: f64, c: usize) -> f64 {
    let ceu = o.forca_ambiente;
    match t.dir {
        None => ceu * multi_ricochete(t.vis, a) + t.sol,
        // O céu visto DIRECTAMENTE tem o ganho do sector; o que as paredes devolvem (`mr − V`) é o céu
        // médio delas (`oraculo/README.md` §E2: o ganho do sector também no rebote e o rebote da lâmpada
        // pioraram a régua).
        Some(d) => ceu * (d[c] * t.vis + (multi_ricochete(t.vis, a) - t.vis)) + t.sol,
    }
}

/// A visibilidade com a FORÇA `f` da fenda: as CHANCES de oclusão `(1−V)/V` multiplicam-se por `f`
/// (`V' = V / (V + f·(1−V))`). `0` = céu inteiro, `1` = `V` (bit a bit), acima escurece o meio-tom e a fenda
/// funda devagar: `V^f` levava a fenda vermelha aos pontos pretos já em `f = 2` (`oraculo/README.md` §E1).
#[must_use]
pub fn forca_da_fenda(v: f64, f: f64) -> f64 {
    if f == 1.0 {
        return v;
    }
    if f <= 0.0 || v >= 1.0 {
        return 1.0;
    }
    v / (v + f * (1.0 - v))
}

/// O multi-ricochete do céu pela cor da tinta (Jimenez 2016): a fenda recebe a luz das paredes da MESMA
/// tinta e fica no tom dela. Devolve a visibilidade CORRIGIDA (substitui `v`; nunca `v × isto`). Contra o Cycles no vermelho (`oraculo/calibra_ao.py`, 26/10), em V 0,78 ·
/// 0,70 · 0,56 · 0,47: ganho 1,16 · 1,23 · 1,36 · 1,47 × esta fórmula 1,17 · 1,25 · 1,43 · 1,58.
#[must_use]
pub fn multi_ricochete(v: f64, albedo: f64) -> f64 {
    let a = 2.0404 * albedo - 0.3324;
    let b = -4.7951 * albedo + 0.6417;
    let c = 2.7552 * albedo + 0.6903;
    v.max(((v * a + b) * v + c) * v)
}

/// O realce enche a FOLGA até 1 em vez de cortar: sem brilho a base fica intacta (neutralidade); o brilho
/// aproxima-se do branco sem degrau. O Cycles com o HDRI de estúdio põe 23 % da tinta acima de 1.
#[must_use]
pub fn rola(base: f64, espec: f64) -> f64 {
    let folga = 1.0 - base;
    if folga <= 1e-9 {
        return base.min(1.0);
    }
    1.0 - folga * libm::exp(-espec.max(0.0) / folga)
}

/// Ilumina a cor `rgb` (sRGB 0..255, sem pré-multiplicar) da célula `(cx, cy)`, de `cobertura` `0..=1`.
/// Devolve sRGB 0..255. O brilho é do filme de tinta: na orla fina ele vai com a cobertura.
pub fn ilumina(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    rgb: [f64; 3],
    cobertura: f64,
) -> [f64; 3] {
    ilumina_com(o, campo, cx, cy, rgb, cobertura, &campo.material_em(cx, cy))
}

/// [`ilumina`] com o MATERIAL da célula (W2 E3-B), combinado com o do cartão ([`MaterialCartao`]).
pub fn ilumina_com(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    rgb: [f64; 3],
    cobertura: f64,
    mat: &MaterialCelula,
) -> [f64; 3] {
    let a = rgb.map(srgb255_to_linear);
    cor_dos_termos(o, &termos_com(o, campo, cx, cy, mat, a), a, cobertura)
}

/// [`ilumina`] e a [`sombra_no_papel`] da mesma célula pintada, com UMA conta dos termos.
pub fn ilumina_e_sombra(
    o: &OleoReal,
    campo: &CampoAltura,
    cx: usize,
    cy: usize,
    rgb: [f64; 3],
    cobertura: f64,
) -> ([f64; 3], f64) {
    let a = rgb.map(srgb255_to_linear);
    let t = termos_com(o, campo, cx, cy, &campo.material_em(cx, cy), a);
    // Sob a orla PARCIAL o papel vê-se e é plano: a sombra dele é a do papel. Sob a tinta opaca ele não se vê e a
    // dos termos (a mesma conta, sem a segunda marcha) basta.
    let s = if cobertura < 1.0 {
        sombra_no_papel(o, campo, cx, cy)
    } else {
        menos_rebote(o, campo, cx, cy, sombra_dos_termos(o, &t))
    };
    (cor_dos_termos(o, &t, a, cobertura), s)
}

/// A cor sRGB `0..255` da tinta de albedo linear `a` e `cobertura` pelos termos `t` da célula.
pub(super) fn cor_dos_termos(o: &OleoReal, t: &Termos, a: [f64; 3], cobertura: f64) -> [f64; 3] {
    let t = *t;
    let cob = cobertura.clamp(0.0, 1.0);
    let mut out = [0.0; 3];
    for c in 0..3 {
        let mut base = a[c] * difuso(o, &t, a[c], c);
        if t.sss[c] != 0.0 {
            base = a[c] * (difuso(o, &t, a[c], c) + t.sss[c]);
        }
        if t.transmissao > 0.0 {
            let d = difuso(o, &t, crate::material::filtro(a[c], t.vidro_claro), c) * o.papel[c];
            base = crate::material::transmite(base, a[c], d, t.transmissao, t.vidro_claro);
        }
        if t.metal > 0.0 || t.sob_verniz != 1.0 {
            base = base * (1.0 - t.metal) * t.sob_verniz;
        }
        out[c] = srgb255_of_linear(rola(base, t.espec[c] * cob).clamp(0.0, 1.0));
    }
    out
}

/// A luz directa que chega à célula `(x, y)` de altura `h0`, `0..=1`: marcha para a luz, o raio sobe
/// `tan(alt)` por célula; a penumbra é uma faixa de ELEVAÇÃO (um sol com tamanho), por isso alarga-se com
/// a distância ao obstáculo; `sombra` escala-a linearmente (M6).
pub(super) fn luz_directa(o: &OleoReal, campo: &CampoAltura, x: isize, y: isize, h0: f64) -> f64 {
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
        let q = campo.at_sombra_em(x as f64 + dx * t, y as f64 + dy * t);
        horizonte = horizonte.max((q - h0) / t);
    }
    let luz = ((sobe - horizonte) / o.af.penumbra.max(1e-6)).clamp(0.0, 1.0);
    1.0 - o.sombra * (1.0 - luz)
}

/// A normal do PAPEL (o Sobel do plano dá `−0`: o papel longe da tinta fica ao bit).
pub(super) const PAPEL_PLANO: [f64; 3] = [-0.0, -0.0, 1.0];

/// A normal da altura FINA na célula `(x, y)` (Sobel 3×3).
pub(super) fn normal(campo: &CampoAltura, x: isize, y: isize) -> [f64; 3] {
    let h = |dx: isize, dy: isize| campo.at_fina(x + dx, y + dy);
    // §BUG-ressalto: a célula sem relevo é o PAPEL, plano — o Sobel ao pé da parede inclinava-o para fora dela e
    // ele via o céu inteiro (a faixa branca).
    if h(0, 0) == 0.0 {
        return PAPEL_PLANO;
    }
    let fx = (h(1, -1) + 2.0 * h(1, 0) + h(1, 1) - h(-1, -1) - 2.0 * h(-1, 0) - h(-1, 1)) / 8.0;
    let fy = (h(-1, 1) + 2.0 * h(0, 1) + h(1, 1) - h(-1, -1) - 2.0 * h(0, -1) - h(1, -1)) / 8.0;
    let inv = 1.0 / exato::sqrt(fx * fx + fy * fy + 1.0);
    [-fx * inv, -fy * inv, inv]
}
