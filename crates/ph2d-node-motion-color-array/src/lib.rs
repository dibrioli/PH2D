#![forbid(unsafe_code)]
//! `motion.color_array` — **cycle a palette across instances by index**: the Cinema 4D
//! MoGraph "Color" / a palette cycler (Motion Nodes M1, colour — doc 01 §1.7 / doc 29).
//! Where `motion.color_ramp` is the *continuous* colour node (a gradient), this is the
//! *discrete* one: element `i` gets palette slot `i mod colours`, writing the `tint`
//! column — the classic hard-striped clone colouring.
//!
//! **Algorithm — a modular palette lookup.** Element `i` takes palette slot
//! `(i + offset) mod colours`, where `colours` is the authored list's own length. An
//! `offset` **value** input shifts which slot each index gets, so a `value.lfo` marches
//! the palette across the set. Transcendental-free (HR-5): a modulo lookup, no maths.
//! `Effect::Pure`.
//!
//! ## O `offset` é um CAMPO, não um escalar (doc 89 folha 09)
//!
//! A porta lê a escada `0/1/n` que o resto da família de valor já fala (o
//! `motion.look_at::target_at`, o `t` do `motion.color_ramp`): **ausente** → 0;
//! **comprimento 1** → um deslocamento global, difundido a todas as peças;
//! **comprimento n** → um por elemento. Até esta wave o nó lia `.first()`, ou
//! seja: um campo por-instância era **silenciosamente DESCARTADO** e o índice do
//! elemento 0 pintava o conjunto inteiro — o *Color Array indexável pelo clone*
//! do Cavalry era inexprimível. Os dois primeiros degraus são byte-idênticos ao
//! que shipava.
//!
//! ## E ele cozinha no DEVICE (a terceira célula da folha)
//!
//! A paleta é uma lista de comprimento variável, que o uniforme `KernelParams`
//! não pode carregar — foi por isso que este era o único dos quatro nós de cor
//! sem kernel, e um grafo que o usasse perdia a aceleração inteira. A cura é o
//! canal de **LUT** que o `value.pattern` estreou: a lista viaja num `storage` de
//! floats com a CONTAGEM no slot 0, e o corpo lê o buffer **direto**, nunca o
//! `_sample(t)` que o gerador também emite — aquele LERPA entre vizinhos, que é
//! certo para uma rampa e errado para uma lista, cujas cores são autoradas uma a
//! uma e têm de sair **exactas**. Ver [`palette`].

mod palette;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `offset` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";
/// The text-param key carrying the palette — read by [`EvalCtx::text_param`] on the
/// CPU and by the LUT fill on the device, so the two share one name.
const PALETTE_KEY: &str = "palette";

