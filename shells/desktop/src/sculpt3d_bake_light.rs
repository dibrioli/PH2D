//! **AS DUAS LUZES, sobre a MESMA forma** — a medição e o gate que ela produziu.
//!
//! Report do Enio no smoke da W8.6, com duas fotos e uma frase — *"não coincide perfeitamente"* —,
//! com as setas no **ARO** da esfera. Um aro é onde a normal deita, e é onde duas leis de luz podem
//! divergir sem que o miolo mude; então a pergunta não era *"quanto difere?"* e sim ***onde***
//! **difere**: um erro concentrado nos primeiros texels de dentro da silhueta tem causa diferente de
//! um espalhado pelo corpo, e as curas são opostas.
//!
//! ## O que a medição respondeu
//!
//! **O ARO ESTÁ INOCENTE.** Com o **mesmo albedo dos dois lados** — o controle sem o qual a tabela
//! não vale nada — a diferença é **0,0020 de média e 0,0049 de pico** no texel da borda, e ela é
//! **PLANA** ao longo da distância à silhueta (0,0020 · 0,0019 · 0,0019 · 0,0019 …). Não há aro: as
//! duas leis concordam a meio por cento onde o report apontava.
//!
//! **O que difere é o ALBEDO, e o mecanismo é o TETO.** O passe da tinta leva `m` até **1,65**
//! (`AMBIENT + (1 − AMBIENT) · 2`), e `light_pixel` clampa `albedo × m` em 1,0 — enquanto o barro
//! vivo escreve **HDR** (`Rgba16Float`, 28.913 canais acima de 1,0 nesta cena) e deixa o tonemap
//! fazer o roll-off. Um sprite é `unorm8` e não tem esse teto para gastar. Medido, dentro da
//! silhueta:
//!
//! | albedo do sprite | texels ESTOURADOS (a forma some ali) |
//! |---|---|
//! | branco `255` (a tela do smoke) | **43,6 %** |
//! | o barro `189` (`CLAY`) | **8,3 %** |
//!
//! ⚠️ **É por isso que o objeto assado lê como "chapado" e o vivo não**: sobre branco quase metade
//! da esfera vira `(255,255,255)` com gradiente ZERO. Não é a luz — é o albedo não ter alcance para
//! a luz caber.
//!
//! ⚠️ **E a medição derrubou uma afirmação MINHA**: o doc do [`crate::baked_form::planes::neutral_planes`] dizia que a
//! cobertura importa *"na borda da silhueta, onde o peso é parcial (o antialiasing do G-buffer)"*.
//! Medido, texels com `0 < w < 1`: **ZERO**. O G-buffer é `sample_count: 1` e o `fs_gbuffer` escreve
//! `w = 1.0` — o peso é **binário**, e não há antialiasing nenhum. O doc foi corrigido.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::bake::light_measure -- --ignored --nocapture
//! ```

use ph2d_gpu::GpuContext;
use ph2d_light::LightRig;
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_painter_brush::material::SpecLut;
use ph2d_render::ImpastoLightPass;

use crate::baked_form::planes::{
    BakePlanes, build_input, neutral_planes, resolved_lamps, upload_rgba,
};

/// O barro do `mesh.wgsl`. ⚠️ Cópia — se o `CLAY` de lá mudar, esta medição descreve outra coisa. Os
/// dois números do barro que TÊM gate são `CLAY_SHINE`/`CLAY_EXPONENT`.
const CLAY: [f32; 3] = [0.74, 0.70, 0.66];

/// Lado do quadro medido. Quadrado de propósito: a `measure_bake_framing` já mostrou que o aspecto
/// não muda a projeção, e um quadrado deixa a silhueta centrada nos dois eixos.
const SIDE: u32 = 512;

/// Baldes de distância à borda: `0, 1, 2, 3, 4-7, 8-15, 16-31, 32+`.
const NAMES: [&str; 8] = ["0", "1", "2", "3", "4-7", "8-15", "16-31", "32+"];

/// Até que balde a comparação ainda é *"o aro"* — o que o report apontava.
const RIM_BUCKETS: usize = 6;

