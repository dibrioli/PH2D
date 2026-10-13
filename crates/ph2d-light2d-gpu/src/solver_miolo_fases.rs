//! ⭐ W5o — o INSTRUMENTO das fases do miolo ([`Solver::medir_fases_do_miolo`]): cada corrida de
//! passos da mesma fase no seu passe, com carimbos próprios. Módulo-filho de `solver_miolo.rs` (o
//! tecto de 700 LOC).

use super::*;

/// ⭐ W5o — as fases do instrumento [`Solver::medir_fases_do_miolo`], na ordem de gravação.
const FASES: [&str; 8] = [
    "classifica+puxa",
    "cascata: mais grosso",
    "cascata: desce",
    "ciclo: defeito+restringe",
    "ciclo: mais grosso",
    "ciclo: sobe",
    "ciclo: soma",
    "final",
];

/// ⭐ W5o — o instrumento: cada corrida de passos da mesma fase no seu passe de cômputo, com um par
/// de carimbos próprio.
pub(super) struct Fases {
    pub(super) set: wgpu::QuerySet,
    pub(super) resolve: wgpu::Buffer,
    pub(super) leitura: wgpu::Buffer,
    pub(super) periodo_ns: f64,
    pub(super) corridas: Vec<(usize, std::ops::Range<usize>)>,
}

impl DoMiolo {
    pub(super) fn ligar_fases(&mut self, gpu: &GpuContext) {
        let mut corridas: Vec<(usize, std::ops::Range<usize>)> = Vec::new();
        for (i, p) in self.passos.iter().enumerate() {
            match corridas.last_mut() {
                Some((f, r)) if *f == p.fase => r.end = i + 1,
                _ => corridas.push((p.fase, i..i + 1)),
            }
        }
        let n = 2 * corridas.len() as u32;
        let buffer = |label, usage| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: u64::from(n) * 8,
                usage,
                mapped_at_creation: false,
            })
        };
        self.fases = Some(Fases {
            set: gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("luz2d/fases do miolo"),
                ty: wgpu::QueryType::Timestamp,
                count: n,
            }),
            resolve: buffer(
                "luz2d/fases do miolo resolve",
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            leitura: buffer(
                "luz2d/fases do miolo leitura",
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            periodo_ns: f64::from(gpu.queue.get_timestamp_period()),
            corridas,
        });
    }

    pub(super) fn ler_fases(&self, gpu: &GpuContext) -> Option<Vec<(&'static str, f64, usize)>> {
        let f = self.fases.as_ref()?;
        let fatia = f.leitura.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |r| r.expect("map das fases do miolo"));
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let ts: Vec<u64> = fatia
            .get_mapped_range()
            .as_chunks::<8>()
            .0
            .iter()
            .map(|b| u64::from_le_bytes(*b))
            .collect();
        f.leitura.unmap();
        let mut out: Vec<(&'static str, f64, usize)> = FASES.iter().map(|&n| (n, 0.0, 0)).collect();
        for (c, (fase, r)) in f.corridas.iter().enumerate() {
            let ns = ts[2 * c + 1].wrapping_sub(ts[2 * c]) as f64 * f.periodo_ns;
            out[*fase].1 += ns / 1e6;
            out[*fase].2 += r.len();
        }
        Some(out)
    }
}

impl Solver {
    /// ⭐ W5o — INSTRUMENTO: daqui em diante cada fase do miolo grava no seu passe, com carimbos
    /// próprios (o `c3_miolo` dos [`TemposMs`] fica 0; a fronteira de passe soma um pouco). `false`
    /// sem o miolo ou sem `TIMESTAMP_QUERY`.
    #[doc(hidden)]
    pub fn medir_fases_do_miolo(&mut self, gpu: &GpuContext) -> bool {
        let pode = gpu
            .device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY);
        match self.miolo.as_mut().filter(|_| pode) {
            Some(m) => {
                m.ligar_fases(gpu);
                true
            }
            None => false,
        }
    }

    /// ⭐ W5o — as fases do miolo do último quadro submetido: (nome, ms de placa, despachos).
    #[doc(hidden)]
    #[must_use]
    pub fn fases_do_miolo(&self, gpu: &GpuContext) -> Option<Vec<(&'static str, f64, usize)>> {
        self.miolo.as_ref()?.ler_fases(gpu)
    }
}
