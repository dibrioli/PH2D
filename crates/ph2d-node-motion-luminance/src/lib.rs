#![forbid(unsafe_code)]
//! `motion.luminance` — **read colour back into a value**: the adapter that turns the
//! `tint` column into a per-instance scalar (Motion Nodes M1, adapters — doc 01 §1.7 /
//! doc 31). The inverse of the colour nodes — where `motion.color_ramp` maps a value TO
//! a colour, this maps a colour back TO a value, closing the loop so an instance's
//! appearance can drive its size / position / anything downstream.
//!
//! **Algorithm — the CHANNEL the `channel` param names** ([`CHANNEL_LABELS`]): the
//! Rec. 709 luma (`0.2126·R + 0.7152·G + 0.0722·B`, the default and index 0), the HSV
//! hue / saturation / value, or one of the four RGBA lanes. Reads the input stream's
//! `tint` and emits a **value field** — a `VALUE`-typed output (like
//! `value.instance_field`), so it plugs straight into any value input (a colour-ramp
//! `t`, a math node). An absent `tint` reads as transparent black, which is `0` in every
//! channel. Transcendental-free (HR-5): weighted sums, `min`/`max` and a division.
//! `Effect::Pure`.
//!
//! ⚠️ **Este é o ÚNICO leitor de cor do catálogo** (doc 89, fam. 9 §0: *o loop de cor é
//! one-way e LOSSY*), e é por isso que um param aqui não é enfeite — com um canal só, o
//! sistema sabia responder *"quão clara?"* e **nada mais** sobre a cor de uma instância.
//! Com o canal escolhível, a **saturação** ou o **matiz** de uma instância passam a
//! poder alimentar `sim.spawn` / `motion.cull` / `force.*`: o loop *aparência →
//! simulação*, que a `SUPERAR:` 3 do doc 89 nomeia como coisa que nenhuma referência faz.