fn bucket(d: u32) -> usize {
    match d {
        0..=3 => d as usize,
        4..=7 => 4,
        8..=15 => 5,
        16..=31 => 6,
        _ => 7,
    }
}

/// O que uma corrida mede.
struct Comparison {
    count: [u64; 8],
    mean_diff: [f64; 8],
    max_diff: [f32; 8],
    mean_live: [f64; 8],
    mean_bake: [f64; 8],
    /// Canais do barro vivo acima de 1,0 — o HDR que um sprite `unorm8` não carrega.
    over_one: u64,
    /// Texels do assado em `(255,255,255)`: onde a forma não sobrevive.
    blown: u64,
    /// Texels do G-buffer com peso PARCIAL. Se for zero, não há antialiasing a discutir.
    partial: u64,
    inside: u64,
}

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// Lê uma textura de volta, já sem o padding de linha.
fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32, bpt: u32) -> Vec<u8> {
    let bpr = (w * bpt).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sculpt3d light readback"),
        size: u64::from(bpr) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
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
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let mapped = slice.get_mapped_range();
    let row = (w * bpt) as usize;
    let mut out = vec![0u8; row * h as usize];
    for y in 0..h as usize {
        out[y * row..(y + 1) * row]
            .copy_from_slice(&mapped[y * bpr as usize..y * bpr as usize + row]);
    }
    drop(mapped);
    buffer.unmap();
    out
}

/// `f16` → `f32`, à mão. ⚠️ Não vale uma dependência nova para uma medição, e é **exato**: todo
/// valor de `f16` é representável em `f32`, e cada operação abaixo é uma potência de dois ou uma
/// divisão por 1024.
fn f16_to_f32(bits: u16) -> f32 {
    let sign = if bits & 0x8000 != 0 { -1.0f32 } else { 1.0 };
    let exp = i32::from((bits >> 10) & 0x1f);
    let man = f32::from(bits & 0x3ff);
    let v = if exp == 0 {
        man * 2f32.powi(-24)
    } else if exp == 31 {
        if man == 0.0 { f32::INFINITY } else { f32::NAN }
    } else {
        (1.0 + man / 1024.0) * 2f32.powi(exp - 15)
    };
    sign * v
}

/// O barro VIVO, num alvo `Rgba16Float` — o mesmo formato do `game_rt`, então nada é clampado aqui e
/// a saída pode passar de 1,0, que é justamente o que o modelo dele promete.
fn render_live(
    gpu: &GpuContext,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
    rig: &ph2d_light::ResolvedRig,
    size: (u32, u32),
) -> Vec<f32> {
    let (w, h) = size;
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sculpt3d light live"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // O `render` preserva o que está embaixo (`LoadOp::Load`), então o alvo é limpo antes.
    drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("sculpt3d light clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    }));
    renderer.render(
        &gpu.device,
        &gpu.queue,
        &mut enc,
        &view,
        camera,
        Some(rig),
        // ⚠️ **A vista PADRÃO, e é a premissa da medição.** Esta sonda compara a
        // luz da malha com a do passe de tinta; a cavidade é um termo que só a
        // primeira tem e o matcap é outra luz inteira — qualquer um dos dois
        // ligado aqui faria a comparação medir a diferença entre dois MODELOS em
        // vez da diferença entre duas implementações do mesmo.
        ph2d_mesh_render::Shade::default(),
        size,
    );
    gpu.queue.submit([enc.finish()]);

    let bytes = readback(gpu, &tex, w, h, 8);
    let mut out = vec![0f32; (w * h * 4) as usize];
    for (i, v) in out.iter_mut().enumerate() {
        *v = f16_to_f32(u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]));
    }
    out
}

