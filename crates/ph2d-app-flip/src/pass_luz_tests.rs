//! ⭐ **O CONTRATO da camada do Flip para a luz 2D** (W5e, `CamadaDeOclusao`): o que o Flip desenha
//! para a luz é a ÁREA do traço — dois discos de tinta (o traço de largura = o diâmetro) contra a área
//! analítica de cada píxel — no lugar do mundo pedido, e nada para quem a luz não pediu. A paridade da
//! luz com essa área é da luz (`ph2d-app-light2d`, a cena 7).

use super::*;
use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};

/// Os discos: `(centro, raio)` — os da cena 7 da luz.
const DISCOS: [([f64; 2], f64); 2] = [([-2.6, 1.6], 0.9), ([2.4, 1.6], 0.5)];
/// A janela do mundo e o alvo: 80 px por metro.
const MUNDO: [f64; 4] = [-4.0, 0.0, 4.0, 3.0];
const TAMANHO: (u32, u32) = (640, 240);

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

fn documento() -> (FlipDoc, FlipObjectId) {
    let mut doc = FlipDoc::default();
    let id = doc.push_object("Tinta");
    let obj = doc.object_mut(id).expect("objeto");
    let camada = obj.add_layer("Tinta");
    let d = obj
        .insert_frame(camada, 0, Hold::Implicit, KeyKind::Keyframe)
        .expect("desenho");
    let tracos = &mut obj.drawing_mut(d).expect("desenho").strokes;
    for (c, r) in DISCOS {
        let mut s = FlipStroke::new();
        for dx in [-5e-4, 5e-4] {
            s.push_point(Point {
                pos: Vec2::new((c[0] + dx) as f32, c[1] as f32),
                width: (2.0 * r) as f32,
                opacity: 1.0,
                color: Rgba([1.0; 4]),
            });
        }
        s.hardness = 1.0;
        tracos.push(s);
    }
    (doc, id)
}

/// A área analítica dos discos em cada píxel (16×16 subamostras), linha 0 = `MUNDO[3]`.
fn area_exata() -> Vec<f64> {
    let (w, h) = TAMANHO;
    let [x0, y0, x1, y1] = MUNDO;
    let (pw, ph) = ((x1 - x0) / f64::from(w), (y1 - y0) / f64::from(h));
    (0..h * w)
        .map(|k| {
            let (i, j) = (f64::from(k % w), f64::from(k / w));
            let mut dentro = 0_u32;
            for sj in 0..16 {
                for si in 0..16 {
                    let x = x0 + (i + (f64::from(si) + 0.5) / 16.0) * pw;
                    let y = y1 - (j + (f64::from(sj) + 0.5) / 16.0) * ph;
                    if DISCOS.iter().any(|(c, r)| (x - c[0]).hypot(y - c[1]) <= *r) {
                        dentro += 1;
                    }
                }
            }
            f64::from(dentro) / 256.0
        })
        .collect()
}

/// O alfa de cada píxel do alvo depois de `desenhar` (o alvo nasce limpo).
fn alfa_desenhado(gpu: &GpuContext, o: &mut OclusaoDoFlip<'_>, quem: &[u64]) -> (bool, Vec<f64>) {
    let (w, h) = TAMANHO;
    let alvo = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("flip/luz/contrato"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMATO_DA_OCLUSAO,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let vista = alvo.create_view(&wgpu::TextureViewDescriptor::default());
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &vista,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    }));
    gpu.queue.submit([enc.finish()]);
    let desenhou = o.desenhar(gpu, &vista, TAMANHO, MUNDO, quem);
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(w * h * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        alvo.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: None,
            },
        },
        alvo.size(),
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let alfa = buf.slice(..).get_mapped_range()[..]
        .chunks_exact(4)
        .map(|p| f64::from(p[3]) / 255.0)
        .collect();
    (desenhou, alfa)
}

/// O erro da área TOTAL e o pior píxel — MEDIDOS + 10 % (RTX 5060 Ti): +0,078 % e 0,146 (a borda do
/// traço é a do rasterizador, antisserrilhada; o interior é 1).
const TOL_AREA_TOTAL: f64 = 0.000_86;
const TOL_PIXEL: f64 = 0.161;

#[test]
fn a_camada_do_flip_para_a_luz_e_a_area_do_traco() {
    let Some(gpu) = gpu() else {
        return;
    };
    let (doc, id) = documento();
    let mut entidades = FlipEntityMap::new();
    entidades.insert(id, 42);
    let (playhead, mut composite) = (Playhead::default(), None);
    let mut o = OclusaoDoFlip {
        doc: &doc,
        composite: &mut composite,
        models: &[],
        playhead: &playhead,
        entidades: &entidades,
    };
    let (desenhou, alfa) = alfa_desenhado(&gpu, &mut o, &[7]);
    assert!(!desenhou, "a luz não pediu este objeto");
    assert!(alfa.iter().all(|a| *a == 0.0), "e nada foi desenhado");
    let (desenhou, alfa) = alfa_desenhado(&gpu, &mut o, &[42]);
    assert!(desenhou);
    let exata = area_exata();
    let (soma, soma_exata) = (alfa.iter().sum::<f64>(), exata.iter().sum::<f64>());
    let pior = alfa
        .iter()
        .zip(&exata)
        .map(|(a, e)| (a - e).abs())
        .fold(0.0_f64, f64::max);
    let erro_total = (soma - soma_exata) / soma_exata;
    eprintln!(
        "o Flip para a luz: a área {soma:.1} × a exata {soma_exata:.1} px² ({:+.3} %), o pior píxel {pior:.3}",
        100.0 * erro_total
    );
    assert!(
        erro_total.abs() <= TOL_AREA_TOTAL,
        "a área do traço: {erro_total:+.4}"
    );
    assert!(pior <= TOL_PIXEL, "um píxel fora do disco: {pior:.3}");
}
