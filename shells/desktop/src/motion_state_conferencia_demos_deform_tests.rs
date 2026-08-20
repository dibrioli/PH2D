//! Os gates da cena `=68` — os deformadores.
//!
//! ⚠️ **Cada par separa, e o oráculo mede a grandeza que a banda anuncia.** Uma cena de
//! deformadores é onde *"as duas listas diferem"* mais engana: qualquer knob mexe em tudo.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// As posições de cada banda, **relativas ao centro dela** (o `motion.move` do layout sai da
/// conta, então os oráculos falam de forma e não de onde a banda está).
fn bands() -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_deform_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 10, "cinco pares");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            let st = cook.cook(&doc.graph, &reg, *s, 0.0).expect("coze")[0].as_stream();
            let p = match st.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            };
            let n = p.len() as f32;
            let c = p
                .iter()
                .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
            let (cx, cy) = (c[0] / n, c[1] / n);
            p.iter().map(|q| [q[0] - cx, q[1] - cy]).collect()
        })
        .collect()
}

/// A extensão da banda em cada eixo.
fn extent(b: &[[f32; 2]]) -> [f32; 2] {
    let f = |k: usize| {
        let (lo, hi) = b
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), q| (l.min(q[k]), h.max(q[k])));
        hi - lo
    };
    [f(0), f(1)]
}

/// **A DOBRA MUDA DE EIXO** — e a prova é a forma, não a diferença.
///
/// ⚠️ Uma dobra no eixo X põe a grelha em arco e faz a extensão VERTICAL crescer; a mesma
/// dobra a `55°` reparte esse crescimento pelos dois eixos. O oráculo é a RAZÃO das extensões:
/// se a direção só somasse ao `angle`, ela mudaria o quanto, nunca o para-onde.
#[test]
fn the_bend_pair_separates_on_which_axis_grew() {
    let b = bands();
    let (flat, turned) = (extent(&b[0]), extent(&b[1]));
    let r0 = flat[1] / flat[0];
    let r1 = turned[1] / turned[0];
    assert!(
        (r0 - r1).abs() > 0.1,
        "a razão das extensões tem de mudar com a direção: {r0:.3} contra {r1:.3}"
    );
}

/// **O ARO AUTORADO ENCOLHE A ZONA QUE AINDA GIRA** — e o resto satura.
///
/// ⚠️ O oráculo é o ângulo da peça do MEIO do caminho. Com o aro automático ela leva metade da
/// volta; com o aro em `RIM` ela já está fora dele e leva a volta inteira, como o aro.
#[test]
fn the_rim_pair_separates_on_where_the_twist_stops_growing() {
    let b = bands();
    // A peça mais distante do centro em cada banda, e uma a meio caminho.
    let arm = |band: &Vec<[f32; 2]>, frac: f32| {
        let far = band.iter().map(|q| q[0].hypot(q[1])).fold(0.0f32, f32::max);
        let target = far * frac;
        band.iter()
            .min_by(|a, c| {
                (a[0].hypot(a[1]) - target)
                    .abs()
                    .total_cmp(&(c[0].hypot(c[1]) - target).abs())
            })
            .copied()
            .expect("há peças")
    };
    // O ângulo polar da peça a meio raio diz quanto ela girou.
    let mid = |band: &Vec<[f32; 2]>| {
        let q = arm(band, 0.5);
        q[1].atan2(q[0])
    };
    let (auto, rim) = (mid(&b[2]), mid(&b[3]));
    assert!(
        (auto - rim).abs() > 0.2,
        "com o aro apertado a peça a meio raio tem de estar noutro ângulo: {auto:.3} vs {rim:.3}"
    );
}

/// **O PERFIL MUDA O CAMINHO E NÃO O DESTINO** — a peça do aro chega ao mesmo sítio.
///
/// ⚠️ É a metade que separa um perfil de um ângulo menor: se o `Smoother` estivesse só a
/// reduzir a volta, a peça mais externa também mudaria. Ela não pode.
#[test]
fn the_profile_pair_agrees_at_the_rim_and_differs_inside() {
    let b = bands();
    let far = |band: &Vec<[f32; 2]>| {
        band.iter()
            .max_by(|a, c| a[0].hypot(a[1]).total_cmp(&c[0].hypot(c[1])))
            .copied()
            .expect("há peças")
    };
    let (a, c) = (far(&b[4]), far(&b[5]));
    assert!(
        (a[0] - c[0]).abs() < 0.05 && (a[1] - c[1]).abs() < 0.05,
        "a peça do aro tem de coincidir nos dois perfis: {a:?} vs {c:?}"
    );
    assert_ne!(b[4], b[5], "…e o miolo tem de diferir");
}

/// **A LENTE ELÍPTICA INCHA UM EIXO E NÃO O OUTRO.**
#[test]
fn the_lens_pair_separates_on_the_aspect_it_produces() {
    let b = bands();
    let (round, flat) = (extent(&b[6]), extent(&b[7]));
    let ar = round[0] / round[1];
    let af = flat[0] / flat[1];
    assert!(
        (ar - 1.0).abs() < 0.05,
        "a lente redonda sobre uma grelha quadrada sai quadrada: {ar:.3}"
    );
    assert!(
        af > ar + 0.1,
        "a elíptica tem de sair mais larga que alta: {af:.3} contra {ar:.3}"
    );
}

/// **`Keep Length` OCUPA MENOS CURVA QUE `Fit`** — a escala do layout sobrevive.
#[test]
fn the_curve_pair_separates_on_how_much_of_the_curve_is_used() {
    let b = bands();
    let (fit, keep) = (extent(&b[8]), extent(&b[9]));
    assert!(
        fit[0] > keep[0] + 0.3,
        "o Fit estica até as pontas e o Keep Length não: {:.3} contra {:.3}",
        fit[0],
        keep[0]
    );
}
