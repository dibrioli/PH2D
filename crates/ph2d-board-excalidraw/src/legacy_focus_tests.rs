use super::*;
use ph2d_board_model::{ElementId, FracKey, Rgba, RichText, Shape, Style};

fn shape(kind: ShapeType, round: bool, bx: [f64; 4], angle: f64) -> Element {
    let ink = Rgba([30, 30, 30, 255]);
    let mut st = Style::new(None, Some(ink), ink);
    st.round = round;
    let mut el = Element::new_shape(
        ElementId(1),
        FracKey::between(None, None),
        Shape {
            kind,
            style: st,
            text: RichText::plain(""),
        },
        bx,
    );
    el.angle = angle;
    el
}

/// As formas da lei das peças que o escritor grava: losango vivo e redondo (`type 2`), rectângulo
/// redondo (`type 3`, abaixo e acima do tecto do raio), medidas inteiras e fraccionárias, rodadas.
fn piece_shapes() -> Vec<(&'static str, Element)> {
    vec![
        (
            "losango 200×120",
            shape(ShapeType::Diamond, false, [0.0, 0.0, 200.0, 120.0], 0.0),
        ),
        (
            "losango 173,4×97,8 rodado",
            shape(ShapeType::Diamond, false, [10.0, 20.0, 173.4, 97.8], 0.2),
        ),
        (
            "losango redondo 173,4×97,8 rodado",
            shape(ShapeType::Diamond, true, [10.0, 20.0, 173.4, 97.8], 0.3),
        ),
        (
            "losango redondo 301,5×211,3 rodado",
            shape(ShapeType::Diamond, true, [-50.0, 5.0, 301.5, 211.3], -0.7),
        ),
        (
            "rectângulo redondo 300×200",
            shape(ShapeType::Rectangle, true, [0.0, 0.0, 300.0, 200.0], 0.0),
        ),
        (
            "rectângulo redondo 61,3×39,7 rodado",
            shape(ShapeType::Rectangle, true, [3.0, 4.0, 61.3, 39.7], 0.45),
        ),
    ]
}

/// ⭐ Um ponto fixo fundo numa elipse ACHATADA, junto ao eixo maior: o nível que passa por ele tem
/// um semi-eixo abaixo de [`THINNEST`] (0,55 px) e recusa-se — fica o ponto do foco (`gap 0`). A
/// elipse degenerada do 0.18.1 errou 9,8 px no oráculo (semi-eixo 6e-11, `X10_nosso` ph2d-19).
#[test]
fn a_level_thinner_than_a_pixel_is_refused() {
    let el = shape(ShapeType::Ellipse, false, [0.0, 0.0, 200.0, 20.0], 0.0);
    let (tip, from) = ([160.0, 9.6], [161.0, -190.0]);
    assert!(ellipse_gap(100.0, 10.0, [60.0, -0.4]).is_none());
    let (_, gap) = legacy(&el, from, tip, 6.0);
    assert_eq!(gap, 0.0, "o nível de meio pixel escreveu-se");
}

/// ⭐⭐ W23 B9 — o INVERSO da lei das peças: toda ponta que o 0.18.1 põe por entrada (≥ 2 cruzamentos;
/// vãos −5 · 0,5 · 6 · 20, 24 direcções × 10 focos) volta, pelo par que o escritor grava, ao mesmo
/// sítio a ≤ 0,01 px — e o vão gravado é o de partida (a menos do [`UNTIE`]).
#[test]
fn the_written_pair_lands_on_our_tip_by_the_pieces_law() {
    let mut n = 0;
    let mut worst: f64 = 0.0;
    for (name, t) in piece_shapes() {
        let c = t.center();
        let reach = t.w.max(t.h) / 2.0 + 140.0;
        for i in 0..24 {
            let th = (2.5 + 15.0 * f64::from(i)).to_radians();
            let from = [c[0] + reach * th.cos(), c[1] + reach * th.sin()];
            for k in 0..10 {
                let f0 = -0.9 + 0.2 * f64::from(k);
                for g0 in [-5.0, 0.5, 6.0, 20.0] {
                    let Some(tip) = landing(&t, from, f0, g0) else {
                        continue;
                    };
                    let aim = landing(&t, from, f0, 0.0).unwrap();
                    if dist(tip, aim) < 1e-9 {
                        continue; // o ramo de um só cruzamento: a ponta é o ponto do foco
                    }
                    let (f, g) = legacy(&t, from, tip, 0.0);
                    let got = landing(&t, from, f, g).unwrap_or([f64::NAN; 2]);
                    let e = dist(got, tip);
                    worst = worst.max(e);
                    assert!(
                        e <= 0.01 && (g - g0).abs() < 1e-3,
                        "{name} θ {i} focus {f0} gap {g0}: o par ({f}, {g}) põe a ponta a {e}"
                    );
                    n += 1;
                }
            }
        }
    }
    eprintln!("{n} pontas, pior {worst:.2e}");
    assert!(n > 3000, "{n}");
}

/// ⭐ W23 B9 — um ponto fixo DENTRO no losango e no rectângulo redondo: o vão negativo que a lei leva
/// exactamente ao ponto quando existe; quando não (do lado de LÁ da recta, ou a corda num só canto), o
/// de menor erro. Fracção exprimível MEDIDA (por forma, 12 direcções × a grelha de 0,2 da meia-caixa).
#[test]
fn an_inside_point_is_reached_where_the_law_can_and_nearest_where_not() {
    let mut total = (0usize, 0usize);
    for (name, t) in piece_shapes() {
        let c = t.center();
        let reach = t.w.max(t.h) / 2.0 + 140.0;
        let (mut n, mut hit, mut far) = (0usize, 0usize, 0.0f64);
        for ix in -4..=4 {
            for iy in -4..=4 {
                let local = [
                    f64::from(ix) / 5.0 * t.w / 2.0,
                    f64::from(iy) / 5.0 * t.h / 2.0,
                ];
                let tip = t.rotate([c[0] + local[0], c[1] + local[1]]);
                if !ph2d_board_geom::inside(&t, tip) {
                    continue;
                }
                for j in 0..12 {
                    let th = (15.0 + 30.0 * f64::from(j)).to_radians();
                    let from = [c[0] + reach * th.cos(), c[1] + reach * th.sin()];
                    let (f, g) = legacy(&t, from, tip, 0.0);
                    assert!(g == 0.0 || g.abs() >= MIN_GAP, "{name}: gap {g}");
                    let e = landing(&t, from, f, g).map_or(f64::INFINITY, |q| dist(q, tip));
                    n += 1;
                    if e <= 0.01 {
                        hit += 1;
                    } else {
                        far = far.max(e);
                    }
                }
            }
        }
        eprintln!(
            "{name}: {hit}/{n} exprimíveis ({:.1} %), o pior dos outros a {far:.2} px",
            100.0 * hit as f64 / n as f64
        );
        total.0 += hit;
        total.1 += n;
    }
    let frac = total.0 as f64 / total.1 as f64;
    eprintln!("total {}/{} = {:.3}", total.0, total.1, frac);
    assert!(frac >= INSIDE_REACHED, "{frac}");
}

/// A fracção medida dos pontos dentro que a lei exprime exactamente (20/10: 2 076 / 4 176 = 0,4971;
/// do lado de LÁ da recta nenhum se exprime, metade das direcções).
const INSIDE_REACHED: f64 = 0.497;
