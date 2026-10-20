//! ⭐ W5q — o INSTRUMENTO das etapas da holográfica ([`Solver::medir_fases_da_hrc`]): cada despacho
//! no seu passe, com carimbos próprios, somados por (etapa, nível). Módulo-filho de `solver_hrc.rs`.

use super::*;

/// ⭐ W5q — os carimbos por despacho; `gravacoes` = quantas vezes a cascata corre no quadro (os
/// rebotes a mais gravam-na de novo).
pub(super) struct FasesHrc {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    leitura: wgpu::Buffer,
    periodo_ns: f64,
    gravacoes: u32,
    /// As gravações que o último quadro fez (sem refletor o rebote a mais não corre): só os carimbos
    /// delas se resolvem — um carimbo nunca escrito prende a resolução da placa para sempre.
    feitas: std::cell::Cell<u32>,
}

/// (etapa, nível, ms de placa somados nos quadrantes e nas gravações, despachos).
pub type FaseHrc = (&'static str, u32, f64, usize);

impl Holografica {
    pub(super) fn ligar_fases(&mut self, gpu: &GpuContext, gravacoes: u32) {
        let n = 2 * self.passos.len() as u32 * gravacoes;
        let buffer = |label, usage| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: u64::from(n) * 8,
                usage,
                mapped_at_creation: false,
            })
        };
        self.fases = Some(FasesHrc {
            set: gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("luz2d/fases da holográfica"),
                ty: wgpu::QueryType::Timestamp,
                count: n,
            }),
            resolve: buffer(
                "luz2d/fases da holográfica resolve",
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            leitura: buffer(
                "luz2d/fases da holográfica leitura",
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            periodo_ns: f64::from(gpu.queue.get_timestamp_period()),
            gravacoes,
            feitas: std::cell::Cell::new(0),
        });
    }

    /// A gravação `k` do quadro com um passe por despacho; `false` sem o instrumento (grave pelo
    /// [`Self::gravar`]).
    pub(crate) fn gravar_por_fase(&self, enc: &mut wgpu::CommandEncoder, k: u32) -> bool {
        let Some(f) = self.fases.as_ref().filter(|f| k < f.gravacoes) else {
            return false;
        };
        let base = 2 * k * self.passos.len() as u32;
        for (i, (etapa, grupo, [x, y, z])) in self.passos.iter().enumerate() {
            let par = base + 2 * i as u32;
            // ⭐ W5s: a máscara só na 1.ª gravação (um passe vazio escreve os carimbos dela).
            let vazio = k > 0 && matches!(etapa, Etapa::Ladrilhos);
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(ENTRADAS[*etapa as usize]),
                timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                    query_set: &f.set,
                    beginning_of_pass_write_index: Some(par),
                    end_of_pass_write_index: Some(par + 1),
                }),
            });
            if vazio {
                continue;
            }
            p.set_bind_group(0, &self.cena, &[]);
            p.set_pipeline(&self.pipelines[*etapa as usize]);
            p.set_bind_group(2, &self.g2, &[]);
            p.set_bind_group(3, grupo, &[]);
            p.dispatch_workgroups(*x, *y, *z);
        }
        true
    }

    /// Resolve os carimbos das `feitas` gravações deste quadro.
    pub(crate) fn resolver_fases(&self, enc: &mut wgpu::CommandEncoder, feitas: u32) {
        if let Some(f) = &self.fases {
            f.feitas.set(feitas.min(f.gravacoes));
            let n = 2 * self.passos.len() as u32 * f.feitas.get();
            enc.resolve_query_set(&f.set, 0..n, &f.resolve, 0);
            enc.copy_buffer_to_buffer(&f.resolve, 0, &f.leitura, 0, u64::from(n) * 8);
        }
    }

    fn ler_fases(&self, gpu: &GpuContext) -> Option<Vec<FaseHrc>> {
        let f = self.fases.as_ref()?;
        let fatia = f.leitura.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |r| {
            r.expect("map das fases da holográfica")
        });
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let ts: Vec<u64> = fatia
            .get_mapped_range()
            .as_chunks::<8>()
            .0
            .iter()
            .map(|b| u64::from_le_bytes(*b))
            .collect();
        f.leitura.unmap();
        let mut out: Vec<FaseHrc> = Vec::new();
        for k in 0..f.feitas.get() as usize {
            for (i, ((etapa, _, _), u)) in self.passos.iter().zip(&self.uniformes).enumerate() {
                let par = 2 * (k * self.passos.len() + i);
                let ms = ts[par + 1].wrapping_sub(ts[par]) as f64 * f.periodo_ns / 1e6;
                let nome = ENTRADAS[*etapa as usize];
                match out.iter_mut().find(|o| o.0 == nome && o.1 == u.nivel) {
                    Some(o) => {
                        o.2 += ms;
                        o.3 += 1;
                    }
                    None => out.push((nome, u.nivel, ms, 1)),
                }
            }
        }
        Some(out)
    }
}

impl Solver {
    /// ⭐ W5q — INSTRUMENTO: daqui em diante cada despacho da holográfica grava no seu passe, com
    /// carimbos próprios (a `c2` dos [`TemposMs`] fica 0; a fronteira de passe soma um pouco).
    /// `false` sem a holográfica ou sem `TIMESTAMP_QUERY`.
    #[doc(hidden)]
    pub fn medir_fases_da_hrc(&mut self, gpu: &GpuContext) -> bool {
        let pode = gpu
            .device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY);
        let gravacoes = self.cfg.rebotes_por_quadro.clamp(1, self.n.max(1));
        match self.hrc.as_mut().filter(|_| pode) {
            Some(h) => {
                h.ligar_fases(gpu, gravacoes);
                true
            }
            None => false,
        }
    }

    /// ⭐ W5s — INSTRUMENTO (o controlo de `medida_luz2d_w5s_salta`): a máscara do vazio deixa de ser
    /// escrita — fica a zero, a traça salta TODO raio e a fluência TEM de mudar.
    #[doc(hidden)]
    pub fn hrc_sem_mascara(&mut self) {
        if let Some(h) = &mut self.hrc {
            h.passos.retain(|p| !matches!(p.0, Etapa::Ladrilhos));
        }
    }

    /// ⭐ W5q — as etapas da holográfica do último quadro submetido, na ordem de gravação.
    #[doc(hidden)]
    #[must_use]
    pub fn fases_da_hrc(&self, gpu: &GpuContext) -> Option<Vec<FaseHrc>> {
        self.hrc.as_ref()?.ler_fases(gpu)
    }
}
