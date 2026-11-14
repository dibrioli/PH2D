//! Sonda B11.3 A×B (filha de [`super`] por LOC).

use super::*;

/// **Sonda B11.3 A×B** (smoke, `--ignored --nocapture --test-threads=1`): tela 3840×2160, razão 2, Wet Drip com Smooth
/// Edges 1 em três traços, e o custo do fio da tela por quadro enquanto a água corre pelo compositor. A =
/// `set_otica_rgba_suave(false)` (os filtros de antes: o laço de píxel em casa) × B = os RGBA do compositor, no mesmo
/// processo, intercalados; mínimo por variante da soma e da mediana dos quadros.
#[test]
#[ignore = "sonda: --ignored --nocapture --test-threads=1, perfil smoke"]
fn sonda_b11_3_rgba_suave_4k() {
    const W: usize = 3840;
    const H: usize = 2160;
    const QUADROS_MEDIDOS: usize = 60;
    let uma = |novo: bool| -> (f64, f64, f64, u32, u8) {
        crate::wet_diag::set_otica_rgba_suave(novo);
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        forca_no_teste(Some(true));
        let wd = "Watercolor/Fluid/Wet Drip.ph2d-brush.png";
        t.preset_aplicar(&preset(wd)).expect("o preset");
        t.set_wet_grid_ratio(2.0);
        t.set_brush_size_px(120.0);
        t.set_brush_color_srgb8([230, 200, 30]);
        t.set_wet_smooth_edges(1.0);
        for (i, y) in [700.0f32, 1080.0, 1460.0].into_iter().enumerate() {
            let x0 = 500.0 + 200.0 * i as f32;
            let _ = t.on_canvas_pointer(ponto(x0, y, PointerPhase::Down));
            for k in 1..=30 {
                let x = x0 + 2600.0 * k as f32 / 30.0;
                let _ = t.on_canvas_pointer(ponto(x, y + 2.0 * k as f32, PointerPhase::Move));
                tique(&mut t);
                std::thread::sleep(QUADRO);
            }
            let _ = t.on_canvas_pointer(ponto(x0 + 2600.0, y, PointerPhase::Up));
        }
        let antes = sess(&t).otica.rgba_usadas;
        let mut ms = Vec::with_capacity(QUADROS_MEDIDOS);
        for _ in 0..QUADROS_MEDIDOS {
            let t0 = Instant::now();
            tique(&mut t);
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            std::thread::sleep(QUADRO);
        }
        let usadas = sess(&t).otica.rgba_usadas - antes;
        let ratio = sess(&t).ratio;
        let soma: f64 = ms.iter().sum();
        ms.sort_by(f64::total_cmp);
        (soma, ms[ms.len() / 2], ms[ms.len() - 1], usadas, ratio)
    };
    let mut melhor = [[f64::MAX; 3]; 2];
    for rodada in 0..3 {
        for novo in if rodada % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let (soma, med, max, usadas, ratio) = uma(novo);
            println!(
                "rodada {rodada} {}: soma {soma:.1} ms · mediana {med:.2} · máx {max:.1} · faixas com RGBA {usadas} · razão {ratio}",
                if novo {
                    "B (RGBA do compositor)"
                } else {
                    "A (filtros antigos)"
                }
            );
            let m = &mut melhor[usize::from(novo)];
            for (a, v) in m.iter_mut().zip([soma, med, max]) {
                *a = a.min(v);
            }
        }
    }
    crate::wet_diag::set_otica_rgba_suave(true);
    println!("mín A soma·mediana·máx {:?} × B {:?}", melhor[0], melhor[1]);
}
