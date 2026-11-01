//! E10d1: a camada fina tem o matiz da cor escolhida; o vidrado sobre tinta curada continua por banda.

use super::desvio_de_matiz;
use crate::grid::Grid;
use crate::grid::tinta::{self, garante};
use crate::render::tinta::ecra_da_celula;
use ph2d_paint_tinta::{Bandas, Soma, Tinta, camada, decompoe, optica_celula};

/// As cores da sonda F0 (`sonda_varredura_oleo.rs`: `AZ`, `AM`).
const AZ: [f32; 3] = [40.0, 90.0, 220.0];
const AM: [f32; 3] = [245.0, 205.0, 25.0];

/// Volumes (µm) de quase nada ao masstone: a camada fina inteira.
fn volumes() -> Vec<f32> {
    (0..48).map(|k| 0.002 * 1.25f32.powi(k)).collect()
}

/// O que se vê sobre o papel branco (`cor·α + 1 − α`, sRGB 0..1) de cada volume `v` da tinta `cor` (ao ar, seca).
fn sobre_branco(cor: [f32; 3], meio: f32) -> Vec<(f32, [f32; 3])> {
    let vs = volumes();
    let mut g = Grid::new(vs.len(), 1);
    let s = g.s;
    let t = Tinta::de_srgb(cor.map(|c| c / 255.0), 0.0);
    let cel = garante(&mut g);
    for (k, &v) in vs.iter().enumerate() {
        cel[(k + 1) + s].aberto = Soma::de(t, v).com_meio(meio);
    }
    vs.iter()
        .enumerate()
        .filter_map(|(k, &v)| {
            let (c, a) = ecra_da_celula(&g, (k + 1) + s)?;
            let a = a as f32;
            Some((v, c.map(|x| (x as f32 / 255.0) * a + (1.0 - a))))
        })
        .collect()
}

#[test]
fn a_camada_fina_tem_o_matiz_da_cor_escolhida() {
    for (nome, cor) in [("azul", AZ), ("amarelo", AM)] {
        let esc = Tinta::de_srgb(cor.map(|c| c / 255.0), 0.0).lab;
        for meio in [0.0, 1.0] {
            let mut pior = (0.0f32, 0.0f32, [0.0f32; 3]);
            for (v, e) in sobre_branco(cor, meio) {
                // O papel quase nu (`< 1 %` de tinta) não tem matiz mensurável em 8 bits.
                if e.iter().all(|&x| x > 0.99) {
                    continue;
                }
                let d = desvio_de_matiz(e, esc);
                if d > pior.0 {
                    pior = (d, v, e.map(|x| (x * 255.0).round()));
                }
            }
            eprintln!(
                "[{nome} m={meio}] desvio máx {:.2}° em v={:.4} µm, cor {:?}",
                pior.0, pior.1, pior.2
            );
            assert!(
                pior.0 <= 2.0,
                "{nome} m={meio}: a camada fina muda de matiz {:.2}° (v={}, {:?})",
                pior.0,
                pior.1,
                pior.2
            );
        }
    }
}

#[test]
fn o_vidrado_sobre_tinta_curada_continua_por_banda() {
    let azul = Tinta::de_srgb(AZ.map(|c| c / 255.0), 0.0);
    // O vidrado: amarelo TRANSPARENTE (Transparency 0,9), como o de `tinta_tests.rs`.
    let amarelo = Tinta::de_srgb(AM.map(|c| c / 255.0), 0.9);
    let curado = optica_celula(&Soma::default(), &Soma::de(azul, 6.0), 0.0, false);
    let r_g = camada(&curado, &Bandas([1.0; 3])).0;
    let r_g0 = camada(&curado, &Bandas([0.0; 3])).0;
    let mut g = Grid::new(1, 1);
    let i = 1 + g.s;
    let c = &mut garante(&mut g)[i];
    c.aberto = Soma::de(amarelo, 0.6);
    c.r_g = r_g;
    c.r_g0 = r_g0;
    // A régua: a pilha por banda de hoje, decomposta (sem prender o matiz).
    let c = tinta::em(&g, i);
    let o = optica_celula(&c.filme, &c.aberto, c.meio(), false);
    let (cor, a) = {
        let (rw, mc) = tinta::pilha(&o, &c);
        decompoe(&rw, mc)
    };
    let (vista, va) = ecra_da_celula(&g, i).expect("tinta");
    eprintln!(
        "[vidrado] fundo r_g {r_g:?} r_g0 {r_g0:?} · régua {:?} α {a} · ecrã {vista:?} α {va}",
        cor.0.map(|x| x * 255.0)
    );
    for b in 0..3 {
        assert!(
            (vista[b] / 255.0 - f64::from(cor.0[b])).abs() <= 1e-6,
            "banda {b}"
        );
    }
    assert!((va - f64::from(a)).abs() <= 1e-6);
    // Amarelo sobre azul curado = VERDE.
    assert!(
        vista[1] > vista[0] && vista[1] > vista[2],
        "verde: {vista:?}"
    );
}

