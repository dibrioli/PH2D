//! **A SONDA DA ESCALA** — irmã de [`super::crossing_probe`], separada por ASSUNTO e não por tamanho.
//!
//! O pai responde *união ou composição?* numa cena pequena e fixa. Este responde a pergunta que a
//! medição de 2026-08-12 abriu depois de refutar aquela: **o défice do aro na quina côncava é função
//! de `edge_spread / raio`**, e para vê-lo é preciso varrer as duas grandezas — o que exige tela,
//! pincel e réguas próprias (as consts do pai são da cena pequena e não servem aqui).
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release crossing_scale -- --ignored --nocapture`

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::BrushSpec;

use super::accumulate_probe::cp;
use super::crossing_probe::{INK, arm, white};

/// **A ESCALA DA FOTO** — a cena acima usa raio 24 e o `warp` de fábrica é **6 px ABSOLUTOS**, então
/// ali a borda esfarrapada mede um quarto da largura do braço; na foto do Enio o braço tem ~150 px e
/// a mesma raggedness mede 4% dele. Uma cunha que só existe numa das duas escalas é um fato sobre a
/// RAZÃO `warp / raio`, e a fixture pequena não a contém.
///
/// Esta sonda re-mede o alcance da quina e o perfil da bissetriz com o pincel GRANDE, na sua própria
/// tela — as consts do módulo são da cena pequena e não servem aqui.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_notch_at_the_photos_scale() {
    for (radius, spread) in [(24.0f32, 7.0f32), (48.0, 7.0), (75.0, 7.0), (110.0, 7.0)] {
        let size: u32 = 512;
        let c = 256.0f32;
        let mut t = PainterTool::default();
        t.set_source(white(size), size, size);
        let spec = BrushSpec {
            radius_px: radius,
            color: [0.85, 0.15, 0.15],
            space_attenuation: false,
            watercolor: true,
            edge_spread: spread,
            ..Default::default()
        };
        arm(&mut t, spec);
        let mut draw = |from: [f32; 2], to: [f32; 2]| {
            t.on_canvas_pointer(cp(from, PointerPhase::Down));
            let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
            for i in 1..=48 {
                let f = i as f32 / 48.0;
                t.on_canvas_pointer(cp([from[0] + dx * f, from[1] + dy * f], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp(to, PointerPhase::Up));
        };
        draw([40.0, c], [472.0, c]);
        draw([c, 40.0], [c, 472.0]);

        let a = |x: f32, y: f32| -> f32 {
            let (xi, yi) = (x.round() as u32, y.round() as u32);
            let i = ((yi * size) + xi) as usize * 4;
            (255.0 - f32::from(t.canvas_rgba[i + 1])) / 255.0
        };
        // `w`: meia-largura do braço, longe do cruzamento — e ⚠️ **longe da PONTA também**: a 1ª
        // versão media em `c − 4·raio`, que num pincel de 75 cai FORA da tela (o `as u32` satura em 0)
        // e a sonda passou a medir a calota do começo do traço em vez do flanco (`w = 48` para um
        // braço de ~65). O ponto tem de ficar entre a ponta e o braço vertical.
        let lone_x = c - radius * 1.8;
        let mut w = 0.0f32;
        let mut y = 0.0f32;
        while y < radius * 2.0 {
            if a(lone_x, c + y) > INK {
                w = y;
            }
            y += 0.5;
        }
        let s = std::f32::consts::FRAC_1_SQRT_2;
        println!(
            "\n=== raio {radius:.0} · spread {spread:.0} · warp 6 (warp/raio = {:.2}) ===",
            6.0 / radius
        );
        println!(
            "   w = {w:.1}   bissetriz esperada = {:.1}",
            w * std::f32::consts::SQRT_2
        );
        for (label, sx, sy) in [("++", s, s), ("+-", s, -s), ("-+", -s, s), ("--", -s, -s)] {
            let mut last = 0.0f32;
            let mut gap_from = 0.0f32;
            let mut gap_len = 0.0f32;
            let mut run = 0.0f32;
            let mut d = 0.0f32;
            while d < radius * 3.0 {
                if a(c + sx * d, c + sy * d) > INK {
                    if run > gap_len && last > 0.0 {
                        gap_len = run;
                        gap_from = d - run;
                    }
                    last = d;
                    run = 0.0;
                } else {
                    run += 0.5;
                }
                d += 0.5;
            }
            println!(
                "   {label}  alcance {last:5.1}   maior BURACO cercado de tinta: {gap_len:4.1} px \
                 (a partir de {gap_from:5.1})"
            );
        }
        // O perfil, do miolo à borda, ao longo da bissetriz e ao longo do eixo (o controle).
        // Os dois perfis são comparados na MESMA régua: a distância PERPENDICULAR ao eixo de uma
        // faixa. Na bissetriz o ponto `(c+k, c+k)` está a `k` de CADA eixo, então o eixo é amostrado
        // em `k` também — é isso que torna "a quina tem o mesmo aro que o flanco" uma afirmação.
        // O ARO, de 1 em 1 px, na faixa onde ele vive (as últimas 24 px do braço) — os QUATRO cantos
        // contra o flanco reto. Uma cunha aparece como um aro mais fraco ou NÃO-MONÓTONO num canto.
        let lo = (w - 22.0).max(0.0);
        let prof = |f: &dyn Fn(f32) -> f32| {
            let mut s2 = String::new();
            let mut k = lo;
            while k <= w + 4.0 {
                s2.push_str(&format!("{:4.0}", f(k) * 100.0));
                k += 1.0;
            }
            s2
        };
        println!("   aro por s = {lo:.0}..{:.0} px do eixo", w + 4.0);
        println!("     flanco reto {}", prof(&|k| a(lone_x, c + k)));
        println!("     quina ++    {}", prof(&|k| a(c + k, c + k)));
        println!("     quina +-    {}", prof(&|k| a(c + k, c - k)));
        println!("     quina -+    {}", prof(&|k| a(c - k, c + k)));
        println!("     quina --    {}", prof(&|k| a(c - k, c - k)));
    }
}

/// **O DÉFICE DO ARO NA QUINA, contra `spread / raio`** — a tabela que nomeia a causa.
///
/// O aro é derivado de `cw − blur(hard)`, e o borrão tem raio `core_r = min(edge_spread, raio/2)` —
/// um número em **px ABSOLUTOS** enquanto o ombro da silhueta escala com o PINCEL. Num pincel grande
/// o ombro fica muito mais largo que o borrão, o aro enfraquece, e **na quina côncava ele se rompe**:
/// ali o borrão enxerga a tinta do OUTRO braço (a quina é genuinamente mais funda) e o `inner` sobe.
///
/// A tabela mede o pico do aro no flanco RETO e o pior dos QUATRO cantos, para cada `(raio, spread)`.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_rim_deficit_against_spread_over_radius() {
    println!("\n=== O DEFICE DO ARO NA QUINA (pior dos 4 cantos contra o flanco reto) ===");
    println!("   raio  spread  spread/raio   flanco   quina   defice");
    for radius in [24.0f32, 48.0, 75.0, 110.0] {
        for spread in [7.0f32, 16.0, 32.0, 48.0] {
            let size: u32 = 512;
            let c = 256.0f32;
            let mut t = PainterTool::default();
            t.set_source(white(size), size, size);
            let spec = BrushSpec {
                radius_px: radius,
                color: [0.85, 0.15, 0.15],
                space_attenuation: false,
                watercolor: true,
                edge_spread: spread,
                ..Default::default()
            };
            arm(&mut t, spec);
            let mut draw = |from: [f32; 2], to: [f32; 2]| {
                t.on_canvas_pointer(cp(from, PointerPhase::Down));
                let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
                for i in 1..=48 {
                    let f = i as f32 / 48.0;
                    t.on_canvas_pointer(cp(
                        [from[0] + dx * f, from[1] + dy * f],
                        PointerPhase::Move,
                    ));
                }
                t.on_canvas_pointer(cp(to, PointerPhase::Up));
            };
            draw([40.0, c], [472.0, c]);
            draw([c, 40.0], [c, 472.0]);
            let a = |x: f32, y: f32| -> f32 {
                let (xi, yi) = (x.round() as u32, y.round() as u32);
                let i = ((yi * size) + xi) as usize * 4;
                (255.0 - f32::from(t.canvas_rgba[i + 1])) / 255.0
            };
            let lone_x = c - radius * 1.8;
            // O pico do aro é procurado na MESMA faixa de `s` nos cinco lugares — o aro da quina mora
            // um pouco mais fundo (a quina é geometricamente mais funda), então a faixa é generosa.
            let peak = |f: &dyn Fn(f32) -> f32| {
                let mut m = 0.0f32;
                let mut k = 0.0f32;
                while k < radius * 1.2 {
                    m = m.max(f(k));
                    k += 0.5;
                }
                m
            };
            let flank = peak(&|k| a(lone_x, c + k));
            let corner = [
                peak(&|k| a(c + k, c + k)),
                peak(&|k| a(c + k, c - k)),
                peak(&|k| a(c - k, c + k)),
                peak(&|k| a(c - k, c - k)),
            ]
            .into_iter()
            .fold(1.0f32, f32::min);
            println!(
                "   {radius:4.0}  {spread:6.0}   {:9.2}   {flank:6.2}  {corner:6.2}  {:+6.0}%",
                spread / radius,
                (corner / flank - 1.0) * 100.0
            );
        }
    }
}
