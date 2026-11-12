//! ⭐ A pilha da aba ([`ph2d_panel_code::stack`]): o código tem a prioridade. Numa aba curta os
//! cartões encolhem pela ordem até ao mínimo, depois até ao cabeçalho, e por fim saem (o depurador
//! por último) até o código guardar as suas linhas; em toda a altura os rects não se cruzam nem saem
//! da aba (a janela de 900×560, 11/11, punha a Bancada por cima do depurador e o código a zero).

use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::stack::{Placed, Want, fit, tab};

const fn w(h: f32, least: f32, head: f32) -> Want {
    Want {
        h,
        least,
        head,
        asked: false,
    }
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
    // Cada um cai INTEIRO ao cabeçalho (nada de corpo meio cortado); o que sobra vai ao código.
    assert_eq!(
        hs,
        [30.0, 26.0, 56.0],
        "a Bancada e o depurador ao cabeçalho, a faixa no mínimo"
    );
    assert!(used(&hs) <= 150.0, "{hs:?}");
    assert_eq!(
        fit(CODE + 108.0, CODE, GAP, WANTS, ORDER),
        [30.0, 26.0, 28.0]
    );
}

#[test]
fn degrau_3_saem_a_bancada_a_faixa_e_por_fim_o_depurador() {
    // Sem a Bancada: 26 + 28 + 16 = 70 cabe em 80; os mínimos (50 + 56 + 16) não.
    assert_eq!(fit(CODE + 80.0, CODE, GAP, WANTS, ORDER), [0.0, 26.0, 28.0]);
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

/// ⛔ O mínimo do código mede-se em LINHAS VISÍVEIS pelo layout real (o poço dentro do cartão, a
/// busca, o respiro de cima e a barra de estado), não pela fórmula que o próprio mínimo usa.
#[test]
fn o_minimo_do_codigo_mostra_as_suas_linhas_inteiras() {
    use crate::{Deco, pinta, texto, ts};
    use ph2d_code_buffer::Buffer;
    use ph2d_panel_code::CodeView;
    use ph2d_panel_code::stack::{CODE_LEAST_ROWS, code_least};
    let buf = Buffer::new(&texto(40, 30));
    let (mut v, mut ts) = (CodeView::new(), ts());
    pinta(
        &mut v,
        &buf,
        &mut ts,
        Rect::new(0.0, 0.0, 600.0, 400.0),
        &Deco::default(),
    );
    let m = *v.metrics();
    for find_h in [0.0, 30.0] {
        let card = Rect::new(0.0, 0.0, 600.0, code_least(find_h));
        let well = ph2d_panel_code::cards::code_well(card, find_h);
        pinta(&mut v, &buf, &mut ts, well, &Deco::default());
        let tr = v.text_rect();
        let mut inteiras: Vec<usize> = v
            .probe_columns()
            .into_iter()
            .filter(|(_, mid)| {
                *mid - m.line_h * 0.5 >= tr.y - 0.01 && *mid + m.line_h * 0.5 <= tr.y + tr.h + 0.01
            })
            .map(|(c, _)| buf.text().line_of_byte(c.byte))
            .collect();
        inteiras.dedup();
        assert!(
            inteiras.len() >= CODE_LEAST_ROWS,
            "busca {find_h}: {} linhas inteiras à vista em {tr:?} (mínimo {CODE_LEAST_ROWS})",
            inteiras.len()
        );
    }
}

/// ⛔ A faixa que o artista PEDIU (aberta, ou o ⊗/⚠ da barra de estado clicado) guarda o cabeçalho e
/// uma linha em toda a altura onde isso cabe: numa aba baixa com o depurador parado o clique no
/// contador não mostrava nada (a faixa saía antes da barra do depurador). Saem antes a Bancada e o
/// corpo do depurador; a barra com Continue fica se ainda couber.
#[test]
fn a_faixa_pedida_fica_em_toda_a_altura_onde_cabe_o_minimo() {
    let band = Want {
        asked: true,
        ..w(150.0, 56.0, 56.0)
    };
    let wants = [WANTS[0], WANTS[1], band];
    for h in (120..=720).step_by(5).map(|h| h as f32 / 2.0) {
        let p = tab(Rect::new(0.0, 0.0, 400.0, h), CODE, GAP, wants);
        if h >= CODE + band.least + GAP {
            assert!(
                p.band.h >= band.least - 1e-3,
                "{h}: a faixa pedida saiu: {p:?}"
            );
        }
        if h >= CODE + band.least + WANTS[1].head + GAP * 2.0 {
            assert!(p.debug.is_some(), "{h}: a barra do depurador cabe: {p:?}");
        }
        if p.debug.is_some_and(|d| d.h > WANTS[1].head) {
            assert!(
                p.band.h >= band.least,
                "{h}: o corpo do depurador antes da faixa: {p:?}"
            );
        }
    }
    // Controlo: a faixa NÃO pedida sai antes da barra do depurador (a ordem de sempre).
    let p = tab(
        Rect::new(0.0, 0.0, 400.0, CODE + 60.0),
        CODE,
        GAP,
        [WANTS[0], WANTS[1], w(150.0, 56.0, 28.0)],
    );
    assert!(p.debug.is_some() && p.band.h == 0.0, "{p:?}");
}

/// A janela de 900×560 (a aba de ≈225 px): o código nas suas linhas, a barra do depurador e a faixa
/// pedida (cabeçalho e uma linha) cabem juntos — medido com as métricas e os tokens reais.
#[test]
fn na_janela_de_900x560_cabem_o_codigo_a_barra_e_a_faixa_pedida() {
    use crate::{Deco, pinta, texto, ts};
    use ph2d_code_buffer::Buffer;
    use ph2d_panel_code::CodeView;
    use ph2d_panel_code::stack::code_least;
    let buf = Buffer::new(&texto(4, 20));
    let (mut v, mut ts) = (CodeView::new(), ts());
    pinta(
        &mut v,
        &buf,
        &mut ts,
        Rect::new(0.0, 0.0, 600.0, 400.0),
        &Deco::default(),
    );
    let code = code_least(0.0);
    let gap = ph2d_panel_code::cards::margin();
    let bar = ph2d_tokens::row_pitch_px() + ph2d_tokens::Spacing::Sm.px() * 2.0;
    let band = ph2d_tokens::ROW_H_PX * 2.0;
    let need = code + bar + band + gap * 2.0;
    eprintln!(
        "900x560: código {code} + barra {bar} + faixa {band} + vãos {} = {need}",
        gap * 2.0
    );
    let p = tab(
        Rect::new(0.0, 0.0, 600.0, 225.0),
        code,
        gap,
        [
            w(120.0, 40.0, 30.0),
            w(200.0, 80.0, bar),
            Want {
                asked: true,
                ..w(150.0, band, band)
            },
        ],
    );
    assert!(p.band.h >= band - 1e-3, "a faixa pedida à vista: {p:?}");
    assert!(
        p.code.h >= code - 1e-3,
        "o código guarda as suas linhas: {p:?}"
    );
    if need <= 225.0 {
        assert!(p.debug.is_some(), "a barra cabe ({need} ≤ 225): {p:?}");
    }
}

/// ⛔ O `want` REAL da faixa (não um cartão inventado): aberta pelo artista, pede-se (`asked`) e
/// o cabeçalho é o mínimo dela (cabeçalho e uma linha); fechada, não se pede.
#[test]
fn a_faixa_aberta_pede_se_e_guarda_o_minimo_como_cabecalho() {
    let c = ph2d_panel_code::ConsoleView::default();
    for area_h in [240.0, 600.0, 1200.0] {
        let aberta = c.want(true, area_h);
        assert!(aberta.asked, "aberta pelo artista: {aberta:?}");
        assert_eq!(
            aberta.head, aberta.least,
            "o cabeçalho é o mínimo: {aberta:?}"
        );
        assert!(
            aberta.least < aberta.h,
            "o mínimo cabe abaixo do que quer: {aberta:?}"
        );
        let fechada = c.want(false, area_h);
        assert!(!fechada.asked, "fechada não se pede: {fechada:?}");
        assert_eq!(
            fechada.head, fechada.h,
            "fechada é só o cabeçalho: {fechada:?}"
        );
    }
}
