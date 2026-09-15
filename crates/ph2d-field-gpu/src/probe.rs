//! ⭐ **Correr um WGSL sobre uma lista de `vec4` e trazer a resposta** — o avaliador dos gates de
//! paridade.
//!
//! # Porque ele existe, e porque mora AQUI
//!
//! Uma lei que vai para o dispositivo tem de ser medida contra a irmã de CPU **antes** de tocar num
//! pixel, e cada uma dessas medições precisa da mesma canalização: compilar, ligar buffers,
//! despachar, ler de volta. ⛔ Escrevê-la em cada crate que tem uma lei obrigaria a `wgpu` a entrar
//! em crates que não desenham nada — o céu do estúdio vive na família `field3d`, que é composição.
//!
//! ⚠️ **Não é caminho de produto.** Ele abre um dispositivo por chamada se não lhe derem um.

/// A ordem dos bindings que o WGSL tem de declarar, no grupo `0`:
///
/// | binding | o quê |
/// |---:|---|
/// | `0..u` | um por `uniforms`, na ordem dada |
/// | `u..u+s` | um por `storages`, `read` |
/// | `u+s` | `entrada: array<vec4<f32>>`, `read` |
/// | `u+s+1` | `saida: array<vec4<f32>>`, `read_write` |
///
/// `outs` é quantos `vec4` cada entrada escreve.
// O dispositivo, a fila, a fonte, a entrada do shader, os dois grupos de buffers, as amostras e
// quantas saídas cada uma escreve — oito coisas independentes. Uma struct só as renomearia.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn evaluate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &str,
    entry: &str,
    uniforms: &[&[f32]],
    storages: &[&[f32]],
    inputs: &[[f32; 4]],
    outs: usize,
) -> Vec<[f32; 4]> {
    use wgpu::util::DeviceExt;
    let bytes = |v: &[f32]| -> Vec<u8> { v.iter().flat_map(|f| f.to_le_bytes()).collect() };
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("sonda"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("sonda"),
        layout: None,
        module: &modulo,
        entry_point: Some(entry),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let mut buffers: Vec<wgpu::Buffer> = Vec::new();
    for u in uniforms {
        buffers.push(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &bytes(u),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );
    }
    for s in storages {
        buffers.push(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &bytes(s),
                usage: wgpu::BufferUsages::STORAGE,
            }),
        );
    }
    let planos: Vec<f32> = inputs.iter().flat_map(|a| *a).collect();
    buffers.push(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("entrada"),
            contents: &bytes(&planos),
            usage: wgpu::BufferUsages::STORAGE,
        }),
    );
    let n = (inputs.len() * outs * 16) as u64;
    let saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("saida"),
        size: n.max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: n.max(16),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut entradas: Vec<wgpu::BindGroupEntry<'_>> = buffers
        .iter()
        .enumerate()
        .map(|(i, b)| wgpu::BindGroupEntry {
            #[allow(clippy::cast_possible_truncation)]
            binding: i as u32,
            resource: b.as_entire_binding(),
        })
        .collect();
    entradas.push(wgpu::BindGroupEntry {
        #[allow(clippy::cast_possible_truncation)]
        binding: buffers.len() as u32,
        resource: saida.as_entire_binding(),
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entradas,
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(&pipeline);
        cp.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        cp.dispatch_workgroups((inputs.len() as u32).div_ceil(64), 1, 1);
    }
    enc.copy_buffer_to_buffer(&saida, 0, &leitura, 0, n.max(16));
    queue.submit([enc.finish()]);
    leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let dados = leitura.slice(..).get_mapped_range();
    let f4 = |q: &[u8; 16], o: usize| f32::from_le_bytes([q[o], q[o + 1], q[o + 2], q[o + 3]]);
    let out = dados
        .as_chunks::<16>()
        .0
        .iter()
        .take(inputs.len() * outs)
        .map(|q| [0, 1, 2, 3].map(|c| f4(q, c * 4)))
        .collect();
    drop(dados);
    out
}
