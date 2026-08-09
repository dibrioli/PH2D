//! `motion.morph` — a **vertex crossfade** between two instance streams (Motion
//! Nodes M3, deformers — doc 01 §3 / doc 19). It linearly interpolates each
//! element's position from stream `a` toward stream `b` by a `blend` value, so one
//! layout melts into another: a `motion.grid` into a `motion.fibonacci` spiral, a
//! spiral into a `motion.scatter` cloud. This is the classic morph target /
//! blend-shape reduced to points — the deformer that reshapes by INTERPOLATION
//! rather than by a spatial function (`motion.twist`) or a random field.
//!
//! **The blend is a value input** (the value domain — doc 12), so it can be
//! ANIMATED: wire a `value.lfo` to `blend` and the two shapes trade back and forth
//! (the boot scene does exactly this). `blend = 0` is all `a`, `1` all `b`, and it
//! is per-element (a length-N `blend` morphs each element on its own schedule — a
//! staggered dissolve); unconnected it reads as `0` (all `a`). The output length
//! is the `min` of the two streams (only the PAIRED elements morph, by row order).
//!
//! **O STREAM INTEIRO atravessa** (doc 89 fam. 7): as quantidades que o lowering desenha
//! ([`LERPED`]) desvanecem, e toda outra coluna é carregada pelo **vizinho mais próximo** —
//! uma identidade não é uma grandeza, e a média de duas texturas é uma terceira que ninguém
//! autorou. Antes disto o nó emitia **só `P`**, então morfar dois `source.object` perdia a
//! aparência e o lowering caía na tile 0.
//!
//! `Pure` — no clock, no state; the animation comes from the `blend` input.
//! Transcendental-free (HR-5): a lerp.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `blend` input — the per-instance scalar field on the `v`
/// column (mirror of `ph2d_node_pulse_counter::VALUE`; kept local, leaf crate).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value stream's column.
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.morph"),
    name: "motion.morph",
    inputs: &[
        PortSpec {
            name: "a",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "b",
            ty: INST_VEC2,
        },
        // The 0..1 crossfade — a value, so it can be animated. Optional:
        // unconnected reads as 0 (all `a`).
        PortSpec {
            name: "blend",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// As colunas que um morph **INTERPOLA** — as quantidades contínuas que o lowering desenha
/// (`ph2d-eval-motion::lower`: posição, tamanho, rotação, cor). Toda outra coluna é
/// **CARREGADA** pelo vizinho mais próximo.
///
/// ⚠️ **A lista é branca, e a assimetria é o argumento inteiro.** As duas listas possíveis
/// apodrecem — mas o modo de falha delas é oposto:
///
/// - lista **NEGRA** ("não interpole `id`/`texture_id`/`geometry_id`"): a coluna de
///   identidade que alguém acrescentar amanhã é MEDIADA em silêncio. O lowering lê
///   `texture_id` com `as u32`, então a média de 0 e 7 vira a textura **3** — uma que o
///   artista nunca pediu, sem erro e sem aviso.
/// - lista **BRANCA** (esta): a quantidade que alguém acrescentar amanhã **PULA** em vez de
///   desvanecer. Visível, inofensiva, e o conserto é uma linha aqui.
///
/// *Escolha a lista cujo apodrecimento se VÊ.*
///
/// ⚠️ **`uv_rect` fica FORA de propósito, e não é esquecimento:** ele nomeia QUAIS pixels,
/// junto com o `texture_id` — interpolar o retângulo varre o átlas e mostra o que estiver
/// entre duas tiles. Ele não é um `_id`, então uma regra por SUFIXO o teria perdido.
const LERPED: &[&str] = &["P", "size", "rot", "tint"];

/// `a + (b − a)·t`, a forma **exata nos extremos**: em `t = 1` o primeiro termo é
/// `a·0.0`, que o IEEE-754 faz exatamente zero para qualquer finito, e o segundo é `b·1.0`.
/// É isso que deixa `blend = 1` ser `b` **ao bit**, e não *quase* `b`.
fn lerp1(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

/// The `blend` for element `i`: **unconnected (empty) → 0.0** (all `a`); length-1
/// broadcasts; length-N is per-element. Clamped to `[0, 1]`.
fn blend_at(vals: &[f32], i: usize) -> f32 {
    let t = match vals.len() {
        0 => 0.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(0.0),
    };
    t.clamp(0.0, 1.0)
}

/// Uma coluna truncada às `n` primeiras linhas (o irmão exato do `trunc` do
/// `motion.mixer`) — o que sobra de uma coluna que só UM lado carrega.
fn trunc(c: &Column, n: usize) -> Column {
    match c {
        Column::Scalar(v) => Column::Scalar(v[..n.min(v.len())].to_vec()),
        Column::Vec2(v) => Column::Vec2(v[..n.min(v.len())].to_vec()),
        Column::Vec3(v) => Column::Vec3(v[..n.min(v.len())].to_vec()),
        Column::Vec4(v) => Column::Vec4(v[..n.min(v.len())].to_vec()),
    }
}

/// Interpola duas colunas do MESMO variant, lane a lane, pelo blend por-elemento.
/// `None` quando os variants discordam — aí quem decide é o chamador (o vizinho próximo),
/// porque somar um `Vec2` a um `Vec4` não significa nada.
fn lerp_col(a: &Column, b: &Column, blend: &[f32], n: usize) -> Option<Column> {
    macro_rules! lanes {
        ($x:expr, $y:expr, $w:literal) => {
            (0..n)
                .map(|i| {
                    let t = blend_at(blend, i);
                    let mut r = $x[i];
                    for c in 0..$w {
                        r[c] = lerp1($x[i][c], $y[i][c], t);
                    }
                    r
                })
                .collect()
        };
    }
    Some(match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => Column::Scalar(
            (0..n)
                .map(|i| lerp1(x[i], y[i], blend_at(blend, i)))
                .collect(),
        ),
        (Column::Vec2(x), Column::Vec2(y)) => Column::Vec2(lanes!(x, y, 2)),
        (Column::Vec3(x), Column::Vec3(y)) => Column::Vec3(lanes!(x, y, 3)),
        (Column::Vec4(x), Column::Vec4(y)) => Column::Vec4(lanes!(x, y, 4)),
        _ => return None,
    })
}

/// O **VIZINHO MAIS PRÓXIMO**, por elemento: abaixo de meio caminho a linha de `a`, daí em
/// diante a de `b`.
///
/// ⚠️ **Não é gosto — é o contrato do próprio nó.** O doc dele promete *"`blend = 0` is all
/// `a`, `1` all `b`"*, e uma identidade não desvanece: segurar `a` faria `blend = 1`
/// **não** ser `b`, contradizendo a promessa. O corte no meio é o preço honesto disso, e é
/// por elemento — um blend escalonado troca cada linha no seu próprio tempo.
fn nearest_col(a: &Column, b: &Column, blend: &[f32], n: usize) -> Column {
    macro_rules! pick {
        ($x:expr, $y:expr, $ctor:path) => {
            $ctor(
                (0..n)
                    .map(|i| {
                        if blend_at(blend, i) < 0.5 {
                            $x[i]
                        } else {
                            $y[i]
                        }
                    })
                    .collect(),
            )
        };
    }
    match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => pick!(x, y, Column::Scalar),
        (Column::Vec2(x), Column::Vec2(y)) => pick!(x, y, Column::Vec2),
        (Column::Vec3(x), Column::Vec3(y)) => pick!(x, y, Column::Vec3),
        (Column::Vec4(x), Column::Vec4(y)) => pick!(x, y, Column::Vec4),
        // Variants que discordam: o lado `a` truncado. Um stream cujo `size` é `Vec2` de um
        // lado e `Scalar` do outro é malformado a montante, e inventar uma conversão aqui
        // esconderia isso.
        _ => trunc(a, n),
    }
}

/// O stream inteiro morfado: as quantidades desvanecem, o resto é carregado.
///
/// ⚠️ **Uma coluna que só UM lado carrega é PROPAGADA, não descartada** — descartar é
/// exatamente o defeito que esta wave conserta, e a alternativa (inventar a identidade do
/// lado ausente) escolheria por um artista que não pediu nada.
fn morph_stream(a: &Stream, b: &Stream, blend: &[f32]) -> Stream {
    let n = a.count().min(b.count());
    let mut out = Stream::new(n);
    for (name, ca) in a.columns() {
        let col = match b.get(name) {
            Some(cb) if LERPED.contains(&name.as_str()) => {
                lerp_col(ca, cb, blend, n).unwrap_or_else(|| nearest_col(ca, cb, blend, n))
            }
            Some(cb) => nearest_col(ca, cb, blend, n),
            None => trunc(ca, n),
        };
        out.set(name.clone(), col);
    }
    for (name, cb) in b.columns() {
        if a.get(name).is_none() {
            out.set(name.clone(), trunc(cb, n));
        }
    }
    out
}

struct MotionMorph;

impl NodeOp for MotionMorph {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let blend: Vec<f32> = match ctx.input(2).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let out = morph_stream(ctx.input(0), ctx.input(1), &blend);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMorph))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Morph",
            // Transform blue: a spatial deformer (crossfade of two layouts).
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // No params — the crossfade is a value input, so it can be animated.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// O `P` que o PRODUTO produz — os quatro gates de crossfade abaixo chamavam uma
    /// especialização `Vec2` que, depois desta wave, **ficou sem chamador de produto**:
    /// eles teriam continuado verdes sobre um espelho enquanto o `morph_stream` quebrava.
    /// Agora entram pela mesma porta que o `eval`.
    fn morph_p(a: &[[f32; 2]], b: &[[f32; 2]], blend: &[f32]) -> Vec<[f32; 2]> {
        let sa = Stream::new(a.len()).with("P", Column::Vec2(a.to_vec()));
        let sb = Stream::new(b.len()).with("P", Column::Vec2(b.to_vec()));
        match morph_stream(&sa, &sb, blend).get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        }
    }

    /// `blend = 0` is all `a`, `1` all `b`, `0.5` the midpoint — the crossfade.
    #[test]
    fn it_crossfades_from_a_to_b() {
        let a = [[0.0, 0.0], [10.0, 0.0]];
        let b = [[0.0, 10.0], [10.0, 10.0]];
        assert_eq!(
            morph_p(&a, &b, &[0.0]),
            vec![[0.0, 0.0], [10.0, 0.0]],
            "0 = a"
        );
        assert_eq!(
            morph_p(&a, &b, &[1.0]),
            vec![[0.0, 10.0], [10.0, 10.0]],
            "1 = b"
        );
        assert_eq!(
            morph_p(&a, &b, &[0.5]),
            vec![[0.0, 5.0], [10.0, 5.0]],
            "0.5 = midpoint"
        );
    }

    /// Unconnected `blend` (empty) reads as 0 → the output is `a` untouched, so a
    /// bare morph is not a surprise. Values clamp to `[0, 1]`.
    #[test]
    fn an_empty_blend_is_a_and_values_clamp() {
        let a = [[1.0, 2.0]];
        let b = [[9.0, 9.0]];
        assert_eq!(morph_p(&a, &b, &[]), vec![[1.0, 2.0]], "empty → a");
        assert_eq!(
            morph_p(&a, &b, &[2.0]),
            vec![[9.0, 9.0]],
            "over-1 clamps to b"
        );
        assert_eq!(
            morph_p(&a, &b, &[-1.0]),
            vec![[1.0, 2.0]],
            "under-0 clamps to a"
        );
    }

    /// FALSIFICATION of the per-element blend: a length-N `blend` morphs each
    /// element on its OWN schedule (a staggered dissolve), not the whole set at one
    /// rate. Element 0 stays at `a`, element 1 reaches `b`.
    #[test]
    fn a_per_element_blend_staggers_the_morph() {
        let a = [[0.0, 0.0], [0.0, 0.0]];
        let b = [[10.0, 0.0], [0.0, 10.0]];
        assert_eq!(
            morph_p(&a, &b, &[0.0, 1.0]),
            vec![[0.0, 0.0], [0.0, 10.0]],
            "element 0 held at a, element 1 at b"
        );
    }

    /// Mismatched lengths morph only the PAIRED prefix (`min` of the two) — no
    /// out-of-bounds, no invented points.
    #[test]
    fn mismatched_lengths_morph_the_paired_prefix() {
        let a = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let b = [[0.0, 8.0]]; // only one point in b
        let out = morph_p(&a, &b, &[1.0]);
        assert_eq!(
            out,
            vec![[0.0, 8.0]],
            "only the first (paired) element morphs"
        );
    }

    /// End to end through the cook: two position sources crossfade at blend 0.5.
    #[test]
    fn morphs_two_sources_through_the_cook() {
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.morph.test.src"),
            name: "motion.morph.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[ph2d_nodegraph::node::ParamSpec {
                name: "y",
                default: 0.0,
            }],
            lowerings: &[LoweringKind::Cpu],
        };
        static VAL: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.morph.test.val"),
            name: "motion.morph.test.val",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: VALUE,
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
                let y = ctx.param("y");
                ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, y]])));
            }
        }
        struct Val;
        impl NodeOp for Val {
            fn manifest(&self) -> &'static NodeManifest {
                &VAL
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![0.5])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == VAL.id => Some(&Val),
                    t if t == MANIFEST.id => Some(&MotionMorph),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let a = g.add_node("motion.morph.test.src");
        let b = g.add_node("motion.morph.test.src");
        let blend = g.add_node("motion.morph.test.val");
        let m = g.add_node("motion.morph");
        g.set_param(a, "y", 0.0);
        g.set_param(b, "y", 10.0);
        for (from, port) in [(a, 0), (b, 1), (blend, 2)] {
            g.connect(Edge {
                from: (from, 0),
                to: (m, port),
                delayed: false,
            })
            .unwrap();
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 5.0]], "blend 0.5 → midpoint"),
            _ => panic!("P"),
        }
    }

    /// Um stream de `n` linhas com as colunas que um objeto de verdade carrega.
    fn rich(p: [f32; 2], size: f32, tint: [f32; 4], tex: f32) -> Stream {
        Stream::new(2)
            .with("P", Column::Vec2(vec![p; 2]))
            .with("size", Column::Vec2(vec![[size, size]; 2]))
            .with("rot", Column::Scalar(vec![0.0; 2]))
            .with("tint", Column::Vec4(vec![tint; 2]))
            .with("texture_id", Column::Scalar(vec![tex; 2]))
            .with("uv_rect", Column::Vec4(vec![[0.0, 0.0, 1.0, 1.0]; 2]))
    }
    fn scalar0(s: &Stream, name: &str) -> f32 {
        match s.get(name) {
            Some(Column::Scalar(v)) => v[0],
            _ => panic!("`{name}` sumiu do morph"),
        }
    }

    /// **O MORPH CARREGA A APARÊNCIA EM VEZ DE A DESCARTAR** — o defeito que a §1 do doc 89
    /// fam. 7 mediu: `ctx.emit(Stream::new(n).with("P", …))` jogava fora `size` · `tint` ·
    /// `rot` · `id` · `uv_rect` · **`texture_id`** · **`geometry_id`**.
    ///
    /// ⚠️ A consequência era medível e cara: morfar dois `source.object` **perdia a
    /// aparência** — as convenções do doc 86 / ADR-0154 sumiam e o lowering caía na tile 0,
    /// ou seja quads brancos. Este gate nasce VERMELHO no código anterior, onde a saída tem
    /// exatamente UMA coluna.
    #[test]
    fn the_morph_carries_the_appearance_instead_of_dropping_it() {
        let a = rich([0.0, 0.0], 1.0, [1.0, 0.0, 0.0, 1.0], 0.0);
        let b = rich([10.0, 0.0], 3.0, [0.0, 0.0, 1.0, 1.0], 7.0);
        let out = morph_stream(&a, &b, &[0.5]);
        for name in ["P", "size", "rot", "tint", "texture_id", "uv_rect"] {
            assert!(
                out.get(name).is_some(),
                "`{name}` tem de atravessar o morph — descartar É o defeito"
            );
        }
    }

    /// **UMA IDENTIDADE É CARREGADA PELO VIZINHO MAIS PRÓXIMO, NUNCA MEDIADA.**
    ///
    /// ⚠️ O oráculo é o número que o lowering LERIA: ele faz `as u32`, então a média de 0 e
    /// 7 vira a textura **3** — uma que ninguém pediu, sem erro e sem aviso. O gate exige
    /// que o valor seja SEMPRE um dos dois autorados, varrendo o blend inteiro.
    #[test]
    fn an_identity_is_carried_by_the_nearest_neighbour_never_averaged() {
        let a = rich([0.0, 0.0], 1.0, [1.0; 4], 0.0);
        let b = rich([10.0, 0.0], 1.0, [1.0; 4], 7.0);
        for k in 0..=10 {
            let t = k as f32 / 10.0;
            let tex = scalar0(&morph_stream(&a, &b, &[t]), "texture_id");
            assert!(
                tex == 0.0 || tex == 7.0,
                "blend {t}: a textura tem de ser UMA das duas autoradas, veio {tex}"
            );
            assert_eq!(tex, if t < 0.5 { 0.0 } else { 7.0 }, "o vizinho próximo");
        }
    }

    /// **AS QUANTIDADES DESVANECEM, E `blend = 1` É `b` AO BIT.**
    ///
    /// ⚠️ A forma `a·(1−t) + b·t` é exata nos extremos por IEEE-754 (em `t = 1` o primeiro
    /// termo é `a·0.0`, exatamente zero para qualquer finito) — é isso que faz o fim do
    /// morph ser `b`, e não *quase* `b`.
    #[test]
    fn the_quantities_fade_and_blend_one_is_b_to_the_bit() {
        let a = rich([0.0, 0.0], 1.0, [1.0, 0.0, 0.0, 1.0], 0.0);
        let b = rich([10.0, 0.0], 3.0, [0.0, 0.0, 1.0, 0.5], 0.0);
        let mid = morph_stream(&a, &b, &[0.5]);
        match (mid.get("size"), mid.get("tint")) {
            (Some(Column::Vec2(s)), Some(Column::Vec4(t))) => {
                assert_eq!(s[0], [2.0, 2.0], "o tamanho desvanece");
                assert_eq!(t[0], [0.5, 0.0, 0.5, 0.75], "a cor desvanece, alfa incluso");
            }
            _ => panic!("size/tint têm de sobreviver com o variant certo"),
        }
        let end = morph_stream(&a, &b, &[1.0]);
        for name in ["size", "tint"] {
            assert_eq!(
                end.get(name),
                b.get(name),
                "`{name}` em blend 1 tem de ser `b` AO BIT"
            );
        }
    }

    /// **UMA COLUNA QUE SÓ UM LADO CARREGA É PROPAGADA, NÃO DESCARTADA.**
    ///
    /// Descartar é o defeito; e inventar a identidade do lado ausente escolheria por um
    /// artista que não pediu nada.
    #[test]
    fn a_column_only_one_side_carries_is_propagated() {
        let a = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
            .with("geometry_id", Column::Scalar(vec![5.0; 2]));
        let b = Stream::new(2)
            .with("P", Column::Vec2(vec![[1.0, 0.0]; 2]))
            .with("life", Column::Scalar(vec![9.0; 2]));
        let out = morph_stream(&a, &b, &[0.5]);
        assert_eq!(scalar0(&out, "geometry_id"), 5.0, "a coluna só de `a` fica");
        assert_eq!(scalar0(&out, "life"), 9.0, "a coluna só de `b` também");
    }

    /// **E A TROCA DE IDENTIDADE É POR ELEMENTO** — um blend escalonado vira cada linha no
    /// seu próprio tempo, como a posição dela já fazia.
    #[test]
    fn the_identity_switches_per_element_like_the_position_does() {
        let a = rich([0.0, 0.0], 1.0, [1.0; 4], 0.0);
        let b = rich([10.0, 0.0], 1.0, [1.0; 4], 7.0);
        let out = morph_stream(&a, &b, &[0.2, 0.8]);
        match out.get("texture_id") {
            Some(Column::Scalar(v)) => assert_eq!(
                (v[0], v[1]),
                (0.0, 7.0),
                "cada elemento troca no seu próprio meio-caminho"
            ),
            _ => panic!("texture_id"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
