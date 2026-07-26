//! **O custo ÚNICO que só o PRIMEIRO traço paga** (doc 28 §4.8).
//!
//! O `PainterGpuPreview` da shell é criado **lazily** (`get_or_insert_with`), ou seja no primeiro frame
//! que precisa do preview GPU — que é o primeiro traço do artista. Ele constrói três coisas, e cada uma
//! **COMPILA shaders**: o `LayerCompositor`, o `ImpastoLightPass` e o `PreviewPremul`.
//!
//! Compilação de pipeline é o clássico *hitch de primeiro frame*, e é invisível para toda sonda do
//! `ph2d-tool-painter` — elas medem o tool, e o tool não tem GPU. Esta mede o que falta.
//!
//! `#[ignore]`: precisa de um adapter real.

use ph2d_gpu::GpuContext;
use ph2d_render::{
    ImpastoLamp, ImpastoLightInput, ImpastoLightPass, LayerCompositor, PreviewPremul, Region,
};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn ms(f: &mut dyn FnMut()) -> f64 {
    let t = std::time::Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1000.0
}

#[test]
#[ignore = "perf measurement — needs a real adapter; run with --release --ignored"]
fn what_the_first_stroke_pays_to_build_the_gpu_preview() {
    let Some(gpu) = try_headless_gpu() else {
        println!("[first-gpu] sem adapter — pulado");
        return;
    };
    // ⚠️ Uma criação a MAIS antes de medir, para separar o custo da PEÇA do custo de acordar o driver
    // (a primeira chamada a qualquer coisa de wgpu paga inicialização que não é desta peça). Sem esse
    // controle a 1ª linha da tabela leva a culpa das outras duas.
    let warm = ms(&mut || {
        let _ = LayerCompositor::new(&gpu);
    });
    let comp = ms(&mut || {
        let _ = LayerCompositor::new(&gpu);
    });
    let light = ms(&mut || {
        let _ = ImpastoLightPass::new(&gpu);
    });
    let premul = ms(&mut || {
        let _ = PreviewPremul::new(&gpu);
    });
    let total = comp + light + premul;
    println!("[first-gpu] (aquecimento do driver, descartado)  {warm:>8.2} ms");
    println!("[first-gpu] LayerCompositor::new                 {comp:>8.2} ms");
    println!("[first-gpu] ImpastoLightPass::new                {light:>8.2} ms");
    println!("[first-gpu] PreviewPremul::new                   {premul:>8.2} ms");
    println!("[first-gpu] TOTAL pago no 1o traco               {total:>8.2} ms");
}

/// **E o custo que ESCALA COM A TELA** — o que o smoke do Enio isolou (2026-07-26): *"quanto menor o
/// IMG menor o atraso; 1024 nem se percebe"*.
///
/// ⚠️ **Isso REFUTA a compilação de pipeline como causa principal**: um pipeline é compilado uma vez e
/// **independe do tamanho do canvas** — os 28 ms seriam os mesmos a 1024 e a 4096. O que escala com a
/// tela **e** é pago uma vez são os **RECURSOS**: as texturas do passe de luz nascem do tamanho do
/// canvas e a primeira execução as ALOCA e as SEMEIA (upload dos três planos por PCIe).
///
/// E o gatilho é o mesmo do relatório: uma pilha recém-bindada é **trivial**, então o caminho GPU é
/// recusado e nada é alocado; o **primeiro traço com relevo** a torna não-trivial, e é ali que tudo
/// nasce. Esta sonda mede a 1ª execução contra a 2ª, por tamanho: a diferença é o que o primeiro traço
/// paga a mais.
#[test]
#[ignore = "perf measurement — needs a real adapter; run with --release --ignored"]
fn what_the_first_lit_stroke_pays_per_canvas_size() {
    let Some(gpu) = try_headless_gpu() else {
        println!("[first-gpu] sem adapter — pulado");
        return;
    };
    println!("[first-gpu] tela | 1a execucao | 2a | DIFERENCA (o que so o 1o traco paga)");
    for side in [1024u32, 2048, 4096] {
        let n = (side as usize) * (side as usize);
        let relief = vec![0.5f32; n];
        let cover = vec![200u8; n];
        let mat0 = vec![0u8; n * 4];
        let mat1 = vec![0u8; n * 4];
        let spec_lut = vec![0.5f32; 256 * 65];
        let src = make_rgba(&gpu, side, side);
        let lamps = [ImpastoLamp {
            dir: [0.4, 0.4, 0.8],
            half: [0.2, 0.2, 0.95],
            tint: [1.0, 1.0, 1.0],
        }];
        let mut pass = ImpastoLightPass::new(&gpu);
        let input = ImpastoLightInput {
            width: side,
            height: side,
            region: Region::full(side, side),
            plane_region: Region::full(side, side),
            relief: &relief,
            cover: &cover,
            mat0: &mat0,
            mat1: &mat1,
            lamps: &lamps,
            spec_lut: &spec_lut,
            lut_width: 256,
            rough_levels: 65,
        };
        let first = ms(&mut || {
            let _ = pass.run(&gpu, &src, &input);
        });
        let second = ms(&mut || {
            let _ = pass.run(&gpu, &src, &input);
        });
        println!(
            "[first-gpu] {side}^2 | {first:>8.2} | {second:>8.2} | {:>8.2} ms",
            first - second
        );
    }
}

/// Uma textura RGBA do tamanho pedido, para o passe ter o que iluminar.
fn make_rgba(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("first-gpu src"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    })
}
