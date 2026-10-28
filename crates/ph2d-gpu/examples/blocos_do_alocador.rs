//! **O preço dos blocos pequenos do alocador quando se criam e largam texturas GRANDES** (F38,
//! `docs/UI_New_and_Simple/F38_plano.md` §2). Dois dispositivos no MESMO adaptador e no MESMO
//! processo — um com `MemoryHints::MemoryUsage` (8–64 MiB, o de omissão desde a F37), outro com
//! `Performance` (128–256 MiB) — e, intercalados em blocos rodados, cada um cria, limpa na placa e
//! larga texturas do tamanho de uma tela de pintura. Imprime o MÍNIMO e a mediana por variante.
//!
//!     PH2D_GPU=1 bash scripts/ph2d-run.sh ./target/smoke/examples/blocos_do_alocador 12
//!
//! A placa é a que o carregador do Vulkan mostrar (`VK_ICD_FILENAMES=…radeon_icd.json` = a onboard).

use std::time::Instant;

#[derive(Clone, Copy)]
enum Caso {
    /// Texturas de `lado²` (uma tela que se refaz inteira).
    Textura(u32, wgpu::TextureFormat),
    /// Buffers de `lado² × 4 B` (os `base`/`out`/`read` do lote de dabs, `ph2d-paint-gpu` `StampPass::run`,
    /// que nascem e morrem a cada lote, do tamanho da caixa do lote).
    Buffer(u32),
}

/// A tela 4K inteira em 8 e 16 bits, um retângulo sujo típico, e lotes de dabs de pincel médio e grande.
const CASOS: [(&str, Caso); 5] = [
    (
        "textura 4096² rgba8",
        Caso::Textura(4096, wgpu::TextureFormat::Rgba8Unorm),
    ),
    (
        "textura 4096² rgba16f",
        Caso::Textura(4096, wgpu::TextureFormat::Rgba16Float),
    ),
    (
        "textura 1024² rgba8",
        Caso::Textura(1024, wgpu::TextureFormat::Rgba8Unorm),
    ),
    ("lote 512² (buffers)", Caso::Buffer(512)),
    ("lote 2048² (buffers)", Caso::Buffer(2048)),
];
/// Recursos criados e largados por medida (um quadro que refaz a pilha, ou os três buffers de um lote).
const POR_MEDIDA: usize = 3;

fn dispositivo(adapter: &wgpu::Adapter, hints: wgpu::MemoryHints) -> (wgpu::Device, wgpu::Queue) {
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("blocos_do_alocador"),
        memory_hints: hints,
        ..Default::default()
    }))
    .expect("dispositivo")
}

/// Cria, limpa na placa e larga `POR_MEDIDA` recursos; devolve os ms de relógio até a placa acabar.
fn medida(device: &wgpu::Device, queue: &wgpu::Queue, caso: Caso) -> f64 {
    match caso {
        Caso::Textura(lado, formato) => medida_textura(device, queue, lado, formato),
        Caso::Buffer(lado) => medida_buffer(device, queue, lado),
    }
}

fn medida_buffer(device: &wgpu::Device, queue: &wgpu::Queue, lado: u32) -> f64 {
    let t = Instant::now();
    let bufs: Vec<_> = (0..POR_MEDIDA)
        .map(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("blocos_do_alocador lote"),
                size: u64::from(lado) * u64::from(lado) * 4,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        })
        .collect();
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    for b in &bufs {
        enc.clear_buffer(b, 0, None);
    }
    queue.submit([enc.finish()]);
    drop(bufs);
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    t.elapsed().as_secs_f64() * 1e3
}

fn medida_textura(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    lado: u32,
    formato: wgpu::TextureFormat,
) -> f64 {
    let t = Instant::now();
    let texs: Vec<_> = (0..POR_MEDIDA)
        .map(|_| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("blocos_do_alocador tela"),
                size: wgpu::Extent3d {
                    width: lado,
                    height: lado,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: formato,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        })
        .collect();
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    for tex in &texs {
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    queue.submit([enc.finish()]);
    drop(texs);
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    t.elapsed().as_secs_f64() * 1e3
}

fn main() {
    let rondas: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(12);
    let inst = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle_from_env()
    });
    ph2d_gpu::vez_da_placa();
    let adapter = pollster::block_on(inst.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("placa");
    println!("[blocos] placa: {}", adapter.get_info().name);
    let variantes = [
        ("uso", dispositivo(&adapter, wgpu::MemoryHints::MemoryUsage)),
        (
            "desempenho",
            dispositivo(&adapter, wgpu::MemoryHints::Performance),
        ),
    ];
    // Aquecimento: os pipelines internos de limpeza e o primeiro bloco de cada dispositivo.
    for (_, (d, q)) in &variantes {
        for &(_, caso) in &CASOS {
            medida(d, q, caso);
        }
    }
    let mut ms = vec![vec![Vec::new(); variantes.len()]; CASOS.len()];
    for r in 0..rondas {
        for k in 0..variantes.len() {
            let v = (k + r) % variantes.len();
            let (d, q) = &variantes[v].1;
            for (c, &(_, caso)) in CASOS.iter().enumerate() {
                ms[c][v].push(medida(d, q, caso));
            }
        }
    }
    for (c, &(nome, ..)) in CASOS.iter().enumerate() {
        for (v, (var, _)) in variantes.iter().enumerate() {
            let s = &mut ms[c][v];
            s.sort_by(f64::total_cmp);
            println!(
                "[blocos] {nome} ×{POR_MEDIDA} {var}: min {:.2} ms · mediana {:.2} ms ({} medidas)",
                s[0],
                s[s.len() / 2],
                s.len()
            );
        }
    }
}
