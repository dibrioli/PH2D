//! **Outro programa a ocupar a placa** (F35, `docs/UI_New_and_Simple/F35_plano.md` §2): reproduz o
//! smoke de 26/10 (o `ollama` com 13,7 de 16 GB, o editor em branco) sem depender do `ollama`. Ocupa a
//! placa principal em blocos de 256 MiB, limpos na placa (residentes), até sobrar `<livre>` MiB segundo
//! o `nvidia-smi` (ou, numa AMD, o `mem_info_vram_*` do `amdgpu`), e segura-os `<segundos>` (prazo
//! obrigatório: nada de processo sem fim).
//!
//!     bash scripts/ph2d-run.sh cargo run --profile smoke -p ph2d-gpu --example ocupa_a_placa -- 1500 300

const BLOCO: u64 = 256 << 20;

/// A VRAM livre da placa AMD, pelo `amdgpu` (F38: a Radeon onboard não tem `nvidia-smi`).
fn livre_mib_amd() -> Option<u64> {
    std::fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .find_map(|e| {
            let d = e.path().join("device");
            let le = |f: &str| std::fs::read_to_string(d.join(f)).ok();
            (le("vendor")?.trim() == "0x1002").then_some(())?;
            let total: u64 = le("mem_info_vram_total")?.trim().parse().ok()?;
            let usado: u64 = le("mem_info_vram_used")?.trim().parse().ok()?;
            Some(total.saturating_sub(usado) >> 20)
        })
}

fn livre_mib_nvidia() -> Option<u64> {
    let out = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.free", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()?
        .trim()
        .parse()
        .ok()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let alvo: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1500);
    let segundos: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(300);
    let inst = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle_from_env()
    });
    ph2d_gpu::vez_da_placa();
    let adapter = pollster::block_on(inst.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("placa");
    let mut limits = wgpu::Limits::default();
    limits.max_buffer_size = limits.max_buffer_size.max(BLOCO);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ocupa_a_placa"),
        required_limits: limits,
        ..Default::default()
    }))
    .expect("dispositivo");
    println!("[ocupa] placa: {}", adapter.get_info().name);
    let livre_mib = if adapter.get_info().vendor == 0x1002 {
        livre_mib_amd
    } else {
        livre_mib_nvidia
    };
    let mut blocos = Vec::new();
    while livre_mib().is_some_and(|l| l > alvo + (BLOCO >> 20)) {
        let escopo = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ocupa_a_placa bloco"),
            size: BLOCO,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.clear_buffer(&b, 0, None);
        queue.submit([enc.finish()]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        if let Some(e) = pollster::block_on(escopo.pop()) {
            println!("[ocupa] a placa recusou mais um bloco: {e}");
            break;
        }
        blocos.push(b);
    }
    println!(
        "[ocupa] {} MiB ocupados, {:?} MiB livres; seguro {segundos} s",
        (blocos.len() as u64 * BLOCO) >> 20,
        livre_mib()
    );
    std::thread::sleep(std::time::Duration::from_secs(segundos));
}
