#![forbid(unsafe_code)]
//! `motion.noise` — a Motion **behaviour**: a coherent Perlin **gradient**-noise
//! FIELD that displaces a chosen channel, added to the existing value and scaled
//! per-instance by the multiplicative `falloff` column (§1.2; absent → `1.0`).
//! Reads the playhead but holds no state → `Effect::Temporal`.
//!
//! **Field, not jitter — the distinction from `motion.wiggle`.** Wiggle samples
//! `noise(time, instance_index)`: each element jitters on its own row, so they
//! move INDEPENDENTLY (nervous jitter). Noise samples `noise(position·scale,
//! time)`: neighbouring elements read nearby points of one continuous field, so
//! they flow TOGETHER — coherent turbulence (smoke, current, drift). And it is
//! **gradient** noise, not the **value** noise wiggle uses: gradient noise is
//! zero at every lattice point, so it has none of value noise's grid artifacts
//! (see [`noise`] and docs/Motion Nodes/07).
//!
//! Gold standard (doc 07): Improved Perlin 2002 (quintic fade, 8 isotropic
//! gradients) + fBm, transcendental-free (HR-5). Param surface is the
//! cross-tool intersection (Cavalry/AE/Houdini/Blender): scale, octaves,
//! roughness, type, speed, seed.
//!
//! `delta_i = fbm(P_i·scale, seed, octaves, roughness, type @ t·speed) ·
//! amplitude · falloff_i`, added to the chosen channel.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod kernel;
mod params_ui;
mod trig;
use kernel::GPU_KERNEL;
use params_ui::{PARAM_CHANNEL_RANGE, PARAM_GATES, PARAM_GROUPS, PARAM_HINTS, PARAM_UNITS};
mod noise;
use channel::{apply_channel_delta, clock_at, falloff_at, scalar_values};
use noise::{NoiseType, fbm};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// O tipo da porta `time` — espelho local do `VALUE` do `motion.drive`. Esta é uma
/// crate-folha: o vocabulário partilhado é a **porta**, nunca um símbolo importado.
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// A coluna que um stream de valor carrega (o que o `value.time` emite).
const VALUE_COL: &str = "v";