/// Distância (em texels) de cada texel DENTRO da silhueta até a borda dela; `u32::MAX` fora. É esta
/// régua que separa *"um aro"* de *"o corpo inteiro"*, e sem ela a média de uma tela cheia esconde
/// exatamente a região que o report apontava.
fn depth_from_edge(form: &[f32], w: u32, h: u32) -> Vec<u32> {
    let (wi, hi) = (w as usize, h as usize);
    let inside = |x: usize, y: usize| form[(y * wi + x) * 4 + 3] > 0.0;
    let mut d = vec![u32::MAX; wi * hi];
    let mut queue: Vec<(usize, usize)> = Vec::new();
    for y in 0..hi {
        for x in 0..wi {
            if !inside(x, y) {
                continue;
            }
            let border = x == 0
                || y == 0
                || x + 1 == wi
                || y + 1 == hi
                || !inside(x - 1, y)
                || !inside(x + 1, y)
                || !inside(x, y - 1)
                || !inside(x, y + 1);
            if border {
                d[y * wi + x] = 0;
                queue.push((x, y));
            }
        }
    }
    let mut head = 0;
    while head < queue.len() {
        let (x, y) = queue[head];
        head += 1;
        let here = d[y * wi + x];
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx >= wi || ny >= hi || !inside(nx, ny) {
                continue;
            }
            if d[ny * wi + nx] > here + 1 {
                d[ny * wi + nx] = here + 1;
                queue.push((nx, ny));
            }
        }
    }
    d
}

/// **Uma corrida: o barro vivo contra o sprite assado, sobre a MESMA forma.**
///
/// ⚠️ O sprite recebe `base_rgb` como albedo, e é o chamador que escolhe se isso é o CONTROLE (o
/// albedo do próprio barro ⇒ o que sobra é a LEI) ou o PRODUTO (a tela branca do smoke).
fn compare(
    gpu: &GpuContext,
    renderer: &mut MeshRenderer,
    camera: &Camera3d,
    rig: &LightRig,
    base_rgb: [u8; 3],
) -> Comparison {
    let size = (SIDE, SIDE);
    let n = (SIDE * SIDE) as usize;
    let resolved = ph2d_light::resolve(rig).expect("o rig default tem lampada acesa");

    let live = render_live(gpu, renderer, camera, &resolved, size);
    let form = renderer
        .form_plane(&gpu.device, &gpu.queue, camera, size, ph2d_mesh_render::Shade::default(), None)
        .expect("a malha esta la'");

    let mut base = vec![0u8; n * 4];
    for px in base.chunks_exact_mut(4) {
        px.copy_from_slice(&[base_rgb[0], base_rgb[1], base_rgb[2], 255]);
    }
    let (relief, cover, mat0, mat1) = neutral_planes(&base);
    let planes = BakePlanes {
        relief,
        cover,
        mat0,
        mat1,
        lamps: resolved_lamps(&resolved),
    };
    let src = upload_rgba(gpu, size, &base);
    let mut pass = ImpastoLightPass::new(gpu);
    let input = build_input(size, &planes, &form.normal, &form.occlusion, SpecLut::get());
    let baked = {
        let out = pass.run(gpu, &src, &input).expect("o passe aceitou");
        readback(gpu, out, SIDE, SIDE, 4)
    };

    let depth = depth_from_edge(&form.normal, SIDE, SIDE);
    let mut c = Comparison {
        count: [0; 8],
        mean_diff: [0.0; 8],
        max_diff: [0.0; 8],
        mean_live: [0.0; 8],
        mean_bake: [0.0; 8],
        over_one: 0,
        blown: 0,
        partial: 0,
        inside: 0,
    };
    for i in 0..n {
        let w = form.normal[i * 4 + 3];
        if w > 0.0 && w < 1.0 {
            c.partial += 1;
        }
        if depth[i] == u32::MAX {
            continue;
        }
        c.inside += 1;
        if (0..3).all(|k| baked[i * 4 + k] == 255) {
            c.blown += 1;
        }
        let b = bucket(depth[i]);
        let (mut d, mut l_sum, mut b_sum) = (0f32, 0f32, 0f32);
        for k in 0..3 {
            let lv = live[i * 4 + k];
            let bv = f32::from(baked[i * 4 + k]) / 255.0;
            if lv > 1.0 {
                c.over_one += 1;
            }
            d = d.max((lv.clamp(0.0, 1.0) - bv).abs());
            l_sum += lv.clamp(0.0, 1.0);
            b_sum += bv;
        }
        c.count[b] += 1;
        c.mean_diff[b] += f64::from(d);
        c.max_diff[b] = c.max_diff[b].max(d);
        c.mean_live[b] += f64::from(l_sum / 3.0);
        c.mean_bake[b] += f64::from(b_sum / 3.0);
    }
    for b in 0..8 {
        if c.count[b] > 0 {
            let k = c.count[b] as f64;
            c.mean_diff[b] /= k;
            c.mean_live[b] /= k;
            c.mean_bake[b] /= k;
        }
    }
    c
}

