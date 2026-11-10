use super::*;
use ph2d_gpu::GpuContext;
use std::sync::OnceLock;

fn gpu() -> Option<GpuContext> {
    static G: OnceLock<Option<GpuContext>> = OnceLock::new();
    G.get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// Imagem determinista com todos os níveis, alfa variável e bordas duras (pixel art).
fn imagem(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    let mut s: u32 = 0x9E37_79B9;
    for _ in 0..w * h {
        for _ in 0..4 {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            v.push((s >> 24) as u8);
        }
    }
    v
}

fn ligada() -> CrtParams {
    CrtParams {
        scanline: 0.35,
        mascara: 0.25,
        curvatura: 0.12,
        brilho: 1.2,
        bloom: 0.15,
    }
}

#[test]
fn a_crt_desligada_e_a_identidade() {
    for &(w, h, e) in &[(1, 1, 1), (7, 5, 3), (32, 17, 4), (64, 64, 1)] {
        let img = imagem(w, h);
        assert_eq!(
            crt_cpu(&img, w, h, e, &CrtParams::default()),
            amplia_nearest(&img, w, h, e),
            "{w}×{h}×{e}"
        );
    }
    // O mapa da curvatura com k = 0 é a identidade até ao tecto Q14.
    for lado in [1, 2, 3, 1000, 2048, LADO_MAXIMO - 1] {
        for x in [0, lado / 3, lado / 2, lado - 1] {
            let r2 = r2_q(x, x.min(lado - 1), lado, lado);
            assert_eq!(curva_eixo(x, lado, r2, 0), Some(x), "lado {lado} x {x}");
        }
    }
}

#[test]
fn a_curvatura_nao_mexe_o_centro_e_apaga_os_cantos() {
    let (w, h, e) = (40, 30, 4);
    let (ow, oh) = (w * e, h * e);
    let p = CrtParams {
        curvatura: 0.3,
        ..CrtParams::default()
    };
    let img = imagem(w, h);
    let saida = crt_cpu(&img, w, h, e, &p);
    let reta = amplia_nearest(&img, w, h, e);
    let px = |b: &[u8], x: u32, y: u32| {
        let i = ((y * ow + x) * 4) as usize;
        [b[i], b[i + 1], b[i + 2], b[i + 3]]
    };
    for (x, y) in [(ow / 2, oh / 2), (ow / 2 - 1, oh / 2 - 1)] {
        assert_eq!(px(&saida, x, y), px(&reta, x, y), "centro ({x},{y})");
    }
    for (x, y) in [(0, 0), (ow - 1, 0), (0, oh - 1), (ow - 1, oh - 1)] {
        assert_eq!(px(&saida, x, y), [0, 0, 0, 255], "canto ({x},{y})");
    }
    // E a curvatura de facto curva: o meio da borda de cima amostra para dentro, não a linha 0.
    let r2 = r2_q(ow / 2, 0, ow, oh);
    let sy = curva_eixo(0, oh, r2, p.k_q12());
    assert!(
        sy.is_none() || sy.unwrap() > 0,
        "a borda curva para dentro: {sy:?}"
    );
}

#[test]
fn a_scanline_escurece_so_a_linha_impar() {
    let (w, h) = (4, 4);
    let img = vec![200u8; (w * h * 4) as usize];
    let p = CrtParams {
        scanline: 0.5,
        ..CrtParams::default()
    };
    let s = crt_cpu(&img, w, h, 2, &p);
    let ow = w * 2;
    assert_eq!(s[0], 200, "linha 0 (par) intacta");
    assert_eq!(s[(ow * 4) as usize], 100, "linha 1 (ímpar) a metade");
}

#[test]
fn a_placa_e_o_gabarito_concordam_na_crt() {
    let Some(gpu) = gpu() else {
        eprintln!("SKIP: sem GPU — gate saltado");
        return;
    };
    let casos = [
        (61, 43, 3, ligada()),
        (64, 64, 4, CrtParams::default()),
        (
            97,
            31,
            5,
            CrtParams {
                curvatura: CURVATURA_MAXIMA,
                bloom: 0.8,
                brilho: 1.6,
                scanline: 1.0,
                mascara: 1.0,
            },
        ),
    ];
    for (w, h, e, p) in casos {
        let img = imagem(w, h);
        let cpu = crt_cpu(&img, w, h, e, &p);
        let placa = gpu::crt_na_placa(&gpu, &img, w, h, e, &p).expect("leitura");
        assert_eq!(cpu.len(), placa.len());
        let mut pior = 0u8;
        let mut onde = 0;
        for (i, (a, b)) in cpu.iter().zip(&placa).enumerate() {
            let d = a.abs_diff(*b);
            if d > pior {
                pior = d;
                onde = i;
            }
        }
        let ow = w * e;
        let px = (onde / 4) as u32;
        eprintln!("CRT {w}×{h}×{e}: pior diferença {pior} nível(is)");
        assert!(
            pior <= 1,
            "{w}×{h}×{e} {p:?}: pior {pior} em ({}, {}) canal {} — cpu {} placa {}",
            px % ow,
            px / ow,
            onde % 4,
            cpu[onde],
            placa[onde]
        );
    }
}

/// Medida (não gate): o passe a 2048² na placa, mínimo de 20, por timestamp query.
#[test]
fn mede_a_crt_na_placa_a_2048() {
    let Some(gpu) = gpu() else {
        eprintln!("SKIP: sem GPU — medida saltada");
        return;
    };
    let (w, h, e) = (512, 512, 4);
    let img = imagem(w, h);
    let crt = gpu::CrtGpu::new(&gpu.device, wgpu::TextureFormat::Rgba8Unorm);
    let c = gpu::prepara(&gpu, &img, w, h, e);
    let o = c
        .origem
        .create_view(&wgpu::TextureViewDescriptor::default());
    let a = c.alvo.create_view(&wgpu::TextureViewDescriptor::default());
    let p = ligada();
    let tem_ts = gpu
        .device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY);
    let qs = tem_ts.then(|| {
        gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("crt ts"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        })
    });
    let resolve = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let periodo = f64::from(gpu.queue.get_timestamp_period());
    let mut tempos = Vec::new();
    for i in 0..23 {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let ts = qs.as_ref().map(|q| wgpu::RenderPassTimestampWrites {
            query_set: q,
            beginning_of_pass_write_index: Some(0),
            end_of_pass_write_index: Some(1),
        });
        let t0 = std::time::Instant::now();
        crt.codifica(&gpu.device, &gpu.queue, &mut enc, &o, &a, c.dims, &p, ts);
        if let Some(q) = &qs {
            enc.resolve_query_set(q, 0..2, &resolve, 0);
            enc.copy_buffer_to_buffer(&resolve, 0, &leitura, 0, 16);
        }
        gpu.queue.submit([enc.finish()]);
        let ms = if qs.is_some() {
            let (tx, rx) = std::sync::mpsc::channel();
            leitura.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            gpu.device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("poll");
            rx.recv().expect("canal").expect("map");
            let m = leitura.slice(..).get_mapped_range().expect("mapa");
            let t = |k: usize| u64::from_le_bytes(m[k * 8..k * 8 + 8].try_into().expect("8"));
            #[allow(clippy::cast_precision_loss)]
            let ms = (t(1).wrapping_sub(t(0))) as f64 * periodo / 1e6;
            drop(m);
            leitura.unmap();
            ms
        } else {
            gpu.device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("poll");
            t0.elapsed().as_secs_f64() * 1e3
        };
        if i >= 3 {
            tempos.push(ms);
        }
    }
    let min = tempos.iter().copied().fold(f64::INFINITY, f64::min);
    let info = gpu.adapter.get_info();
    eprintln!(
        "CRT na placa 2048² ({}, {}): mínimo de 20 = {min:.3} ms",
        info.name,
        if tem_ts {
            "timestamp query"
        } else {
            "relógio com poll"
        }
    );
}
