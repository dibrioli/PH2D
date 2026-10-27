//! O MATERIAL da tinta, por célula (Óleo Real): o Principled SIMPLIFICADO que o pincel pousa e que se mistura
//! no molhado pesado pela massa, como o [`crate::grid::Grid::relevo`]. Os 8 controlos do Blender (W3, contrato em
//! `docs/Painter/49_realshader/ESTADO_MISSAO_W3.md`): Metallic · Roughness · IOR Level · Coat · Coat Roughness ·
//! Sheen · Subsurface · Subsurface Radius. A tinta comum é [`MaterialCelula::NEUTRO`] — a imagem de antes, bit a bit.

use crate::oleo_real::MaterialCartao;

/// O raio padrão do Subsurface, em px de tela (o NEUTRO do Subsurface Radius).
pub const RAIO_SUBSUPERFICIE_PADRAO: f32 = 4.0;

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
}

/// A ÚNICA regra de combinação célula × cartão: `max` nos de neutro `0` (Metallic, Coat, Sheen, Subsurface — o
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
    }
}
