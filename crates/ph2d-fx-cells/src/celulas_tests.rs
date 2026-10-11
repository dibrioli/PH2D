use super::*;
use crate::passes::{buffer, compute, reloca, vazio};

/// ⭐ **A relocação na placa É a soma da CPU**: a instância `idx` copiada palavra a palavra, só
/// a posição somada do `delta` — lida de volta e comparada campo a campo.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_device_relocation_is_the_cpu_sum() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    let r = RenderInstance {
        world_pos: [0.37, -0.45],
        size: [1.1, 0.6],
        atlas_uv: [0.1, 0.2, 0.3, 0.4],
        tint: [0.3, 0.6, 0.9, 1.0],
        basis: [0.8, 0.5, -0.5, 0.8],
        premultiplied: 0.0,
        anchor: [0.1, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 0.5,
        flip_uv: 7,
        texture_id: 3,
        z_order: 9,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 4,
    };
    let mut outra = r;
    outra.world_pos = [5.0, 6.0];
    let pipeline = compute(&gpu, RELOCAR_WGSL, "fx_cells relocar (gate)");
    let origem = buffer(
        &gpu,
        "origem",
        bytemuck::cast_slice(&[outra, r]),
        wgpu::BufferUsages::STORAGE,
    );
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    let delta = [-8.7f32, 6.25];
    let tam = std::mem::size_of::<RenderInstance>() as u64;
    let d = reloca(
        &pipeline,
        &gpu,
        &mut enc,
        &origem,
        &[1],
        &[delta],
        (PALAVRAS_SPRITE, POS_SPRITE),
        wgpu::BufferUsages::COPY_SRC,
    );
    let l = vazio(
        &gpu,
        "leitura",
        tam,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    );
    enc.copy_buffer_to_buffer(&d, 0, &l, 0, tam);
    gpu.queue.submit([enc.finish()]);
    l.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    #[expect(clippy::cast_possible_truncation, reason = "188 B")]
    let lida: RenderInstance =
        *bytemuck::from_bytes(&l.slice(..).get_mapped_range()[..tam as usize]);
    let mut esperada = r;
    esperada.world_pos = [r.world_pos[0] + delta[0], r.world_pos[1] + delta[1]];
    assert_eq!(bytemuck::bytes_of(&lida), bytemuck::bytes_of(&esperada));
}
