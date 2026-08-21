//! Os gates do `motion.color_array`.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::ParamSpec;

const PAL: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 0.0, 1.0],
];

/// Sem deslocamento nenhum — o degrau 0 da escada.
const NO_OFFSET: &[f32] = &[];

/// The palette cycles by index: com uma paleta de 3 cores, o elemento `i` recebe
/// a cor `i mod 3`. FALSIFIED se todos recebessem uma cor só.
#[test]
fn the_palette_cycles_by_index() {
    let c = cycle(7, &PAL[..3], NO_OFFSET, &Stream::new(7));
    assert_eq!(c[0], PAL[0]);
    assert_eq!(c[1], PAL[1]);
    assert_eq!(c[2], PAL[2]);
    assert_eq!(c[3], PAL[0], "wraps after 3");
    assert_eq!(c[6], PAL[0]);
}

/// `offset` marcha a paleta: 1 desloca cada elemento uma casa.
#[test]
fn offset_marches_the_palette() {
    let base = cycle(4, &PAL, NO_OFFSET, &Stream::new(4));
    let shifted = cycle(4, &PAL, &[1.0], &Stream::new(4));
    assert_eq!(shifted[0], base[1], "element 0 took slot 1");
    assert_eq!(shifted[3], base[0], "element 3 wrapped to slot 0");
}

/// **O COMPRIMENTO DA LISTA É O COMPRIMENTO DO CICLO** — não há um segundo número.
#[test]
fn the_palette_length_is_the_cycle_length() {
    let c = cycle(6, &PAL[..2], NO_OFFSET, &Stream::new(6));
    for col in &c {
        assert!(*col == PAL[0] || *col == PAL[1], "só duas cores: {col:?}");
    }
}

/// **The field masks the palette, and no field is byte-identical** (doc 89 fam. 9, P0).
///
/// The DISCRETE colour node gets the same law as the continuous one — a `field.*`
/// writes `falloff` and the stripes paint only where it reaches. The two halves are
/// asserted with `assert_eq!` on raw bits, not an epsilon: at `f = 1` the lerp is
/// `existing·0 + slot·1`, exactly the slot, and at `f = 0` exactly what was there.
#[test]
fn the_field_masks_the_palette_and_no_field_changes_nothing() {
    let existing = vec![[1.0, 0.0, 0.0, 1.0]; 3];
    let masked = Stream::new(3)
        .with("tint", Column::Vec4(existing.clone()))
        .with("falloff", Column::Scalar(vec![1.0, 0.5, 0.0]));
    let got = cycle(3, &PAL[..3], NO_OFFSET, &masked);
    let bare = cycle(3, &PAL[..3], NO_OFFSET, &Stream::new(3));

    assert_eq!(got[0], bare[0], "full falloff takes the slot EXACTLY");
    assert_eq!(got[2], existing[2], "zero falloff keeps the colour EXACTLY");
    // Half must be strictly between — the half a boolean mask would collapse.
    let (lo, hi) = (
        existing[1][0].min(bare[1][0]),
        existing[1][0].max(bare[1][0]),
    );
    assert!(
        hi - lo > 1e-6 && got[1][0] > lo + 1e-6 && got[1][0] < hi - 1e-6,
        "half falloff must land BETWEEN {lo} and {hi}, got {}",
        got[1][0]
    );

    // …and a stream carrying a colour but NO mask is the substitution of before.
    let no_field = Stream::new(3).with("tint", Column::Vec4(existing));
    assert_eq!(
        cycle(3, &PAL[..3], NO_OFFSET, &no_field),
        bare,
        "absent falloff must write the palette slot bit for bit"
    );
}

// ─── A fiação: fonte + resolver partilhados pelos gates que cozinham ─────────

static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.color_array.test.src"),
    name: "motion.color_array.test.src",
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
        ctx.emit(Stream::new(4).with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        ));
    }
}

/// Uma fonte de VALOR de comprimento escolhido — o que enche a porta `offset`.
static VSRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.color_array.test.vsrc"),
    name: "motion.color_array.test.vsrc",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "n",
            default: 0.0,
        },
        ParamSpec {
            name: "v",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};
struct VSrc;
impl NodeOp for VSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &VSRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        #[expect(clippy::cast_sign_loss, reason = "n é escrito pelo gate, >= 0")]
        #[expect(clippy::cast_possible_truncation, reason = "contagens pequenas")]
        let n = ctx.param("n").round() as usize;
        let base = ctx.param("v");
        // Um por elemento, ASCENDENTE, para que cada peça peça uma cor diferente.
        #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
        let col: Vec<f32> = (0..n).map(|k| base + k as f32).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(col)));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == VSRC.id => Some(&VSrc),
            t if t == MANIFEST.id => Some(&MotionColorArray),
            _ => None,
        }
    }
}

