//! O MATERIAL da tinta, por célula (Óleo Real): o Principled SIMPLIFICADO que o pincel pousa e que se mistura
//! no molhado pesado pela massa, como o [`crate::grid::Grid::relevo`]. Os 8 controlos do Blender (W3, contrato em
//! `docs/Painter/49_realshader/ESTADO_MISSAO_W3.md`): Metallic · Roughness · IOR Level · Coat · Coat Roughness ·
//! Sheen · Subsurface · Subsurface Radius · Transparency (`1 − Alpha`, dono 27/10) · Transmission · Relevo do papel
//! (W3b). A tinta comum é [`MaterialCelula::NEUTRO`] — a imagem de antes, bit a bit.

use crate::oleo_real::MaterialCartao;

/// O raio padrão do Subsurface, em px de tela (o NEUTRO do Subsurface Radius).
pub const RAIO_SUBSUPERFICIE_PADRAO: f32 = 4.0;
/// O tecto do Subsurface Radius EFECTIVO, em px de tela. Recurso: o CPU do recompor — a região suja cresce
/// `3·raio` de cada lado (`oraculo/README.md` §M-SSS).
pub const RAIO_SUBSUPERFICIE_MAX: f32 = 16.0;

/// Os ajustes do Principled guardados por célula. Todos misturam-se LINEARMENTE pela massa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCelula {
    /// Metallic: 0 = dieléctrico, 1 = metal (F0 = a cor da tinta).
    pub metal: f32,
    /// Roughness ABSOLUTA (`0..1`); o NEUTRO `0,5` = a do cartão. O `roughnessMul` m antigo ↔ `0,5·m`.
    pub rugosidade: f32,
    /// IOR Level (o Specular do Blender 4): `0,5` = IOR 1,5.
    pub nivel_ior: f32,
    /// Coat (o verniz por cima da tinta).
    pub verniz: f32,
    /// Coat Roughness (absoluta).
    pub verniz_rugosidade: f32,
    /// Sheen (o veludo rasante).
    pub veludo: f32,
    /// Subsurface (o peso).
    pub subsuperficie: f32,
    /// Subsurface Radius, em px de tela.
    pub raio_subsuperficie: f32,
    /// Transparency (`1 − Alpha` do Principled, dono 27/10): a fracção da cobertura de COMPOSIÇÃO que a tinta deixa
    /// ver o que está por baixo — o pigmento, a espessura e o relevo ficam ([`transparente`]).
    pub transparencia: f32,
    /// Transmission (o «Transmission Weight» do Principled, W3b 27/10): a fracção da luz que ATRAVESSA o filme,
    /// bate no papel e volta tingida pela cor da tinta — a tinta viva e saturada com o papel por baixo
    /// ([`transmite`]). A cobertura fica (não é o Alpha).
    pub transmissao: f32,
    /// Relevo do papel (W3b 27/10): quanto a SUPERFÍCIE da tinta segue o dente do papel por baixo (a tinta fina
    /// que assenta no grão). `0` = a tinta enterra o dente (§BUG-papel), `1` = o grão inteiro na tinta.
    pub relevo_papel: f32,
}

impl MaterialCelula {
    pub const NEUTRO: Self = Self {
        metal: 0.0,
        rugosidade: 0.5,
        nivel_ior: 0.5,
        verniz: 0.0,
        verniz_rugosidade: 0.03,
        veludo: 0.0,
        subsuperficie: 0.0,
        raio_subsuperficie: RAIO_SUBSUPERFICIE_PADRAO,
        transparencia: 0.0,
        transmissao: 0.0,
        relevo_papel: 0.0,
    };

    /// `a` misturado com `b` na fracção `t` de `b` (a mesma lei de massa do relevo).
    pub fn mistura(a: Self, b: Self, t: f32) -> Self {
        let l = |x: f32, y: f32| x + (y - x) * t;
        Self {
            metal: l(a.metal, b.metal),
            rugosidade: l(a.rugosidade, b.rugosidade),
            nivel_ior: l(a.nivel_ior, b.nivel_ior),
            verniz: l(a.verniz, b.verniz),
            verniz_rugosidade: l(a.verniz_rugosidade, b.verniz_rugosidade),
            veludo: l(a.veludo, b.veludo),
            subsuperficie: l(a.subsuperficie, b.subsuperficie),
            raio_subsuperficie: l(a.raio_subsuperficie, b.raio_subsuperficie),
            transparencia: l(a.transparencia, b.transparencia),
            transmissao: l(a.transmissao, b.transmissao),
            relevo_papel: l(a.relevo_papel, b.relevo_papel),
        }
    }
}

impl Default for MaterialCelula {
    fn default() -> Self {
        Self::NEUTRO
    }
}

