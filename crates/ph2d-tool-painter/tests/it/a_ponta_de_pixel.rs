//! ⭐ **A ponta de PÍXEL** (L18, `docs/Painter/Pinceis/03_MOTORES.md` §4) pela porta do ponteiro e pela linha do
//! painel (`set_ponta`), no Digital: só 0 e 1 na camada, e a linha de 1 px pixel-perfect (a lei do Aseprite).

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{LinhaDaPonta, PaintMedia, PainterTool};

const W: usize = 200;
const H: usize = 120;
/// A queda «Constant» (`Falloff::Constant`): o disco duro, a dureza 1 da UI.
const DURA: u8 = 8;

/// Um traço em curva (meia onda) na camada TRANSPARENTE `W × H`, raio `r`, a ponta dura, vermelho; a tela.
fn traco(r: f32, pixel: f64) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_ponta(LinhaDaPonta::Pixel, pixel);
    t.set_brush_falloff(DURA);
    t.set_brush_size_px(r);
    t.set_brush_color_srgb8([255, 0, 0]);
    let cp = |k: f32, phase| CanvasPointer {
        pos: [12.0 + 1.7 * k, 60.0 + 35.0 * (k * 0.06).sin() + 0.037 * k],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(0.0, PointerPhase::Down));
    for k in 1..=100 {
        t.on_canvas_pointer(cp(k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(100.0, PointerPhase::Up));
    t.run_full().0
}

/// `(alfa fraccionário, opacos de outra cor, opacos)`.
fn fraccoes(px: &[u8]) -> (usize, usize, usize) {
    let (mut frac, mut cor, mut tinta) = (0, 0, 0);
    for p in px.chunks_exact(4) {
        match p[3] {
            0 => {}
            255 => {
                tinta += 1;
                cor += usize::from(p[..3] != [255, 0, 0]);
            }
            _ => frac += 1,
        }
    }
    (frac, cor, tinta)
}

/// ⭐ **Só 0 e 1 na camada**: com a ponta de píxel o traço pinta píxeis INTEIROS da cor do pincel, em todo tamanho.
/// ⚠️ MEDIDO (20/10): a ponta dura de HOJE já é binária no alfa (0 fracções em r 1–8 na camada transparente) —
/// a falta da L18 não é o AA, é o ALINHAMENTO (`a_largura_da_ponta_de_pixel_e_inteira`) e o 1 px.
#[test]
fn so_valores_zero_e_um_na_camada() {
    for r in [0.5, 1.0, 1.5, 3.5, 8.0] {
        for modo in [1.0, 2.0] {
            let (frac, cor, tinta) = fraccoes(&traco(r, modo));
            assert!(tinta > 50, "raio {r} modo {modo}: pintou só {tinta} px");
            assert_eq!((frac, cor), (0, 0), "raio {r} modo {modo}: fracções/cores fora");
        }
    }
}

/// Um recto quase horizontal (de y 40,0 a y 47,0 em 160 px: o centro corre todas as fracções de píxel), raio `r`.
fn recto(r: f32, pixel: f64) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_ponta(LinhaDaPonta::Pixel, pixel);
    t.set_brush_falloff(DURA);
    t.set_brush_size_px(r);
    let cp = |k: f32, phase| CanvasPointer {
        pos: [20.0 + 1.6 * k, 40.0 + 0.07 * k],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(0.0, PointerPhase::Down));
    for k in 1..=100 {
        t.on_canvas_pointer(cp(k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(100.0, PointerPhase::Up));
    t.run_full().0
}

/// As alturas pintadas de cada coluna do miolo do recto (x 40..160).
fn alturas(px: &[u8]) -> Vec<usize> {
    (40..160)
        .map(|x| (0..H).filter(|y| px[(y * W + x) * 4 + 3] > 0).count())
        .collect()
}

/// ⭐ **A largura da ponta de píxel é inteira**: num recto que corre todas as fracções de píxel, nenhuma coluna fica
/// abaixo do diâmetro e só o DEGRAU da recta soma uma (`D` ou `D + 1`); a linha de 1 px perfeita tem 1 em toda
/// coluna. Vermelho hoje: a ponta dura cai entre píxeis e a coluna AFINA (2 → 1 a raio 1; 3 → 2 a raio 1,5) — e o
/// 1 px nem existe (o tamanho mínimo da ponta com AA é 2).
#[test]
fn a_largura_da_ponta_de_pixel_e_inteira() {
    let um = alturas(&recto(0.5, 2.0));
    assert!(um.iter().all(|&a| a == 1), "1 px perfeito: {um:?}");
    let hoje_um = alturas(&recto(0.5, 0.0));
    assert!(hoje_um.iter().filter(|&&a| a >= 2).count() > 60, "o controlo: hoje o «1 px» já pinta 1? {hoje_um:?}");
    for (r, d) in [(1.0, 2), (1.5, 3), (2.5, 5)] {
        let hoje = alturas(&recto(r, 0.0));
        let px = alturas(&recto(r, 1.0));
        let (lo, hi) = (*px.iter().min().unwrap(), *px.iter().max().unwrap());
        println!("[pixel] r {r}: hoje {:?} · pixel {lo}..{hi}", (hoje.iter().min(), hoje.iter().max()));
        assert!(lo == d && hi <= d + 1, "r {r}: colunas {lo}..{hi} ≠ {d}..{}", d + 1);
        assert!(hoje.iter().any(|&a| a < d), "o controlo: hoje nenhuma coluna afina abaixo de {d} (r {r})");
    }
}

fn pintados(px: &[u8]) -> Vec<bool> {
    px.chunks_exact(4).map(|p| p[3] > 0).collect()
}

/// As quinas de L: um píxel com um vizinho de lado na horizontal e outro na vertical que se tocam na diagonal.
fn quinas(s: &[bool]) -> usize {
    let at = |x: i64, y: i64| {
        x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H && s[y as usize * W + x as usize]
    };
    (0..H as i64)
        .flat_map(|y| (0..W as i64).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            at(x, y)
                && [(-1, -1), (-1, 1), (1, -1), (1, 1)]
                    .iter()
                    .any(|&(dx, dy)| at(x + dx, y) && at(x, y + dy))
        })
        .count()
}

/// As componentes 8-conexas.
fn componentes(s: &[bool]) -> usize {
    let mut visto = vec![false; s.len()];
    let mut n = 0;
    for i in 0..s.len() {
        if !s[i] || visto[i] {
            continue;
        }
        n += 1;
        let mut pilha = vec![i];
        visto[i] = true;
        while let Some(j) = pilha.pop() {
            let (x, y) = ((j % W) as i64, (j / W) as i64);
            for (dx, dy) in (-1..=1).flat_map(|a| (-1..=1).map(move |b| (a, b))) {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= W as i64 || ny >= H as i64 {
                    continue;
                }
                let k = ny as usize * W + nx as usize;
                if s[k] && !visto[k] {
                    visto[k] = true;
                    pilha.push(k);
                }
            }
        }
    }
    n
}

/// ⭐ **A linha de 1 px é pixel-perfect** (o Aseprite): uma só componente 8-conexa e ZERO quinas de L (sem degraus
/// duplos). O controlo: a mesma linha só ALINHADA tem quinas — é o pixel-perfect que as tira, sem píxel novo.
#[test]
fn a_linha_de_um_pixel_e_pixel_perfect() {
    let alinhada = pintados(&traco(0.5, 1.0));
    let perfeita = pintados(&traco(0.5, 2.0));
    assert_eq!(componentes(&alinhada), 1, "a alinhada partiu-se");
    assert_eq!(componentes(&perfeita), 1, "a perfeita partiu-se");
    let (qa, qp) = (quinas(&alinhada), quinas(&perfeita));
    println!("[pixel] quinas alinhada {qa} · perfeita {qp}");
    assert!(qa > 5, "o controlo: a alinhada tem só {qa} quinas");
    assert_eq!(qp, 0, "a perfeita tem {qp} quinas (degraus duplos)");
    assert!(perfeita.iter().zip(&alinhada).all(|(p, a)| !*p || *a));
    // O 1 px é um píxel de largura: a coluna do meio tem 1 píxel pintado.
    let col = (0..H).filter(|y| perfeita[y * W + 100]).count();
    assert_eq!(col, 1, "a coluna do meio tem {col} píxeis");
}

/// O neutro: a fábrica é a ponta desligada (o carimbo de hoje ao bit: `os_outros_meios_nao_mudam`).
#[test]
fn a_ponta_desligada_e_o_carimbo_de_hoje() {
    let mut t = PainterTool::default();
    t.set_ponta(LinhaDaPonta::Pixel, 0.0);
    assert_eq!(t.ponta(), ph2d_tool_painter::PONTA_DE_FABRICA);
    assert_eq!(PainterTool::default().ponta(), ph2d_tool_painter::PONTA_DE_FABRICA);
}
