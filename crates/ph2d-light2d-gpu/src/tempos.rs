//! Carimbos `TIMESTAMP_QUERY` por passe: C0, C1 (todos os saltos do JFA num passe), os dois da
//! luz direta (C3) e cada cascata.

/// Dois carimbos (início, fim) por passe: C0, C1, C3 visibilidade, C3 fina e as `n` cascatas.
pub(crate) fn n_carimbos(n_cascatas: u32) -> u32 {
    2 * (4 + n_cascatas)
}

pub(crate) const C0: u32 = 0;
pub(crate) const C1: u32 = 2;
pub(crate) const C3_VISIBILIDADE: u32 = 4;
pub(crate) const C3_FINA: u32 = 6;

/// O par de carimbos da cascata `nivel`.
pub(crate) fn da_cascata(nivel: u32) -> u32 {
    8 + 2 * nivel
}

/// Milissegundos de placa do último quadro, por passe.
#[derive(Clone, Debug, PartialEq)]
pub struct TemposMs {
    pub c0: f64,
    pub c1: f64,
    /// Por nível (índice = cascata).
    pub c2: Vec<f64>,
    /// ⭐ C3: a visibilidade das lâmpadas nas sondas da c0 (0 sem a grade fina).
    pub c3_visibilidade: f64,
    /// ⭐ C3: a grade fina (as cascatas interpoladas + a luz direta).
    pub c3_fina: f64,
}

impl TemposMs {
    #[must_use]
    pub fn c2_total(&self) -> f64 {
        self.c2.iter().sum()
    }

    /// O solver de luz propriamente dito (campo + cascatas), sem a captura.
    #[must_use]
    pub fn rc(&self) -> f64 {
        self.c1 + self.c2_total()
    }

    #[must_use]
    pub fn total(&self) -> f64 {
        self.c0 + self.rc() + self.c3_visibilidade + self.c3_fina
    }

    /// ⭐ A parte que segue a célula do NÍVEL (`1/célula²`): tudo menos a grade fina, que tem o
    /// mesmo tamanho em todo nível (o governador prevê a subida por ela).
    #[must_use]
    pub fn por_nivel(&self) -> f64 {
        self.total() - self.c3_fina
    }
}

pub(crate) struct Tempos {
    pub set: wgpu::QuerySet,
    pub resolve: wgpu::Buffer,
    pub leitura: wgpu::Buffer,
    pub n: u32,
    pub periodo_ns: f64,
    /// ⭐ A leitura SEM ESPERA em curso (o governador do editor, W4): `Some` = o `leitura` foi
    /// pedido ao mapa e ainda não foi colhido — enquanto isso o quadro NÃO copia para ele (copiar
    /// para um buffer mapeado é erro de validação). `true` dentro = a placa já o entregou.
    pub pedido: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

impl Tempos {
    pub fn escrita_compute(&self, par: u32) -> wgpu::ComputePassTimestampWrites<'_> {
        wgpu::ComputePassTimestampWrites {
            query_set: &self.set,
            beginning_of_pass_write_index: Some(par),
            end_of_pass_write_index: Some(par + 1),
        }
    }

    pub fn escrita_render(&self, par: u32) -> wgpu::RenderPassTimestampWrites<'_> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.set,
            beginning_of_pass_write_index: Some(par),
            end_of_pass_write_index: Some(par + 1),
        }
    }

    pub fn resolver(&self, enc: &mut wgpu::CommandEncoder) {
        let n = n_carimbos(self.n);
        enc.resolve_query_set(&self.set, 0..n, &self.resolve, 0);
        if self.pedido.is_none() {
            enc.copy_buffer_to_buffer(&self.resolve, 0, &self.leitura, 0, u64::from(n) * 8);
        }
    }

    /// ⭐ **A leitura SEM ESPERA** — chamada depois do submit do quadro: sem pedido em curso, pede
    /// o mapa da cópia que acabou de subir e devolve `None`; com um, colhe-o se a placa já o
    /// entregou (`device.poll` sem bloquear). O governador vê o custo de um quadro de há poucos
    /// quadros — nunca espera pela placa.
    pub fn ler_sem_esperar(&mut self, device: &wgpu::Device) -> Option<TemposMs> {
        let Some(pronto) = &self.pedido else {
            let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let f = flag.clone();
            self.leitura
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |r| {
                    if r.is_ok() {
                        f.store(true, std::sync::atomic::Ordering::Release);
                    }
                });
            self.pedido = Some(flag);
            return None;
        };
        let _ = device.poll(wgpu::PollType::Poll);
        if !pronto.load(std::sync::atomic::Ordering::Acquire) {
            return None;
        }
        self.pedido = None;
        let ts: Vec<u64> = self
            .leitura
            .slice(..)
            .get_mapped_range()
            .as_chunks::<8>()
            .0
            .iter()
            .map(|b| u64::from_le_bytes(*b))
            .collect();
        self.leitura.unmap();
        Some(self.em_ms(&ts))
    }

    fn em_ms(&self, ts: &[u64]) -> TemposMs {
        let ms = |par: u32| {
            let (a, b) = (ts[par as usize], ts[par as usize + 1]);
            b.wrapping_sub(a) as f64 * self.periodo_ns / 1e6
        };
        TemposMs {
            c0: ms(C0),
            c1: ms(C1),
            c2: (0..self.n).map(|i| ms(da_cascata(i))).collect(),
            c3_visibilidade: ms(C3_VISIBILIDADE),
            c3_fina: ms(C3_FINA),
        }
    }

    /// Lê os carimbos do último quadro SUBMETIDO (bloqueia até a placa terminar).
    pub fn ler(&self, device: &wgpu::Device) -> Option<TemposMs> {
        if self.pedido.is_some() {
            // A leitura sem espera é a dona do buffer agora (o editor); a sonda não a mistura.
            return None;
        }
        let fatia = self.leitura.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |r| r.expect("map dos carimbos"));
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let ts: Vec<u64> = fatia
            .get_mapped_range()
            .as_chunks::<8>()
            .0
            .iter()
            .map(|b| u64::from_le_bytes(*b))
            .collect();
        self.leitura.unmap();
        Some(self.em_ms(&ts))
    }
}
