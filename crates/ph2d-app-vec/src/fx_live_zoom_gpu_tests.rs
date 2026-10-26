//! **G5 (placa) — a paridade do recorte**: cada pixel VISÍVEL do job recortado à vista é o pixel do
//! job inteiro, para os 15 tipos. A fonte é um disco analítico no espaço de tela (o raster do Vello é
//! equivariante a deslocamentos inteiros; o que se prova aqui é a PILHA sobre a janela). Mais a
//! régua de ms da placa do plano §1.

use super::{
    BORDA, CENTRO, Cena, R, SEM_RECORTE, VIEW, ZOOMS, camera, cena, cena_op, inter, kinds,
};
use ph2d_ecs::FxOp;
use ph2d_render::{FxStackPass, Origem};
use ph2d_vector::Affine;

fn gpu() -> Option<ph2d_gpu::GpuContext> {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
}

fn tex(gpu: &ph2d_gpu::GpuContext, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("F34 paridade"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// O disco de raio `R·zoom` centrado em `centro` (tela), premultiplicado, na janela `[ex0, ey0]+w×h`.
fn disco(w: u32, h: u32, ex0: f64, ey0: f64, centro: [f64; 2], raio: f64) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (ex0 + f64::from(x) + 0.5, ey0 + f64::from(y) + 0.5);
            let d = (sx - centro[0]).hypot(sy - centro[1]);
            let a = (raio - d + 0.5).clamp(0.0, 1.0);
            let o = ((y * w + x) * 4) as usize;
            let c = |v: f64| (v * a).round() as u8;
            out[o..o + 4].copy_from_slice(&[c(235.0), c(175.0), c(60.0), c(255.0)]);
        }
    }
    out
}

