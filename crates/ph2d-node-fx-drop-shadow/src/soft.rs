//! **A MACIEZ da sombra** (doc 89, folha 11) — o disco de taps que substitui o
//! único fantasma quando o artista pede uma penumbra.
//!
//! ## A cerca C1, e o que dela sobrevive
//!
//! O doc-header deste nó dizia, desde 2026-07-12: *"What is deliberately NOT here:
//! blur … rather than a fake softness built from a stack of ghosts."* A cerca tinha
//! **duas** razões e elas envelheceram de formas diferentes:
//!
//! 1. **«O borrão é raster e pertence ao compositor»** — continua VERDADE para a
//!    rota de passe, e a folha 11 §0 diz porquê com o mecanismo: o passe do Motion
//!    compõe **aditivamente** (`One`/`One`), e **um halo escuro não pode ser
//!    somado**. Um borrão de verdade exigiria um passe ANTES do de sprites, que é
//!    decisão de renderer. ⚠️ **Nada aqui toca nisso** — estes taps são instâncias
//!    comuns, desenhadas atrás, com o mesmo alpha-blend que a sombra dura já usa.
//! 2. **«Maciez falsa a partir de uma PILHA de fantasmas»** — esta era sobre
//!    ENCADEAR o nó (`drop_shadow → drop_shadow`), e a própria célula mediu porque
//!    aquilo é ruim: dá um *smear* ao longo de UMA direção, não alarga
//!    perpendicular, e o alfa **compõe multiplicativamente** (`0,35² = 0,1225` na
//!    2ª ordem). ⚠️ **Um disco de UM passe não tem nenhum desses três defeitos**, e
//!    a diferença é medível — é o que esta folha registra.
//!
//! ## O que isto É, dito com o número
//!
//! Uma sombra macia é a sombra dura **convolvida** com um disco. Aqui ela é
//! amostrada em [`TAPS`] pontos e a união das coberturas faz a penumbra. Não é uma
//! gaussiana; é uma aproximação por dispersão, que é como um motor 2D sem passe
//! próprio a faz. O erro tem forma conhecida: **bandas** quando o raio cresce muito
//! para o número de taps.
//!
//! ⚠️ **A DENSIDADE do miolo é preservada, e não por acaso.** Sobrepor `N` cópias
//! de alfa `a` dá `1 − (1−a)^N`, então para o interior continuar a valer o mesmo
//! `A` da sombra dura é preciso `a = 1 − (1−A)^(1/N)`. Um `A/N` ingénuo daria
//! `1 − e^(−A)` e a sombra **escureceria menos** ao ligar a maciez — ligar um knob
//! não pode mudar a densidade, só a borda.
//!
//! ⚠️ **É por isso que [`TAPS`] é 16 e não 12**: `x^(1/16)` são quatro `sqrt`
//! encadeados, e o `sqrt` do IEEE-754 é **correctamente arredondado** — a raiz sai
//! determinística em toda plataforma. Um `powf(1.0/12.0)` seria transcendental
//! (HR-5) e daria um número diferente por libm.

use super::trig;

/// Quantos taps compõem o disco. ⚠️ **Uma potência de dois de propósito** — ver o
/// cabeçalho: a densidade pede a raiz `N`-ésima, e só assim ela é `sqrt` puro.
///
/// ⚠️ **O recurso é a CONTAGEM DE INSTÂNCIAS**, e o número está do lado do teto: com
/// a maciez ligada cada elemento vira `TAPS + 1` linhas, então o `MAX_INSTANCES` de
/// `262_144` corta em **n ≤ 15 420** (contra `131 072` com a sombra dura). É o mesmo
/// portão que já existia, a contar o número certo.
pub(crate) const TAPS: usize = 16;

/// O ângulo de ouro em CICLOS (`137,5077° / 360`) — o passo do disco de Vogel, a
/// distribuição que enche um disco sem anéis nem eixos preferidos. Ela é o que
/// impede o artefacto que um anel regular de 16 pontos produziria: uma penumbra com
/// **raios** visíveis.
const GOLDEN_TURN: f32 = 0.381_966_02;

/// Os `TAPS` deslocamentos do disco de raio `r`, em unidades de mundo.
///
/// Vogel: `rₖ = r·√((k+½)/N)`, `θₖ = k·φ`. A raiz é o que dá densidade UNIFORME por
/// área — sem ela os taps amontoam-se no centro e a penumbra fica dura por dentro e
/// rala por fora.
pub(crate) fn disc(r: f32) -> [[f32; 2]; TAPS] {
    let mut out = [[0.0f32; 2]; TAPS];
    for (k, o) in out.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "k < 16")]
        let t = (k as f32 + 0.5) / TAPS as f32;
        let rad = r * t.sqrt();
        #[expect(clippy::cast_precision_loss, reason = "k < 16")]
        let (c, s) = trig::cos_sin_cycles(k as f32 * GOLDEN_TURN);
        *o = [rad * c, rad * s];
    }
    out
}

/// O alfa que cada tap leva para a UNIÃO dos [`TAPS`] valer `a` — a inversa exacta
/// de `1 − (1−x)^N`, por quatro `sqrt` (ver o cabeçalho).
///
/// ⚠️ **Entrada fora de `[0,1]` devolve a própria entrada.** Um alfa negativo ou
/// maior que 1 não tem raiz real útil aqui, e o chamador já trata o caso morto; o
/// que esta guarda impede é um `NaN` a viajar para a coluna `tint`.
pub(crate) fn per_tap_alpha(a: f32) -> f32 {
    if !(0.0..=1.0).contains(&a) {
        return a;
    }
    // (1−a)^(1/16) = sqrt(sqrt(sqrt(sqrt(1−a))))
    let mut x = 1.0 - a;
    for _ in 0..4 {
        x = x.sqrt();
    }
    1.0 - x
}

#[cfg(test)]
#[path = "soft_tests.rs"]
mod tests;
