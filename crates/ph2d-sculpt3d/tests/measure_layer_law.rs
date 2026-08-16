//! **O QUE A LEI DO LAYER FAZ** — a sonda que decide a W8 antes de uma linha
//! dela ser escrita.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_layer_law \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! O plano 21 §5.1 item 6 promete uma *"demão de **altura constante**,
//! saturante e apagável"*, e o `layer.cc` do Blender tem a lei inteira em três
//! linhas:
//!
//! ```text
//! disp[v] += f[v] · força · (1,05 − |disp[v]|)      // satura
//! disp[v]  = clamp(disp[v], −(1−máscara), 1−máscara)
//! alvo     = orig[v] + normal_orig[v] · altura · disp[v]
//! ```
//!
//! ⚠️ **A pergunta que decide o desenho não é *"como isto satura?"* — é *"em
//! que ALTURA cada vértice para?"***. Se cada vértice da pegada converge para
//! `disp = 1`, a demão é um **PLATÔ de altura constante** e o falloff é uma
//! TAXA (quão depressa cada um lá chega), não um perfil. Se ela converge para
//! algo proporcional ao peso, o verbo é um Draw com teto — e aí o chip novo
//! não tem conteúdo, que é o que o §4 recusa.
//!
//! ⚠️ **E há uma segunda pergunta, sobre a NOSSA arquitetura.** O Blender
//! move a posição VIVA (`pos += (alvo − pos)·f`), e o nosso aplicador anda do
//! `pre` CONGELADO. Se eu copiasse o `·f` do lado de cá ele seria re-aplicado
//! **do base a cada dab**, e o platô convergido passaria a valer `f · altura` —
//! o falloff vazando para dentro da propriedade que define o verbo. A sonda
//! mede as duas recorrências lado a lado para o número existir antes da
//! decisão.

use ph2d_mesh::{Face, Mesh, shapes};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// A recorrência do `offset_displacement_factors`, verbatim.
fn step(disp: f32, f: f32, strength: f32) -> f32 {
    (disp + f * strength * (1.05 - disp.abs())).clamp(-1.0, 1.0)
}

/// Uma grade plana `n × n` em `z = 0`, meia-largura `half`.
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

/// **P1 + P2 — para onde cada peso converge, e em quantos dabs.**
#[test]
#[ignore = "sonda"]
fn where_each_weight_lands_and_how_fast() {
    println!("== P1/P2: a lei do `disp`, por peso do falloff (forca 1,0)");
    println!("     peso |    d1     d2     d4     d8    d16    d32    d64   d256");
    for f in [1.0f32, 0.75, 0.5, 0.25, 0.1, 0.02] {
        print!("  {f:>7.2} |");
        let mut disp = 0.0f32;
        let mut k = 0usize;
        for want in [1usize, 2, 4, 8, 16, 32, 64, 256] {
            while k < want {
                disp = step(disp, f, 1.0);
                k += 1;
            }
            print!(" {disp:6.4}");
        }
        println!();
    }
    println!();
    println!("  ⇒ se TODA linha converge para 1,0000, a demao e' um PLATO de");
    println!("    altura constante e o falloff e' uma TAXA, nao um perfil.");

    // Quantos dabs cada peso precisa para chegar a 99 % do teto?
    println!();
    println!("  dabs ate' 0,99 por peso:");
    for f in [1.0f32, 0.5, 0.25, 0.1, 0.02] {
        let mut disp = 0.0f32;
        let mut k = 0usize;
        while disp < 0.99 && k < 100_000 {
            disp = step(disp, f, 1.0);
            k += 1;
        }
        println!("    peso {f:>4.2}: {k} dabs (disp {disp:.4})");
    }
}

/// **P4 — a recorrência do `pre` CONGELADO contra a do VIVO.**
///
/// Blender: `pos_{k+1} = pos_k + (T_k − pos_k)·f`, com `T_k = orig + h·disp_k`.
/// Nós, com `accum = 1`: `pos_k = T_k` (o alvo já traz o peso).
/// Nós, se copiássemos o `·f`: `pos_k = base + (T_k − base)·f` — que **NÃO**
/// converge para `T`, e é o buraco que esta metade da sonda existe para medir.
#[test]
#[ignore = "sonda"]
fn the_frozen_recurrence_against_the_live_one() {
    let h = 1.0f32; // altura unitária: a coluna É a fração da demão
    println!("== P4: onde cada recorrencia POUSA (altura 1,0, forca 1,0)");
    println!("     peso |  vivo(blender)   nosso(accum=1)   nosso(se copiasse .f)");
    for f in [1.0f32, 0.5, 0.25, 0.1] {
        let mut disp = 0.0f32;
        let mut live = 0.0f32; // pos − orig, a recorrência do Blender
        for _ in 0..4096 {
            disp = step(disp, f, 1.0);
            let t = h * disp;
            live += (t - live) * f;
        }
        let ours_unit = h * disp;
        let ours_lagged = (h * disp) * f;
        println!("  {f:>7.2} |  {live:12.6}   {ours_unit:12.6}   {ours_lagged:18.6}");
    }
    println!();
    println!("  ⇒ o VIVO e o `accum = 1` pousam no MESMO lugar; copiar o `.f`");
    println!("    para o aplicador do `pre` congelado faz o falloff vazar para");
    println!("    dentro do PLATO, que e' a propriedade que o verbo entrega.");

    // E o TRANSIENTE: quantos dabs até as duas recorrências concordarem a 1 %?
    println!();
    println!("  transiente (dabs ate' |vivo − nosso| < 1 % da altura):");
    for f in [1.0f32, 0.5, 0.25, 0.1] {
        let mut disp = 0.0f32;
        let mut live = 0.0f32;
        let mut k = 0usize;
        let mut worst = 0.0f32;
        loop {
            disp = step(disp, f, 1.0);
            let t = h * disp;
            live += (t - live) * f;
            k += 1;
            worst = worst.max((t - live).abs());
            if (t - live).abs() < 0.01 * h || k > 100_000 {
                break;
            }
        }
        println!("    peso {f:>4.2}: {k} dabs (pior separacao {worst:.4})");
    }
}

