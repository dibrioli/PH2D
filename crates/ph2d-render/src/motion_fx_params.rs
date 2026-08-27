//! **O QUE O ARTISTA AUTORA NO HALO, E O QUE CADA NÚMERO SIGNIFICA** — os params do bloom e as
//! derivações que eles alimentam (a curva do joelho, a base da tenda, o teto do bright-pass, a
//! tag da operação, a bandeira da fonte).
//!
//! ⚠️ **O corte foi FORÇADO pelo HR-18** (o `motion_fx.rs` passou de 700 ao ganhar a rampa,
//! doc 89 folha 11) **e a costura é por responsabilidade**: isto responde *o que o artista pediu
//! e o que aquilo quer dizer*, e o irmão responde *que passes o desenham*. As duas crescem por
//! razões diferentes — esta quando um knob entra, aquela quando um passe entra.

use super::trig;

/// The three glow knobs the document carries (doc 67). Plain data — the `fx.glow`
/// node authors it, the shell hands it to [`bloom_over`](MotionFx::bloom_over).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BloomParams {
    /// Brightness above which a pixel starts to glow (premult `max(r,g,b)`).
    /// `1.0` = only genuinely HDR (emissive) pixels bloom.
    pub threshold: f32,
    /// Soft-knee width around the threshold — the glow ramps in over
    /// `[threshold-knee, threshold]` instead of switching on hard.
    pub knee: f32,
    /// Multiplier on the accumulated glow before it is added to the scene.
    pub intensity: f32,
    /// Scales the upsample tent radius — a wider radius spreads the halo further.
    pub radius: f32,
    /// `0` pulls the glow to grey (a white bloom), `1` keeps the source colour.
    pub saturation: f32,
    /// Multiplies the (desaturated) glow — default white `[1,1,1,1]` is a no-op.
    pub tint: [f32; 4],
    /// **A ANAMORFOSE** — a razão entre o alcance do halo ao longo de
    /// [`Self::angle`] e o alcance perpendicular a ele (doc 89 folha 11). `1` é o
    /// halo redondo que sempre shipou; `>1` estica na direção do ângulo e aperta na
    /// outra, que é o *streak* anamórfico do cinema (o `Glow Dimensions H/V` do AE,
    /// o *Anamorphic Ratio* do Unity, a *Bloom Convolution* do Unreal).
    ///
    /// ⚠️ **Ela mora na TENDA do upsample, não na cadeia de mips.** Os mips são a
    /// máquina que arredonda as quinas da fonte (é literalmente o que os torna
    /// melhores que um box blur largo); torcê-los tornaria a queda direcional em
    /// TODAS as escalas e o halo perderia o miolo. A tenda de 9 taps é onde a
    /// referência põe a razão, e é o passe que corre uma vez por nível.
    pub stretch: f32,
    /// A direção do *streak*, em GRAUS — a unidade autorada única do app. Sem
    /// efeito em [`Self::stretch`] `= 1` (um círculo rodado é o mesmo círculo), e o
    /// `ParamGate` do nó esconde-a ali.
    pub angle: f32,
    /// **O TETO do bright-pass** — o antídoto dos *fireflies* (o `Clamp` do Bloom do
    /// Unity URP). `0` = **desligado**, o caminho literal que sempre shipou.
    ///
    /// ⚠️ **O recurso é a REPRESENTAÇÃO, e o número está medido.** O `tint` de uma
    /// instância é `[f32; 4]` **sem clamp** (doc 67 §4), e o bright-pass não limita
    /// nada: para `brightness → ∞` a contribuição tende a `1` e a saída tende ao
    /// próprio `c`. Então um único elemento com `tint = 5000` entra inteiro na
    /// cadeia, espalha-se por seis níveis de mip e lava a tela. O único teto que
    /// existe hoje é o do FORMATO — `Rgba16Float` guarda até **65 504** e depois é
    /// `inf`, que envenena a soma de toda a cadeia. Este param é o teto AUTORADO,
    /// que é o que a referência expõe.
    pub clamp: f32,
    /// **A OPERAÇÃO do halo** (doc 89 folha 11, o *Glow Operation* do AE): `0` = `Add`, o passe
    /// aditivo que sempre shipou; `1` = `Screen`.
    ///
    /// ⚠️ **`Multiply` não existe aqui, e é uma decisão medida pela navalha do §0**: o halo
    /// compõe-se sobre a cena já desenhada, sem profundidade, então um modo que ESCUREÇA
    /// pintaria por cima do que estivesse à frente. `Screen` (`a + b − ab`) é monótono e nunca
    /// escurece — é o único dos três do AE que sobrevive a essa navalha.
    pub operation: f32,
    /// **De que o bright-pass se alimenta** (o *Glow Based On* do AE): `0` = a luminância
    /// premultiplicada (o de sempre), `1` = o **alfa**.
    ///
    /// ⚠️ Com luma, uma silhueta **preta e opaca** não tem nada acima do limiar e nunca acende;
    /// com alfa ela acende pela COBERTURA. É a diferença entre um halo de EMISSÃO e uma AURA.
    pub source: f32,
    /// **QUANTO a máscara de sujidade acende** (doc 89 folha 11) — o `Dirt Intensity` do Unity
    /// URP / o `Bloom Dirt Mask Intensity` do Unreal. `0` = o passe de sempre.
    ///
    /// ⚠️ **Ele soma-se ao `tint`, não multiplica o halo** — `glow · (tint + dirt·isto)`, a forma
    /// da referência. A distinção importa: uma máscara que multiplicasse não poderia ACRESCENTAR
    /// cor, e um mapa de sujidade é uma fotografia colorida de pó e riscos.
    ///
    /// ⚠️ **A identidade do quadro NÃO vive neste número.** Sem imagem escolhida o binding leva
    /// uma textura preta de 1×1, então `dirt = 0` e a soma é literal — inclusive com este knob
    /// alto, que é o estado em que um artista fica ao apagar o nome da imagem.
    pub dirt_intensity: f32,
}

