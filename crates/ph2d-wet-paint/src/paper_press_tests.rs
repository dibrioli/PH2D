//! ⭐ **Paper Press** (dono 28/10, `project-memory/project_paper_press_is_a_brush_control_2026_10_28.md`): o
//! controlo do pincel que leva do «segue o papel» (`0`: o dente do papel na ALTURA e na SOMBRA da tinta, como a
//! tinta fina sobre a grossa) ao «amassa» de sempre (`1`, a omissão, ao bit).

use super::transmissao_tests::{cel, lisa_sobre_papel};
use crate::grid::Grid;
use crate::oleo_real::{CampoAltura, OleoReal};

const W: usize = 48;

fn campo(g: &Grid) -> CampoAltura {
    let o = OleoReal {
        dente_papel: 1.0,
        ..OleoReal::default()
    };
    CampoAltura::de(&[(g, true)], &o, 1, 1, 1, W, W)
}

/// A correlação de Pearson entre `a` e `b`.
fn correlacao(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    sab / (saa * sbb).sqrt()
}

/// ⭐ O GATE-LEI: com `1` o campo é o de hoje AO BIT (a tinta sem o campo do material); com `0` o dente do papel
/// aparece DENTRO da tinta lisa na altura fina e na da sombra (a diferença segue o papel, correlação > 0,99) e o
/// papel nu continua plano; `½` é metade do caminho.
#[test]
fn o_paper_press_poe_o_dente_do_papel_na_altura_e_na_sombra() {
    let lisa = lisa_sobre_papel();
    let hoje = campo(&lisa);
    let com = |p: f32| campo(&cel(&lisa, |m| m.amassa_papel = p));
    let (um, meio, zero) = (com(1.0), com(0.5), com(0.0));
    for y in 1..=W as isize {
        for x in 1..=W as isize {
            assert_eq!(
                hoje.at_fina(x, y).to_bits(),
                um.at_fina(x, y).to_bits(),
                "1 = hoje: altura ({x},{y})"
            );
            assert_eq!(
                hoje.at_sombra(x, y).to_bits(),
                um.at_sombra(x, y).to_bits(),
                "1 = hoje: sombra ({x},{y})"
            );
        }
    }
    let (mut papel, mut fina0, mut sombra0, mut sombra_meio) = (vec![], vec![], vec![], vec![]);
    for y in 14..=34isize {
        for x in 14..=34isize {
            papel.push(f64::from(lisa.paper[x as usize + y as usize * lisa.s]));
            fina0.push(zero.at_fina(x, y) - hoje.at_fina(x, y));
            sombra0.push(zero.at_sombra(x, y) - hoje.at_sombra(x, y));
            sombra_meio.push(meio.at_sombra(x, y) - hoje.at_sombra(x, y));
        }
    }
    let (cf, cs) = (correlacao(&papel, &fina0), correlacao(&papel, &sombra0));
    let amp = |v: &[f64]| {
        v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
    };
    let (a0, am) = (amp(&sombra0), amp(&sombra_meio));
    println!(
        "[paper press] correlação com o papel: altura {cf:.4} · sombra {cs:.4}; amplitude 0 {a0:.4} · ½ {am:.4}"
    );
    assert!(
        cf > 0.99 && cs > 0.99,
        "com 0 o dente do papel está na altura e na sombra: {cf} · {cs}"
    );
    assert!(a0 > 0.4, "com 0 o dente tem a amplitude do papel: {a0}");
    assert!(
        (am / a0 - 0.5).abs() < 0.02,
        "½ é metade do caminho: {am} ÷ {a0}"
    );
    assert_eq!(zero.at_fina(2, 2), 0.0, "o papel nu continua plano");
}

/// A tinta lisa sobre um papel liso em PX (`0,5 + 0,5·sin(0,13·px)·cos(0,07·py)`), numa grade de `48 ÷ razao` células.
fn lisa_em_px(razao: usize) -> Grid {
    let n = W / razao;
    let mut g = Grid::new(n, n);
    for y in 0..g.h + 2 {
        for x in 0..g.s {
            let i = x + y * g.s;
            if i < g.paper.len() {
                let (px, py) = ((x * razao) as f32, (y * razao) as f32);
                g.paper[i] = 0.5 + 0.5 * (px * 0.13).sin() * (py * 0.07).cos();
            }
        }
    }
    for y in 2..=n - 1 {
        for x in 2..=n - 1 {
            let i = x + y * g.s;
            g.sett[i] = 6000.0;
            g.sett_rgb[i] = [40.0, 70.0, 200.0];
        }
    }
    g
}

/// ⭐ O Paper Press fala em PX (a altura em unidades de célula, como a [`OleoReal::profundidade`]): o dente que o
/// Paper Press 0 põe na altura e na sombra, o mesmo papel em px, a razão 1 e 2, é o MESMO em px (a altura da célula ×
/// a razão) em todo o miolo; tolerância `1e-5` (medido: `0`, dente máximo `1,83` px). Sem a divisão pela
/// razão a razão 2 dava o DOBRO.
#[test]
fn o_paper_press_e_o_mesmo_em_px_a_toda_a_razao() {
    let o = OleoReal {
        dente_papel: 1.0,
        ..OleoReal::default()
    };
    let dente = |razao: usize| {
        let lisa = lisa_em_px(razao);
        let n = W / razao;
        let campo = |g: &Grid| CampoAltura::de(&[(g, true)], &o, razao, 1, 1, n, n);
        let (hoje, zero) = (campo(&lisa), campo(&cel(&lisa, |m| m.amassa_papel = 0.0)));
        move |x: isize, y: isize| {
            let r = razao as f64;
            (
                (zero.at_fina(x, y) - hoje.at_fina(x, y)) * r,
                (zero.at_sombra(x, y) - hoje.at_sombra(x, y)) * r,
            )
        }
    };
    let (um, dois) = (dente(1), dente(2));
    let (mut pior, mut amp) = (0.0f64, 0.0f64);
    for y in 8..=16isize {
        for x in 8..=16isize {
            let (a, b) = (um(2 * x, 2 * y), dois(x, y));
            pior = pior.max((a.0 - b.0).abs()).max((a.1 - b.1).abs());
            amp = amp.max(a.0.abs()).max(a.1.abs());
        }
    }
    println!(
        "[paper press px] maior diferença razão 1 × 2: {pior:.2e} px; maior dente {amp:.4} px"
    );
    assert!(
        amp > 0.1,
        "controlo: o Paper Press 0 põe o dente na altura ({amp})"
    );
    assert!(
        pior <= 1e-5,
        "o dente em px: razão 1 × razão 2 diferem {pior}"
    );
}
