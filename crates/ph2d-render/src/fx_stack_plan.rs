//! **O que a pilha CUSTA** — quantos saltos o campo de distância precisa e quanto cada degrau
//! espalha para fora do que recebeu.
//!
//! Irmão de [`super::fx_stack`] pelo teto de LOC, e o corte é por responsabilidade: aqui não se
//! despacha nada, só se responde *quanta textura esta pilha exige* — a pergunta que o chamador faz
//! ANTES de alocar, e que o [`stack_reach`] publica.

use ph2d_ecs::FxOp;

use crate::fx_stack::{FxOpGpu, MAX_HALF, Plan, kernel_half};

/// **Quantos saltos o JFA precisa para uma banda de `band_px`.** Os saltos são `K, K/2, …, 1` com
/// `K = 2^(n-1)`, e o alcance do JFA é a SOMA deles (`2K-1`), logo `n = bits(w)` cobre `w`.
pub fn jump_count(band_px: f32) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let w = band_px.max(1.0).ceil() as u32;
    (u32::BITS - w.leading_zeros()) as usize
}

pub(crate) fn plan_of(op: &FxOpGpu, raster_seeded: bool) -> Plan {
    let spec = FxOp::spec(op.kind);
    // ⚠️ **O TIPO decide antes do MODO, e a ordem é load-bearing.** O `mode` é um índice na lista
    // DO TIPO, então o mesmo `1` quer dizer coisas diferentes em tipos diferentes: `MODE_CONTOUR`
    // no Inner Shadow e `MODE_CREASED` na turbulência. A regra abaixo pergunta *"tem modos, e
    // escolheu o 1?"* — o que mandaria uma turbulência *Creased* para o campo de distância, um
    // efeito completamente outro, sem erro nenhum. Perguntar o tipo primeiro é a única forma que
    // não apodrece quando o 3º tipo com modos chegar.
    if op.kind == FxOp::TURBULENCE {
        return Plan::Warp;
    }
    // ⚠️ **Quem decide é o MODO, não o ser-de-dentro.** A condição dizia `spec.inner && Contour`, o
    // que era verdade só enquanto os degraus de dentro fossem os únicos com modos — uma
    // enumeração disfarçada de regra. Perguntar *"este tipo oferece escolha, e ele escolheu
    // Contour?"* faz o Glow entrar por construção, e faz o próximo tipo com modos entrar também.
    let by_distance = matches!(op.kind, FxOp::OUTLINE | FxOp::FEATHER | FxOp::BEVEL)
        || (!spec.modes.is_empty() && op.mode == FxOp::MODE_CONTOUR);
    if by_distance {
        // ⚠️ **Com geometria não há JFA.** O finalize computa o pé exato POR TEXEL, então a semente
        // e os saltos ficariam a produzir uma textura que ninguém lê — e uma mutação que os
        // neutralizava sobreviveu a todos os gates, que é como trabalho morto se anuncia.
        return Plan::Field {
            jumps: if raster_seeded {
                jump_count(op.sigma_px)
            } else {
                0
            },
            raster_seed: raster_seeded,
        };
    }
    if spec.radius_label.is_none() {
        return Plan::Point;
    }
    Plan::Blur
}

/// **Quanto ESTE degrau espalha para fora do que recebeu**, em pixels.
///
/// Três respostas, e cada uma é um fato sobre o tipo, não uma margem "por segurança":
/// - quem não cresce ([`FxKindSpec::grows`](ph2d_ecs::FxKindSpec) falso) espalha **zero** — o
///   Inner Shadow / Inner Glow desenham só DENTRO da forma, e o Color Overlay não move um texel de
///   cobertura. Margem para eles seria textura paga a troco de nada;
/// - o **Outline** espalha a LARGURA dele (`σ`), não o suporte do kernel (`3σ`): o corte é duro em
///   `Φ(−1)`, então além de `σ` não sobra nada para recortar. (O kernel ainda percorre `3σ` — o
///   *suporte* e o *alcance* são perguntas diferentes, e é por isso que são duas funções.)
/// - o resto espalha o suporte da Gaussiana.
pub fn op_reach(op: &FxOpGpu) -> u32 {
    if !FxOp::spec(op.kind).grows {
        return 0;
    }
    // **A turbulência alcança exatamente o que ela desloca.** O ruído vive em `[-1,1]` (o `fbm`
    // normaliza pela soma das amplitudes), então nenhum texel viaja mais que `Amount` — e um `3σ`
    // aqui seria margem paga por um borrão que não existe. O `+1` cobre o vizinho que a
    // interpolação bilinear lê.
    if op.kind == FxOp::TURBULENCE {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        return (op.sigma_px.max(0.0).ceil() as u32 + 1).clamp(1, MAX_HALF);
    }
    // Um Glow em modo Contour é uma BANDA, não um borrão: a queda vale exatamente zero em `w`, e
    // pagar `3σ` de margem seria textura comprada a troco de nada.
    let contour_glow = op.kind == FxOp::GLOW && op.mode == FxOp::MODE_CONTOUR;
    if contour_glow || matches!(op.kind, FxOp::OUTLINE | FxOp::FEATHER) {
        // O contorno alcança a LARGURA dele; o feather alcança METADE dela (a rampa é centrada na
        // fronteira). Nenhum dos dois paga o suporte do kernel, que é 3×.
        let span = if op.kind == FxOp::FEATHER {
            op.sigma_px * 0.5
        } else {
            op.sigma_px
        };
        return (span.max(0.0).ceil() as u32 + 1).clamp(1, MAX_HALF);
    }
    kernel_half(op.sigma_px)
}

/// **Quanto a pilha inteira espalha, em pixels, para cada lado.** Devolve
/// `(esquerda, cima, direita, baixo)`.
///
/// Cada degrau espalha o que recebeu — logo as reaches **somam** ao longo da pilha, e a margem é
/// função da pilha, nunca do maior degrau. O borrão espalha para os quatro lados; o deslocamento
/// da sombra só para o lado para onde aponta, e é por isso que a margem é assimétrica (uma sombra
/// longa para a direita não paga textura à esquerda).
///
/// ⚠️ O deslocamento de um op de DENTRO não conta: ele desloca o halo *dentro* da silhueta, e a
/// máscara o corta na borda. Quem decide é a mesma [`FxOp::spec`] que decide as rows do painel.
#[must_use]
pub fn stack_reach(ops: &[FxOpGpu]) -> (u32, u32, u32, u32) {
    let (mut l, mut t, mut r, mut b) = (0u32, 0u32, 0u32, 0u32);
    for op in ops {
        let reach = op_reach(op);
        l += reach;
        t += reach;
        r += reach;
        b += reach;
        let spec = FxOp::spec(op.kind);
        if spec.offset_labels.is_some() && spec.grows {
            let (ox, oy) = (op.offset_px[0], op.offset_px[1]);
            l += ox.min(0).unsigned_abs();
            r += ox.max(0).unsigned_abs();
            t += oy.min(0).unsigned_abs();
            b += oy.max(0).unsigned_abs();
        }
    }
    (l, t, r, b)
}
