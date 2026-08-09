//! **A metade DEVICE do kill-criterion do [ADR-0157]** — o número que o ADR nomeia como *o primeiro da
//! implementação* e sem o qual **nenhum passo de grade entra no código** (`CLAUDE.md` §0).
//!
//! A metade CPU já está medida ([`super::cook_probe`]): o cook é `O(nós × N)` exato, a **31,0 ns por
//! (nó · dab)** no pior caso, **serial num núcleo**. A pergunta que decide o desenho é a outra: *quanto
//! custa compor N dabs por pixel no dispositivo?* — porque é a resposta dela que diz se a grade precisa
//! ser grossa (e quanto erro de reconstrução isso assa) ou se ela pode ser o próprio pixel.
//!
//! ## Por que este arquivo mora AQUI e não numa crate-folha
//!
//! O oráculo é [`super::field::compose_at`], que é `pub(super)`. Uma crate irmã só alcançaria a lei se a
//! superfície privada do tool fosse alargada para a workspace inteira — e a lei ainda vai MUDAR de casa
//! na W1 (o ADR pede um dab AUTORADO, pequeno e animável, que não cabe num módulo privado de tool).
//! Medir onde o oráculo já está evita alargar hoje o que se move amanhã.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --lib cook_gpu -- --ignored --nocapture --test-threads=1`
//!
//! ⚠️ `#[ignore]` porque **precisa de adapter**. Sem GPU os dois testes fazem *skip* — e um skip **não
//! é verde**, então quem fecha a linha roda-os na máquina com placa e lê os números.
//!
//! [ADR-0157]: ../../../../../../docs/architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md

use super::field::{DabField, DeformMode, build_rotor_table, compose_at, crosses_to_the_device};
use std::time::Instant;
use wgpu::util::DeviceExt as _;

/// O retrato de um dab no device. ⚠️ Só escalares e pares — ver o comentário do `cook_walk.wgsl`:
/// o que falta num layout desalinhado é sempre o fim, e é onde ninguém olha.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuDab {
    center: [f32; 2],
    mv: [f32; 2],
    perp: [f32; 2],
    inv_r2: f32,
    radius: f32,
    signed_v: f32,
    pressure: f32,
    distortion: f32,
    twist_deg_max: f32,
    mode: u32,
    _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    side: u32,
    dab_count: u32,
    rotor_len: u32,
    _pad: u32,
    origin: [f32; 2],
    step: [f32; 2],
}

/// ⚠️ **A recusa é do lado que se lê.** Um dab com ruído não tem resposta no device (splitmix64 é `u64`,
/// que o WGSL do core não tem), então ele não vira payload — em vez de o shader devolver outra coisa
/// em silêncio.
fn payload(dabs: &[DabField]) -> Vec<GpuDab> {
    dabs.iter()
        .map(|f| {
            assert!(
                crosses_to_the_device(f),
                "este dab carrega value_noise: o port do ruido e' decisao da W1 (ADR-0157)"
            );
            let d = f.device_fields();
            GpuDab {
                center: d.center,
                mv: d.mv,
                perp: d.perp,
                inv_r2: d.inv_r2,
                radius: d.radius,
                signed_v: d.signed,
                pressure: d.pressure,
                distortion: d.distortion,
                twist_deg_max: d.twist_deg_max,
                mode: d.mode,
                _pad: 0,
            }
        })
        .collect()
}

fn context() -> Option<ph2d_gpu::GpuContext> {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
}

