//! **A sonda dos perfis nomeados** (W2b) — o que cada candidato DESENHA, antes de o número
//! virar constante.
//!
//! Duas perguntas, e a segunda é a que decide:
//!
//! 1. **a forma lê o nome?** — `at(t)` em cinco pontos do arco: um *taper* tem de descer
//!    monotonicamente, um *bulge* tem de subir e voltar. Um perfil cujo nome não descreve a
//!    curva é pior que perfil nenhum.
//! 2. **a ponta de largura ZERO produz fita?** — `0.0` é o pincel de ponta fina, e é o único
//!    valor da tabela que toca a borda do domínio (`MIN_WIDTH_FACTOR`). Se o sweep devolvesse
//!    vazio, ou área zero, ou coordenada não-finita, o preset seria um botão que apaga o traço.
//!
//! Rodar: `cargo test -p ph2d-vec-boolean --test measure_width_presets -- --nocapture --ignored`

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, WidthProfile};

/// Uma reta horizontal de comprimento `len`, com traço de largura `w`.
fn bar(len: f64, w: f64) -> VecPath {
    VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([len * 0.5, 0.0]),
            VecVertex::corner([len, 0.0]),
        ],
        closed: false,
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), w)),
        ..VecPath::default()
    }
}

/// A área com sinal do contorno (shoelace sobre as âncoras) — grosseira de propósito: a
/// pergunta é *"sobrou forma?"*, não *"que área exatamente?"*.
fn rough_area(p: &VecPath) -> f64 {
    let n = p.verts.len();
    if n < 3 {
        return 0.0;
    }
    let mut a = 0.0f64;
    for i in 0..n {
        let [x0, y0] = p.verts[i].anchor;
        let [x1, y1] = p.verts[(i + 1) % n].anchor;
        a += x0 * y1 - x1 * y0;
    }
    (a * 0.5).abs()
}

#[test]
#[ignore = "sonda de medição, não gate"]
fn what_each_candidate_profile_draws() {
    let candidates: [(&str, WidthProfile); 4] = [
        ("Uniform", WidthProfile::UNIFORM),
        (
            "Taper ",
            WidthProfile {
                start: 1.0,
                mid: 0.55,
                end: 0.0,
                position: 0.5,
            },
        ),
        (
            "Both  ",
            WidthProfile {
                start: 0.0,
                mid: 1.0,
                end: 0.0,
                position: 0.5,
            },
        ),
        (
            "Bulge ",
            WidthProfile {
                start: 1.0,
                mid: 1.8,
                end: 1.0,
                position: 0.5,
            },
        ),
    ];
    println!("\n  perfil   t=0.00  0.25  0.50  0.75  1.00   | fitas  âncoras  área   finito");
    for (name, p) in candidates {
        let stops = p.to_stops();
        let mut row = format!("  {name}  ");
        for i in 0..=4 {
            let t = f64::from(i) / 4.0;
            row.push_str(&format!("{:6.3}", stops.at(t)));
        }
        let out = ph2d_vec_boolean::power_stroke(&bar(100.0, 8.0), &stops);
        let anchors: usize = out.iter().map(|q| q.verts.len()).sum();
        let area: f64 = out.iter().map(rough_area).sum();
        let finite = out.iter().all(|q| {
            q.verts
                .iter()
                .all(|v| v.anchor.iter().all(|c| c.is_finite()))
        });
        println!(
            "{row}   | {:5}  {anchors:7}  {area:7.1}  {finite}",
            out.len()
        );
    }
    println!();
}
