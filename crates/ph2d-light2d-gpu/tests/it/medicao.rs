//! O arnês de RELÓGIO (`docs/DevOps/MEDIR_VELOCIDADE.md`): variantes no MESMO processo, aquecidas
//! fora da régua, intercaladas em rodadas com a ordem rodada; vale o MÍNIMO, a mediana ao lado.
//! A régua é o relógio da PLACA (`TIMESTAMP_QUERY`, um par de carimbos por passe).

use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Solver, TemposMs};

pub struct Caso {
    pub nome: String,
    pub solver: Solver,
    pub cena: CenaDeLuz,
}

/// O resumo de um caso: mínimo e mediana por grandeza, em ms.
#[derive(Clone, Debug)]
pub struct Resumo {
    pub nome: String,
    pub c0: [f64; 2],
    pub c1: [f64; 2],
    pub c2: [f64; 2],
    /// C1 + C2 — o solver propriamente dito.
    pub rc: [f64; 2],
    pub c2_por_nivel: Vec<f64>,
}

fn min_med(mut v: Vec<f64>) -> [f64; 2] {
    v.sort_by(f64::total_cmp);
    [v[0], v[v.len() / 2]]
}

pub const AQUECIMENTO: usize = 3;

/// `None` se a placa não tem `TIMESTAMP_QUERY`.
pub fn rodadas(gpu: &GpuContext, casos: &mut [Caso], rodadas: usize) -> Option<Vec<Resumo>> {
    for c in casos.iter_mut() {
        for _ in 0..AQUECIMENTO {
            c.solver.quadro(gpu, &c.cena);
            c.solver.tempos(gpu)?;
        }
    }
    let n = casos.len();
    let mut amostras: Vec<Vec<TemposMs>> = vec![Vec::with_capacity(rodadas); n];
    for r in 0..rodadas {
        for o in 0..n {
            let k = (o + r) % n;
            let c = &mut casos[k];
            c.solver.quadro(gpu, &c.cena);
            amostras[k].push(c.solver.tempos(gpu)?);
        }
    }
    Some(
        casos
            .iter()
            .zip(amostras)
            .map(|(c, a)| {
                let niveis = a[0].c2.len();
                Resumo {
                    nome: c.nome.clone(),
                    c0: min_med(a.iter().map(|t| t.c0).collect()),
                    c1: min_med(a.iter().map(|t| t.c1).collect()),
                    c2: min_med(a.iter().map(TemposMs::c2_total).collect()),
                    rc: min_med(a.iter().map(TemposMs::rc).collect()),
                    c2_por_nivel: (0..niveis)
                        .map(|i| min_med(a.iter().map(|t| t.c2[i]).collect())[0])
                        .collect(),
                }
            })
            .collect(),
    )
}

pub fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

/// O perfil de build. A régua é o relógio da PLACA (os shaders não mudam com o perfil); o perfil
/// só pesa no lado CPU (captura, subida), que esta régua não mede.
pub fn perfil() -> &'static str {
    if cfg!(debug_assertions) {
        "de build DEBUG (opt 0; a régua é a placa)"
    } else {
        "de build otimizado (smoke/release)"
    }
}
