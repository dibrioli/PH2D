//! `value.instance_field` — the value-domain SOURCE OF PER-ELEMENT VARIATION
//! (Motion Nodes M2, the value domain — doc 12/13/14). It is the **only** node
//! that mints a genuinely length-N value field out of instance *identity* — every
//! other value so far is either a length-1 global (an LFO, a count) broadcast to
//! all, or a field whose per-element variation was smuggled in through a
//! behaviour's own `phase_stagger`. This is the sanctioned, first-class way that
//! per-element variation is BORN: Houdini's `@ptnum`/`@id`, Cavalry's Index /
//! Falloff, vvvv's spread index, TouchDesigner's Pattern CHOP ramp.
//!
//! **Modes** (`mode`):
//! - **Index** — `0, 1, 2, …, N−1` (the raw ordinal; feed a `value.map_range` to
//!   normalize).
//! - **Ramp** — `i / (N−1)` in `[0,1]` (the normalized gradient; `N=1 → 0`).
//! - **Random** — a stateless hash of `(seed, index) → [0,1)` (Jarzynski/Olano;
//!   transcendental-free, HR-5; see `hash.rs`). A per-instance jitter that
//!   reproduces bit-for-bit under scrub.
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column (doc 12). Cardinality follows the geometry: the
//! optional `in` port is read for its **count only** (like `value.lfo`) —
//! unconnected → a length-1 field (a degenerate single value). Nothing from the
//! input stream is passed through; this mints a fresh value.
//!
//! `Effect::Pure` — no clock, no state; a pure function of `(N, mode, seed)`.
//! Emits the column `v`. Transcendental-free (HR-5): only integer hash + one
//! division for Ramp.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::rand01;

/// The instance stream type — read for its count only (the optional `in` port).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.instance_field"),
    name: "value.instance_field",
    // Optional: connected → count N; unconnected → a single degenerate value.
    // Read for its count only; never passed through.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Pure: a fresh field per cook, a pure function of N + params. No clock/state.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Index (0..N-1) · 1 Ramp (0..1) · 2 Random (hash → [0,1)).
        ParamSpec {
            name: "mode",
            default: 1.0,
        },
        // ⚠️ **Apendado**: `0` = Index, o nó que sempre shipou. Ver [`KeyBy`].
        ParamSpec {
            name: "key",
            default: 0.0,
        },
        // Random seed (Random mode only). Integer-valued; rounded at eval.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⚠️ **Apendado, e `0` é o mundo de hoje.** Ver [`decorrelate`].
        ParamSpec {
            name: "unique_per_node",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// How the per-instance value is minted from the instance's ordinal.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum FieldMode {
    /// `0, 1, 2, … N−1` — the raw ordinal.
    Index,
    /// `i / (N−1)` in `[0,1]` — the normalized gradient.
    Ramp,
    /// A stateless hash of `(seed, index)` → `[0,1)`.
    Random,
}

impl FieldMode {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            0 => FieldMode::Index,
            2 => FieldMode::Random,
            _ => FieldMode::Ramp,
        }
    }
}

/// **Que número identifica um elemento** (doc 89 folha 15, o P0).
///
/// ⚠️ **A chave era o ÍNDICE, e o índice é POSIÇÃO NA LISTA, não identidade.** Um
/// `motion.sort` ou um `motion.cull` a montante reordena o conjunto e **todo
/// valor aleatório troca de dono** — em silêncio, porque nada na tela diz que a
/// variação mudou de elemento. O Blender separa **ID** de **Seed** pelo mesmo
/// motivo, e a doc do *Distribute Points on Faces* escreve a consequência: *"when
/// the mesh is deformed or the density changes the values will be consistent for
/// each remaining point"*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyBy {
    /// A posição na lista — o nó que sempre shipou, e o default.
    Index,
    /// A coluna `id`, quando ela existe. ⚠️ **Sem coluna `id` a chave CAI no
    /// índice**, e não em zero: um conjunto sem identidade tem a identidade que
    /// tem. É a semântica do Blender (*"ID: default o atributo `id` se existir,
    /// senão o índice"*), e é o que impede uma grade — que não carrega `id` — de
    /// ficar com todos os elementos a partilhar o MESMO valor aleatório.
    Id,
}