/// Cozinha a cena `src(4) → color_array`, com a paleta `pal` e, se `offset` for
/// `Some((n, v))`, uma fonte de valor de `n` linhas começando em `v`.
fn cook_scene(pal: &[[f32; 4]], offset: Option<(f32, f32)>) -> Vec<[f32; 4]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.color_array.test.src");
    let ca = g.add_node("motion.color_array");
    g.set_text_param(ca, PALETTE_KEY, ph2d_color::serialize_palette(pal));
    g.connect(Edge {
        from: (src, 0),
        to: (ca, 0),
        delayed: false,
    })
    .unwrap();
    if let Some((n, v)) = offset {
        let vs = g.add_node("motion.color_array.test.vsrc");
        g.set_param(vs, "n", n);
        g.set_param(vs, "v", v);
        g.connect(Edge {
            from: (vs, 0),
            to: (ca, 1),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, ca, 0.0).unwrap();
    let s = out[0].as_stream();
    assert!(s.get("P").is_some(), "geometry passes through");
    match s.get("tint").unwrap() {
        Column::Vec4(v) => {
            assert_eq!(v.len(), 4, "tint at full count");
            v.clone()
        }
        _ => panic!("tint"),
    }
}

/// Deterministic + cooks through the registry: writes the `tint` column at the full
/// count and passes geometry through.
#[test]
fn registers_and_colours_through_the_cook() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
    // A TWO-colour palette: the count is the list's length, not a slider that caps
    // a longer fixed list — so authoring "two colours" is authoring two colours.
    let v = cook_scene(&[[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]], None);
    assert_eq!(v[0], v[2], "slot 0 repeats at index 2 (2-colour cycle)");
    assert_ne!(v[0], v[1], "adjacent differ");
}

/// **UM CAMPO POR-INSTÂNCIA CHEGA A CADA PEÇA — a célula da folha 09.**
///
/// A cena liga um `offset` de QUATRO linhas ascendentes `[0,1,2,3]` a quatro
/// peças e uma paleta de quatro cores. Somado ao próprio índice, o resultado é
/// `(i + i) mod 4` = `[0, 2, 0, 2]` — duas cores alternadas em pares.
///
/// ⚠️ **O oráculo é escolhido para não ser o da lei antiga.** Com `.first()` (o
/// que shipava) o deslocamento seria `0` para todos e a saída `[0,1,2,3]`; o gate
/// exige explicitamente que NÃO seja isso, senão um regresso a `.first()` passava
/// verde com uma cena que só olha "as cores variam".
#[test]
fn a_per_instance_offset_field_reaches_every_element() {
    let got = cook_scene(&PAL, Some((4.0, 0.0)));
    assert_eq!(
        got,
        vec![PAL[0], PAL[2], PAL[0], PAL[2]],
        "(i + offset_i) mod 4 com offset = [0,1,2,3]"
    );
    let discarded = cook_scene(&PAL, None);
    assert_ne!(
        got, discarded,
        "se o campo fosse descartado (a lei `.first()`), isto seria igual"
    );
}

/// **OS DOIS PRIMEIROS DEGRAUS DA ESCADA SÃO BYTE-IDÊNTICOS AO QUE SHIPAVA.**
///
/// Ausente ⇒ 0. Comprimento 1 ⇒ DIFUNDIDO — e o degrau da difusão é o que o
/// `.first()` já fazia por acidente, então ele tem de sair igual ao dia anterior.
#[test]
fn the_absent_and_length_one_rungs_are_what_shipped() {
    let bare = cook_scene(&PAL, None);
    assert_eq!(bare, vec![PAL[0], PAL[1], PAL[2], PAL[3]], "ausente ⇒ 0");
    // Comprimento 1, valor 1 ⇒ toda peça anda uma casa.
    let broadcast = cook_scene(&PAL, Some((1.0, 1.0)));
    assert_eq!(
        broadcast,
        vec![PAL[1], PAL[2], PAL[3], PAL[0]],
        "um valor global marcha a paleta inteira, não só o elemento 0"
    );
}

/// **UM DESLOCAMENTO NEGATIVO ANDA PARA TRÁS, e não sai do vetor.**
///
/// ⚠️ `rem_euclid`, nunca `%`: em Rust `(0 - 1) % 4` é `-1`, que como índice é
/// pânico. O braço `if (ca_k < 0)` do WGSL existe pela mesma razão.
#[test]
fn a_negative_offset_wraps_backwards_instead_of_leaving_the_list() {
    let got = cook_scene(&PAL, Some((1.0, -1.0)));
    assert_eq!(got, vec![PAL[3], PAL[0], PAL[1], PAL[2]]);
}

