//! Sonda da fila 44, item 8: **o pincel composto deposita RELEVO no Impasto?**

use super::composite::{CompositeLayer, CompositeOp, N_CAMADAS};
use super::*;
use ph2d_painter_brush::Falloff;

const S: u32 = 128;

pub(super) fn tela(camadas: &[CompositeOp]) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 0.0,
        falloff: Falloff::Smooth,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_brush_impasto(true);
    t.set_brush_impasto_depth(1.0);
    t.set_brush_falloff(Falloff::Smooth as u8);
    t.paint.composite_enabled = !camadas.is_empty();
    for pos in 0..N_CAMADAS {
        t.paint.composite[pos] = camadas
            .get(pos)
            .map_or_else(CompositeLayer::default, |op| CompositeLayer::nova(*op));
    }
    t
}

pub(super) fn traco(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    let mut x = 20.0;
    while x < 108.0 {
        x += 3.0;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
}

pub(super) fn relevo(t: &PainterTool) -> (f32, f32, usize) {
    let layer = t.layers.active().expect("camada");
    let Some(h) = t.heights.get(&layer) else {
        return (0.0, 0.0, 0);
    };
    let soma: f32 = h.iter().sum();
    let max = h.iter().copied().fold(0.0, f32::max);
    (soma, max, h.iter().filter(|v| **v > 0.0).count())
}

pub(super) fn tinta(t: &PainterTool) -> u64 {
    t.canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u64::from(255 - p[0]))
        .sum()
}

#[test]
#[ignore = "sonda: o relevo sob o pincel composto"]
fn diag_o_relevo_da_pilha() {
    use CompositeOp::{Blur, Brush, Erase};
    for (nome, camadas) in [
        ("sem pilha", &[][..]),
        ("1 Brush", &[Brush][..]),
        ("2 Brush", &[Brush, Brush][..]),
        ("Brush+Blur", &[Brush, Blur][..]),
        ("Brush+Erase", &[Brush, Erase][..]),
    ] {
        let mut t = tela(camadas);
        traco(&mut t);
        let (soma, max, n) = relevo(&t);
        println!(
            "  {nome:<12} activa={} relevo soma {soma:9.2} max {max:.3} texels {n:5} · tinta {}",
            t.composite_active(),
            tinta(&t)
        );
    }
}

