//! **O QUE OS VERBOS QUE APERTAM FAZEM, MODO A MODO** — a sonda do report do
//! Enio (2026-08-15): *"Blob modo B bom! Blob modo L ruim. Pinch em B e S bons
//! mas idênticos ou quase idênticos. Em L Pinch ruim. Crease OK."*
//!
//! ⚠️ **Ela dirige `SculptStroke::dab`, a porta do artista** — a §7.11 já pagou
//! duas vezes por medir peça isolada, e o que decide um chip é o que se vê.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_pinch_family_modes -- --ignored --nocapture`

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(64, 96, 1.0)
}

/// O centro do dab, no polo `+z`; o olho está em `+z` a olhar para `-z`.
const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.30;

/// **Quanto o traço ANDA**, em raios de pincel.
///
/// ⚠️ **Ele era zero e a sonda MENTIA sobre o `B` do Pinch.** A lei daquele modo
/// é o `pinch.cc`, que precisa da direção do gesto e **recusa** sem ela; com
/// todos os dabs no mesmo centro o [`ph2d_sculpt3d::Dab::path`] fica zero e a
/// tabela reportava pico `0,0000` — *"o verbo não faz nada"*, que é falso para
/// qualquer gesto real. Curto de propósito: as bandas radiais abaixo são medidas
/// contra o `TIP`, e um traço longo as borraria.
const WALK: f32 = 0.12;