/// Cozinha a grade `side × side` no device `iters` vezes e devolve `(campo, ms por despacho)`.
///
/// ⚠️ **Um despacho por submit, e a espera no fim.** Encadear despachos dentro do MESMO passe deixaria o
/// driver sobrepô-los (a WebGPU não os sincroniza sozinha), e o relógio mediria concorrência em vez de
/// custo. Submits consecutivos sobre o MESMO buffer de saída serializam — que é o que se quer medir.
fn cook(
    ctx: &ph2d_gpu::GpuContext,
    dabs: &[DabField],
    side: u32,
    origin: [f32; 2],
    step: [f32; 2],
    iters: u32,
) -> (Vec<[f32; 2]>, f64) {
    let device = &ctx.device;
    let queue = &ctx.queue;
    let gpu_dabs = payload(dabs);
    let rotors = build_rotor_table(360);
    let nodes = (side as usize) * (side as usize);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("liquify:cook-walk"),
        source: wgpu::ShaderSource::Wgsl(include_str!("cook_walk.wgsl").into()),
    });
    let storage = |read_only: bool| wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    let entry = |binding: u32, ty: wgpu::BindingType| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    };
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("liquify:cook-layout"),
        entries: &[
            entry(
                0,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            ),
            entry(1, storage(true)),
            entry(2, storage(true)),
            entry(3, storage(false)),
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("liquify:cook-pl"),
        bind_group_layouts: &[&layout],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("liquify:cook"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let params = Params {
        side,
        dab_count: u32::try_from(gpu_dabs.len()).expect("dab list fits a u32"),
        rotor_len: u32::try_from(rotors.len()).expect("rotor table fits a u32"),
        _pad: 0,
        origin,
        step,
    };
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("liquify:params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let db = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("liquify:dabs"),
        contents: bytemuck::cast_slice(&gpu_dabs),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let rb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("liquify:rotors"),
        contents: bytemuck::cast_slice(&rotors),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let out_bytes = nodes as u64 * 8;
    let ob = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("liquify:out"),
        size: out_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let stage = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("liquify:stage"),
        size: out_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("liquify:bind"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: ub.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: db.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: rb.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: ob.as_entire_binding(),
            },
        ],
    });

    let groups = side.div_ceil(8);
    let dispatch = || {
        let mut enc =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("liquify:cook-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(groups, groups, 1);
        }
        queue.submit(Some(enc.finish()));
    };

    // Aquecimento: o PRIMEIRO despacho carrega a compilação do pipeline, que é custo de UMA vez e não
    // do cook (o `prewarm` do preview já pagou essa lição).
    dispatch();
    let _ = device.poll(wgpu::PollType::wait_indefinitely());

    let t = Instant::now();
    for _ in 0..iters {
        dispatch();
    }
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let ms = t.elapsed().as_secs_f64() * 1e3 / f64::from(iters);

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_buffer_to_buffer(&ob, 0, &stage, 0, out_bytes);
    queue.submit(Some(enc.finish()));
    let slice = stage.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let field = bytemuck::cast_slice::<u8, [f32; 2]>(&slice.get_mapped_range()).to_vec();
    stage.unmap();
    (field, ms)
}

/// Uma pilha de Twist parada — o gesto do report (*"veja as linhas sumindo"*) e o pior caso honesto do
/// cook: todo dab cobre o mesmo lugar, então todo nó daquela vizinhança paga a lista inteira.
fn twist_dabs(n: usize, centre: [f32; 2], radius: f32) -> Vec<DabField> {
    (0..n)
        .map(|k| {
            DabField::new(
                DeformMode::Twist,
                centre,
                radius,
                [0.0, 0.0],
                [0.0, 0.0],
                1.0,
                0.8,
                0.0,
                0.0,
                k as u64 + 1,
            )
        })
        .collect()
}

