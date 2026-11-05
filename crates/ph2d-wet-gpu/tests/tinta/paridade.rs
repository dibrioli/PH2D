//! ⭐ O portão da E11: meio e pasta, placa × CPU por campo e na cor, a mesma placa consigo mesma ao bit, e o ciclo do worker.

use super::comum::*;
use ph2d_wet_gpu::passo::placa_sabe;
use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::grid::tinta::{M_UM, em};
use ph2d_wet_paint::sim::sim_step;
use ph2d_wet_paint::tuning::Knob;

/// Os passos entre duas idas e voltas no ciclo do worker: a escala de um traço curto.
const CADA: usize = 12;

#[test]
#[ignore = "placa — corra com --ignored --nocapture"]
fn paridade_da_tinta_por_placa() {
    let Some(gpu) = contexto() else { return };
    let passos = 120usize;
    let mut falhas = Vec::new();
    for c in &CENAS {
        let e = pinta(c, 320, 200);
        let idx = e.active_layer;
        let g0 = &e.layers[idx].grid;
        assert!(g0.has_fluid, "{}: o traço deixou água", c.nome);
        assert_eq!(
            g0.oleo.iter().any(|o| o.meio > 0.0),
            c.medium > 0.0,
            "{}: o meio pousou",
            c.nome
        );
        assert_eq!(
            g0.oleo.iter().any(|o| o.corpo > 0.0),
            c.body > 0.0,
            "{}: o corpo pousou",
            c.nome
        );
        // A rota do produto: sem isto a cena mediria uma placa que o worker nunca chama.
        assert!(
            placa_sabe(&e.sim, g0, &e.tuning),
            "{}: a régua não pode passar fora da rota do produto",
            c.nome
        );
        let (sc, gc) = da_cpu(&e, passos);
        let (sg, gg) = da_placa(&gpu, &e, passos);
        let (_, gg2) = da_placa(&gpu, &e, passos);
        let (_, gw) = do_ciclo(&gpu, &e, passos, CADA);
        assert_eq!(sg.frame, sc.frame, "{}: o relógio", c.nome);
        assert!(
            sc.frame - e.sim.frame >= passos as u64 / 2,
            "{}: a cena secou cedo e o passo não andou (muda)",
            c.nome
        );

        // A mesma placa consigo mesma; e o ciclo do worker contra a descida inteira: a descida suja trouxe tudo.
        let bit = dif_bit(&gg, &gg2);
        let bit_ciclo = dif_bit_sem_paredes(&gg, &gw);

        let massa = |g: &Grid| -> f64 { g.susp.iter().chain(&g.sett).map(|&v| f64::from(v)).sum() };
        let (mc, mg) = (massa(&gc), massa(&gg));
        let rel = (mc - mg).abs() / mc.max(1e-9);
        let vm = |g: &Grid| (total(g, |t| t.filme.v) + total(g, |t| t.aberto.v)) * f64::from(M_UM);
        let (vc, vg) = (vm(&gc), vm(&gg));
        let rel_v = (vc - vg).abs() / vc.max(1e-9);
        // O controlo: a CPU contra ela própria com film, susp e sett de partida a ±1 ulp, célula sim, célula não, nos dois sentidos, e a
        // evaporação a 1 ulp de f32 (a conta que a placa faz em f32 e a CPU em f64, à frente do limiar `film < 1e-4`).
        let perturbada = |par: usize| {
            let (mut s1, mut g1) = (e.sim.clone(), e.layers[idx].grid.clone());
            for plano in [&mut g1.film, &mut g1.susp, &mut g1.sett] {
                for (i, f) in plano.iter_mut().enumerate().filter(|(_, f)| **f > 0.0) {
                    *f = f32::from_bits(if i % 2 == par {
                        f.to_bits() + 1
                    } else {
                        f.to_bits() - 1
                    });
                }
            }
            let mut t1 = e.tuning.clone();
            let ev = t1.get(Knob::Evaporation) as f32;
            t1.set(
                Knob::Evaporation,
                f64::from(f32::from_bits(ev.to_bits() + 1)),
            );
            for _ in 0..passos {
                sim_step(&mut s1, &mut g1, &t1);
            }
            (s1, g1)
        };
        let (s1, g1) = perturbada(0);
        let (s2, g2) = perturbada(1);
        let cor_de = |a: &Grid, b: &Grid| desvio_cor(a, b, &com_tinta(a, b));
        let (cu_m, cu_p, cu_n) = cor_de(&gc, &g1);
        let (_, uu_p, _) = cor_de(&g1, &g2);
        let (_, nb_p, _) = cor_de(&gc, &g2);
        // ⭐ O teto é duro: o controlo não sobe limite nenhum. Se a própria CPU se parte a 1 ulp (a lei bifurca: um arredondamento
        // escolhe o ramo), a referência é o ramo da CPU mais perto da placa, e ela responde pelos MESMOS limites contra ele. Tabela no doc.
        let bifurca = cu_p > COR_P999 || uu_p > COR_P999;
        let (sr, gr, nome_ref): (&_, &Grid, &str) = if bifurca {
            [
                (&sc, &gc, "CPU nominal"),
                (&s1, &g1, "CPU ulp A"),
                (&s2, &g2, "CPU ulp B"),
            ]
            .into_iter()
            .min_by(|a, b| cor_de(a.1, &gg).1.total_cmp(&cor_de(b.1, &gg).1))
            .expect("três ramos")
        } else {
            (&sc, &gc, "CPU nominal")
        };
        let pop = com_tinta(gr, &gg);
        let (cm, cp, acima) = desvio_cor(gr, &gg, &pop);
        // A devolução: a CPU continua de onde a placa parou (a soma tem de ter descido com a massa).
        let (mut sa, mut ga, mut sb, mut gb) = (sr.clone(), gr.clone(), sg.clone(), gg.clone());
        for _ in 0..12 {
            sim_step(&mut sa, &mut ga, &e.tuning);
            sim_step(&mut sb, &mut gb, &e.tuning);
        }
        let pop_d = com_tinta(&ga, &gb);
        let (dm, dp, dn) = desvio_cor(&ga, &gb, &pop_d);
        let (wm, wp, _) = desvio_cor(gr, &gw, &pop);
        let frac = |n: usize, p: &[usize]| n as f64 / p.len() as f64;
        eprintln!(
            "{}: tinta {} · cor máx {cm:.3} p99.9 {cp:.4} frac {:.2e} · CPU×ulp máx {cu_m:.3} p99.9 {cu_p:.4} ({cu_n} cél.) · A×B p99.9 {uu_p:.4} · CPU×B p99.9 {nb_p:.4} · \
             ref {nome_ref} · devolvido máx {dm:.3} p99.9 {dp:.4} frac {:.2e} · ciclo máx {wm:.3} p99.9 {wp:.4} · bit {bit} · ciclo {bit_ciclo} · \
             massa rel {rel:.2e} · V rel {rel_v:.2e}",
            c.nome,
            pop.len(),
            frac(acima, &pop),
            frac(dn, &pop_d),
        );

        // Os campos, um a um, na população com tinta (o film, onde há tinta ou água).
        let pop_film: Vec<usize> = (0..gc.cells)
            .filter(|&i| gr.film[i] > 0.0 || gg.film[i] > 0.0 || gr.susp[i] + gr.sett[i] > 0.0)
            .collect();
        let tem = |i: usize| gr.susp[i] + gr.sett[i] > 0.0 && gg.susp[i] + gg.sett[i] > 0.0;
        let pop_meio: Vec<usize> = pop.iter().copied().filter(|&i| tem(i)).collect();
        let p999 = |k: usize, g: &Grid, p: &[usize]| {
            desvio_em(p, |i| {
                f64::from(campos(gr, i)[k]) - f64::from(campos(g, i)[k])
            })
            .1
        };
        for (nome, k, teto) in [
            ("film", 0, FILM_P999),
            ("susp", 1, MASSA_P999),
            ("sett", 2, MASSA_P999),
            ("filme.v", 3, SOMA_P999),
            ("filme.L", 4, SOMA_P999),
            ("filme.a", 5, SOMA_P999),
            ("filme.b", 6, SOMA_P999),
            ("filme.t", 7, SOMA_P999),
            ("filme.m", 8, SOMA_P999),
            ("aberto.v", 9, SOMA_P999),
            ("aberto.L", 10, SOMA_P999),
            ("aberto.a", 11, SOMA_P999),
            ("aberto.b", 12, SOMA_P999),
            ("aberto.t", 13, SOMA_P999),
            ("aberto.m", 14, SOMA_P999),
            ("meio", 15, MEIO_P999),
        ] {
            let p: &[usize] = match k {
                0 => &pop_film,
                15 => &pop_meio,
                _ => &pop,
            };
            let d = p999(k, &gg, p);
            eprintln!("   {nome:<10} p99.9 {d:.3e} · teto {teto:.1e}");
            if d > teto {
                falhas.push(format!("{}: {nome} p99.9 {d:.3e} > {teto:.1e}", c.nome));
            }
        }
        // A abertura só onde há aberto que ela governe: em `v ~ 1e-39` a placa zera o subnormal e o `o` de cada lado não decide nada.
        let pop_o: Vec<usize> = pop
            .iter()
            .copied()
            .filter(|&i| {
                em(gr, i).aberto.v > ABERTO_QUE_CONTA || em(&gg, i).aberto.v > ABERTO_QUE_CONTA
            })
            .collect();
        let d_o = |g: &Grid| desvio_em(&pop_o, |i| f64::from(em(gr, i).o) - f64::from(em(g, i).o));
        let ((om, op), (wom, wop)) = (d_o(&gg), d_o(&gw));
        eprintln!(
            "   abertura   máx {om:.3e} p99.9 {op:.3e} · ciclo máx {wom:.3e} p99.9 {wop:.3e} · células {}",
            pop_o.len()
        );
        if op > ABERTURA_P999 || wop > ABERTURA_P999 || om > ABERTURA_MAX || wom > ABERTURA_MAX {
            falhas.push(format!(
                "{}: abertura máx {om:.3e} p99.9 {op:.3e}, ciclo máx {wom:.3e} p99.9 {wop:.3e}",
                c.nome
            ));
        }
        // O espelho sRGB, onde a lei nova o escreve (a fatia tem tinta dos dois lados).
        let d_e = desvio_em(&pop, |i| {
            if em(gr, i).aberto.v <= 0.0 || em(&gg, i).aberto.v <= 0.0 {
                return 0.0;
            }
            (0..3)
                .map(|k| f64::from((gr.sett_rgb[i][k] - gg.sett_rgb[i][k]).abs()))
                .fold(0.0, f64::max)
        })
        .1;
        eprintln!("   espelho    p99.9 {d_e:.5e}");
        if d_e > ESPELHO_P999 {
            falhas.push(format!("{}: espelho p99.9 {d_e:.3e}", c.nome));
        }
        if bit != 0 {
            falhas.push(format!("{}: {bit} células mudam entre corridas", c.nome));
        }
        if bit_ciclo != 0 {
            falhas.push(format!(
                "{}: {bit_ciclo} células diferem entre o ciclo do worker e a descida inteira",
                c.nome
            ));
        }
        let cabe = |m: f64, p: f64, n: usize, pp: &[usize]| {
            m <= COR_MAX && p <= COR_P999 && frac(n, pp) <= FRACAO_NA_FRANJA
        };
        if !cabe(cm, cp, acima, &pop) {
            falhas.push(format!(
                "{}: cor máx {cm} p99.9 {cp}, {acima} de {} células",
                c.nome,
                pop.len()
            ));
        }
        if rel > MASSA_REL || rel_v > MASSA_REL {
            falhas.push(format!(
                "{}: massa rel {rel:.2e}, V rel {rel_v:.2e}",
                c.nome
            ));
        }
        if !cabe(dm, dp, dn, &pop_d) {
            falhas.push(format!(
                "{}: devolvido, cor máx {dm} p99.9 {dp}, {dn} de {} células",
                c.nome,
                pop_d.len()
            ));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}
