//! **O que um degrau JÁ RESOLVIDO é** — o tipo que a shell entrega ao passe.
//!
//! Irmão de [`super::fx_stack`] pelo teto de LOC, e o corte é por responsabilidade: aquele arquivo
//! é *o FOLD* (que passes correm, em que ordem, com que bind groups) e este é *o que um degrau É
//! depois de a câmara ter feito a conta dela*. É o mesmo corte que já separou o `fx_stack_res`
//! (o que o passe ALOCA) e o `fx_stack_plan` (quanto ele PERCORRE).

/// **Um degrau da pilha, já resolvido em PIXELS DE TELA.**
///
/// A conversão mundo→pixel é [`FxOpGpu::resolve`], a porta ÚNICA dos dois autores (o Vector e a
/// camada do Motion): o passe não sabe o que é uma câmera, só a parte LINEAR dela.
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
    pub offset_px: [f32; 2],
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
    /// **Os STOPS da rampa** (Gradient Map) — RGBA reto em `[0,1]`, o alfa é a FORÇA do stop.
    /// ⚠️ Chegam **ORDENADOS por posição**: quem ordena é o host, uma vez, e não o shader por texel.
    pub stops: [[f32; 4]; 8],
    /// As posições dos stops em `[0,1]`, empacotadas em dois `vec4` (o stride de uniform do WGSL).
    pub stop_pos: [[f32; 4]; 2],
    /// Quantos stops valem (`0` = a rampa preto→branco, a identidade em luma).
    pub stop_count: u32,
}

impl FxOpGpu {
    /// **O degrau AUTORADO resolvido em pixels de tela.** `linear = [a, b, c, d]` é a parte linear
    /// da transformação mundo → pixel (a do Vector é a vista do Vello; a do Motion inverte o Y).
    ///
    /// Comprimentos (raio, tamanho do ruído, crescimento) atravessam a ESCALA média da câmera; o
    /// deslocamento atravessa a matriz SEM arredondar — o borrão amostra a gaussiana deslocada e o
    /// finalize interpola o campo (doc 129 §5.5 A7: a luz do bevel saltava ao dar zoom-out). ⚠️ **Cores, matiz e frações NÃO cruzam a câmera** — dar zoom não muda a cor.
    /// O `blend_code`/`detail_clamped` são a metade de HONRAR das portas do `FxOp`.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "pixels de tela cabem num i32/f32"
    )]
    pub fn resolve(o: &ph2d_fx_op::FxOp, linear: [f64; 4]) -> Self {
        let [a, b, c, d] = linear;
        let cam_scale = ((a * a + b * b).sqrt() + (c * c + d * d).sqrt()) as f32 * 0.5;
        let (ox, oy) = (f64::from(o.offset[0]), f64::from(o.offset[1]));
        let (stops, stop_pos, stop_count) = o.ramp_for_device();
        Self {
            kind: o.kind,
            sigma_px: (o.radius * cam_scale).max(0.0),
            offset_px: [(a * ox + c * oy) as f32, (b * ox + d * oy) as f32],
            tint: o.color,
            tint_b: o.color_b,
            opacity: o.opacity,
            mode: o.mode,
            blend: o.blend_code(),
            stops,
            stop_pos,
            stop_count,
            noise_scale_px: (o.scale * cam_scale).max(0.0),
            detail: o.detail_clamped(),
            seed: o.seed,
            // ⚠️ Sem `max(0.0)`: aqui o sinal É a operação.
            grow_px: o.grow * cam_scale,
            hue: o.hue,
            sat: o.sat,
            bright: o.bright,
        }
    }
}
