//! **A LENTE NA PLACA = A LENTE DA CPU** (doc 129 §4, W1) — o `LensPass` contra
//! `ph2d_lens::aplica_cpu`, pixel a pixel, em cada efeito sozinho e nos quatro juntos.
//!
//! A cena é a do oráculo (gradiente + discos + barra branca) num SUB-RECT de um alvo maior, como a
//! cena no split do editor: o centro da lente é o do sub-rect, a borda replicada é a dele, e o que
//! fica fora não muda um byte. O alvo é o do acumulador do mundo (`Bgra8Unorm`, sRGB
//! pré-multiplicado). A régua é de 8 bits: o passe escreve num alvo de 8 bits, e a lei cumpre-se
//! se nenhum canal se afastar mais de **1 nível** do da CPU. O hash do grão, esse, é exacto AO BIT
//! (a sonda de compute abaixo).
//!
//! Rodar: `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-render --test it lens_gpu --
//! --ignored`.

use ph2d_color::srgb::{linear_to_srgb_unit, srgb_to_linear_unit};
use ph2d_gpu::GpuContext;
use ph2d_lens::{Aberracao, Cubo, Grade, Grao, Imagem, Lente, Vinheta, aplica_cpu, grao};
use ph2d_render::{LensPass, WorldRt};

use crate::fx_stack_common::{readback, try_headless_gpu};

const ALVO: (u32, u32) = (128, 88);
const RECT: [u32; 4] = [20, 9, 96, 64];
/// O que está FORA da cena (o chrome do split): não pode mudar.
const FORA: [u8; 4] = [30, 200, 10, 255];

fn neutra() -> Lente {
    Lente {
        aberracao: Aberracao {
            amount: 0.0,
            linear: false,
            angle: 0.0,
        },
        vinheta: Vinheta {
            amount: 0.0,
            midpoint: 1.0,
            roundness: 0.0,
            feather: 0.5,
            center: [0.0; 2],
            cor: [0.0; 3],
        },
        grade: Grade {
            exposure: 0.0,
            temperatura: 0.0,
            matiz: 0.0,
            contraste: 0.0,
            saturacao: 0.0,
            lut_mix: 0.0,
        },
        grao: Grao {
            amount: 0.0,
            size: 1.0,
            response: 0.0,
            colorido: false,
            animado: false,
            semente: 9,
        },
    }
}

/// A cena em sRGB de 8 bits (RGBA, alfa cheio): gradiente, dois discos, barra branca.
fn cena() -> Vec<[u8; 4]> {
    let (w, h) = (RECT[2] as f32, RECT[3] as f32);
    let mut px = Vec::new();
    for y in 0..RECT[3] {
        for x in 0..RECT[2] {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut c = [fx / w, fy / h, 0.5 * (1.0 - fx / w)];
            if (fx - 0.3 * w).hypot(fy - 0.4 * h) < 0.18 * h {
                c = [0.95, 0.25, 0.1];
            }
            if (fx - 0.7 * w).hypot(fy - 0.6 * h) < 0.22 * h {
                c = [0.1, 0.35, 0.9];
            }
            if (0.45 * w..0.5 * w).contains(&fx) {
                c = [1.0; 3];
            }
            px.push([
                (c[0] * 255.0).round() as u8,
                (c[1] * 255.0).round() as u8,
                (c[2] * 255.0).round() as u8,
                255,
            ]);
        }
    }
    px
}

/// Uma LUT 9³ com mistura entre canais (a forma da `cubo.py` do oráculo).
fn cubo() -> Cubo {
    let n = 9;
    let mut dados = Vec::new();
    for b in 0..n {
        for g in 0..n {
            for r in 0..n {
                let (r, g, b) = (
                    r as f32 / (n - 1) as f32,
                    g as f32 / (n - 1) as f32,
                    b as f32 / (n - 1) as f32,
                );
                dados.push([
                    0.8 * r + 0.2 * g * g,
                    0.7 * g + 0.3 * b.sqrt(),
                    0.6 * b + 0.4 * r * g,
                ]);
            }
        }
    }
    Cubo {
        n,
        dados,
        dominio_min: [0.0; 3],
        dominio_max: [1.0; 3],
    }
}

/// O alvo com a cena no sub-rect e [`FORA`] à volta, em bytes BGRA (a ordem do `Bgra8Unorm`).
fn alvo(gpu: &GpuContext) -> (wgpu::Texture, wgpu::TextureView) {
    let src = cena();
    let mut bytes = Vec::with_capacity((ALVO.0 * ALVO.1 * 4) as usize);
    for y in 0..ALVO.1 {
        for x in 0..ALVO.0 {
            let dentro = (RECT[0]..RECT[0] + RECT[2]).contains(&x)
                && (RECT[1]..RECT[1] + RECT[3]).contains(&y);
            let c = if dentro {
                src[((y - RECT[1]) * RECT[2] + (x - RECT[0])) as usize]
            } else {
                FORA
            };
            bytes.extend_from_slice(&[c[2], c[1], c[0], c[3]]);
        }
    }
    let extent = wgpu::Extent3d {
        width: ALVO.0,
        height: ALVO.1,
        depth_or_array_layers: 1,
    };
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test lens alvo"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: WorldRt::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(ALVO.0 * 4),
            rows_per_image: Some(ALVO.1),
        },
        extent,
    );
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    (tex, view)
}