/// Quantas operações de composição existem — o tamanho do array de pipelines.
///
/// ⚠️ **É esta contagem que a lista de rótulos do nó tem de ter** (`OPERATION_LABELS`), e quem
/// liga as duas pontas é um gate na shell: um nó é uma folha e não alcança o `ph2d-render`,
/// então sem ele um modo a mais no dropdown seria escolhível e silenciosamente rebaixado.
pub const COMPOSITE_OPERATIONS: usize = 2;

impl Default for BloomParams {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            knee: 0.6,
            intensity: 0.8,
            radius: 1.0,
            saturation: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            stretch: 1.0,
            angle: 0.0,
            clamp: 0.0,
            operation: 0.0,
            source: 0.0,
            dirt_intensity: 0.0,
        }
    }
}

impl BloomParams {
    /// O índice do pipeline de composição — grampeado à lista que de facto existe.
    ///
    /// ⚠️ **Grampeado, e não `unwrap`**: o número vem de um param que um documento carregado ou
    /// uma edição por MCP pode ter posto fora da faixa, e escolher um pipeline que não existe
    /// seria um `panic` no meio do quadro. Fora da faixa cai no `Add`, que é o neutro.
    #[must_use]
    pub(super) fn operation_tag(&self) -> usize {
        if self.operation.is_finite() && self.operation >= 0.5 {
            1
        } else {
            0
        }
    }

    /// `1` quando o bright-pass lê o ALFA, `0` quando lê a luminância — o número que o shader
    /// compara com `0,5`. Um valor lixo conta como a luminância, o caminho de sempre.
    #[must_use]
    pub(super) fn source_flag(&self) -> f32 {
        if self.source.is_finite() && self.source >= 0.5 {
            1.0
        } else {
            0.0
        }
    }

    /// Pack the soft-knee curve the prefilter shader expects:
    /// `(threshold, threshold-knee, 2·knee, 0.25/knee)` (COD/Karis).
    pub(super) fn prefilter_curve(&self) -> [f32; 4] {
        let knee = self.knee.max(1e-4);
        [
            self.threshold,
            self.threshold - knee,
            2.0 * knee,
            0.25 / knee,
        ]
    }
}

impl BloomParams {
    /// **A BASE 2×2 da tenda do upsample**, em UV: `[du.x, du.y, dv.x, dv.y]`.
    ///
    /// Os 9 taps deixam de ser `(±x, ±y)` e passam a ser `(±du ±dv)`. No neutro
    /// (`stretch = 1`) o caminho é **LITERAL** e devolve `[fr, 0, 0, fr·aspect]`,
    /// que reconstrói tap a tap os offsets de sempre — `uv + (−du + dv)` é
    /// `uv + (−fr, fr·aspect)`, a mesma soma, ao bit.
    ///
    /// ⚠️ **A anisotropia é calculada em PIXELS e convertida no fim.** O `aspect`
    /// existe para o halo sair redondo na tela; aplicá-lo antes da rotação faria o
    /// ângulo significar coisas diferentes em janelas diferentes — o mesmo `45°`
    /// apontaria para outro sítio ao redimensionar.
    pub(super) fn upsample_basis(&self, aspect: f32) -> [f32; 4] {
        let fr = BASE_FILTER_RADIUS * self.radius.max(0.0);
        let s = self.stretch.max(MIN_STRETCH);
        if s == 1.0 {
            return [fr, 0.0, 0.0, fr * aspect];
        }
        let (c, sn) = trig::cos_sin_cycles(self.angle / 360.0);
        // Ao longo do ângulo alarga por `s`; perpendicular aperta por `1/s`, para o
        // «raio» continuar a ser a média geométrica dos dois e o knob não mudar a
        // ENERGIA do halo, só a forma dele.
        let (ax, ay) = (c * fr * s, sn * fr * s);
        let (bx, by) = (-sn * fr / s, c * fr / s);
        [ax, ay * aspect, bx, by * aspect]
    }

    /// O teto do bright-pass, como o shader o quer: `0` (desligado) vira o maior
    /// finito do `Rgba16Float`, que é o teto que a REPRESENTAÇÃO já impunha — então
    /// o `min` do shader é um no-op sobre qualquer valor que o RT consiga guardar.
    pub(super) fn clamp_limit(&self) -> f32 {
        if self.clamp > 0.0 {
            self.clamp
        } else {
            F16_MAX
        }
    }
}

/// Upsample tent radius in UV at `radius = 1` (the mip chain does the heavy
/// spreading; this is the per-level tent overlap). Scaled by `BloomParams::radius`.
pub(super) const BASE_FILTER_RADIUS: f32 = 0.006;
/// Piso da anamorfose: abaixo disto o eixo estreito colapsa e a tenda deixa de
/// cobrir o próprio texel (o `1/s` explodiria o outro eixo).
pub(super) const MIN_STRETCH: f32 = 0.05;
/// O maior finito representável em `Rgba16Float` — o teto que o formato do RT já
/// impõe, e o valor com que o clamp desligado passa pelo `min` sem morder.
pub(super) const F16_MAX: f32 = 65_504.0;
