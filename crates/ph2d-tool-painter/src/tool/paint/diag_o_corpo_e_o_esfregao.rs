//! Sondas do report de 2026-09-30 (o corpo e o esfregão): o volume com o Smear na pilha, o rectângulo
//! sem corpo depois de soltar, e a cor do esfregão sem corpo. Cortadas do `diag_o_relevo_da_pilha.rs`
//! pelo tecto de LOC; os ajudantes são os dele.

use super::composite::{CompositeLayer, CompositeOp};
use super::diag_o_relevo_da_pilha::{relevo, tela, tinta, traco};
use super::*;

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

/// **O esfregão leva a COR e não o CORPO?** (report de 30/09, foto). Tela com o traço antigo, depois
/// o rabisco da pilha; conta os píxeis com a cor viva do Brush e sem corpo.
#[test]
#[ignore = "sonda: a cor do esfregão sem corpo"]
fn diag_a_cor_do_esfregao_sem_corpo() {
    use CompositeOp::{Blur, Brush, Smear};
    let corre = |ops: &[CompositeOp], smear: f32| -> (Vec<u8>, Vec<f32>) {
        let mut t = tela(ops);
        t.set_source(vec![255u8; 512 * 512 * 4], 512, 512);
        t.set_brush_size_norm(0.25);
        t.paint.brush.color = [1.0, 0.0, 0.0];
        t.set_compor_por_quadro(true);
        for (p, op) in ops.iter().enumerate() {
            if *op == Smear {
                t.paint.composite[p].strength = smear;
            }
        }
        let pts: Vec<[f32; 2]> = (0..=200)
            .map(|i| {
                let s = i as f32 * 0.08;
                [
                    256.0 + 150.0 * (1.3 * s).sin(),
                    256.0 + 130.0 * (0.9 * s).cos(),
                ]
            })
            .collect();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        for (i, p) in pts.iter().enumerate().skip(1) {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            if i % 4 == 0 {
                t.compoe_o_pendente();
            }
        }
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        let layer = t.layers.active().expect("camada");
        let hs = (**t.heights.get(&layer).expect("relevo")).clone();
        ((*t.canvas_rgba).clone(), hs)
    };
    let vermelho = |c: &[u8], i: usize| c[i * 4] > 150 && c[i * 4 + 1] < 110 && c[i * 4 + 2] < 110;
    for sem_corpo in [true, false] {
        super::composite_relevo::ESFREGAO_SEM_CORPO.with(|c| c.set(sem_corpo));
        for (nome, ops) in [
            ("Smear/Brush", vec![Smear, Brush]),
            ("Blur/Smear/Brush", vec![Blur, Smear, Brush]),
        ] {
            let (cor, hs) = corre(&ops, 1.0);
            let (cor0, _) = corre(&ops, 0.0);
            eprintln!(
                "    soma do corpo {:.1} (Smear a 0: {:.1})",
                hs.iter().sum::<f32>(),
                corre(&ops, 0.0).1.iter().sum::<f32>()
            );
            let hmax = hs.iter().copied().fold(0.0f32, f32::max);
            if let Ok(dir) = std::env::var("PH2D_DIAG_DIR") {
                // A cor COM o relevo como sombreado — o que o olho julga: uma lombada que acompanha a cor.
                let mut ppm = b"P6 512 512 255\n".to_vec();
                for y in 0..512usize {
                    for x in 0..512usize {
                        let i = y * 512 + x;
                        let dx = hs[(i + 1).min(hs.len() - 1)] - hs[i.saturating_sub(1)];
                        let dy = hs[(i + 512).min(hs.len() - 1)] - hs[i.saturating_sub(512)];
                        let luz = (0.75 + 2.0 * (dx + dy)).clamp(0.2, 1.4);
                        for k in 0..3 {
                            ppm.push((f32::from(cor[i * 4 + k]) * luz).clamp(0.0, 255.0) as u8);
                        }
                    }
                }
                let _ = std::fs::write(format!("{dir}/corpo_{sem_corpo}_{}.ppm", ops.len()), ppm);
            }
            let (mut arrastado, mut sem) = (0u32, 0u32);
            for (i, h) in hs.iter().enumerate() {
                if vermelho(&cor, i) && !vermelho(&cor0, i) {
                    arrastado += 1;
                    if *h < 0.1 * hmax {
                        sem += 1;
                    }
                }
            }
            println!(
                "  sem_corpo={sem_corpo:<5} {nome:<18} cor ARRASTADA para fora do traço {arrastado:6} px · sem corpo {sem:6} ({:.1} %)",
                100.0 * f64::from(sem) / f64::from(arrastado.max(1))
            );
        }
    }
    super::composite_relevo::ESFREGAO_SEM_CORPO.with(|c| c.set(false));
}
