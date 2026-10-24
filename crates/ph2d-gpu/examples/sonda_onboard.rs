//! Em que FASE um processo que escolhe a placa integrada aparece na tabela da NVIDIA? Fases de 3 s
//! com carimbo; o `nvidia-smi` amostra-se por fora. É a medição que prende a fila de testes ao
//! Vulkan (`docs/DevOps/A_VEZ_DA_PLACA.md` §7; 23/10: com o GL, aparece ao enumerar; só Vulkan, nunca).
//!
//!     bash scripts/ph2d-run.sh cargo run --profile smoke -p ph2d-gpu --example sonda_onboard -- vulkan
//!
//!     arg: `todos` (Instance::default, GL incluído) | `vulkan` (só Vulkan)

use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn fase(nome: &str) {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    println!("FASE {nome} {t:.2}");
    std::thread::sleep(Duration::from_secs(3));
}

fn main() {
    ph2d_gpu::vez_da_placa();
    fase("0_vez");
    let modo = std::env::args().nth(1).unwrap_or_default();
    let inst = if modo == "vulkan" {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle_from_env()
        })
    } else {
        wgpu::Instance::default()
    };
    fase("1_instancia");
    let adapters = pollster::block_on(inst.enumerate_adapters(wgpu::Backends::all()));
    for a in &adapters {
        let i = a.get_info();
        println!("  {} ({:?} / {:?})", i.name, i.device_type, i.backend);
    }
    fase("2_enumerou");
    let a = adapters
        .into_iter()
        .find(|a| {
            let i = a.get_info();
            i.device_type == wgpu::DeviceType::IntegratedGpu && i.backend == wgpu::Backend::Vulkan
        })
        .expect("sem integrada");
    let (device, queue) =
        pollster::block_on(a.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 1 << 20,
        usage: wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buf, 0, &[1u8; 1024]);
    queue.submit([]);
    fase("3_device_integrada");
    println!("FASE 4_fim");
}