use ph2d_color::rgb_to_hsv;
use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value-field output type (mirror of `value.instance_field`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Rec. 709 luma weights (linear RGB → relative luminance).
const WR: f32 = 0.2126;
const WG: f32 = 0.7152;
const WB: f32 = 0.0722;

/// **What a colour is read AS** (doc 89, fam. 9) — the artist word, and the index the
/// `channel` param stores. AE's *Colorama → Get Phase From* and Blender's *Separate
/// Color* are the same control; this node is the app's only colour→value reader, and
/// with one channel it could answer *"how bright?"* and nothing else about a colour.
///
/// ⚠️ **`Luma` é o índice 0 e o default**, então um grafo escrito antes deste param
/// rende o Rec. 709 de sempre — bit a bit, porque o braço dele é a expressão VERBATIM
/// que shipava e o `_ =>` recolhe qualquer índice fora da faixa (a convenção do
/// `channel_column` do `motion.drive`).
///
/// ⚠️ **HSL ficou de FORA, e não por falta de espaço** (o teto de opções é 48): a
/// *lightness* do HSL é `(max+min)/2`, a MESMA pergunta que o `Luma` responde com os
/// pesos perceptuais — duas respostas para *"quão clara é esta cor?"* dentro do mesmo
/// picker, e a nova seria a pior. E a saturação do HSL só é definida em termos dessa
/// lightness, então ela sai junto: uma *Saturation*, a do HSV, que é a que todo picker
/// de cor mostra.
///
/// ⚠️ **`Red`/`Green`/`Blue`/`Alpha` SOBREPÕEM a escada de lane do `value.attribute`**
/// (`tint` + `MODE_COMPONENT_BASE + k`) e isso é aceito com o mecanismo à vista: as
/// duas rotas leem a MESMA coluna e o MESMO lane, então não podem divergir em valor —
/// o que difere é a pergunta (*o lane k de uma coluna nomeada* × *o vermelho desta
/// cor*), e mandar o artista digitar `tint` com um modo mágico para pegar o vermelho é
/// o jargão que o picker de canais existiu para remover.
pub const CHANNEL_LABELS: &[&str] = &[
    "Luma",
    "Hue",
    "Saturation",
    "Value",
    "Red",
    "Green",
    "Blue",
    "Alpha",
];

/// The one value `channel` reads out of a colour — the law the WGSL body mirrors
/// branch for branch.
fn channel_of(c: [f32; 4], channel: i32) -> f32 {
    match channel {
        1 => rgb_to_hsv(c).0,
        2 => rgb_to_hsv(c).1,
        3 => rgb_to_hsv(c).2,
        4 => c[0],
        5 => c[1],
        6 => c[2],
        7 => c[3],
        // Luma, and every out-of-range index: the Rec. 709 sum, verbatim.
        _ => WR * c[0] + WG * c[1] + WB * c[2],
    }
}

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.luminance"),
    name: "motion.luminance",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "channel",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// The chosen channel of each tint (absent tint → the channel of transparent black,
/// which is `0` for every one of the eight).
fn luminance(tint: &[[f32; 4]], n: usize, channel: i32) -> Vec<f32> {
    (0..n)
        .map(|i| channel_of(tint.get(i).copied().unwrap_or([0.0; 4]), channel))
        .collect()
}

/// GPU compute kernel (ADR-0126) — the chosen channel per element.
///
/// A **bare** emitter: instances in, a VALUE stream out (one `v` column). The
/// sequencer derives that from the manifest's port types, so nothing is declared
/// here; riding the base would hand downstream a VALUE stream still carrying
/// `P`/`size`, which the CPU's does not have.
///
/// An absent `tint` reads the `[0,0,0,0]` identity and yields 0 — the same answer
/// the CPU's `tint.get(i).unwrap_or([0.0; 4])` gives, which is what makes the
/// column-absent variant of this module honest rather than merely compiling.
///
/// ⚠️ **UM corpo com um `if`, não `variant_by_param`.** As variantes existem quando
/// as BINDINGS diferem (o `motion.wiggle` escreve `P` ou `rot` ou `size`, e o módulo
/// gerado só define `write_<col>` para coluna BOUND) — aqui a leitura é sempre `tint`
/// e a escrita é sempre `v`, então oito variantes seriam oito cópias do mesmo par de
/// bindings esperando divergir. O `channel` é uniforme no dispatch inteiro, logo o
/// ramo não diverge entre invocações.
///
/// ⚠️ **O `lum_hsv` é o `rgb_to_hsv` da `ph2d-color`, branch por branch** — a mesma
/// ordem de `max`/`min`, a mesma guarda `delta <= 0`, a mesma divisão por 6. É o que
/// mantém a paridade CPU×GPU dentro do épsilon da casa (`1e-5`) em vez de aproximada
/// por sorte.
///
/// ⚠️ **E ela NÃO é bit-exata — medido, no canal que esta wave nem tocou:** o `Luma`
/// diverge por **1 ulp** (5,96e-8), porque o WGSL pode fundir multiplicação-e-soma e
/// `WR*r + WG*g + WB*b` com FMA arredonda diferente de três mul+add. É a mesma razão
/// pela qual *bit-a-bit não é a política deste projeto*; o gate mede contra `1e-5`, e
/// uma diferença de LEI (uma comparação trocada aqui) vale ~0,79 — quatro ordens de
/// grandeza acima, medido pela mutação.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let lum_c = read_tint(i);\n\
        let lum_ch = i32(round(params.channel));\n\
        var lum_v: f32;\n\
        if (lum_ch == 4) { lum_v = lum_c.x; }\n\
        else if (lum_ch == 5) { lum_v = lum_c.y; }\n\
        else if (lum_ch == 6) { lum_v = lum_c.z; }\n\
        else if (lum_ch == 7) { lum_v = lum_c.w; }\n\
        else if (lum_ch >= 1 && lum_ch <= 3) {\n\
        \x20   let lum_h = lum_hsv(lum_c);\n\
        \x20   if (lum_ch == 1) { lum_v = lum_h.x; }\n\
        \x20   else if (lum_ch == 2) { lum_v = lum_h.y; }\n\
        \x20   else { lum_v = lum_h.z; }\n\
        } else { lum_v = 0.2126 * lum_c.x + 0.7152 * lum_c.y + 0.0722 * lum_c.z; }\n\
        write_v(i, lum_v);\n",
    wgsl_lib: "\
        fn lum_hsv(c: vec4<f32>) -> vec3<f32> {\n\
        \x20   let mx = max(max(c.x, c.y), c.z);\n\
        \x20   let mn = min(min(c.x, c.y), c.z);\n\
        \x20   let d = mx - mn;\n\
        \x20   var h = 0.0;\n\
        \x20   if (d > 0.0) {\n\
        \x20       if (mx == c.x) { h = (c.y - c.z) / d + select(0.0, 6.0, c.y < c.z); }\n\
        \x20       else if (mx == c.y) { h = (c.z - c.x) / d + 2.0; }\n\
        \x20       else { h = (c.x - c.y) / d + 4.0; }\n\
        \x20       h = h / 6.0;\n\
        \x20   }\n\
        \x20   var s = 0.0;\n\
        \x20   if (mx > 0.0) { s = d / mx; }\n\
        \x20   return vec3<f32>(h, s, mx);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["channel"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "channel",
    label: "Read",
    min: 0.0,
    max: (CHANNEL_LABELS.len() - 1) as f32,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: CHANNEL_LABELS,
    },
}];

struct MotionLuminance;