/// O material EFECTIVO de uma célula (célula ∘ cartão), em `f64`, nas unidades do Principled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Efectivo {
    pub metal: f64,
    pub rugosidade: f64,
    pub nivel_ior: f64,
    pub verniz: f64,
    pub verniz_rugosidade: f64,
    pub veludo: f64,
    pub subsuperficie: f64,
    pub raio_subsuperficie: f64,
    pub transparencia: f64,
    pub transmissao: f64,
    pub relevo_papel: f64,
}

/// A ÚNICA regra de combinação célula × cartão: `max` nos de neutro `0` (Metallic, Coat, Sheen, Subsurface,
/// Transparency, Transmission, Relevo do papel — o
/// cartão põe verniz em toda a tinta, a célula metálica continua metálica); `cartão × célula / neutro` nos outros
/// (Roughness, IOR Level, Coat Roughness, Subsurface Radius). Célula NEUTRA ⇒ o cartão; cartão no padrão ⇒ a célula
/// (ao bit nos três de padrão = neutro, e na Roughness pelo factor de potência de 2 — gate
/// `o_roughness_do_cartao_no_padrao_e_a_imagem_de_antes`).
#[must_use]
pub fn efectivo(c: &MaterialCelula, k: &MaterialCartao) -> Efectivo {
    let mx = |a: f32, b: f64| f64::from(a).max(b).clamp(0.0, 1.0);
    // `célula × (cartão / padrão do cartão) × (padrão do cartão / neutro da célula)`: com o cartão no padrão o
    // primeiro factor é `1` exacto e o segundo uma potência de 2 vezes o padrão (ou `1`) — a célula ao bit.
    let pr = |cel: f32, n: f32, kv: f64, kn: f64| f64::from(cel) * (kv / kn) * (kn / f64::from(n));
    let n = MaterialCelula::NEUTRO;
    let kn = MaterialCartao::NEUTRO;
    Efectivo {
        metal: mx(c.metal, k.metal),
        rugosidade: pr(c.rugosidade, n.rugosidade, k.rugosidade, kn.rugosidade),
        nivel_ior: pr(c.nivel_ior, n.nivel_ior, k.nivel_ior, kn.nivel_ior),
        verniz: mx(c.verniz, k.verniz),
        verniz_rugosidade: pr(
            c.verniz_rugosidade,
            n.verniz_rugosidade,
            k.verniz_rugosidade,
            kn.verniz_rugosidade,
        ),
        veludo: mx(c.veludo, k.veludo),
        subsuperficie: mx(c.subsuperficie, k.subsuperficie),
        raio_subsuperficie: pr(
            c.raio_subsuperficie,
            n.raio_subsuperficie,
            k.raio_subsuperficie,
            kn.raio_subsuperficie,
        ),
        transparencia: transparencia(c.transparencia, k.transparencia),
        transmissao: transparencia(c.transmissao, k.transmissao),
        relevo_papel: transparencia(c.relevo_papel, k.relevo_papel),
    }
}

/// A Transparency EFECTIVA (célula × cartão, `max` presa a `0..1`) — a mesma regra de [`efectivo`], sem o resto.
#[inline]
#[must_use]
pub fn transparencia(celula: f32, cartao: f64) -> f64 {
    f64::from(celula).max(cartao).clamp(0.0, 1.0)
}

/// ⭐ **A lei da Transparency** (dono 27/10, `1 − Alpha` do Principled): a cobertura de COMPOSIÇÃO `alfa` (a de
/// [`crate::opacity::alpha_of_mass`]) que a tinta de transparência `t` deixa: `alfa × (1 − t)`. `t = 0` devolve
/// `alfa` ao bit (`x × 1` é exacto). A massa, a espessura e o relevo ([`crate::oleo_real::CampoAltura`]) não a
/// lêem: tinta transparente com volume.
#[inline]
#[must_use]
pub fn transparente(alfa: f64, t: f64) -> f64 {
    alfa * (1.0 - t)
}

/// ⭐ **A lei da Transmission** (W3b 27/10, o «Transmission Weight» do Principled; medida contra o Cycles em
/// `docs/Painter/49_realshader/oraculo/README.md` §Transmission): o difuso LINEAR `base` de um canal de albedo
/// `a` com o peso `w`. A luz atravessa o filme (tinge-se por `a`), bate no papel e volta (tinge-se outra vez):
/// `a²` — sem o branco que o difuso de uma tinta opaca reflecte, por isso saturada. `d` é o difuso relativo da
/// célula (a luz que chega, igual para o papel sob o filme fino). `w = 0` devolve `base` ao bit.
#[inline]
#[must_use]
pub fn transmite(base: f64, a: f64, d: f64, w: f64) -> f64 {
    if w <= 0.0 {
        return base;
    }
    base * (1.0 - w) + w * a * a * d
}

#[cfg(test)]
#[path = "transparencia_tests.rs"]
mod transparencia_tests;

#[cfg(test)]
#[path = "transmissao_tests.rs"]
mod transmissao_tests;
