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
    growth_report();
    expansion_report();
    smoothness_sweep();
    reveal_report();

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

/// **O CRESCIMENTO, molde a molde** — a pergunta do Enio de 2026-08-29:
/// *"porque vários presets não têm crescimento suave, mas em saltos? é normal?"*
///
/// Duas grandezas DIFERENTES, e a bancada separa-as:
///  1. **suavidade** — quanto do percurso total cabe no pior passo. Uniforme = `1/N`; um
///     salto = `1,00`.
///  2. **congelamento** — a partir de que ponto do slider a planta deixa de mudar de todo
///     (o orçamento de módulos saturou e a derivação larga a geração inteira que não coube).
#[allow(dead_code)]
fn growth_report() {
    const N: usize = 24;
    let base = |p: &ls::Preset| -> Vec<(&'static str, f32)> {
        vec![
            ("angle", p.angle),
            ("step", p.step),
            ("width", p.width),
            ("width_scale", default_of("width_scale")),
            ("length_scale", default_of("length_scale")),
            ("root_angle", default_of("root_angle")),
            ("seed", default_of("seed")),
            ("mode", ls::MODE_GRAMMAR as f32),
        ]
    };
    println!("\n== CRESCIMENTO: e' suave, ou aos saltos? ==");
    println!("   (varredura de 1,0 ate' as geracoes que o molde declara, em {N} passos)");
    println!(
        "{:8} {:>10} {:>12} {:>10}   veredito",
        "molde", "reescreve", "pior_passo", "congela_em"
    );
    for p in ls::PRESETS {
        // A gramática reescreve o símbolo que DESENHA? É esta a pergunta que separa as duas
        // famílias — e ela lê-se do texto.
        let redraws = p
            .rules
            .split(';')
            .filter_map(|r| r.split("->").next())
            .any(|head| {
                let h = head.trim().trim_start_matches(|c: char| !c.is_alphabetic());
                h.starts_with('F') || h.starts_with('G')
            });
        let hs: Vec<f32> = (0..=N)
            .map(|k| {
                let g = 1.0 + (p.generations - 1.0) * k as f32 / N as f32;
                let s = shoot(p.axiom, p.rules, g, &base(p));
                s.w.max(s.h)
            })
            .collect();
        let rise = hs[N] - hs[0];
        let worst = hs
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f32::max);
        // E onde ele congela: o primeiro `g` (em passos de 0,25 ate' 12) apos o qual o tamanho
        // deixa de mudar.
        let mut frozen = None;
        let mut prev = -1.0f32;
        let mut since = 0;
        let mut at = 0.0f32;
        let mut g = 1.0f32;
        while g <= 12.0 {
            let s = shoot(p.axiom, p.rules, g, &base(p));
            let v = s.w.max(s.h);
            if (v - prev).abs() < 1e-6 {
                since += 1;
                if since == 1 {
                    at = g;
                }
                if since >= 8 {
                    frozen = Some(at);
                    break;
                }
            } else {
                since = 0;
            }
            prev = v;
            g += 0.25;
        }
        let frac = if rise.abs() > 1e-6 { worst / rise } else { 1.0 };
        println!(
            "{:8} {:>10} {:>11.0}% {:>10}   {}",
            p.label,
            if redraws { "o DESENHO" } else { "so' a PONTA" },
            frac * 100.0,
            frozen.map_or("nunca".to_string(), |g| format!("{g:.2}")),
            if frac > 0.5 { "SALTA" } else { "suave" }
        );
    }
    println!(
        "\n   (uniforme em {N} passos seria {:.0}%)",
        100.0 / N as f32
    );
}

/// **A RAZÃO DE EXPANSÃO de cada molde** — quanto a figura cresce por geração.
///
/// É dela que sai o passo-por-geração que mantém uma gramática de REFINAMENTO do mesmo
/// tamanho: `step_scale = 1 / razão`. Sem isso, «refinar» e «crescer» são a mesma coisa para
/// o desenho, e nenhuma interpolação pode ser suave — a figura muda de TAMANHO em cada
/// travessia de geração.
#[allow(dead_code)]
fn expansion_report() {
    println!("\n== A RAZÃO DE EXPANSÃO (tamanho da geração n+1 / da n), no angulo do molde ==");
    println!(
        "{:8} {:>7} {:>34}   {:>10}",
        "molde", "ang", "razoes por geracao", "1/mediana"
    );
    for p in ls::PRESETS {
        let base: Vec<(&str, f32)> = vec![
            ("angle", p.angle),
            ("step", 0.5),
            ("width", default_of("width")),
            ("width_scale", default_of("width_scale")),
            ("length_scale", default_of("length_scale")),
            ("root_angle", default_of("root_angle")),
            ("seed", default_of("seed")),
            ("mode", ls::MODE_GRAMMAR as f32),
        ];
        let sizes: Vec<f32> = (1..=6)
            .map(|g| {
                let s = shoot(p.axiom, p.rules, g as f32, &base);
                s.w.max(s.h)
            })
            .collect();
        let mut r: Vec<f32> = sizes.windows(2).map(|w| w[1] / w[0].max(1e-6)).collect();
        let line: String = r.iter().map(|x| format!("{x:6.2}")).collect();
        r.sort_by(f32::total_cmp);
        let med = r[r.len() / 2];
        println!(
            "{:8} {:>7.1} {line:>34}   {:>10.4}",
            p.label,
            p.angle,
            1.0 / med
        );
    }
}

