//! Os gates do `value.smooth` — a lei da janela, as três formas de peso, e a
//! MEDIÇÃO que decide se um knob de `iterations` acrescenta alguma coisa.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// **O box blur CONGELADO** — exactamente como shipava antes deste grupo, sem
/// pesos: soma crua dividida por `2r+1`.
///
/// ⚠️ Ele vive sob `cfg(test)` porque um `fn` sem chamador no produto é uma
/// SEGUNDA resposta esperando alguém chamá-la; aqui ele é o oráculo, e só isso.
fn frozen_box(field: &[f32], radius: usize) -> Vec<f32> {
    let n = field.len();
    if radius == 0 || n == 0 {
        return field.to_vec();
    }
    let r = radius as isize;
    let last = n as isize - 1;
    (0..n)
        .map(|i| {
            let mut sum = 0.0f32;
            let mut k = i as isize - r;
            let hi = i as isize + r;
            while k <= hi {
                let idx = k.clamp(0, last) as usize;
                sum += field[idx];
                k += 1;
            }
            sum / (2 * radius + 1) as f32
        })
        .collect()
}

/// Um campo com estrutura de sobra para um filtro morder: um degrau, um pico
/// isolado e um dente-de-serra. Um campo liso deixaria qualquer peso concordar
/// com qualquer outro — a fixture TEM de conter o fenómeno.
fn structured_field(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let step = if i > n / 2 { 4.0 } else { 0.0 };
            let spike = if i == n / 4 { 9.0 } else { 0.0 };
            let saw = (i % 7) as f32 * 0.3;
            step + spike + saw
        })
        .collect()
}

/// **O peso `Box` é BIT a BIT o filtro que já shipava** — o controle da wave. Os
/// dois pesos novos entram ao lado dele, nunca por dentro.
#[test]
fn the_box_weight_is_bit_identical_to_the_filter_that_shipped() {
    for n in [1usize, 5, 33, 128] {
        let f = structured_field(n);
        for r in [0usize, 1, 2, 5, 17, 200] {
            let got = smooth(&f, r, Weight::Box, Window::Centered);
            let want = frozen_box(&f, r);
            assert_eq!(got.len(), want.len());
            for (i, (a, b)) in got.iter().zip(want.iter()).enumerate() {
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "n={n} r={r} i={i}: o Box moveu-se ({a} vs {b})"
                );
            }
        }
    }
}

/// **Radius 0 é um passthrough bit-exato em TODOS os pesos** — o neutro não é do
/// `weight`, é do `radius`: sem janela não há o que pesar.
#[test]
fn radius_zero_is_the_identity_under_every_weight() {
    let f = vec![3.0, -1.0, 7.5, 2.0];
    for w in [Weight::Box, Weight::Triangle, Weight::Smooth] {
        assert_eq!(smooth(&f, 0, w, Window::Centered), f, "{w:?}");
    }
}

/// **A weight de FÁBRICA é `Box`** — e este gate não menciona nenhum outro
/// número, que é o que o torna um teste do DEFAULT e não da tabela.
#[test]
fn the_factory_weight_is_the_plain_mean() {
    assert_eq!(
        Weight::from_param(MANIFEST.param_default("weight").unwrap()),
        Weight::Box
    );
    assert_eq!(MANIFEST.param_default("radius").unwrap(), 0.0);
}

/// **A spike is spread across its window** — a single tall value in a flat
/// field drops and its neighbours rise. A BOX blur spreads it to a flat
/// PLATEAU (not a rounded peak — that is what the other two weights are for),
/// and with zero boundaries the total is conserved (mass is moved, never
/// created). `[0,0,9,0,0]` at radius 1 becomes `[0,3,3,3,0]`.
#[test]
fn a_spike_is_spread_across_its_window() {
    let f = vec![0.0, 0.0, 9.0, 0.0, 0.0];
    let out = smooth(&f, 1, Weight::Box, Window::Centered);
    assert!(out[2] < 9.0, "the spike drops");
    assert!(out[1] > 0.0 && out[3] > 0.0, "the neighbours rise");
    assert_eq!(out[1], out[2], "the window becomes a plateau, not a peak");
    assert_eq!(out[2], out[3], "the window becomes a plateau, not a peak");
    let before: f32 = f.iter().sum();
    let after: f32 = out.iter().sum();
    assert!(
        (before - after).abs() < 1e-5,
        "mass conserved (zero boundaries)"
    );
}

