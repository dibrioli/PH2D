//! **Sonda (31/10, temporária): por que a miniatura de fábrica mudou** — cada preset que o portão
//! `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` reprova, pintado pelo MESMO `miniatura_do_preset` em 4 variantes
//! no mesmo processo (produto · sem Bingham · sem piso · sem os dois). Doc: `docs/Painter/Arquitetura/FECHO_31_10.md` §7.

use super::*;
use std::sync::atomic::Ordering;

fn variante(sem_bingham: bool, sem_piso: bool) {
    let tau: f32 = if sem_bingham { -1.0 } else { 0.0 };
    ph2d_wet_paint::solver::PAPEL_BINGHAM.store(tau.to_bits(), Ordering::Relaxed);
    ph2d_wet_paint::trail::sonda::SEM_PISO.store(sem_piso, Ordering::Relaxed);
}

fn difere(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// RGB sobre o branco 250 (a tinta tem alfa), como os `grava_png` das sondas do óleo.
fn ppm(p: &std::path::Path, rgba: &[u8], w: u32, h: u32) {
    let mut o = format!("P6\n{w} {h}\n255\n").into_bytes();
    for q in rgba.chunks(4) {
        let a = f32::from(q[3]) / 255.0;
        for c in &q[..3] {
            o.push((f32::from(*c) * a + 250.0 * (1.0 - a)).round() as u8);
        }
    }
    std::fs::write(p, o).unwrap();
}

#[test]
#[ignore = "sonda: bissecção das miniaturas que mudaram (Bingham × piso); PH2D_SONDA_FOTOS=<pasta> grava PPMs"]
fn sonda_miniaturas_bingham_x_piso() {
    let fotos = std::env::var("PH2D_SONDA_FOTOS").ok().map(PathBuf::from);
    let mut linhas = Vec::new();
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        variante(false, false);
        let prod = miniatura_do_preset(&p);
        if prod == p.miniatura {
            continue;
        }
        let mut n = [difere(&prod.rgba, &p.miniatura.rgba), 0, 0, 0];
        for (k, (sb, sp)) in [(true, false), (false, true), (true, true)]
            .into_iter()
            .enumerate()
        {
            variante(sb, sp);
            n[k + 1] = difere(&miniatura_do_preset(&p).rgba, &p.miniatura.rgba);
        }
        variante(false, false);
        let mut maxd = 0u8;
        for (a, c) in prod.rgba.iter().zip(&p.miniatura.rgba) {
            maxd = maxd.max(a.abs_diff(*c));
        }
        if let Some(d) = &fotos {
            let nome = rel
                .rsplit('/')
                .next()
                .unwrap()
                .replace(".ph2d-brush.png", "")
                .replace(' ', "_");
            let dif: Vec<u8> = prod
                .rgba
                .chunks(4)
                .zip(p.miniatura.rgba.chunks(4))
                .flat_map(|(a, c)| {
                    let mut q = [0u8; 4];
                    for i in 0..3 {
                        q[i] = 255 - a[i].abs_diff(c[i]).saturating_mul(4);
                    }
                    q[3] = 255;
                    q
                })
                .collect();
            ppm(
                &d.join(format!("{nome}_a_fabrica.ppm")),
                &p.miniatura.rgba,
                p.miniatura.largura,
                p.miniatura.altura,
            );
            ppm(
                &d.join(format!("{nome}_b_produto.ppm")),
                &prod.rgba,
                prod.largura,
                prod.altura,
            );
            ppm(
                &d.join(format!("{nome}_c_dif4.ppm")),
                &dif,
                prod.largura,
                prod.altura,
            );
        }
        linhas.push(format!(
            "| {rel} | {} | {} | {} | {} | {maxd} |",
            n[0], n[1], n[2], n[3]
        ));
    }
    println!("| pincel | produto | sem Bingham | sem piso | sem os dois | máx Δ canal |");
    for l in linhas {
        println!("{l}");
    }
}
