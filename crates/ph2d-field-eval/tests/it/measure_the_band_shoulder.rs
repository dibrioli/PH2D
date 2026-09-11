//! **SONDA** — quão dura é a transição no fim da banda da torção. Report do Enio, 2026-08-30:
//! *«smoke ok mas muito dura a transição»*, com a seta na dobra.
//!
//! ⚠️ **A régua é a NORMAL, e não a silhueta** — é a lei que a W54 deste módulo já pagou: uma
//! polilinha erra `0,079 %` da peça (invisível) e a normal salta `6,43°`, e *é a normal que a luz
//! mostra*.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::Field;

fn barra(turns: f32, lower: f32, upper: f32, falloff: f32) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.34, 0.11, 0.62],
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    n.mods.push(Unary::Twist {
        turns,
        lower,
        upper,
        falloff,

        axis: ph2d_field::mods::TWIST_AXIS,
    });
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// A normal da superfície à altura `z`, na direcção `ang` a partir do eixo.
fn normal(f: &Field, z: f64, ang: f64) -> [f64; 3] {
    let (dx, dy) = (ang.cos(), ang.sin());
    let (mut lo, mut hi) = (0.0f64, 2.0f64);
    for _ in 0..50 {
        let m = f64::midpoint(lo, hi);
        if f.at(m * dx, m * dy, z) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    let r = f64::midpoint(lo, hi);
    let (x, y) = (r * dx, r * dy);
    let e = 1e-4;
    let g = [
        f.at(x + e, y, z) - f.at(x - e, y, z),
        f.at(x, y + e, z) - f.at(x, y - e, z),
        f.at(x, y, z + e) - f.at(x, y, z - e),
    ];
    let n = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
    [g[0] / n, g[1] / n, g[2] / n]
}

fn angulo(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

/// ⭐⭐ **O giro da normal por unidade de altura, à prova de QUINA.**
///
/// ⛔ A 1.ª redacção media numa direcção fixa (`+X`) e leu `2 695 °/un` numa altura — **o mesmo
/// número para o ombro `0,00`, `0,06` e `0,12`**, o que o denuncia: um valor que não se mexe quando
/// a cura se mexe é da sonda, não da peça. Ali o raio apanhava a **quina** da barra, onde a normal
/// vira de facto de golpe.
///
/// ⇒ mede-se em **24 direcções** e fica-se com a **mediana**: as quatro quinas são uma minoria por
/// construção, e a mediana não as vê. *Uma sonda que a cura não move está a medir outra coisa.*
fn giro(f: &Field, z: f64) -> f64 {
    const N: usize = 24;
    let mut v: Vec<f64> = (0..N)
        .map(|i| {
            let a = i as f64 / N as f64 * std::f64::consts::TAU;
            angulo(normal(f, z - 0.004, a), normal(f, z + 0.004, a)) / 0.008
        })
        .collect();
    v.sort_by(f64::total_cmp);
    v[N / 2]
}

#[test]
fn measure_the_band_shoulder() {
    // A banda acaba em z = 0: abaixo a peça é rígida, acima ela torce.
    // ⭐ A CURVATURA: quantos graus a normal gira por unidade de altura, medida de UM lado só.
    println!("\n ombro | giro por unidade em 11 alturas | maior salto");
    for w in [0.0f32, 0.06, 0.12, 0.22, 0.35] {
        let doc = barra(0.35, 0.0, 9.0, w);
        let f = Field::new(&doc);
        let zs = [
            -0.40f64, -0.20, -0.08, -0.02, 0.02, 0.06, 0.08, 0.10, 0.14, 0.20, 0.40,
        ];
        let t: Vec<f64> = zs.iter().map(|z| giro(&f, *z)).collect();
        let salto = t
            .windows(2)
            .map(|p| (p[1] - p[0]).abs())
            .fold(0.0f64, f64::max);
        print!("  {w:.2} | ");
        for v in &t {
            print!("{v:6.1} ");
        }
        println!(" | {salto:7.1}");
    }
}

/// ⭐⭐⭐ **O GATE**: o ombro tira o degrau da curvatura, e o corte duro é o CONTROLE.
///
/// | ombro | maior salto no giro (°/un) |
/// |---:|---:|
/// | `0,00` (duro) | **`136,9`** |
/// | `0,12` | `48,4` |
/// | **`0,22`** | **`31,8`** |
/// | `0,35` | `35,2` |
///
/// ⭐ O joelho está em `0,22` numa barra de meia-altura `0,62`; acima disso não melhora. A barra do
/// gate é **metade** do corte duro — longe do medido e longe do defeito, e não colada a nenhum.
#[test]
fn the_band_shoulder_is_not_a_crease() {
    let zs = [
        -0.40f64, -0.20, -0.08, -0.02, 0.02, 0.06, 0.10, 0.14, 0.20, 0.40,
    ];
    let maior_salto = |w: f32| {
        let doc = barra(0.35, 0.0, 9.0, w);
        let f = Field::new(&doc);
        let t: Vec<f64> = zs.iter().map(|z| giro(&f, *z)).collect();
        t.windows(2)
            .map(|p| (p[1] - p[0]).abs())
            .fold(0.0f64, f64::max)
    };
    let duro = maior_salto(0.0);
    let macio = maior_salto(0.22);
    // ⛔ **O CONTROLE**: sem ele o gate passaria numa lei que devolvesse zero em todo o lado.
    assert!(
        duro > 100.0,
        "o corte DURO mede {duro:.1} °/un de salto — a sonda deixou de ver o degrau que ela existe \
         para medir, e o gate abaixo passa a não provar nada"
    );
    assert!(
        macio < duro * 0.5,
        "com ombro o salto é {macio:.1} °/un contra {duro:.1} do corte duro — o ombro deixou de \
         amaciar, e o fim da banda volta a ler-se como quina (report do Enio, 2026-08-30)"
    );
}