/// **Os três pesos desenham três perfis DIFERENTES sobre o mesmo pico** — e a
/// diferença é a que os nomes prometem: o Box faz um PLATÔ, o Triangle uma
/// rampa recta e o Smooth um S.
///
/// ⚠️ **A comparação é no OMBRO, e a primeira versão deste gate mediu o CENTRO
/// e reprovou sobre código correcto.** Triangle e Smooth têm a MESMA soma de
/// pesos, e isso não é acaso: os dois perfis são partições da unidade em torno
/// do meio (`t + (1−t) = 1` e `s(t) + s(1−t) = 1`), então os taps emparelham e
/// `Σw` sai igual nos dois. No centro de um pico isolado o valor é
/// `9·w(0)/Σw` — e com `w(0)` também igual, os dois respondem **exactamente o
/// mesmo número**. Onde eles divergem é onde a curva difere da recta: o ombro.
/// Medido a `r = 3` sobre um pico de 9: Box `[1,286 …]` chato · Triangle
/// `[0,563 · 1,125 · 1,688 · 2,25]` · Smooth `[0,352 · 1,125 · 1,898 · 2,25]`.
#[test]
fn the_three_weights_draw_three_different_profiles() {
    let f = vec![0.0, 0.0, 0.0, 0.0, 9.0, 0.0, 0.0, 0.0, 0.0];
    let (c, r) = (4usize, 3usize);
    let b = smooth(&f, r, Weight::Box, Window::Centered);
    let t = smooth(&f, r, Weight::Triangle, Window::Centered);
    let s = smooth(&f, r, Weight::Smooth, Window::Centered);
    // O Box é CHATO dentro da janela inteira — é o que "box" quer dizer.
    for d in 1..=r {
        assert_eq!(b[c], b[c - d], "Box: platô em -{d}");
        assert_eq!(b[c], b[c + d], "Box: platô em +{d}");
    }
    // Os outros dois têm crista, e ela desce monotonicamente.
    for (name, o) in [("Triangle", &t), ("Smooth", &s)] {
        for d in 1..=r {
            assert!(o[c] > o[c + d], "{name}: o centro é o máximo (+{d})");
            assert!(o[c + d - 1] > o[c + d], "{name}: desce em +{d}");
        }
        assert!(o[c] > b[c] + 0.5, "{name}: a crista passa do platô do Box");
    }
    // ⚠️ E é no OMBRO que Triangle e Smooth se separam — o `d = r` mede
    // 0,563 contra 0,352, enquanto o centro e o meio COINCIDEM nos dois.
    assert!(
        (t[c + r] - s[c + r]).abs() > 0.15,
        "Triangle {} e Smooth {} têm de divergir no ombro",
        t[c + r],
        s[c + r]
    );
    assert!(
        (t[c] - s[c]).abs() < 1e-5,
        "e coincidir no centro — a partição da unidade que este doc explica"
    );
}

/// **Um campo constante sobrevive a qualquer peso e a qualquer raio** — a média
/// ponderada de valores iguais é esse valor, e o edge-extend mantém-no.
#[test]
fn a_constant_field_is_unchanged() {
    let f = vec![4.0; 7];
    for w in [Weight::Box, Weight::Triangle, Weight::Smooth] {
        for r in [0usize, 1, 3, 20] {
            let out = smooth(&f, r, w, Window::Centered);
            for (i, v) in out.iter().enumerate() {
                assert!((v - 4.0).abs() < 1e-5, "{w:?} r={r} i={i}: {v}");
            }
        }
    }
}

