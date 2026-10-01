//! ⏱️⭐⭐⭐⭐ **O RELÓGIO POR PASSE NA PLACA** — quanto cada despacho de um quadro custou, medido
//! pela própria placa (`TIMESTAMP_QUERY`) e não inferido de ablações.
//!
//! Existe porque a escada de ablação da oclusão no tempo (2026-09-30) deixou a pergunta que decide
//! a wave seguinte sem resposta directa: *o que come o quadro de movimento do nó?* — tirar uma parte
//! de cada vez mede a parte E o que ela arrasta (a carga da máquina, a ocupação dos warps, o
//! compilador), e com a máquina a `load 40` duas corridas iguais leram `±20 %`.
//!
//! ⚠️ **Só as SONDAS o ligam** (`PH2D_GPU_CRONOMETRO=1`), e só numa placa que ofereça a feature: o
//! pedido da feature e os `timestamp_writes` ficam fora do caminho do produto, que continua a pedir
//! `Features::empty()` e a despachar sem marcas.
//!
//! O ciclo: cada passe pede um par de marcas com um RÓTULO ([`Cronometro::marca`]); cada encoder
//! resolve o troço que escreveu antes de ser submetido ([`Cronometro::resolve`]); e quando a fila
//! está vazia ([`Cronometro::colhe`]) as durações somam-se por rótulo.

use std::collections::BTreeMap;

/// Quantos PARES de marcas cabem num quadro — um por passe; o quadro mais longo tem `~16`.
const PARES: u32 = 64;

/// ⭐ **O relógio está ligado nesta sessão?** — lido uma vez.
#[must_use]
pub fn ligado() -> bool {
    static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADO.get_or_init(|| std::env::var("PH2D_GPU_CRONOMETRO").is_ok_and(|v| v.trim() != "0"))
}

/// ⭐ **A feature a pedir à placa** — vazia fora das sondas ou numa placa que não a tem.
#[must_use]
pub fn feature(adapter: &wgpu::Adapter) -> wgpu::Features {
    if ligado() && adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        wgpu::Features::TIMESTAMP_QUERY
    } else {
        wgpu::Features::empty()
    }
}

/// ⏱️ O relógio de UM dispositivo.
pub struct Cronometro {
    marcas: wgpu::QuerySet,
    resolvidas: wgpu::Buffer,
    leitura: wgpu::Buffer,
    rotulos: Vec<&'static str>,
    /// Até onde o último `resolve` chegou, em marcas.
    feitas: u32,
    periodo_ns: f32,
    /// Por rótulo: milissegundos somados e quantas vezes.
    somas: BTreeMap<&'static str, (f64, u32)>,
    /// ⏱️ Os CONTADORES que um passe deixou num buffer (contagens, não tempos) — ver
    /// [`Cronometro::contadores`].
    contas: wgpu::Buffer,
    contas_rotulos: Vec<&'static str>,
}

/// Quantos contadores cabem num quadro.
const CONTAS: u64 = 16;