/// Hard ceiling on octaves — an untrusted `f32` param drives the fBm loop count.
/// 8 is past the point of visible return (each octave halves the feature size).
const MAX_OCTAVES: u32 = 8;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.noise"),
    name: "motion.noise",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // ⚠️ **A PORTA DE TEMPO, e ela é per-ELEMENTO** (folha 06, `SUPERAR 1`).
        // Desligada ⇒ `ctx.playhead()`, **byte-idêntico**; APENDADA, nunca inserida
        // (as arestas de um doc salvo guardam o ÍNDICE da porta).
        //
        // ⚠️ **Aqui ela é a resposta ao `stagger` que a folha pedia** (linha 23) —
        // e resposta MELHOR do que o knob: o `motion.noise` é o de CAMPO e o
        // `motion.wiggle` é o de ÍNDICE (cerca 5), então um `stagger` por-índice
        // *dentro* do noise apagaria a razão de os dois existirem. Uma porta não
        // apaga nada: o campo continua espacial, e quem quiser defasar o TEMPO por
        // índice liga um `value.time(stagger)` — a mesma peça que serve os irmãos.
        //
        // ⚠️ E o `loop_len` deste nó passa a fechar o ciclo **por elemento**: o wrap
        // (`ph2d_fbm::loop_times`) mudou-se para dentro do laço.
        PortSpec {
            name: "time",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead → pull-side; the noise is nonetheless deterministic.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // 0 X · 1 Y · 2 Rotation · 3 Size — the shared channel vocabulary.
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        // Peak of the displacement (channel-native units), before falloff.
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        // Spatial frequency: feature size of the field. World units are metres
        // (~single digits), so a smaller scale than a pixel tool's — 0.4 gives
        // features a couple of metres across.
        ParamSpec {
            name: "scale",
            default: 0.4,
        },
        // fBm octaves (= AE "Complexity" / Blender "Detail").
        ParamSpec {
            name: "octaves",
            default: 3.0,
        },
        // Per-octave amplitude falloff (= Houdini/Blender "Roughness", the
        // gain/persistence). 0.5 is the universal default.
        ParamSpec {
            name: "roughness",
            default: 0.5,
        },
        // 0 fBm · 1 Turbulence · 2 Ridged.
        ParamSpec {
            name: "type",
            default: 0.0,
        },
        // Temporal scroll speed (= AE "Evolution" / Cavalry "Time Scale"): the
        // field drifts through the elements over playhead-seconds.
        ParamSpec {
            name: "speed",
            default: 0.4,
        },
        // O comprimento do LOOP em segundos (`0` = nunca fecha, o mundo de sempre).
        ParamSpec {
            name: "loop_len",
            default: 0.0,
        },
        // Decorrelates several Noise nodes.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⚠️ **Apendado**, e o default é o valor que era const: `2.0` reproduz o
        // mundo de antes AO BIT (escalar por potência de dois não arredonda).
        ParamSpec {
            name: "lacunarity",
            default: 2.0,
        },
        // ── O ESPAÇO do campo (folha 06 linha 20) ────────────────────────────────
        //
        // ⚠️ **A célula pedia três eixos e a MEDIÇÃO (`measure_noise_space`) deixou
        // dois.** O *offset* já saía do sanduíche `motion.move(+d) → noise →
        // motion.move(−d)` (medido: a pose volta com `|Δx| = 0` e o campo desloca-se
        // `0,63`), e o *scale UNIFORME* **já era este `scale` aqui**: o sanduíche
        // `motion.transform(s) … (1/s)` é, **bit-a-bit**, `scale·s` com a amplitude
        // dividida por `s` (pior `|Δy|` entre as duas rotas: **0,000000**).
        //
        // ⚠️ **A rotação, essa, o sanduíche NÃO dá — e o número diz porquê.** Com
        // `motion.orbit(+90°) → noise → orbit(−90°)` o segundo nó roda o **DELTA** que
        // o ruído acabou de somar: o `y` sai **exactamente zero** e o deslocamento
        // inteiro vai parar ao X (`|Δx| = 0,436`). Não é um espaço rodado; é um
        // deslocamento rodado. *Uma translação comuta com «somar δ a um canal»; uma
        // rotação não.*
        //
        // O PIVÔ é a origem do mundo, de propósito: é a mesma factorização que o
        // `motion.transform` e o `motion.orbit` já escreveram um nível acima — o
        // centro vem do sanduíche de offset, a rotação vem daqui.
        ParamSpec {
            name: "rotation",
            default: 0.0,
        },
        // Escala NÃO-uniforme (AE *Fractal Noise* → Scale Width/Height). O trio é o
        // do `motion.scale` (`amount`/`uniform`/`amount_y`), que é o precedente vivo
        // deste módulo: com `uniform ≠ 0` o `scale_y` **não é lido**, e o `ParamGate`
        // nem o pinta.
        ParamSpec {
            name: "uniform",
            default: 1.0,
        },
        ParamSpec {
            name: "scale_y",
            default: 0.4,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct MotionNoise;

impl NodeOp for MotionNoise {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let amplitude = ctx.param("amplitude");
        let space = FieldSpace::of(
            ctx.param("scale"),
            ctx.param("scale_y"),
            ctx.param("uniform"),
            ctx.param("rotation"),
        );
        let octaves = (ctx.param("octaves").round().max(1.0) as u32).min(MAX_OCTAVES);
        let roughness = ctx.param("roughness");
        let ty = NoiseType::from_index(ctx.param("type"));
        let speed = ctx.param("speed");
        let seed = ctx.param("seed").round() as i32;
        let spec = ph2d_fbm::Spec {
            octaves,
            lacunarity: ctx.param("lacunarity"),
            roughness,
            ty,
        };
        // ⚠️ A costura do laço no tempo mudou-se para a folha `ph2d_fbm` — ela é a
        // terceira peça com UM dono e dois consumidores futuros (a família de
        // forças herda o `loop_period` no doc 89 folha 02). O raciocínio inteiro
        // (por que o tempo tem de WRAPAR primeiro, e por que o peso é smoothstep
        // e não linear) viajou com ela.
        //
        // ⚠️ **Ela é chamada DENTRO do laço agora**, porque o relógio pode ser um
        // campo: com a porta ligada cada elemento fecha o PRÓPRIO ciclo. Com ela
        // desligada os `n` cálculos partem do mesmo número e dão o mesmo resultado —
        // byte-idêntico ao que se calculava uma vez.
        let playhead = ctx.playhead() as f32;
        let loop_len = ctx.param("loop_len");
        let times = scalar_values(ctx.input(1), VALUE_COL);

        let out = {
            let input = ctx.input(0);
            let n = input.count();
            debug_assert!(
                matches!(times.len(), 0 | 1) || times.len() == n,
                "a porta `time` tem {} valores para {n} instancias",
                times.len()
            );
            // Each element's own world position is the sample point, so the field
            // is spatially coherent; the playhead scrolls it along Y (the field
            // "flows" through the elements).
            let pos = positions(input, n);
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    let (px, py) = pos[i];
                    let (t_a, t_b, w) =
                        ph2d_fbm::loop_times(clock_at(&times, i, playhead), loop_len);
                    // ⚠️ O espaço é transformado ANTES de o tempo entrar: o `speed`
                    // rola o campo pelo eixo Y **do próprio campo**, e rodá-lo junto
                    // com o espaço faria o `rotation` mudar a DIREÇÃO da rolagem — um
                    // knob a mexer no que o outro promete.
                    let (sx, sy) = space.at(px, py);
                    let sample = |tt: f32| fbm(sx, sy + tt * speed, seed, spec);
                    // `w == 0` é o caminho de sempre: a segunda amostra nem é avaliada.
                    let s = if w == 0.0 {
                        sample(t_a)
                    } else {
                        let a = sample(t_a);
                        a + (sample(t_b) - a) * w
                    };
                    s * amplitude * falloff_at(input, i)
                })
                .collect();
            apply_channel_delta(input, channel, &deltas)
        };
        ctx.emit(out);
    }
}