/// **O ARREDONDAMENTO É MEIO-PARA-LONGE-DO-ZERO**, o do Rust — e é o que o
/// `ca_round` do WGSL replica.
///
/// FALSIFICADO por um `floor`/`trunc` ou pelo `round` meio-para-o-par do device:
/// em `0.5` o meio-par daria `0` e este daria `1`, e as duas cores existem na
/// paleta, por isso a divergência seria visível e silenciosa.
#[test]
fn the_offset_rounds_half_away_from_zero_like_the_device() {
    assert_eq!(offset_at(&[0.5], 0), 1, "0.5 → 1 (meio-par daria 0)");
    assert_eq!(offset_at(&[1.5], 0), 2);
    assert_eq!(offset_at(&[-0.5], 0), -1, "e para o outro lado também");
    assert_eq!(offset_at(&[0.4], 0), 0);
}

// ─── O device ───────────────────────────────────────────────────────────────

/// **O NÓ REGISTA KERNEL E LUT** — a célula da folha 09 que dizia *"0
/// `register_gpu_kernel`, contra 1 em cada um dos outros três da família"*.
///
/// ⚠️ Sem isto o grafo inteiro cai para a CPU quando este nó aparece nele: o
/// custo não é o deste nó, é o da cadeia toda.
#[test]
fn the_node_lowers_to_the_device_like_the_rest_of_the_colour_family() {
    use ph2d_nodegraph::gpu::KernelResolver;
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(
        reg.gpu_kernel(MANIFEST.id).is_some(),
        "o quarto nó de cor tem de ter kernel"
    );
    let luts = reg.luts(MANIFEST.id);
    assert_eq!(luts.len(), 1, "uma LUT: a lista de cores");
    assert_eq!(luts[0].text_key, PALETTE_KEY, "a MESMA string que a CPU lê");
    assert_eq!(luts[0].resolution, palette::LUT_LEN);
}

/// **O CORPO LÊ O BUFFER DIRECTO, NUNCA O `_sample(t)`.**
///
/// ⛔ Esta é a recusa medida que o `value.pattern` deixou escrita: o acessor que o
/// gerador emite LERPA entre vizinhos, e duas cores de uma paleta não têm nada
/// entre si. Um `sample` num `t` derivado do índice passaria por `(k/last)·last`,
/// que em `f32` não devolve `k` para todo par — a cor `k` sairia misturada com a
/// `k±1` por ~1e-7, invisível num teste de olho e fatal num gate de paridade.
#[test]
fn the_kernel_indexes_the_lut_and_never_interpolates_it() {
    assert!(
        GPU_KERNEL.wgsl.contains("lut_ca_pal[ca_b]"),
        "o corpo tem de indexar o buffer"
    );
    assert!(
        !GPU_KERNEL.wgsl.contains("_sample("),
        "⛔ o acessor interpolado não pode aparecer neste corpo"
    );
    assert!(
        GPU_KERNEL.wgsl.contains("lut_ca_pal[0]"),
        "a contagem vem do slot 0, não de um `arrayLength` (que é a capacidade)"
    );
}

/// **AS TRÊS LIGAÇÕES DE COLUNA, E O `offset` É O LEITOR DE DIFUSÃO.**
///
/// ⚠️ `ReadBroadcast` é o que dá ao device a MESMA escada `0/1/n` da CPU — e o que
/// faz um campo de qualquer OUTRO comprimento ser recusado no cook
/// (`BroadcastLengthMismatch`) em vez de pintar metade do conjunto errado. Um
/// `Read` simples ali seria a divergência silenciosa entre os dois caminhos.
#[test]
fn the_bindings_mirror_the_cpus_ladder_and_mask() {
    let by = |c: &str| {
        GPU_KERNEL
            .bindings
            .iter()
            .find(|b| b.column == c)
            .unwrap_or_else(|| panic!("falta a ligação de `{c}`"))
    };
    assert_eq!(by("tint").access, ColumnAccess::ReadWrite);
    assert_eq!(by("tint").identity, [1.0; 4], "ausente = branco opaco");
    assert_eq!(by("falloff").access, ColumnAccess::Read);
    assert_eq!(by("falloff").identity, [1.0; 4], "ausente = sem máscara");
    let off = by(VALUE_COL);
    assert_eq!(
        off.access,
        ColumnAccess::ReadBroadcast,
        "a escada 0/1/n do device"
    );
    assert_eq!(off.port, 1, "o `offset` é a segunda porta");
}
