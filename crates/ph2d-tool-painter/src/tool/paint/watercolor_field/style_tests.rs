//! **T1-41 (B04, DF-07): a 256.ª pincelada da aguada não reescreve o estilo das antigas.** Cada traço
//! numa célula própria (nunca sobre o vizinho), todos na MESMA sessão molhada, cada um com a sua cor:
//! a célula do traço `k` tem de continuar ao byte o que era logo depois do pen-up dele.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

const CEL: u32 = 16;
const COLS: u32 = 18;
const LADO: u32 = CEL * COLS;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Tela branca com o pincel de aguada desta prova (geometria FIXA: só a cor muda de traço a traço,
/// para que o máximo da sessão não alargue a janela e a única diferença seja o estilo por dono).
pub(super) fn aguada_em_celulas() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    let b = BrushSpec {
        radius_px: 2.5,
        hardness: 1.0,
        falloff: Falloff::Constant,
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 1.5,
        edge_gain: 1.0,
        edge_spread: 2.0,
        warp: 0.0,
        granulation: 0.0,
        wet_rewet: 0.0,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t
}

/// A cor do traço `k` (1-based): pares azuis, ímpares vermelhos, o verde anda — o 255.º e o 256.º
/// ficam longe um do outro.
fn cor(k: u32) -> [f32; 3] {
    let g = (k % 7) as f32 * 0.1;
    if k.is_multiple_of(2) {
        [0.1, g, 0.9]
    } else {
        [0.85, g, 0.1]
    }
}

fn centro(k: u32) -> (u32, u32) {
    let i = k - 1;
    ((i % COLS) * CEL + CEL / 2, (i / COLS) * CEL + CEL / 2)
}

/// O traço `k` (1-based) na célula dele: um risco curto vertical.
pub(super) fn traco(t: &mut PainterTool, k: u32) {
    t.paint.brush.color = cor(k);
    t.paint.brush_by_mode.fill(t.paint.brush);
    let (cx, cy) = centro(k);
    let (x, y) = (cx as f32, cy as f32);
    assert!(t.on_canvas_pointer(cp([x, y - 2.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    t.on_canvas_pointer(cp([x, y + 2.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([x, y + 2.0], PointerPhase::Up));
}

/// Os bytes RGBA da célula do traço `k`.
pub(super) fn celula(t: &PainterTool, k: u32) -> Vec<u8> {
    let (cx, cy) = centro(k);
    let (x0, y0) = (cx - CEL / 2, cy - CEL / 2);
    let mut v = Vec::with_capacity((CEL * CEL * 4) as usize);
    for y in y0..y0 + CEL {
        let i = ((y * LADO + x0) * 4) as usize;
        v.extend_from_slice(&t.canvas_rgba[i..i + (CEL * 4) as usize]);
    }
    v
}

pub(super) fn fnv(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| {
        (h ^ u64::from(x)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn delta(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

#[test]
fn o_256_o_traco_nao_reescreve_o_estilo_do_255_o() {
    const N: u32 = 300;
    let mut t = aguada_em_celulas();
    let mut feitas: Vec<Vec<u8>> = Vec::with_capacity(N as usize);
    let mut piores: Vec<(u32, u32, u8)> = Vec::new(); // (depois do traço, célula, Δ)
    for k in 1..=N {
        traco(&mut t, k);
        feitas.push(celula(&t, k));
        if k == 255 {
            eprintln!(
                "T1-41: depois de 255 traços, fnv(canvas) = {:#018x}",
                fnv(&t.canvas_rgba)
            );
        }
        if k >= 2 {
            for j in 1..k {
                let d = delta(&feitas[(j - 1) as usize], &celula(&t, j));
                if d > 0 {
                    piores.push((k, j, d));
                }
            }
        }
    }
    assert!(
        t.paint.wet_styles.table.len() >= 255,
        "a fixture tem de manter UMA sessão molhada (tabela {})",
        t.paint.wet_styles.table.len()
    );
    let pior = piores.iter().map(|p| p.2).max().unwrap_or(0);
    let primeiro = piores.first().copied();
    assert!(
        piores.is_empty(),
        "{} (traço, célula) mudaram depois do próprio pen-up; o primeiro {primeiro:?}, Δmax {pior}",
        piores.len()
    );
}

/// O ORÁCULO: o tecto `u8` antigo, verbatim ([`super::WetSessionStyles::teto_u8_antigo`]). Até 255
/// traços o dono `u16` dá o MESMO bit (tela e mapa de donos); no 256.º o oráculo repinta o 255.º —
/// prova de que a fixture tem o fenómeno. O pen-up pela UNIÃO da sessão: desde o B16.2 cont3 o pen-up
/// só recompõe o traço ⊕ pad, e a célula do 255.º (fora dela) guardaria o estilo reescrito latente.
#[test]
fn ate_255_tracos_o_dono_u16_e_o_u8_antigo_dao_o_mesmo_bit() {
    use crate::tool::paint::watercolor_render::COMMIT_PELA_UNIAO;
    let antes_uniao = COMMIT_PELA_UNIAO.replace(true);
    let mut novo = aguada_em_celulas();
    let mut velho = aguada_em_celulas();
    velho.paint.wet_styles.teto_u8_antigo = true;
    for k in 1..=255 {
        traco(&mut novo, k);
        traco(&mut velho, k);
        assert!(
            novo.canvas_rgba == velho.canvas_rgba,
            "traço {k}: a tela difere do caminho u8"
        );
        assert_eq!(
            novo.paint.wet_styles.owner, velho.paint.wet_styles.owner,
            "traço {k}: donos"
        );
    }
    let antes = celula(&velho, 255);
    traco(&mut novo, 256);
    traco(&mut velho, 256);
    COMMIT_PELA_UNIAO.set(antes_uniao);
    assert_eq!(celula(&novo, 255), antes, "o u16 mantém o 255.º");
    assert_ne!(
        celula(&velho, 255),
        antes,
        "o oráculo u8 repinta o 255.º (o defeito)"
    );
}
