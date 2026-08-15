//! **O VALE** — a sonda que mede o report do Enio: *"num vale a tool correta
//! tende a FECHAR o vale, e a nossa tende a AUMENTÁ-LO"*.
//!
//! A forma é a de um vale liso que corre ao longo de `x`; a secção transversal
//! é medida ao longo de `y`, na COLUNA central da grade — por ÍNDICE e nunca
//! por coordenada, pela razão que a sonda irmã do Draw Sharp pagou (os
//! vértices andam em `x` também, e um filtro por `x ≈ 0` perde metade deles).
//!
//! O número que decide é a **PROFUNDIDADE**: quanto o chão do vale está abaixo
//! das cristas. Fechar o vale é ela DIMINUIR.

use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

const N: usize = 80;
const HALF: f32 = 2.0;
/// Meia-largura do vale.
const W: f32 = 0.5;
/// Quão fundo ele é.
const AMP: f32 = 0.4;

/// `z` da superfície em repouso: um vale liso ao longo de `x`.
fn valley_z(y: f32) -> f32 {
    if y.abs() >= W {
        0.0
    } else {
        -AMP * 0.5 * (1.0 + (std::f32::consts::PI * y / W).cos())
    }
}

fn valley_grid() -> ph2d_mesh::Mesh {
    let mut pos = Vec::with_capacity((N + 1) * (N + 1));
    for j in 0..=N {
        for i in 0..=N {
            let f = |k: usize| (k as f32 / N as f32) * 2.0 * HALF - HALF;
            let (x, y) = (f(i), f(j));
            pos.push([x, y, valley_z(y)]);
        }
    }
    let at = |i: usize, j: usize| (j * (N + 1) + i) as u32;
    let mut faces = Vec::with_capacity(N * N * 2);
    for j in 0..N {
        for i in 0..N {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("indices validos")
}

/// A secção transversal na coluna central: `(y, z)` por índice de linha.
fn section(mesh: &ph2d_mesh::Mesh) -> Vec<(f32, f32)> {
    (0..=N)
        .map(|j| {
            let p = mesh.positions()[j * (N + 1) + N / 2];
            (p[1], p[2])
        })
        .collect()
}

/// A PROFUNDIDADE do vale: crista menos chão, na secção.
///
/// A crista é o maior `z` dentro de `|y| <= 2·W` (o ombro do vale, não a chapa
/// distante), e o chão é o menor `z` da secção inteira.
fn depth(sec: &[(f32, f32)]) -> (f32, f32, f32) {
    let ridge = sec
        .iter()
        .filter(|&&(y, _)| y.abs() <= 2.0 * W)
        .map(|&(_, z)| z)
        .fold(f32::NEG_INFINITY, f32::max);
    let floor = sec.iter().map(|&(_, z)| z).fold(f32::INFINITY, f32::min);
    (ridge, floor, ridge - floor)
}

fn stroke(verb: Verb, radius: f32, dabs: usize) -> ph2d_mesh::Mesh {
    stroke_lifted(verb, radius, dabs, 0.0)
}

/// O mesmo traço, com o lift EFETIVO do plano deslocado pelo knob do artista.
///
/// ⚠️ **O `plane_offset` SOMA ao [`ph2d_sculpt3d::STRIP_PLANE_FRACTION`]**
/// (`stroke_plane.rs`: `off = plane_offset · raio`), então varrê-lo é varrer o
/// lift **pela porta do produto**, sem tocar na constante.
fn stroke_lifted(verb: Verb, radius: f32, dabs: usize, plane_offset: f32) -> ph2d_mesh::Mesh {
    stroke_tuned(verb, radius, dabs, plane_offset, 1.0)
}

/// ⚠️ **O `gain` é emulado pela FORÇA, e isso é exato:** o depósito é linear no
/// `weight()` (`intensity = weight · pressão`, e o alvo é `add(live, n, reach ·
/// w)`), então multiplicar a força é multiplicar o depósito. É o que deixa a
/// sonda varrer um ganho que ainda não é uma constante do produto.
fn stroke_tuned(
    verb: Verb,
    radius: f32,
    dabs: usize,
    plane_offset: f32,
    gain: f32,
) -> ph2d_mesh::Mesh {
    let mut mesh = valley_grid();
    let brush = Brush {
        verb,
        radius,
        strength: 0.5 * gain,
        plane_offset,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..dabs {
        let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * radius;
        // O cursor pousa NA superfície, que no eixo do vale é o chão dele.
        s.dab(
            &mut mesh,
            &brush,
            &Dab::at([x, 0.0, valley_z(0.0)], radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    mesh
}

#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_whether_a_stroke_closes_the_valley_or_deepens_it() {
    let base = section(&valley_grid());
    let (r0, f0, d0) = depth(&base);
    println!("REPOUSO           crista {r0:+.4}  chao {f0:+.4}  PROFUNDIDADE {d0:.4}");
    println!();
    for radius in [0.35f32, 0.5, 0.8] {
        for verb in [Verb::ClayStrips, Verb::Draw] {
            for dabs in [1usize, 9] {
                let mesh = stroke(verb, radius, dabs);
                let sec = section(&mesh);
                let (r, f, d) = depth(&sec);
                let verdict = if d < d0 - 1e-4 {
                    "FECHA"
                } else if d > d0 + 1e-4 {
                    "AUMENTA"
                } else {
                    "igual"
                };
                let name = format!("{verb:?}");
                println!(
                    "r {radius:.2}  {name:<11}  dabs {dabs}  crista {r:+.4}  chao {f:+.4}  \
                     prof {d:.4}  ({:+.4})  {verdict}",
                    d - d0
                );
            }
        }
        println!();
    }
}

/// **O LIFT decide se a faixa ENCHE ou EXAGERA o relevo.**
///
/// A parábola `z·(1 − z)` cresce até `z = 0,5` e cai depois, e `z` é a
/// profundidade abaixo do plano em raios. Logo o depósito só aumenta com a
/// profundidade **enquanto o ponto está a menos de meio raio abaixo do plano**;
/// mais fundo que isso, quanto mais fundo MENOS tinta.
///
/// O plano fica a `lift` raios acima da média da pegada ⇒ a faixa enche relevo
/// até `(0,5 − lift)` raios abaixo da média, e **exagera o que passa disso**.
/// Com `lift = 0,5` essa folga é ZERO: nenhum vale enche.
///
/// Esta sonda mede a folga em vez de a deduzir, e mede junto o preço — quanto a
/// faixa deposita em chapa PLANA, que é o outro lado do trade.
#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_how_the_plane_lift_decides_fill_versus_exaggerate() {
    let (_, _, d0) = depth(&section(&valley_grid()));
    println!("lift   prof(vale)   delta   veredito     chapa plana (deposito)");
    for off in [-0.50f32, -0.40, -0.32, -0.25, -0.20, -0.10, 0.0] {
        let lift = ph2d_sculpt3d::STRIP_PLANE_FRACTION + off;
        let mesh = stroke_lifted(Verb::ClayStrips, 0.8, 9, off);
        let (_, _, d) = depth(&section(&mesh));
        let verdict = if d < d0 - 1e-4 {
            "FECHA  "
        } else if d > d0 + 1e-4 {
            "AUMENTA"
        } else {
            "igual  "
        };
        // A chapa plana: o MESMO traço longe do vale, onde a superfície é lisa.
        let flat = flat_deposit(0.8, 9, off);
        println!(
            "{lift:.2}   {d:.4}      {:+.4}  {verdict}      {flat:.4}",
            d - d0
        );
    }
}

/// **O LIFT com a MAGNITUDE presa** — a varredura que escolhe o número.
///
/// O ganho é `0,25 / (lift · (1 − lift))`: ele repõe no pico da parábola o que o
/// lift tira, de modo que **a chapa plana recebe exatamente o que recebia** e a
/// única coisa que a varredura move é a FORMA. Sem isso, baixar o lift muda
/// duas coisas ao mesmo tempo e o smoke não é legível.
#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_the_lift_with_the_magnitude_held_fixed() {
    let (_, _, d0) = depth(&section(&valley_grid()));
    // O que a faixa deposita hoje em chapa plana, o alvo a preservar.
    let today = flat_deposit(0.8, 9, 0.0);
    println!("hoje: lift 0,50, chapa plana {today:.4}, vale {:+.4}", {
        let (_, _, d) = depth(&section(&stroke(Verb::ClayStrips, 0.8, 9)));
        d - d0
    });
    println!();
    println!("lift   ganho   chapa plana   vale(delta)   folga de enchimento");
    for lift in [0.10f32, 0.15, 0.18, 0.22, 0.25, 0.30] {
        let gain = 0.25 / (lift * (1.0 - lift));
        let off = lift - ph2d_sculpt3d::STRIP_PLANE_FRACTION;
        let (_, _, d) = depth(&section(&stroke_tuned(Verb::ClayStrips, 0.8, 9, off, gain)));
        let flat = flat_deposit_tuned(0.8, 9, off, gain);
        // Quão fundo abaixo da MÉDIA a faixa ainda enche, em raios.
        let reach = 0.5 - lift;
        println!(
            "{lift:.2}   {gain:.3}   {flat:.4}        {:+.4}       {reach:.2} r",
            d - d0
        );
    }
}

/// Quanto a faixa levanta uma chapa PLANA — o outro lado do trade do lift.
fn flat_deposit(radius: f32, dabs: usize, plane_offset: f32) -> f32 {
    flat_deposit_tuned(radius, dabs, plane_offset, 1.0)
}

fn flat_deposit_tuned(radius: f32, dabs: usize, plane_offset: f32, gain: f32) -> f32 {
    let mut mesh = {
        let mut pos = Vec::with_capacity((N + 1) * (N + 1));
        for j in 0..=N {
            for i in 0..=N {
                let f = |k: usize| (k as f32 / N as f32) * 2.0 * HALF - HALF;
                pos.push([f(i), f(j), 0.0]);
            }
        }
        let at = |i: usize, j: usize| (j * (N + 1) + i) as u32;
        let mut faces = Vec::with_capacity(N * N * 2);
        for j in 0..N {
            for i in 0..N {
                faces.push(ph2d_mesh::Face::tri(
                    at(i, j),
                    at(i + 1, j),
                    at(i + 1, j + 1),
                ));
                faces.push(ph2d_mesh::Face::tri(
                    at(i, j),
                    at(i + 1, j + 1),
                    at(i, j + 1),
                ));
            }
        }
        ph2d_mesh::Mesh::from_parts(pos, faces).expect("indices validos")
    };
    let brush = Brush {
        verb: Verb::ClayStrips,
        radius,
        strength: 0.5 * gain,
        plane_offset,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..dabs {
        let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * radius;
        s.dab(
            &mut mesh,
            &brush,
            &Dab::at([x, 0.0, 0.0], radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    mesh.positions()
        .iter()
        .map(|p| p[2])
        .fold(f32::NEG_INFINITY, f32::max)
}

/// O perfil inteiro, para OLHAR a forma em vez de um escalar.
#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_the_valley_cross_section() {
    let base = section(&valley_grid());
    let clay = section(&stroke(Verb::ClayStrips, 0.5, 9));
    let draw = section(&stroke(Verb::Draw, 0.5, 9));
    println!("      y     repouso        strips          draw");
    for j in 0..=N {
        let y = base[j].0;
        if y.abs() > 1.2 {
            continue;
        }
        println!(
            "{y:+7.3}  {:+10.4}  {:+12.4}  {:+12.4}",
            base[j].1, clay[j].1, draw[j].1
        );
    }
}

/// **O OUTRO LADO: a superfície CONVEXA.**
///
/// Numa cúpula o miolo da pegada fica ACIMA da média e o aro ABAIXO. Como a
/// parábola vale zero no plano, um lift baixo faz o miolo receber quase nada e o
/// aro receber o pico: a faixa deixaria de ser uma banda e viraria um **ANEL**.
///
/// É esta a pressão que põe um PISO no lift — o enchimento sozinho é monótono e
/// não tem joelho. A sonda mede a razão `miolo ÷ aro` do depósito: `1` é uma
/// banda de topo chato, `< 1` é um anel.
const DOME_R: f32 = 1.5;
const DOME_H: f32 = 0.5;

fn dome_z(x: f32, y: f32) -> f32 {
    let q = (x * x + y * y) / (DOME_R * DOME_R);
    if q >= 1.0 { 0.0 } else { DOME_H * (1.0 - q) }
}

fn dome_grid() -> ph2d_mesh::Mesh {
    let mut pos = Vec::with_capacity((N + 1) * (N + 1));
    for j in 0..=N {
        for i in 0..=N {
            let f = |k: usize| (k as f32 / N as f32) * 2.0 * HALF - HALF;
            let (x, y) = (f(i), f(j));
            pos.push([x, y, dome_z(x, y)]);
        }
    }
    let at = |i: usize, j: usize| (j * (N + 1) + i) as u32;
    let mut faces = Vec::with_capacity(N * N * 2);
    for j in 0..N {
        for i in 0..N {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("indices validos")
}

#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_whether_a_low_lift_turns_the_band_into_a_ring() {
    let base = dome_grid();
    let before: Vec<f32> = section(&base).iter().map(|&(_, z)| z).collect();
    println!("lift   ganho   miolo    aro      miolo/aro   forma");
    for lift in [0.10f32, 0.15, 0.18, 0.22, 0.25, 0.30, 0.50] {
        let gain = 0.25 / (lift * (1.0 - lift));
        let off = lift - ph2d_sculpt3d::STRIP_PLANE_FRACTION;
        let mut mesh = dome_grid();
        let brush = Brush {
            verb: Verb::ClayStrips,
            radius: 0.8,
            strength: 0.5 * gain,
            plane_offset: off,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for k in 0..9 {
            let x = (k as f32 - 4.0) * 0.15 * 0.8;
            s.dab(
                &mut mesh,
                &brush,
                &Dab::at([x, 0.0, dome_z(x, 0.0)], 0.8, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        let after = section(&mesh);
        // O deslocamento por linha da secção, e onde ele cai.
        let disp: Vec<(f32, f32)> = after
            .iter()
            .zip(&before)
            .map(|(&(y, z), &z0)| (y, z - z0))
            .collect();
        let core = disp
            .iter()
            .filter(|&&(y, _)| y.abs() <= 0.15)
            .map(|&(_, d)| d)
            .fold(f32::NEG_INFINITY, f32::max);
        let rim = disp
            .iter()
            .filter(|&&(y, _)| (0.35..=0.85).contains(&y.abs()))
            .map(|&(_, d)| d)
            .fold(f32::NEG_INFINITY, f32::max);
        let ratio = if rim > 1e-6 {
            core / rim
        } else {
            f32::INFINITY
        };
        let shape = if ratio >= 0.95 {
            "banda"
        } else if ratio >= 0.75 {
            "banda achatada"
        } else {
            "ANEL"
        };
        println!("{lift:.2}   {gain:.3}   {core:.4}   {rim:.4}   {ratio:.3}       {shape}");
    }
}

/// **A FAIXA SATURA, ou CRESCE SEM LIMITE?**
///
/// Com o plano ajustado sobre a superfície VIVA, cada dab levanta o barro, o
/// plano sobe junto e o dab seguinte volta a levantar a partir do plano novo —
/// a geometria relativa nunca muda, então o depósito é o mesmo outra vez e a
/// crista cresce **linearmente com o número de dabs**. Com o plano do pen-down
/// o barro sobe até ele e PARA.
///
/// A referência (`sculpt.cc::calc_area_normal_and_center_node_mesh`) escolhe
/// entre os dois pelo `!ss.cache->accum`: **congelado com o Accumulate
/// desligado**, vivo com ele ligado.
#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_whether_the_strip_saturates_or_grows_without_bound() {
    println!("verbo         accum   1 dab    3       9       27      81      27/9");
    for verb in [Verb::ClayStrips, Verb::Clay, Verb::Draw] {
        for accumulate in [false, true] {
            let mut row = Vec::new();
            for dabs in [1usize, 3, 9, 27, 81] {
                let mut mesh = flat_grid();
                let brush = Brush {
                    verb,
                    radius: 0.5,
                    strength: 0.5,
                    accumulate,
                    ..Brush::default()
                };
                let mut s = SculptStroke::default();
                s.begin(&mesh);
                for k in 0..dabs {
                    let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * 0.5;
                    s.dab(
                        &mut mesh,
                        &brush,
                        &Dab::at([x, 0.0, 0.0], 0.5, [0.0, 0.0, -1.0]),
                        Symmetry::default(),
                    );
                }
                row.push(
                    mesh.positions()
                        .iter()
                        .map(|p| p[2])
                        .fold(f32::NEG_INFINITY, f32::max),
                );
            }
            let name = format!("{verb:?}");
            println!(
                "{name:<12}  {accumulate:<5}   {:.4}  {:.4}  {:.4}  {:.4}  {:.4}  {:.2}x",
                row[0],
                row[1],
                row[2],
                row[3],
                row[4],
                row[3] / row[2].max(1e-6)
            );
        }
    }
}

fn flat_grid() -> ph2d_mesh::Mesh {
    let mut pos = Vec::with_capacity((N + 1) * (N + 1));
    for j in 0..=N {
        for i in 0..=N {
            let f = |k: usize| (k as f32 / N as f32) * 2.0 * HALF - HALF;
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let at = |i: usize, j: usize| (j * (N + 1) + i) as u32;
    let mut faces = Vec::with_capacity(N * N * 2);
    for j in 0..N {
        for i in 0..N {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("indices validos")
}

/// **A SECÇÃO DA BANDA** — a forma que a faixa de facto deixa, atravessada.
///
/// Um clay strip é uma LAJE: topo chato, largura ~2 raios, altura pequena. Uma
/// LÂMINA é o oposto — estreita, alta, de borda dura. A sonda imprime o perfil
/// para se OLHAR, em vez de o inferir de um escalar.
#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_the_band_cross_section_on_flat_and_curved_clay() {
    for (nome, dabs) in [("9 dabs", 9usize), ("27 dabs", 27)] {
        let mut mesh = flat_grid();
        let brush = Brush {
            verb: Verb::ClayStrips,
            radius: 0.5,
            strength: 0.5,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for k in 0..dabs {
            let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * 0.5;
            s.dab(
                &mut mesh,
                &brush,
                &Dab::at([x, 0.0, 0.0], 0.5, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        let sec = section(&mesh);
        let peak = sec.iter().map(|&(_, z)| z).fold(0.0f32, f32::max);
        // A largura a meia altura, e a largura do PLATÔ (>= 90% do pico).
        let w_half = sec
            .iter()
            .filter(|&&(_, z)| z >= peak * 0.5)
            .map(|&(y, _)| y.abs())
            .fold(0.0f32, f32::max);
        let w_top = sec
            .iter()
            .filter(|&&(_, z)| z >= peak * 0.9)
            .map(|&(y, _)| y.abs())
            .fold(0.0f32, f32::max);
        println!(
            "{nome}: pico {peak:.4}  meia-largura {w_half:.3} ({:.2} r)  \
             platô {w_top:.3} ({:.2} r)  razão altura/largura {:.3}",
            w_half / 0.5,
            w_top / 0.5,
            peak / (2.0 * w_half.max(1e-6))
        );
        print!("  perfil:");
        for &(y, z) in &sec {
            if y.abs() <= 0.85 && (y * 100.0).round() as i32 % 5 == 0 {
                print!(" {z:.3}");
            }
        }
        println!();
    }
}

/// **A FAIXA DEPOSITA NAS COSTAS?** — o terceiro termo da cadeia de fatores da
/// referência (`clay_strips.cc::calc_faces` → `calc_front_face`).
///
/// O olho é RASANTE, que é a situação da foto: o artista olha um membro de lado
/// e passa a faixa perto da silhueta. Um vértice de costas tem `n · (−eye) < 0`
/// e a referência zera o fator dele (`factors[i] *= max(dot, 0)`).
#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_whether_the_strip_deposits_on_back_facing_clay() {
    // Olhar quase de lado: é assim que a silhueta entra na pegada.
    let eye = {
        let v = [1.0f32, 0.0, -0.25];
        let l = (v[0] * v[0] + v[2] * v[2]).sqrt();
        [v[0] / l, 0.0, v[2] / l]
    };
    println!("modo   deposito FRENTE   deposito COSTAS   costas/frente");
    for mode in [ph2d_sculpt3d::RefMode::S, ph2d_sculpt3d::RefMode::B] {
        let rest = dome_grid();
        let mut mesh = dome_grid();
        let brush = Brush {
            verb: Verb::ClayStrips,
            radius: 0.8,
            strength: 0.5,
            mode,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // O traço corre pela LOMBADA, atravessando a silhueta vista do olho.
        for k in 0..9 {
            let y = (k as f32 - 4.0) * 0.15 * 0.8;
            s.dab(
                &mut mesh,
                &brush,
                &Dab::at([0.9, y, dome_z(0.9, y)], 0.8, eye),
                Symmetry::default(),
            );
        }
        let (mut front, mut back) = (0.0f32, 0.0f32);
        for (i, p) in mesh.positions().iter().enumerate() {
            let d = (p[0] - rest.positions()[i][0]).abs()
                + (p[1] - rest.positions()[i][1]).abs()
                + (p[2] - rest.positions()[i][2]).abs();
            if d <= 1e-6 {
                continue;
            }
            let n = rest.normals()[i];
            let facing = -(n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2]);
            if facing >= 0.0 { front += d } else { back += d }
        }
        println!(
            "{mode:?}      {front:9.4}         {back:9.4}         {:.3}",
            back / front.max(1e-9)
        );
    }
}
