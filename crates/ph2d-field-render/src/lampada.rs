//! ⭐⭐ **A LÂMPADA DA CENA** — um ponto no mundo e a radiância que ele entrega.
//!
//! Sobrou do sombreamento traçado do modo Render (retirado em 03/10): o tipo e o piso são a
//! linguagem das luzes da cena, que o desenhista de jogo (`ph2d-mesh-forward`) lê.

/// ⭐⭐⭐ **UMA LUZ QUE É UM OBJECTO DA CENA** — um ponto no MUNDO (ordem do dono, 2026-09-14).
///
/// Ancorada no MUNDO: a direcção e a distância mudam de pixel para pixel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointLamp {
    /// Onde ela está, no **MUNDO** — a pose da entidade, propagada.
    pub world: [f32; 3],
    /// A radiância que ela entrega a **UMA unidade** de distância.
    ///
    /// ⚠️ A queda é `1/r²`, logo este é o numerador. A unidade é a do rig da casa (`cor ×
    /// intensidade × π`), medida a uma unidade — ver o doc do `ph2d_field_ecs::FieldLight`.
    pub radiance_at_one: [f32; 3],
}

/// **O PISO DA DISTÂNCIA** de uma [`PointLamp`], em unidades de mundo — a remoção de uma
/// singularidade: `1/r²` diverge quando a superfície alcança o ponto. *É o que uma luz ESFÉRICA de
/// raio `0,05` faria.* Abaixo dele a direcção passa a ser a NORMAL (o vector zero não tem direcção).
pub const POINT_LAMP_MIN_DISTANCE: f32 = 0.05;