/// Um traço de `steps` eventos que ANDA ao longo de `+x` — o caminho do produto.
fn stroke(verb: Verb, mode: RefMode, strength: f32, steps: usize) -> Mesh {
    let mut mesh = sphere();
    let b = Brush {
        verb,
        mode,
        radius: R,
        strength,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 1..=steps {
        let t = k as f32 / steps as f32;
        let x = TIP[0] + (t - 1.0) * WALK * R;
        let mut d = Dab::pulling([x, TIP[1], TIP[2]], R, EYE, [0.0; 3]);
        d.amount = t;
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    mesh
}

/// As bandas de `r/R` em que o perfil é lido — a última cobre o alcance inteiro
/// do campo (`KELVINLET_REACH = 3`).
const BANDS: [(f32, f32); 7] = [
    (0.00, 0.25),
    (0.25, 0.50),
    (0.50, 0.75),
    (0.75, 1.00),
    (1.00, 1.50),
    (1.50, 2.00),
    (2.00, 3.10),
];

/// Por banda: `(lateral médio, normal médio, quantos vértices)`.
///
/// O sinal do lateral é **para DENTRO positivo** (apertar), e o do normal é
/// **para FORA positivo** (subir). O eixo é `+z`, a normal da esfera no polo.
fn profile(rest: &Mesh, out: &Mesh) -> Vec<(f32, f32, usize)> {
    let (a, b) = (rest.positions(), out.positions());
    let mut acc = vec![(0.0f32, 0.0f32, 0usize); BANDS.len()];
    for i in 0..a.len() {
        let p = a[i];
        let q = [p[0] - TIP[0], p[1] - TIP[1]];
        let r = (q[0] * q[0] + q[1] * q[1]).sqrt();
        let t = r / R;
        let disp = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        let Some(k) = BANDS.iter().position(|&(lo, hi)| t >= lo && t < hi) else {
            continue;
        };
        let lateral = if r > 1e-9 {
            -(disp[0] * q[0] + disp[1] * q[1]) / r
        } else {
            0.0
        };
        acc[k].0 += lateral;
        acc[k].1 += disp[2];
        acc[k].2 += 1;
    }
    acc.iter()
        .map(|&(l, n, c)| {
            if c == 0 {
                (0.0, 0.0, 0)
            } else {
                (l / c as f32, n / c as f32, c)
            }
        })
        .collect()
}

/// O `r/R` do vértice movido mais distante — *até onde o artista vê a malha
/// mexer*, contra o anel do cursor que vale `1,0`.
fn reach(rest: &Mesh, out: &Mesh) -> f32 {
    let (a, b) = (rest.positions(), out.positions());
    let mut worst: f32 = 0.0;
    for i in 0..a.len() {
        let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 1e-5 {
            continue;
        }
        let q = [a[i][0] - TIP[0], a[i][1] - TIP[1]];
        worst = worst.max((q[0] * q[0] + q[1] * q[1]).sqrt() / R);
    }
    worst
}

/// O maior deslocamento de um vértice, em unidades de raio de pincel.
fn peak(rest: &Mesh, out: &Mesh) -> f32 {
    let (a, b) = (rest.positions(), out.positions());
    let mut worst: f32 = 0.0;
    for i in 0..a.len() {
        let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        worst = worst.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    worst / R
}

/// Quanto duas malhas divergem: `(pior vértice, média sobre os que se moveram)`,
/// em unidades de raio de pincel.
fn divergence(x: &Mesh, y: &Mesh) -> (f32, f32) {
    let (a, b) = (x.positions(), y.positions());
    let mut worst: f32 = 0.0;
    let mut sum = 0.0;
    let mut n = 0;
    for i in 0..a.len() {
        let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        worst = worst.max(m);
        if m > 1e-9 {
            sum += m;
            n += 1;
        }
    }
    (worst / R, if n == 0 { 0.0 } else { sum / n as f32 / R })
}

fn show(name: &str, rest: &Mesh, out: &Mesh) {
    let prof = profile(rest, out);
    println!(
        "  {name:<26} alcance {:.2}r  pico {:.4}r",
        reach(rest, out),
        peak(rest, out)
    );
    print!("    lateral(dentro+) ");
    for (l, _, _) in &prof {
        print!("{l:>9.5} ");
    }
    println!();
    print!("    normal (fora+)   ");
    for (_, n, _) in &prof {
        print!("{n:>9.5} ");
    }
    println!();
}

#[test]
#[ignore = "sonda: roda a mao com --nocapture"]
fn what_the_pinching_verbs_do_mode_by_mode() {
    let rest = sphere();
    println!(
        "\nbandas r/R: {}",
        BANDS
            .iter()
            .map(|&(a, b)| format!("{a:.2}-{b:.2}"))
            .collect::<Vec<_>>()
            .join("  ")
    );
    for verb in [Verb::Pinch, Verb::Crease, Verb::Blob] {
        println!("\n=== {} ===", verb.label());
        for mode in RefMode::ALL {
            if !mode.declares(verb) {
                println!("  {:?}: nao declarado", mode);
                continue;
            }
            let out = stroke(verb, mode, 0.75, 8);
            show(&format!("{mode:?} (forca 0,75)"), &rest, &out);
        }
    }
}

/// **POR QUE `B` E `S` SE PARECEM NO PINCH** — a ablação dos três eixos que os
/// separam, cada um medido sozinho.
#[test]
#[ignore = "sonda: roda a mao com --nocapture"]
fn how_far_apart_the_s_and_b_pinch_really_are() {
    let rest = sphere();
    println!("\n== Pinch: S contra B, por forca ==");
    println!("  forca |  pior desvio |  desvio medio |  pico S |  pico B");
    for strength in [0.25f32, 0.5, 0.75, 1.0] {
        let s = stroke(Verb::Pinch, RefMode::S, strength, 8);
        let b = stroke(Verb::Pinch, RefMode::B, strength, 8);
        let (worst, mean) = divergence(&s, &b);
        println!(
            "  {strength:>5.2} | {worst:>12.6} | {mean:>13.6} | {:>7.4} | {:>7.4}",
            peak(&rest, &s),
            peak(&rest, &b)
        );
    }
    println!("\n  (desvios em unidades de raio de pincel)");

    // Os DOIS eixos que sobram quando a curva de forca e neutralizada: em
    // `forca = 1` o `x²` do B e a identidade, entao o que resta e so a lei de
    // kernel (lateral tangencial + front-face continuo).
    let s1 = stroke(Verb::Pinch, RefMode::S, 1.0, 8);
    let b1 = stroke(Verb::Pinch, RefMode::B, 1.0, 8);
    let (worst, mean) = divergence(&s1, &b1);
    println!("  em forca 1,00 a curva e a IDENTIDADE nos dois modos:");
    println!("    o que sobra (lateral + front-face) = pior {worst:.6}r, medio {mean:.6}r");
}

/// O volume com sinal da malha fechada (teorema da divergência sobre os
/// triângulos) — a pergunta *"este verbo REMOVE materia?"*.
fn volume(m: &Mesh) -> f64 {
    let p = m.positions();
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    let mut v = 0.0f64;
    for t in tris {
        let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
        let (a, b, c) = (
            [a[0] as f64, a[1] as f64, a[2] as f64],
            [b[0] as f64, b[1] as f64, b[2] as f64],
            [c[0] as f64, c[1] as f64, c[2] as f64],
        );
        v += a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);
    }
    v / 6.0
}

/// Quanto do barro movido cai FORA do anel do cursor — a fracao que o artista
/// nao pediu, porque o circulo desenhado diz `1,0`.
fn outside_share(rest: &Mesh, out: &Mesh) -> f32 {
    let (a, b) = (rest.positions(), out.positions());
    let (mut inside, mut outside) = (0.0f32, 0.0f32);
    for i in 0..a.len() {
        let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        // ⚠️ **A pegada de um TRAÇO é o tubo em volta do segmento percorrido**,
        // não um disco: medir contra um ponto acusaria de "fora do anel" o barro
        // que o outro extremo do gesto alcança legitimamente.
        let q = [a[i][0] - TIP[0], a[i][1] - TIP[1]];
        let along = q[0].clamp(-WALK * R, 0.0);
        let d2 = (q[0] - along) * (q[0] - along) + q[1] * q[1];
        if d2.sqrt() / R <= 1.0 {
            inside += m;
        } else {
            outside += m;
        }
    }
    if inside + outside <= 0.0 {
        0.0
    } else {
        outside / (inside + outside)
    }
}

/// **O ANEL DO CURSOR DIZ `1,0`, E O CAMPO NAO OBEDECE** — mais a pergunta que
/// o doc do [`Verb::Pinch`] AFIRMA: o campo *"deixa de remover volume"*?
#[test]
#[ignore = "sonda: roda a mao com --nocapture"]
fn how_much_of_the_gesture_lands_outside_the_ring() {
    let rest = sphere();
    let v0 = volume(&rest);
    println!("\n  verbo   modo | fora do anel |   dV/V (10^-4) | pico");
    for verb in [Verb::Pinch, Verb::Crease, Verb::Blob] {
        for mode in RefMode::ALL {
            if !mode.declares(verb) {
                continue;
            }
            let out = stroke(verb, mode, 0.75, 8);
            println!(
                "  {:<7} {:?}   |    {:>6.1} %  |   {:>+10.4}   | {:.4}",
                verb.label(),
                mode,
                outside_share(&rest, &out) * 100.0,
                (volume(&out) - v0) / v0 * 1.0e4,
                peak(&rest, &out)
            );
        }
    }
}
