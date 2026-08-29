//! **A BANCADA DOS MOLDES** — o que cada `PRESET` de facto desenha, com os defaults do painel.
//!
//! Report do Enio (2026-08-29): *"o modo tree funciona aparentemente bem. os demais tem
//! resultado questionável."* Esta bancada existe para que a auditoria disso seja sobre
//! NÚMEROS e não sobre impressões — ela corre o MESMO `build` do produto (`probe_build`).
//!
//!   cargo run -p ph2d-node-source-lsystem --example preset_report --release

use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::{Column, Stream};

/// Os defaults do manifesto, lidos do próprio manifesto — nunca escritos à mão.
fn default_of(name: &str) -> f32 {
    ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map_or(0.0, |p| p.default)
}

struct Shot {
    count: usize,
    w: f32,
    h: f32,
    cy: f32,
    size_min: f32,
    size_max: f32,
    syms: String,
    drawn: usize,
}

fn vec2(s: &Stream, n: &str) -> Vec<[f32; 2]> {
    match s.get(n) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn scal(s: &Stream, n: &str) -> Vec<f32> {
    match s.get(n) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn shoot(axiom: &str, rules: &str, gens: f32, over: &[(&str, f32)]) -> Shot {
    let s = ls::probe_build(axiom, rules, gens, over);
    let p = vec2(&s, "P");
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for q in &p {
        x0 = x0.min(q[0]);
        x1 = x1.max(q[0]);
        y0 = y0.min(q[1]);
        y1 = y1.max(q[1]);
    }
    let size: Vec<f32> = vec2(&s, "size").iter().map(|v| v[0]).collect();
    let sym = scal(&s, "sym");
    let mut kinds: Vec<u8> = sym.iter().map(|v| *v as u8).collect();
    kinds.sort_unstable();
    kinds.dedup();
    Shot {
        count: s.count(),
        w: if p.is_empty() { 0.0 } else { x1 - x0 },
        h: if p.is_empty() { 0.0 } else { y1 - y0 },
        cy: if p.is_empty() { 0.0 } else { (y0 + y1) * 0.5 },
        size_min: size.iter().copied().fold(f32::MAX, f32::min),
        size_max: size.iter().copied().fold(f32::MIN, f32::max),
        syms: kinds.iter().map(|c| *c as char).collect(),
        // Quantos elementos DESENHAM um traço (`F`/`G`) contra os que só marcam.
        drawn: sym
            .iter()
            .filter(|v| **v as u8 == b'F' || **v as u8 == b'G')
            .count(),
    }
}

fn main() {
    let base: Vec<(&str, f32)> = vec![
        ("angle", default_of("angle")),
        ("step", default_of("step")),
        ("width", default_of("width")),
        ("width_scale", default_of("width_scale")),
        ("length_scale", default_of("length_scale")),
        ("root_angle", default_of("root_angle")),
        ("seed", default_of("seed")),
        ("mode", ls::MODE_GRAMMAR as f32),
    ];
    let gens_default = default_of("generations");

    println!("== OS DEFAULTS DO PAINEL ==");
    for (k, v) in &base {
        println!("   {k:14} = {v}");
    }
    println!("   {:14} = {gens_default}", "generations");
    println!("   (a bbox de referencia da cena =108 e' ~3,2 x ~4 unidades de mundo por coluna)\n");

    println!(
        "== O 'ANTES': cada molde com os defaults do MANIFESTO (generations = {gens_default}) =="
    );
    println!("   (nao e' o que o artista ve' desde 29/08 — hoje o molde escreve o proprio");
    println!("    enquadramento; a tabela de baixo e' a que conta)");
    println!(
        "{:8} {:>7} {:>7} {:>8} {:>8} {:>8} {:>9} {:>9}  simbolos",
        "molde", "elems", "desenh", "largura", "altura", "centro_y", "size_min", "size_max"
    );
    for ls::Preset {
        label: name,
        axiom,
        rules,
        ..
    } in ls::PRESETS
    {
        let s = shoot(axiom, rules, gens_default, &base);
        println!(
            "{name:8} {:>7} {:>7} {:>8.2} {:>8.2} {:>8.2} {:>9.4} {:>9.4}  {}",
            s.count, s.drawn, s.w, s.h, s.cy, s.size_min, s.size_max, s.syms
        );
    }

    println!("\n== EXPLOSAO: quantos elementos por geracao ==");
    print!("{:8}", "molde");
    for g in 1..=8 {
        print!("{g:>9}");
    }
    println!();
    for ls::Preset {
        label: name,
        axiom,
        rules,
        ..
    } in ls::PRESETS
    {
        print!("{name:8}");
        for g in 1..=8 {
            print!("{:>9}", shoot(axiom, rules, g as f32, &base).count);
        }
        println!();
    }

    println!("\n== TAMANHO: a maior dimensao da caixa, por geracao ==");
    print!("{:8}", "molde");
    for g in 1..=8 {
        print!("{g:>9}");
    }
    println!();
    for ls::Preset {
        label: name,
        axiom,
        rules,
        ..
    } in ls::PRESETS
    {
        print!("{name:8}");
        for g in 1..=8 {
            let s = shoot(axiom, rules, g as f32, &base);
            print!("{:>9.2}", s.w.max(s.h));
        }
        println!();
    }

    println!("\n== QUE ANGULO CADA MOLDE QUER (altura/largura da caixa, e a largura) ==");
    print!("{:8}", "molde");
    for a in [15.0f32, 20.0, 25.0, 30.0, 45.0, 60.0, 90.0] {
        print!("{:>14}", format!("{a:.0}deg"));
    }
    println!();
    for ls::Preset {
        label: name,
        axiom,
        rules,
        ..
    } in ls::PRESETS
    {
        print!("{name:8}");
        for a in [15.0f32, 20.0, 25.0, 30.0, 45.0, 60.0, 90.0] {
            let mut o = base.clone();
            o[0] = ("angle", a);
            let s = shoot(axiom, rules, gens_default, &o);
            print!("{:>14}", format!("{:.2}/{:.2}", s.h / s.w.max(1e-6), s.w));
        }
        println!();
    }

    println!("\n== CRESCE CONTINUAMENTE? (altura em 4,00 .. 5,00, passo 0,25) ==");
    for ls::Preset {
        label: name,
        axiom,
        rules,
        ..
    } in ls::PRESETS
    {
        print!("{name:8}");
        let hs: Vec<f32> = (0..=4)
            .map(|k| shoot(axiom, rules, 4.0 + k as f32 * 0.25, &base).h)
            .collect();
        for h in &hs {
            print!("{h:>9.2}");
        }
        let rise = hs[4] - hs[0];
        let worst = hs
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max);
        print!("   subida {rise:>7.2}  pior_passo {worst:>7.2}");
        if rise.abs() > 1e-4 && worst > 0.6 * rise.abs() {
            print!("   <== SALTA");
        }
        if rise.abs() <= 1e-4 {
            print!("   <== NAO CRESCE");
        }
        println!();
    }

    derive_framing();

    println!("\n== A ESPESSURA ESCORRE? (size do 1o elemento contra o ultimo) ==");
    for ls::Preset {
        label: name,
        axiom,
        rules,
        ..
    } in ls::PRESETS
    {
        let s = ls::probe_build(axiom, rules, gens_default, &base);
        let size: Vec<f32> = vec2(&s, "size").iter().map(|v| v[0]).collect();
        let flat = size.iter().copied().fold(f32::MIN, f32::max)
            - size.iter().copied().fold(f32::MAX, f32::min)
            < 1e-6;
        println!(
            "{name:8} primeiro {:>8.4}  ultimo {:>8.4}{}",
            size.first().copied().unwrap_or(0.0),
            size.last().copied().unwrap_or(0.0),
            if flat { "   <== CHAPADA" } else { "" }
        );
    }
}

/// **O ENQUADRAMENTO DE CADA MOLDE, CONFERIDO** — a bancada que derivou a tabela, agora a
/// medir se ela ainda está certa.
///
/// ⚠️ **Ela lê o ângulo e as gerações do PRÓPRIO molde**, nunca de uma lista escrita aqui: uma
/// segunda cópia daquelas escolhas envelheceria na primeira vez que alguém editasse a tabela.
/// O que ela imprime é o `step` que PORIA cada molde do tamanho da mediana — se ele diferir do
/// que a tabela declara, a tabela está fora do enquadramento.
///
/// ⚠️ As réguas que **reprovam** vivem em `tests/presets_frame_themselves.rs`; isto imprime.
#[allow(dead_code)]
fn derive_framing() {
    const WIDTH_OVER_STEP: f32 = 0.09 / 0.28;
    let base: Vec<(&str, f32)> = vec![
        ("angle", 0.0),
        ("step", 0.0),
        ("width", default_of("width")),
        ("width_scale", default_of("width_scale")),
        ("length_scale", default_of("length_scale")),
        ("root_angle", default_of("root_angle")),
        ("seed", default_of("seed")),
        ("mode", ls::MODE_GRAMMAR as f32),
    ];
    let shot_of = |p: &ls::Preset| {
        let mut o = base.clone();
        o[0] = ("angle", p.angle);
        o[1] = ("step", p.step);
        shoot(p.axiom, p.rules, p.generations, &o)
    };

    let sizes: Vec<f32> = ls::PRESETS
        .iter()
        .map(|p| {
            let s = shot_of(p);
            s.w.max(s.h)
        })
        .collect();
    let mut sorted = sizes.clone();
    sorted.sort_by(f32::total_cmp);
    let median = (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) * 0.5;

    println!("\n== O ENQUADRAMENTO, COMO A TABELA O DECLARA ==");
    println!("   (mediana = {median:.3} unidades de mundo; uma coluna da cena tem ~4)");
    println!(
        "{:8} {:>6} {:>6} {:>8} {:>9} {:>7} {:>10} {:>11}",
        "molde", "ang", "gens", "elems", "mundo", "x_med", "step", "step_ideal"
    );
    for (p, size) in ls::PRESETS.iter().zip(&sizes) {
        let s = shot_of(p);
        let ideal = p.step * median / size;
        println!(
            "{:8} {:>6.1} {:>6.1} {:>8} {size:>9.2} {:>7.2} {:>10.4} {ideal:>11.4}{}",
            p.label,
            p.angle,
            p.generations,
            s.count,
            size / median,
            p.step,
            if (ideal / p.step - 1.0).abs() > 0.25 {
                "   <== FORA"
            } else {
                ""
            }
        );
    }
    println!("\n   (a espessura que acompanha e' `{WIDTH_OVER_STEP:.3} x step`)");
}
