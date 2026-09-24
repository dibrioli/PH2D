//! ⭐⭐⭐ **O PAINTER PINTA A PEÇA, PELO CAMINHO DO PRODUTO** — irmão (`#[path]`)
//! do [`super`], com o arnês dele.
//!
//! ⚠️ A LEI (a amostragem, a oclusão, a máscara) é gateada sem placa na
//! `ph2d_sculpt3d::tela_na_malha`, e a TELA do lado do Painter na
//! `ph2d_tool_painter`. O que ESTE afirma é a corrente que nenhum dos dois vê:
//! a tela prende-se à vista da cena, um traço do Painter — o motor dele, pela
//! porta pública do ponteiro — aterra na peça, o traço fecha como um traço de
//! pintura, e o `Ctrl+Z` devolve a peça.

use super::{amostras, cena_52, tecla};
use crate::painter_na_malha::{entrega, quadro};
use ph2d_editor_core::tool::PointerPhase;
use ph2d_tool_painter::PainterTool;

fn painter_vermelho() -> PainterTool {
    let mut p = PainterTool::default();
    p.set_brush_color_srgb8([255, 0, 0]);
    p.set_brush_strength(1.0);
    p.set_brush_size_px(24.0);
    p
}

/// Um traço pelo caminho da shell: um quadro prende a tela, os pontos entram
/// em coordenadas de janela, e o `Up` pousa e fecha.
fn traco(s: &mut crate::Sculpt3dScene, p: &mut PainterTool, x0: f32) -> bool {
    quadro(Some(&mut *s), Some(&mut *p));
    let ok = entrega(s, p, x0, 350.0, 1.0, PointerPhase::Down);
    for k in 1..=8u8 {
        entrega(
            s,
            p,
            x0 + 6.0 * f32::from(k),
            350.0,
            1.0,
            PointerPhase::Move,
        );
        quadro(Some(&mut *s), Some(&mut *p));
    }
    entrega(s, p, x0 + 54.0, 350.0, 1.0, PointerPhase::Up);
    ok
}

/// A cor EFECTIVA por vértice: sem canal, toda a peça é a cor de fábrica —
/// um traço materializa o canal, e o desfazer devolve-lhe os valores, não a
/// ausência (o mesmo que o pincel de pintura faz).
fn cores(s: &crate::Sculpt3dScene) -> Vec<[f32; 3]> {
    let m = s.mesh();
    m.colors().map_or_else(
        || vec![ph2d_mesh::DEFAULT_COLOR; m.vert_count()],
        <[[f32; 3]]>::to_vec,
    )
}

fn vermelha(c: &[f32; 3]) -> bool {
    c[0] > 0.9 && c[1] < 0.1 && c[2] < 0.1
}

/// ⭐⭐⭐ **GATE — UM TRAÇO DO PAINTER ATERRA NA TINTA FINA DA PEÇA, E UM
/// `Ctrl+Z` DEVOLVE-A AO BIT.**
///
/// ⚠️ **O CONTROLO é a tela não estar presa antes do 1.º quadro:** sem ele o
/// gate passaria sobre uma costura em que a escultura pintasse por outro
/// caminho qualquer.
#[test]
#[ignore = "precisa de adaptador"]
fn um_traco_do_painter_pinta_a_peca_e_o_ctrl_z_devolve() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let virgem = amostras(&s);
    assert!(!virgem.is_empty(), "o plano de tinta fina está armado");
    let mut p = painter_vermelho();
    assert!(
        !p.on_screen_canvas(),
        "o CONTROLO: antes do quadro a tela não existe"
    );

    assert!(traco(&mut s, &mut p, 420.0), "o pen-down foi do Painter");
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois = amostras(&s);
    let vermelhas = depois.iter().filter(|c| vermelha(c)).count();
    assert!(vermelhas > 0, "o traço do Painter não aterrou na peça");
    assert!(
        depois.iter().any(|c| !vermelha(c)),
        "o traço pintou a peça inteira: a tela não foi limitada ao traço"
    );
    assert!(
        s.stroke.tinta_fina.is_none() && s.painter_tela.is_none(),
        "o traço fechou e devolveu o plano à peça"
    );

    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(amostras(&s), virgem, "o Ctrl+Z não devolveu a peça, ao bit");
}

/// ⭐⭐ **GATE — sem tinta fina o traço aterra na COR POR VÉRTICE**, pela mesma
/// captura do pincel de pintura, e o `Ctrl+Z` também a devolve.
#[test]
#[ignore = "precisa de adaptador"]
fn sem_tinta_fina_o_painter_pinta_os_vertices() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.tinta_nivel = None;
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert!(
        amostras(&s).is_empty(),
        "o CONTROLO: o plano está desarmado"
    );
    let antes = cores(&s);
    let mut p = painter_vermelho();
    assert!(traco(&mut s, &mut p, 420.0));
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois = cores(&s);
    assert!(
        depois.iter().any(vermelha),
        "o traço não aterrou nos vértices"
    );
    assert!(tecla(&mut s, false));
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(cores(&s), antes, "o Ctrl+Z não devolveu a cor por vértice");
}

/// **Sem barro no ecrã a tela solta-se** — a ponte da sprite volta a mandar.
#[test]
#[ignore = "precisa de adaptador"]
fn sem_cena_a_tela_solta_se() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    let mut p = painter_vermelho();
    quadro(Some(&mut s), Some(&mut p));
    assert!(
        p.on_screen_canvas(),
        "o CONTROLO: com a peça no ecrã a tela prende"
    );
    quadro(None, Some(&mut p));
    assert!(!p.on_screen_canvas());
}

