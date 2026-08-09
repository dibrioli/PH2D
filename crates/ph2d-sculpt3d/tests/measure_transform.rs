//! **A FASE 0 DO TRANSFORM** — a LEI da interpolação, medida antes de escolher.
//!
//! O `Transform.js` da referência aplica a matriz e **lerpa a POSIÇÃO** pelo
//! peso da máscara (`vAr[j] = x*(1-m) + transformado*m`). Para uma translação
//! isso é exato: o lerp de duas translações é uma translação. Para uma
//! **ROTAÇÃO** não é — o lerp corta pela **CORDA**, e um ponto de peso `w` cai
//! *dentro* do círculo em vez de sobre ele.
//!
//! É a mesma doença que a `line/FLIP` nomeou no tween v2 (*"um lerp corta pela
//! CORDA, então todo giro encolhia o traço"*), aqui na banda de transição da
//! máscara — que é **exatamente onde o artista pinta com pincel macio**.
//!
//! A previsão fechada: um vértice a distância `r` do eixo, com peso `w`, girado
//! por `θ`, fica a `r` sob a lei fracionária e a
//! `r·√((1−w)² + w² + 2w(1−w)·cos θ)` sob o lerp — a norma da soma convexa de
//! dois unitários separados por `θ`.
//!
//! ⚠️ **A primeira versão desta sonda previa `r·cos(w·θ/2)` e a medição a
//! derrubou** (0,70711 medido contra 0,92388 previsto em `θ=90°, w=0,5`). Eu
//! derivei a corda por ANALOGIA com o meio-ângulo, e a corda do meio-ângulo é o
//! caso `w = ½` **do ângulo inteiro**, não o ângulo reduzido. A fórmula acima é
//! a verdadeira, e a coluna `previsto` bate com o `r lerp` em toda linha — é
//! isso que a torna um oráculo em vez de uma segunda medição.
//!
//! A segunda pergunta é o **CUSTO**: o transform é `O(vértices)` por evento de
//! ponteiro (não `O(pegada)` como um dab), então ele é a primeira operação deste
//! módulo cujo preço é o do DOCUMENTO. Se ele não couber num quadro, o gesto não
//! existe ao vivo.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_transform -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes};
use std::time::Instant;

/// Roda `p` em torno do eixo `axis` (unitário) por `radians` — Rodrigues.
fn rotate(p: [f32; 3], axis: [f32; 3], radians: f32) -> [f32; 3] {
    let (s, c) = radians.sin_cos();
    let d = axis[0] * p[0] + axis[1] * p[1] + axis[2] * p[2];
    let cross = [
        axis[1] * p[2] - axis[2] * p[1],
        axis[2] * p[0] - axis[0] * p[2],
        axis[0] * p[1] - axis[1] * p[0],
    ];
    [
        p[0] * c + cross[0] * s + axis[0] * d * (1.0 - c),
        p[1] * c + cross[1] * s + axis[1] * d * (1.0 - c),
        p[2] * c + cross[2] * s + axis[2] * d * (1.0 - c),
    ]
}

/// A distância ao eixo Y — a grandeza que um giro em torno de Y **preserva**.
fn radius_from_y_axis(p: [f32; 3]) -> f32 {
    p[0].mul_add(p[0], p[2] * p[2]).sqrt()
}

/// Uma máscara SUAVE: o peso livre varre 0..1 com a latitude, então a fixture
/// contém a banda de transição inteira — e é ela que a lei decide.
///
/// Devolve o peso `w = 1 - mask` por vértice (a convenção NOSSA: `0` livre,
/// `1` protegido — a inversa da referência).
fn soft_mask(mesh: &mut Mesh) -> Vec<f32> {
    let n = mesh.vert_count();
    let w: Vec<f32> = (0..n)
        .map(|i| (0.5 + mesh.positions()[i][1]).clamp(0.0, 1.0))
        .collect();
    let m = mesh.masks_mut();
    for i in 0..n {
        m[i] = 1.0 - w[i];
    }
    w
}