/// **O GATE.** A segunda implementação da lei responde o que a primeira responde.
///
/// ⚠️ **"Bit-a-bit" não é a política desta casa** (o compositor declara que runtime não é bit-idêntico
/// entre backends — contração FMA), então o template é o do `ImpastoLightPass`: **pior delta E quantos
/// nós diferem**, porque *quão longe* e *quantos* são perguntas diferentes.
#[test]
#[ignore = "precisa de adapter — rode com `-- --ignored` na máquina com GPU"]
fn the_device_walk_reproduces_the_cpu_law() {
    let Some(ctx) = context() else {
        eprintln!("sem adapter: skip");
        return;
    };
    const SIDE: u16 = 64;
    const TOL_PX: f32 = 1e-3;

    for (name, dabs) in [
        ("Twist x32", twist_dabs(32, [32.0, 32.0], 40.0)),
        (
            "Push x32",
            (0..32u16)
                .map(|k| {
                    DabField::new(
                        DeformMode::Push,
                        [20.0 + f32::from(k) * 0.4, 32.0],
                        26.0,
                        [1.5, -0.7],
                        [0.0, 0.0],
                        0.0,
                        0.9,
                        0.0,
                        0.0,
                        u64::from(k) + 1,
                    )
                })
                .collect::<Vec<_>>(),
        ),
        (
            "Pinch x32",
            (0..32u16)
                .map(|k| {
                    DabField::new(
                        DeformMode::Pinch,
                        [32.0, 32.0],
                        40.0,
                        [0.0, 0.0],
                        [0.0, 0.0],
                        -1.0,
                        0.9,
                        0.0,
                        0.0,
                        u64::from(k) + 1,
                    )
                })
                .collect::<Vec<_>>(),
        ),
    ] {
        let (gpu, _) = cook(&ctx, &dabs, u32::from(SIDE), [0.0, 0.0], [1.0, 1.0], 1);
        let mut worst = 0.0_f32;
        let mut differing = 0usize;
        let mut moved = 0usize;
        for y in 0..SIDE {
            for x in 0..SIDE {
                let want = compose_at(&dabs, [f32::from(x), f32::from(y)]);
                let got = gpu[usize::from(y) * usize::from(SIDE) + usize::from(x)];
                if want[0].abs() + want[1].abs() > 0.01 {
                    moved += 1;
                }
                let e = (want[0] - got[0]).abs().max((want[1] - got[1]).abs());
                worst = worst.max(e);
                if e > TOL_PX {
                    differing += 1;
                }
            }
        }
        // Um campo IDENTICAMENTE ZERO passaria em qualquer tolerância — a fixture tem de conter o
        // fenômeno que ela julga.
        assert!(moved > 500, "{name}: a fixture mal deforma ({moved} nós)");
        let total = usize::from(SIDE) * usize::from(SIDE);
        println!(
            "{name}: pior delta {worst:.6} px · {differing} de {total} nós acima de {TOL_PX} · {moved} deformados"
        );
        assert!(
            differing == 0,
            "{name}: {differing} nós divergem (pior {worst:.6} px)"
        );
    }
}

/// **A MEDIÇÃO que o ADR bloqueia.** Custo por (nó · dab) no device, e a tabela de passo derivada dela.
#[test]
#[ignore = "probe: measures, does not assert"]
fn measure_the_device_cook() {
    let Some(ctx) = context() else {
        eprintln!("sem adapter: skip");
        return;
    };
    println!("\n=== o COOK no DEVICE (Twist, pincel cobrindo a grade inteira) ===");
    println!(
        "{:>8} {:>10} {:>12} {:>16} {:>14}",
        "lado", "N dabs", "ms/cook", "ns/(nó·dab)", "M nós"
    );
    for (side, sidef) in [
        (256u32, 256.0f32),
        (512, 512.0),
        (1024, 1024.0),
        (2048, 2048.0),
    ] {
        for n in [16usize, 64, 256] {
            let dabs = twist_dabs(n, [sidef * 0.5, sidef * 0.5], sidef);
            let iters = if side >= 1024 { 8 } else { 32 };
            let (_, ms) = cook(&ctx, &dabs, side, [0.0, 0.0], [1.0, 1.0], iters);
            let nodes = f64::from(side) * f64::from(side);
            let work = nodes * n as f64;
            println!(
                "{side:>8} {n:>10} {ms:>12.3} {:>16.3} {:>14.2}",
                ms * 1e6 / work,
                nodes / 1e6
            );
        }
    }
    println!(
        "\n⚠️ A comparação honesta é contra os 31,0 ns/(nó·dab) SERIAIS de `measure_the_lattice_pitch_table`."
    );
}
