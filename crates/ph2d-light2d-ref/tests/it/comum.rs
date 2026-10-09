//! Construtores de fixture partilhados.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};

pub fn disco(centro: [f32; 2], raio: f32, radiancia: [f32; 3], alcance_m: f32) -> Emissor {
    Emissor::Disco {
        centro,
        raio,
        radiancia,
        alcance_m,
    }
}

pub fn cena(emissores: Vec<Emissor>, oclusores: Vec<Oclusor>) -> CenaDeLuz {
    CenaDeLuz {
        emissores,
        oclusores,
        ceu: [0.0; 3],
        ..Default::default()
    }
}

pub fn f64x2(p: [f32; 2]) -> [f64; 2] {
    [f64::from(p[0]), f64::from(p[1])]
}

/// Rotação por `phi` + translação por `origem`, arredondada a `f32` (a precisão da cena).
pub fn para_mundo(phi: f64, origem: [f32; 2], local: [f64; 2]) -> [f32; 2] {
    let (s, c) = phi.sin_cos();
    [
        (c * local[0] - s * local[1] + f64::from(origem[0])) as f32,
        (s * local[0] + c * local[1] + f64::from(origem[1])) as f32,
    ]
}

pub fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
