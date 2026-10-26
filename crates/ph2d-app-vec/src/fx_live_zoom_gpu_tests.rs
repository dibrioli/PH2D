//! **G5 (placa) — a paridade do recorte**: cada pixel VISÍVEL do job recortado à vista é o pixel do
//! job inteiro, para os 15 tipos. A fonte é um disco analítico no espaço de tela (o raster do Vello é
//! equivariante a deslocamentos inteiros; o que se prova aqui é a PILHA sobre a janela). Mais a
//! régua de ms da placa do plano §1.

use super::{
    BORDA, CENTRO, Cena, R, SEM_RECORTE, VIEW, ZOOMS, camera, cena, cena_op, inter, kinds,
};
use ph2d_ecs::FxOp;
use ph2d_render::{FxStackPass, Origem};
use ph2d_vector::Affine;

fn gpu() -> Option<ph2d_gpu::GpuContext> {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
}

fn tex(gpu: &ph2d_gpu::GpuContext, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("F34 paridade"),
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
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// O disco de raio `R·zoom` centrado em `centro` (tela), premultiplicado, na janela `[ex0, ey0]+w×h`.
fn disco(w: u32, h: u32, ex0: f64, ey0: f64, centro: [f64; 2], raio: f64) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (ex0 + f64::from(x) + 0.5, ey0 + f64::from(y) + 0.5);
            let d = (sx - centro[0]).hypot(sy - centro[1]);
            let a = (raio - d + 0.5).clamp(0.0, 1.0);
            let o = ((y * w + x) * 4) as usize;
            let c = |v: f64| (v * a).round() as u8;
            out[o..o + 4].copy_from_slice(&[c(235.0), c(175.0), c(60.0), c(255.0)]);
        }
    }
    out
}

