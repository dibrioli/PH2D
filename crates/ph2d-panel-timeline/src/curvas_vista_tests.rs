use super::*;
use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind};
use ph2d_tokens::ColorToken;

fn faixa(keys: &[(f64, f32)]) -> TrackView {
    TrackView {
        target: AnimTarget::new(1),
        prop: PropKind::TranslationX,
        entity: 1,
        missing: false,
        keys: keys
            .iter()
            .enumerate()
            .map(|(i, &(t, v))| KeyView {
                id: KeyId::new(i as u64 + 1),
                t_seconds: t,
                value: v,
                interp: Interp::Linear,
                selected: false,
                roving: false,
            })
            .collect(),
        buffer_ghost: None,
        pre: ph2d_timeline::Extrap::Hold,
        post: ph2d_timeline::Extrap::Hold,
        expr: None,
    }
}

/// ⭐ A curva secundária normaliza-se pelo PRÓPRIO intervalo: o mínimo toca o fundo e o máximo o topo
/// do eixo direito (o 0 e o 1), qualquer que seja a unidade. *Mutação: `Band::fit` (com folga) no lugar
/// de `from_range` ⇒ não toca as bordas ⇒ vermelho.*
#[test]
fn a_curva_secundaria_cabe_no_zero_um_do_eixo_direito() {
    let banda = Rect::new(100.0, 50.0, 400.0, 120.0);
    let view = TimeView {
        time_x: 100.0,
        right: 500.0,
        view_start: 0.0,
        px_per_s: 400.0,
    };
    for keys in [
        &[(0.0, -3000.0), (1.0, 5000.0)][..],
        &[(0.0, 0.2), (0.5, 0.9), (1.0, 0.4)],
    ] {
        let p = curva_normalizada(&banda, view, &faixa(keys));
        let (lo, hi) = p
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &(_, y)| (a.min(y), b.max(y)));
        assert!(
            (lo - banda.y).abs() < 0.01,
            "o máximo toca o topo (1): {lo}"
        );
        assert!(
            (hi - (banda.y + banda.h)).abs() < 0.01,
            "o mínimo toca o fundo (0): {hi}"
        );
    }
}

/// Os pisos OKLab das duas curvas sobre o fundo do grafo (`Bg0`, o da moldura) em todo tema.
#[test]
fn as_curvas_se_leem_sobre_o_fundo_do_grafo_em_todo_tema() {
    let lab = |c: ph2d_vector::Color| {
        let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let k = c.components;
        let (l, ch, h) = ph2d_tokens::srgb_to_oklch(u(k[0]), u(k[1]), u(k[2]));
        let h = h.to_radians();
        (l, ch * h.cos(), ch * h.sin())
    };
    let dist = |a, b| {
        let (x, y): ((f64, f64, f64), (f64, f64, f64)) = (lab(a), lab(b));
        ((x.0 - y.0).powi(2) + (x.1 - y.1).powi(2) + (x.2 - y.2).powi(2)).sqrt()
    };
    let mut pior = [(f64::MAX, Theme::Forge); 2];
    for theme in Theme::ALL {
        let fundo = resolve(ColorToken::Bg0, theme);
        let d = [
            dist(resolve(ColorToken::TimelineCurve, theme), fundo),
            dist(
                crate::folha_tinta::tinta(TimelineTinta::CurvaSecundaria, theme),
                fundo,
            ),
        ];
        for (p, d) in pior.iter_mut().zip(d) {
            if d < p.0 {
                *p = (d, theme);
            }
        }
    }
    eprintln!(
        "principal × fundo {:?} · secundária × fundo {:?}",
        pior[0], pior[1]
    );
    for (p, piso) in pior.iter().zip([PISO_PRINCIPAL, PISO_SECUNDARIA]) {
        assert!(p.0 >= piso, "{:.3} < {piso} em {:?}", p.0, p.1);
    }
}

const PISO_PRINCIPAL: f64 = 0.33; // medido 0,388 (Candy), 10/11
const PISO_SECUNDARIA: f64 = 0.37; // medido 0,433 (PlumberGreen), 10/11
