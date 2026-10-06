//! ⭐⭐⭐ **O ESPETO — a cúbica que sai da forma e volta pela mesma recta.**
//!
//! Report do dono, 2026-09-29, com foto: *«surgiu um tipo de linha anómala no stroke (linha branca
//! atravessando a forma)»*. A fixtura são as **`65` amostras EXACTAS** do segmento que o produziu
//! (barra da cena em **C a `60°`**, segmento-fonte `4`, tolerância `0,00212`), copiadas da sonda que
//! o reproduziu — ⚠️ a primeira quarta parte é **quase recta** e começa DEVAGAR (a cúbica-fonte tem
//! uma alça curta no nó), que é o regime em que a medida do `kurbo` só olha num sentido.

use super::{Assado, ajusta};
use kurbo::{ParamCurve, ParamCurveNearest, PathEl, Point};

/// A tolerância com que o produto ajustou aquele segmento.
const TOL: f64 = 0.002_121_320_343_559_642_4;

fn amostras() -> Vec<Point> {
    vec![
        Point::new(-6.399679388602564, 5.771836397627524),
        Point::new(-6.397505010901952, 5.768070265491772),
        Point::new(-6.391073430545404, 5.75693044306918),
        Point::new(-6.380521976650854, 5.738654791336742),
        Point::new(-6.365987978336236, 5.713481171271454),
        Point::new(-6.347608764719482, 5.681647443850309),
        Point::new(-6.325521664918526, 5.643391470050304),
        Point::new(-6.299864008051301, 5.598951110848432),
        Point::new(-6.270773123235742, 5.548564227221689),
        Point::new(-6.2383863395897805, 5.49246868014707),
        Point::new(-6.202840986231351, 5.430902330601569),
        Point::new(-6.1642743922783865, 5.364103039562181),
        Point::new(-6.1228238868488205, 5.2923086680059015),
        Point::new(-6.0786267990605864, 5.215757076909725),
        Point::new(-6.031820458031618, 5.134686127250646),
        Point::new(-5.982542192879848, 5.04933368000566),
        Point::new(-5.93092933272321, 4.959937596151761),
        Point::new(-5.875925679721307, 4.868746685232507),
        Point::new(-5.818002942462375, 4.77621793260423),
        Point::new(-5.754828716347849, 4.689582040617393),
        Point::new(-5.688238315357527, 4.611719897921631),
        Point::new(-5.6236117528978635, 4.53962483367992),
        Point::new(-5.5655190412938245, 4.476014587901689),
        Point::new(-5.52230847633168, 4.415439639467519),
        Point::new(-5.5011457620574955, 4.355923747560673),
        Point::new(-5.506938552239562, 4.296374847029891),
        Point::new(-5.539907812021895, 4.2348379706296),
        Point::new(-5.595579766892127, 4.169643235060135),
        Point::new(-5.666829308650167, 4.0975500445005615),
        Point::new(-5.745628180841712, 4.013768953642334),
        Point::new(-5.821509593013521, 3.9086851991587452),
        Point::new(-5.894967791918038, 3.793742369879344),
        Point::new(-5.966346057256061, 3.6737604077402466),
        Point::new(-6.035117931385186, 3.5519707769718516),
        Point::new(-6.0994762210829245, 3.4307943700610615),
        Point::new(-6.152831193418298, 3.3123590811827923),
        Point::new(-6.189021024895136, 3.201552592951481),
        Point::new(-6.21859346906675, 3.1027220473339896),
        Point::new(-6.2499382930528355, 3.02020123594055),
        Point::new(-6.2878397790837175, 2.9590043926751264),
        Point::new(-6.3370555768831265, 2.9220278747487063),
        Point::new(-6.398557127362594, 2.908983957963935),
        Point::new(-6.470693077957609, 2.9160350730096187),
        Point::new(-6.5542576341930125, 2.9336481894250332),
        Point::new(-6.645619431577641, 2.956384712239648),
        Point::new(-6.742675619322247, 2.97935830802438),
        Point::new(-6.8496342383905855, 2.9897673661617983),
        Point::new(-6.956600470129339, 2.99693090661263),
        Point::new(-7.062560090221878, 2.999880302370878),
        Point::new(-7.1657257080078125, 3.0),
        Point::new(-7.2642822265625, 3.0),
        Point::new(-7.3578948974609375, 3.0),
        Point::new(-7.4462890625, 3.0),
        Point::new(-7.5291900634765625, 3.0),
        Point::new(-7.6063232421875, 3.0),
        Point::new(-7.6774139404296875, 3.0),
        Point::new(-7.7421875, 3.0),
        Point::new(-7.8003692626953125, 3.0),
        Point::new(-7.8516845703125, 3.0),
        Point::new(-7.8958587646484375, 3.0),
        Point::new(-7.9326171875, 3.0),
        Point::new(-7.9616851806640625, 3.0),
        Point::new(-7.9827880859375, 3.0),
        Point::new(-7.9956512451171875, 3.0),
        Point::new(-8.0, 3.0),
    ]
}

/// O quanto a PIOR cúbica de `path` se afasta da fonte — medido no sentido CÚBICA → FONTE, que é o
/// que o `kurbo` não mede. A fonte é a curva do bake avaliada densa.
fn pior_afastamento(src: &Assado<'_>, path: &kurbo::BezPath) -> f64 {
    let fonte: Vec<Point> = (0..=4096)
        .map(|i| src.em(f64::from(i) / 4096.0).0)
        .collect();
    let mut cur = Point::ZERO;
    let mut pior: f64 = 0.0;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => cur = p,
            PathEl::CurveTo(a, b, c) => {
                let cub = kurbo::CubicBez::new(cur, a, b, c);
                for i in 0..=64 {
                    let q = cub.eval(f64::from(i) / 64.0);
                    let d = fonte
                        .windows(2)
                        .map(|w| kurbo::Line::new(w[0], w[1]).nearest(q, 1e-12).distance_sq)
                        .fold(f64::INFINITY, f64::min)
                        .sqrt();
                    pior = pior.max(d);
                }
                cur = c;
            }
            PathEl::LineTo(p) => cur = p,
            _ => {}
        }
    }
    pior
}

/// ⭐⭐⭐ **GATE — o ajuste nunca aceita uma cúbica que saia da fonte.**
///
/// # As duas metades
///
/// 1. ⛔ **O CONTROLO: o `kurbo::fit_to_bezpath` sozinho ESPETA nesta fonte** — sem isso a fixtura
///    não conteria o fenómeno e a metade 2 passaria por vácuo. Medido: uma alça a `~9,5` de
///    distância numa corda de `0,94`.
/// 2. **O [`ajusta`] fica dentro de `2 × tolerância`**, que é a folga que ele próprio exige.
#[test]
fn o_ajuste_nunca_sai_da_fonte() {
    let pts = amostras();
    let src = Assado(&pts, None);
    let so_kurbo = kurbo::fit_to_bezpath(&src, TOL);
    let espeto = pior_afastamento(&src, &so_kurbo);
    assert!(
        espeto > 1.0,
        "o kurbo sozinho afastou-se só {espeto} — a fixtura deixou de conter o espeto"
    );
    let nosso = ajusta(&src, TOL);
    let pior = pior_afastamento(&src, &nosso);
    assert!(
        pior <= 2.0 * TOL,
        "o ajuste aceitou uma cúbica a {pior} da fonte (tolerância {TOL}) — é a linha que atravessa \
         a forma"
    );
    println!("  kurbo sozinho {espeto:.4} · o ajuste {pior:.6} (tolerância {TOL:.6})");
}