/// The palette an untouched node paints with — red / green / blue / yellow, the four
/// the fixed params used to default to, so a graph that never authored a palette looks
/// exactly like it did before this wave.
///
/// ⚠️ **This is a DEFAULT, not a cap** — the length lives in the text param, and
/// `cycle` reads `palette.len()`.
///
/// ⚠️ **Há um teto, e ele não é este: é [`palette::MAX_COLORS`]**, o do BUFFER do
/// device (medido, com a tabela ao lado da const). Esta linha dizia *"não há
/// máximo"* e teria envelhecido calada no dia em que o kernel entrou; quem move
/// o número que tornava uma nota verdadeira tem de reconferir a nota (§0).
pub use ph2d_color::DEFAULT_PALETTE_FALLBACK as DEFAULT_PALETTE;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.color_array"),
    name: "motion.color_array",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Shifts which slot each index gets (animatable). Optional: unconnected → 0.
        PortSpec {
            name: "offset",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // ⚠️ **NO f32 params, and that IS the wave** (Enio: *"color array poderia ter
    // quantas cores o usuário quisesse, tire os limites"*). The palette used to be four
    // colours because four is how many `ParamSpec`s were written down, and the `colors`
    // count existed to shrink that fixed list. Both were limits of the REPRESENTATION.
    //
    // The palette now travels as the text param `palette` (`ph2d_color::palette_text`),
    // the canonical channel for a param that is not one f32 — the same road
    // `motion.color_ramp` took for its gradient, and for the same reason: a
    // variable-length list of colours cannot be a fixed set of `f32` params.
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
///
/// ⚠️ The tenth-and-something copy of this helper in the node library — see the note on
/// `motion.color_ramp::falloff_at`. The extraction is a wave of its own across nine
/// crates; this follows the convention the library already chose.
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The instance's existing colour (absent → opaque white).
fn existing_tint(stream: &Stream, i: usize) -> [f32; 4] {
    match stream.get("tint") {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or([1.0; 4]),
        _ => [1.0; 4],
    }
}

/// `lerp(existing, target, f)` per RGBA channel, endpoint-EXACT: at `f = 1` the first
/// term is `existing · 0.0` and the second `target · 1.0`, so an absent mask writes the
/// palette slot **bit for bit** — the substitution this node has always performed.
fn mixed_tint(existing: [f32; 4], target: [f32; 4], f: f32) -> [f32; 4] {
    let lerp = |e: f32, t: f32| e * (1.0 - f) + t * f;
    [
        lerp(existing[0], target[0]),
        lerp(existing[1], target[1]),
        lerp(existing[2], target[2]),
        lerp(existing[3], target[3]),
    ]
}

/// O deslocamento do elemento `i` — a escada `0/1/n` (ver os docs do módulo).
///
/// ⚠️ **Arredondado meio-para-longe-do-zero** (`f32::round`), que é o que o
/// `ca_round` do WGSL replica: o `round` nativo do device é meio-para-o-PAR, e
/// num `offset = 0.5` vindo de um `value.lfo` os dois caminhos escolheriam cores
/// diferentes.
fn offset_at(offsets: &[f32], i: usize) -> i64 {
    let v = match offsets.len() {
        0 => return 0,
        // Difusão: um deslocamento global marcha a paleta inteira.
        1 => offsets[0],
        _ => offsets.get(i).copied().unwrap_or(0.0),
    };
    v.round() as i64
}

/// Assign palette slot `(i + offset) mod colours` to each of `n` elements, masked by
/// `falloff` — the same law `motion.color_ramp` and `motion.tint` apply, so a `field.*`
/// reaches the DISCRETE colour node and the continuous one identically (doc 89 fam. 9).
///
/// ⚠️ **O comprimento do ciclo é `palette.len()`, e não há um segundo número a
/// dizê-lo.** O `colors` que existia aqui era sempre igual a `palette.len()` no
/// único sítio que chamava esta função — e o device NÃO tem por onde receber um
/// terceiro valor (a LUT carrega uma contagem só). Um param que só um dos dois
/// caminhos conhecesse é a divergência à espera de acontecer; *uma representação
/// para um facto*.
fn cycle(n: usize, palette: &[[f32; 4]], offsets: &[f32], input: &Stream) -> Vec<[f32; 4]> {
    // Garantido por `palette::palette_of`, que nunca devolve a lista vazia.
    debug_assert!(!palette.is_empty(), "a paleta é não-vazia por contrato");
    let colors = palette.len() as i64;
    (0..n)
        .map(|i| {
            let idx = (i as i64 + offset_at(offsets, i)).rem_euclid(colors) as usize;
            mixed_tint(existing_tint(input, i), palette[idx], falloff_at(input, i))
        })
        .collect()
}

/// Uma coluna `Scalar` inteira (ausente / mal-tipada → a lista vazia, que é o
/// degrau "nada ligado" da escada).
fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// GPU compute kernel (ADR-0126 side channel) — o porte WGSL de [`cycle`],
/// elemento a elemento.
///
/// **A paleta viaja pela LUT, e o corpo lê o BUFFER direto.** O layout é
/// `[len, r0,g0,b0,a0, …]` ([`palette::fill_lut`]); `lut_ca_pal[0]` é o
/// comprimento do ciclo, que é a única coisa que o device precisa de saber e que
/// um `arrayLength` (a CAPACIDADE) não diz. ⛔ Não troque isto por
/// `ca_pal_sample(t)`: aquele acessor interpola entre vizinhos, e duas cores
/// adjacentes de uma paleta não têm nada entre si — o índice `k` sairia misturado
/// com o `k±1` sem ninguém pedir (a lição que o `value.pattern` deixou escrita).
///
/// **O `offset` monta o leitor de difusão** ([`ColumnAccess::ReadBroadcast`],
/// ADR-0136): ele pareia um campo por-elemento posicionalmente e prende um campo
/// de comprimento 1 à linha 0 — a escada `0/1/n` da CPU, byte a byte — e RECUSA
/// no cook um campo de qualquer outro comprimento (`BroadcastLengthMismatch`),
/// em vez de pintar metade do conjunto com a cor errada.
///
/// **HR-5:** `ca_round` é o `f32::round` do Rust (meio para longe do zero), nunca
/// o `round` do WGSL (meio para o par); o resto é `%` inteiro.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        // O cabecalho: quantas cores a paleta autorada carrega. Nunca 0 (a\n\
        // `palette_of` cai na de fabrica), mas o `max` mantem o modulo seguro\n\
        // se alguem encolher a `resolution` a zero.\n\
        let ca_n = max(i32(lut_ca_pal[0]), 1);\n\
        // O deslocamento: ausente -> 0; ligado -> o campo (ou a linha 0 difundida).\n\
        var ca_off = 0;\n\
        if (HAS_offset_v) { ca_off = i32(ca_round(read_offset_v(i))); }\n\
        var ca_k = (i32(i) + ca_off) % ca_n;\n\
        if (ca_k < 0) { ca_k = ca_k + ca_n; }\n\
        let ca_b = 1u + 4u * u32(ca_k);\n\
        let ca_t = vec4<f32>(\n\
        \x20   lut_ca_pal[ca_b], lut_ca_pal[ca_b + 1u],\n\
        \x20   lut_ca_pal[ca_b + 2u], lut_ca_pal[ca_b + 3u]);\n\
        // A mascara: `lerp(existing, slot, falloff)` na forma exacta nos extremos,\n\
        // logo um `falloff` ausente (identidade 1.0) escreve a cor bit a bit.\n\
        let ca_e = read_in_tint(i);\n\
        let ca_f = read_in_falloff(i);\n\
        write_tint(i, vec4<f32>(\n\
        \x20   ca_e.x * (1.0 - ca_f) + ca_t.x * ca_f,\n\
        \x20   ca_e.y * (1.0 - ca_f) + ca_t.y * ca_f,\n\
        \x20   ca_e.z * (1.0 - ca_f) + ca_t.z * ca_f,\n\
        \x20   ca_e.w * (1.0 - ca_f) + ca_t.w * ca_f));\n",
    wgsl_lib: "\
        fn ca_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            // `ReadWrite`: a máscara faz o nó BLENDAR sobre a cor que já lá está,
            // então ele tem de a ler. Ausente lê a identidade — branco opaco, a
            // mesma base do `existing_tint` da CPU — e é sempre escrita.
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            // A máscara (`field.*` escreve-a). Identidade 1.0 = *sem máscara*.
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &[],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// A LUT da paleta. **Uma só**, contra as quatro do `motion.color_ramp`, e a
/// diferença é o que cada canal guarda: lá são quatro TABELAS escalares indexadas
/// pelo mesmo `t` contínuo; aqui é uma LISTA de tuplos, onde as quatro
/// componentes de uma cor são vizinhas e lidas de uma vez pelo mesmo índice
/// inteiro. Quatro buffers dariam a mesma resposta com quatro vezes o overhead de
/// binding.
static LUTS: &[LutSpec] = &[LutSpec {
    name: "ca_pal",
    text_key: PALETTE_KEY,
    resolution: palette::LUT_LEN,
    fill: palette::fill_lut,
}];

struct MotionColorArray;

impl NodeOp for MotionColorArray {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // The authored palette, or the factory one when the artist has not touched it
        // — one door, shared with the LUT fill so the device reads the same list.
        let palette = palette::palette_of(ctx.text_param(PALETTE_KEY));
        let offsets = scalar_col(ctx.input(1), VALUE_COL);
        let input = ctx.input(0);
        let n = input.count();
        let tint = cycle(n, &palette, &offsets, input);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "tint" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("tint", Column::Vec4(tint));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionColorArray))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Color Array",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // ⚠️ O `Consumes("falloff")` que aqui estava era a declaração MANUAL que um nó
    // sem kernel precisa (ADR-0155). Com o kernel abaixo registado, a `ColumnBinding`
    // de `falloff` passa a ser a fonte, e o diagnoser deriva o papel dela — declarar
    // as duas seria a segunda cópia de um facto.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_luts(MANIFEST.id, LUTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// ONE row: the palette itself. The count is `palette.len()`, the colours are the
/// colours — there is nothing else to say, and no `colors` slider to disagree with the
/// list it was capping.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "palette",
    label: "Palette",
    min: 0.0,
    max: 0.0,
    step: 0.0,
    widget: ParamWidget::Palette,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