impl Cronometro {
    /// `None` fora das sondas ou sem a feature no dispositivo.
    #[must_use]
    pub fn novo(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let n = u64::from(PARES * 2);
        Some(Self {
            marcas: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("cronometro"),
                ty: wgpu::QueryType::Timestamp,
                count: PARES * 2,
            }),
            resolvidas: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("cronometro-resolvidas"),
                size: n * 8,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            leitura: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("cronometro-leitura"),
                size: n * 8,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            rotulos: Vec::new(),
            feitas: 0,
            periodo_ns: queue.get_timestamp_period(),
            somas: BTreeMap::new(),
            contas: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("cronometro-contas"),
                size: CONTAS * 4,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            contas_rotulos: Vec::new(),
        })
    }

    /// ⭐ **As marcas de um passe** — `None` quando o quadro já gastou os [`PARES`] (o passe corre
    /// sem ser medido, nunca deixa de correr).
    pub fn marca(&mut self, rotulo: &'static str) -> Option<wgpu::ComputePassTimestampWrites<'_>> {
        let k = u32::try_from(self.rotulos.len()).ok()?;
        if k >= PARES {
            return None;
        }
        self.rotulos.push(rotulo);
        Some(wgpu::ComputePassTimestampWrites {
            query_set: &self.marcas,
            beginning_of_pass_write_index: Some(2 * k),
            end_of_pass_write_index: Some(2 * k + 1),
        })
    }

    /// Resolve as marcas que este encoder escreveu — antes do `finish`.
    pub fn resolve(&mut self, enc: &mut wgpu::CommandEncoder) {
        #[allow(clippy::cast_possible_truncation)]
        let ate = (self.rotulos.len() as u32) * 2;
        if ate <= self.feitas {
            return;
        }
        enc.resolve_query_set(
            &self.marcas,
            self.feitas..ate,
            &self.resolvidas,
            u64::from(self.feitas) * 8,
        );
        enc.copy_buffer_to_buffer(
            &self.resolvidas,
            u64::from(self.feitas) * 8,
            &self.leitura,
            u64::from(self.feitas) * 8,
            u64::from(ate - self.feitas) * 8,
        );
        self.feitas = ate;
    }

    /// ⏱️ **Copia contadores `u32`** de `origem` (a partir de `offset`, em bytes) — um por rótulo; o
    /// relatório dá a MÉDIA por quadro de cada um.
    pub fn contadores(
        &mut self,
        enc: &mut wgpu::CommandEncoder,
        origem: &wgpu::Buffer,
        offset: u64,
        rotulos: &[&'static str],
    ) {
        let k = self.contas_rotulos.len() as u64;
        let n = (rotulos.len() as u64).min(CONTAS - k);
        if n == 0 {
            return;
        }
        enc.copy_buffer_to_buffer(origem, offset, &self.contas, k * 4, n * 4);
        self.contas_rotulos
            .extend_from_slice(&rotulos[..n as usize]);
    }

    /// ⭐ **Soma o quadro** — chamado com a fila vazia (depois do `poll` que espera o quadro).
    pub fn colhe(&mut self, device: &wgpu::Device) {
        if !self.contas_rotulos.is_empty() {
            let bytes = self.contas_rotulos.len() as u64 * 4;
            self.contas
                .slice(..bytes)
                .map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::PollType::wait_indefinitely()).ok();
            {
                let dados = self.contas.slice(..bytes).get_mapped_range();
                for (rotulo, b) in self.contas_rotulos.iter().zip(dados.as_chunks::<4>().0) {
                    let e = self.somas.entry(rotulo).or_insert((0.0, 0));
                    e.0 += f64::from(u32::from_le_bytes(*b));
                    e.1 += 1;
                }
            }
            self.contas.unmap();
            self.contas_rotulos.clear();
        }
        if self.feitas == 0 {
            self.rotulos.clear();
            return;
        }
        let bytes = u64::from(self.feitas) * 8;
        self.leitura
            .slice(..bytes)
            .map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
        {
            let dados = self.leitura.slice(..bytes).get_mapped_range();
            let t: Vec<u64> = dados
                .as_chunks::<8>()
                .0
                .iter()
                .map(|b| u64::from_le_bytes(*b))
                .collect();
            for (k, rotulo) in self.rotulos.iter().enumerate() {
                if 2 * k + 1 >= t.len() {
                    break;
                }
                let ns = t[2 * k + 1].saturating_sub(t[2 * k]) as f64 * f64::from(self.periodo_ns);
                let e = self.somas.entry(rotulo).or_insert((0.0, 0));
                e.0 += ns * 1e-6;
                e.1 += 1;
            }
        }
        self.leitura.unmap();
        self.rotulos.clear();
        self.feitas = 0;
    }

    /// ⏱️ **Um troço de CPU** — soma o tempo de parede desde `desde` sob o rótulo e devolve o agora,
    /// para o troço seguinte começar onde este acabou.
    pub fn cpu(&mut self, rotulo: &'static str, desde: std::time::Instant) -> std::time::Instant {
        let agora = std::time::Instant::now();
        let e = self.somas.entry(rotulo).or_insert((0.0, 0));
        e.0 += (agora - desde).as_secs_f64() * 1e3;
        e.1 += 1;
        agora
    }

    /// ⏱️ **O relatório**: por rótulo, a média em ms e quantas vezes correu — e esvazia as somas.
    pub fn relatorio(&mut self) -> Vec<(&'static str, f64, u32)> {
        let r = self
            .somas
            .iter()
            .map(|(k, (ms, n))| (*k, ms / f64::from((*n).max(1)), *n))
            .collect();
        self.somas.clear();
        r
    }
}
