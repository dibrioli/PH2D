//! **Prancha B24.1 + B24.2**: o Composite Brush (Brush por baixo, Blur por cima) numa camada
//! TRANSPARENTE sobre 3 fundos × 1080p/4K. O mesmo ficheiro compila na base (`b2bc9e02b0`, o
//! «antes») e na cura: cada corrida grava os crus em `PH2D_PRANCHA_B24_SAIDA` (omissão: temp); com
//! `PH2D_PRANCHA_B24_ANTES` (a pasta da base) monta as PNG em `docs/Painter/missao_4k/pranchas/B24/`.

use super::{CompositeLayer, CompositeOp, N_CAMADAS};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

/// A imagem achatada inteira depois de UMA onda do Composite (Brush amarelo + Blur `tam_blur`× o
/// pincel de raio `raio`·h/1080), na camada nova transparente ou (`!nova`) no próprio fundo.
fn cena(w: u32, h: u32, qual: u8, raio: f32, tam_blur: f32, nova: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    if nova {
        t.add_raster_layer("tinta").expect("a camada transparente");
    }
    let s = h as f32 / 1080.0;
    t.paint.brush.radius_px = raio * s;
    t.paint.composite_enabled = true;
    for pos in 0..N_CAMADAS {
        t.paint.composite[pos] = CompositeLayer::default();
    }
    // Posição 0 = o TOPO: o Blur (o dobro do tamanho) borra o que o Brush pintou para dentro do
    // vazio da camada — é aí, onde o texel era `0,0,0,0`, que a mistura recta punha a orla escura.
    t.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Blur,
        strength: 1.0,
        size: tam_blur,
        ..CompositeLayer::default()
    };
    t.paint.composite[2] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        color: Some([0.98, 0.86, 0.08]),
        ..CompositeLayer::default()
    };
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 4.0);
    let passos = 120;
    let ponto = |k: usize| {
        let f = k as f32 / passos as f32;
        let x = cx - l + 2.0 * l * f;
        [x, cy + 0.35 * l * (std::f32::consts::TAU * f).sin()]
    };
    t.on_canvas_pointer(cp(ponto(0), PointerPhase::Down));
    for k in 1..passos {
        t.on_canvas_pointer(cp(ponto(k), PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(ponto(passos), PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// O quadrado `lado²` a partir de `(x0, y0)`, ampliado `z×` pelo vizinho mais próximo.
fn recorte(img: &[u8], w: u32, x0: u32, y0: u32, lado: u32, z: u32) -> Vec<[u8; 4]> {
    let mut out = Vec::with_capacity((lado * lado * z * z) as usize);
    for y in 0..lado * z {
        for x in 0..lado * z {
            let i = (((y0 + y / z) * w + x0 + x / z) * 4) as usize;
            out.push([img[i], img[i + 1], img[i + 2], img[i + 3]]);
        }
    }
    out
}

/// Antes | depois | o que muda (vermelho ∝ a diferença): o recorte central `h/2` e, embaixo, a lupa
/// 4× de lado `h/8` centrada em `lupa_c`.
fn monta(a: &[u8], d: &[u8], w: u32, h: u32, lupa_c: (u32, u32)) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, (h - lado) / 2);
    let lupa = lado / 4;
    let (lx, ly) = (lupa_c.0 - lupa / 2, lupa_c.1 - lupa / 2);
    let gap = 8u32;
    let larg = 3 * lado + 2 * gap;
    let mut par =
        image::RgbaImage::from_pixel(larg, 2 * lado + gap, image::Rgba([40, 40, 40, 255]));
    for (linha, (ox, oy, l, z)) in [(x0, y0, lado, 1u32), (lx, ly, lupa, 4)]
        .into_iter()
        .enumerate()
    {
        let (ra, rd) = (recorte(a, w, ox, oy, l, z), recorte(d, w, ox, oy, l, z));
        let top = linha as u32 * (lado + gap);
        for (i, (pa, pd)) in ra.iter().zip(&rd).enumerate() {
            let (x, y) = (i as u32 % lado, top + i as u32 / lado);
            let m = (0..4).map(|c| pa[c].abs_diff(pd[c])).max().unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([pa[0], pa[1], pa[2], 255]));
            par.put_pixel(x + lado + gap, y, image::Rgba([pd[0], pd[1], pd[2], 255]));
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B24: grava píxeis crus; com PH2D_PRANCHA_B24_ANTES monta as PNG"]
fn prancha_b24_o_blur_do_composite_na_camada_transparente() {
    // Sem a variável, os crus vão para a pasta temporária: a corrida genérica `--ignored` também grava.
    let saida = std::env::var("PH2D_PRANCHA_B24_SAIDA").unwrap_or_else(|_| {
        std::env::temp_dir()
            .join("ph2d_prancha_b24_crua")
            .display()
            .to_string()
    });
    std::fs::create_dir_all(&saida).expect("pasta crua");
    let antes = std::env::var("PH2D_PRANCHA_B24_ANTES").ok();
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B24"
    );
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let d = cena(w, h, qual, 28.0, 2.0, true);
            std::fs::write(format!("{saida}/{res}_{nome}.rgba"), &d).expect("cru");
            let Some(pasta) = &antes else { continue };
            let a = std::fs::read(format!("{pasta}/{res}_{nome}.rgba")).expect("o antes");
            assert_eq!(a.len(), d.len(), "{res} {nome}: tamanhos");
            let difs = a
                .as_chunks::<4>()
                .0
                .iter()
                .zip(d.as_chunks::<4>().0)
                .filter(|(x, y)| x != y)
                .count();
            let pior = a
                .iter()
                .zip(&d)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap_or(0);
            eprintln!("PRANCHA {res} {nome}: {difs} píxeis diferentes, pior {pior}");
            // A lupa (4×): o corte da onda no 1.º quarto (o vale), as duas bordas onde a tinta encontra o vazio.
            let par = monta(&a, &d, w, h, (w / 2 - h / 8, h / 2 + h * 35 / 400));
            std::fs::create_dir_all(dir).expect("pasta");
            par.save(format!("{dir}/{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}

/// **Prancha B24.5**: o Composite de pincel GRANDE (raio 120·h/1080, Blur do mesmo tamanho) no
/// próprio fundo: «antes» = o `k` da pilha no tecto do binomial (32), «depois» = `0,34·r`. Oráculo
/// no mesmo processo ([`super::COM_O_TECTO_DO_NUCLEO`]).
#[test]
#[ignore = "prancha B24.5: escreve PNGs em docs/Painter/missao_4k/pranchas/B24/"]
fn prancha_b24_5_o_borrao_do_pincel_grande() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B24"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            super::COM_O_TECTO_DO_NUCLEO.set(true);
            let a = cena(w, h, qual, 120.0, 1.0, false);
            super::COM_O_TECTO_DO_NUCLEO.set(false);
            let d = cena(w, h, qual, 120.0, 1.0, false);
            let difs = a
                .as_chunks::<4>()
                .0
                .iter()
                .zip(d.as_chunks::<4>().0)
                .filter(|(x, y)| x != y)
                .count();
            let pior = a
                .iter()
                .zip(&d)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap_or(0);
            eprintln!("PRANCHA B24.5 {res} {nome}: {difs} píxeis diferentes, pior {pior}");
            // A lupa na borda de baixo da onda, no vale (y = centro + 0,0875·h + o raio).
            let centro = (w / 2 - h / 8, h / 2 + h * 35 / 400 + h * 120 / 1080);
            monta(&a, &d, w, h, centro)
                .save(format!("{dir}/b24_5_{res}_{nome}_antes_depois.png"))
                .expect("png");
        }
    }
}
