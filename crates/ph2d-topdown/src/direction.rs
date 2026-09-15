//! **A QUANTIZAÇÃO da intenção** — em que direcções este corpo aceita andar.
//!
//! ⚠️⚠️ **Ela corre no espaço da INTENÇÃO, antes do viewpoint** (ver o cabeçalho da
//! crate): num tabuleiro isométrico, *«4 direcções»* tem de encaixar nas diagonais
//! do tabuleiro, e um `snap` feito depois da reprojecção encaixa nos eixos do
//! **ecrã**. As duas ordens compilam e só uma é um jogo isométrico.
//!
//! # ⚠️ E a diagonal NÃO é mais rápida
//!
//! Duas teclas dão `(1, 1)`, que tem comprimento `√2` — um corpo que ande isso
//! anda **41 % mais depressa na diagonal**, que é o defeito de principiante mais
//! antigo do género (e que o `Input.get_vector` do Godot cura normalizando). Aqui
//! a cura é da lei: o comprimento é **cortado a 1**, nunca esticado, para que um
//! manípulo analógico a meio curso continue a andar a meia velocidade.

use crate::{Vec2, len, normalize};

/// **Em que direcções o corpo aceita andar.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DirectionMode {
    /// Qualquer ângulo — o manípulo analógico e o rato.
    Free,
    /// Os oito rumos, de 45 em 45 graus. É o default, e é o do *8 Direction* do
    /// Construct e do *Top-down movement* do GDevelop com diagonais ligadas.
    #[default]
    EightWay,
    /// Só os quatro rumos cardeais. O sokoban, o roguelike, o Pokémon.
    FourWay,
    /// Só o eixo horizontal da intenção.
    AxisX,
    /// Só o eixo vertical da intenção.
    AxisY,
}

impl DirectionMode {
    /// Todos, na ordem em que o painel os mostra. ⛔ **É a FONTE da contagem** —
    /// um modo novo aparece no painel sem ninguém somar nada à mão.
    pub const ALL: [Self; 5] = [
        Self::Free,
        Self::EightWay,
        Self::FourWay,
        Self::AxisX,
        Self::AxisY,
    ];

    /// O rótulo que o painel pinta.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Free => "Free",
            Self::EightWay => "8 Directions",
            Self::FourWay => "4 Directions",
            Self::AxisX => "Horizontal Only",
            Self::AxisY => "Vertical Only",
        }
    }

    /// De quantos em quantos graus este modo encaixa — `None` é livre.
    ///
    /// ⚠️ Os dois modos de eixo **não** são um encaixe de 180°: eles apagam uma
    /// componente, e apagar não é o mesmo que rodar para o rumo mais perto (uma
    /// intenção a 80° daria `→` num encaixe e `↑` num apagamento de `x`).
    #[must_use]
    pub const fn snap_deg(self) -> Option<f32> {
        match self {
            Self::Free | Self::AxisX | Self::AxisY => None,
            Self::EightWay => Some(45.0),
            Self::FourWay => Some(90.0),
        }
    }
}

/// **A porta**: entrada crua ⇒ intenção quantizada, de comprimento `<= 1`.
///
/// O comprimento sobrevive (cortado a `1`) para que um manípulo a meio curso ande
/// a meia velocidade; só a **direcção** é encaixada.
#[must_use]
pub fn quantize(raw: Vec2, mode: DirectionMode) -> Vec2 {
    let bruto = match mode {
        DirectionMode::AxisX => [raw[0], 0.0],
        DirectionMode::AxisY => [0.0, raw[1]],
        _ => raw,
    };
    let comprimento = len(bruto).min(1.0);
    if comprimento < 1.0e-6 {
        return [0.0, 0.0];
    }
    let Some(dir) = normalize(bruto) else {
        return [0.0, 0.0];
    };
    let Some(passo) = mode.snap_deg() else {
        // ⚠️ **Sem encaixe e dentro do corte, a saída é a ENTRADA, ao bit.** A 1.ª
        // redacção devolvia `dir * comprimento` — uma ida e volta por `normalize`
        // que muda o último bit e faz `Free + TopDown` deixar de ser identidade.
        // O gate `o_neutro_atravessa_a_porta_sem_tocar_no_vector` apanhou-o.
        return if len(bruto) <= 1.0 {
            bruto
        } else {
            [dir[0] * comprimento, dir[1] * comprimento]
        };
    };
    // ⚠️ `atan2`/`cos`/`sin` do `libm`, nunca do `std`: esta direcção entra na
    // velocidade de um corpo que o `physics_ecs_c9` compara entre TRÊS sistemas
    // operacionais, e ali 1 ulp é um bug.
    let ang = libm::atan2f(dir[1], dir[0]);
    let passo_rad = passo.to_radians();
    let encaixado = libm::roundf(ang / passo_rad) * passo_rad;
    [
        libm::cosf(encaixado) * comprimento,
        libm::sinf(encaixado) * comprimento,
    ]
}
