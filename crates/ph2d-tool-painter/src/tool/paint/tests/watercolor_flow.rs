//! **O Ragged Edge com FLUXO e PAPEL** (2026-10-02): o padrão que desenha a borda, a parte dela que segue
//! o dente do Paper, e a faixa do Bleed nos vales. Plano e medições: BUGS_painter #31.

use super::*;
use super::super::media::PaintMedia;
use ph2d_painter_brush::TextureKind;

/// FNV-1a de 64 bits — estável entre versões do Rust (o `DefaultHasher` não promete isso).
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Um traço de aquarela horizontal no estado de fábrica do app. `ajusta` mexe no pincel antes do traço.
fn traco(w: u32, y: f32, ajusta: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * w * 4) as usize], w, w);
    t.set_paint_media(PaintMedia::Watercolor);
    t.paint.brush.color = [0.2, 0.3, 0.8];
    t.paint.brush.radius_px = 14.0;
    ajusta(&mut t);
    risca(&mut t, y, w);
    t
}

fn risca(t: &mut PainterTool, y: f32, w: u32) {
    let x1 = w as f32 - 24.0;
    t.on_canvas_pointer(cp([24.0, y], PointerPhase::Down));
    let mut x = 24.0;
    while x < x1 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        frame(t);
    }
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    frame(t);
}

/// As configurações que o caminho Classic tem de reproduzir AO BYTE.
fn configs_classicas() -> Vec<(&'static str, Box<dyn Fn(&mut PainterTool)>)> {
    vec![
        ("fábrica", Box::new(|_| {})),
        ("ragged 24", Box::new(|t| t.paint.brush.warp = 24.0)),
        ("ragged 48 sem AA", Box::new(|t| {
            t.paint.brush.warp = 48.0;
            t.paint.brush.smooth_edges = false;
        })),
        ("ragged 24 Paper Rough", Box::new(|t| {
            t.paint.brush.warp = 24.0;
            t.set_brush_paper_kind(TextureKind::PaperRough.to_u8());
        })),
        ("ladrilho", Box::new(|t| {
            t.paint.brush.warp = 24.0;
            t.paint.tiling = [true, true];
        })),
    ]
}

/// ⭐ **O Classic e o Paper Edge a 0 são o caminho de HOJE, ao byte** — gravado ANTES da wave
/// (commit `b03bbfba2`). Inclui a sessão de DOIS donos com Ragged diferentes (o campo de estilo da #18).
///
/// **Mutação que tem de sangrar:** trocar o `warp_offset` do Classic pela tabela dos padrões.
#[test]
fn o_classic_e_o_paper_edge_zero_sao_o_byte_de_hoje() {
    let mut lidos = Vec::new();
    for (nome, ajusta) in configs_classicas() {
        lidos.push((nome, fnv(&traco(128, 64.0, ajusta).canvas_rgba)));
    }
    let mut dois = traco(128, 52.0, |t| t.paint.brush.warp = 8.0);
    dois.paint.brush.warp = 40.0;
    risca(&mut dois, 76.0, 128);
    lidos.push(("dois donos", fnv(&dois.canvas_rgba)));
    for (nome, h) in &lidos {
        eprintln!("FLOW-BASE {nome}: {h:#018x}");
    }
    let esperado: [(&str, u64); 6] = BASE;
    for ((nome, h), (_, e)) in lidos.iter().zip(esperado) {
        assert_eq!(*h, e, "{nome}: o caminho Classic mudou de byte");
    }
}

const BASE: [(&str, u64); 6] = [
    ("fábrica", 0xa571_f120_d9eb_f288),
    ("ragged 24", 0x1860_91ef_4ed9_b617),
    ("ragged 48 sem AA", 0xac14_b804_34ed_b932),
    ("ragged 24 Paper Rough", 0x321f_923b_7fd3_b060),
    ("ladrilho", 0x8b25_8d8a_18ec_fef2),
    ("dois donos", 0xb52d_61bf_7582_3c3b),
];