impl KeyBy {
    fn from_param(v: f32) -> Self {
        if v.round() as i32 == 1 {
            Self::Id
        } else {
            Self::Index
        }
    }
}

/// O golden-ratio de 32 bits — o MESMO multiplicador que o `rand01` usa no slot
/// da semente (`hash.rs`), reusado aqui para o mesmo fim: espalhar.
const GOLDEN32: u32 = 0x9e37_79b9;

/// **A semente EFETIVA:** a autorada, decorrelacionada pela identidade do nó
/// quando o artista pede.
///
/// ⚠️ **Dois `value.instance_field` arrastados para a tela com os defaults
/// produziam campos IDÊNTICOS** — o *"Noise com Use Layer as Seed evita gêmeos"*
/// do Cavalry (doc 63 §D). A única saída era o artista lembrar de digitar uma
/// semente diferente em cada um, e nada na tela dizia que era preciso.
///
/// ⚠️ **É opt-in, e o default `0` é LITERALMENTE o nó de antes** — `node_key`
/// dobrado por default mudaria a aleatoriedade de **todo grafo já salvo** (lei 6).
///
/// ⚠️ **SOMAR o `node_key` cru seria colisão fácil:** um nó de semente 5 e id 3 e
/// outro de semente 3 e id 5 cairiam na MESMA semente efetiva. Multiplicar a
/// identidade pelo golden-ratio antes de somar espalha os ids por toda a faixa,
/// e é o mesmo espalhamento que o hash já faz um degrau adiante.
///
/// ⚠️ **O param chama-se `unique_per_node`, e não `use_node_as_seed`** (o nome da
/// feature no Cavalry, e o que a folha 15 escreve). Ele é um BOOLEANO *sobre* a
/// semente, não uma semente — e o gate de convenção `a_seed_wears_the_seed_widget`
/// pegou o nome antigo na hora, porque um scanner por nome lê `*_seed` como *"isto
/// é uma semente"* e exige o widget de semente. O rótulo do painel já dizia a
/// verdade ("Unique Per Node"); o identificador passou a dizê-la também.
pub(crate) fn decorrelate(seed: u32, node_key: u32, use_node: bool) -> u32 {
    if use_node {
        seed.wrapping_add(node_key.wrapping_mul(GOLDEN32))
    } else {
        seed
    }
}