/// **A saída é finita e preserva o comprimento** para qualquer campo, peso e
/// raio — inclusive um raio maior que o campo (o edge-extend clampa).
#[test]
fn output_is_finite_and_length_preserving() {
    let f = vec![-3.0, 100.0, -50.0, 0.0, 8.0];
    for w in [Weight::Box, Weight::Triangle, Weight::Smooth] {
        for r in [0usize, 1, 2, 5, 100] {
            let out = smooth(&f, r, w, Window::Centered);
            assert_eq!(out.len(), f.len(), "{w:?} r={r}");
            assert!(out.iter().all(|x| x.is_finite()), "{w:?} r={r}");
        }
    }
}

/// **Nenhum tap da janela pesa zero, nem no aro** — um perfil que zerasse na
/// borda desperdiçaria dois taps e faria o `radius` mentir sobre o próprio
/// alcance.
#[test]
fn no_tap_in_the_window_weighs_nothing() {
    for r in [1u32, 2, 7, 64] {
        for w in [Weight::Box, Weight::Triangle, Weight::Smooth] {
            for d in 0..=r {
                assert!(tap_weight(w, d, r) > 0.0, "{w:?} r={r} d={d}");
            }
            // E o centro é sempre o mais pesado (ou empata, no Box).
            assert!(tap_weight(w, 0, r) >= tap_weight(w, r, r));
        }
    }
}

/// **N passes de box, o knob que a referência ship'a** — o oráculo da medição
/// abaixo. Vive no teste porque o produto NÃO o oferece.
fn box_passes(field: &[f32], radius: usize, passes: usize) -> Vec<f32> {
    let mut cur = field.to_vec();
    for _ in 0..passes {
        cur = smooth(&cur, radius, Weight::Box, Window::Centered);
    }
    cur
}

/// O maior desvio entre dois campos, em fracção da amplitude do campo original.
fn worst_rel(a: &[f32], b: &[f32], range: f32) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max)
        / range
}

/// **UMA janela larga com peso `Smooth` alcança o que uma PILHA de passes de box
/// alcança** — a medição que fecha a linha `iterations` da folha 15 sem um knob
/// novo.
///
/// A folha marcava `iterations`/`passes` como P1 e supunha que a forma do peso
/// *cavalgava* nele (*"3 passes de box ≈ gaussiana"*). A relação é a INVERSA e é
/// aritmética: `N` convoluções de um box são uma B-spline de grau `N−1`, isto é,
/// um PESO — logo o peso é o parâmetro geral e a contagem de passes é um caso
/// dele. E o peso tem a propriedade que a contagem não pode ter: ele corre em UM
/// passe, logo **no device** (o kernel é um dispatch, e `N` passes precisariam de
/// `N` — cada um lendo o resultado inteiro do anterior).
///
/// ⚠️ **O `ParamHardMax` do raio é o que torna isto verdade**, e é por isso que
/// ele entrou nesta wave: três passes de raio `r` alcançam `3r`, então sem um
/// teto digitável acima do slider a equivalência seria inalcançável na UI.
///
/// ⚠️ **E a medição deu uma LEI, não só um número:** o raio que melhor reproduz
/// `N` passes de raio `r` é **o SUPORTE deles, `r·N`** — medido em `2×3 → 6`,
/// `4×3 → 12`, `6×4 → 23`, `8×3 → 25`. É esse o oráculo aqui (um raio que caísse
/// noutro lugar seria coincidência), e o erro residual fica em **1-2% da
/// amplitude**.
#[test]
fn a_wide_smooth_window_reaches_what_repeated_box_passes_reach() {
    let f = structured_field(160);
    let range = f.iter().fold(0.0f32, |a, b| a.max(*b)) - f.iter().fold(f32::MAX, |a, b| a.min(*b));
    for (r, passes) in [(2usize, 3usize), (4, 3), (6, 4), (8, 3)] {
        let want = box_passes(&f, r, passes);
        // Varre o raio de UMA janela `Smooth` e fica com o melhor.
        let (best, at) = (1..=(4 * r * passes))
            .map(|rr| {
                (
                    worst_rel(
                        &smooth(&f, rr, Weight::Smooth, Window::Centered),
                        &want,
                        range,
                    ),
                    rr,
                )
            })
            .fold((f32::MAX, 0usize), |a, b| if b.0 < a.0 { b } else { a });
        assert!(
            best < 0.025,
            "r={r} passes={passes}: a melhor janela Smooth erra {best:.4} da amplitude — \
             se isto subir, a refutação do knob `iterations` deixou de valer"
        );
        let support = r * passes;
        assert!(
            at.abs_diff(support) <= 2,
            "r={r} passes={passes}: o melhor raio foi {at} e o SUPORTE é {support} — \
             a equivalência tem de ser previsível, não uma coincidência de varredura"
        );
    }
}

