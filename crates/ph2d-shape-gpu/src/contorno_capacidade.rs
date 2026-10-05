//! doc 121 §9.15 (c2) — **a capacidade de uma cena NOVA medida antes do 1.º quadro dela.**
//!
//! A leitura do total é assíncrona (`Contorno::colhe`): a capacidade medida chega dois quadros depois, e
//! até lá cada cópia vai pelo caminho pixel a pixel — numa cena tracejada pela variante COMPLETA. Medido
//! na sonda (iGPU, `72` estrelas esticadas tracejadas): `76` ms por quadro contra `1,27` no regime. ⇒ na
//! 1.ª vez (e quando as geometrias carregadas mudam, `ShapePass::set_geometries`) a contagem corre já,
//! o total lê-se BLOQUEANDO e as capacidades crescem antes do desenho. Uma espera de um quadro de placa
//! no início da cena, contra dois quadros pixel a pixel.
//!
//! ⚠️ Na rota do dispositivo, se o cozimento deste quadro ainda não foi submetido, a contagem lê as
//! cópias do anterior: a capacidade sai curta e a leitura assíncrona corrige-a — a imagem é a mesma.

use super::*;

impl Contorno {
    pub(crate) fn mede_a_capacidade(
        &mut self,
        gpu: &GpuContext,
        grupo0: &wgpu::BindGroup,
        count: u32,
        tracejado: bool,
    ) {
        let (escrita, _) = self.prepara(gpu, count);
        let leitura = buffer(
            gpu,
            "ph2d-shape-gpu capacidade (medida)",
            16,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let grupos = count.div_ceil(64);
        let x = grupos.min(65_535);
        let y = grupos.div_ceil(x.max(1));
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-shape-gpu capacidade (medida)"),
            });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-shape-gpu capacidade (medida)"),
                timestamp_writes: None,
            });
            pass.set_bind_group(0, grupo0, &[]);
            pass.set_bind_group(1, &self.vazio, &[]);
            pass.set_bind_group(2, &escrita, &[]);
            pass.set_pipeline(self.conta.de(tracejado));
            pass.dispatch_workgroups(x, y, 1);
            pass.set_pipeline(&self.soma);
            pass.dispatch_workgroups(1, 1, 1);
        }
        let n = u64::from(count);
        enc.copy_buffer_to_buffer(&self.contagem, n * 4, &leitura, 0, 4);
        enc.copy_buffer_to_buffer(&self.contagem, (2 * n + 1) * 4, &leitura, 4, 4);
        gpu.queue.submit([enc.finish()]);
        leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        if gpu
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .is_err()
        {
            return;
        }
        let dados = leitura.slice(..).get_mapped_range();
        let total: u32 = bytemuck::pod_read_unaligned(&dados[..4]);
        let total_m: u32 = bytemuck::pod_read_unaligned(&dados[4..8]);
        self.total_visto = self.total_visto.max(u64::from(total));
        self.total_visto_m = self.total_visto_m.max(u64::from(total_m));
    }

    /// Garante os buffers para `n` cópias e para o total medido.
    pub(super) fn garante(&mut self, gpu: &GpuContext, n: u64) {
        if n > self.cap_copias {
            let cap = n.next_power_of_two();
            let armazens = wgpu::BufferUsages::STORAGE;
            // Cinco quintos de `n + 1`: as arestas reservadas, as células, as linhas de ecrã, as
            // arestas escritas e o ajuste do tracejado (§9.15).
            self.contagem = buffer(
                gpu,
                "ph2d-shape-gpu contagem",
                QUINTOS * (cap + 1) * 4,
                armazens | wgpu::BufferUsages::COPY_SRC,
            );
            // Três `vec4<u32>` por cópia. `COPY_SRC`: o instrumento `copias_com_contorno` lê-o.
            self.copias = buffer(
                gpu,
                "ph2d-shape-gpu copias do contorno",
                cap * 48,
                armazens | wgpu::BufferUsages::COPY_SRC,
            );
            self.caixas = buffer(gpu, "ph2d-shape-gpu caixas do contorno", cap * 16, armazens);
            self.cap_copias = cap;
        }
        let pedido = self
            .total_visto
            .max(n * ARESTAS_POR_COPIA_INICIAL)
            .min(self.tecto_arestas);
        if pedido > self.cap_arestas {
            let cap = pedido
                .next_power_of_two()
                .min(self.tecto_arestas)
                .next_multiple_of(BLOCO);
            let armazens = wgpu::BufferUsages::STORAGE;
            self.arestas = buffer(gpu, "ph2d-shape-gpu arestas", cap * ARESTA, armazens);
            self.cap_arestas = cap;
            // doc 121 §9.17 (c): a memória das arestas na cena do app (a reserva do tracejado a faz crescer).
            if self.relata {
                let mb = cap * ARESTA / (1024 * 1024);
                let reservadas = self.total_visto;
                eprintln!(
                    "[formas] arestas: capacidade {cap} ({mb} MB) para {reservadas} reservadas por {n} copias"
                );
            }
        }
        let tecto_m = self.tecto_celulas.min(self.celulas_no_maximo);
        let pedido_m = self.total_visto_m.min(tecto_m);
        if pedido_m > self.cap_celulas {
            let cap = ao_oitavo_do_degrau(pedido_m).min(tecto_m);
            let armazens = wgpu::BufferUsages::STORAGE;
            self.celulas_buf = buffer(
                gpu,
                "ph2d-shape-gpu celulas do contorno",
                cap * REGISTO,
                armazens,
            );
            self.acumula = buffer(
                gpu,
                "ph2d-shape-gpu acumulacao das celulas",
                cap * ACUMULA,
                armazens | wgpu::BufferUsages::COPY_SRC,
            );
            self.cap_celulas = cap;
            if self.relata {
                let mb = cap * (REGISTO + ACUMULA) / (1024 * 1024);
                eprintln!(
                    "[formas] celulas: capacidade {cap} ({mb} MB) para {pedido_m} pedidas por {n} copias"
                );
            }
        }
    }

    pub(crate) fn tem_subgrupo(&self) -> bool {
        self.varre_sg.is_some()
    }
}