/// Mint the length-`n` value field for `mode`/`seed`, keyed by index or by `id`.
///
/// ⚠️ **O `Ramp` ignora a chave, com motivo:** uma rampa é *onde você está na
/// lista*, que é posicional por definição — e `id / (n − 1)` sobre ids esparsos
/// nem fica em `[0, 1]` nem é uma rampa. O `ParamGate` esconde o seletor nesse
/// modo, então não há knob morto a explicar.
fn field(n: usize, mode: FieldMode, seed: u32, key: KeyBy, ids: Option<&[f32]>) -> Vec<f32> {
    // Com a chave no ÍNDICE nada disto é lido, e a expressão abaixo reduz
    // LITERALMENTE ao nó que shipava — `i as f32` e `i as u32`, sem passar por
    // um `f32` intermediário que arredondaria acima de 2²⁴.
    let key_f = |i: usize| -> f32 {
        match key {
            KeyBy::Index => i as f32,
            KeyBy::Id => ids.and_then(|v| v.get(i)).copied().unwrap_or(i as f32),
        }
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "um id é um inteiro pequeno carregado num f32 (a convenção de coluna);                   o `max(0)` cobre um id negativo autorado à mão"
    )]
    let key_u = |i: usize| -> u32 {
        match key {
            KeyBy::Index => i as u32,
            KeyBy::Id => ids
                .and_then(|v| v.get(i))
                .map_or(i as u32, |v| v.max(0.0) as u32),
        }
    };
    (0..n)
        .map(|i| match mode {
            FieldMode::Index => key_f(i),
            // `N == 1` has no span → the low end (0.0), never a divide by zero.
            FieldMode::Ramp => {
                if n > 1 {
                    i as f32 / (n - 1) as f32
                } else {
                    0.0
                }
            }
            FieldMode::Random => rand01(seed, key_u(i)),
        })
        .collect()
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`field`], element for
/// element, including the integer hash.
///
/// **The measurement that asked for this.** `two_seam_hybrid_timing` found that
/// at 2 M elements **71% of a two-seam hybrid's CPU cost was the CPU prefix** —
/// paid to cook a `grid` plus two of THESE, because they had no kernel. A node
/// that mints per-element variation sits at the head of a great many chains, so
/// leaving it uncovered taxed every one of them with a seam.
///
/// `if_round` is round-half-away-from-zero to match Rust's `f32::round`: `mode`
/// picks a BRANCH and `seed` an integer, so WGSL's half-even would choose a
/// different field at a half-integer
/// ([[feedback_cpu_gpu_rounding_conventions_diverge]]).
///
/// The hash is `hash.rs` transliterated. WGSL `u32` arithmetic wraps and `>>` is
/// a logical shift, which is exactly what `wrapping_mul` and Rust's `u32 >>` do —
/// so this is the same integer sequence, not an approximation of it. The
/// `1 << 24` divisor keeps the draw exactly representable and never 1.0.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let if_mode = i32(if_round(params.mode));\n\
        // A chave: o indice, ou a coluna `id` quando ela existe. ⚠️ `HAS_id`\n\
        // e o que impede a coluna AUSENTE de ler zero para todo elemento (a\n\
        // identidade do binding), o que daria a TODOS o mesmo valor aleatorio.\n\
        let if_by_id = i32(if_round(params.key)) == 1 && HAS_id;\n\
        var if_kf = f32(i);\n\
        var if_ku = i;\n\
        if (if_by_id) {\n\
        \x20   if_kf = read_id(i);\n\
        \x20   if_ku = u32(max(if_kf, 0.0));\n\
        }\n\
        var if_v: f32;\n\
        if (if_mode == 0) {\n\
        \x20   if_v = if_kf;\n\
        } else if (if_mode == 2) {\n\
        \x20   var if_seed = u32(max(if_round(params.seed), 0.0));\n\
        \x20   // A semente EFETIVA (ver `decorrelate`): a identidade do no,\n\
        \x20   // espalhada pelo golden-ratio, quando o artista a pede. Off e a\n\
        \x20   // adicao de zero, logo byte-identico.\n\
        \x20   if (i32(if_round(params.unique_per_node)) == 1) {\n\
        \x20       if_seed = if_seed + params.node_key * 0x9e3779b9u;\n\
        \x20   }\n\
        \x20   if_v = if_rand01(if_seed, if_ku);\n\
        } else {\n\
        \x20   // N == 1 has no span: the low end, never a divide by zero.\n\
        \x20   if (params.count > 1u) {\n\
        \x20       if_v = f32(i) / f32(params.count - 1u);\n\
        \x20   } else {\n\
        \x20       if_v = 0.0;\n\
        \x20   }\n\
        }\n\
        write_v(i, if_v);\n",
    wgsl_lib: "\
        fn if_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn if_rand01(seed: u32, index: u32) -> f32 {\n\
            var h = seed * 0x9e3779b9u + index * 0x85ebca6bu;\n\
            h = h ^ (h >> 16u);\n\
            h = h * 0x7feb352du;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x846ca68bu;\n\
            h = h ^ (h >> 16u);\n\
            return f32(h >> 8u) / 16777216.0;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // ⚠️ A `identity` de uma coluna `id` AUSENTE e zero, e zero nao e o
        // indice — e por isso o corpo ramifica no `HAS_id` em vez de confiar
        // nela. Sem essa ramificacao um conjunto sem `id` daria a todo elemento
        // a MESMA chave, e o modo Random pintaria tudo do mesmo valor.
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["mode", "key", "seed", "unique_per_node"],
    // Same law as `value.lfo`, and the same expression `eval` uses: connected it
    // is one value per instance, unconnected it is ONE degenerate value. The
    // engine's default ("as wide as port 0") would size an unconnected one at 0
    // and skip the stage.
    count_law: Some(instance_field_count),
    variant_by_param: None,
    applicable: None,
};

