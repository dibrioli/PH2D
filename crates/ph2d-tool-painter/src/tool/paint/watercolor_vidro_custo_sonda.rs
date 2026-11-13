//! **Sonda (ignorada): o custo da cura do vidro do escorrido** (`69158c3890`, ADR-0204) a 4096². O Wet sobre a aguada
//! em papel de cor, a água a correr depois do pen-up (o escorrido escreve o vidro), e então o pen-down seguinte e o
//! Ctrl+Z — duas telas no MESMO processo, intercaladas: `cura` (o produto) × `antes` (o vidro elidido não testemunha,
//! [`SEM_SEGURAR_O_VIDRO`]: o custo de antes da cura, e a resposta errada). Resumo = o MÍNIMO.
//!
//! ```text
//! bash scripts/cargo-test-narrow.sh ph2d-tool-painter sonda_custo_da_cura_do_vidro -- --ignored --nocapture
//! ```

use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use crate::undo::absorb::{ROTA_DO_VIDRO, RotaDoVidro, VIDRO_COMPOSTO, VIDRO_SEGURADO};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use std::time::Instant;

const LADO: u32 = 4096;
const DT: f32 = 1000.0 / 60.0;

fn ev(t: &mut PainterTool, pos: [f32; 2], phase: PointerPhase) {
    t.on_canvas_pointer(CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    });
}

fn tick(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(DT);
        let _ = t.take_preview_arc();
    }
}

/// Um traço de `a` a `b` em passos de 4 px, um tick a cada 16 eventos (o quadro do app).
fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    ev(t, a, PointerPhase::Down);
    resto_do_traco(t, a, b);
}

fn resto_do_traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 4.0).ceil() as usize;
    for s in 1..=n {
        let f = s as f32 / n as f32;
        ev(
            t,
            [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
            PointerPhase::Move,
        );
        if s % 16 == 0 {
            tick(t, 1);
        }
    }
    ev(t, b, PointerPhase::Up);
}

fn tela() -> PainterTool {
    crate::undo::window::set_vidro_pelo_journal(true);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.papel = Some([200, 170, 120]);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(60.0);
    for k in 0..4 {
        let y = 1200.0 + 500.0 * k as f32;
        traco(&mut t, [600.0, y], [3500.0, y + 200.0]);
        tick(&mut t, 3);
    }
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8([200, 40, 30]);
    t.set_brush_size_px(40.0);
    t
}

/// `(nome da parte, ms)` de um gesto, com o total.
fn partes(f: impl FnOnce()) -> (f64, Vec<(&'static str, f64)>) {
    crate::sonda_partes::liga();
    let t0 = Instant::now();
    f();
    let total = t0.elapsed().as_secs_f64() * 1e3;
    let v = crate::sonda_partes::tira()
        .into_iter()
        .map(|(n, ms, _)| (n, ms))
        .collect();
    (total, v)
}

fn parte(v: &[(&'static str, f64)], nome: &str) -> f64 {
    v.iter().filter(|(n, _)| *n == nome).map(|(_, ms)| ms).sum()
}

#[derive(Default)]
struct Coluna {
    pen_down: Vec<f64>,
    pd_reset: Vec<f64>,
    pd_absorcao: Vec<f64>,
    desfazer: Vec<f64>,
    dz_reset: Vec<f64>,
    dz_absorcao: Vec<f64>,
    seguradas: u32,
}

thread_local! {
    static PARTES_FINAIS: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn min(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

fn med(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    s[s.len() / 2]
}

#[test]
#[ignore = "sonda de custo a 4096² — corre à mão"]
fn sonda_custo_da_cura_do_vidro() {
    const RONDAS: usize = 6;
    const ROTAS: [(RotaDoVidro, &str); 3] = [
        (RotaDoVidro::Composta, "composta"),
        (RotaDoVidro::Segurada, "segurada"),
        (RotaDoVidro::Antes, "antes"),
    ];
    let mut telas = [tela(), tela(), tela()];
    let mut col = [Coluna::default(), Coluna::default(), Coluna::default()];
    for r in 0..RONDAS {
        // Rodado: quem vai primeiro alterna.
        for d in 0..3 {
            let i = (r + d) % 3;
            let t = &mut telas[i];
            ROTA_DO_VIDRO.with(|c| c.set(ROTAS[i].0));
            let y = 1100.0 + 380.0 * (r % 6) as f32;
            traco(t, [700.0, y], [3300.0, y + 300.0]);
            tick(t, 30); // o escorrido a correr: escreve o vidro depois do commit
            let a = [900.0, y + 120.0];
            let disparos = || {
                VIDRO_SEGURADO.with(std::cell::Cell::get)
                    + VIDRO_COMPOSTO.with(std::cell::Cell::get)
            };
            let antes = disparos();
            let (pd, v) = partes(|| ev(t, a, PointerPhase::Down));
            col[i].pen_down.push(pd);
            col[i].pd_reset.push(parte(&v, "vidro_do_reset"));
            col[i].pd_absorcao.push(parte(&v, "absorcao"));
            if r == RONDAS - 1 {
                eprintln!("partes do pen-down {}: {v:?}", ROTAS[i].1);
                PARTES_FINAIS
                    .with(|p| p.borrow_mut().push_str(&format!("{}: {v:?}\n", ROTAS[i].1)));
                let cx = crate::undo_delta::composto::SONDA_CAIXAS.with(std::cell::Cell::get);
                PARTES_FINAIS.with(|p| {
                    p.borrow_mut().push_str(&format!(
                        "  caixas (lidas, topo, journal, diferem) {cx:?}\n"
                    ))
                });
            }
            resto_do_traco(t, a, [3100.0, y + 160.0]);
            tick(t, 30);
            let (dz, v) = partes(|| assert!(t.undo_last(), "desfazer"));
            col[i].desfazer.push(dz);
            col[i].dz_reset.push(parte(&v, "vidro_do_reset"));
            col[i].dz_absorcao.push(parte(&v, "absorcao"));
            col[i].seguradas += disparos() - antes;
            assert!(t.redo_last(), "refazer");
            ROTA_DO_VIDRO.with(|c| c.set(RotaDoVidro::Composta));
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let mut s = format!(
        "CURA DO VIDRO a {LADO}² — {RONDAS} rondas, load {}\n",
        load.trim()
    );
    for (i, (_, nome)) in ROTAS.iter().enumerate() {
        let c = &col[i];
        s += &format!(
            "{nome:>8}: pen-down min {:.2} med {:.2} (vidro_do_reset {:.2} · absorção {:.2}) | Ctrl+Z min {:.2} med {:.2} \
             (vidro_do_reset {:.2} · absorção {:.2}) | disparos {}\n",
            min(&c.pen_down),
            med(&c.pen_down),
            min(&c.pd_reset),
            min(&c.pd_absorcao),
            min(&c.desfazer),
            med(&c.desfazer),
            min(&c.dz_reset),
            min(&c.dz_absorcao),
            c.seguradas
        );
    }
    s += &PARTES_FINAIS.with(|p| p.borrow().clone());
    // O `cargo-test-narrow` esconde o stdout do verde: o resumo vai também a um ficheiro.
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/sondas");
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(format!("{dir}/cura_do_vidro.txt"), &s);
    print!("{s}");
    assert!(
        col[0].seguradas > 0 && col[1].seguradas > 0,
        "controlo: a composta ou a segurada nunca disparou — a sonda mede outro caminho"
    );
}
