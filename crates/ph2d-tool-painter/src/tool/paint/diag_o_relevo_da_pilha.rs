//! Sonda da fila 44, item 8: **o pincel composto deposita RELEVO no Impasto?**

use super::composite::{CompositeLayer, CompositeOp, N_CAMADAS};
use super::*;
use ph2d_painter_brush::Falloff;

const S: u32 = 128;

fn tela(camadas: &[CompositeOp]) -> PainterTool {
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

fn traco(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([20.0, 64.0], PointerPhase::Down));
    let mut x = 20.0;
    while x < 108.0 {
        x += 3.0;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([108.0, 64.0], PointerPhase::Up));
}

fn relevo(t: &PainterTool) -> (f32, f32, usize) {
    let layer = t.layers.active().expect("camada");
    let Some(h) = t.heights.get(&layer) else {
        return (0.0, 0.0, 0);
    };
    let soma: f32 = h.iter().sum();
    let max = h.iter().copied().fold(0.0, f32::max);
    (soma, max, h.iter().filter(|v| **v > 0.0).count())
}

fn tinta(t: &PainterTool) -> u64 {
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

/// **Report do dono (30/09): Smear entre Blur e Brush faz o RELEVO do traço desaparecer.**
#[test]
#[ignore = "sonda: o relevo com o Smear na pilha"]
fn diag_o_relevo_com_o_smear() {
    use CompositeOp::{Blur, Brush, Smear};
    for (nome, camadas) in [
        ("Brush", &[Brush][..]),
        ("Smear/Brush", &[Smear, Brush][..]),
        ("Brush/Smear", &[Brush, Smear][..]),
        ("Blur/Brush", &[Blur, Brush][..]),
        ("Blur/Smear/Brush", &[Blur, Smear, Brush][..]),
        ("Smear/Blur/Brush", &[Smear, Blur, Brush][..]),
        ("Blur/Brush/Smear", &[Blur, Brush, Smear][..]),
    ] {
        let mut t = tela(camadas);
        traco(&mut t);
        let (soma, max, n) = relevo(&t);
        println!(
            "  {nome:<18} relevo soma {soma:9.2} max {max:.3} texels {n:5} · tinta {}",
            tinta(&t)
        );
    }
}

/// A pilha da FOTO do dono (a da cena `PH2D_COMPOSITE_SMOKE`), no Impasto, com o Smear na posição
/// `smear` (0 = topo). Mede o relevo VIVO a meio do traço e o ASSENTE depois de soltar.
#[test]
#[ignore = "sonda: a pilha do dono com o Smear a subir"]
fn diag_a_pilha_do_dono_com_o_smear_a_subir() {
    use CompositeOp::{Blur, Brush, Erase, Smear};
    let base = [
        (Blur, 1.0f32, 2.048f32),
        (Brush, 0.133, 0.574),
        (Brush, 0.204, 1.002),
        (Brush, 0.176, 1.221),
        (Erase, 0.104, 1.0),
    ];
    for por_cima in [false, true] {
        println!("  por cima de um traço já assente = {por_cima}");
        for smear_em in [5usize, 4, 1, 0] {
            let mut pilha: Vec<(CompositeOp, f32, f32)> = base.to_vec();
            pilha.insert(smear_em.min(pilha.len()), (Smear, 0.596, 1.0));
            let mut t = tela(&[]);
            t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
            t.set_brush_size_norm(0.4);
            if por_cima {
                t.on_canvas_pointer(cp([450.0, 300.0], PointerPhase::Down));
                let mut y = 300.0;
                while y < 720.0 {
                    y += 3.0;
                    t.on_canvas_pointer(cp([450.0, y], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([450.0, 720.0], PointerPhase::Up));
            }
            let (antes, _, _) = relevo(&t);
            t.paint.composite_enabled = true;
            t.paint.composite_len = pilha.len();
            for (i, &(op, s, sz)) in pilha.iter().enumerate() {
                t.paint.composite[i] = CompositeLayer {
                    strength: s,
                    size: sz,
                    ..CompositeLayer::nova(op)
                };
            }
            t.on_canvas_pointer(cp([200.0, 512.0], PointerPhase::Down));
            let mut x = 200.0;
            while x < 700.0 {
                x += 3.0;
                t.on_canvas_pointer(cp([x, 512.0 + 60.0 * (x / 80.0).sin()], PointerPhase::Move));
            }
            let vivo: f32 = t.paint.relief.stroke_height.iter().sum();
            t.on_canvas_pointer(cp([700.0, 512.0], PointerPhase::Up));
            let (soma, max, n) = relevo(&t);
            let layer = t.layers.active().expect("camada");
            let cobre: u64 = t
                .covers
                .get(&layer)
                .map_or(0, |c| c.iter().map(|v| u64::from(*v)).sum());
            // A RÉGUA do report: tinta DESTE traço (a cor mudou contra a tela branca) sem CORPO nenhum.
            let hs = t.heights.get(&layer).cloned().unwrap_or_default();
            let sem_corpo = t
                .canvas_rgba
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter(|(i, px)| {
                    let tinta = px[..3].iter().any(|&c| c < 200);
                    tinta && hs.get(*i).copied().unwrap_or(0.0) < 0.02
                })
                .count();
            let com_tinta = t
                .canvas_rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|px| px[..3].iter().any(|&c| c < 200))
                .count();
            println!("     tinta sem corpo: {sem_corpo} de {com_tinta} px com tinta");
            let ops: Vec<_> = pilha.iter().map(|p| format!("{:?}", p.0)).collect();
            println!(
                "  {:<44} antes {antes:9.1} · vivo {vivo:10.1} · assente {soma:10.1} max {max:.3} texels {n:6} · cobertura {cobre}",
                ops.join("/")
            );
        }
    }
}

/// DOIS traços da pilha do dono seguidos, que se cruzam — o volume do 1.º sobrevive ao 2.º?
#[test]
#[ignore = "sonda: dois traços da pilha com o Smear"]
fn diag_dois_tracos_da_pilha_com_o_smear() {
    use CompositeOp::{Blur, Brush, Erase, Smear};
    let base = [
        (Blur, 1.0f32, 2.048f32),
        (Brush, 0.133, 0.574),
        (Brush, 0.204, 1.002),
        (Brush, 0.176, 1.221),
        (Erase, 0.104, 1.0),
    ];
    let risca = |t: &mut PainterTool, de: [f32; 2], ate: [f32; 2]| {
        t.on_canvas_pointer(cp(de, PointerPhase::Down));
        for i in 1..=120 {
            let s = i as f32 / 120.0;
            t.on_canvas_pointer(cp(
                [de[0] + (ate[0] - de[0]) * s, de[1] + (ate[1] - de[1]) * s],
                PointerPhase::Move,
            ));
            if i % 3 == 0 {
                t.compoe_o_pendente();
            }
        }
        t.on_canvas_pointer(cp(ate, PointerPhase::Up));
    };
    for por_quadro in [false, true] {
        println!("  por quadro = {por_quadro}");
        for smear_em in [5usize, 4, 1, 0, 99] {
            let mut pilha: Vec<(CompositeOp, f32, f32)> = base.to_vec();
            if smear_em < 99 {
                pilha.insert(smear_em.min(pilha.len()), (Smear, 0.596, 1.0));
            }
            let mut t = tela(&[]);
            t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
            t.set_brush_size_norm(0.4);
            t.paint.composite_enabled = true;
            t.set_compor_por_quadro(por_quadro);
            t.paint.composite_len = pilha.len();
            for (i, &(op, s, sz)) in pilha.iter().enumerate() {
                t.paint.composite[i] = CompositeLayer {
                    strength: s,
                    size: sz,
                    ..CompositeLayer::nova(op)
                };
            }
            risca(&mut t, [200.0, 512.0], [800.0, 512.0]);
            let (um, _, _) = relevo(&t);
            // O volume do 1.º traço numa faixa LONGE do 2.º.
            let layer = t.layers.active().expect("camada");
            let faixa = |t: &PainterTool| -> f32 {
                let h = t.heights.get(&layer).cloned().unwrap_or_default();
                (450..570)
                    .flat_map(|y| (200..380).map(move |x| y * 1024 + x))
                    .map(|i| h.get(i).copied().unwrap_or(0.0))
                    .sum()
            };
            let longe_antes = faixa(&t);
            risca(&mut t, [600.0, 200.0], [600.0, 820.0]);
            let (dois, _, _) = relevo(&t);
            let ops: Vec<_> = pilha.iter().map(|p| format!("{:?}", p.0)).collect();
            println!(
                "  {:<44} 1.º {um:9.1} · depois do 2.º {dois:9.1} · faixa longe do 2.º {longe_antes:8.1} -> {:8.1}",
                ops.join("/"),
                faixa(&t)
            );
        }
    }
}

/// **O report com fotos (30/09):** pilha Blur 1,339 / Smear / Brush, tudo a força 1; o volume do
/// traço NOVO some num RECTÂNGULO depois de soltar, e só por cima de tinta com volume.
/// Régua: o volume final contra o da MESMA pilha com o Smear a zero, e a caixa onde ele falta.
#[test]
#[ignore = "sonda: o rectângulo sem corpo"]
fn diag_o_rectangulo_sem_corpo() {
    use CompositeOp::{Blur, Brush, Smear};
    let corre = |smear: f32, por_quadro: bool| -> (Vec<f32>, Vec<f32>) {
        let mut t = tela(&[]);
        t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
        t.set_brush_size_norm(0.4);
        t.paint.composite_enabled = true;
        t.set_compor_por_quadro(por_quadro);
        t.paint.composite_len = 3;
        for (i, (op, s, sz)) in [(Blur, 1.0, 1.339), (Smear, smear, 1.0), (Brush, 1.0, 1.0)]
            .into_iter()
            .enumerate()
        {
            t.paint.composite[i] = CompositeLayer {
                strength: s,
                size: sz,
                ..CompositeLayer::nova(op)
            };
        }
        let risca = |t: &mut PainterTool, pts: &[[f32; 2]]| {
            t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
            for (i, p) in pts.iter().enumerate().skip(1) {
                t.on_canvas_pointer(cp(*p, PointerPhase::Move));
                if i % 3 == 0 {
                    t.compoe_o_pendente();
                }
            }
            let caixa = |t: &PainterTool| -> f32 {
                let Some(l) = t.layers.active() else {
                    return -1.0;
                };
                let h = t.heights.get(&l).cloned().unwrap_or_default();
                (560..740usize)
                    .flat_map(|y| (455..690usize).map(move |x| y * 1024 + x))
                    .map(|i| h.get(i).copied().unwrap_or(0.0))
                    .sum()
            };
            let a = caixa(t);
            let pend = t.paint.pilha.pendente;
            let b = caixa(t);
            t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
            eprintln!(
                "    caixa: antes de compor o pendente {a:.1} · depois {b:.1} · depois de soltar {:.1} · pendente {pend:?} · warp activo {}",
                caixa(t),
                t.paint.warp.active
            );
        };
        let curva = |cx: f32, cy: f32, fase: f32| -> Vec<[f32; 2]> {
            (0..=160)
                .map(|i| {
                    let s = i as f32 / 160.0 * 6.0;
                    [
                        cx + 160.0 * (s + fase).cos() * (s / 6.0),
                        cy + 160.0 * (s + fase).sin() * (s / 6.0),
                    ]
                })
                .collect()
        };
        risca(&mut t, &curva(450.0, 450.0, 0.0));
        let layer = t.layers.active().expect("camada");
        let antes = (**t.heights.get(&layer).expect("relevo")).clone();
        risca(&mut t, &curva(560.0, 520.0, 2.0));
        (antes, (**t.heights.get(&layer).expect("relevo")).clone())
    };
    for por_quadro in [false, true] {
        let (_, com) = corre(1.0, por_quadro);
        let (_, sem) = corre(0.0, por_quadro);
        let (mut n, mut x0, mut y0, mut x1, mut y1) = (0u32, u32::MAX, u32::MAX, 0u32, 0u32);
        for (i, (a, b)) in com.iter().zip(&sem).enumerate() {
            if b - a > 0.3 {
                n += 1;
                let (x, y) = (i as u32 % 1024, i as u32 / 1024);
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
        if let Ok(dir) = std::env::var("PH2D_DIAG_DIR") {
            let grava = |nome: &str, f: &dyn Fn(usize) -> f32| {
                let mut img = b"P5 1024 1024 255\n".to_vec();
                img.extend((0..1024 * 1024).map(|i| (f(i).clamp(0.0, 1.0) * 255.0) as u8));
                let _ = std::fs::write(format!("{dir}/{nome}_{por_quadro}.pgm"), img);
            };
            grava("com", &|i| com[i] / 3.0);
            grava("sem", &|i| sem[i] / 3.0);
            grava("falta", &|i| (sem[i] - com[i]) / 1.5);
        }
        println!(
            "  por quadro {por_quadro}: {n} texels com MENOS corpo que sem o Smear, caixa {x0},{y0} {}x{} · soma com {:.1} sem {:.1}",
            x1.saturating_sub(x0) + 1,
            y1.saturating_sub(y0) + 1,
            com.iter().sum::<f32>(),
            sem.iter().sum::<f32>()
        );
    }
}
