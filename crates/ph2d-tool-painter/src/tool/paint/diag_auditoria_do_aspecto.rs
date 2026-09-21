//! SONDA — **a auditoria do report de 2026-09-21**: *«dois aspectos no mesmo círculo · se dois
//! círculos cada um tem um aspecto · artefatos retangulares nas laterais»*, com a pista que manda:
//! *«com boolean tudo melhora»*.
//!
//! ⚠️ **O oráculo aqui é a IMAGEM.** As réguas de alfa que esta linha já tem medem SOMAS e
//! CONTAGENS, e um arco com outro aspecto não move nenhuma delas — a mesma família que o `Q` que
//! era uma média e o `χ` cego à almofada. Logo: desenha-se e OLHA-SE.
//!
//! ⚠️ A rota paralela do borrão está **ILIBADA por construção**: ela é byte-idêntica à série, com
//! gate que desde 2026-09-21 corre também acima do piso da banda (`800` px). Se um rectângulo
//! aparecer, ele não vem dali.

use super::diag_composite_e_as_formas::cp2;
use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const L: u32 = 700;

/// (A1) A MATRIZ do report: um círculo contra dois, com e sem boolean, em três pilhas.
/// `PH2D_AUDIT_DIR=/tmp/aud cargo test -p ph2d-tool-painter --release --lib
/// diag_o_aspecto_do_anel -- --ignored --nocapture`
#[test]
#[ignore = "render-and-look: escreve PNG"]
fn diag_o_aspecto_do_anel() {
    use composite::CompositeOp::{Blur, Brush, Smear};
    let dir = std::env::var("PH2D_AUDIT_DIR").unwrap_or_else(|_| "/tmp/aud".into());
    std::fs::create_dir_all(&dir).unwrap();
    let pilhas: [(&str, &[(composite::CompositeOp, f32)]); 3] = [
        ("blur", &[(Blur, 1.0), (Brush, 1.0)]),
        ("smear", &[(Smear, 1.0), (Brush, 1.0)]),
        ("blursmear", &[(Blur, 1.0), (Smear, 1.0), (Brush, 1.0)]),
    ];
    for (nome_p, camadas) in pilhas {
        for dois in [false, true] {
            for boolean in [false, true] {
                let mut t = PainterTool::default();
                t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
                t.paint.brush.radius_px = 30.0;
                t.paint.brush.color = [0.75, 0.12, 0.12];
                t.paint.brush.space_attenuation = false;
                t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
                t.paint.composite_enabled = true;
                for (op, s) in camadas {
                    t.acrescenta_camada(op.to_u8());
                    let pos = t.composite_len() - 1;
                    t.set_composite_layer_strength(pos, *s);
                }
                if boolean {
                    t.set_stroke_op_mode(1); // Add
                }
                let mut circulo = |c: [f32; 2], r: f32| {
                    t.on_canvas_pointer(cp2(c, PointerPhase::Down));
                    for i in 1..=8 {
                        let u = i as f32 / 8.0;
                        t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
                    }
                    t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
                };
                circulo([350.0, 350.0], 215.0);
                if dois {
                    circulo([480.0, 430.0], 160.0);
                }
                // Compor sobre BRANCO — é o que o artista vê; o PNG cru mostraria a transparência.
                let sobre_branco: Vec<u8> = t
                    .canvas_rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|p| {
                        let a = f32::from(p[3]) / 255.0;
                        let c = |i: usize| (f32::from(p[i]) + 255.0 * (1.0 - a)).min(255.0) as u8;
                        [c(0), c(1), c(2), 255]
                    })
                    .collect();
                let png = super::composite_look::png_rgba(&sobre_branco, L, L);
                let n = format!(
                    "{}_{nome_p}{}",
                    if dois { "dois" } else { "um" },
                    if boolean { "_bool" } else { "" }
                );
                std::fs::write(format!("{dir}/{n}.png"), png).unwrap();
                eprintln!("{n}");
            }
        }
    }
}

