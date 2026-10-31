//! Quanto custa a advecção na placa: ms de PLACA por passada (timestamps) a 4096×2560 de
//! cobertura total e numa região suja de 1024², o upload/readback da folha e da região, e a
//! CPU (`advect_jacobi`, a rota do produto) no MESMO processo, intercalada. Resumo = mínimo.
//!
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo run --profile smoke -p ph2d-wet-gpu --example mede_advect
//! ```
//! (Radeon: as env vars do README §2b de docs/Painter/49_realshader/.)

use std::time::Instant;

use ph2d_gpu::GpuContext;
use ph2d_wet_gpu::{AdvectGpu, PASSADAS, Rect, Uniforms, cena, pack_state};
use ph2d_wet_paint::sim::Sim;
use ph2d_wet_paint::solver::advect_jacobi;
use ph2d_wet_paint::tuning::{Knob, Tuning};

const W: usize = 4096;
const H: usize = 2560;
const BLOCO: u32 = 10;
const RONDAS: usize = 7;

fn min_med(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn espera(gpu: &GpuContext) {
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}

fn main() {
    eprintln!(
        "loadavg (início): {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("GpuContext");
    let info = gpu.adapter.get_info();
    eprintln!(
        "placa: {} · {:?} · {}",
        info.name, info.backend, info.driver_info
    );
    if !gpu
        .device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
    {
        eprintln!("⛔ sem TIMESTAMP_QUERY — nada medido");
        std::process::exit(3);
    }
    let periodo = f64::from(gpu.queue.get_timestamp_period());
    let p = Sim::default().gather_params(&Tuning::default());
    let mv = p.k(Knob::MaxVelocity);
    let mut g = cena::poca(W, H, 1, 0.9);
    let a = AdvectGpu::new(&gpu, &g);
    let cheia = Rect::full(&g);
    let suja = Rect {
        x0: 1537,
        y0: 769,
        x1: 2560,
        y1: 1792,
    };
    a.upload(&gpu, &g, &Uniforms::from_grid(&g, mv, cheia));
    espera(&gpu);

    let nq = BLOCO * 4 * 2;
    let qs = gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("mede_advect"),
        ty: wgpu::QueryType::Timestamp,
        count: nq,
    });
    let resolve = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("resolve"),
        size: u64::from(nq) * 8,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: u64::from(nq) * 8,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let idx = |step: u32, k: u32| 2 * (step * 4 + k);
    // Um bloco de BLOCO passos sobre `r`: ms de placa médios por passo, por passada.
    let bloco = |r: Rect| -> [f64; 4] {
        a.set_uniforms(&gpu, &Uniforms::from_grid(&g, mv, r));
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        a.encode(&mut enc, r, BLOCO, Some((&qs, &idx)));
        enc.resolve_query_set(&qs, 0..nq, &resolve, 0);
        enc.copy_buffer_to_buffer(&resolve, 0, &leitura, 0, u64::from(nq) * 8);
        gpu.queue.submit([enc.finish()]);
        leitura
            .slice(..)
            .map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        espera(&gpu);
        let mut out = [0.0; 4];
        {
            let d = leitura.slice(..).get_mapped_range().expect("mapped");
            let ts: &[u64] = bytemuck::cast_slice(&d);
            for s in 0..BLOCO {
                for (k, o) in out.iter_mut().enumerate() {
                    let i = idx(s, k as u32) as usize;
                    *o += ts[i + 1].saturating_sub(ts[i]) as f64 * periodo / 1e6;
                }
            }
        }
        leitura.unmap();
        out.map(|v| v / f64::from(BLOCO))
    };
    let nq6 = BLOCO * 24 * 2;
    let qs6 = gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("mede_advect suave"),
        ty: wgpu::QueryType::Timestamp,
        count: nq6,
    });
    let resolve6 = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("resolve6"),
        size: u64::from(nq6) * 8,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura6 = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura6"),
        size: u64::from(nq6) * 8,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    // O quadro de suavização inteiro (smooth → momentum → advecção): ms de placa por passo.
    a.set_knobs(&gpu, &ph2d_wet_gpu::Knobs::from_params(&p, g.flow.rf));
    let quadro = |r: Rect, plano: &[usize]| -> f64 {
        let nq = BLOCO * plano.len() as u32 * 2;
        let mut u = Uniforms::from_grid(&g, mv, r);
        u.gy = 0.012;
        a.set_uniforms(&gpu, &u);
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        a.encode_plano(&mut enc, &u, BLOCO, plano, Some(&qs6));
        enc.resolve_query_set(&qs6, 0..nq, &resolve6, 0);
        enc.copy_buffer_to_buffer(&resolve6, 0, &leitura6, 0, u64::from(nq) * 8);
        gpu.queue.submit([enc.finish()]);
        leitura6
            .slice(..)
            .map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        espera(&gpu);
        let soma: u64 = {
            let d = leitura6.slice(..).get_mapped_range().expect("mapped");
            let ts: &[u64] = bytemuck::cast_slice(&d);
            (0..nq as usize / 2)
                .map(|i| ts[2 * i + 1].saturating_sub(ts[2 * i]))
                .sum()
        };
        leitura6.unmap();
        soma as f64 * periodo / 1e6 / f64::from(BLOCO)
    };
    let suave = |r: Rect| quadro(r, &ph2d_wet_gpu::PLANO_SUAVE);
    let campo = |r: Rect| quadro(r, &ph2d_wet_gpu::PLANO_CAMPO);
    // O passo sem secagem inteiro, médio sobre o ciclo de 12 quadros (mmc de 2, 3 e 4).
    let passo = |r: Rect| -> f64 {
        (1..=12u64)
            .map(|n| quadro(r, &ph2d_wet_gpu::plano_passo(n, false)))
            .sum::<f64>()
            / 12.0
    };
    let st_full = pack_state(&g.film, &g.susp);
    let upload_cheia = || {
        let t = Instant::now();
        gpu.queue
            .write_buffer(&a.st, 0, bytemuck::cast_slice(&st_full));
        gpu.queue
            .write_buffer(&a.rgb, 0, bytemuck::cast_slice(&g.susp_rgb));
        gpu.queue.submit([]);
        espera(&gpu);
        t.elapsed().as_secs_f64() * 1e3
    };
    let upload_suja = || {
        let t = Instant::now();
        for y in suja.y0..=suja.y1 {
            let i0 = suja.x0 as usize + y as usize * g.s;
            let i1 = i0 + suja.cols() as usize;
            gpu.queue
                .write_buffer(&a.st, i0 as u64 * 8, bytemuck::cast_slice(&st_full[i0..i1]));
            gpu.queue.write_buffer(
                &a.rgb,
                i0 as u64 * 12,
                bytemuck::cast_slice(&g.susp_rgb[i0..i1]),
            );
        }
        gpu.queue.submit([]);
        espera(&gpu);
        t.elapsed().as_secs_f64() * 1e3
    };
    let readback_cheia = || {
        let t = Instant::now();
        let _ = a.readback(&gpu);
        t.elapsed().as_secs_f64() * 1e3
    };
    let (cols, rows) = (u64::from(suja.cols()), u64::from(suja.rows()));
    let stage = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("stage suja"),
        size: cols * rows * 20,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let readback_suja = || {
        let t = Instant::now();
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        for (k, y) in (suja.y0..=suja.y1).enumerate() {
            let i0 = (suja.x0 as u64) + y as u64 * g.s as u64;
            let o = k as u64 * cols;
            enc.copy_buffer_to_buffer(&a.st, i0 * 8, &stage, o * 8, cols * 8);
            enc.copy_buffer_to_buffer(&a.rgb, i0 * 12, &stage, rows * cols * 8 + o * 12, cols * 12);
        }
        gpu.queue.submit([enc.finish()]);
        stage
            .slice(..)
            .map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        espera(&gpu);
        let soma: f32 = {
            let d = stage.slice(..).get_mapped_range().expect("mapped");
            bytemuck::cast_slice::<u8, f32>(&d)
                .iter()
                .step_by(4096)
                .sum()
        };
        stage.unmap();
        std::hint::black_box(soma);
        t.elapsed().as_secs_f64() * 1e3
    };

    // Aquecer.
    let _ = bloco(cheia);
    let _ = bloco(suja);
    let _ = suave(cheia);
    let _ = campo(cheia);
    let mut v: Vec<Vec<f64>> = vec![Vec::new(); 23];
    for _ in 0..RONDAS {
        let c = bloco(cheia);
        let s = bloco(suja);
        for k in 0..4 {
            v[k].push(c[k]);
            v[4 + k].push(s[k]);
        }
        v[8].push(upload_cheia());
        v[9].push(readback_cheia());
        v[10].push(upload_suja());
        v[11].push(readback_suja());
        v[12].push(c.iter().sum());
        v[13].push(s.iter().sum());
        v[14].push(suave(cheia));
        v[15].push(suave(suja));
        // O quadro do produto: passo suave na região + readback só da região suja.
        let t = suave(suja);
        v[16].push(t + readback_suja());
        v[17].push(campo(cheia));
        v[18].push(campo(suja));
        let t = campo(suja);
        v[19].push(t + readback_suja());
        v[20].push(passo(cheia));
        v[21].push(passo(suja));
        let t = passo(suja);
        v[22].push(t + readback_suja());
    }
    // A CPU (a rota do produto) depois — o estado da placa não depende dela.
    let mut cpu = Vec::new();
    for _ in 0..RONDAS.min(5) {
        let t = Instant::now();
        advect_jacobi(&mut g, &p, 0.0, 0.0);
        cpu.push(t.elapsed().as_secs_f64() * 1e3);
    }
    println!("| medida | mínimo ms | mediana ms |");
    println!("|---|---|---|");
    let nomes: Vec<String> = PASSADAS
        .iter()
        .map(|n| format!("placa 4096×2560 {n}"))
        .chain(PASSADAS.iter().map(|n| format!("placa suja 1024² {n}")))
        .chain(
            [
                "upload folha (st+rgb, 210 MB)",
                "readback folha (st+rgb)",
                "upload suja 1024² (por linha)",
                "readback suja 1024² (por linha)",
                "placa 4096×2560 passo inteiro",
                "placa suja 1024² passo inteiro",
                "placa 4096×2560 quadro suave (smooth+momentum+advecção)",
                "placa suja 1024² quadro suave",
                "quadro suave suja 1024² + readback da região (parede)",
                "placa 4096×2560 quadro de campo (campo+carimbo+momentum+advecção)",
                "placa suja 1024² quadro de campo",
                "quadro de campo suja 1024² + readback da região (parede)",
                "placa 4096×2560 PASSO sem secagem (média do ciclo de 12)",
                "placa suja 1024² PASSO sem secagem",
                "PASSO sem secagem suja 1024² + readback da região (parede)",
            ]
            .map(String::from),
        )
        .collect();
    for (n, mut x) in nomes.iter().zip(v) {
        let (mn, md) = min_med(&mut x);
        println!("| {n} | {mn:.3} | {md:.3} |");
    }
    let (mn, md) = min_med(&mut cpu);
    println!("| CPU advect_jacobi 4096×2560 (rayon) | {mn:.3} | {md:.3} |");
    eprintln!(
        "loadavg (fim): {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
