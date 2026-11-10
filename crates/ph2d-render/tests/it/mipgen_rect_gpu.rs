//! B20.3: the mip chain of a dirty rect is the whole chain, and the rest of every level is untouched.

use ph2d_gpu::GpuContext;
use ph2d_render::mipgen::{MipGenerator, mip_levels};

const F: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn tex(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test mip rect"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: mip_levels(w, h),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: F,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn escreve(
    gpu: &GpuContext,
    t: &wgpu::Texture,
    nivel: u32,
    (x, y, w, h): (u32, u32, u32, u32),
    b: &[u8],
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: t,
            mip_level: nivel,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        b,
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

/// Every level, tight `w*h*4` rows, level after level.
fn le(gpu: &GpuContext, t: &wgpu::Texture) -> Vec<Vec<u8>> {
    (0..t.mip_level_count())
        .map(|l| {
            let (w, h) = ((t.width() >> l).max(1), (t.height() >> l).max(1));
            let padded = (w * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: u64::from(padded * h),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: t,
                    mip_level: l,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buf,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded),
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
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            let m = buf.slice(..).get_mapped_range().expect("mapped");
            (0..h as usize)
                .flat_map(|r| m[r * padded as usize..][..w as usize * 4].to_vec())
                .collect()
        })
        .collect()
}

#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn mips_do_retangulo_e_a_cadeia_inteira() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    let gen_ = MipGenerator::new(&gpu, F);
    let ruido = |w: u32, h: u32, s: u32| -> Vec<u8> {
        (0..w * h * 4)
            .map(|i| ((i ^ s).wrapping_mul(2_654_435_761) >> 11) as u8)
            .collect()
    };
    for (w, h) in [(101u32, 67u32), (128, 128)] {
        for rect in [
            (13u32, 9u32, 17u32, 11u32),
            (0, 0, 3, 2),
            (w - 5, h - 1, 5, 1),
            (40, 30, 1, 1),
        ] {
            let (base, novo) = (ruido(w, h, 7), ruido(rect.2, rect.3, 1_234));
            let (a, b) = (tex(&gpu, w, h), tex(&gpu, w, h));
            for t in [&a, &b] {
                escreve(&gpu, t, 0, (0, 0, w, h), &base);
                gen_.run(&gpu, t, t.mip_level_count());
                escreve(&gpu, t, 0, rect, &novo);
            }
            // Sentinela no canto oposto do nível 1 (o 2 não o relê): o caminho por região não lhe toca.
            let nivel = 1u32;
            let (lw, lh) = (w >> nivel, h >> nivel);
            let canto = if rect.0 < w / 2 {
                (lw - 1, lh - 1)
            } else {
                (0, 0)
            };
            escreve(&gpu, &a, nivel, (canto.0, canto.1, 1, 1), &[1, 2, 3, 4]);
            gen_.run_rect(&gpu, &a, a.mip_level_count(), rect);
            gen_.run(&gpu, &b, b.mip_level_count());
            let (ra, rb) = (le(&gpu, &a), le(&gpu, &b));
            let i = ((canto.1 * lw + canto.0) * 4) as usize;
            assert_eq!(
                &ra[nivel as usize][i..i + 4],
                &[1, 2, 3, 4],
                "{w}x{h} {rect:?}: o canto longe foi reescrito"
            );
            for (l, (x, y)) in ra.iter().zip(&rb).enumerate() {
                let mut x = x.clone();
                if l == nivel as usize {
                    x[i..i + 4].copy_from_slice(&y[i..i + 4]);
                }
                assert!(
                    x == *y,
                    "{w}x{h} {rect:?}: nível {l} difere da cadeia inteira"
                );
            }
        }
    }
}
