//! Carimbos `TIMESTAMP_QUERY` por passe: C0, C1 (todos os saltos do JFA num passe), os dois da
//! luz direta (C3), a captura fina e o preenchimento (W5a), as máscaras dos distantes (W5b) e cada
//! cascata.

/// Dois carimbos (início, fim) por passe: C0, C1, C3 visibilidade, C3 fina, C0 fina, C3 preenche,
/// C3 máscaras, os dois do rebote (W5c), o miolo (W5h) e as `n` cascatas.
pub(crate) fn n_carimbos(n_cascatas: u32) -> u32 {
    2 * (10 + n_cascatas)
}

pub(crate) const C0: u32 = 0;
pub(crate) const C1: u32 = 2;
pub(crate) const C3_VISIBILIDADE: u32 = 4;
pub(crate) const C3_FINA: u32 = 6;
pub(crate) const C0_FINA: u32 = 8;
pub(crate) const C3_PREENCHE: u32 = 10;
pub(crate) const C3_MASCARAS: u32 = 12;
pub(crate) const C0_REBOTE: u32 = 14;
pub(crate) const C3_REBOTE: u32 = 16;
pub(crate) const C3_MIOLO: u32 = 18;

/// O par de carimbos da cascata `nivel`.
pub(crate) fn da_cascata(nivel: u32) -> u32 {
    20 + 2 * nivel
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
    /// ⭐ W5a: a captura fina dos opacos (0 sem ela).
    pub c0_fina: f64,
    /// ⭐ W5a (F3): o JFA do complemento (0 sem o preenchimento).
    pub c3_preenche: f64,
    /// ⭐ W5b (F4): as máscaras dos sprites distantes (0 sem nenhum).
    pub c3_mascaras: f64,
    /// ⭐ W5c: o rebote — a soma na emissão e a fonte para o quadro seguinte (0 sem ele).
    pub c0_rebote: f64,
    pub c3_rebote: f64,
    /// ⭐ W5h: o miolo dos opacos pela difusão (0 sem ele).
    pub c3_miolo: f64,
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
        self.c0
            + self.rc()
            + self.c3_visibilidade
            + self.c3_fina
            + self.c0_fina
            + self.c3_preenche
            + self.c3_mascaras
            + self.c0_rebote
            + self.c3_rebote
            + self.c3_miolo
    }

    /// ⭐ A parte do tamanho da grade FINA, o mesmo em todo nível: ela, a captura fina e o
    /// preenchimento e o miolo (o governador prevê a subida sem a multiplicar) — e as máscaras dos
    /// distantes, que seguem o objeto e não o nível.
    #[must_use]
    pub fn fixo(&self) -> f64 {
        self.c3_fina + self.c0_fina + self.c3_preenche + self.c3_mascaras + self.c3_miolo
    }

    /// ⭐ A parte que segue a célula do NÍVEL (`1/célula²`): tudo menos o [`Self::fixo`].
    #[must_use]
    pub fn por_nivel(&self) -> f64 {
        self.total() - self.fixo()
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
            c0_fina: ms(C0_FINA),
            c3_preenche: ms(C3_PREENCHE),
            c3_mascaras: ms(C3_MASCARAS),
            c0_rebote: ms(C0_REBOTE),
            c3_rebote: ms(C3_REBOTE),
            c3_miolo: ms(C3_MIOLO),
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
