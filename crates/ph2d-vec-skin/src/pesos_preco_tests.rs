//! ⭐ **O PREÇO do campo por ETAPA** (A3, 2026-10-04): prender uma forma com um *Repeater* `39 × 39`
//! que gira (`~1 000` contornos) não acabava em DEBUG em 10 min. Sonda; corra em `--release` com o
//! `loadavg` ao lado.

use super::{contornos_fechados, malha_do_dominio_com_regua, pesos_dos_pontos};
use ph2d_skin_weights::Handle;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{ShapeKind, cook_tinted};
use std::time::Instant;

fn barra(copias: f64) -> ph2d_vec_scene::VecPath {
    let mut b = cook_tinted(ShapeKind::RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5], [200, 140, 60]);
    if copias > 1.0 {
        b.effects = vec![FxEntry::new(PathEffect::Repeat(ph2d_vec_scene::fx_repeat::RepeatSpec {
            copies_x: copias,
            move_x: -80.0,
            copies_y: copias,
            move_y: -80.0,
            spin: -72.0,
            orbit: -72.0,
        }))];
    }
    b.cooked().into_owned()
}

#[test]
#[ignore = "sonda de preço: --release, máquina calma"]
fn diag_o_preco_do_campo_por_etapa() {
    let passo = (-1.8f64 - -8.2) / 3.0;
    let ossos: Vec<Handle> = (0..3)
        .map(|k| Handle {
            a: [-8.2 + passo * f64::from(k), 2.5],
            b: [-8.2 + passo * f64::from(k + 1), 2.5],
        })
        .collect();
    for copias in [1.0, 5.0, 13.0, 25.0, 39.0] {
        let p = barra(copias);
        let t = Instant::now();
        let aneis = contornos_fechados(&p);
        let t_aneis = t.elapsed();
        let t = Instant::now();
        let Some((malha, _)) = malha_do_dominio_com_regua(&aneis, &ossos) else {
            println!("  {copias}²: sem malha");
            continue;
        };
        let t_malha = t.elapsed();
        let t = Instant::now();
        let w = ph2d_skin_weights::bounded_biharmonic(&malha, &ossos, ph2d_skin_weights::Options::default());
        let t_solver = t.elapsed();
        let t = Instant::now();
        let campo = super::campo_do_caminho(&p, &ossos);
        let t_campo = t.elapsed();
        let t = Instant::now();
        let n = campo.as_ref().map_or(0, |c| pesos_dos_pontos(&p, c).len());
        let t_tabela = t.elapsed();
        println!(
            "  {copias}²: {} contornos, {} pontos de anel · malha {} vértices {} triângulos · \
             ms: anéis {:.1} · malha {:.1} · solver {:.1} ({}) · campo inteiro {:.1} · tabela {:.1} ({n}) · loadavg {}",
            aneis.len(),
            aneis.iter().map(Vec::len).sum::<usize>(),
            malha.rest.len(),
            malha.tris.len(),
            t_aneis.as_secs_f64() * 1e3,
            t_malha.as_secs_f64() * 1e3,
            t_solver.as_secs_f64() * 1e3,
            if w.is_some() { "ok" } else { "None" },
            t_campo.as_secs_f64() * 1e3,
            t_tabela.as_secs_f64() * 1e3,
            std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim()
        );
    }
}

/// ⭐⭐⭐ **GATE — o índice dos anéis responde como a VARREDURA**, nas duas perguntas da grelha, em
/// toda célula de `4` px da caixa de `512` (e com folga), numa forma de um anel e num *Repeater*
/// `13 × 13` que gira. ⛔ **O CONTROLO:** as duas respostas aparecem (há dentro e fora, toca e não
/// toca) — senão a igualdade não mediria nada.
#[test]
fn o_indice_dos_aneis_responde_como_a_varredura() {
    for copias in [1.0, 13.0] {
        let p = barra(copias);
        let aneis = contornos_fechados(&p);
        let (mut cx, mut cy, mut mx, mut my) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for q in aneis.iter().flatten() {
            cx = cx.min(q[0]);
            cy = cy.min(q[1]);
            mx = mx.max(q[0]);
            my = my.max(q[1]);
        }
        let s = 512.0 / (mx - cx).max(my - cy);
        let am: Vec<Vec<[f64; 2]>> = aneis
            .iter()
            .map(|a| a.iter().map(|q| [(q[0] - cx) * s, (q[1] - cy) * s]).collect())
            .collect();
        let ix = super::aneis::IndiceDosAneis::novo(&am);
        let (mut dentro, mut toca, mut n) = (0, 0, 0);
        for j in -2..130 {
            for i in -2..130 {
                let (x0, y0) = (f64::from(i) * 4.0 + 0.37, f64::from(j) * 4.0 + 0.61);
                let q = [x0 + 1.3, y0 + 2.1];
                let d = super::dentro(&am, q);
                assert_eq!(ix.dentro(&am, q), d, "{copias}²: dentro difere em {q:?}");
                let t = am.iter().any(|a| {
                    (0..a.len()).any(|k| super::cruza_a_celula(a[k], a[(k + 1) % a.len()], x0, y0, x0 + 4.0, y0 + 4.0))
                });
                assert_eq!(ix.toca(&am, x0, y0, x0 + 4.0, y0 + 4.0), t, "{copias}²: toca difere em {x0},{y0}");
                dentro += usize::from(d);
                toca += usize::from(t);
                n += 1;
            }
        }
        println!("  {copias}²: {n} células · dentro {dentro} · toca {toca}");
        assert!(dentro > 0 && dentro < n && toca > 0 && toca < n, "o CONTROLO: uma resposta só");
    }
}
