//! **O QUE O FRONT-FACE FAZ COM A DEMÃO** — a sonda que decide a hipótese
//! líder do report do Enio (*"se aumentar hardness ou Auto Smooth, Layer fica
//! muito ruim"*) antes de uma linha de cura ser escrita.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_layer_front_face \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! # A hipótese, e por que ela é sobre o HARDNESS
//!
//! O peso de um dab é `shape = curva(hardness(t)) · alpha · facing · keep`. O
//! `apply_hardness_to_distances` da referência (`sculpt.cc:7549`) empurra o
//! platô para fora: com `hardness = h` toda distância `t < h` vira **zero**, e
//! zero é onde a curva vale **um**. ⇒ **quanto mais duro o pincel, maior a
//! fração da pegada em que `curva ≡ 1`** — e ali `shape` colapsa em
//! `facing · keep`.
//!
//! Sem máscara, `keep = 1`. Logo, em pincel duro, **o único termo espacial que
//! sobra é o `facing`**, que é `max(−n·olho, 0)` — o cosseno do ângulo de
//! visão. Numa esfera isso é um DOMO: a demão deixaria de subir à meta e
//! passaria a vestir o perfil da CÂMERA.
//!
//! ⚠️ **Numa GRADE PLANA o `facing` é identicamente 1** (toda normal é `+z`, o
//! olho é `−z`), então a fixture que a suíte do Layer usa hoje **não contém o
//! fenômeno**: os dois mundos são byte-idênticos ali. É por isso que esta sonda
//! mede numa ESFERA e traz a grade como CONTROLE.
//!
//! # E o que a referência de facto faz
//!
//! `layer.cc:149` é `if (brush.flag & BRUSH_FRONTFACE) calc_front_face(...)` —
//! um **checkbox do artista** (`use_frontface`, rotulado *"Front Faces Only"* em
//! `properties_paint_common.py:1354`), e **nenhuma linha do Blender inteiro o
//! LIGA** (varrido: o único hit fora de leitura é `use_front_face_ = brush_->flag
//! & BRUSH_FRONTFACE`, que também lê). Nós o aplicamos **incondicionalmente** em
//! todo verbo do modo `B`.
//!
//! A sonda não decide a cura — ela mede o tamanho do fenômeno para a cura ser
//! escolhida com número na mão.

use ph2d_mesh::{Face, Mesh, shapes};
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

/// Uma grade plana `n × n` em `z = 0` — o CONTROLE, onde `facing ≡ 1`.
fn grid(n: usize, half: f32) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let f = |k: usize| (k as f32 / n as f32) * 2.0 * half - half;
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            faces.push(Face::tri(at(i, j), at(i + 1, j), at(i + 1, j + 1)));
            faces.push(Face::tri(at(i, j), at(i + 1, j + 1), at(i, j + 1)));
        }
    }
    Mesh::from_parts(pos, faces).expect("grade")
}

/// Um pincel de demão com a dureza pedida.
///
/// ⚠️ **A ALAVANCA É O FLAG, e a versão anterior desta sonda estava VÁCUA.**
/// Ela usava `RefMode::S` contra `RefMode::B`, e o `kernel_for` é
/// `for_verb(verb).kernel()` — o `for_verb` **RECUA para `B`** num verbo que o
/// modo não declara, e o `S` não declara a demão (`ref_mode.rs:323`). Os dois
/// braços corriam a MESMA lei e a sonda imprimia duas colunas idênticas sob os
/// rótulos *LIGADO* e *desligado*: um instrumento que responde com confiança à
/// pergunta errada é pior que instrumento nenhum, que é a lição que o
/// `PH2D_FLUID_PROFILE` do Painter já pagou três vezes.
///
/// ⚠️ **Hoje a alavanca EXISTE no produto** (`Brush::front_faces_only`, o
/// `BRUSH_FRONTFACE`), então o A/B é o do artista e não uma ablação inventada.
/// O texto abaixo descreve a lei que o `mode` carrega, e ela continua exacta: o
/// **declara** o Layer — o produto não o oferece —, mas o `kernel_for` é um
/// `match` puro sobre o modo, então ele serve como a rota de ablação que isola
/// UMA linha da cadeia de fatores. É medição, não uma configuração alcançável.
fn coat(radius: f32, hardness: f32, front_faces_only: bool) -> Brush {
    Brush {
        verb: Verb::Layer,
        // ⚠️ **O modo é `B` FIXO, e não é escolha:** ele é o único que declara
        // a lei, e é onde a demão cai por construção (`profile_s(Layer)` é
        // `None` e o `for_verb` recua). Variá-lo aqui foi o que tornou a versão
        // anterior desta sonda vácua.
        mode: RefMode::B,
        radius,
        front_faces_only,
        strength: Verb::Layer.default_strength(),
        falloff: Verb::Layer.default_falloff(RefMode::B),
        hardness,
        ..Brush::default()
    }
}

