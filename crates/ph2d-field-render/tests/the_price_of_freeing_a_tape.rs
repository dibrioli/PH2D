//! ⭐⭐⭐ **QUANTO CUSTA LIBERTAR UMA FITA — e de que isso depende** (W89).
//!
//! # A pergunta que escolhe a cura
//!
//! O despejo da cache custa `270–365 ms` para `~1 700` fitas ⇒ **`~150 µs` cada**, o que é absurdo
//! para uma libertação de memória. Se esse preço for **intrínseco**, espalhar o despejo não cura
//! nada (a população roda `~100` fitas por quadro ⇒ `15 ms` de orçamento comidos por quadro, para
//! sempre). Se for **contenção**, a cura é tirá-lo do caminho de quem desenha.
//!
//! ⚠️ A hipótese tem mecanismo: uma fita é um `mmap` de código executável, e `munmap` num processo
//! com 32 threads **vivas** obriga a invalidar a TLB de todos os núcleos (IPI a cada um, e espera).
//! Com as threads paradas o mesmo `munmap` não fala com ninguém.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test the_price_of_freeing_a_tape -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
use ph2d_field_eval::{RegionCompiler, hybrid::RegionTape};

fn peca() -> FieldDoc {
    let c: Vec<[f32; 2]> = (0..168)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / 168.0;
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![c], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("extrusão")
}

fn constroi(n: usize, rc: &RegionCompiler, doc: &FieldDoc) -> Vec<RegionTape> {
    (0..n)
        .map(|i| {
            // Regiões pequenas e distintas, como as de um ladrilho.
            let t = (i as f32) / (n as f32) - 0.5;
            let lo = [t, -0.1 + t * 0.1, -0.1];
            let hi = [t + 0.08, 0.1 + t * 0.1, 0.1];
            RegionTape::compile(rc.compile(doc, lo, hi))
        })
        .collect()
}

#[test]
#[ignore = "sonda; roda com --nocapture"]
fn measure_whether_freeing_a_tape_is_intrinsic_or_contention() {
    const N: usize = 1700;
    let doc = peca();
    let rc = RegionCompiler::new(&doc);
    // (a) MÁQUINA PARADA
    let v = constroi(N, &rc, &doc);
    let t0 = std::time::Instant::now();
    drop(v);
    let parada = t0.elapsed().as_secs_f64() * 1000.0;
    // (a2) SÓ AS ÁRVORES — a metade que a rota do produto nunca lê.
    let arvores: Vec<_> = (0..N)
        .map(|i| {
            let t = (i as f32) / (N as f32) - 0.5;
            rc.compile(
                &doc,
                [t, -0.1 + t * 0.1, -0.1],
                [t + 0.08, 0.1 + t * 0.1, 0.1],
            )
        })
        .collect();
    let t0 = std::time::Instant::now();
    drop(arvores);
    let so_arvores = t0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "libertar {N} ÁRVORES (sem fita)       | {so_arvores:8.1} ms | {:6.1} µs cada",
        so_arvores * 1000.0 / N as f64
    );
    // (b) COM AS THREADS VIVAS — o que acontece dentro do laço de ladrilhos.
    let v = constroi(N, &rc, &doc);
    let pare = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut fios = Vec::new();
    for _ in 0..31 {
        let p = std::sync::Arc::clone(&pare);
        fios.push(std::thread::spawn(move || {
            // Trabalho a sério: toca memória, como a marcha faz.
            let mut buf = vec![0.0f32; 1 << 16];
            let mut k = 0usize;
            while !p.load(std::sync::atomic::Ordering::Relaxed) {
                k = (k + 1) & (buf.len() - 1);
                buf[k] = buf[k].mul_add(1.000_001, 1.0);
            }
            buf[0]
        }));
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    let t0 = std::time::Instant::now();
    drop(v);
    let ocupada = t0.elapsed().as_secs_f64() * 1000.0;
    pare.store(true, std::sync::atomic::Ordering::Relaxed);
    for f in fios {
        let _ = f.join();
    }
    println!(
        "libertar {N} fitas — máquina PARADA   | {parada:8.1} ms | {:6.1} µs cada",
        parada * 1000.0 / N as f64
    );
    println!(
        "libertar {N} fitas — 31 threads VIVAS | {ocupada:8.1} ms | {:6.1} µs cada",
        ocupada * 1000.0 / N as f64
    );
    println!("razão: {:.1}x", ocupada / parada.max(1e-9));
}
