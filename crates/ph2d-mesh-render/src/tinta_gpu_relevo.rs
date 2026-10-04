//! ⭐⭐ **SÓ O RELEVO SOBE** (`docs/3D/30` §15, a W4) — filho (`#[path]`) de
//! [`super`]: a profundidade ou o modo de uma camada mudou, e o relevo da peça
//! (a dobra da pilha) com eles; a cor não — a placa compô-la ela mesma.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

use super::corridas_das_sujas;
use crate::MeshRenderer;

/// O relevo `[altura, corpo]` e as inclinações de um slot, lidos da placa.
pub type RelevoLido = (Vec<[f32; 2]>, Vec<[f32; 3]>);

impl MeshRenderer {
    /// ⭐⭐ **Sobe o relevo `[altura, corpo]` inteiro e refaz as inclinações
    /// das amostras cuja altura mudou desde a foto** — sem tocar nas amostras
    /// de cor (o plano da CPU pode estar atrás da placa).
    ///
    /// `false` = não pode (o device não tem este plano com relevo, ou as
    /// inclinações são de outra adjacência): quem chama sobe o plano inteiro.
    pub fn upload_tinta_relevo_at(
        &mut self,
        queue: &wgpu::Queue,
        index: usize,
        mesh: &Mesh,
        tinta: &Tinta,
    ) -> bool {
        let Some(slot) = self.slots.get_mut(index) else {
            return false;
        };
        let g = &mut slot.gpu.tinta;
        let Some(alt) = tinta.relevo() else {
            return false;
        };
        if !g.armado || !g.relevo || g.n_amostras != tinta.amostras().len() {
            return false;
        }
        let Some(inc) = g.inc.as_mut().filter(|i| i.serve(tinta)) else {
            return false;
        };
        queue.write_buffer(&g.alturas, 0, bytemuck::cast_slice(alt));
        let sujas = g.inc_foto.o_que_mudou(mesh.positions(), alt);
        let mut mudadas = Vec::new();
        let todas = inc.refaz(
            tinta,
            &|f| mesh.faces()[f].verts(),
            mesh.positions(),
            &sujas,
            &mut mudadas,
        );
        let gb: &[u8] = bytemuck::cast_slice(inc.por_amostra());
        if todas {
            queue.write_buffer(&g.inclinacoes, 0, gb);
            return true;
        }
        let mut corridas = Vec::new();
        corridas_das_sujas(&mut mudadas, &mut corridas);
        for &(de, ate) in &corridas {
            queue.write_buffer(&g.inclinacoes, de as u64, &gb[de..ate]);
        }
        true
    }

    /// O relevo `[altura, corpo]` e as inclinações do slot `k` lidos de volta
    /// (BLOQUEIA — gates). `None` se o slot não tem um plano com relevo.
    #[must_use]
    pub fn le_relevo_at(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        k: usize,
    ) -> Option<RelevoLido> {
        let g = &self.slots.get(k)?.gpu.tinta;
        if !g.armado || !g.relevo {
            return None;
        }
        let n = g.n_amostras as u64;
        let alturas = le(device, queue, &g.alturas, n * 8)?;
        let inclinacoes = le(device, queue, &g.inclinacoes, n * 12)?;
        Some((
            bytemuck::cast_slice(&alturas).to_vec(),
            bytemuck::cast_slice(&inclinacoes).to_vec(),
        ))
    }
}

/// Os primeiros `bytes` de `buf` lidos de volta (bloqueia).
fn le(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buf: &wgpu::Buffer,
    bytes: u64,
) -> Option<Vec<u8>> {
    let destino = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-mesh relevo leitura"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ph2d-mesh relevo leitura"),
    });
    enc.copy_buffer_to_buffer(buf, 0, &destino, 0, bytes);
    queue.submit([enc.finish()]);
    let fatia = destino.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
    let lido = fatia.get_mapped_range().to_vec();
    destino.unmap();
    Some(lido)
}
