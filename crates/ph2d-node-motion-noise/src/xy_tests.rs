//! Os gates do canal **`Position XY`** — o `Separate Channels` da folha 06.
//!
//! ⚠️ Arquivo próprio por TETO DE LOC (HR-18): o `tests.rs` já mede 616 linhas.
//!
//! ## A régua tem de ser a CORRELAÇÃO, e não "houve deslocamento"
//!
//! O modo de falha desta feature não é ela não mexer — é ela mexer **igual nos dois
//! eixos**. Um mesmo campo escrito em X e em Y põe toda peça a andar sobre uma
//! **diagonal a 45°**: um segmento, não um vaguear. Isso passa em qualquer gate que
//! meça excursão, contagem de linhas ou "o eixo Y mudou", e é exactamente o que a
//! referência resolve. ⇒ o oráculo é o coeficiente de correlação entre os dois
//! eixos, com o **controle** no deslocamento zero (que tem de dar `1` exacto).

use super::*;

/// A barra da decorrelação — ver o doc de
/// [`the_xy_channel_writes_both_axes_and_they_are_decorrelated`].
const MAX_R: f32 = 0.5;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId as GNodeId};

/// Uma grelha 24×24 — grande o bastante para uma correlação querer dizer alguma
/// coisa (a fixture de 9 peças do `tests.rs` não serve para isto).
static GRID: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.noise.test.xygrid"),
    name: "motion.noise.test.xygrid",
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
const SIDE: usize = 24;
struct Grid;
impl NodeOp for Grid {
    fn manifest(&self) -> &'static NodeManifest {
        &GRID
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p: Vec<[f32; 2]> = (0..SIDE)
            .flat_map(|r| (0..SIDE).map(move |c| [c as f32 * 0.35, r as f32 * 0.35]))
            .collect();
        ctx.emit(Stream::new(SIDE * SIDE).with("P", Column::Vec2(p)));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == GRID.id => Some(&Grid),
            t if t == MANIFEST.id => {
                static N: MotionNoise = MotionNoise;
                Some(&N)
            }
            _ => None,
        }
    }
}

