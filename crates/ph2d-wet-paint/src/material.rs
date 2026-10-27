//! O MATERIAL da tinta, por célula (Óleo Real W2 E3): o Principled que o pincel pousa e que se mistura no molhado
//! pesado pela massa, como o [`crate::grid::Grid::relevo`]. A tinta comum é [`MaterialCelula::NEUTRO`] — a imagem
//! de antes da E3, bit a bit. Plano: `docs/Painter/49_realshader/PROMPT_MISSAO_W2_material_ambiente_pinceis.md` §E3.

/// Os ajustes do Principled guardados por célula. Todos misturam-se LINEARMENTE pela massa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCelula {
    /// 0 = dieléctrico, 1 = metal (F0 = a cor da tinta).
    pub metal: f32,
    /// Multiplicador da rugosidade que o Gloss do cartão dá (1 = a de hoje).
    pub rugosidade: f32,
    /// Specular do Principled (0,5 = IOR 1,5, o de hoje).
    pub especular: f32,
    /// Quanto o reflexo dieléctrico toma a cor da tinta (0 = branco, o de hoje).
    pub tinta_reflexo: f32,
    /// Verniz (clearcoat) por cima da tinta.
    pub verniz: f32,
    /// Rugosidade do verniz.
    pub verniz_rugosidade: f32,
    /// Sheen (o veludo rasante).
    pub veludo: f32,
    /// Anisotropia do brilho ao longo da estria (0 = isotrópico).
    pub anisotropia: f32,
    /// Luz que atravessa a película fina e volta do papel.
    pub translucidez: f32,
}

impl MaterialCelula {
    pub const NEUTRO: Self = Self {
        metal: 0.0,
        rugosidade: 1.0,
        especular: 0.5,
        tinta_reflexo: 0.0,
        verniz: 0.0,
        verniz_rugosidade: 0.03,
        veludo: 0.0,
        anisotropia: 0.0,
        translucidez: 0.0,
    };

    /// `a` misturado com `b` na fracção `t` de `b` (a mesma lei de massa do relevo).
    pub fn mistura(a: Self, b: Self, t: f32) -> Self {
        let l = |x: f32, y: f32| x + (y - x) * t;
        Self {
            metal: l(a.metal, b.metal),
            rugosidade: l(a.rugosidade, b.rugosidade),
            especular: l(a.especular, b.especular),
            tinta_reflexo: l(a.tinta_reflexo, b.tinta_reflexo),
            verniz: l(a.verniz, b.verniz),
            verniz_rugosidade: l(a.verniz_rugosidade, b.verniz_rugosidade),
            veludo: l(a.veludo, b.veludo),
            anisotropia: l(a.anisotropia, b.anisotropia),
            translucidez: l(a.translucidez, b.translucidez),
        }
    }
}

impl Default for MaterialCelula {
    fn default() -> Self {
        Self::NEUTRO
    }
}