/// **P1 — quanto da pegada o hardness satura.**
///
/// Puramente a aritmética da cadeia, sem malha: `curva(shaped_distance(t))`.
/// Se ela chega a `1,0000` numa fração grande do raio, o `facing` deixa de ser
/// uma correção de borda e passa a ser o perfil inteiro.
#[test]
#[ignore = "sonda"]
fn how_much_of_the_footprint_the_hardness_saturates() {
    println!("== P1: a curva depois do hardness (o que SOBRA para o facing decidir)");
    println!("  hardness |  t=0,1  t=0,3  t=0,5  t=0,7  t=0,9  |  fracao com curva >= 0,99");
    for h in [0.0f32, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let b = coat(0.4, h, false);
        print!("  {h:>8.2} |");
        for t in [0.1f32, 0.3, 0.5, 0.7, 0.9] {
            print!(" {:6.4}", b.falloff.weight(b.shaped_distance(t)));
        }
        // Onde a curva deixa de ser 1? Varredura fina.
        let mut sat = 0.0f32;
        let mut t = 0.0f32;
        while t <= 1.0 {
            if b.falloff.weight(b.shaped_distance(t)) >= 0.99 {
                sat = t;
            }
            t += 0.001;
        }
        println!("  |  {:>5.1} % do raio", sat * 100.0);
    }
    println!();
    println!("  ⇒ com a curva saturada, `shape` colapsa em `facing · keep`, e");
    println!("    sem mascara `keep = 1`: o facing vira o perfil INTEIRO.");
}

/// **P2 — o `facing` DENTRO da pegada, contra a grade.**
///
/// ⚠️ **A primeira versão desta sonda mediu a coisa errada duas vezes, e as
/// duas ficam escritas porque a segunda é fina.** (1) Ela somava os anéis por
/// raio **XY**, que numa esfera casa o hemisfério de trás junto — normal `−z`,
/// facing zero — e reportava `0,49` uniforme, que se lê como *"o facing é uma
/// constante"* e é a média de `1` com `0` sobre vértices que o dab **nem
/// alcança**. (2) E o raio: um dab de `0,45` no polo de uma esfera de raio `1`
/// cobre `26°` de arco, onde o cosseno vai de `1,00` a `0,90` — a fixture
/// **não continha o fenômeno** que o A/B existe para ver.
///
/// Aqui a pegada é a de VERDADE (distância 3-D ao centro do dab) e a coluna é a
/// FAIXA, não a média: uma média esconde exatamente a variação que decide se o
/// facing é um perfil ou uma constante.
#[test]
#[ignore = "sonda"]
fn what_the_facing_weighs_inside_the_footprint() {
    let eye = [0.0f32, 0.0, -1.0];
    let facing = |n: [f32; 3]| (-(n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2])).max(0.0);
    let d3 = |p: [f32; 3], c: [f32; 3]| {
        ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt()
    };

    println!("== P2: o facing DENTRO da pegada (dist 3-D < raio)");
    println!("   fixture              raio |   min    max   media  |  n");
    for (name, mesh, centre) in [
        ("GRADE (o controle)", grid(80, 1.2), [0.0f32, 0.0, 0.0]),
        ("ESFERA", shapes::sculpt_sphere(1.0), [0.0f32, 0.0, 1.0]),
    ] {
        let normals = mesh.normals();
        for r in [0.2f32, 0.45, 0.9, 1.4] {
            let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f32, 0usize);
            for (i, p) in mesh.positions().iter().enumerate() {
                if d3(*p, centre) < r {
                    let f = facing(normals[i]);
                    lo = lo.min(f);
                    hi = hi.max(f);
                    sum += f;
                    n += 1;
                }
            }
            if n > 0 {
                println!(
                    "  {name:<20} {r:>4.2} | {lo:6.4} {hi:6.4} {:6.4}  |  {n}",
                    sum / n as f32
                );
            }
        }
    }
    println!();
    println!("  ⇒ na GRADE o facing e' 1,0000 em toda parte: os dois mundos sao");
    println!("    byte-identicos ali, e NENHUM gate que use a grade pode ver isto.");
    println!("  ⇒ na esfera ele so' varre a faixa toda com raio GRANDE — um dab");
    println!("    pequeno no polo mal sai de 1,0, e um A/B ali mede zero.");
}

