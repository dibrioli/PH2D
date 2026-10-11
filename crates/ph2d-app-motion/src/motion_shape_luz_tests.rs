//! ⭐ **O CONTRATO da camada do Motion para a luz 2D** (W5e, `CamadaDeOclusao`): o que as formas do
//! Motion desenham para a luz é a ÁREA delas — dois discos da placa (as cópias de uma elipse unitária)
//! contra a área analítica de cada píxel —, só para um objeto Motion que a luz pediu, e nada sem a
//! placa ativa. A paridade da luz com essa área é da luz (`ph2d-app-light2d`, a cena 7).

use super::*;
use crate::motion_shape_gen::VecPathStore;
use ph2d_eval_motion::VectorInstance;

/// Os discos: `(centro, raio)` — os da cena 7 da luz.
const DISCOS: [([f64; 2], f64); 2] = [([-2.6, 1.6], 0.9), ([2.4, 1.6], 0.5)];
/// A janela do mundo e o alvo: 80 px por metro.
const MUNDO: [f64; 4] = [-4.0, 0.0, 4.0, 3.0];
const TAMANHO: (u32, u32) = (640, 240);

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
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
fn alfa_desenhado(gpu: &GpuContext, o: &mut OclusaoDoMotion<'_>, quem: &[u64]) -> (bool, Vec<f64>) {
    let (w, h) = TAMANHO;
    let alvo = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("motion/luz/contrato"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ph2d_light2d_gpu::FORMATO_DA_OCLUSAO,
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

/// O erro da área TOTAL e o pior píxel — MEDIDOS + 10 % (RTX 5060 Ti): −0,184 % (o polígono da placa
/// fica DENTRO do círculo, a 0,25 px) e 0,118.
const TOL_AREA_TOTAL: f64 = 0.002_03;
const TOL_PIXEL: f64 = 0.13;

#[test]
fn a_camada_do_motion_para_a_luz_e_a_area_das_formas() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut store = VecPathStore::default();
    let h = store.push(ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5));
    let insts: Vec<VectorInstance> = DISCOS
        .iter()
        .map(|(c, r)| VectorInstance {
            geometry_id: h,
            texture_id: 0,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            premultiplied: 0.0,
            world_pos: [c[0] as f32, c[1] as f32],
            size: [(2.0 * r) as f32; 2],
            basis: [1.0, 0.0, 0.0, 1.0],
            tint: [1.0; 4],
            anchor: [0.0, 0.0],
            sampling: 0,
            blend_linha: 0,
            mistura: Default::default(),
        })
        .collect();
    let mut geos = GeometriasDaPlaca::default();
    let mut placa = PlacaDeFormas::default();
    let mut o = OclusaoDoMotion {
        placa: &mut placa,
        geometrias: &geos,
        dispositivo: None,
        objetos: vec![42],
    };
    let (desenhou, _) = alfa_desenhado(&gpu, &mut o, &[42]);
    assert!(
        !desenhou,
        "sem a placa ativa (nenhuma forma neste quadro) não há camada"
    );
    assert!(placa.decide(
        true,
        &insts,
        &store,
        &mut geos,
        ph2d_vector::Affine::IDENTITY
    ));
    let mut o = OclusaoDoMotion {
        placa: &mut placa,
        geometrias: &geos,
        dispositivo: None,
        objetos: vec![42],
    };
    let (desenhou, alfa) = alfa_desenhado(&gpu, &mut o, &[7]);
    assert!(!desenhou, "a luz não pediu nenhum objeto Motion");
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
        "o Motion para a luz: a área {soma:.1} × a exata {soma_exata:.1} px² ({:+.3} %), o pior píxel {pior:.3}",
        100.0 * erro_total
    );
    assert!(
        erro_total.abs() <= TOL_AREA_TOTAL,
        "a área das formas: {erro_total:+.4}"
    );
    assert!(pior <= TOL_PIXEL, "um píxel fora do disco: {pior:.3}");
}