/// O fecho com [`super::pilha_presa`] é invisível: a camada fina presa que cura guarda o matiz e o `α`.
#[test]
fn fechar_a_camada_fina_guarda_o_matiz() {
    for cor in [AZ, AM] {
        let t = Tinta::de_srgb(cor.map(|c| c / 255.0), 0.0);
        for v in [0.003f32, 0.03, 0.3, 3.0] {
            let mut g = Grid::new(1, 1);
            let i = 1 + g.s;
            garante(&mut g)[i].aberto = Soma::de(t, v);
            let antes = ecra_da_celula(&g, i).expect("tinta");
            let c = tinta::em(&g, i);
            let o = optica_celula(&Soma::default(), &c.aberto, 0.0, false);
            let (r_g, r_g0) = super::pilha_presa(&o, &c, None);
            let cel = &mut g.tinta[i];
            (cel.r_g, cel.r_g0, cel.aberto) = (r_g, r_g0, Soma::default());
            let depois = ecra_da_celula(&g, i).expect("fundo");
            let d = (0..3)
                .map(|b| (antes.0[b] - depois.0[b]).abs())
                .fold((antes.1 - depois.1).abs() * 255.0, f64::max);
            assert!(
                d <= 0.01,
                "{cor:?} v={v}: o fecho mudou o ecrã {d} ({antes:?} → {depois:?})"
            );
        }
    }
}

/// O ecrã por banda de HOJE (sem prender o matiz), sobre branco: a régua «antes» da sonda.
fn por_banda_sobre_branco(cor: [f32; 3], v: f32) -> [f32; 3] {
    let t = Tinta::de_srgb(cor.map(|c| c / 255.0), 0.0);
    let mut c = crate::grid::tinta::CelulaTinta::NEUTRA;
    c.aberto = Soma::de(t, v);
    let o = optica_celula(&c.filme, &c.aberto, 0.0, false);
    let (rw, mc) = tinta::pilha(&o, &c);
    let (k, a) = decompoe(&rw, mc);
    k.0.map(|x| x * a + (1.0 - a))
}

fn pct(v: &mut [f32], q: f32) -> f32 {
    v.sort_by(f32::total_cmp);
    v.get(((v.len() as f32 - 1.0) * q).round() as usize)
        .copied()
        .unwrap_or(0.0)
}

/// A borda fina antes × depois (E10d1 §3): faixas do quase-nada ao masstone (48 volumes) do azul e do amarelo da
/// sonda F0 sobre o papel branco — linha 1 antes (por banda), linha 2 depois (matiz preso). Grava
/// `$PH2D_SONDA_OUT/borda_fina_faixas.ppm` e imprime a tabela (desvio de matiz em 8 bits, píxeis com tinta).
#[test]
#[ignore = "sonda: PH2D_SONDA_OUT=<dir> … sonda_borda_fina -- --ignored --nocapture"]
fn sonda_borda_fina() {
    let (cw, ch) = (12usize, 40usize);
    let vs = volumes();
    let mut linhas: Vec<Vec<[f32; 3]>> = vec![];
    for (nome, cor) in [("azul", AZ), ("amarelo", AM)] {
        let esc = Tinta::de_srgb(cor.map(|c| c / 255.0), 0.0).lab;
        let antes: Vec<[f32; 3]> = vs.iter().map(|&v| por_banda_sobre_branco(cor, v)).collect();
        let depois: Vec<[f32; 3]> = sobre_branco(cor, 0.0).into_iter().map(|(_, e)| e).collect();
        for (lei, faixa) in [("antes", &antes), ("depois", &depois)] {
            let q8: Vec<[f32; 3]> = faixa
                .iter()
                .map(|e| e.map(|x| (x * 255.0).round() / 255.0))
                .collect();
            let mut d: Vec<f32> = q8
                .iter()
                .filter(|e| e.iter().any(|&x| x < 0.99))
                .map(|&e| desvio_de_matiz(e, esc))
                .collect();
            let (k, pior) = q8
                .iter()
                .enumerate()
                .filter(|(_, e)| e.iter().any(|&x| x < 0.99))
                .map(|(k, &e)| (k, desvio_de_matiz(e, esc)))
                .fold((0, 0.0f32), |a, b| if b.1 > a.1 { b } else { a });
            println!(
                "| {nome} | {lei} | n {} | p50 {:.1}° | p99 {:.1}° | máx {:.1}° em v={:.4} µm cor {:?} |",
                d.len(),
                pct(&mut d.clone(), 0.5),
                pct(&mut d, 0.99),
                pior,
                vs[k],
                q8[k].map(|x| (x * 255.0).round() as u8)
            );
            linhas.push(faixa.clone());
        }
    }
    let Ok(dir) = std::env::var("PH2D_SONDA_OUT") else {
        return;
    };
    let (w, h) = (vs.len() * cw, linhas.len() * ch);
    let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
    for faixa in &linhas {
        for _ in 0..ch {
            for e in faixa {
                for _ in 0..cw {
                    ppm.extend(e.map(|x| (x * 255.0).round().clamp(0.0, 255.0) as u8));
                }
            }
        }
    }
    std::fs::write(format!("{dir}/borda_fina_faixas.ppm"), ppm).expect("ppm");
}