/// **P3 — o PERFIL na malha: a demão é chata no topo?**
#[test]
#[ignore = "sonda"]
fn the_coat_is_flat_on_top() {
    let r = 0.4f32;
    for dabs in [1usize, 4, 16, 64] {
        let mut mesh = grid(80, 1.2);
        let b = Brush {
            verb: Verb::Draw, // o CONTROLE: o verbo que hoje deposita
            radius: r,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for _ in 0..dabs {
            s.dab(
                &mut mesh,
                &b,
                &Dab::at([0.0, 0.0, 0.0], r, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        let peak = mesh
            .positions()
            .iter()
            .map(|q| q[2])
            .fold(f32::NEG_INFINITY, f32::max);
        println!("  CONTROLE Draw, {dabs:>3} dabs sobre o mesmo ponto: pico {peak:.5}");
    }
    println!();
    println!("  ⇒ o Draw sem Accumulate PARA (envelope); o teto dele e' o `reach`");
    println!("    de UM dab (forca · raio · 0,1 = {:.4}).", 1.0 * r * 0.1);
}

/// **P5 — a ESCALA do nosso mundo: quanto vale `height` aqui?**
///
/// A referência declara `height` em unidades de OBJETO (`PROP_DISTANCE`), com
/// default `0,5`, faixa dura `[0, 1]` e faixa macia `[0, 0,2]`. O número só é
/// legível contra o tamanho do barro que este app de facto abre.
#[test]
#[ignore = "sonda"]
fn what_an_object_unit_is_worth_here() {
    let mesh = shapes::sculpt_sphere(1.0);
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in mesh.positions() {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let ext = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    println!("== P5: a esfera de fabrica");
    println!("  vertices {}  extensao {ext:?}", mesh.positions().len());
    println!();
    println!("  a altura da referencia, em fracao do RAIO desta esfera:");
    for h in [0.05f32, 0.1, 0.2, 0.5, 1.0] {
        println!("    height {h:>4.2}  =  {:>5.1} % do raio", h / 1.0 * 100.0);
    }
    println!();
    println!("  e o deposito de UM dab de Draw, para comparar (forca 1, raio r):");
    for r in [0.1f32, 0.2, 0.4] {
        println!("    raio {r:>4.2}: reach {:.4}", r * 0.1);
    }
}

/// **P6 — O PERFIL RADIAL: o falloff pousa na SUPERFÍCIE, ou só na taxa?**
///
/// ⚠️ **É a pergunta que o smoke devolveu** (*"falloff provavelmente errado,
/// resultado muito diferente e pior"*): a demão saía com uma parede quase
/// vertical e um topo chato até a borda do círculo, onde a referência entrega
/// um ombro. A P1 já dizia que todo peso converge para `disp = 1` — o que esta
/// mede é **quantos dabs um traço de facto entrega**, e portanto qual metade da
/// curva o artista vê: o transiente (um ombro) ou o limite (um top-hat).
#[test]
#[ignore = "sonda"]
fn the_radial_profile_of_a_coat() {
    let r = 0.4f32;
    let b = Brush {
        verb: Verb::Layer,
        radius: r,
        strength: Verb::Layer.default_strength(),
        falloff: Verb::Layer.default_falloff(),
        ..Brush::default()
    };
    println!(
        "== P6: perfil radial (forca {:.2}, falloff {:?}, altura {:.2}, raio {r})",
        b.strength, b.falloff, b.layer_height
    );
    println!("   dabs |  t=0,1  t=0,3  t=0,5  t=0,7  t=0,8  t=0,9  t=0,95   (fracao da altura)");
    for dabs in [1usize, 2, 4, 8, 16, 32, 64, 256] {
        let mut mesh = grid(240, 0.55);
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for _ in 0..dabs {
            s.dab(
                &mut mesh,
                &b,
                &Dab::at([0.0, 0.0, 0.0], r, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        print!("  {dabs:>5} |");
        for t in [0.1f32, 0.3, 0.5, 0.7, 0.8, 0.9, 0.95] {
            // média dos vértices num anel fino em torno de t·r
            let (mut sum, mut n) = (0.0f32, 0usize);
            for p in mesh.positions() {
                let d = (p[0] * p[0] + p[1] * p[1]).sqrt() / r;
                if (d - t).abs() < 0.02 {
                    sum += p[2];
                    n += 1;
                }
            }
            let frac = if n == 0 {
                f32::NAN
            } else {
                sum / n as f32 / b.layer_height
            };
            print!(" {frac:6.3}");
        }
        println!();
    }
    println!();
    println!("  ⇒ uma linha que sai `1,000` em toda coluna e' um TOP-HAT: a parede");
    println!("    e' vertical e o falloff nao esta' na superficie, so' no caminho.");
}

/// **P7 — O MODO EM QUE A DEMÃO NASCE, e o que ele faz com a força.**
///
/// ⚠️ **O `S` não declara este verbo** ([`ph2d_sculpt3d::RefMode::declares`]) e
/// mesmo assim é o modo em que a shell o faz nascer. O
/// [`ph2d_sculpt3d::Brush::weight`] pergunta `profile(mode)` **sem** o recuo que
/// o `kernel_for` tem, então um modo que não declara o verbo devolve `None` e o
/// slider vira o peso CRU — onde a referência deste verbo o eleva ao QUADRADO
/// (`sculpt.cc:2337-2339`).
#[test]
#[ignore = "sonda"]
fn the_mode_the_coat_is_born_in() {
    use ph2d_sculpt3d::RefMode;
    println!("== P7: quem declara a DEMAO, e o peso que cada modo entrega");
    for m in RefMode::ALL {
        println!(
            "  {m:?}: declara={} peso(slider 0,50)={:.4}",
            m.declares(Verb::Layer),
            Brush {
                verb: Verb::Layer,
                mode: m,
                strength: 0.5,
                ..Brush::default()
            }
            .weight()
        );
    }
    println!(
        "  nascimento da shell: RefMode::default() = {:?}",
        RefMode::default()
    );
    println!(
        "  oferecidos no painel: {:?}",
        RefMode::offered_for(Verb::Layer).collect::<Vec<_>>()
    );

    let r = 0.4f32;
    println!();
    println!("  perfil radial por MODO (mesmo slider 0,50, mesmos dabs):");
    println!("   modo | dabs |  t=0,5  t=0,7  t=0,8  t=0,9  t=0,95");
    for m in [RefMode::S, RefMode::B] {
        let b = Brush {
            verb: Verb::Layer,
            mode: m,
            radius: r,
            strength: 0.5,
            falloff: Verb::Layer.default_falloff(),
            ..Brush::default()
        };
        for dabs in [8usize, 16, 64] {
            let mut mesh = grid(240, 0.55);
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            for _ in 0..dabs {
                s.dab(
                    &mut mesh,
                    &b,
                    &Dab::at([0.0, 0.0, 0.0], r, [0.0, 0.0, -1.0]),
                    Symmetry::default(),
                );
            }
            print!("     {m:?} | {dabs:>4} |");
            for t in [0.5f32, 0.7, 0.8, 0.9, 0.95] {
                let (mut sum, mut n) = (0.0f32, 0usize);
                for p in mesh.positions() {
                    let d = (p[0] * p[0] + p[1] * p[1]).sqrt() / r;
                    if (d - t).abs() < 0.02 {
                        sum += p[2];
                        n += 1;
                    }
                }
                print!(" {:6.3}", sum / n as f32 / b.layer_height);
            }
            println!();
        }
    }
}

/// **P8 — O CENSO: quantos verbos a shell faz nascer num modo que não os
/// declara?** A demão é o que o smoke pegou; a pergunta é de quantos ela é.
#[test]
#[ignore = "sonda"]
fn the_census_of_verbs_born_in_a_mode_that_does_not_declare_them() {
    use ph2d_sculpt3d::RefMode;
    println!("== P8: nascimento `[RefMode::default(); N]` contra o que declara");
    let mut wrong = 0usize;
    for v in Verb::ALL {
        let born = RefMode::default();
        if !born.declares(v) {
            wrong += 1;
            let first = RefMode::offered_for(v).next();
            println!(
                "  {v:?}: nasce {born:?} (nao declara) -> deveria {first:?}; \
                 peso(0,50) {:.4} contra {:.4}",
                Brush {
                    verb: v,
                    mode: born,
                    strength: 0.5,
                    ..Brush::default()
                }
                .weight(),
                Brush {
                    verb: v,
                    mode: first.unwrap(),
                    strength: 0.5,
                    ..Brush::default()
                }
                .weight(),
            );
        }
    }
    println!("  ⇒ {wrong} de {} verbos", Verb::ALL.len());
}