fn upload(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture, w: u32, h: u32, bytes: &[u8]) {
    gpu.queue.write_texture(
        t.as_image_copy(),
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

fn readback(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let row = (w * 4).div_ceil(256) * 256;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("F34 readback"),
        size: u64::from(row * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
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
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let data = buf.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let o = (y * row) as usize;
        out.extend_from_slice(&data[o..o + (w * 4) as usize]);
    }
    out
}

/// Corre a pilha do job sobre o disco, como o `cook_batch` (segmentos recortados, origem da forma).
fn corre(
    gpu: &ph2d_gpu::GpuContext,
    pass: &mut FxStackPass,
    c: &Cena,
    cam: Affine,
    job: &super::Job,
    centro: [f64; 2],
    raio: f64,
) -> Vec<u8> {
    let (w, h) = (job.key.w, job.key.h);
    let src = tex(gpu, w, h);
    upload(
        gpu,
        &src,
        w,
        h,
        &disco(w, h, job.ex0, job.ey0, centro, raio),
    );
    let dst = tex(gpu, w, h);
    let alc = ph2d_render::stack_read(&job.key.ops) as f32;
    let geom = c.segs(cam, job, [-alc, -alc, w as f32 + alc, h as f32 + alc]);
    let origem = Origem::Ponto(job.ruido);
    pass.run_from_com_origem(gpu, &src, [0, 0], &dst, w, h, &job.key.ops, &geom, origem);
    readback(gpu, &dst, w, h)
}

#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn every_visible_pixel_of_a_clipped_job_is_the_pixel_of_the_whole_job() {
    let Some(gpu) = gpu() else {
        eprintln!("[fx-zoom-paridade] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let (z, foco) = (16.0, BORDA);
    let cam = camera(z, foco, 0.0);
    let centro = [VIEW[2] / 2.0 - (foco[0] * z), VIEW[3] / 2.0 - (foco[1] * z)];
    // A vista, e dois recortes só à esquerda que deslocam a janela por um número PAR e um ÍMPAR de
    // texels — o `round` do WGSL levava o meio ao par, e só o ímpar o mostrava (Morphology, 6/255).
    let janelas = [
        VIEW,
        [-1000.0, -1.0e6, 1.0e6, 1.0e6],
        [-1001.0, -1.0e6, 1.0e6, 1.0e6],
    ];
    let mut piores = Vec::new();
    // Os 15 tipos de omissão, e a Morphology LARGA nos dois sentidos (16 px, semeada pelo raster). A
    // cadeia de saltos do JFA NÃO pede margem além da banda: medido (F34), alargá-la não muda um pixel.
    let larga = |grow: f32| {
        let mut op = FxOp::new(FxOp::MORPHOLOGY);
        op.grow = grow;
        op
    };
    let casos = kinds().map(FxOp::new).chain([larga(1.0), larga(-1.0)]);
    for op in casos {
        let kind = op.kind;
        let c = cena_op(op);
        let inteiro = c.job(cam, SEM_RECORTE).expect("job");
        let a = corre(&gpu, &mut pass, &c, cam, &inteiro, centro, R * z);
        for vista in janelas {
            let recortado = c.job(cam, vista).expect("job");
            assert!(
                recortado.key.w < inteiro.key.w,
                "tipo {kind}: o job devia recortar"
            );
            let b = corre(&gpu, &mut pass, &c, cam, &recortado, centro, R * z);
            let vis = inter(inter(super::rect(&inteiro), super::rect(&recortado)), VIEW);
            let texel = |job: &super::Job, sx: i64, sy: i64| {
                let (x, y) = (sx - job.ex0 as i64, sy - job.ey0 as i64);
                (y * i64::from(job.key.w) + x) as usize * 4
            };
            let mut pior = 0u8;
            for sy in vis[1] as i64..vis[3] as i64 {
                for sx in vis[0] as i64..vis[2] as i64 {
                    let (pa, pb) = (texel(&inteiro, sx, sy), texel(&recortado, sx, sy));
                    for k in 0..4 {
                        pior = pior.max(a[pa + k].abs_diff(b[pb + k]));
                    }
                }
            }
            eprintln!(
                "[fx-zoom-paridade] tipo {kind:>2}: inteiro {}x{} · recortado {}x{} · pior diferença {pior}",
                inteiro.key.w, inteiro.key.h, recortado.key.w, recortado.key.h
            );
            piores.push((kind, recortado.ex0, pior));
        }
    }
    // ±1 é a precisão `f32` das coordenadas LOCAIS (as do recorte são menores, logo mais finas).
    let maus: Vec<_> = piores.iter().filter(|(_, _, p)| *p > 1).collect();
    assert!(
        maus.is_empty(),
        "pixels visíveis diferem do job inteiro: {maus:?}"
    );
}

/// **Os ms da pilha na placa**, por tipo e zoom, no tamanho que o job pede — a régua do plano §1.
#[test]
#[ignore = "needs a real GPU device; sonda F34"]
fn fx_zoom_regua_ms_na_placa() {
    let Some(gpu) = gpu() else {
        eprintln!("[fx-zoom] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    for (nome, foco) in [("centro", CENTRO), ("borda", BORDA)] {
        for kind in kinds() {
            let c = cena(kind);
            for z in ZOOMS {
                let cam = camera(z, foco, 0.0);
                let Some(job) = c.job(cam, VIEW) else {
                    continue;
                };
                let (w, h) = (job.key.w, job.key.h);
                let src = tex(&gpu, w, h);
                let dst = tex(&gpu, w, h);
                let alc = ph2d_render::stack_read(&job.key.ops) as f32;
                let geom = c.segs(cam, &job, [-alc, -alc, w as f32 + alc, h as f32 + alc]);
                let run = |pass: &mut FxStackPass| {
                    let o = Origem::Ponto(job.ruido);
                    pass.run_from_com_origem(
                        &gpu,
                        &src,
                        [0, 0],
                        &dst,
                        w,
                        h,
                        &job.key.ops,
                        &geom,
                        o,
                    );
                    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
                };
                run(&mut pass);
                let mut best = f64::INFINITY;
                for _ in 0..3 {
                    let t = std::time::Instant::now();
                    run(&mut pass);
                    best = best.min(t.elapsed().as_secs_f64() * 1000.0);
                }
                eprintln!(
                    "[fx-zoom-ms] foco {nome:>6} kind {kind:>2} zoom {z:>4}× {w}x{h} segs {:>4} {best:>8.2} ms",
                    geom.len()
                );
            }
        }
    }
}

/// **A sonda do piscar** (smoke F34 do dono: *«com o máximo de zoom ainda pisca»*). Repete o padrão de
/// registo do `FxLive` no renderer principal — textura nova quando o TAMANHO muda, a velha desregistada
/// no quadro seguinte — e conta os quadros em que a imagem NÃO chega à tela. `passo` = quanto o lado
/// cresce por quadro (0 = tamanho fixo, o controlo).
fn quadros_sem_imagem(
    gpu: &ph2d_gpu::GpuContext,
    lado: (u32, u32),
    passo: u32,
    quadros: u32,
    degrau: u32,
) -> u32 {
    use ph2d_render::VelloPass;
    use ph2d_vector::{ImageQuality, StableImage, VectorScene};
    let tela = (1930u32, 1040u32);
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let mut principal = VelloPass::new(gpu, fmt, tela).expect("vello");
    let mut atual: Option<(ph2d_vector::ImageData, (u32, u32))> = None;
    let mut pendentes = Vec::new();
    let mut faltas = 0;
    for q in 0..quadros {
        for img in pendentes.drain(..) {
            principal.unregister_texture(img);
        }
        let sobe = |v: u32| v.div_ceil(degrau) * degrau;
        let (w, h) = (sobe(lado.0 + q * passo), sobe(lado.1 + q * passo));
        if atual.as_ref().is_none_or(|(_, s)| *s != (w, h)) {
            let t = tex(gpu, w, h);
            upload(gpu, &t, w, h, &vec![255u8; (w * h * 4) as usize]);
            let novo = principal.register_texture(t);
            if let Some((velho, _)) = atual.replace((novo, (w, h))) {
                pendentes.push(velho);
            }
        }
        let img = &atual.as_ref().expect("imagem").0;
        principal.mark_texture_dirty(img);
        let mut cena = VectorScene::new();
        let rect = (0.0, 0.0, f64::from(w), f64::from(h));
        cena.draw_stable_image(
            &StableImage::from_image_data(img.clone()),
            rect,
            ImageQuality::Medium,
        );
        principal
            .render_to_intermediate(gpu, cena.inner(), tela, ph2d_vector::Color::TRANSPARENT)
            .expect("render");
        let px = readback(gpu, principal.intermediate_texture(), tela.0, tela.1);
        let o = ((tela.1 / 2 * tela.0 + tela.0 / 2) * 4) as usize;
        if px[o + 3] < 250 {
            faltas += 1;
        }
    }
    faltas
}

#[test]
#[ignore = "needs a real GPU device; sonda F34 (o piscar)"]
fn fx_zoom_sonda_do_piscar() {
    let Some(gpu) = gpu() else {
        return;
    };
    for degrau in [1, 256] {
        for (nome, lado, passo) in [
            ("fixo 2100x1900", (2100, 1900), 0),
            ("cresce 1/quadro desde 1311x1059", (1311, 1059), 1),
            ("cresce 1/quadro desde 2100x1900", (2100, 1900), 1),
            ("cresce 8/quadro desde 2100x1900", (2100, 1900), 8),
            ("cresce 8/quadro desde 3000x2600", (3000, 2600), 8),
        ] {
            let f = quadros_sem_imagem(&gpu, lado, passo, 120, degrau);
            eprintln!("[fx-piscar] degrau {degrau:>3} · {nome}: {f} de 120 quadros sem a imagem");
        }
    }
}

/// **G7 (placa) — uma textura MAIOR que o conteúdo desenha só o conteúdo.** A capacidade (G6) deixa
/// texels velhos fora dos `texels`; o `dispatch` tem de os recortar e manter a escala 1:1 do canto.
/// Fixtura: 512² com o canto 300×200 BRANCO e o resto VERMELHO (o lixo), para `rect` 300×200.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_texture_larger_than_its_content_draws_only_the_content() {
    use ph2d_render::VelloPass;
    use ph2d_vec_render::{FxImage, FxImages};
    use ph2d_vector::{StableImage, VectorScene};
    let Some(gpu) = gpu() else {
        return;
    };
    let c = cena(FxOp::BEVEL);
    let (cap, conteudo) = (512u32, (300u32, 200u32));
    let mut bytes = vec![0u8; (cap * cap * 4) as usize];
    for y in 0..cap {
        for x in 0..cap {
            let o = ((y * cap + x) * 4) as usize;
            let cor = if x < conteudo.0 && y < conteudo.1 {
                [255, 255, 255, 255]
            } else {
                [255, 0, 0, 255]
            };
            bytes[o..o + 4].copy_from_slice(&cor);
        }
    }
    let t = tex(&gpu, cap, cap);
    upload(&gpu, &t, cap, cap, &bytes);
    let tela = (640u32, 480u32);
    let mut principal = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, tela).expect("vello");
    let image = StableImage::from_image_data(principal.register_texture(t));
    let mut fx = FxImages::new();
    let rect = (100.0, 100.0, 400.0, 300.0);
    fx.insert(
        c.id,
        FxImage {
            image,
            rect,
            texels: conteudo,
        },
    );
    let mut alvo = VectorScene::new();
    ph2d_vec_render::dispatch(
        &c.scene,
        &ph2d_vec_scene::VecViewState::default(),
        &c.xf,
        &c.live,
        &fx,
        &ph2d_vec_render::WidgetSkins::new(),
        &ph2d_vec_render::PatternTiles::new(),
        &ph2d_vec_render::BrushArts::new(),
        &ph2d_vec_render::DilatedPaints::new(),
        Affine::IDENTITY,
        &mut alvo,
    );
    principal
        .render_to_intermediate(&gpu, alvo.inner(), tela, ph2d_vector::Color::TRANSPARENT)
        .expect("render");
    let px = readback(&gpu, principal.intermediate_texture(), tela.0, tela.1);
    let em = |x: u32, y: u32| {
        let o = ((y * tela.0 + x) * 4) as usize;
        [px[o], px[o + 1], px[o + 2], px[o + 3]]
    };
    for (x, y) in [(110, 110), (395, 295), (250, 200)] {
        assert_eq!(
            em(x, y),
            [255, 255, 255, 255],
            "dentro do rect em ({x},{y}) não é o conteúdo"
        );
    }
    for (x, y) in [(450, 150), (200, 350), (430, 330)] {
        assert_eq!(
            em(x, y)[3],
            0,
            "fora do rect em ({x},{y}) apareceu a textura: {:?}",
            em(x, y)
        );
    }
}

/// **G8 (placa) — o PRODUTO inteiro num gesto, quadro a quadro**: `FxLive::recook` → `dispatch` → o
/// render principal → readback, com a câmera a fazer zoom e pan perto do máximo. Conta os quadros em
/// que um ponto VISÍVEL da forma (dentro dela, longe da borda) sai sem tinta — o piscar do smoke.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_gesture_never_drops_the_filtered_shape_from_a_frame() {
    use ph2d_render::VelloPass;
    use ph2d_vec_render::{BrushArts, DilatedPaints, PatternTiles, WidgetSkins};
    use ph2d_vec_scene::VecViewState;
    use ph2d_vector::VectorScene;
    let Some(gpu) = gpu() else {
        return;
    };
    let bevel = FxOp::new(FxOp::BEVEL);
    let mut c = cena_op(bevel);
    let sombra = FxOp::new(FxOp::DROP_SHADOW);
    crate::fx_live::set_filter(
        &mut c.sim,
        &c.map,
        &[c.id],
        Some(ph2d_ecs::VecFilter::from_ops(vec![bevel, sombra])),
    );
    let tela = (VIEW[2] as u32, VIEW[3] as u32);
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let mut principal = VelloPass::new(&gpu, fmt, tela).expect("vello");
    let mut fx = crate::fx_live::FxLive::default();
    let (view, pats) = (VecViewState::default(), PatternTiles::new());
    let mut faltas = Vec::new();
    // 40 quadros de zoom de 32× a 64× no centro, e 40 de pan de 5 px na borda a 64×.
    let quadros: Vec<(Affine, [u32; 2])> = (0..40)
        .map(|q| {
            (
                camera(32.0 * 2f64.powf(f64::from(q) / 39.0), CENTRO, 0.0),
                [960, 540],
            )
        })
        .chain((0..40).map(|q| (camera(64.0, BORDA, -f64::from(q) * 5.0), [600, 540])))
        .collect();
    for (q, (cam, [x, y])) in quadros.into_iter().enumerate() {
        fx.recook(
            &c.scene,
            &c.sim,
            &c.map,
            &c.xf,
            &c.live,
            &c.live,
            &view,
            &pats,
            cam,
            VIEW,
            &gpu,
            fmt,
            &mut principal,
        );
        let mut alvo = VectorScene::new();
        ph2d_vec_render::dispatch(
            &c.scene,
            &view,
            &c.xf,
            &c.live,
            fx.images(),
            &WidgetSkins::new(),
            &pats,
            &BrushArts::new(),
            &DilatedPaints::new(),
            cam,
            &mut alvo,
        );
        principal
            .render_to_intermediate(&gpu, alvo.inner(), tela, ph2d_vector::Color::TRANSPARENT)
            .expect("render");
        let px = readback(&gpu, principal.intermediate_texture(), tela.0, tela.1);
        let a = px[((y * tela.0 + x) * 4 + 3) as usize];
        if a < 250 || fx.images().is_empty() {
            faltas.push((q, a, fx.images().len()));
        }
    }
    assert!(
        faltas.is_empty(),
        "quadros sem a forma (quadro, alfa, imagens): {faltas:?}"
    );
}

/// Sonda: UMA troca de textura (10 quadros em `a`, depois 10 em `a + 256`), por tamanho — em que
/// quadro a imagem falta.
#[test]
#[ignore = "needs a real GPU device; sonda F34 (uma troca)"]
fn fx_zoom_sonda_de_uma_troca() {
    use ph2d_render::VelloPass;
    use ph2d_vector::{ImageQuality, StableImage, VectorScene};
    let Some(gpu) = gpu() else {
        return;
    };
    for a in [1024u32, 1500, 2048, 2100, 2600, 3000, 3600, 4100] {
        let tela = (1930u32, 1040u32);
        let mut principal =
            VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, tela).expect("vello");
        let mut atual: Option<(ph2d_vector::ImageData, u32)> = None;
        let mut pendentes = Vec::new();
        let mut faltas = Vec::new();
        for q in 0..20u32 {
            for img in pendentes.drain(..) {
                principal.unregister_texture(img);
            }
            let lado = if q < 10 { a } else { a + 256 };
            if atual.as_ref().is_none_or(|(_, s)| *s != lado) {
                let t = tex(&gpu, lado, lado);
                upload(
                    &gpu,
                    &t,
                    lado,
                    lado,
                    &vec![255u8; (lado * lado * 4) as usize],
                );
                let novo = principal.register_texture(t);
                if let Some((velho, _)) = atual.replace((novo, lado)) {
                    pendentes.push(velho);
                }
            }
            let img = &atual.as_ref().expect("img").0;
            principal.mark_texture_dirty(img);
            let mut cena = VectorScene::new();
            let r = (0.0, 0.0, f64::from(lado), f64::from(lado));
            cena.draw_stable_image(
                &StableImage::from_image_data(img.clone()),
                r,
                ImageQuality::Medium,
            );
            principal
                .render_to_intermediate(&gpu, cena.inner(), tela, ph2d_vector::Color::TRANSPARENT)
                .expect("render");
            let px = readback(&gpu, principal.intermediate_texture(), tela.0, tela.1);
            if px[((500 * tela.0 + 900) * 4 + 3) as usize] < 250 {
                faltas.push(q);
            }
        }
        eprintln!("[fx-troca] {a} → {}: faltam os quadros {faltas:?}", a + 256);
    }
}