/// A referência: a cena descodificada, a lente da CPU, codificada e arredondada a 8 bits.
fn referencia(l: &Lente, lut: Option<&Cubo>, tique: u32) -> Vec<[u8; 3]> {
    let img = Imagem {
        w: RECT[2],
        h: RECT[3],
        px: cena()
            .iter()
            .map(|c| {
                let d = |v: u8| srgb_to_linear_unit(f32::from(v) / 255.0);
                [d(c[0]), d(c[1]), d(c[2]), 1.0]
            })
            .collect(),
    };
    aplica_cpu(&img, l, lut, tique)
        .px
        .iter()
        .map(|c| [0, 1, 2].map(|i| (linear_to_srgb_unit(c[i]) * 255.0).round() as u8))
        .collect()
}

/// Corre a lente na placa e compara: devolve `(maior desvio, quantos canais diferem)`; reprova se
/// um byte FORA do sub-rect mudou.
fn compara(gpu: &GpuContext, l: &Lente, lut: Option<&Cubo>, tique: u32) -> (u8, usize) {
    let mut pass = LensPass::new(gpu, WorldRt::FORMAT);
    pass.carrega_lut(gpu, lut);
    let (tex, view) = alvo(gpu);
    assert!(
        pass.aplica(gpu, &tex, &view, RECT, l, tique),
        "a lente não correu"
    );
    let got = readback(gpu, &tex, ALVO.0, ALVO.1);
    let want = referencia(l, lut, tique);
    let (mut pior, mut quantos) = (0u8, 0usize);
    for y in 0..ALVO.1 {
        for x in 0..ALVO.0 {
            let i = ((y * ALVO.0 + x) * 4) as usize;
            let rgba = [got[i + 2], got[i + 1], got[i], got[i + 3]];
            let dentro = (RECT[0]..RECT[0] + RECT[2]).contains(&x)
                && (RECT[1]..RECT[1] + RECT[3]).contains(&y);
            if !dentro {
                assert_eq!(rgba, FORA, "a lente escreveu FORA da cena em ({x}, {y})");
                continue;
            }
            assert_eq!(rgba[3], 255, "o alfa mudou em ({x}, {y})");
            let w = want[((y - RECT[1]) * RECT[2] + (x - RECT[0])) as usize];
            for c in 0..3 {
                let d = rgba[c].abs_diff(w[c]);
                pior = pior.max(d);
                quantos += usize::from(d > 0);
            }
        }
    }
    (pior, quantos)
}

fn afirma(nome: &str, gpu: &GpuContext, l: &Lente, lut: Option<&Cubo>, tique: u32) {
    let (pior, quantos) = compara(gpu, l, lut, tique);
    eprintln!(
        "[lens_gpu] {nome}: desvio máximo {pior} nível(is), {quantos} de {} canais diferem",
        RECT[2] * RECT[3] * 3
    );
    assert!(pior <= 1, "{nome}: a placa afasta-se {pior} níveis da CPU");
}

/// ⭐ **Cada efeito sozinho**, e os dois modos da aberração e os dois sinais da vinheta.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn each_effect_alone_matches_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[lens_gpu] sem adapter — skip");
        return;
    };
    let mut l = neutra();
    l.aberracao.amount = 0.03;
    afirma("aberração radial", &gpu, &l, None, 0);

    let mut l = neutra();
    l.aberracao = Aberracao {
        amount: 0.02,
        linear: true,
        angle: 0.125,
    };
    afirma("aberração linear", &gpu, &l, None, 0);

    let mut l = neutra();
    l.vinheta = Vinheta {
        amount: 0.6,
        midpoint: 0.8,
        roundness: 0.3,
        feather: 0.4,
        center: [0.1, -0.05],
        cor: [0.05, 0.02, 0.1],
    };
    afirma("vinheta escura", &gpu, &l, None, 0);
    l.vinheta.amount = -0.5;
    l.vinheta.roundness = -0.5;
    afirma("vinheta clara, quase retângulo", &gpu, &l, None, 0);

    let mut l = neutra();
    l.grade = Grade {
        exposure: 0.4,
        temperatura: 0.5,
        matiz: -0.3,
        contraste: 0.3,
        saturacao: 0.4,
        lut_mix: 0.0,
    };
    afirma("grade sem LUT", &gpu, &l, None, 0);

    let mut l = neutra();
    l.grade.lut_mix = 0.7;
    afirma("só a LUT", &gpu, &l, Some(&cubo()), 0);

    let mut l = neutra();
    l.grao = Grao {
        amount: 0.4,
        size: 1.5,
        response: 0.6,
        colorido: false,
        animado: true,
        semente: 9,
    };
    afirma("grão mono", &gpu, &l, None, 3);
    l.grao.colorido = true;
    l.grao.size = 1.0;
    afirma("grão colorido", &gpu, &l, None, 4);
}