/// A esfera do smoke `=11`, a câmera do escultor e o rig do artista.
fn stage(gpu: &GpuContext) -> (MeshRenderer, Camera3d, LightRig) {
    let mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let mut renderer = MeshRenderer::new(&gpu.device, wgpu::TextureFormat::Rgba16Float);
    renderer.upload_at(&gpu.device, &gpu.queue, 0, &mesh);
    let camera = Camera3d::framing(mesh.bounds(), core::f32::consts::FRAC_PI_4, 1.0);
    (renderer, camera, LightRig::default())
}

/// **ONDE as duas luzes divergem** — a tabela que decidiu que o report não era o aro.
#[test]
#[ignore = "precisa de adapter"]
fn measure_the_two_lights_over_the_same_form() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let clay = [
        (CLAY[0] * 255.0 + 0.5) as u8,
        (CLAY[1] * 255.0 + 0.5) as u8,
        (CLAY[2] * 255.0 + 0.5) as u8,
    ];

    // O CONTROLE primeiro — mesmo albedo dos dois lados ⇒ o que sobra é a LEI.
    let c = compare(&gpu, &mut renderer, &camera, &rig, clay);
    println!("\n=== CONTROLE: o sprite veste o albedo do barro {clay:?} ===");
    println!("  dist |    texels |  |dif| medio |   |dif| max |  vivo  | assado");
    for (b, name) in NAMES.iter().enumerate() {
        if c.count[b] == 0 {
            continue;
        }
        println!(
            "  {:>4} | {:>9} | {:>11.4} | {:>11.4} | {:>6.4} | {:>6.4}",
            name, c.count[b], c.mean_diff[b], c.max_diff[b], c.mean_live[b], c.mean_bake[b]
        );
    }
    println!(
        "  canais do barro VIVO acima de 1,0 (o HDR que um sprite unorm8 nao carrega): {}",
        c.over_one
    );
    println!(
        "  e com ESTE albedo o assado ja' estoura {} de {} texels ({:.1}%) -- o barro e' TINGIDO \
         (189/179/168), entao o azul satura depois; um cinza 189 chapado estoura 19,5%",
        c.blown,
        c.inside,
        100.0 * c.blown as f64 / c.inside.max(1) as f64
    );
    println!(
        "  texels do G-buffer com peso PARCIAL (0 < w < 1): {} -- se for ZERO, nao ha' \
         antialiasing e a cobertura NAO age na borda",
        c.partial
    );

    // E o TETO, que é o que o artista de fato vê: quanto de cada albedo sobrevive.
    println!("\n=== O TETO: quanto da forma sobra, por albedo do sprite ===");
    println!("  albedo | estourados | % da silhueta | luma medio do assado");
    for v in [255u8, 224, 192, clay[0], 160, 128] {
        let k = compare(&gpu, &mut renderer, &camera, &rig, [v, v, v]);
        let inside = k.inside.max(1) as f64;
        let luma: f64 = (0..8)
            .map(|b| k.mean_bake[b] * k.count[b] as f64)
            .sum::<f64>()
            / inside;
        println!(
            "  {v:>6} | {:>10} | {:>12.1}% | {luma:>19.4}",
            k.blown,
            100.0 * k.blown as f64 / inside
        );
    }
    println!(
        "  (o `m` do passe da tinta chega a 1,65, entao todo albedo acima de ~0,61 satura em algum\n   \
         ponto do lado iluminado -- e onde satura a FORMA some: gradiente zero)"
    );
}

