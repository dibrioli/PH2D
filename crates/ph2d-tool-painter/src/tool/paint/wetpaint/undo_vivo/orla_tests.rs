//! B12.2: o enterro de um passo desfeito para lá do tecto anda só a caixa do toque, ao bit do da grade inteira.

use super::super::super::undo_tinta_morre_tests::{AGUADA, tela, teto, traco};
use super::*;

/// Junta a célula `(x, y)` à caixa `c` (0-based, inclusiva).
fn junta(
    c: Option<(usize, usize, usize, usize)>,
    (x, y): (usize, usize),
) -> Option<(usize, usize, usize, usize)> {
    Some(c.map_or((x, y, x, y), |(a, b, e, d)| {
        (a.min(x), b.min(y), e.max(x), d.max(y))
    }))
}

#[test]
fn perto_na_caixa_da_o_bit_do_antigo() {
    let (gw, gh) = (90, 70);
    let casos: [&[(usize, usize)]; 6] = [
        &[(0, 0)],
        &[(89, 69)],
        &[(45, 35)],
        &[(3, 60), (80, 2)],
        &[(10, 10), (11, 10), (40, 50)],
        &[(0, 34), (89, 35)],
    ];
    for r in [0, 1, 7, ALCANCE_DA_ORLA] {
        for cs in casos {
            let (mut m, mut toque) = (vec![false; gw * gh], None);
            for &(x, y) in cs {
                m[y * gw + x] = true;
                toque = junta(toque, (x, y));
            }
            let (velho, caixa) = antigo::perto_de(&m, gw, gh, r);
            let p = perto_na_caixa(&m, toque, gw, gh, r).expect("fixture: há marcadas");
            let mut novo = vec![false; gw * gh];
            for (y, x0, l) in p.linhas() {
                novo[y * gw + x0..y * gw + x0 + l.len()].copy_from_slice(l);
            }
            assert_eq!(Some(p.caixa), caixa, "a caixa, r {r}, {cs:?}");
            assert!(novo == velho, "as células perto, r {r}, {cs:?}");
        }
    }
    assert!(perto_na_caixa(&vec![false; gw * gh], None, gw, gh, 4).is_none());
}

type Depois = (u64, usize, Vec<u32>, Vec<u32>, Vec<u8>, Vec<u8>);

/// Três traços curtos, o tecto de um passo: o 2.º undo desfaz para lá dele (`wet_enterra_sem_passo`). Devolve as
/// células que a orla andou nesse undo, o tamanho da grade, a grade, a tela e a `base` depois dele.
fn desfaz_para_la_do_tecto(velho: bool) -> Depois {
    antigo::set(velho);
    teto::set(Some(1));
    let mut t = tela(1800, 1400, AGUADA, 40.0);
    traco(&mut t, 380.0, 470.0, 180.0); // a tinta que fica
    traco(&mut t, 380.0, 470.0, 300.0);
    traco(&mut t, 380.0, 470.0, 420.0);
    assert!(t.undo_last(), "o passo do topo, com fotografia");
    let (gw, gh) = t.paint.wetpaint.session.as_ref().expect("a sessão").grid;
    VISITAS.with(|v| v.set(0));
    assert!(t.undo_last(), "para lá do tecto");
    let visitas = VISITAS.with(|v| v.get());
    teto::set(None);
    antigo::set(false);
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let g = &s.engine.layers[0].grid;
    let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<_>>();
    (
        visitas,
        gw * gh,
        bits(&g.sett),
        bits(&g.susp),
        t.canvas_rgba.to_vec(),
        s.base.to_vec(),
    )
}

/// ⭐ B12.2: a orla do traço desfeito (as células perto do toque e as invisíveis) anda a caixa dele, não a grade.
#[test]
fn para_la_do_tecto_o_enterro_da_orla_anda_so_a_caixa() {
    let (visitas, celulas, ..) = desfaz_para_la_do_tecto(false);
    assert!(
        visitas > 0,
        "fixture: o undo para lá do tecto enterra a orla"
    );
    assert!(
        visitas * 4 <= celulas as u64,
        "a orla de um traço curto andou {visitas} células (a grade tem {celulas})"
    );
}

/// B12.2: pela caixa a grade, a tela e a `base` são as do enterro da grade inteira, ao bit.
#[test]
fn o_enterro_sem_passo_na_caixa_da_o_bit_do_antigo() {
    let (_, _, sett, susp, tela, base) = desfaz_para_la_do_tecto(false);
    let (_, _, sett_a, susp_a, tela_a, base_a) = desfaz_para_la_do_tecto(true);
    assert!(
        sett.iter().chain(&susp).any(|v| *v != 0),
        "fixture: há tinta"
    );
    assert!(sett == sett_a && susp == susp_a, "a grade");
    assert!(tela == tela_a, "a tela");
    assert!(base == base_a, "a base");
}