impl NodeOp for MotionLuminance {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let input = ctx.input(0);
        let n = input.count();
        let tint: Vec<[f32; 4]> = match input.get("tint") {
            Some(Column::Vec4(v)) => v.clone(),
            _ => Vec::new(),
        };
        let v = luminance(&tint, n, channel);
        // A pure value-field output (like `value.instance_field`): just the `v` column.
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionLuminance))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Luminance",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O canal `Luma`, explícito em toda fixture desta seção: elas testam a LEI do
    /// Rec. 709, não o default, e uma fixture que chega ao canal por omissão inverte
    /// de sentido no dia em que o default se mover — seguindo VERDE sobre o oposto.
    const LUMA: i32 = 0;

    /// White is full luminance (the weights sum to 1), black is zero, mid-grey is ~0.5.
    #[test]
    fn white_is_one_black_is_zero() {
        let v = luminance(
            &[
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.5, 0.5, 0.5, 1.0],
            ],
            3,
            LUMA,
        );
        assert!((v[0] - 1.0).abs() < 1e-5, "white -> 1: {}", v[0]);
        assert!(v[1].abs() < 1e-5, "black -> 0: {}", v[1]);
        assert!((v[2] - 0.5).abs() < 1e-5, "grey -> 0.5: {}", v[2]);
    }

    /// Green reads brighter than red reads brighter than blue (the Rec. 709 ordering).
    /// FALSIFIED by a flat average (all three equal).
    #[test]
    fn green_is_brightest_blue_is_dimmest() {
        let v = luminance(
            &[
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
            ],
            3,
            LUMA,
        );
        assert!(v[1] > v[0] && v[0] > v[2], "G > R > B: {v:?}");
    }

    /// An absent tint reads as black (0), not a panic — **em todo canal**, porque a cor
    /// que o `unwrap_or` entrega é o preto transparente e os oito o leem como 0. É isto
    /// que mantém "coluna ausente = campo de zeros de comprimento N" verdadeiro depois
    /// do param, em vez de só no default.
    #[test]
    fn absent_tint_is_zero() {
        for ch in 0..CHANNEL_LABELS.len() as i32 {
            assert_eq!(
                luminance(&[], 3, ch),
                vec![0.0, 0.0, 0.0],
                "canal {}",
                CHANNEL_LABELS[ch as usize]
            );
        }
    }

    /// Uma amostra de cores com estrutura — cinzas, primárias puras, uma cor
    /// dessaturada e uma escura —, porque um canal só se distingue do vizinho onde a
    /// cor tem o que os separar (cinza colapsa matiz E saturação).
    fn spread() -> Vec<[f32; 4]> {
        vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.5],
            [0.0, 0.0, 1.0, 0.25],
            [0.5, 0.5, 0.5, 1.0],
            [0.8, 0.2, 0.4, 0.75],
            [0.1, 0.3, 0.2, 1.0],
        ]
    }

    /// **O default é o Rec. 709 que este nó sempre shipou, AO BIT.**
    ///
    /// ⚠️ O oráculo é a expressão LITERAL que shipava — pesos escritos à mão, sem
    /// tocar as consts —, então ele é uma referência congelada e não um espelho do
    /// `channel_of`. A mutação que ele existe para pegar é o braço `_ =>` deixar de
    /// ser a soma ponderada.
    #[test]
    fn the_default_channel_is_the_luma_this_node_always_shipped() {
        for c in spread() {
            let shipped = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            assert_eq!(
                channel_of(c, LUMA),
                shipped,
                "o canal 0 tem de ser byte-idêntico ao luma de sempre em {c:?}"
            );
        }
        // E o índice fora da faixa recolhe no MESMO braço (a convenção do
        // `channel_column`): um grafo com um `channel` estranho lê luma, não zero.
        assert_eq!(
            channel_of([0.8, 0.2, 0.4, 1.0], 99),
            channel_of([0.8, 0.2, 0.4, 1.0], LUMA)
        );
    }

    /// **Cada canal é uma FACE conhecida da cor** — a tabela vem da definição de HSV /
    /// Rec. 709, não do nosso código, que é o que a torna oráculo.
    ///
    /// Vermelho puro: matiz 0 · saturação 1 · valor 1 · R 1 · G 0 · B 0 · alfa 1, e o
    /// luma é o peso `WR`. FALSIFICADO por qualquer canal que devolva outro canal.
    #[test]
    fn each_channel_reads_the_face_of_the_colour_it_names() {
        let red = [1.0, 0.0, 0.0, 1.0];
        let expected = [0.2126_f32, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0];
        for (ch, want) in expected.iter().enumerate() {
            assert!(
                (channel_of(red, ch as i32) - want).abs() < 1e-6,
                "{} do vermelho puro: {} != {want}",
                CHANNEL_LABELS[ch],
                channel_of(red, ch as i32)
            );
        }
        // Verde puro tem matiz 1/3 e azul puro 2/3 — a volta do círculo, que um
        // "matiz" que na verdade devolvesse saturação (ambos 1 aqui) não teria.
        assert!((channel_of([0.0, 1.0, 0.0, 1.0], 1) - 1.0 / 3.0).abs() < 1e-6);
        assert!((channel_of([0.0, 0.0, 1.0, 1.0], 1) - 2.0 / 3.0).abs() < 1e-6);
    }

    /// **`Value` e `Luma` respondem perguntas DIFERENTES**, e é isso que justifica os
    /// dois estarem na lista: o azul puro tem valor `1.0` (é o teto dos canais) e luma
    /// `0.0722` (é o peso perceptual). FALSIFICADO por um `Value` que delegue ao luma.
    #[test]
    fn value_is_not_a_second_name_for_luma() {
        let blue = [0.0, 0.0, 1.0, 1.0];
        assert!((channel_of(blue, 3) - 1.0).abs() < 1e-6);
        assert!((channel_of(blue, 0) - 0.0722).abs() < 1e-6);
    }

    /// **Há UMA definição de matiz neste app.** O canal delega ao `ph2d_color::rgb_to_hsv`
    /// — o MESMO que o `RampColorMode::Hsv` do `motion.color_ramp` interpola —, então um
    /// grafo que rampeia em HSV e lê o canal de volta encontra o mesmo número.
    ///
    /// ⚠️ Este gate PINA A PORTA, não a aritmética: ele fica vermelho no dia em que
    /// alguém escrever uma cópia local do HSV aqui dentro, que é a única forma de as
    /// duas respostas divergirem.
    #[test]
    fn the_hue_saturation_and_value_come_from_the_one_colour_door() {
        for c in spread() {
            let (h, s, v) = ph2d_color::rgb_to_hsv(c);
            assert_eq!(channel_of(c, 1), h, "matiz de {c:?}");
            assert_eq!(channel_of(c, 2), s, "saturação de {c:?}");
            assert_eq!(channel_of(c, 3), v, "valor de {c:?}");
        }
    }

    /// **O param chega ao cook** — a metade que os gates de kernel não veem: `eval` lê
    /// `channel` e o passa adiante. FALSIFICADO por um `eval` que cravasse o luma.
    #[test]
    fn the_cook_honours_the_channel_param() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.luminance.test.blue"),
            name: "motion.luminance.test.blue",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(1)
                        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
                        .with("tint", Column::Vec4(vec![[0.0, 0.0, 1.0, 0.25]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionLuminance),
                    _ => None,
                }
            }
        }
        // Azul puro com alfa 0.25 separa os quatro: luma 0.0722 · valor 1 · azul 1 ·
        // alfa 0.25. Um `eval` que ignore o param devolve 0.0722 nas quatro leituras.
        for (ch, want) in [(0_i32, 0.0722_f32), (3, 1.0), (6, 1.0), (7, 0.25)] {
            let mut g = Graph::new();
            let src = g.add_node("motion.luminance.test.blue");
            let lum = g.add_node("motion.luminance");
            g.connect(Edge {
                from: (src, 0),
                to: (lum, 0),
                delayed: false,
            })
            .unwrap();
            g.set_param(lum, "channel", ch as f32);
            let mut cook = Cook::new();
            let out = cook.cook(&g, &Ops, lum, 0.0).unwrap();
            match out[0].as_stream().get("v").unwrap() {
                Column::Scalar(v) => assert!(
                    (v[0] - want).abs() < 1e-6,
                    "{} pelo cook: {} != {want}",
                    CHANNEL_LABELS[ch as usize],
                    v[0]
                ),
                _ => panic!("v"),
            }
        }
    }

    /// Deterministic + cooks through the registry: writes `v` from `tint` and passes the
    /// geometry through.
    #[test]
    fn registers_and_reads_luma_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.luminance.test.src"),
            name: "motion.luminance.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
                        .with(
                            "tint",
                            Column::Vec4(vec![[1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]]),
                        ),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionLuminance),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.luminance.test.src");
        let lum = g.add_node("motion.luminance");
        g.connect(Edge {
            from: (src, 0),
            to: (lum, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, lum, 0.0).unwrap();
        let s = out[0].as_stream();
        // A pure value field: the `v` column, not the geometry.
        match s.get("v").unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 2, "one luma per instance");
                assert!(
                    (v[0] - 1.0).abs() < 1e-5 && v[1].abs() < 1e-5,
                    "white->1, black->0"
                );
            }
            _ => panic!("v"),
        }
    }
}
