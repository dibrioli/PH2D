//! **Sonda (ignorada, W1g): as opções numa rodada só** (§0.10) — a Água ao vivo e a Borda que pesa
//! (`Sim::sedimenta`, a fracção por passo por unidade de `|g|`), na bancada do doc 06 §2 (Wet Round, as
//! quatro águas, direita · meia · máxima: a orla de baixo ÷ de cima e quanto desce) e no S do oráculo
//! (a régua do recto: orla plana, fios largos, pontas quadradas), no MESMO processo. `PH2D_SONDA_OUT`
//! grava os `.ppm` das pranchas.
use super::sonda_achatamento::{Variante, cena};
use super::sonda_gota::bancada_com;

const AGUAS: [f64; 4] = [1.0, 0.7, 0.4, 0.2];

fn grava(nome: &str, w: usize, h: usize, px: &[u8]) {
    if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
        let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
        for p in px.chunks(4) {
            out.extend_from_slice(&p[..3]);
        }
        let _ = std::fs::write(format!("{dir}/{nome}.ppm"), out);
    }
}

#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_opcoes_numa_rodada() {
    let ks: Vec<f64> = std::env::var("PH2D_SONDA_KS")
        .ok()
        .map(|v| v.split(',').filter_map(|s| s.parse().ok()).collect())
        .unwrap_or_else(|| vec![0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 40.0]);
    let casos: Vec<(bool, f64)> = [false, true]
        .iter()
        .flat_map(|&v| ks.iter().map(move |&k| (v, k)))
        .collect();
    let mut linhas: Vec<String> = std::thread::scope(|sc| {
        let fios: Vec<_> = casos
            .iter()
            .map(|&(viva, k)| sc.spawn(move || um_caso(viva, k)))
            .collect();
        fios.into_iter().map(|f| f.join().expect("caso")).collect()
    });
    linhas.sort();
    for l in linhas {
        println!("{l}");
    }
}

fn um_caso(viva: bool, k: f64) -> String {
    let ajusta = move |s: &mut ph2d_wet_paint::sim::Sim| {
        s.sedimenta = k;
        s.sim_while_painting = viva;
    };
    let mut linha = format!("[opções] viva {viva} k {k:>5}:");
    for (nome, anel) in [("direita", None), ("meia", Some(4)), ("máxima", Some(8))] {
        let px = bancada_com(0.005, 0.0, &[], anel, AGUAS, &ajusta);
        grava(
            &format!("bancada_v{}_k{k}_{nome}", u8::from(viva)),
            1000,
            640,
            &px,
        );
        let l: Vec<(f64, usize)> = (0..4).map(|i| super::gota_tests::le(&px, i)).collect();
        linha += &format!(
            " | bancada {nome}: orla {:.2} {:.2} {:.2} {:.2} · desce {}",
            l[0].0,
            l[1].0,
            l[2].0,
            l[3].0,
            l.iter().map(|x| x.1).max().unwrap_or(0)
        );
    }
    for (nome, anel) in [("meia", 4u8), ("máxima", 8)] {
        let v = Variante {
            sedimenta: Some(k),
            agua_no_traco: Some(viva),
            ..Variante::default()
        };
        let px = cena(&v, Some((3, anel)), 30, &mut |_, _| {}).0;
        grava(
            &format!("s_v{}_k{k}_{nome}", u8::from(viva)),
            1000,
            640,
            &px,
        );
        linha += &format!(" | S {nome}: recto {:?}", super::achatamento_tests::le(&px));
    }
    linha
}

/// A Live Water vê-se A PINTAR: o S no instante do Up (0 s de água depois), desligada × ligada, nas três
/// inclinações — as fotos da prancha.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_live_water_no_up() {
    for (nome, tilt) in [
        ("direita", None),
        ("meia", Some((3, 4))),
        ("máxima", Some((3, 8))),
    ] {
        for viva in [false, true] {
            let v = Variante {
                live_water: Some(viva),
                ..Variante::default()
            };
            let px = cena(&v, tilt, 0, &mut |_, _| {}).0;
            grava(&format!("up_v{}_{nome}", u8::from(viva)), 1000, 640, &px);
            println!(
                "[up] {nome} viva {viva}: tinta abaixo da orla {:?}",
                super::achatamento_tests::le(&px)
            );
        }
    }
}
