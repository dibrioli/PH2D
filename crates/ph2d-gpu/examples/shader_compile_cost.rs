//! ⭐⭐⭐ **QUANTO CUSTA COMPILAR UM SHADER DESTE TAMANHO** — a medição que escolhe entre as duas
//! rotas de levar um campo implícito ao dispositivo (`docs/Render3d/05` §33).
//!
//! | rota | catálogo | scratch por thread | compilação |
//! |---|---|---|---|
//! | **interpretar a FITA** | as `62` primitivas de graça (já estão lowered em aritmética) | ⛔ **`464` valores vivos na pior cena** = `116 KB` por workgroup — **não cabe** | nenhuma |
//! | **gerar WGSL** | idem, e o compilador optimiza | nenhum (o compilador aloca registos) | ⚠️ **é isto que esta sonda mede** |
//!
//! ⚠️ **E a compilação paga-se por ESTRUTURA, não por edição.** Arrastar um slider muda um NÚMERO;
//! a árvore fica igual. Se os números viajarem num uniforme, um arrasto **não recompila nada** — e
//! a pergunta passa a ser só *«quanto custa a primeira vez, e quando se acrescenta uma forma?»*.
//!
//! ```text
//! cargo run --release -p ph2d-gpu --example shader_compile_cost
//! ```

/// Um shader com `n` operações encadeadas — a forma de uma fita de campo achatada.
fn tape_of(n: usize) -> String {
    let mut s = String::from(
        "@group(0) @binding(0) var<storage, read_write> out: array<f32>;\n\
         fn field(p: vec3<f32>) -> f32 {\n  var v0 = p.x;\n  var v1 = p.y;\n  var v2 = p.z;\n",
    );
    for i in 3..n {
        let (a, b) = (i - 1, i - 2);
        // A mistura de operações que uma fita de SDF de facto tem.
        match i % 6 {
            0 => s.push_str(&format!("  var v{i} = v{a} + v{b};\n")),
            1 => s.push_str(&format!("  var v{i} = v{a} * 0.5 - v{b};\n")),
            2 => s.push_str(&format!("  var v{i} = min(v{a}, v{b});\n")),
            3 => s.push_str(&format!("  var v{i} = max(v{a}, v{b});\n")),
            4 => s.push_str(&format!("  var v{i} = sqrt(abs(v{a}) + 1e-6) + v{b};\n")),
            _ => s.push_str(&format!("  var v{i} = v{a} - abs(v{b});\n")),
        }
    }
    s.push_str(&format!("  return v{};\n}}\n", n - 1));
    s.push_str(
        "@compute @workgroup_size(64)\n\
         fn main(@builtin(global_invocation_id) g: vec3<u32>) {\n\
         \x20 out[g.x] = field(vec3<f32>(f32(g.x), 1.0, 2.0));\n}\n",
    );
    s
}

fn main() {
    let instance = ph2d_gpu::GpuContext::default_instance();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) else {
        println!("sem adaptador de GPU — nada a medir");
        return;
    };
    println!("GPU: {}", adapter.get_info().name);
    let (device, _queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("compile cost"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("device");

    println!("\n   ops · WGSL · naga (módulo) · pipeline · TOTAL · (a cena real com este tamanho)");
    for &n in &[50usize, 150, 450, 900, 3000] {
        let src = tape_of(n);
        let t = std::time::Instant::now();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tape"),
            source: wgpu::ShaderSource::Wgsl(src.as_str().into()),
        });
        let modulo_ms = t.elapsed().as_secs_f64() * 1e3;
        let t = std::time::Instant::now();
        let _p = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("tape"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
        let pipeline_ms = t.elapsed().as_secs_f64() * 1e3;
        let cena = match n {
            50 => "a ponte (49)",
            150 => "um verbo por forma (126)",
            450 => "o lote da W103 (451)",
            900 => "o polígono de N (851)",
            _ => "a PIOR medida (2 969)",
        };
        println!(
            "  {n:5} · {:4} KB · {modulo_ms:11.1} ms · {pipeline_ms:6.1} ms · {:5.1} ms · {cena}",
            src.len() / 1024,
            modulo_ms + pipeline_ms
        );
    }
}
