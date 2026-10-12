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
        .as_chunks::<4>()
        .0
        .iter()
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

/// Dois objetos Flip, um disco cada (os de [`DISCOS`]): `(doc, [A, B])`.
fn dois_objetos() -> (FlipDoc, [FlipObjectId; 2]) {
    let mut doc = FlipDoc::default();
    let ids = DISCOS.map(|(c, r)| {
        let id = doc.push_object("Tinta");
        let obj = doc.object_mut(id).expect("objeto");
        let camada = obj.add_layer("Tinta");
        let d = obj
            .insert_frame(camada, 0, Hold::Implicit, KeyKind::Keyframe)
            .expect("desenho");
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
        obj.drawing_mut(d).expect("desenho").strokes.push(s);
        id
    });
    (doc, ids)
}

/// O centro de cada disco no alvo (80 px/m, linha 0 = `MUNDO[3]`), lido do `game_rt` (`Rgba16Float`).
fn centros_lidos(gpu: &GpuContext, rt: &ph2d_render::GameRt) -> [f64; 2] {
    let (w, h) = TAMANHO;
    let linha = (w * 8).next_multiple_of(256);
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(linha * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        rt.texture().as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(linha),
                rows_per_image: None,
            },
        },
        rt.texture().size(),
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let px = buf.slice(..).get_mapped_range().to_vec();
    DISCOS.map(|(c, _)| {
        let (x, y) = ((c[0] - MUNDO[0]) * 80.0, (MUNDO[3] - c[1]) * 80.0);
        let i = (y as u32 * linha + x as u32 * 8) as usize;
        f64::from(half::f16::from_le_bytes([px[i], px[i + 1]]).to_f32())
    })
}

/// ⭐ W5j — **quem não recebe a luz fica na cor de sempre** (o ABERTO 4 da W5e): dois objetos Flip
/// brancos sob uma luz UNIFORME de meia intensidade (`F = π`); com B marcado `NaoRecebeLuz`, A fica a
/// ½ e B a 1. Controlos na mesma corrida: ninguém de fora ⇒ os dois a ½; a luz apagada ⇒ os dois a 1.
/// (Antes da W5j o blit acendia o Flip inteiro ⇒ B a ½, RED.)
#[test]
fn quem_nao_recebe_a_luz_no_flip_fica_na_cor_de_sempre() {
    use ph2d_flip_render::{FlipCompose, FlipRenderer};
    use ph2d_render::{Camera2d, GameRt, LuzRecebida};
    let Some(gpu) = gpu() else {
        return;
    };
    let (doc, [a, b]) = dois_objetos();
    let fluencia = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("flip/luz/uniforme"),
        size: wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let pi = half::f16::from_f32(std::f32::consts::PI);
    let texel: Vec<u8> = (0..16).flat_map(|_| pi.to_le_bytes()).collect();
    gpu.queue.write_texture(
        fluencia.as_image_copy(),
        &texel,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(16),
            rows_per_image: None,
        },
        fluencia.size(),
    );
    let vista = fluencia.create_view(&wgpu::TextureViewDescriptor::default());
    let sem_luz = std::collections::BTreeSet::new();
    let luz = LuzRecebida {
        fluencia: &vista,
        origem_m: [MUNDO[0], MUNDO[1]],
        texel_m: 4.0,
        tamanho: [2, 2],
        sem_luz: &sem_luz,
    };
    let mapa = [[1.0 / 80.0, 0.0, MUNDO[0]], [0.0, -1.0 / 80.0, MUNDO[3]]];
    let camera = Camera2d {
        center: [0.0, 1.5],
        height_world: 3.0,
        ..Camera2d::default()
    };
    let janela = ph2d_host::WindowSize::new(TAMANHO.0, TAMANHO.1);
    let mut raster = FlipRenderer::new(&gpu.device, GameRt::FORMAT);
    let mut compose = FlipCompose::new(&gpu.device, GameRt::FORMAT);
    let mut composite = None;
    let playhead = Playhead::default();
    let mut corre = |acesa: bool, de_fora: Vec<u64>| {
        compose.receber_luz(&gpu, acesa.then_some((&luz, mapa)));
        compose.nao_recebem(de_fora);
        let rt = GameRt::new(&gpu, TAMANHO);
        crate::pass::render(
            &doc,
            &mut raster,
            &mut compose,
            &mut composite,
            None,
            ph2d_flip::FlipTarget::default(),
            &[],
            &playhead,
            None,
            None,
            &rt,
            &camera,
            janela,
            None,
            &gpu,
        );
        centros_lidos(&gpu, &rt)
    };
    let casos = [
        ("ninguém de fora", true, vec![], [0.5, 0.5]),
        ("B de fora", true, vec![b.0], [0.5, 1.0]),
        ("A de fora", true, vec![a.0], [1.0, 0.5]),
        ("apagada, B de fora", false, vec![b.0], [1.0, 1.0]),
    ];
    for (nome, acesa, de_fora, esperado) in casos {
        let lido = corre(acesa, de_fora);
        eprintln!("{nome}: A {:.4} · B {:.4}", lido[0], lido[1]);
        for k in 0..2 {
            assert!(
                (lido[k] - esperado[k]).abs() <= 2e-3,
                "{nome}: o disco {k} leu {:.4}, esperava {}",
                lido[k],
                esperado[k]
            );
        }
    }
}