/// **A VARREDURA QUE ESCOLHE O `step_scale`** — a pergunta do Enio de 2026-08-29
/// (*"acho que o ideal e' o crescimento suave"*).
///
/// Para cada molde, com os dois interruptores LIGADOS, mede o pior passo de uma varredura
/// fina do `Generations`. O `step_scale` sai daqui, medido — nunca escolhido.
#[allow(dead_code)]
fn smoothness_sweep() {
    const N: usize = 40;
    println!("\n== QUAL `step_scale` ALISA CADA MOLDE (pior passo, % da subida total) ==");
    print!("{:8}", "molde");
    let scales = [1.0f32, 0.80, 0.67, 0.50, 0.40, 0.3333, 0.25, 0.20];
    for k in scales {
        print!("{:>9}", format!("{k:.3}"));
    }
    println!("   melhor");
    for p in ls::PRESETS {
        print!("{:8}", p.label);
        let mut best = (f32::MAX, 0.0f32);
        for k in scales {
            let base: Vec<(&str, f32)> = vec![
                ("angle", p.angle),
                ("step", p.step),
                ("width", p.width),
                ("width_scale", default_of("width_scale")),
                ("length_scale", default_of("length_scale")),
                ("root_angle", default_of("root_angle")),
                ("seed", default_of("seed")),
                ("mode", ls::MODE_GRAMMAR as f32),
                ("step_scale", k),
            ];
            let hs: Vec<f32> = (0..=N)
                .map(|j| {
                    let g = 1.0 + (p.generations - 1.0) * j as f32 / N as f32;
                    let s = shoot(p.axiom, p.rules, g, &base);
                    s.w.max(s.h)
                })
                .collect();
            // ⚠️⚠️ **O DENOMINADOR É A MÉDIA, não a subida total** — e a 1.ª redacção usava a
            // subida, o que fez esta varredura imprimir **619 050 %**. O motivo é o próprio
            // objectivo: com o `step_scale` certo uma gramática de refinamento fica do MESMO
            // TAMANHO e só ganha detalhe, então a subida é ~0 e a razão explode. *Uma régua
            // normalizada pelo que a cura leva a zero mede a cura ao contrário.*
            let mean = hs.iter().sum::<f32>() / hs.len() as f32;
            let worst = hs
                .windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0, f32::max);
            let frac = worst / mean.max(1e-6);
            if frac < best.0 {
                best = (frac, k);
            }
            print!("{:>8.0}%", frac * 100.0);
        }
        println!("   {:.4}", best.1);
    }
    println!("   (a barra e' o pior passo contra a MEDIA do tamanho)");
}

/// **E SE A ANIMACAO CERTA FOR OUTRA?** — a lei da casa: *antes de construir um item de lista
/// aberta, MEÇA se a composição já o exprime.*
///
/// Uma figura de refinamento não CRESCE — ela é redesenhada mais fina. A animação que os
/// motion designers usam para uma curva assim não é «mais gerações»: é **revelar o traçado**
/// (o *trim path*), que aqui é `source.lsystem → field.index_range(Index) → motion.cull`.
///
/// Isto mede exactamente essa composição sem montar o grafo: revelar os primeiros `k %` dos
/// elementos é o que o `motion.cull` faz sobre um campo de índice.
#[allow(dead_code)]
fn reveal_report() {
    const N: usize = 40;
    println!("\n== REVELAR O TRACADO (o que `index_range + cull` ja' faz hoje) ==");
    println!(
        "{:8} {:>12} {:>12}   veredito",
        "molde", "pior_passo", "ramifica?"
    );
    for p in ls::PRESETS {
        let base: Vec<(&str, f32)> = vec![
            ("angle", p.angle),
            ("step", p.step),
            ("width", p.width),
            ("width_scale", default_of("width_scale")),
            ("length_scale", default_of("length_scale")),
            ("root_angle", default_of("root_angle")),
            ("seed", default_of("seed")),
            ("mode", ls::MODE_GRAMMAR as f32),
        ];
        let st = ls::probe_build(p.axiom, p.rules, p.generations, &base);
        let pts = match st.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let parent = match st.get("parent") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        // Ramifica? Dois elementos pendurados no mesmo pai. Numa CURVA isso nao acontece, e a
        // ordem da cadeia E' a ordem do traco.
        let mut seen = std::collections::BTreeMap::<i64, usize>::new();
        for q in parent.iter().filter(|x| **x >= 0.0) {
            *seen.entry(*q as i64).or_default() += 1;
        }
        let branches = seen.values().filter(|c| **c > 1).count();

        // A ponta revelada anda de forma contínua? A régua é o SALTO da ponta entre dois
        // quadros, contra o tamanho da figura.
        let n = pts.len();
        let bbox = {
            let x0 = pts.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = pts.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = pts.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = pts.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            (x1 - x0).max(y1 - y0).max(1e-6)
        };
        let mut worst = 0.0f32;
        for j in 1..=N {
            let a = (n as f32 * (j - 1) as f32 / N as f32) as usize;
            let b = (n as f32 * j as f32 / N as f32) as usize;
            if a == 0 || b == 0 || a >= n || b >= n {
                continue;
            }
            let d = ((pts[b][0] - pts[a][0]).powi(2) + (pts[b][1] - pts[a][1]).powi(2)).sqrt();
            worst = worst.max(d / bbox);
        }
        println!(
            "{:8} {:>11.0}% {:>12}   {}",
            p.label,
            worst * 100.0,
            if branches > 0 {
                format!("{branches} nos")
            } else {
                "CURVA".to_string()
            },
            if worst < 0.25 {
                "revela SUAVE"
            } else {
                "salta entre ramos"
            }
        );
    }
}
