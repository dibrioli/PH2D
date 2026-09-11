//! ⭐⭐⭐ **SONDA — o `Expand` é o único dos cinco que não SATURA, e quanto?**
//!
//! O item aberto diz que ele explode, e **três documentos escritos em dias
//! consecutivos dão três números diferentes** (`753×`, `155×`, `55×` de esticão).
//! ⛔ Nenhum deles se cita aqui: esta sonda mede o de HOJE, nos valores de
//! FÁBRICA, e é ela a fonte.
//!
//! # O mecanismo que ela existe para expor
//!
//! O tecto de esticão compara `|aresta|` contra `tecto × ℓ`, e o `ℓ` que ele usa
//! é o comprimento de repouso **corrente** — `ℓ_material + τ`. O *Expand* é o
//! único dos cinco que mexe no `τ`. ⇒ *o denominador da régua cresce com a lei
//! que ela devia limitar*, e o tecto lê «não está esticado» enquanto a peça
//! incha. É a forma do espelho que não acusa.
//!
//! ⇒ por isso a coluna que decide é o **esticão contra o material** (o repouso
//! ORIGINAL), nunca contra o repouso corrente.
//!
//! ⛔ **A sonda é `#[ignore]` e não afirma nada** — imprime a tabela. Quem
//! escrever uma barra a partir dela tem de dizer de que RECURSO ela é.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test sonda_do_expand_que_nao_para -- --ignored
//! --nocapture
//! ```

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{ClothFilterKind, ClothFilterProps, ClothFilterStep, SculptStroke};

fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 64, 1.0)
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// A maior razão `|aresta| / |aresta do MATERIAL|`.
///
/// ⚠️ **Contra o material, não contra o repouso corrente** — é essa a diferença
/// que o tecto não vê.
fn estica(rest: &Mesh, now: &Mesh) -> f64 {
    let (a, b) = (rest.positions(), now.positions());
    let mut mx: f64 = 0.0;
    for f in rest.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (i, j) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
            let l0 = dist(a[i], a[j]);
            if l0 > 1e-9 {
                mx = mx.max(f64::from(dist(b[i], b[j]) / l0));
            }
        }
    }
    mx
}

/// O volume com sinal da peça.
fn volume(m: &Mesh) -> f64 {
    let mut t = Vec::new();
    for f in m.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            t.push([v[0], v[k], v[k + 1]]);
        }
    }
    let x: Vec<[f64; 3]> = m
        .positions()
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
        .collect();
    ph2d_cloth::verlet::volume_de(&x, &t)
}

/// `n` arrastos completos, cada um com `s` de zero a `1,0` em `120` passos —
/// mil pixels de arrasto, que é o curso inteiro.
fn correr(props: ClothFilterProps, kind: ClothFilterKind, gestos: usize) -> Mesh {
    let mut m = esfera();
    for _ in 0..gestos {
        let mut st = SculptStroke::default();
        st.cloth_filter_begin(&m, props, kind, [0.0, 0.9, 0.45]);
        for k in 0..120 {
            let passo = ClothFilterStep {
                s: (k as f32 + 1.0) / 120.0,
                gravity_axis: [0.0, -1.0, 0.0],
                frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                axes: [true, true, true],
                eye: [0.0, 0.0, 1.0],
            };
            st.cloth_filter_step(&mut m, kind, &passo);
        }
        st.cloth_filter_end();
    }
    m
}

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn quais_dos_cinco_saturam() {
    let repouso = esfera();
    let v0 = volume(&repouso);
    let tipos = [
        ("Gravity", ClothFilterKind::Gravity),
        ("Inflate", ClothFilterKind::Inflate),
        ("Expand", ClothFilterKind::Expand),
        ("Pinch", ClothFilterKind::Pinch),
        ("Scale", ClothFilterKind::Scale),
    ];

    println!("\n=== VALORES DE FABRICA (tecto 1,10 ligado, dobra 0, volume off) ===");
    println!(
        "{:>8} {:>7} | {:>12} {:>12}",
        "tipo", "gestos", "estica(mat)", "volume/V0"
    );
    println!("{}", "-".repeat(46));
    for (nome, k) in tipos {
        for gestos in [1usize, 2, 3] {
            let m = correr(ClothFilterProps::default(), k, gestos);
            let e = estica(&repouso, &m);
            let v = volume(&m) / v0;
            println!("{nome:>8} {gestos:>7} | {e:>12.3} {v:>12.3}");
        }
    }

    println!("\n=== O TECTO MOVE O EXPAND? (1 gesto, tecto na faixa toda) ===");
    println!(
        "{:>10} | {:>12} {:>12}",
        "tecto", "estica(mat)", "volume/V0"
    );
    println!("{}", "-".repeat(40));
    for tecto in [
        ClothFilterProps::STRETCH.0,
        1.10f32,
        1.50,
        ClothFilterProps::STRETCH.1,
    ] {
        let props = ClothFilterProps {
            stretch_max: tecto,
            ..ClothFilterProps::default()
        };
        let m = correr(props, ClothFilterKind::Expand, 1);
        println!(
            "{tecto:>10.2} | {:>12.3} {:>12.3}",
            estica(&repouso, &m),
            volume(&m) / v0
        );
    }
    println!(
        "\n⇒ se a coluna do tecto mal se mexer, esta' medido que ele NAO alcanca o \
         Expand:\n  ele compara contra `l_material + tau`, e o `tau` e' o que o \
         Expand cresce.\n"
    );
}