/// (A2) **OS ARTEFACTOS RECTANGULARES** — a foto do dono é de uma figura com ALÇAS, ou seja a ser
/// AJUSTADA, e um ajuste é uma sequência de re-carimbos. A composição escreve de volta só dentro
/// de `caixa_nova` e repõe a orla até `alvo`: *se algum rectângulo se vir, é num ARRASTO e não no
/// desenho inicial*, que é o que a (A1) mede.
#[test]
#[ignore = "render-and-look: escreve PNG"]
fn diag_o_rectangulo_do_arrasto() {
    use composite::CompositeOp::{Blur, Brush, Smear};
    let dir = std::env::var("PH2D_AUDIT_DIR").unwrap_or_else(|_| "/tmp/aud".into());
    std::fs::create_dir_all(&dir).unwrap();
    let pilhas: [(&str, &[(composite::CompositeOp, f32)]); 3] = [
        ("blur", &[(Blur, 1.0), (Brush, 1.0)]),
        ("smear", &[(Smear, 1.0), (Brush, 1.0)]),
        ("blursmear", &[(Blur, 1.0), (Smear, 1.0), (Brush, 1.0)]),
    ];
    for (nome_p, camadas) in pilhas {
        for arrastos in [0usize, 6] {
            let mut t = PainterTool::default();
            t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
            t.paint.brush.radius_px = 30.0;
            t.paint.brush.color = [0.75, 0.12, 0.12];
            t.paint.brush.space_attenuation = false;
            t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
            t.paint.composite_enabled = true;
            for (op, s) in camadas {
                t.acrescenta_camada(op.to_u8());
                let pos = t.composite_len() - 1;
                t.set_composite_layer_strength(pos, *s);
            }
            let c = [350.0f32, 350.0];
            let r = 215.0f32;
            t.on_canvas_pointer(cp2(c, PointerPhase::Down));
            for i in 1..=8 {
                let u = i as f32 / 8.0;
                t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
            // O AJUSTE: o artista agarra a alça e mexe. Cada evento é um re-carimbo da figura
            // inteira, e a `caixa_nova` de cada um é PEQUENA — é ali que um rectângulo apareceria.
            for i in 0..arrastos {
                let d = (i % 2) as f32 * 6.0 - 3.0;
                t.on_canvas_pointer(cp2(c, PointerPhase::Down));
                t.on_canvas_pointer(cp2([c[0] + r + d, c[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp2([c[0] + r + d, c[1]], PointerPhase::Up));
            }
            let sobre_branco: Vec<u8> = t
                .canvas_rgba
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|p| {
                    let a = f32::from(p[3]) / 255.0;
                    let g = |i: usize| (f32::from(p[i]) + 255.0 * (1.0 - a)).min(255.0) as u8;
                    [g(0), g(1), g(2), 255]
                })
                .collect();
            let png = super::composite_look::png_rgba(&sobre_branco, L, L);
            let n = format!("arr{arrastos}_{nome_p}");
            std::fs::write(format!("{dir}/{n}.png"), png).unwrap();
            eprintln!("{n}");
        }
    }
}

/// (A11) A SEMENTE, na configuração do DONO — *«uma textura em Shape · Jitter 0»* (2026-09-21).
///
/// A auditoria levantou que a semente de aleatoriedade é por FIGURA sem boolean e partilhada com
/// ele. Com `Jitter 0` isso parecia inerte — mas o **Shape** desenha a silhueta de cada dab a
/// partir do mesmo fluxo (`tex_rng`), logo a pergunta é outra: *dois círculos CONGRUENTES saem
/// iguais?*
///
/// ⚠️ A régua compara os dois recortes **byte a byte**, o que só é honesto porque as duas figuras
/// são congruentes por construção (mesmo raio, mesmo pincel, mesma pilha) e a fixtura o afirma.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_a_semente_com_shape -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_a_semente_com_shape() {
    use composite::CompositeOp::Brush;
    let (a, b, r) = ([175.0f32, 175.0], [525.0f32, 525.0], 110.0f32);
    const J: u32 = 300; // o recorte à volta de cada figura

    // Uma silhueta com estrutura: faixas diagonais, para uma realização diferente se NOTAR.
    let (sw, sh) = (64u32, 64u32);
    let lum: Vec<u8> = (0..sw * sh)
        .map(|i| {
            let (x, y) = (i % sw, i / sw);
            if (x + y) % 16 < 8 { 255 } else { 40 }
        })
        .collect();

    for com_shape in [false, true] {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 24.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.jitter = 0.0;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.paint.composite_enabled = true;
        t.acrescenta_camada(Brush.to_u8());
        t.set_composite_layer_strength(0, 1.0);
        if com_shape {
            t.set_brush_shape_image(lum.clone(), sw, sh);
        }
        let mut circulo = |c: [f32; 2]| {
            t.on_canvas_pointer(cp2(c, PointerPhase::Down));
            for i in 1..=8 {
                let u = i as f32 / 8.0;
                t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        };
        circulo(a);
        circulo(b);

        // Os dois recortes, centrados em cada figura.
        let recorte = |c: [f32; 2]| -> Vec<u8> {
            let (x0, y0) = (
                (c[0] - J as f32 / 2.0) as u32,
                (c[1] - J as f32 / 2.0) as u32,
            );
            let mut v = Vec::with_capacity((J * J * 4) as usize);
            for y in 0..J {
                let i = (((y0 + y) * L + x0) * 4) as usize;
                v.extend_from_slice(&t.canvas_rgba[i..i + (J * 4) as usize]);
            }
            v
        };
        let (ra, rb) = (recorte(a), recorte(b));
        let n_a = ra.iter().skip(3).step_by(4).filter(|&&x| x > 4).count();
        let n_b = rb.iter().skip(3).step_by(4).filter(|&&x| x > 4).count();
        let difs = ra.iter().zip(&rb).filter(|(x, y)| x != y).count();
        println!(
            "  Shape {:>3} · figura A {n_a:6} texels · figura B {n_b:6} · bytes DIFERENTES {difs:7} \
             ({:5.1} % do recorte)",
            if com_shape { "ON" } else { "off" },
            100.0 * difs as f32 / ra.len() as f32
        );
    }
}