fn instance_field_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.first().copied().unwrap_or(0).max(1) as usize)
}

struct ValueInstanceField;

impl NodeOp for ValueInstanceField {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = FieldMode::from_param(ctx.param("mode"));
        let seed = decorrelate(
            ctx.param("seed").round().max(0.0) as u32,
            ctx.node_key(),
            ctx.param("unique_per_node").round() as i32 == 1,
        );
        let key = KeyBy::from_param(ctx.param("key"));
        // Cardinality follows the geometry; unconnected → one degenerate value.
        let input = ctx.input(0);
        let n = input.count().max(1);
        let ids = match input.get("id") {
            Some(Column::Scalar(v)) => Some(v.clone()),
            _ => None,
        };
        let v = field(n, mode, seed, key, ids.as_deref());
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueInstanceField))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ⚠️ **ELE PRECISA DA GEOMETRIA, e antes de 2026-08-20 não o dizia a ninguém.**
    // O doc deste nó é explícito — *"Cardinality follows the geometry; unconnected →
    // one degenerate value"* —, mas o diagnoser (ADR-0155) não tinha como saber: o
    // `P` chega aqui pela CONTAGEM e pelo `id`, não por uma binding que o leia, então
    // nem o kernel nem uma `Requires` o denunciavam.
    //
    // O preço foi medido num smoke reprovado (Enio: *"todas as peças paradas"*): uma
    // cena deixou este nó SOLTO, ele devolveu o valor degenerado, um `motion.drive` o
    // transmitiu a toda a fileira, e ele era **zero** — densidade zero é empuxo
    // nenhum. Nenhum gate viu, porque o nó não declarava precisar de nada.
    //
    // ⚠️ **`Requires` e não `Consumes`:** ele não come uma coluna transiente — ele
    // exige que a geometria ESTEJA lá para ter cardinalidade. É a mesma declaração
    // que um deformador faz.
    reg.register_couplings(MANIFEST.id, &[ph2d_node_registry::Coupling::Requires("P")]);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Instance Field",
            // Utility grey: a value SOURCE, plumbing (not a visible transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // ⚠️ O `Ramp` é POSICIONAL por definição (*onde você está na lista*), e
    // `id / (n − 1)` sobre ids esparsos nem fica em `[0, 1]`. O seletor some
    // nesse modo em vez de ficar lá sem fazer nada.
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// O seletor de chave só aparece nos modos que a LEEM (`0` Index · `2` Random).
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "key",
        when: "mode",
        values: &[0, 2],
    },
    // ⚠️ A semente e a decorrelação só existem no modo que as LÊ. Index e Ramp
    // são funções puras do ordinal — um toggle ali seria controle morto.
    ph2d_node_registry::ParamGate {
        param: "unique_per_node",
        when: "mode",
        values: &[2],
    },
    // ⚠️ **Esta linha faltava, e o comentário acima AFIRMAVA que ela existia** — *"o `Seed` ao
    // lado dele já é gateado pelo mesmo motivo"*, e não estava (doc 90 §2, 2026-08-22). O nó
    // nasce em `Ramp`, então o campo `Seed` aparecia com a roleta de re-roll ao lado, num modo
    // que é função pura do ordinal e nunca o lê.
    //
    // ⚠️ É o mais traiçoeiro dos dezanove: um doc-comment que garante a cura torna a ausência
    // dela **invisível a quem relê o arquivo**. *Uma nota sobre o que o código faz é uma
    // afirmação, e ela envelhece com a mesma facilidade que qualquer outra.*
    ph2d_node_registry::ParamGate {
        param: "seed",
        when: "mode",
        values: &[2],
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "key",
        label: "Key By",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Index", "Id"],
        },
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Index", "Ramp", "Random"],
        },
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "unique_per_node",
        label: "Unique Per Node",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