/// ⭐ **Os quatro juntos**, na ordem fixa, com a LUT.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_four_together_match_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[lens_gpu] sem adapter — skip");
        return;
    };
    let l = Lente {
        aberracao: Aberracao {
            amount: 0.025,
            linear: false,
            angle: 0.0,
        },
        vinheta: Vinheta {
            amount: 0.5,
            midpoint: 0.9,
            roundness: 0.0,
            feather: 0.5,
            center: [0.0; 2],
            cor: [0.0; 3],
        },
        grade: Grade {
            exposure: 0.2,
            temperatura: 0.3,
            matiz: 0.1,
            contraste: 0.2,
            saturacao: 0.25,
            lut_mix: 0.5,
        },
        grao: Grao {
            amount: 0.25,
            size: 2.0,
            response: 0.5,
            colorido: true,
            animado: true,
            semente: 77,
        },
    };
    afirma("os quatro", &gpu, &l, Some(&cubo()), 12);
}

/// ⭐ **Lente neutra ⇒ o alvo fica BYTE-IDÊNTICO** — o passe nem corre, mesmo com uma LUT
/// carregada e `lut_mix = 0`.
/// (Mutação: `LensUniform::de` sem o `neutra` ⇒ o passe corre e reescreve cada texel — RED pelo
/// `aplica` devolver `true`, e pelos bytes se a ida-e-volta sRGB não for exacta.)
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_neutral_lens_leaves_the_target_byte_identical() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[lens_gpu] sem adapter — skip");
        return;
    };
    let mut pass = LensPass::new(&gpu, WorldRt::FORMAT);
    pass.carrega_lut(&gpu, Some(&cubo()));
    let (tex, view) = alvo(&gpu);
    let antes = readback(&gpu, &tex, ALVO.0, ALVO.1);
    assert!(!pass.aplica(&gpu, &tex, &view, RECT, &neutra(), 5));
    assert_eq!(readback(&gpu, &tex, ALVO.0, ALVO.1), antes);
}

/// ⭐ **O hash do grão é o MESMO bit nas duas línguas** — o `pcg3d` (só inteiros de 32 bits) e a
/// gaussiana de célula que sai dele, por uma sonda de compute sobre a MESMA biblioteca do passe,
/// em células negativas e positivas, três canais.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_grain_hash_is_the_same_bit_on_the_gpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[lens_gpu] sem adapter — skip");
        return;
    };
    const N: u32 = 256;
    let src = format!(
        "{}\n@group(0) @binding(0) var<storage, read_write> saida: array<vec4<u32>>;
@compute @workgroup_size(64)
fn sonda(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= {N}u) {{ return; }}
    let cel = vec2<i32>(i32(i % 16u) - 8, i32(i / 16u) - 8);
    let semente = 4000000000u + i * 7919u;
    let canal = i % 3u;
    let h = lz_pcg3d(vec3<u32>(bitcast<u32>(cel.x), bitcast<u32>(cel.y), semente));
    saida[i] = vec4<u32>(h, bitcast<u32>(lz_gauss_celula(cel, semente, canal)));
}}",
        ph2d_render::lens::LENS_LIB_WGSL
    );
    let device = &gpu.device;
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("test lens sonda do hash"),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("test lens sonda do hash"),
        layout: None,
        module: &modulo,
        entry_point: Some("sonda"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let tamanho = u64::from(N) * 16;
    let saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test lens saida"),
        size: tamanho,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test lens leitura"),
        size: tamanho,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: saida.as_entire_binding(),
        }],
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.dispatch_workgroups(N.div_ceil(64), 1, 1);
    }
    enc.copy_buffer_to_buffer(&saida, 0, &leitura, 0, tamanho);
    gpu.queue.submit([enc.finish()]);
    let slice = leitura.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let got: Vec<[u32; 4]> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    for i in 0..N {
        let cel = [(i % 16) as i32 - 8, (i / 16) as i32 - 8];
        let semente = 4_000_000_000u32.wrapping_add(i.wrapping_mul(7919));
        let h = grao::pcg3d([cel[0] as u32, cel[1] as u32, semente]);
        let g = grao::gauss_celula(cel, semente, i % 3);
        assert_eq!(
            got[i as usize],
            [h[0], h[1], h[2], g.to_bits()],
            "célula {cel:?}: a placa e a CPU dão hashes diferentes"
        );
    }
}
