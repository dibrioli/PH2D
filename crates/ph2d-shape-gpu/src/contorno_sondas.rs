//! Os INSTRUMENTOS do contorno (doc 121 §9.6–§9.13): o que gates e sondas lêem de volta da placa,
//! bloqueando — nenhum corre no quadro do produto.

use super::*;

impl Contorno {
    /// **Quantas das `n` cópias ganharam contorno** no último cálculo — lido de volta, bloqueando.
    /// Instrumento: um gate e uma sonda que perguntam se o caminho novo CORREU (as duas imagens são
    /// iguais, logo nenhuma régua de pixel o distingue do caminho de sempre).
    pub(crate) fn copias_com_contorno(&self, gpu: &GpuContext, n: u32) -> (u32, u64) {
        let bytes = (u64::from(n) * 48).max(16);
        let leitura = buffer(
            gpu,
            "ph2d-shape-gpu contorno (sonda)",
            bytes,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_buffer_to_buffer(&self.copias, 0, &leitura, 0, u64::from(n) * 48);
        gpu.queue.submit([enc.finish()]);
        leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let dados = leitura.slice(..).get_mapped_range();
        let copias: &[[u32; 4]] = bytemuck::cast_slice(&dados[..(u64::from(n) * 48) as usize]);
        // O 1.º de cada trio: blocos do preenchimento, das marcas e do contorno.
        let com = copias
            .iter()
            .step_by(3)
            .filter(|c| c[1] + c[2] + c[3] > 0)
            .count();
        (u32::try_from(com).unwrap_or(u32::MAX), self.cap_arestas)
    }

    /// Uma palavra da `contagem` do último cálculo, lida de volta (bloqueia).
    fn palavra_da_contagem(&self, gpu: &GpuContext, indice: u64) -> u32 {
        let leitura = buffer(
            gpu,
            "ph2d-shape-gpu contagem (sonda)",
            16,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_buffer_to_buffer(&self.contagem, indice * 4, &leitura, 0, 4);
        gpu.queue.submit([enc.finish()]);
        leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let dados = leitura.slice(..).get_mapped_range();
        bytemuck::pod_read_unaligned(&dados[..4])
    }

    /// **Quantas células o último cálculo PEDIU**, e a capacidade delas — lido de volta, bloqueando
    /// (doc 121 §9.12). Pedido acima da capacidade ⇒ alguma cópia não coube e foi desenhada pelo
    /// caminho de sempre. Instrumento de gates e sondas.
    pub(crate) fn celulas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64) {
        let pedido = self.palavra_da_contagem(gpu, 2 * u64::from(self.ultimo_n) + 1);
        (u64::from(pedido), self.cap_celulas)
    }

    /// **Quantas cópias o último cálculo mandou ao passe de GRUPO** (doc 121 §9.14) — o total do 4.º
    /// quarto da contagem, lido de volta (bloqueia). Instrumento de gates e sondas: as duas escritas
    /// desenham a mesma imagem, logo só ele diz qual correu.
    pub(crate) fn copias_do_grupo_do_ultimo_quadro(&self, gpu: &GpuContext) -> u32 {
        self.palavra_da_contagem(gpu, 4 * u64::from(self.ultimo_n) + 3)
    }

    /// As cópias do último desenho de uma cena com tracejado, por variante `(enxuta, completa)` —
    /// lido de volta da placa (bloqueia).
    pub(crate) fn copias_por_variante(&self, gpu: &GpuContext) -> (u32, u32) {
        let leitura = buffer(
            gpu,
            "ph2d-shape-gpu desenhos (sonda)",
            DESPACHO - DESENHO_ENXUTA,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_buffer_to_buffer(
            &self.despacho,
            DESENHO_ENXUTA,
            &leitura,
            0,
            DESPACHO - DESENHO_ENXUTA,
        );
        gpu.queue.submit([enc.finish()]);
        leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let dados = leitura.slice(..).get_mapped_range();
        let w: &[u32] = bytemuck::cast_slice(&dados);
        (w[1], w[5])
    }

    /// doc 121 §9.13 — **quantas das células em uso o último quadro TOCOU** (a alavanca da variante
    /// esparsa): uma célula tocada tem um depósito das marcas ou do contorno por apagar, ou a
    /// cobertura que o `cs_varre` gravou não é a mesma nos `PIXELS_DA_CELULA` pixels. ⚠️ Cota por
    /// BAIXO: depósitos do preenchimento que se anulam numa célula não se vêem. Lido de volta (bloqueia).
    pub(crate) fn celulas_tocadas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64) {
        let (pedido, cap) = self.celulas_do_ultimo_quadro(gpu);
        let usadas = pedido.min(cap);
        if usadas == 0 {
            return (0, 0);
        }
        let leitura = buffer(
            gpu,
            "ph2d-shape-gpu celulas tocadas (sonda)",
            usadas * ACUMULA,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_buffer_to_buffer(&self.acumula, 0, &leitura, 0, usadas * ACUMULA);
        gpu.queue.submit([enc.finish()]);
        leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let dados = leitura.slice(..).get_mapped_range();
        let palavras: &[u32] = bytemuck::cast_slice(&dados);
        let px = usize::try_from(PIXELS_DA_CELULA).unwrap_or(32);
        let tocadas = palavras
            .chunks_exact(3 * px)
            .filter(|c| c[px..].iter().any(|&w| w != 0) || c[..px].iter().any(|&w| w != c[0]))
            .count();
        (tocadas as u64, usadas)
    }
}
