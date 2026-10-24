//! ⭐ bug 7 (W6i) — como o `pack2x16float` desta placa arredonda f32 → f16 (o WGSL deixa-o à placa: a RADV
//! trunca, a NVIDIA vai ao mais próximo — `docs/Luz2D/BUGS_luz2d.md` bug 7). A holográfica só paga a correção
//! (`Config::hrc_meio_float_mais_proximo`, +6–13 % do `c2` na RTX) onde a placa trunca; onde já arredonda, a
//! correção é o mesmo bit e sai de graça.

use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use ph2d_gpu::GpuContext;

/// Os expoentes das amostras: `x = 2^k · (1 + ¾ · 2⁻¹⁰)`, ¾ de ULP acima de um f16 — o mais próximo sobe, o
/// para-zero desce.
const EXPOENTES: [i32; 4] = [-4, 0, 4, 8];

/// A `Config` efetiva do Solver: o pedido da correção só vale onde a placa trunca; onde já arredonda, a
/// correção é o mesmo bit e não se paga.
pub(crate) fn paga_o_mais_proximo(pedido: &crate::Config, gpu: &GpuContext) -> bool {
    pedido.hrc_meio_float_mais_proximo && o_pack_da_placa_trunca(gpu)
}

/// O `pack2x16float` desta placa trunca (para zero)? Medido uma vez por adaptador no processo. Se a
/// medição falha (placa perdida, leitura recusada), responde `true`: a correção é o mesmo bit onde a placa
/// arredonda, só custa — e a montagem segue para recusar ela própria, com o aviso, em vez de um pânico.
#[must_use]
pub fn o_pack_da_placa_trunca(gpu: &GpuContext) -> bool {
    static MEDIDO: Mutex<BTreeMap<String, bool>> = Mutex::new(BTreeMap::new());
    let i = gpu.adapter.get_info();
    let chave = format!(
        "{} · {} · {} · {:?}",
        i.name, i.driver, i.driver_info, i.backend
    );
    let mut m = MEDIDO.lock().unwrap_or_else(PoisonError::into_inner);
    *m.entry(chave).or_insert_with(|| {
        // O âmbito próprio: um erro da medição não se lê como recusa da montagem que a chamou.
        let validacao = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let medido = medir(gpu);
        let invalido = pollster::block_on(validacao.pop()).is_some();
        medido.filter(|_| !invalido).unwrap_or(true)
    })
}

fn medir(gpu: &GpuContext) -> Option<bool> {
    use wgpu::util::DeviceExt;
    let d = &gpu.device;
    let x: Vec<f32> = EXPOENTES
        .iter()
        .map(|&k| 2f32.powi(k) * (1.0 + 0.75 * 2f32.powi(-10)))
        .collect();
    let m = d.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("luz2d/o arredondamento da placa"),
        source: wgpu::ShaderSource::Wgsl(
            "@group(0) @binding(0) var<storage, read> x: array<f32>;
             @group(0) @binding(1) var<storage, read_write> p: array<u32>;
             @compute @workgroup_size(4) fn main(@builtin(local_invocation_index) i: u32) {
                 p[i] = pack2x16float(vec2<f32>(x[i], 0.0)) & 0xffffu;
             }"
            .into(),
        ),
    });
    let pipe = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("luz2d/o arredondamento da placa"),
        layout: None,
        module: &m,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bx = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("luz2d/o arredondamento da placa: x"),
        contents: bytemuck::cast_slice(&x),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let bytes = 4 * EXPOENTES.len() as u64;
    let bp = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("luz2d/o arredondamento da placa: p"),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = d.create_buffer(&wgpu::BufferDescriptor {
        label: Some("luz2d/o arredondamento da placa: leitura"),
        size: bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipe.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: bx.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: bp.as_entire_binding(),
            },
        ],
    });
    let mut enc = d.create_command_encoder(&Default::default());
    {
        let mut c = enc.begin_compute_pass(&Default::default());
        c.set_pipeline(&pipe);
        c.set_bind_group(0, &bg, &[]);
        c.dispatch_workgroups(1, 1, 1);
    }
    enc.copy_buffer_to_buffer(&bp, 0, &leitura, 0, bytes);
    gpu.queue.submit([enc.finish()]);
    let f = leitura.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    f.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r.is_ok());
    });
    d.poll(wgpu::PollType::wait_indefinitely()).ok()?;
    if rx.try_recv() != Ok(true) {
        return None;
    }
    let p: Vec<u32> = bytemuck::cast_slice(&f.get_mapped_range()).to_vec();
    Some(
        EXPOENTES
            .iter()
            .zip(&p)
            .any(|(&k, &b)| b == ((15 + k) as u32) << 10),
    )
}
