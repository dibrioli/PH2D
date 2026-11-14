//! ⭐ **O CUSTO no caminho do app** (`docs/Painter/Pinceis/exames/custo_no_app.md`): o Filbert de fábrica deixava o
//! app a 12 FPS e a bancada lia-o a 9,7 ms. A bancada andava 9 px por quadro; o app anda 30–135, e o custo de um
//! pincel de cápsula é por px. A 64 px a caixa do dab (4 225 células) ficava abaixo do piso de células do depósito
//! (`ph2d_wet_paint::par::MIN_CELLS_DEPOSIT`, medido com a queda redonda do motor, `~7 ns` por célula) e o depósito
//! corria em SÉRIE — mas a silhueta do Filbert (as cerdas, a ponta desfiada, o relevo e a cor de cada fio) custa
//! `~120 ns` por célula: `512 µs` por sub-dab contra `76 µs` em paralelo, `60 ms` por quadro a 90 px.
//!
//! O gate: no traço do app, o depósito de cada pincel PESADO de fábrica pela rota do produto
//! ([`ph2d_wet_paint::par::PISO_NS_DEPOSITO`]) custa o da rota paralela (o piso `0`) e fica bem abaixo do da rota de
//! antes (o piso `u64::MAX`: abaixo das células, sempre em série) — as três no MESMO processo, intercaladas, o mínimo
//! das medianas. As rotas dão os MESMOS bytes (`deposit_rows`); só o relógio as distingue. A 1.ª metade só reprova
//! pelo defeito; a 2.ª (que apanha o roteador partido, onde produto e «paralelo» passam ambos a série) pode
//! aproximar-se do limiar com a máquina cheia — família `FLAKES_DE_CARGA`.
//!
//! Red-first (o piso do produto posto a `u64::MAX`, a rota de antes): Filbert `52,87` contra `23,53` ms (`2,25×`).
//!
//! A 2.ª metade pede `série > 1,2 × produto` (era `1,5×`): ela separa o roteador partido (produto = série, razão
//! `~1,0`) do são — não mede quanto o pool ganha, que é o custo por célula do pincel. A missão 4K (cerdas por
//! referência, sem `map` no desvio) baixou esse custo e as DUAS rotas ganharam. Medido 14/11 (integ 13/11, smoke,
//! binários do main `4f46a3573c` e da integ intercalados, 2 rodadas, load 22–36, ms/quadro do Filbert):
//!
//! | árvore | produto | paralelo | série | série ÷ produto |
//! |---|---|---|---|---|
//! | main | 57,7 · 58,1 | 82,7 · 53,3 | 93,2 · 99,2 | 1,62 · 1,71 |
//! | integ | 39,1 · 38,6 | 39,6 · 37,1 | 53,5 · 53,3 | 1,37 · 1,38 |

use crate::exame_bancada::pinceis;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use std::sync::atomic::Ordering;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A mediana dos ms de ponteiro por quadro de um traço do app: 3 eventos de 30 px por quadro (90 px, a caneta a
/// ~180 Hz num gesto rápido), `quadros` quadros, sem ritmo de parede (só o ponteiro conta).
fn ms_por_quadro(t: &mut PainterTool, y: f32, quadros: usize) -> f64 {
    let mut x = 60.0f32;
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    let mut ms = Vec::with_capacity(quadros);
    for _ in 0..quadros {
        let t0 = Instant::now();
        for _ in 0..3 {
            x += 30.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

#[test]
fn o_deposito_de_um_pincel_pesado_de_fabrica_vai_pela_rota_que_paga() {
    let piso = &ph2d_wet_paint::par::PISO_NS_DEPOSITO;
    let do_produto = piso.load(Ordering::Relaxed);
    for p in pinceis("Filbert|Express Oil") {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 2400 * 900 * 4], 2400, 900);
        p.aplica(&mut t, false);
        let _ = ms_por_quadro(&mut t, 60.0, 3);
        // [produto, paralelo, série]: o piso de cada rota e o mínimo das medianas dela.
        let pisos = [do_produto, 0, u64::MAX];
        let mut ms = [f64::INFINITY; 3];
        for r in 0..2 {
            for k in 0..3 {
                let v = (k + r) % 3;
                piso.store(pisos[v], Ordering::Relaxed);
                let y = 120.0 + 110.0 * (3 * r + k) as f32;
                ms[v] = ms[v].min(ms_por_quadro(&mut t, y, 8));
            }
        }
        piso.store(do_produto, Ordering::Relaxed);
        let [produto, paralelo, serie] = ms;
        println!(
            "[custo] {}: produto {produto:.2} · paralelo {paralelo:.2} · antes {serie:.2} ms/quadro",
            p.nome
        );
        assert!(
            produto < 1.6 * paralelo && serie > 1.2 * produto,
            "{}: o traço do app custa {produto:.2} ms por quadro pela rota do produto, {paralelo:.2} pela paralela e \
             {serie:.2} pela de antes (em série) — o depósito de um pincel pesado voltou a correr em série",
            p.nome
        );
    }
}

/// A mediana dos ms por quadro à velocidade da caneta do DONO (Bug #84, `PH2D_PAINT_PERF` 10/11): 9 eventos de 40 px por
/// quadro (~4 000 px/s) e a drenagem da prévia uma vez por quadro, como a ponte do app (o plano do traço compõe aí).
fn ms_por_quadro_do_dono(t: &mut PainterTool, y: f32, quadros: usize) -> f64 {
    let mut x = 40.0f32;
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    let mut ms = Vec::with_capacity(quadros);
    for _ in 0..quadros {
        let t0 = Instant::now();
        for _ in 0..9 {
            x += 40.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        let _ = t.take_preview_dirty();
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

/// ⭐ **O Wet Wash Mix de fábrica à velocidade do dono vai pela meia definição** (Bug #84, cura 6): o pincel de mistura
/// que o gate acima não tinha na população (o app a 6 fps com ele verde). O preset de fábrica × o mesmo com a película
/// à definição inteira, intercalados no MESMO processo, o mínimo das medianas — uma RAZÃO, que a carga não move.
/// Medido 10/11 (`sonda_custo_no_app`, 360 px/quadro): 24,1 × 43,5 ms (0,55).
#[test]
fn o_wet_wash_mix_a_velocidade_do_dono_vai_pela_meia_definicao() {
    for p in pinceis("Wet Wash Mix") {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 3800 * 700 * 4], 3800, 700);
        p.aplica(&mut t, false);
        t.set_brush_size_px(30.0);
        t.set_compor_por_quadro(true);
        let _ = ms_por_quadro_do_dono(&mut t, 60.0, 2);
        // [fábrica, inteira]
        let mut ms = [f64::INFINITY; 2];
        for r in 0..3 {
            for k in 0..2 {
                let v = (k + r) % 2;
                t.set_mistura_meia_definicao(v == 0);
                let y = 160.0 + 90.0 * (2 * r + k) as f32;
                ms[v] = ms[v].min(ms_por_quadro_do_dono(&mut t, y, 8));
            }
        }
        let [fabrica, inteira] = ms;
        println!(
            "[custo] {}: fábrica {fabrica:.2} · inteira {inteira:.2} ms/quadro (9 × 40 px)",
            p.nome
        );
        assert!(
            fabrica < 0.8 * inteira,
            "{}: à velocidade do dono o preset de fábrica custa {fabrica:.2} ms por quadro e a película inteira \
             {inteira:.2} — a mistura deixou de correr a meia definição (`MisturaOpaca::meia_definicao`, Bug #84)",
            p.nome
        );
    }
}