/// O que as ferramentas AVULSAS fazem a um relevo já pintado — a régua que a pilha tem de seguir.
#[test]
#[ignore = "sonda: o relevo sob as ferramentas avulsas e sob a pilha, sobre um corpo já pintado"]
fn diag_o_relevo_sob_as_operacoes() {
    use CompositeOp::{Blur, Brush, Erase, Smear};
    let corpo = || {
        let mut t = tela(&[]);
        traco(&mut t);
        t
    };
    let base = relevo(&corpo());
    println!(
        "  corpo pintado: soma {:.2} max {:.3} texels {}",
        base.0, base.1, base.2
    );
    let transversal = |t: &mut PainterTool| {
        t.on_canvas_pointer(cp([64.0, 30.0], PointerPhase::Down));
        let mut y = 30.0;
        while y < 98.0 {
            y += 3.0;
            t.on_canvas_pointer(cp([64.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([64.0, 98.0], PointerPhase::Up));
    };
    for (nome, modo, borracha) in [
        ("rail Borracha", PaintMode::Paint, true),
        ("rail Blur", PaintMode::Blur, false),
        ("rail Smear", PaintMode::Smear, false),
    ] {
        let mut t = corpo();
        t.paint.paint_mode = modo;
        t.paint.eraser = borracha;
        transversal(&mut t);
        let r = relevo(&t);
        println!(
            "  {nome:<18} soma {:9.2} max {:.3} texels {:5}",
            r.0, r.1, r.2
        );
    }
    for (nome, camadas) in [
        ("pilha Brush+Erase", &[Brush, Erase][..]),
        ("pilha Brush+Blur", &[Brush, Blur][..]),
        ("pilha Brush+Smear", &[Brush, Smear][..]),
    ] {
        let mut t = corpo();
        t.paint.composite_enabled = true;
        for (pos, op) in camadas.iter().enumerate() {
            t.paint.composite[pos] = CompositeLayer::nova(*op);
        }
        // a camada Brush a força zero: só a operação de cima age sobre o corpo
        t.paint.composite[0].strength = 1e-6;
        transversal(&mut t);
        let r = relevo(&t);
        println!(
            "  {nome:<18} soma {:9.2} max {:.3} texels {:5}",
            r.0, r.1, r.2
        );
    }
}

/// O `Draw To` do pincel dentro da pilha: `Depth` pinta cor? `Color` deposita corpo?
#[test]
#[ignore = "sonda: o Draw To sob a pilha"]
fn diag_o_draw_to_sob_a_pilha() {
    use CompositeOp::Brush;
    use ph2d_painter_brush::DrawTo;
    for d in [DrawTo::ColorAndDepth, DrawTo::Color, DrawTo::Depth] {
        for (nome, camadas) in [("sem pilha", &[][..]), ("2 Brush", &[Brush, Brush][..])] {
            let mut t = tela(camadas);
            t.paint.brush.impasto_draw_to = d;
            traco(&mut t);
            let (soma, _, n) = relevo(&t);
            println!(
                "  {d:?} {nome:<10} relevo {soma:8.2} texels {n:5} · tinta {}",
                tinta(&t)
            );
        }
    }
    // O rail da borracha: a lei com que ela morde o corpo.
    let mut t = tela(&[]);
    traco(&mut t);
    let layer = t.layers.active().unwrap();
    let antes = t.heights.get(&layer).unwrap().clone();
    t.paint.eraser = true;
    t.paint.brush.strength = 0.5;
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Up));
    let depois = t.heights.get(&layer).unwrap();
    let i = (64 * S + 64) as usize;
    println!(
        "  borracha rail forca 0,5 no centro: {:.4} -> {:.4}",
        antes[i], depois[i]
    );
}

/// O Tiling sob a pilha: a COR das cópias embrulhadas chega à tela?
#[test]
#[ignore = "sonda: o Tiling sob a pilha"]
fn diag_o_tiling_sob_a_pilha() {
    use CompositeOp::Brush;
    let pinta_a_esquerda = |t: &PainterTool| -> u64 {
        let mut n = 0;
        for y in 0..S {
            for x in 0..40u32 {
                let i = ((y * S + x) * 4) as usize;
                n += u64::from(t.canvas_rgba[i] < 250);
            }
        }
        n
    };
    for (nome, camadas, impasto) in [
        ("avulso digital", &[][..], false),
        ("pilha digital", &[Brush, Brush][..], false),
        ("avulso impasto", &[][..], true),
        ("pilha impasto", &[Brush, Brush][..], true),
    ] {
        let mut t = tela(camadas);
        if !impasto {
            t.set_brush_impasto(false);
        }
        t.paint.tiling = [true, true];
        t.on_canvas_pointer(cp([90.0, 50.0], PointerPhase::Down));
        for i in 1..=30 {
            let s = i as f32 / 30.0;
            t.on_canvas_pointer(cp([90.0 + 70.0 * s, 50.0 + 20.0 * s], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([160.0, 70.0], PointerPhase::Up));
        let (soma, _, n) = relevo(&t);
        println!(
            "  {nome:<16} cor na borda esquerda (x<40): {:5} px · relevo {soma:8.2} ({n} texels)",
            pinta_a_esquerda(&t)
        );
    }
}

/// A COR da pilha contra a do pincel avulso, com a 2.ª camada quase a zero (só para armar a pilha).
#[test]
#[ignore = "sonda: a cor da pilha no Impasto"]
fn diag_a_cor_da_pilha_no_impasto() {
    use CompositeOp::Brush;
    for impasto in [false, true] {
        let mut avulso = tela(&[]);
        let mut pilha = tela(&[Brush, Brush]);
        pilha.paint.composite[1].strength = 1e-6;
        for t in [&mut avulso, &mut pilha] {
            if !impasto {
                t.set_brush_impasto(false);
            }
            traco(t);
        }
        let (mut dif, mut pior) = (0u64, 0u8);
        for (a, b) in avulso.canvas_rgba.iter().zip(pilha.canvas_rgba.iter()) {
            let d = a.abs_diff(*b);
            dif += u64::from(d > 1);
            pior = pior.max(d);
        }
        println!(
            "  impasto={impasto}: tinta avulso {} · pilha {} · bytes >1 diferentes {dif} · pior {pior}",
            tinta(&avulso),
            tinta(&pilha)
        );
    }
}

/// Paridade de COR e de RELEVO, pilha contra avulso, para o Brush e a borracha, nos dois meios.
#[test]
#[ignore = "sonda: a paridade da pilha contra o avulso"]
fn diag_a_paridade_da_pilha() {
    use CompositeOp::{Brush, Erase};
    for impasto in [false, true] {
        for op in [Brush, Erase] {
            let prepara = |pilha: bool| {
                let mut t = tela(&[]);
                if !impasto {
                    t.set_brush_impasto(false);
                }
                if matches!(op, Erase) {
                    traco(&mut t); // um corpo por baixo para a borracha comer
                }
                if pilha {
                    t.paint.composite_enabled = true;
                    t.paint.composite[0] = CompositeLayer::nova(op);
                    t.paint.composite[1] = CompositeLayer::nova(Brush);
                    t.paint.composite[1].strength = 1e-6;
                } else if matches!(op, Erase) {
                    t.paint.eraser = true;
                }
                t.on_canvas_pointer(cp([64.0, 30.0], PointerPhase::Down));
                for i in 1..=30 {
                    t.on_canvas_pointer(cp(
                        [64.0, 30.0 + 68.0 * i as f32 / 30.0],
                        PointerPhase::Move,
                    ));
                }
                t.on_canvas_pointer(cp([64.0, 98.0], PointerPhase::Up));
                t
            };
            let (a, b) = (prepara(false), prepara(true));
            let (mut dif, mut pior) = (0u64, 0u8);
            for (x, y) in a.canvas_rgba.iter().zip(b.canvas_rgba.iter()) {
                dif += u64::from(x.abs_diff(*y) > 1);
                pior = pior.max(x.abs_diff(*y));
            }
            let (ra, rb) = (relevo(&a).0, relevo(&b).0);
            println!(
                "  impasto={impasto:<5} {op:?}: cor bytes>1 {dif:5} pior {pior:3} · relevo {ra:8.2} contra {rb:8.2}"
            );
        }
    }
}

/// O Tiling ao byte: com e sem grão aleatório, onde a pilha diverge do avulso?
#[test]
#[ignore = "sonda: o tiling ao byte"]
fn diag_o_tiling_ao_byte() {
    use CompositeOp::Brush;
    for grao in [false, true] {
        for atravessa in [false, true] {
            let corre = |t: &mut PainterTool| {
                t.set_brush_impasto(false);
                if grao {
                    t.paint.brush.texture.kind = ph2d_painter_brush::TextureKind::Noise;
                    t.paint.brush.texture.mapping = ph2d_painter_brush::TextureMapping::Random;
                }
                t.paint.tiling = [true, true];
                let (a, b) = if atravessa {
                    (90.0, 160.0)
                } else {
                    (20.0, 80.0)
                };
                t.on_canvas_pointer(cp([a, 50.0], PointerPhase::Down));
                for i in 1..=30 {
                    let s = i as f32 / 30.0;
                    t.on_canvas_pointer(cp([a + (b - a) * s, 50.0 + 20.0 * s], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([b, 70.0], PointerPhase::Up));
            };
            let mut avulso = tela(&[]);
            corre(&mut avulso);
            let mut pilha = tela(&[Brush, Brush]);
            pilha.paint.composite[1].strength = 1e-6;
            corre(&mut pilha);
            let (mut dif, mut pior, mut xs) = (0u64, 0u8, (u32::MAX, 0u32));
            for (i, (x, y)) in avulso
                .canvas_rgba
                .iter()
                .zip(pilha.canvas_rgba.iter())
                .enumerate()
            {
                if x.abs_diff(*y) > 1 {
                    dif += 1;
                    let px = (i / 4) as u32 % S;
                    xs = (xs.0.min(px), xs.1.max(px));
                }
                pior = pior.max(x.abs_diff(*y));
            }
            println!(
                "  grao={grao:<5} atravessa={atravessa:<5}: bytes>1 {dif:5} pior {pior:3} colunas {xs:?}"
            );
        }
    }
}

/// **8b — a borracha POR CIMA de um Brush, no mesmo traço:** a cor vai e o CORPO fica?
///
/// Camada 0 (topo) = Erase, camada 1 = Brush. A borracha cheia (dura, `1,5×` o tamanho) cobre toda a
/// pegada do Brush: a tela tem de voltar ao `pre` e o corpo tem de ser ZERO. A macia mede o meio.
#[test]
#[ignore = "sonda: o corpo fantasma da borracha de cima"]
fn diag_o_corpo_fantasma() {
    use super::composite::EscopoDaBorracha;
    use CompositeOp::{Brush, Erase};
    let (so_brush, _, _) = {
        let mut t = tela(&[Brush]);
        traco(&mut t);
        relevo(&t)
    };
    println!("  referência: 1 Brush, relevo soma {so_brush:9.2}");
    for escopo in [EscopoDaBorracha::Traco, EscopoDaBorracha::Tudo] {
        for (nome, dura) in [("macia", false), ("cheia", true)] {
            for (ordem, camadas) in [
                ("Erase EM CIMA", [Erase, Brush]),
                ("Erase EM BAIXO", [Brush, Erase]),
            ] {
                let mut t = tela(&camadas);
                let e = usize::from(camadas[1] == Erase);
                t.paint.composite[e].erase_scope = escopo;
                if dura {
                    t.paint.composite[e].hardness = Some(1.0);
                    t.paint.composite[e].size = 1.5;
                }
                traco(&mut t);
                let (soma, max, n) = relevo(&t);
                println!(
                    "  {escopo:?} {nome:<5} {ordem:<14} relevo soma {soma:9.2} max {max:.3} texels {n:5} · tinta {}",
                    tinta(&t)
                );
            }
        }
    }
}