/// **O ESPAÇO do campo** — o ponto de amostragem, já escalado por eixo e rodado.
///
/// ⚠️ Resolvido UMA vez por dispatch e não por elemento: os quatro params são
/// uniformes, e o `(cos, sin)` é a única aritmética cara do nó. O corpo WGSL faz o
/// mesmo, e é isso que mantém os dois lados a pagar o mesmo preço.
struct FieldSpace {
    sx: f32,
    sy: f32,
    cos: f32,
    sin: f32,
}

impl FieldSpace {
    /// ⚠️ **`uniform ≠ 0` ignora o `scale_y`** — a lei do `motion.scale`
    /// (`amount`/`uniform`/`amount_y`), o precedente vivo deste módulo. Com o default
    /// (`uniform = 1`, `rotation = 0`) o `at` devolve `(px·scale, py·scale)`, que é
    /// **exactamente** a expressão que estava escrita aqui antes desta wave.
    fn of(scale: f32, scale_y: f32, uniform: f32, rotation_deg: f32) -> Self {
        let sy = if uniform != 0.0 { scale } else { scale_y };
        let (cos, sin) = trig::cos_sin_cycles(rotation_deg / DEG_PER_TURN);
        Self {
            sx: scale,
            sy,
            cos,
            sin,
        }
    }

    /// O ponto `(x, y)` do mundo, no espaço do campo.
    ///
    /// ⚠️ **RODA primeiro, escala depois — e a ordem contrária foi ESCRITA, DEFENDIDA
    /// num comentário e REPROVADA pelo olho do Enio.** Ela dizia *"o `scale_y` não
    /// pode esticar um eixo que a rotação já virou"*, e é falsa. A geometria:
    ///
    /// As feições do campo são a **pré-imagem** de manchas redondas. Com
    /// `M = R·S` (escala primeiro) elas são `S⁻¹R⁻¹(círculo)`, e os eixos dessa
    /// elipse são os de `S⁻¹` — **os eixos do MUNDO**. Ou seja: com o campo
    /// anisotrópico, a rotação **não gira as faixas**; ela só troca *qual* pedaço de
    /// ruído se vê. Medido na cena `=60`: a banda rodada-e-esticada saía com faixas
    /// horizontais idênticas às da só-esticada (variação `→ 0,0077` contra
    /// `↓ 0,0218`). *Um knob que não move o que promete é um knob que mente.*
    ///
    /// Com `M = S·R`, `M⁻¹ = R⁻¹S⁻¹` e os eixos da elipse saem girados de `−θ` — as
    /// faixas viram, que é o que «rodar o campo» quer dizer.
    ///
    /// ⚠️ **E as outras três bandas não mudam:** com escala uniforme `S = s·I`
    /// **comuta** com `R`, então o caso isotrópico é o mesmo nas duas ordens; e sem
    /// rotação não há o que ordenar.
    ///
    /// ⚠️ **Quem guarda esta ordem são DOIS gates, e eles medem coisas diferentes:**
    /// `the_fourth_band_runs_its_stripes_on_the_diagonal` (a direção das faixas — o
    /// que o olho lê) e `the_noise_field_space_matches_the_cpu_on_the_device` (que as
    /// duas cópias da lei concordam). O segundo sozinho deixou a ordem errada passar,
    /// porque *paridade prova que os dois lados fazem o mesmo, nunca que o mesmo é
    /// certo*.
    fn at(&self, px: f32, py: f32) -> (f32, f32) {
        let (x, y) = (px * self.cos - py * self.sin, px * self.sin + py * self.cos);
        (x * self.sx, y * self.sy)
    }
}

/// Graus por volta — a unidade autorada da casa, convertida para ciclos na borda
/// (`deg / 360`, uma divisão IEEE exacta; HR-5).
const DEG_PER_TURN: f32 = 360.0;

/// Each element's `P` (absent → origin), the field's sample points.
fn positions(input: &Stream, n: usize) -> Vec<(f32, f32)> {
    match input.get("P") {
        Some(Column::Vec2(v)) => {
            let mut out: Vec<(f32, f32)> = v.iter().map(|p| (p[0], p[1])).collect();
            out.resize(n, (0.0, 0.0));
            out
        }
        _ => vec![(0.0, 0.0); n],
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionNoise))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Noise",
            // Transform blue: a spatial behaviour that moves elements.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
