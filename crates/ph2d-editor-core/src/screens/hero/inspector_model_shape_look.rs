//! **O Look da forma de colisão** no retrato da física (GEF02 onda 8): o que o cartão Look de uma «Collision
//! Shape» mostra. Irmão do `inspector_model_physics`, que está no teto de linhas.

/// O que o cartão Look de uma forma de colisão lê. `on = false` (o [`ShapeLookFacts::NONE`]): a seleção não é um
/// objeto «Collision Shape», e o cartão não nasce.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShapeLookFacts {
    pub on: bool,
    /// O ponto da grade em que o pivô cai, `[coluna, linha]` com a linha 0 em cima; `None` = Custom.
    pub ponto: Option<[u8; 2]>,
    /// O pivô em metros a partir do centro da forma, `y` para cima (o campo X/Y).
    pub metros: [f32; 2],
    /// A cor e a opacidade do overlay (`ShapeLook::tint`/`opacity`).
    pub tint: [f32; 4],
    pub opacity: f32,
    /// A forma tem cor própria; senão a amostra mostra a cor da `camada` (a do tema).
    pub cor_propria: bool,
    pub camada: u8,
}

impl ShapeLookFacts {
    /// O piso da Opacity do cartão (dono 10/11: 25 %); o mesmo de `ph2d_physics_ecs::shape_look::OPACIDADE_MINIMA`
    /// (gate `o_piso_da_opacidade_e_um_so` na `ph2d-shape-handles`).
    pub const OPACIDADE_MINIMA: f32 = 0.25; // LITERAL-PX-OK: piso de opacidade do dono, não geometria
    pub const NONE: Self = Self {
        on: false,
        ponto: Some([1, 1]),
        metros: [0.0, 0.0],
        tint: [1.0; 4],
        opacity: 1.0,
        cor_propria: false,
        camada: 0,
    };
}

impl Default for ShapeLookFacts {
    fn default() -> Self {
        Self::NONE
    }
}