/// As posições cozidas, e as de origem, para o deslocamento sair por subtração.
fn cooked(setup: impl FnOnce(&mut Graph, GNodeId)) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut g = Graph::new();
    let src = g.add_node("motion.noise.test.xygrid");
    let ns = g.add_node("motion.noise");
    g.connect(Edge {
        from: (src, 0),
        to: (ns, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(ns, "amplitude", 1.0);
    g.set_param(ns, "scale", 0.25);
    setup(&mut g, ns);
    let mut cook = Cook::new();
    let base = match cook.cook(&g, &Ops, src, 0.5).unwrap()[0]
        .as_stream()
        .get("P")
        .unwrap()
    {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    };
    let out = match cook.cook(&g, &Ops, ns, 0.5).unwrap()[0]
        .as_stream()
        .get("P")
        .unwrap()
    {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    };
    (base, out)
}

/// Os dois deslocamentos, elemento a elemento.
fn axis_deltas(setup: impl FnOnce(&mut Graph, GNodeId)) -> (Vec<f32>, Vec<f32>) {
    let (base, out) = cooked(setup);
    let dx = base.iter().zip(&out).map(|(b, o)| o[0] - b[0]).collect();
    let dy = base.iter().zip(&out).map(|(b, o)| o[1] - b[1]).collect();
    (dx, dy)
}

/// O coeficiente de correlação de Pearson. `1` = os dois eixos são o mesmo campo.
pub(crate) fn pearson(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() as f32;
    let ma = a.iter().sum::<f32>() / n;
    let mb = b.iter().sum::<f32>() / n;
    let mut cov = 0.0;
    let mut va = 0.0;
    let mut vb = 0.0;
    for (x, y) in a.iter().zip(b) {
        cov += (x - ma) * (y - mb);
        va += (x - ma) * (x - ma);
        vb += (y - mb) * (y - mb);
    }
    if va <= 0.0 || vb <= 0.0 {
        return 0.0;
    }
    cov / (va * vb).sqrt()
}

/// **OS DOIS EIXOS ANDAM, E SÃO CAMPOS DIFERENTES.**
///
/// ⚠️ As duas metades são precisas. Sem a primeira, um canal que escrevesse zero num
/// dos eixos passaria na correlação (um campo constante correlaciona-se com nada).
/// Sem a segunda, o mesmo campo nos dois eixos passaria em tudo — e é o defeito real.
///
/// ## ⚠️ A barra fica longe de **1**, e não colada em **0** — e o número é medido
///
/// | deslocamento | `r` |
/// |---|---|
/// | `0` (o controle, gate irmão) | **1,000** |
/// | [`AXIS_SEED_OFFSET`], `scale = 0,25` | **0,120** |
///
/// O resíduo de `0,12` **não é acoplamento**, e a sonda [`probe_r_vs_scale`] mostra-o:
/// ele cai monotonamente com a FINURA do campo — `0,120 · 0,067 · 0,044 · 0,025 ·
/// 0,009` para `scale` `0,25 · 0,5 · 1 · 2 · 4`. Um acoplamento real não se dissolve
/// ao encolher a feição; ruído de amostragem sim.
///
/// A razão é que um campo COERENTE tem muito menos amostras independentes do que
/// elementos: peças vizinhas leem pontos vizinhos, então o `n` efectivo é o número de
/// FEIÇÕES, não as 576 peças. ⇒ **uma barra em `0,15` mediria o tamanho da grelha e
/// da feição, não o código** — e reprovaria numa fixture legítima mais grosseira. A
/// barra é `0,5`: metade do caminho até o defeito, que vale exactamente `1`.
#[test]
fn the_xy_channel_writes_both_axes_and_they_are_decorrelated() {
    let (dx, dy) = axis_deltas(|g, ns| g.set_param(ns, "channel", CH_XY as f32));
    let swing = |v: &[f32]| {
        v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
            - v.iter().fold(f32::INFINITY, |m, x| m.min(*x))
    };
    assert!(swing(&dx) > 0.5, "o eixo X anda: {}", swing(&dx));
    assert!(swing(&dy) > 0.5, "o eixo Y anda: {}", swing(&dy));
    let r = pearson(&dx, &dy);
    assert!(
        r.abs() < MAX_R,
        "os dois eixos tem de ser campos DIFERENTES, correlacao {r}"
    );
}

/// **O CONTROLE: com deslocamento ZERO os dois eixos são o MESMO campo.**
///
/// ⚠️ É esta metade que impede o gate acima de passar por vácuo. Ela não mede o
/// produto — mede o que o produto seria **sem** o [`AXIS_SEED_OFFSET`] —, e por isso
/// espelha a lei em vez de a cozer: o `eval` não aceita um deslocamento por param
/// (de propósito: não é um knob, é uma constante de desenho).
#[test]
fn without_the_offset_the_two_axes_would_be_one_field_at_45_degrees() {
    // Um eixo com o seed do artista, o outro com o MESMO seed — a ablação.
    let (dx, _) = axis_deltas(|g, ns| {
        g.set_param(ns, "channel", 0.0); // X
        g.set_param(ns, "seed", 0.0);
    });
    let (_, dy) = axis_deltas(|g, ns| {
        g.set_param(ns, "channel", 1.0); // Y
        g.set_param(ns, "seed", 0.0);
    });
    let r = pearson(&dx, &dy);
    assert!(
        (r - 1.0).abs() < 1e-4,
        "o mesmo seed nos dois eixos e' UM campo (r = {r}) — e' isso que o deslocamento evita"
    );
}

/// **OS QUATRO CANAIS DE SEMPRE FICAM BYTE-IDÊNTICOS.**
///
/// O canal novo é apendado; nenhum documento de ontem pode mudar de aparência.
#[test]
fn the_four_original_channels_are_untouched_by_the_new_one() {
    for ch in 0..=3 {
        let (base, out) = cooked(|g, ns| g.set_param(ns, "channel", ch as f32));
        // X e Y só se movem nos canais 0 e 1; nos outros o `P` passa verbatim.
        let moved_x = base
            .iter()
            .zip(&out)
            .any(|(b, o)| (o[0] - b[0]).abs() > 0.0);
        let moved_y = base
            .iter()
            .zip(&out)
            .any(|(b, o)| (o[1] - b[1]).abs() > 0.0);
        assert_eq!(moved_x, ch == 0, "canal {ch}: X");
        assert_eq!(moved_y, ch == 1, "canal {ch}: Y");
    }
}

/// **O WGSL CARREGA O MESMO DESLOCAMENTO QUE O RUST** — o gémeo literal.
///
/// ⚠️ O corpo do kernel é uma **string**, então ele não vê a const do Rust. Este é o
/// mesmo perigo do guarda do `Divide` no `motion.drive`: dois sítios a dizer o mesmo
/// número, e nada a obrigá-los. Aqui a obrigação é este gate — ele deriva a agulha
/// da const e procura-a no kernel, então mover a const sem mover o WGSL **não
/// compila verde**.
#[test]
fn the_wgsl_carries_the_same_axis_offset_as_the_rust() {
    let needle = format!(", {AXIS_SEED_OFFSET})");
    assert!(
        kernel::pxy_wgsl().contains(&needle),
        "o WGSL do variant XY tem de deslocar por {AXIS_SEED_OFFSET}: {}",
        kernel::pxy_wgsl()
    );
    // O controle: a agulha é específica. Sem isto, um `contains` de algo trivial
    // passaria.
    assert!(!kernel::pxy_wgsl().contains(", 7918)"));
}

/// **O PAINEL OFERECE O CANAL NOVO** — a costura entre a lei e o dropdown.
#[test]
fn the_channel_dropdown_offers_the_two_axis_entry() {
    let row = PARAM_HINTS
        .iter()
        .find(|h| h.param == "channel")
        .expect("a linha do canal");
    let ph2d_node_registry::ParamWidget::Enum { labels } = row.widget else {
        panic!("o canal é um seletor nomeado")
    };
    assert_eq!(labels.len() as i32 - 1, CH_XY, "a ultima etiqueta é o XY");
    assert_eq!(labels[CH_XY as usize], "Position XY");
    assert!((row.max - CH_XY as f32).abs() < f32::EPSILON);
}

/// **SONDA: `r` em função da FINURA do campo** — o instrumento que produziu a
/// leitura acima, e a razão de a barra não estar colada em zero.
///
/// `cargo test -p ph2d-node-motion-noise probe_r_vs_scale -- --ignored --nocapture`
#[test]
#[ignore = "sonda, não um gate"]
fn probe_r_vs_scale() {
    for sc in [0.25f32, 0.5, 1.0, 2.0, 4.0] {
        let (dx, dy) = axis_deltas(|g, ns| {
            g.set_param(ns, "channel", CH_XY as f32);
            g.set_param(ns, "scale", sc);
        });
        println!("scale {sc:>5} -> r = {:+.4}", pearson(&dx, &dy));
    }
}
