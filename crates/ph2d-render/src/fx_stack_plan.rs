//! **O que a pilha CUSTA** — quantos saltos o campo de distância precisa e quanto cada degrau
//! espalha para fora do que recebeu.
//!
//! Irmão de [`super::fx_stack`] pelo teto de LOC, e o corte é por responsabilidade: aqui não se
//! despacha nada, só se responde *quanta textura esta pilha exige* — a pergunta que o chamador faz
//! ANTES de alocar, e que o [`stack_reach`] publica.

use ph2d_ecs::FxOp;

use crate::fx_stack::{FxOpGpu, MAX_HALF, Plan};

/// **Quantos saltos o JFA precisa para uma banda de `band_px`.** Os saltos são `K, K/2, …, 1` com
/// `K = 2^(n-1)`, e o alcance do JFA é a SOMA deles (`2K-1`), logo `n = bits(w)` cobre `w`.
pub fn jump_count(band_px: f32) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let w = band_px.max(1.0).ceil() as u32;
    (u32::BITS - w.leading_zeros()) as usize
}

/// A meia-largura do kernel que o shader de facto percorre para um dado sigma.
///
/// **Porta única**: quem calcula a MARGEM da textura ([`stack_reach`]) e quem a preenche (o
/// shader) têm de concordar, senão o borrão é recortado na borda por uma margem que mentiu.
///
/// ⚠️ Ela mora AQUI, e não no fold: a pergunta é *quanto este passe percorre*, que é a mesma
/// família do [`op_reach`] e do [`jump_count`]. Do outro lado ela obrigava este módulo a importar
/// de volta do arquivo que o importa.
#[must_use]
pub fn kernel_half(sigma_px: f32) -> u32 {
    let sigma = sigma_px.max(1e-4);
    ((3.0 * sigma).ceil() as u32).clamp(1, MAX_HALF)
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
    // ⚠️ **A morfologia é da família do CAMPO e mede a IMAGEM** — as duas metades da frase importam.
    // Ela pergunta *a que distância da borda estou* como o contorno e o feather, mas a borda de que
    // ela fala é a do que ela RECEBEU, não a da silhueta autorada (`FxOp::measures_the_image`).
    // Logo o campo é semeado pela cobertura **sempre**, mesmo quando há geometria a oferecer o pé
    // exato — e o `n_segs` do uniform acompanha, senão o finalize responderia pela forma.
    if FxOp::measures_the_image(op.kind) {
        return Plan::Field {
            jumps: jump_count(op.grow_px.abs()),
            raster_seed: true,
        };
    }
    // ⚠️ **Quem decide é o MODO, não o ser-de-dentro** — mas a pergunta tem de ser feita ao TIPO.
    // A condição dizia `spec.inner && Contour` (verdade só enquanto os de dentro fossem os únicos
    // com modos), depois passou a `tem modos && escolheu o 1?`, e essa **apodreceu duas vezes**: o
    // `1` da turbulência é *Creased* (isento à mão, no early return acima) e o `1` do Gradient Map
    // é *Smooth* — que era varrido para cá e saía **no-op completo**, com o Linear correto ao lado.
    // `mode_selects_the_distance_plan` deriva da DECLARAÇÃO de modos do tipo, então um falloff novo
    // entra por construção e um vocabulário próprio nunca é varrido.
    let by_distance = matches!(op.kind, FxOp::OUTLINE | FxOp::FEATHER | FxOp::BEVEL)
        || (FxOp::mode_selects_the_distance_plan(op.kind) && op.mode == FxOp::MODE_CONTOUR);
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
    // **A morfologia alcança o que ela ENGORDA, e só na direção em que engorda.** Um `grow`
    // negativo AFINA: a silhueta anda para DENTRO, então ele não pede um texel de margem — pedir
    // seria textura comprada para desenhar o que já lá estava, e é a mesma resposta que os degraus
    // de dentro já dão pelo `!grows`. O `+1` do lado positivo cobre a rampa de anti-aliasing.
    if FxOp::measures_the_image(op.kind) {
        if op.grow_px <= 0.0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        return (op.grow_px.ceil() as u32 + 1).clamp(1, MAX_HALF);
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
