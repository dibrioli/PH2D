//! B42.1 — quantas camadas 3840×2160 o array de fatias aguenta NA PLACA, acima do orçamento
//! escolhido (`layer_cache_budget`). Sonda declarada (sem assert): a tabela vai ao BACKLOG.
//!
//!     bash scripts/ph2d-run.sh cargo test -p ph2d-render --test it a_camada_alem_do_orcamento -- --ignored --nocapture

use super::layer_compositor_gpu::{MapProvider, try_headless_gpu};
use ph2d_render::{LayerCompositor, LayerOp, Region};

fn tela(w: u32, h: u32, seed: u32) -> Vec<u8> {
    (0..w * h)
        .flat_map(|p| {
            let s = p.wrapping_mul(37).wrapping_add(seed.wrapping_mul(11));
            [s as u8, (s >> 3) as u8, (s >> 5) as u8, 255]
        })
        .collect()
}

#[test]
#[ignore = "measurement, not a gate"]
fn mede_as_camadas_4k_alem_do_orcamento() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (3840u32, 2160u32);
    let info = gpu.adapter.get_info();
    let base = LayerCompositor::new(&gpu).cache_budget_bytes();
    eprintln!(
        "\n{} ({:?}) orçamento {} MiB = {} fatias 4K",
        info.name,
        info.device_type,
        base >> 20,
        base / (u64::from(w) * u64::from(h) * 4)
    );
    eprintln!(
        "{:<7} {:>8} {:>12} {:>12}",
        "camadas", "MiB", "mín ms", "1.º ms"
    );
    let mut prov = MapProvider::default();
    for k in 0..64u32 {
        prov.insert(u64::from(k), 1, tela(w, h, k + 1));
    }
    for &n in &[16u32, 32, 33, 48, 64] {
        let mut comp = LayerCompositor::new(&gpu);
        comp.set_cache_budget_bytes(u64::MAX / 2);
        let ops: Vec<_> = (0..n)
            .map(|k| LayerOp::Layer {
                key: u64::from(k),
                blend_mode: 0,
                opacity: 1.0,
                mask: None,
                clipping: false,
            })
            .collect();
        let mib = (u64::from(w) * u64::from(h) * 4 * u64::from(n)) >> 20;
        let t0 = std::time::Instant::now();
        let (r, erro) = ph2d_gpu::memoria::sob_escopo_de_memoria(&gpu.device, || {
            comp.composite(&gpu, &ops, &prov, w, h, Region::full(w, h))
        });
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let primeiro = t0.elapsed().as_secs_f64() * 1e3;
        if let Err(e) = r {
            eprintln!("{n:<7} {mib:>8} recusa: {e}");
            continue;
        }
        if let Some(e) = erro {
            eprintln!("{n:<7} {mib:>8} sem memória: {e}");
            continue;
        }
        let mut min = f64::MAX;
        for _ in 0..7 {
            let t = std::time::Instant::now();
            comp.composite(&gpu, &ops, &prov, w, h, Region::full(w, h))
                .expect("composite");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            min = min.min(t.elapsed().as_secs_f64() * 1e3);
        }
        eprintln!("{n:<7} {mib:>8} {min:>12.2} {primeiro:>12.1}");
    }
}

/// B42.1 gate: a 33-layer 3840×2160 document stays on the GPU under the budget the compositor
/// resolves for EITHER kind of device, and (when a card is present) composites there for real.
#[test]
fn a_33a_camada_nao_manda_o_documento_a_cpu() {
    use ph2d_render::{layer_cache_budget, max_layers_for_budget};
    let (w, h) = (3840u32, 2160u32);
    for t in [
        wgpu::DeviceType::IntegratedGpu,
        wgpu::DeviceType::DiscreteGpu,
        wgpu::DeviceType::Cpu,
    ] {
        let cabe = max_layers_for_budget(w, h, layer_cache_budget(t));
        assert!(cabe >= 33, "{t:?}: o orçamento só segura {cabe} camadas 4K");
    }
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut comp = LayerCompositor::new(&gpu);
    let mut prov = MapProvider::default();
    let ops: Vec<_> = (0..33u32)
        .map(|k| {
            prov.insert(u64::from(k), 1, tela(w, h, k + 1));
            LayerOp::Layer {
                key: u64::from(k),
                blend_mode: 0,
                opacity: 1.0,
                mask: None,
                clipping: false,
            }
        })
        .collect();
    comp.composite(&gpu, &ops, &prov, w, h, Region::full(w, h))
        .expect("33 camadas 4K compõem na placa");
    assert_eq!(comp.cached_len(), 33);
}

/// B42.1: an array ask the card refuses (999 slices of 8192², ~250 GiB) comes back as a refusal
/// the caller can route to the CPU, and the budget recedes so the next frame does not ask again.
#[test]
fn o_array_recusado_recua_o_orcamento() {
    use ph2d_render::LayerCompositeError;
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut comp = LayerCompositor::new(&gpu);
    comp.set_cache_budget_bytes(u64::MAX / 2);
    let ops: Vec<_> = (0..999u32)
        .map(|k| LayerOp::Layer {
            key: u64::from(k),
            blend_mode: 0,
            opacity: 1.0,
            mask: None,
            clipping: false,
        })
        .collect();
    let s = 8192u32;
    let r = comp.composite(
        &gpu,
        &ops,
        &MapProvider::default(),
        s,
        s,
        Region::full(s, s),
    );
    assert!(
        matches!(r, Err(LayerCompositeError::TooManyLayers { requested: 999, cap }) if cap < 999),
        "{r:?}"
    );
    assert!(comp.cache_budget_bytes() < 999 * u64::from(s) * u64::from(s) * 4);
}

/// B18.4: o array que CRESCE (o «+» de uma camada) leva as fatias residentes na placa — só a camada nova
/// sobe da CPU, e a saída é a de um compositor novo com a pilha inteira, ao bit.
#[test]
fn o_array_que_cresce_guarda_as_fatias_ao_bit() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (64u32, 48u32);
    let fatia = u64::from(w) * u64::from(h) * 4;
    let mut prov = MapProvider::default();
    prov.insert(0, 1, tela(w, h, 1));
    prov.insert(1, 1, tela(w, h, 2));
    let op = |key, opacity| LayerOp::Layer {
        key,
        blend_mode: 0,
        opacity,
        mask: None,
        clipping: false,
    };
    let duas = [op(0, 1.0), op(1, 0.5)];
    let mut comp = LayerCompositor::new(&gpu);
    comp.composite(&gpu, &duas[..1], &prov, w, h, Region::full(w, h))
        .expect("uma camada");
    let antes = comp.bytes_enviados();
    comp.composite(&gpu, &duas, &prov, w, h, Region::full(w, h))
        .expect("duas camadas");
    assert_eq!(
        comp.bytes_enviados() - antes,
        fatia,
        "a base voltou a subir"
    );
    let mut novo = LayerCompositor::new(&gpu);
    novo.composite(&gpu, &duas, &prov, w, h, Region::full(w, h))
        .expect("oráculo");
    let saida = comp.read_output(&gpu).expect("saída");
    assert!(
        saida == novo.read_output(&gpu).expect("saída"),
        "a fatia levada ≠ a enviada"
    );
}