/// **E o CONTROLE: uma janela de peso BOX NÃO alcança** — sem ele o gate acima
/// seria satisfeito por qualquer filtro que borrasse o suficiente, e a medição
/// não diria nada sobre a FORMA.
///
/// ⚠️ **A separação encolhe com o suporte, e o número está aqui em vez de ser
/// escondido numa barra generosa:** no suporte 6 o Box erra **5,3×** o que o
/// Smooth erra (0,0568 contra 0,0107); no suporte 24 ele erra **1,1×** (0,0230
/// contra 0,0208). É o esperado — quanto mais se borra, menos a forma do núcleo
/// importa —, então o gate mede onde a afirmação de facto vale, e a cauda fica
/// escrita em vez de gateada.
#[test]
fn a_box_window_does_not_reach_it_and_that_is_why_the_shape_matters() {
    let f = structured_field(160);
    let range = f.iter().fold(0.0f32, |a, b| a.max(*b)) - f.iter().fold(f32::MAX, |a, b| a.min(*b));
    let best = |w: Weight, want: &[f32]| {
        (1..=48)
            .map(|rr| worst_rel(&smooth(&f, rr, w, Window::Centered), want, range))
            .fold(f32::MAX, f32::min)
    };
    let want = box_passes(&f, 2, 3); // suporte 6: onde a forma ainda decide
    let (b, s) = (best(Weight::Box, &want), best(Weight::Smooth, &want));
    assert!(
        b > 3.0 * s,
        "no suporte 6 o Box tinha de ficar bem atrás do Smooth (box {b:.4} vs smooth {s:.4})"
    );
}

/// **A sonda que produziu o teto digitável do raio** — ns por tap, e o custo de
/// um campo de driver típico.
#[test]
#[ignore = "sonda: cargo test -p ph2d-node-value-smooth --release -- --ignored --nocapture"]
fn measure_what_a_radius_costs() {
    let n = 10_000usize;
    let f = structured_field(n);
    println!("{:>8} {:>14} {:>12}", "raio", "taps", "ms");
    for r in [1usize, 16, 64, 128, 256, 512] {
        let t0 = std::time::Instant::now();
        let out = smooth(&f, r, Weight::Smooth, Window::Centered);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let taps = (n * (2 * r + 1)) as f64;
        println!("{r:>8} {:>14.0} {ms:>12.3}", taps);
        assert_eq!(out.len(), n);
        println!("        -> {:.3} ns/tap", ms * 1e6 / taps);
    }
}

/// A value source emitting a fixed field, so `value.smooth` can be driven
/// through a real cook (the whole-chain proof, not just the math).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.smooth.test.src"),
    name: "value.smooth.test.src",
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
struct Src(Vec<f32>);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
    }
}