/// **O ARO ESTÁ INOCENTE — as duas luzes concordam onde a forma vira.**
///
/// Este gate é a refutação do report da W8.6 tornada executável: com o **mesmo albedo dos dois
/// lados**, a diferença nos primeiros 15 texels de dentro da silhueta é ruído (medido: **0,0020 de
/// média, 0,0049 de pico**; as barras abaixo têm ~4× de folga). O que o artista viu era o albedo, e
/// está medido em [`measure_the_two_lights_over_the_same_form`].
///
/// ⚠️ **O que ele protege não é a LEI, é a TRADUÇÃO** — os dois lados chamam o mesmo
/// `ph2d_light::resolve` e o mesmo `canvas_normal`, então um defeito no corpo compartilhado moveria
/// os dois juntos e este gate ficaria verde ([[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]]).
/// O que ele vê é o bake traduzir o rig e a forma para o vocabulário da tinta. Três mutações,
/// rodadas:
///
/// | mutação | média do aro | veredito |
/// |---|---|---|
/// | [`crate::baked_form::planes::resolved_lamps`] larga a `dir` | **0,3002** | sangra (30× a barra) |
/// | [`crate::baked_form::planes::build_input`] manda `form: None` | **0,3514** | sangra |
/// | [`crate::baked_form::planes::clay_material`] volta ao `Material::NEUTRAL` | 0,0020 | **VERDE, e está certo** |
///
/// ⚠️ **A terceira é o desenho, não um buraco**: o realce vive no MIOLO (no aro o `ndh` rasante dá
/// especular ≈ 0), então tirá-lo não move o aro um milésimo. Quem sangra por ela é o irmão
/// `planes::tests::a_baked_object_wears_the_clays_highlight`, que afirma o número direto — e essa
/// divisão de trabalho foi **medida**, não suposta.
///
/// ⚠️ **A barra é sobre o ARO e só sobre ele.** No miolo as duas leis divergem de verdade e por
/// desenho — o barro SOMA o realce em HDR e a tinta o faz por `screen` em `unorm8` —, e afirmar
/// acordo lá seria pinar um número que nunca foi prometido.
#[test]
#[ignore = "precisa de adapter"]
fn the_two_lights_agree_where_the_form_turns_away() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a afirmar");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let clay = [
        (CLAY[0] * 255.0 + 0.5) as u8,
        (CLAY[1] * 255.0 + 0.5) as u8,
        (CLAY[2] * 255.0 + 0.5) as u8,
    ];
    let c = compare(&gpu, &mut renderer, &camera, &rig, clay);
    assert!(
        c.count[0] > 300,
        "premissa: a silhueta tem borda para medir (achei {} texels em d=0)",
        c.count[0]
    );
    for (b, name) in NAMES.iter().enumerate().take(RIM_BUCKETS) {
        assert!(
            c.mean_diff[b] < 0.01,
            "o aro divergiu no balde {name}: media {:.4} (medido 0,0020)",
            c.mean_diff[b]
        );
        assert!(
            c.max_diff[b] < 0.02,
            "o aro divergiu no balde {name}: pico {:.4} (medido 0,0049)",
            c.max_diff[b]
        );
    }
    // E a metade que impede a leitura preguiçosa da barra: ela é PLANA na distância. Um aro seria
    // uma borda que difere e um miolo que não, e é exatamente isso que o report descrevia.
    let edge = c.mean_diff[0];
    let deep = c.mean_diff[RIM_BUCKETS - 1];
    assert!(
        (edge - deep).abs() < 0.005,
        "a diferenca nao e' plana na distancia: borda {edge:.4} contra {deep:.4} -- isso SERIA um aro"
    );
}

/// **O que a forma custa GUARDADA** — módulo filho, para `use super::*` alcançar o harness
/// (o `gpu`, o `stage`, o `readback`, o `SIDE`). O corte é por assunto: aqui a pergunta não é *as duas
/// luzes concordam?* e sim *a luz sobrevive à viagem por 8 bits?*.
#[path = "sculpt3d_bake_form_bytes.rs"]
mod form_bytes;
