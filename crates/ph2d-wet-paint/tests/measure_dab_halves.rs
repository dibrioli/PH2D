//! **DE QUE UM DAB MOLHADO É FEITO** — as duas metades do depósito, medidas
//! separadas, e o que o cap do pincel escondia sobre a ESCALA delas.
//!
//! ⚠️ **A nota que abre esta frente foi escrita SOB o cap** (doc 28 §5.50):
//! *"1,86 / 3,34 / 4,37 ms por entrega nos raios 100 / 200 / 300, e a escala é
//! SUB-linear no raio (1 : 1,8 : 2,3 contra 1 : 4 : 9 de uma pegada), provável
//! assinatura do `TRAIL_HALF = 61` que clipa pincel grande"*. A wave seguinte
//! **removeu esse cap**, e o CLAUDE.md §0 é explícito: *quem move o número que
//! tornava algo inalcançável tem de reconferir a nota*. Esta sonda é a
//! reconferência.
//!
//! **A previsão que ela testa:** com a janela seguindo o pincel, as duas
//! metades passam a ser `O(r²)` de verdade — a sub-linearidade era o cap
//! ESCONDENDO trabalho, não uma propriedade do depósito. Se for isso, o pincel
//! grande ficou mais caro do que a nota dizia, e é a escala — não uma constante
//! — que decide se há wave aqui.
//!
//! ⚠️ **Por que a fixture dirige o `Trail` e não o `on_canvas_pointer`:** as
//! duas metades são DUAS chamadas do motor (`accumulate_paint` e
//! `transfer_paint`), e a porta do produto as funde num evento só. Medir por
//! dentro é o único jeito de as separar — e o número que vira decisão de
//! produto continua saindo da porta do produto (`measure_wetpaint_stamp.rs`),
//! contra o qual esta tabela tem de RECONCILIAR.

use std::time::Instant;

use ph2d_wet_paint::brush::BrushShape;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};

const SIDE: usize = 4096;

fn dab_at(x: f64, y: f64, r: f64) -> Dab {
    Dab {
        x,
        y,
        r,
        hardness: 0.5,
        intensity: 1.0,
        water_amount: 0.5,
        dry_gate: 0.0,
        shape: BrushShape::Round,
        dir_x: 1.0,
        dir_y: 0.0,
    }
}

/// As duas metades do depósito, por raio, com a razão contra `r²` ao lado.
///
/// A coluna que decide é a **razão normalizada**: se ela ficar plana, o custo é
/// a PEGADA (o certo, e nada a fazer); se subir, algo percorre mais que a
/// pegada; se descer, algo ainda está capado.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_a_wet_dab_is_made_of() {
    const DABS: u32 = 24;

    println!("\n  AS DUAS METADES DE UM DAB MOLHADO ({SIDE}x{SIDE}, {DABS} dabs)\n");
    println!(
        "    {:>6} {:>8} {:>12} {:>12} {:>12}   {:>10} {:>10}",
        "raio", "janela", "accum ms", "transf ms", "total ms", "ns/r2", "transf %"
    );

    let mut rows = Vec::new();
    for r in [60.0f64, 100.0, 200.0, 300.0, 400.0] {
        let mut e = Engine::new(SIDE, SIDE);
        let p = e.sim.gather_params(&e.tuning);
        let tex: Vec<f32> = e.bristle_texture_for_measure();
        let mut t = Trail::default();
        let (cx, cy) = (2048.0f64, 2048.0f64);
        t.start_stroke(cx, cy, [0.2, 0.3, 0.4], TrailMode::Paint);
        // O espaçamento do produto no Wet Paint (0,025 do diâmetro).
        let spacing = 0.025 * 2.0 * r;
        t.on_segment(spacing * 4.0, spacing);
        let g = e.active_grid_mut();

        // Aquece: a 1ª chamada paga o `fit_to` (alocação das 6 superfícies) e
        // o first-touch delas. O artista paga isso uma vez por traço, e medi-lo
        // junto do resto atribuiria a alocação ao laço.
        let _ = t.accumulate_paint(g, &p, &tex, &dab_at(cx, cy, r), false);
        let _ = t.transfer_paint(g, &p);

        let (mut acc_ms, mut tra_ms) = (0.0f64, 0.0f64);
        for k in 0..DABS {
            let x = cx + f64::from(k) * spacing;
            let t0 = Instant::now();
            let full = t.accumulate_paint(g, &p, &tex, &dab_at(x, cy, r), false);
            acc_ms += t0.elapsed().as_secs_f64() * 1e3;
            if full {
                let t1 = Instant::now();
                let _ = t.transfer_paint(g, &p);
                tra_ms += t1.elapsed().as_secs_f64() * 1e3;
            }
        }
        let total = acc_ms + tra_ms;
        let half = t.window_half_for_measure();
        // Normalizado pela PEGADA: `r²` é a área que um dab de raio `r` cobre.
        let per_r2 = total * 1e6 / (f64::from(DABS) * r * r);
        println!(
            "    {r:>5.0}p {:>7} {acc_ms:>11.3} {tra_ms:>11.3} {total:>11.3}   {per_r2:>9.2} {:>9.1}%",
            half * 2 + 1,
            100.0 * tra_ms / total.max(1e-9),
        );
        rows.push((r, total, per_r2));
    }

    let first = rows[0].2;
    let last = rows[rows.len() - 1].2;
    println!(
        "\n    Leitura: `ns/r2` PLANO = o custo e a PEGADA, que e a forma correta e nao\n    \
         deixa wave. Ele vai de {first:.2} (raio {:.0}) a {last:.2} (raio {:.0}) = {:.2}x.\n    \
         A nota do doc 28 §5.50 media 1 : 1,8 : 2,3 nos raios 100/200/300 — SUB-linear —\n    \
         e atribuia isso ao cap. Com o cap removido, esta tabela diz o que sobrou.",
        rows[0].0,
        rows[rows.len() - 1].0,
        last / first.max(1e-9),
    );
}

