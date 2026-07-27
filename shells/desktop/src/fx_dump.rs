//! **O DESPEJO dos pixels do FX** — a sonda que só o app pode dar.
//!
//! Irmão de [`crate::fx_live`] pelo teto de LOC, e existe por um motivo medido: o smoke reporta
//! dentes no feather DENTRO do app, e a sonda headless (`fx_look_probe`) **não os reproduz sob
//! nenhuma condição** — geometria esparsa, densa, rasterizador próprio ou Vello, tudo dá ripple no
//! nível do controle. Cada diferença entre as duas foi eliminada por medição; o que resta é o que
//! só o app tem, que são os pixels dele.
//!
//! `PH2D_FX_DUMP=<dir>` escreve, por forma cozida e **uma vez só**:
//!
//! - `path<id>.ppm` — a saída do FX, RGB composto sobre o cinza do app (é a foto);
//! - `path<id>.pgm` — o ALFA cru dessa saída (é onde a rampa do feather vive);
//! - `path<id>.txt` — os segmentos da silhueta e os parâmetros da pilha.
//!
//! Com isso a distância prevista pela GEOMETRIA e o alfa que a GPU de facto escreveu passam a ser
//! comparáveis offline. Se discordarem, o campo está exato para uma forma diferente da que o Vello
//! desenhou naquela textura — que é a única hipótese que sobrou.

use std::collections::BTreeSet;
use std::io::Write;

use ph2d_gpu::GpuContext;
use ph2d_render::FxOpGpu;
use ph2d_vec_scene::VecPathId;

/// Quem já foi despejado — o recook roda por frame, e um despejo por frame enche o disco.
#[derive(Default)]
pub(crate) struct FxDump {
    done: BTreeSet<u64>,
}

impl FxDump {
    /// Despeja `tex` (a saída do FX, `Rgba8Unorm`) se `PH2D_FX_DUMP` estiver posto e esta forma
    /// ainda não tiver sido despejada.
    // ⚠️ Oito argumentos porque um despejo honesto precisa do que a GPU recebeu E do que ela
    // devolveu; agrupá-los esconderia justamente o par (geometria, saída) que se confronta.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn maybe(
        &mut self,
        gpu: &GpuContext,
        id: VecPathId,
        tex: &wgpu::Texture,
        w: u32,
        h: u32,
        ops: &[FxOpGpu],
        geom: &[[f32; 4]],
    ) {
        let Ok(dir) = std::env::var("PH2D_FX_DUMP") else {
            return;
        };
        let key = format!("{id:?}")
            .bytes()
            .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(u64::from(b)));
        if !self.done.insert(key) {
            return;
        }
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let px = readback(gpu, tex, w, h);
        let stem = format!("{dir}/path{}", self.done.len());
        write_ppm(&format!("{stem}.ppm"), &px, w, h);
        write_pgm(&format!("{stem}.pgm"), &px, w, h);
        if let Ok(mut f) = std::fs::File::create(format!("{stem}.txt")) {
            let _ = writeln!(f, "id {id:?}  dims {w} {h}  segs {}", geom.len());
            for o in ops {
                let _ = writeln!(
                    f,
                    "op kind {} sigma {} off {:?} opacity {} mode {}",
                    o.kind, o.sigma_px, o.offset_px, o.opacity, o.mode
                );
            }
            for s in geom {
                let _ = writeln!(f, "seg {} {} {} {}", s[0], s[1], s[2], s[3]);
            }
        }
        eprintln!("[fx-dump] {stem}.{{ppm,pgm,txt}}");
    }
}

/// RGB composto sobre o cinza do app. ⚠️ A saída do passe é RGBA **RETO** (o resolve divide pelo
/// alfa), então o over é `a·rgb + (1−a)·bg` — compor como premultiplicado clareia toda borda
/// parcial, que é exatamente onde estes efeitos vivem.
fn write_ppm(path: &str, px: &[u8], w: u32, h: u32) {
    let bg = [0x2c_u8, 0x2e, 0x33];
    let mut body = Vec::with_capacity((w * h * 3) as usize);
    for i in 0..(w * h) as usize {
        let o = i * 4;
        let a = f32::from(px[o + 3]) / 255.0;
        for c in 0..3 {
            let v = a.mul_add(f32::from(px[o + c]), (1.0 - a) * f32::from(bg[c]));
            body.push(v.round().clamp(0.0, 255.0) as u8);
        }
    }
    if let Ok(mut f) = std::fs::File::create(path) {
        let _ = write!(f, "P6\n{w} {h}\n255\n");
        let _ = f.write_all(&body);
    }
}

/// O ALFA cru — a rampa do feather é ELE, e compor sobre um fundo a esconderia.
fn write_pgm(path: &str, px: &[u8], w: u32, h: u32) {
    let body: Vec<u8> = (0..(w * h) as usize).map(|i| px[i * 4 + 3]).collect();
    if let Ok(mut f) = std::fs::File::create(path) {
        let _ = write!(f, "P5\n{w} {h}\n255\n");
        let _ = f.write_all(&body);
    }
}

fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fx dump readback"),
        size: u64::from(padded) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
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
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let _ = rx.recv();
    let view = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    for row in 0..h as usize {
        let s = row * padded as usize;
        out.extend_from_slice(&view[s..s + unpadded as usize]);
    }
    drop(view);
    staging.unmap();
    out
}
