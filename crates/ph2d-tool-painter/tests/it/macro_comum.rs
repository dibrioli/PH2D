//! O que os gates das macros partilham: gravar quadro a quadro, tocar pelo `MacroPlayer`, comparar.

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::PaintMedia;
use std::time::{Duration, Instant};
use ph2d_macro::{MacroDoc, MacroRecorder, RefRect, TrackId};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::FAMILIA;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;

/// A gravação do lado de quem chama: o tempo é do quadro (`16,666 ms`), nunca do relógio de parede.
pub struct Gravador {
    rec: MacroRecorder,
    trilha: TrackId,
    pub relogio_us: u64,
}

impl Gravador {
    /// Arma `t` e abre a trilha com o estado inicial.
    pub fn arma(t: &mut PainterTool) -> Self {
        let (inicial, (w, h)) = t.macro_grava().expect("arma no Wet Paint");
        let mut rec = MacroRecorder::new(
            RefRect {
                w: w as f32,
                h: h as f32,
            },
            0,
        );
        let trilha = rec.open_track(FAMILIA, 0, inicial);
        Self {
            rec,
            trilha,
            relogio_us: 0,
        }
    }

    /// Fecha um quadro: os passos dele entram com o tempo do quadro.
    pub fn quadro(&mut self, t: &mut PainterTool) {
        for c in t.macro_passos() {
            self.rec
                .push(self.trilha, self.relogio_us, c)
                .expect("tempo crescente");
        }
        self.relogio_us += 16_666;
    }

    pub fn para(mut self, t: &mut PainterTool) -> MacroDoc {
        for c in t.macro_para() {
            self.rec
                .push(self.trilha, self.relogio_us, c)
                .expect("tempo crescente");
        }
        self.rec.finish()
    }
}

pub fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Toca a trilha 0 do princípio ao fim, pelo `MacroPlayer`, num tool próprio no estado inicial.
pub fn toca(doc: &MacroDoc) -> Result<TocadorDoPainter, String> {
    let tr = &doc.tracks[0];
    let toc = TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
        .map_err(|e| format!("{e:?}"))?;
    toca_com(doc, toc)
}

pub fn toca_com(doc: &MacroDoc, mut toc: TocadorDoPainter) -> Result<TocadorDoPainter, String> {
    let mut player = ph2d_macro::MacroPlayer::new(doc);
    player
        .advance_to(doc, doc.duration(), |_, _, ev| toc.aplica(&ev.payload))
        .map_err(|e| format!("{e:?}"))?;
    Ok(toc)
}

pub fn texels_diferentes(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len(), "telas de tamanhos diferentes");
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// Seis pinceladas com esperas REAIS e a água no worker — a gravação do produto.
pub fn grava_no_worker(n: u32) -> MacroDoc {
    const ROTEIRO: [([f32; 2], [f32; 2], [u8; 3]); 6] = [
        ([0.15, 0.4], [0.85, 0.4], [200, 50, 30]),
        ([0.2, 0.2], [0.8, 0.7], [30, 80, 200]),
        ([0.5, 0.15], [0.5, 0.85], [230, 200, 30]),
        ([0.15, 0.6], [0.85, 0.62], [40, 160, 70]),
        ([0.3, 0.8], [0.7, 0.3], [150, 40, 160]),
        ([0.25, 0.5], [0.75, 0.5], [20, 20, 20]),
    ];
    const ESPERAS_MS: [u64; 6] = [150, 300, 120, 250, 90, 200];
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(30.0 * n as f32 / 1024.0);
    let mut g = Gravador::arma(&mut t);
    let s = n as f32;
    const QUADRO: Duration = Duration::from_micros(16_666);
    let quadro = |t: &mut PainterTool, g: &mut Gravador, ev: &[_]| {
        let a = Instant::now();
        for &e in ev {
            t.on_canvas_pointer(e);
        }
        let _ = t.take_preview_arc();
        t.on_tick(1000.0 / 60.0);
        g.quadro(t);
        if let Some(r) = QUADRO.checked_sub(a.elapsed()) {
            std::thread::sleep(r);
        }
    };
    for (k, (de, ate, cor)) in ROTEIRO.iter().enumerate() {
        t.set_brush_color_srgb8(*cor);
        let p = |i: u32| {
            let f = i as f32 / 30.0;
            [
                s * (de[0] + (ate[0] - de[0]) * f),
                s * (de[1] + (ate[1] - de[1]) * f),
            ]
        };
        quadro(&mut t, &mut g, &[cp(p(0), PointerPhase::Down)]);
        for q in 0..10 {
            let ev: Vec<_> = (1..=3)
                .map(|m| cp(p(q * 3 + m), PointerPhase::Move))
                .collect();
            quadro(&mut t, &mut g, &ev);
        }
        quadro(&mut t, &mut g, &[cp(p(30), PointerPhase::Up)]);
        let fim = Instant::now() + Duration::from_millis(ESPERAS_MS[k]);
        while Instant::now() < fim {
            quadro(&mut t, &mut g, &[]);
        }
    }
    g.para(&mut t)
}