/// End-to-end through the cook: a `[0, 3, 0]` field through radius 1 becomes
/// `[1, 1, 1]` (each window edge-extends and averages three values), length
/// preserved.
#[test]
fn smooths_a_field_through_the_cook() {
    struct Ops(Vec<f32>);
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => {
                    Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                }
                t if t == MANIFEST.id => Some(&ValueSmooth),
                _ => None,
            }
        }
    }
    let ops = Ops(vec![0.0, 3.0, 0.0]);
    let mut g = Graph::new();
    let src = g.add_node("value.smooth.test.src");
    let vs = g.add_node("value.smooth");
    g.set_param(vs, "radius", 1.0);
    // A premissa DECLARADA: este cook é o do peso de fábrica.
    g.set_param(vs, "weight", 0.0);
    g.connect(Edge {
        from: (src, 0),
        to: (vs, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &ops, vs, 0.0).unwrap();
    match out[0].as_stream().get(VALUE_COL).unwrap() {
        Column::Scalar(v) => {
            // i=0: [0,0,3]/3=1 · i=1: [0,3,0]/3=1 · i=2: [3,0,0]/3=1
            assert_eq!(
                v,
                &vec![1.0, 1.0, 1.0],
                "each edge-extended window averages to 1"
            );
        }
        _ => panic!("v"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
    assert_eq!(
        reg.param_hard_max(MANIFEST.id, "radius"),
        Some(RADIUS_LAST_USEFUL)
    );
}

/// ⭐⭐ **UM FILTRO CAUSAL NÃO REAGE ANTES DO ACONTECIMENTO** (doc 89, folha 15 linha 138).
///
/// O oráculo é um DEGRAU: zeros até `k`, uns a partir dali. A propriedade que separa as três
/// janelas não é a forma da resposta — é **de que lado dela o filtro consegue olhar**.
///
/// ⚠️ **A régua tem de ser um IGUAL EXACTO a zero, e não «pequeno»**: a afirmação de
/// causalidade é que a amostra futura **não entra na soma**, não que entra pouco. Um `< ε`
/// passaria com um peso minúsculo mas não-nulo, que é precisamente o defeito.
#[test]
fn the_causal_windows_only_look_one_way() {
    const K: usize = 8;
    let step: Vec<f32> = (0..16).map(|i| f32::from(u8::from(i >= K))).collect();
    let r = 3;

    let left = smooth(&step, r, Weight::Box, Window::Left);
    for (i, v) in left.iter().enumerate().take(K) {
        assert_eq!(
            *v, 0.0,
            "`Left Half` leu o FUTURO no indice {i}: um filtro causal nao antecipa o degrau"
        );
    }
    assert!(
        left[K] > 0.0,
        "e no degrau ele TEM de reagir: {:?}",
        left[K]
    );

    let right = smooth(&step, r, Weight::Box, Window::Right);
    assert!(
        right[K - 1] > 0.0,
        "`Right Half` olha em frente, logo ANTECIPA o degrau: {:?}",
        right[K - 1]
    );
    for (i, v) in right.iter().enumerate().skip(K) {
        assert_eq!(*v, 1.0, "`Right Half` nao pode ver o passado (indice {i})");
    }

    // ⛔ CONTROLE: a centrada vaza para os DOIS lados — sem isto, um gate que só medisse as
    // meias-janelas passaria mesmo que elas fossem apelidos uma da outra.
    let centred = smooth(&step, r, Weight::Box, Window::Centered);
    assert!(
        centred[K - 1] > 0.0 && centred[K] < 1.0,
        "CONTROLE: a janela centrada le' os dois lados: {:?}",
        (centred[K - 1], centred[K])
    );
    // E as três são a MESMA coisa quando não há janela nenhuma.
    for w in [Window::Centered, Window::Left, Window::Right] {
        assert_eq!(
            smooth(&step, 0, Weight::Box, w),
            step,
            "raio 0 e' a identidade em toda janela"
        );
    }
}