/// **G9 (placa) — a CENA 41 nas condições do dono**: a Nuvem de 120 m com o Bevel + Drop Shadow da
/// cena, a câmera real (`Camera2d`) numa janela de 2560×1080 (o monitor largo do dono), a roda a ir
/// de 1 m de altura ao zoom máximo (0,5 m) e depois o pan lá em cima. Conta os quadros em que um
/// ponto de dentro da forma sai sem tinta.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_scene_41_cloud_never_blinks_on_the_owners_screen() {
    use ph2d_render::{Camera2d, VelloPass};
    use ph2d_vec_render::{BrushArts, DilatedPaints, PatternTiles, WidgetSkins};
    use ph2d_vec_scene::{ShapeKind, VecViewState};
    use ph2d_vector::VectorScene;
    let Some(gpu) = gpu() else {
        return;
    };
    let mut vec = crate::state::VecState::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    crate::vector_mode::draw_shape(&mut vec, &mut scene, ShapeKind::Cloud, [-57.0, 0.0], 60.0);
    let id = scene.paths().first().expect("nuvem").id;
    // Como a cena 41: verde opaco, sem traço.
    let p = scene.path_mut(id).expect("nuvem");
    p.fill = Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
        120, 190, 100, 255,
    )));
    p.stroke = None;
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Bevel"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let filtro = crate::vector_mode_filtro_efeito::filtro();
    crate::fx_live::set_filter(&mut sim, &map, &[id], Some(filtro));
    let janela = ph2d_host::WindowSize {
        width: 2560,
        height: 1080,
    };
    let tela = (janela.width, janela.height);
    let vista = [0.0, 0.0, f64::from(tela.0), f64::from(tela.1)];
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let mut principal = VelloPass::new(&gpu, fmt, tela).expect("vello");
    let mut fx = crate::fx_live::FxLive::default();
    let (view, pats) = (VecViewState::default(), PatternTiles::new());
    let (xf, live) = (
        ph2d_vec_scene::VecXforms::new(),
        ph2d_vec_render::LiveGeometry::new(),
    );
    // O ponto de prova: o centro da vista, 20 m para dentro da nuvem (a borda está perto de 3 m).
    let prova = ph2d_vector::Point::new(-20.0, 0.0);
    let mut camaras = Vec::new();
    let mut h = 1.0f32;
    while h > Camera2d::ZOOM_MIN_HEIGHT_WORLD {
        h = Camera2d::zoomed(h, Camera2d::WHEEL_ZOOM_PER_NOTCH.powf(0.25));
        camaras.push(Camera2d::new([-20.0, 0.0], h));
    }
    for q in 0..40 {
        camaras.push(Camera2d::new(
            [-20.0 + 0.01 * q as f32, 0.003 * q as f32],
            h,
        ));
    }
    let mut faltas = Vec::new();
    let mut tamanhos = std::collections::BTreeSet::new();
    for (q, c) in camaras.iter().enumerate() {
        let cam = c.world_to_screen_affine(janela);
        fx.recook(
            &scene,
            &sim,
            &map,
            &xf,
            &live,
            &live,
            &view,
            &pats,
            cam,
            vista,
            &gpu,
            fmt,
            &mut principal,
        );
        let mut alvo = VectorScene::new();
        ph2d_vec_render::dispatch(
            &scene,
            &view,
            &xf,
            &live,
            fx.images(),
            &WidgetSkins::new(),
            &pats,
            &BrushArts::new(),
            &DilatedPaints::new(),
            cam,
            &mut alvo,
        );
        principal
            .render_to_intermediate(&gpu, alvo.inner(), tela, ph2d_vector::Color::TRANSPARENT)
            .expect("render");
        let px = readback(&gpu, principal.intermediate_texture(), tela.0, tela.1);
        let p = cam * prova;
        let (x, y) = (p.x as u32, p.y as u32);
        // O ponto de prova e uma grelha 5×5 da tela (todos dentro da nuvem: a borda está a 23 m).
        let a = (0..25)
            .map(|k| ((k % 5) * (tela.0 - 1) / 4, (k / 5) * (tela.1 - 1) / 4))
            .chain([(x, y)])
            .map(|(x, y)| px[((y * tela.0 + x) * 4 + 3) as usize])
            .min()
            .unwrap_or(0);
        if let Some(i) = fx.images().get(&id) {
            tamanhos.insert(i.texels);
        }
        if a < 250 || fx.images().is_empty() {
            faltas.push((q, c.height_world, a));
        }
    }
    eprintln!(
        "[g9] {} quadros, {} tamanhos de conteúdo, faltas {faltas:?}",
        camaras.len(),
        tamanhos.len()
    );
    assert!(
        faltas.is_empty(),
        "quadros sem a nuvem (quadro, altura, alfa): {faltas:?}"
    );
}
