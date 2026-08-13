//! **O cluster de NOISE herdado pela família de forças** (doc 89 folha 02).
//!
//! ⚠️ **A metade que carrega o peso é a NEUTRA.** Um cluster novo que mudasse o
//! campo com os defaults moveria toda cena já autorada, em silêncio; os gates de
//! presença provam os knobs, e o de ausência prova que eles não custaram nada a
//! quem não os pediu.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte de posições, para o nó ter onde amostrar o campo.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.curl.cluster.src"),
    name: "force.curl.cluster.src",
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
        let p: Vec<[f32; 2]> = (0..24u8)
            .map(|i| [f32::from(i) * 0.31 - 3.0, f32::from(i % 5) * 0.7])
            .collect();
        ctx.emit(Stream::new(24).with("P", Column::Vec2(p)));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&ForceCurl),
            _ => None,
        }
    }
}

/// Cozinha `src → force.curl` com os params dados e devolve a coluna `accel`.
fn accel(params: &[(&str, f32)], t: f64) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("force.curl.cluster.src");
    let n = g.add_node("force.curl");
    g.set_param(n, "strength", 3.0);
    g.set_param(n, "scale", 0.4);
    g.set_param(n, "speed", 0.9);
    g.set_param(n, "octaves", 3.0);
    for (k, v) in params {
        g.set_param(n, *k, *v);
    }
    g.connect(Edge {
        from: (src, 0),
        to: (n, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, n, t).expect("coze")[0]
        .as_stream()
        .get("accel")
    {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **Os defaults do cluster reproduzem o laço CRAVADO, ao bit.**
///
/// `lacunarity = 2` e `roughness = 0.5` eram os literais `freq *= 2.0` e
/// `amp *= 0.5`; `type = fBm` é o ramo sem retificação; o offset é zero e o laço
/// está desligado (a segunda amostra nem é avaliada). Escrever cada um com o
/// próprio default tem de dar exactamente o mesmo campo que não os escrever.
#[test]
fn the_cluster_defaults_reproduce_the_hardcoded_loop_to_the_bit() {
    let bare = accel(&[], 1.25);
    let spelled = accel(
        &[
            ("type", 0.0),
            ("lacunarity", 2.0),
            ("roughness", 0.5),
            ("offset_x", 0.0),
            ("offset_y", 0.0),
            ("loop_period", 0.0),
        ],
        1.25,
    );
    assert_eq!(bare.len(), 24);
    for (a, b) in bare.iter().zip(&spelled) {
        assert_eq!(a[0].to_bits(), b[0].to_bits(), "x");
        assert_eq!(a[1].to_bits(), b[1].to_bits(), "y");
    }
}

/// **Cada knob novo MUDA o campo** — a lei do botão morto, uma vez por knob.
#[test]
fn every_new_knob_moves_the_field() {
    let base = accel(&[], 1.25);
    for (k, v) in [
        ("type", 1.0),
        ("lacunarity", 2.7),
        ("roughness", 0.85),
        ("offset_x", 3.5),
        ("offset_y", -2.25),
        ("loop_period", 4.0),
    ] {
        let moved = accel(&[(k, v)], 1.25);
        let d = base
            .iter()
            .zip(&moved)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .fold(0.0f32, f32::max);
        assert!(d > 1e-3, "`{k} = {v}` nao mudou o campo (max {d})");
    }
}

/// **O laço FECHA, e é a razão de o `loop_period` existir.**
///
/// O campo em `t` e em `t + L` tem de ser o MESMO — e a comparação é contra um
/// CONTROLE sem laço, que no mesmo par de instantes é diferente. Sem o controle o
/// gate ficaria verde sobre um campo que simplesmente não anda.
#[test]
fn the_loop_period_closes_the_field_in_time() {
    let l = 4.0;
    let a = accel(&[("loop_period", l)], 0.7);
    let b = accel(&[("loop_period", l)], 0.7 + f64::from(l));
    let far = a
        .iter()
        .zip(&b)
        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(0.0f32, f32::max);
    assert!(far < 1e-3, "o campo tem de fechar em L, deu {far}");

    let c = accel(&[], 0.7);
    let d = accel(&[], 0.7 + f64::from(l));
    let ctrl = c
        .iter()
        .zip(&d)
        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
        .fold(0.0f32, f32::max);
    assert!(
        ctrl > 1e-2,
        "o CONTROLE (sem laco) tem de DIFERIR nos mesmos instantes, deu {ctrl}"
    );
}

/// **O campo com laço continua DIVERGENCE-FREE** — a razão de este nó existir.
///
/// ⚠️ A mistura dos dois instantes é linear e a curl é linear, então elas comutam;
/// se a mistura acontecesse depois de algo não-linear o laço quebraria a
/// propriedade **em silêncio**, e nenhum gate de posição a veria.
#[test]
fn the_looped_field_is_still_divergence_free() {
    let spec = ph2d_fbm::Spec {
        octaves: 3,
        lacunarity: 2.0,
        roughness: 0.5,
        ty: ph2d_fbm::NoiseType::Fbm,
    };
    let t = ph2d_fbm::loop_times(0.9, 4.0);
    let h = 0.02;
    let mut worst = 0.0f32;
    for k in 0..40u8 {
        let (x, y) = (f32::from(k) * 0.17 - 3.0, f32::from(k % 7) * 0.4);
        let vx = curl_looped(x + h, y, t, 0.9, 0.0, spec, [0.0, 0.0])[0]
            - curl_looped(x - h, y, t, 0.9, 0.0, spec, [0.0, 0.0])[0];
        let vy = curl_looped(x, y + h, t, 0.9, 0.0, spec, [0.0, 0.0])[1]
            - curl_looped(x, y - h, t, 0.9, 0.0, spec, [0.0, 0.0])[1];
        worst = worst.max(((vx + vy) / (2.0 * h)).abs());
    }
    assert!(worst < 0.5, "divergencia do campo com laco: {worst}");
}