/// O vértice cujo peso está mais perto de `target` — a sonda da banda.
fn probe_at_weight(w: &[f32], target: f32) -> usize {
    let mut best = 0;
    let mut best_d = f32::INFINITY;
    for (i, &wi) in w.iter().enumerate() {
        let d = (wi - target).abs();
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

#[test]
#[ignore = "sonda"]
fn measure_the_law_of_a_weighted_rotation() {
    let mut mesh = shapes::uv_sphere(64, 96, 1.0);
    let w = soft_mask(&mut mesh);
    let axis = [0.0, 1.0, 0.0];

    println!("\n== o LERP contra a ROTACAO FRACIONARIA (esfera 64x96) ==");
    println!("um vertice a distancia r do eixo TEM de ficar a r depois de girar.\n");
    println!(
        "{:>6}  {:>5}  {:>9}  {:>9}  {:>9}  {:>9}",
        "theta", "w", "r antes", "r frac", "r lerp", "previsto"
    );
    for deg in [30.0f32, 90.0, 150.0, 180.0] {
        let theta = deg.to_radians();
        for wt in [0.25f32, 0.5, 0.75] {
            let i = probe_at_weight(&w, wt);
            let p = mesh.positions()[i];
            let wi = w[i];
            let r0 = radius_from_y_axis(p);

            // A lei FRACIONARIA: gira o proprio angulo reduzido.
            let frac = rotate(p, axis, wi * theta);
            // A lei da REFERENCIA: gira inteiro e lerpa a posicao.
            let full = rotate(p, axis, theta);
            let lerp = [
                p[0] + (full[0] - p[0]) * wi,
                p[1] + (full[1] - p[1]) * wi,
                p[2] + (full[2] - p[2]) * wi,
            ];
            // A previsao fechada do encolhimento do lerp: a norma da soma
            // convexa de dois unitarios separados por `theta`.
            let k = wi;
            let predicted =
                r0 * ((1.0 - k) * (1.0 - k) + k * k + 2.0 * k * (1.0 - k) * theta.cos()).sqrt();

            println!(
                "{deg:>5}o  {wi:>5.3}  {r0:>9.5}  {:>9.5}  {:>9.5}  {predicted:>9.5}",
                radius_from_y_axis(frac),
                radius_from_y_axis(lerp),
            );
        }
    }
}

#[test]
#[ignore = "sonda"]
fn measure_what_a_weighted_transform_costs() {
    println!("\n== o CUSTO: o transform e' O(VERTICES), nao O(pegada) ==");
    println!("congelar = UMA vez por gesto | aplicar+refresh = a cada evento\n");
    println!(
        "{:>20}  {:>8}  {:>8}  {:>10}  {:>10}  {:>10}",
        "malha", "verts", "w > 0", "congelar", "aplicar", "refresh"
    );
    for (name, seg, ring) in [
        ("uv_sphere(64,96)", 64usize, 96usize),
        ("uv_sphere(128,192)", 128, 192),
        ("uv_sphere(256,384)", 256, 384),
    ] {
        let mut mesh = shapes::uv_sphere(seg, ring, 1.0);
        let w = soft_mask(&mut mesh);
        let n = mesh.vert_count();
        let moving = w.iter().filter(|&&x| x > 0.0).count();

        // CONGELAR: a foto do pen-down (posicoes + pesos + os indices vivos).
        let t = Instant::now();
        let pre: Vec<[f32; 3]> = mesh.positions().to_vec();
        let idx: Vec<u32> = (0..n as u32).filter(|&i| w[i as usize] > 0.0).collect();
        let freeze = t.elapsed().as_secs_f64() * 1e3;

        // APLICAR: a lei fracionaria sobre os vivos, uma vez.
        let axis = [0.0, 1.0, 0.0];
        let theta = 0.6f32;
        let t = Instant::now();
        {
            let out = mesh.positions_mut();
            for &i in &idx {
                let k = i as usize;
                out[k] = rotate(pre[k], axis, w[k] * theta);
            }
        }
        let apply = t.elapsed().as_secs_f64() * 1e3;

        // ⚠️ **O refresh NAO e' um extra: ele e' a outra metade do evento.** Um
        // dab mexe na pegada e o refresh e' local; o transform mexe em DOIS
        // TERCOS da malha, entao a vizinhanca a re-descobrir e' quase toda ela.
        let mut scratch = ph2d_mesh::RegionScratch::default();
        let t = Instant::now();
        mesh.refresh_region(&idx, &mut scratch);
        let refresh = t.elapsed().as_secs_f64() * 1e3;

        println!(
            "{name:>20}  {n:>8}  {moving:>8}  {freeze:>9.3}ms  {apply:>9.3}ms  {refresh:>9.3}ms"
        );
    }
    println!("\n(um quadro de 60 fps tem 16,6 ms.)");
}
