//! **G10 (gabarito)** — duas corridas bit-idênticas, em threads diferentes, sobre uma cena com
//! todos os ingredientes (disco, ponto, polígono emissor, oclusor côncavo parcial, alcance finito,
//! céu).

use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_ref::fluencia;

fn cena_cheia() -> CenaDeLuz {
    CenaDeLuz {
        emissores: vec![
            Emissor::Disco {
                centro: [-2.0, 1.5],
                raio: 0.4,
                radiancia: [3.0, 2.5, 1.0],
                alcance_m: f32::INFINITY,
            },
            Emissor::Disco {
                centro: [2.5, -1.0],
                raio: 0.25,
                radiancia: [0.5, 1.0, 4.0],
                alcance_m: 4.0,
            },
            Emissor::Ponto {
                centro: [0.3, 2.7],
                intensidade: [1.0, 0.2, 0.2],
                alcance_m: 6.0,
            },
            Emissor::Poligono {
                vertices: vec![[1.0, 1.8], [1.6, 2.1], [1.3, 2.6], [0.8, 2.3]],
                radiancia: [0.0, 2.0, 0.0],
                alcance_m: f32::INFINITY,
            },
        ],
        oclusores: vec![
            // Um «U» girado: côncavo, raios que o atravessam duas vezes.
            Oclusor {
                vertices: vec![
                    [-0.5, -0.5],
                    [0.9, -0.2],
                    [0.8, 0.3],
                    [-0.1, 0.1],
                    [-0.3, 0.9],
                    [0.6, 1.1],
                    [0.5, 1.6],
                    [-0.9, 1.3],
                ],
                opacidade: 0.6,
            },
            Oclusor {
                vertices: vec![[-1.2, -1.8], [-0.4, -2.0], [-0.6, -1.1]],
                opacidade: 1.0,
            },
        ],
        ceu: [0.05, 0.06, 0.1],
    }
}

fn corrida() -> Vec<u64> {
    let cena = cena_cheia();
    let mut bits = Vec::new();
    for j in 0..16 {
        for i in 0..16 {
            let p = [-3.2 + 0.41 * f64::from(i), -2.9 + 0.37 * f64::from(j)];
            bits.extend(fluencia(&cena, p).map(f64::to_bits));
        }
    }
    bits
}

/// Medida (não gate): tempo do gabarito por ponto na cena cheia, o MÍNIMO de 7 rodadas
/// (`docs/DevOps/MEDIR_VELOCIDADE.md`). `cargo test -p ph2d-light2d-ref --release -- --ignored
/// --nocapture medida_tempo`.
#[test]
#[ignore = "medida, não gate"]
fn medida_tempo_por_ponto() {
    let mut minimo = f64::INFINITY;
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        std::hint::black_box(corrida());
        minimo = minimo.min(t0.elapsed().as_secs_f64());
    }
    eprintln!(
        "gabarito: {:.1} µs por ponto (cena cheia, mínimo de 7 rodadas de 256 pontos)",
        minimo / 256.0 * 1e6
    );
}

#[test]
fn g10_duas_corridas_dao_os_mesmos_bits() {
    let a = std::thread::spawn(corrida).join().expect("corrida A");
    let b = std::thread::spawn(corrida).join().expect("corrida B");
    // Controlo: a cena acende e varia (uma resposta constante passaria em qualquer gate de bits).
    let valores: Vec<f64> = a.iter().map(|&x| f64::from_bits(x)).collect();
    assert!(valores.iter().all(|v| v.is_finite() && *v >= 0.0));
    let distintos = {
        let mut v = a.clone();
        v.sort_unstable();
        v.dedup();
        v.len()
    };
    assert!(
        distintos > 500,
        "controlo: só {distintos} valores distintos em 768"
    );
    assert_eq!(a, b, "duas corridas do gabarito divergiram");
}
