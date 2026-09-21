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