/// Um traço por uma lista de pontos de janela, pelo caminho da shell.
fn traco_por(s: &mut crate::Sculpt3dScene, p: &mut PainterTool, pontos: &[(f32, f32)]) -> bool {
    quadro(Some(&mut *s), Some(&mut *p));
    let (x0, y0) = pontos[0];
    let ok = entrega(s, p, x0, y0, 1.0, PointerPhase::Down);
    for &(x, y) in &pontos[1..] {
        entrega(s, p, x, y, 1.0, PointerPhase::Move);
        quadro(Some(&mut *s), Some(&mut *p));
    }
    let &(x1, y1) = pontos.last().expect("um ponto");
    entrega(s, p, x1, y1, 1.0, PointerPhase::Up);
    ok
}

/// ⭐⭐⭐ **GATE — UM BORRÃO DO PAINTER ARRASTA A TINTA QUE A PEÇA JÁ TEM, e um
/// `Ctrl+Z` desfaz SÓ o borrão, ao bit** (etapa 2). ⚠️ Na etapa 1 a tela
/// começava transparente e um borrão borrava o NADA: este gate reprovaria ali
/// com zero amostras mudadas, que é o CONTROLO de que ele mede o retrato.
#[test]
#[ignore = "precisa de adaptador"]
fn um_borrao_do_painter_arrasta_a_tinta_da_peca_e_o_ctrl_z_devolve() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let mut p = painter_vermelho();
    assert!(traco(&mut s, &mut p, 420.0), "o traço vermelho");
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois_do_vermelho = amostras(&s);
    p.set_paint_tool_mode("smear");
    assert!(p.screen_canvas_reads_the_piece());
    let descer: Vec<(f32, f32)> = (0..=20).map(|k| (447.0, 350.0 + 3.0 * k as f32)).collect();
    assert!(traco_por(&mut s, &mut p, &descer), "o pen-down do borrão");
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois_do_borrao = amostras(&s);
    let avermelharam = depois_do_borrao
        .iter()
        .zip(&depois_do_vermelho)
        // ⚠️ A peça é BRANCA (`DEFAULT_COLOR`): sobre o branco o vermelho não
        // sobe — avermelhar é o VERDE e o AZUL descerem. A 1.ª redacção pedia o
        // vermelho a subir e reprovava sobre um borrão certo.
        .filter(|(d, a)| !vermelha(a) && d[1] < a[1] - 0.05 && d[2] < a[2] - 0.05)
        .count();
    assert!(
        avermelharam > 0,
        "o borrão não levou o vermelho a amostra nenhuma que não o tinha"
    );
    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s),
        depois_do_vermelho,
        "o Ctrl+Z não devolveu a peça ao estado de antes do borrão, ao bit"
    );
}

/// 🔎 **SONDA (não é gate)** — desenha a cena depois de uma pincelada do Painter
/// e grava um PNG em `$PH2D_SONDA_PNG`, para se VER a tinta na peça. Sem a
/// variável não grava nada.
#[test]
#[ignore = "sonda: precisa de adaptador e de PH2D_SONDA_PNG"]
fn diag_fotografa_a_pincelada_do_painter() {
    let Some(caminho) = std::env::var_os("PH2D_SONDA_PNG") else {
        return;
    };
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let mut p = painter_vermelho();
    p.set_brush_size_px(14.0);
    for (k, y) in [300.0f32, 340.0, 380.0].into_iter().enumerate() {
        quadro(Some(&mut s), Some(&mut p));
        let x0 = 330.0 + 20.0 * k as f32;
        entrega(&mut s, &mut p, x0, y, 1.0, PointerPhase::Down);
        for i in 1..=30u8 {
            let x = x0 + 8.0 * f32::from(i);
            let yy = y + 25.0 * (f32::from(i) * 0.3).sin();
            entrega(&mut s, &mut p, x, yy, 1.0, PointerPhase::Move);
            quadro(Some(&mut s), Some(&mut p));
        }
        entrega(&mut s, &mut p, x0 + 240.0, y, 1.0, PointerPhase::Up);
    }
    // E um BORRÃO a descer pelas três ondas (etapa 2): o vermelho tem de ser
    // arrastado para baixo, e não borrado sobre o vazio.
    p.set_paint_tool_mode("smear");
    p.set_brush_size_px(24.0);
    for x in [420.0f32, 470.0, 520.0] {
        let descer: Vec<(f32, f32)> = (0..=30).map(|k| (x, 290.0 + 4.0 * k as f32)).collect();
        traco_por(&mut s, &mut p, &descer);
    }
    let (w, h) = (900u32, 700u32);
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sonda painter"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ph2d_render::GameRt::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    s.render(&gpu, &view, (w, h));
    let bpr = (w * 8).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(bpr) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
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
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let dados = buf.slice(..).get_mapped_range();
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h as usize {
        let linha = &dados[y * bpr as usize..];
        for x in 0..w as usize {
            for c in 0..3 {
                let o = x * 8 + c * 2;
                let v = half(u16::from_le_bytes([linha[o], linha[o + 1]])).clamp(0.0, 1.0);
                let s = if v <= 0.003_130_8 {
                    v * 12.92
                } else {
                    1.055 * v.powf(1.0 / 2.4) - 0.055
                };
                rgb.push((s * 255.0 + 0.5) as u8);
            }
        }
    }
    image::save_buffer(caminho, &rgb, w, h, image::ColorType::Rgb8).expect("gravar o PNG");
}

fn half(b: u16) -> f32 {
    let e = i32::from((b >> 10) & 0x1f);
    let m = f32::from(b & 0x3ff);
    let s = if b & 0x8000 != 0 { -1.0 } else { 1.0 };
    s * match e {
        0 => m * 2f32.powi(-24),
        31 => f32::INFINITY,
        _ => (1.0 + m / 1024.0) * 2f32.powi(e - 15),
    }
}
