//! ⭐ A pilha da aba ([`ph2d_panel_code::stack`]): o código tem a prioridade. Numa aba curta os
//! cartões encolhem pela ordem até ao mínimo, depois até ao cabeçalho, e por fim saem (o depurador
//! por último) até o código guardar as suas linhas; em toda a altura os rects não se cruzam nem saem
//! da aba (a janela de 900×560, 11/11, punha a Bancada por cima do depurador e o código a zero).

use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::stack::{Placed, Want, fit, tab};

const fn w(h: f32, least: f32, head: f32) -> Want {
    Want { h, least, head }
}

/// A Bancada, o depurador e a faixa, nesta ordem (a de [`tab`]).
const WANTS: [Want; 3] = [
    w(120.0, 40.0, 30.0),
    w(200.0, 50.0, 26.0),
    w(150.0, 56.0, 28.0),
];
const ORDER: [usize; 3] = [0, 2, 1];
const GAP: f32 = 8.0;
const CODE: f32 = 100.0;

fn used(hs: &[f32]) -> f32 {
    hs.iter().filter(|h| **h > 0.0).map(|h| h + GAP).sum()
}

#[test]
fn com_espaco_cada_cartao_fica_como_quer() {
    assert_eq!(fit(900.0, CODE, GAP, WANTS, ORDER), [120.0, 200.0, 150.0]);
}

#[test]
fn degrau_1_encolhem_pela_ordem_ate_ao_minimo_e_o_codigo_guarda_o_seu() {
    // 400 - 100 de código = 300 para cartões; querem 120 + 200 + 150 + 3 vãos de 8 = 494.
    let hs = fit(400.0, CODE, GAP, WANTS, ORDER);
    assert_eq!(hs[0], 40.0, "a Bancada encolhe primeiro, até ao mínimo");
    assert_eq!(hs[2], 150.0, "a faixa só encolhe se o resto não chega");
    assert!(
        (used(&hs) - 300.0).abs() < 1e-3,
        "o código fica com os seus 100: {hs:?}"
    );
}

#[test]
fn degrau_2_sem_os_minimos_caem_ao_cabecalho_pela_ordem() {
    // Os mínimos: 40 + 50 + 56 + 24 = 170; os cabeçalhos: 30 + 26 + 28 + 24 = 108.
    let hs = fit(CODE + 150.0, CODE, GAP, WANTS, ORDER);
    assert_eq!(hs[0], 30.0, "a Bancada: só o título");
    assert_eq!(hs[2], 56.0, "a faixa fica no mínimo");
    assert!((used(&hs) - 150.0).abs() < 1e-3, "{hs:?}");
    assert_eq!(
        fit(CODE + 108.0, CODE, GAP, WANTS, ORDER),
        [30.0, 26.0, 28.0]
    );
}

#[test]
fn degrau_3_saem_a_bancada_a_faixa_e_por_fim_o_depurador() {
    // Sem a Bancada: 26 + 28 + 16 = 70 cabe em 80, e o que sobra volta à faixa (até ao mínimo).
    let hs = fit(CODE + 80.0, CODE, GAP, WANTS, ORDER);
    assert_eq!(hs[0], 0.0, "{hs:?}");
    assert_eq!(hs[1], 26.0, "{hs:?}");
    assert!((used(&hs) - 80.0).abs() < 1e-3, "{hs:?}");
    assert_eq!(
        fit(CODE + 34.0, CODE, GAP, WANTS, ORDER),
        [0.0, 26.0, 0.0],
        "a barra fica"
    );
    assert_eq!(
        fit(CODE + 20.0, CODE, GAP, WANTS, ORDER),
        [0.0; 3],
        "nem a barra cabe"
    );
    assert_eq!(fit(CODE - 30.0, CODE, GAP, WANTS, ORDER), [0.0; 3]);
    // Um cartão ausente (0) não paga vão nem volta.
    let hs = fit(
        CODE + 60.0,
        CODE,
        GAP,
        [w(0.0, 40.0, 30.0), WANTS[1], WANTS[2]],
        ORDER,
    );
    assert_eq!(hs[0], 0.0);
    assert!(used(&hs) <= 60.0 + 1e-3, "{hs:?}");
}

fn rects(p: &Placed) -> Vec<Rect> {
    let mut v = vec![p.code];
    v.extend(p.debug);
    v.extend(p.bench);
    v.extend((p.band.h > 0.0).then_some(p.band));
    v
}

#[test]
fn em_toda_a_altura_os_rects_nao_se_cruzam_nem_saem_e_o_codigo_guarda_o_seu() {
    let heads: f32 = WANTS.iter().map(|w| w.head + GAP).sum();
    for (bench, band) in [(true, true), (false, true), (true, false)] {
        let wants = [
            if bench { WANTS[0] } else { Want::default() },
            WANTS[1],
            if band { WANTS[2] } else { Want::default() },
        ];
        for h in (120..=720).step_by(20).map(|h| h as f32 / 2.0) {
            let area = Rect::new(10.0, 30.0, 400.0, h);
            let p = tab(area, CODE, GAP, wants);
            let rs = rects(&p);
            let bottom = area.y + area.h + 1e-3;
            for (i, r) in rs.iter().enumerate() {
                assert!(
                    r.y >= area.y && r.y + r.h <= bottom && r.h >= 0.0,
                    "{h}: {r:?}"
                );
                if let Some(n) = rs.get(i + 1) {
                    assert!(n.y >= r.y + r.h, "{h}: cruzam {r:?} {n:?}");
                }
            }
            if h >= CODE + heads {
                assert!(p.code.h >= CODE - 1e-3, "{h}: {p:?}");
                assert!(p.debug.is_some(), "{h}: a barra do depurador fica: {p:?}");
            }
            if h < CODE {
                assert_eq!(rs, vec![p.code], "{h}: só o código");
                assert_eq!(p.code.h, h);
            }
        }
    }
}