/// **P3 — o PERFIL DEPOSITADO na esfera: domo contra mesa.**
///
/// A pergunta do report, medida. Um corte radial da altura ganha, com o facing
/// ligado e desligado, em pincel macio e duro.
/// ⚠️ **E ela mede o TRANSIENTE junto com o limite, porque a demão SATURA.**
/// O `coat_step` converge para o teto qualquer que seja o peso — o peso decide
/// *quão depressa*, não *até onde* (a P1 do `measure_layer_law` já o dizia:
/// *"o falloff é uma TAXA, não um perfil"*). Um A/B feito só no limite mede
/// **zero por construção**, e foi o que a primeira corrida desta sonda mediu:
/// colunas byte-idênticas com 32 dabs.
#[test]
#[ignore = "sonda"]
fn the_deposited_profile_with_and_without_the_facing() {
    let r = 0.9f32; // grande de propósito: é o que faz o facing varrer a faixa
    let centre = [0.0f32, 0.0, 1.0];
    let eye = [0.0f32, 0.0, -1.0];
    let len = |q: [f32; 3]| (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();

    println!("== P3: perfil radial da demao numa ESFERA (raio {r}, altura da fabrica)");
    println!("   (a coluna e' o deslocamento RADIAL ganho, em unidades de mundo)");
    for h in [0.0f32, 0.9] {
        for dabs in [1usize, 2, 8, 32] {
            println!();
            println!("  hardness {h:.2}, {dabs} dab(s)");
            println!("    facing    |  t=0,1  t=0,3  t=0,5  t=0,7  t=0,9  |  borda/centro");
            for (label, ff) in [("LIGADO   ", true), ("desligado", false)] {
                let base = shapes::sculpt_sphere(1.0);
                let mut mesh = base.clone();
                let b = coat(r, h, ff);
                let mut s = SculptStroke::default();
                s.begin(&mesh);
                for _ in 0..dabs {
                    s.dab(&mut mesh, &b, &Dab::at(centre, r, eye), Symmetry::default());
                }

                let mut cols = Vec::new();
                for t in [0.1f32, 0.3, 0.5, 0.7, 0.9] {
                    let (mut sum, mut n) = (0.0f32, 0usize);
                    for (i, p) in mesh.positions().iter().enumerate() {
                        let p0 = base.positions()[i];
                        // A régua é a distância 3-D ao centro do dab, a MESMA
                        // que o kernel usa: um anel por raio XY casaria o
                        // hemisfério de trás, que o dab nem alcança.
                        let d = ((p0[0] - centre[0]).powi(2)
                            + (p0[1] - centre[1]).powi(2)
                            + (p0[2] - centre[2]).powi(2))
                        .sqrt()
                            / r;
                        if (d - t).abs() < 0.06 {
                            sum += len(*p) - len(p0);
                            n += 1;
                        }
                    }
                    cols.push(if n > 0 { sum / n as f32 } else { f32::NAN });
                }
                print!("  {label} |");
                for c in &cols {
                    print!(" {c:6.4}");
                }
                println!("  |  {:.4}", cols[4] / cols[0].max(1e-9));
            }
        }
    }
    println!();
    println!("  ⇒ MEDIDO: divergem em 1-2 dabs (0,3793 contra 0,7828) e CONVERGEM");
    println!("    em 32 (0,9824 contra 0,9831) -- o facing e' uma TAXA nesta lei,");
    println!("    nao um perfil: ele muda quao DEPRESSA a demao fecha, nunca a");
    println!("    altura em que ela para. Um A/B que so' esfregasse ate' saturar");
    println!("    mediria ZERO e chamaria a lei de inerte.");
}