/// **O PREÇO DA FRONTEIRA** — o produto NÃO usa o falloff do motor: ele passa a
/// silhueta do Painter por um `&mut dyn FnMut(i32,i32) -> f64`, chamado uma vez
/// por pixel da caixa do dab (`for_each_stamp_pixel_shaped`).
///
/// ⚠️ **A ablação é entre duas PORTAS reais** (`accumulate_paint` ×
/// `accumulate_paint_shaped`), não entre o produto e um laço meu — e a `sil`
/// desta sonda faz a MESMA aritmética que o ramo interno (`radial_falloff` na
/// distância normalizada), então a diferença é a INDIREÇÃO e mais nada. Uma
/// `sil` mais cara mediria a `sil`, não a fronteira.
///
/// A caixa de um disco tem `4r²` e o disco `πr²`, então **21% das chamadas
/// caem fora do pincel e devolvem zero** — elas pagam a chamada virtual do
/// mesmo jeito. Se a indireção for material, é aqui que aparece.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_hosts_silhouette_costs_over_the_engines_own() {
    const DABS: u32 = 24;

    println!("\n  O FALLOFF DO MOTOR x A SILHUETA DO HOST PELA PORTA SHAPED\n");
    println!(
        "    {:>6} {:>14} {:>14} {:>10}   {:>14}",
        "raio", "motor ms", "host ms", "razao", "por dab"
    );

    for r in [100.0f64, 200.0, 400.0] {
        let mut row = [0.0f64; 2];
        for (slot, shaped) in [(0usize, false), (1, true)] {
            let mut e = Engine::new(SIDE, SIDE);
            let p = e.sim.gather_params(&e.tuning);
            let tex: Vec<f32> = e.bristle_texture_for_measure();
            let mut t = Trail::default();
            let (cx, cy) = (2048.0f64, 2048.0f64);
            t.start_stroke(cx, cy, [0.2, 0.3, 0.4], TrailMode::Paint);
            let spacing = 0.025 * 2.0 * r;
            t.on_segment(spacing * 4.0, spacing);
            let g = e.active_grid_mut();
            let inv_r = 1.0 / r;
            let _ = t.accumulate_paint(g, &p, &tex, &dab_at(cx, cy, r), false);
            let _ = t.transfer_paint(g, &p);

            for k in 0..DABS {
                let x = cx + f64::from(k) * spacing;
                let d = dab_at(x, cy, r);
                let t0 = Instant::now();
                let full = if shaped {
                    // A MESMA aritmética do ramo interno, atravessando o `dyn`.
                    let mut sil = |px: i32, py: i32| -> f64 {
                        let dx = (f64::from(px) - d.x) * inv_r;
                        let dy = (f64::from(py) - d.y) * inv_r;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 1.0 {
                            0.0
                        } else {
                            ph2d_wet_paint::brush::radial_falloff(dist, d.hardness)
                        }
                    };
                    t.accumulate_paint_shaped(g, &p, &tex, &d, false, &mut sil, None)
                } else {
                    t.accumulate_paint(g, &p, &tex, &d, false)
                };
                row[slot] += t0.elapsed().as_secs_f64() * 1e3;
                if full {
                    let _ = t.transfer_paint(g, &p);
                }
            }
        }
        println!(
            "    {r:>5.0}p {:>13.3} {:>13.3} {:>9.2}x   {:>13.3}ms",
            row[0],
            row[1],
            row[1] / row[0].max(1e-9),
            row[1] / f64::from(DABS),
        );
    }
    println!(
        "\n    Leitura: a razao e o preco de a silhueta ser do HOST. Se for ~1,0 a fronteira\n    \
         e gratis e o custo do deposito sao os PIXELS; se for grande, a chamada virtual por\n    \
         pixel e o alvo — e o produto paga ainda mais que isto, porque a `sil` dele faz\n    \
         `falloff_t` + `silhouette_at` (+ Shape image, + sub-amostras de AA) por chamada."
    );
}