#[path = "key_tests.rs"]
mod key_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // A grid source: `n` instances at the origin, so the field can read a count.
    static GRID_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.instance_field.test.grid"),
        name: "value.instance_field.test.grid",
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
    struct Grid(usize);
    impl NodeOp for Grid {
        fn manifest(&self) -> &'static NodeManifest {
            &GRID_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0).with("P", Column::Vec2(vec![[0.0, 0.0]; self.0])));
        }
    }

    // Direct unit tests of the core (no cook needed for the field math).
    #[test]
    fn index_mode_is_the_raw_ordinal() {
        assert_eq!(
            field(4, FieldMode::Index, 0, KeyBy::Index, None),
            vec![0.0, 1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn ramp_mode_is_the_normalized_gradient() {
        assert_eq!(
            field(5, FieldMode::Ramp, 0, KeyBy::Index, None),
            vec![0.0, 0.25, 0.5, 0.75, 1.0]
        );
        // N == 1 has no span → 0, never a divide by zero.
        assert_eq!(field(1, FieldMode::Ramp, 0, KeyBy::Index, None), vec![0.0]);
    }

    /// Random is a per-instance field in `[0,1)` — NOT all-equal (the whole point
    /// vs a broadcast constant) and reproducible for a fixed seed.
    #[test]
    fn random_mode_varies_per_instance_and_reproduces() {
        let a = field(8, FieldMode::Random, 42, KeyBy::Index, None);
        let b = field(8, FieldMode::Random, 42, KeyBy::Index, None);
        assert_eq!(a, b, "a fixed seed reproduces the field bit-for-bit");
        assert!(a.iter().all(|&v| (0.0..1.0).contains(&v)), "in [0,1)");
        assert!(
            a.windows(2).any(|w| w[0] != w[1]),
            "the field varies per instance (not a broadcast constant)"
        );
        // A different seed decorrelates.
        assert_ne!(
            a,
            field(8, FieldMode::Random, 43, KeyBy::Index, None),
            "seed changes the field"
        );
    }

    /// End-to-end through the cook: the field's length follows the connected
    /// geometry, and Ramp really spans 0..1 across the instances.
    #[test]
    fn mints_a_field_sized_to_the_connected_geometry() {
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == GRID_MAN.id => Some(Box::leak(Box::new(Grid(4))) as &dyn NodeOp),
                    t if t == MANIFEST.id => Some(&ValueInstanceField),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let grid = g.add_node("value.instance_field.test.grid");
        let f = g.add_node("value.instance_field");
        g.connect(Edge {
            from: (grid, 0),
            to: (f, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(f, "mode", 1.0); // Ramp
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, f, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]),
            _ => panic!("v"),
        }
    }

    /// UNCONNECTED input → a single degenerate value (length-1) through the cook,
    /// never a panic on an empty input.
    #[test]
    fn an_unconnected_field_is_a_single_value() {
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == MANIFEST.id => Some(&ValueInstanceField),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let f = g.add_node("value.instance_field");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, f, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v.len(), 1, "unconnected → one degenerate value"),
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}

#[cfg(test)]
#[path = "seed_tests.rs"]
mod seed_tests;
