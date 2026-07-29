//! **O que um degrau JÁ RESOLVIDO é** — o tipo que a shell entrega ao passe.
//!
//! Irmão de [`super::fx_stack`] pelo teto de LOC, e o corte é por responsabilidade: aquele arquivo
//! é *o FOLD* (que passes correm, em que ordem, com que bind groups) e este é *o que um degrau É
//! depois de a câmara ter feito a conta dela*. É o mesmo corte que já separou o `fx_stack_res`
//! (o que o passe ALOCA) e o `fx_stack_plan` (quanto ele PERCORRE).

/// **Um degrau da pilha, já resolvido em PIXELS DE TELA.**
///
/// A conversão mundo→pixel (o zoom da câmera) é da shell: este passe não sabe o que é uma câmera,
/// e um segundo lugar a fazer a conta seria um segundo lugar a errá-la.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FxOpGpu {
    /// `0` Blur · `1` Glow · `2` Drop Shadow (os códigos do `ph2d_ecs::FxOp`).
    pub kind: u8,
    /// O desvio do gaussiano, em pixels de tela.
    pub sigma_px: f32,
    /// O deslocamento do halo, em pixels de tela INTEIROS.
    ///
    /// ⚠️ **Inteiros de propósito.** O halo é amostrado por `textureLoad` (sem sampler), então um
    /// deslocamento fracionário custaria interpolação dentro do laço do borrão. Uma sombra não
    /// precisa de posição sub-pixel — e a textura inteira já é alinhada ao pixel da tela.
    pub offset_px: [i32; 2],
    /// A cor RETA do halo, `[0,1]`.
    pub tint: [f32; 4],
    /// A SEGUNDA cor RETA — a ponta CLARA da rampa do Duotone (a [`Self::tint`] é a escura). Só ele
    /// a lê; nos outros tipos ela é inerte.
    pub tint_b: [f32; 4],
    /// A intensidade deste degrau, `[0,1]`.
    pub opacity: f32,
    /// O MODO (o índice em `FxKindSpec::modes`). Só os degraus de DENTRO o leem hoje.
    pub mode: u8,
    /// **A LEI DE MISTURA** — o código de `ph2d_painter_effects::BlendMode`, `0` = Normal.
    ///
    /// ⚠️ **Um `u8` cru, sem acoplamento de enum** — o mesmo desenho que o `LayerCompositor` já
    /// ship (a `ph2d-painter-effects` é dev-dep desta crate, nunca dependência de produção). Quem
    /// decide se o número é honrado é o `FxOp::blend_code` do lado do produtor: aqui ele já chega
    /// resolvido.
    pub blend: u8,
    /// O TAMANHO das ondulações do ruído, em pixels de tela (`escala_mundo × zoom`, como o
    /// `sigma_px`).
    pub noise_scale_px: f32,
    /// Quantas OITAVAS o ruído soma — já clampado pelo produtor (`FxOp::detail_clamped`).
    pub detail: u8,
    /// Qual realização do ruído.
    pub seed: u8,
    /// **Quanto a silhueta engorda, em pixels de tela — COM SINAL** (positivo cresce, negativo
    /// afina, zero não faz nada). Só a morfologia o lê.
    pub grow_px: f32,
    /// **A MATIZ**, em VOLTAS — a unidade do modelo (`HsbParams::h`), não graus. Só o Color
    /// Adjust o lê.
    pub hue: f32,
    /// **A SATURAÇÃO**, `-1..1` (escala do croma em OKLab).
    pub sat: f32,
    /// **O BRILHO**, `-1..1` (lerp para preto/branco em luz linear).
    pub bright: f32,
}