fn upload(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture, w: u32, h: u32, bytes: &[u8]) {
    gpu.queue.write_texture(
        t.as_image_copy(),
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

fn readback(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let row = (w * 4).div_ceil(256) * 256;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("F34 readback"),
        size: u64::from(row * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let data = buf.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let o = (y * row) as usize;
        out.extend_from_slice(&data[o..o + (w * 4) as usize]);
    }
    out
}

/// Corre a pilha do job sobre o disco, como o `cook_batch` (segmentos recortados, origem da forma).
fn corre(
    gpu: &ph2d_gpu::GpuContext,
    pass: &mut FxStackPass,
    c: &Cena,
    cam: Affine,
    job: &super::Job,
    centro: [f64; 2],
    raio: f64,
) -> Vec<u8> {
    let (w, h) = (job.key.w, job.key.h);
    let src = tex(gpu, w, h);
    upload(
        gpu,
        &src,
        w,
        h,
        &disco(w, h, job.ex0, job.ey0, centro, raio),
    );
    let dst = tex(gpu, w, h);
    let alc = ph2d_render::stack_read(&job.key.ops) as f32;
    let geom = c.segs(cam, job, [-alc, -alc, w as f32 + alc, h as f32 + alc]);
    let origem = Origem::Ponto(job.ruido);
    pass.run_from_com_origem(gpu, &src, [0, 0], &dst, w, h, &job.key.ops, &geom, origem);
    readback(gpu, &dst, w, h)
}

#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn every_visible_pixel_of_a_clipped_job_is_the_pixel_of_the_whole_job() {
    let Some(gpu) = gpu() else {
        eprintln!("[fx-zoom-paridade] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let (z, foco) = (16.0, BORDA);
    let cam = camera(z, foco, 0.0);
    let centro = [VIEW[2] / 2.0 - (foco[0] * z), VIEW[3] / 2.0 - (foco[1] * z)];
    // A vista, e dois recortes só à esquerda que deslocam a janela por um número PAR e um ÍMPAR de
    // texels — o `round` do WGSL levava o meio ao par, e só o ímpar o mostrava (Morphology, 6/255).
    let janelas = [
        VIEW,
        [-1000.0, -1.0e6, 1.0e6, 1.0e6],
        [-1001.0, -1.0e6, 1.0e6, 1.0e6],
    ];
    let mut piores = Vec::new();
    // Os 15 tipos de omissão, e a Morphology LARGA nos dois sentidos (16 px, semeada pelo raster). A
    // cadeia de saltos do JFA NÃO pede margem além da banda: medido (F34), alargá-la não muda um pixel.
    let larga = |grow: f32| {
        let mut op = FxOp::new(FxOp::MORPHOLOGY);
        op.grow = grow;
        op
    };
    let casos = kinds().map(FxOp::new).chain([larga(1.0), larga(-1.0)]);
    for op in casos {
        let kind = op.kind;
        let c = cena_op(op);
        let inteiro = c.job(cam, SEM_RECORTE).expect("job");
        let a = corre(&gpu, &mut pass, &c, cam, &inteiro, centro, R * z);
        for vista in janelas {
            let recortado = c.job(cam, vista).expect("job");
            assert!(
                recortado.key.w < inteiro.key.w,
                "tipo {kind}: o job devia recortar"
            );
            let b = corre(&gpu, &mut pass, &c, cam, &recortado, centro, R * z);
            let vis = inter(inter(super::rect(&inteiro), super::rect(&recortado)), VIEW);
            let texel = |job: &super::Job, sx: i64, sy: i64| {
                let (x, y) = (sx - job.ex0 as i64, sy - job.ey0 as i64);
                (y * i64::from(job.key.w) + x) as usize * 4
            };
            let mut pior = 0u8;
            for sy in vis[1] as i64..vis[3] as i64 {
                for sx in vis[0] as i64..vis[2] as i64 {
                    let (pa, pb) = (texel(&inteiro, sx, sy), texel(&recortado, sx, sy));
                    for k in 0..4 {
                        pior = pior.max(a[pa + k].abs_diff(b[pb + k]));
                    }
                }
            }
            eprintln!(
                "[fx-zoom-paridade] tipo {kind:>2}: inteiro {}x{} · recortado {}x{} · pior diferença {pior}",
                inteiro.key.w, inteiro.key.h, recortado.key.w, recortado.key.h
            );
            piores.push((kind, recortado.ex0, pior));
        }
    }
    // ±1 é a precisão `f32` das coordenadas LOCAIS (as do recorte são menores, logo mais finas).
    let maus: Vec<_> = piores.iter().filter(|(_, _, p)| *p > 1).collect();
    assert!(
        maus.is_empty(),
        "pixels visíveis diferem do job inteiro: {maus:?}"
    );
}

/// **Os ms da pilha na placa**, por tipo e zoom, no tamanho que o job pede — a régua do plano §1.
#[test]
#[ignore = "needs a real GPU device; sonda F34"]
fn fx_zoom_regua_ms_na_placa() {
    let Some(gpu) = gpu() else {
        eprintln!("[fx-zoom] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    for (nome, foco) in [("centro", CENTRO), ("borda", BORDA)] {
        for kind in kinds() {
            let c = cena(kind);
            for z in ZOOMS {
                let cam = camera(z, foco, 0.0);
                let Some(job) = c.job(cam, VIEW) else {
                    continue;
                };
                let (w, h) = (job.key.w, job.key.h);
                let src = tex(&gpu, w, h);
                let dst = tex(&gpu, w, h);
                let alc = ph2d_render::stack_read(&job.key.ops) as f32;
                let geom = c.segs(cam, &job, [-alc, -alc, w as f32 + alc, h as f32 + alc]);
                let run = |pass: &mut FxStackPass| {
                    let o = Origem::Ponto(job.ruido);
                    pass.run_from_com_origem(
                        &gpu,
                        &src,
                        [0, 0],
                        &dst,
                        w,
                        h,
                        &job.key.ops,
                        &geom,
                        o,
                    );
                    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
                };
                run(&mut pass);
                let mut best = f64::INFINITY;
                for _ in 0..3 {
                    let t = std::time::Instant::now();
                    run(&mut pass);
                    best = best.min(t.elapsed().as_secs_f64() * 1000.0);
                }
                eprintln!(
                    "[fx-zoom-ms] foco {nome:>6} kind {kind:>2} zoom {z:>4}× {w}x{h} segs {:>4} {best:>8.2} ms",
                    geom.len()
                );
            }
        }
    }
}
