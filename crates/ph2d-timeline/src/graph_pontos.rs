//! ⭐ **Os pontos da curva Custom no gráfico da timeline** (dono 15/11: «permita criar pontos no
//! gráfico da timeline e editar sem criar keyframes, exatamente igual é feito no easing custom»).
//!
//! A curva de um trecho mora no quadrado unitário (`EaseSpline`); o gráfico pinta-a no plano
//! `(tempo, valor)`: `t = t0 + x·Δt`, `v = v0 + y·Δv`. Estas duas contas e a pergunta «este trecho
//! tem curva editável?» são as únicas que o painel precisa — o resto (criar, mover, apagar, tipos de
//! alça) é a MESMA `EaseSpline` que o cartão da chave edita.

use ph2d_anim::{EaseSpline, Interp};

use crate::snapshot::KeyView;

/// A curva editável do trecho `k0 → k1` no quadrado unitário, ou `None` quando o gráfico não a
/// mostra: trecho RETIMADO (a curva desenhada é `E(W(u))`, não a do easing — os pontos ficariam
/// fora dela), sem duração, PARADO (`Δv = 0`: o eixo do valor não tem escala) ou numa família sem
/// forma exata de Bézier (Hold, Nearest, os presets Ease, Bounce…). A `Linear` é a Bézier
/// `(⅓,⅓)(⅔,⅔)` exata, e entra: é onde o primeiro ponto mais se cria.
#[must_use]
pub fn segment_curve(k0: &KeyView, k1: &KeyView) -> Option<EaseSpline> {
    let dv = f64::from(k1.value - k0.value);
    if k0.retime.is_some() || k1.t_seconds <= k0.t_seconds || dv.abs() <= f64::EPSILON {
        return None;
    }
    let f = |v: f64| v as f32;
    match k0.interp {
        Interp::Spline(sp) => Some(sp),
        Interp::Bezier { x1, y1, x2, y2 } => {
            Some(EaseSpline::from_bezier(f(x1), f(y1), f(x2), f(y2)))
        }
        Interp::BezierW { x1, dy1, x2, dy2 } => Some(EaseSpline::from_bezier(
            f(x1),
            f(dy1 / dv),
            f(x2),
            f(1.0 + dy2 / dv),
        )),
        Interp::Linear => Some(EaseSpline::from_bezier(
            1.0 / 3.0,
            1.0 / 3.0,
            2.0 / 3.0,
            2.0 / 3.0,
        )),
        _ => None,
    }
}

/// O ponto `[x, y]` do quadrado unitário no plano `(tempo, valor)` do trecho.
#[must_use]
pub fn unit_to_plane(k0: &KeyView, k1: &KeyView, [x, y]: [f32; 2]) -> (f64, f64) {
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let (span, dv) = (k1.t_seconds - t0, f64::from(k1.value) - v0);
    (t0 + f64::from(x) * span, v0 + f64::from(y) * dv)
}

/// O inverso de [`unit_to_plane`] — sem prender: quem guarda os invariantes é a `EaseSpline`
/// (`move_point`/`move_handle`). Só faz sentido onde [`segment_curve`] é `Some`.
#[must_use]
pub fn plane_to_unit(k0: &KeyView, k1: &KeyView, t: f64, v: f64) -> [f32; 2] {
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let (span, dv) = (k1.t_seconds - t0, f64::from(k1.value) - v0);
    [((t - t0) / span) as f32, ((v - v0) / dv) as f32]
}

/// A `Interp` de uma curva editada: sem pontos volta a ser a `Bezier` de duas alças.
#[must_use]
pub fn interp_of_curve(c: EaseSpline) -> Interp {
    if c.points().is_empty() {
        let ([x1, y1], [x2, y2]) = c.end_handles();
        let f = f64::from;
        Interp::bezier(f(x1), f(y1), f(x2), f(y2))
    } else {
        Interp::Spline(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::sample_keys;

    fn key(t: f64, v: f32, interp: Interp) -> KeyView {
        KeyView {
            retime: None,
            id: ph2d_anim::KeyId::new(0),
            t_seconds: t,
            value: v,
            interp,
            selected: false,
            roving: false,
        }
    }

    /// Um ponto criado na curva de um trecho fica EM CIMA da curva que toca (a forma não muda —
    /// de Casteljau), para Straight, Curve e a curva de tangentes absolutas; o oráculo é o
    /// amostrador da animação (`sample_keys`), não estas contas.
    #[test]
    fn o_ponto_criado_fica_em_cima_da_curva_que_toca() {
        for interp in [
            Interp::Linear,
            Interp::bezier(0.2, 0.0, 0.3, 1.0),
            Interp::bezier_w(0.4, 3.0, 0.7, -1.0),
        ] {
            let (k0, k1) = (key(1.0, 2.0, interp), key(3.0, 12.0, Interp::Linear));
            let mut c = segment_curve(&k0, &k1).expect("editável");
            let i = c.insert(0.4).expect("cabe");
            let (t, v) = unit_to_plane(&k0, &k1, c.node(i).p);
            let mut novo = k0.clone();
            novo.interp = interp_of_curve(c);
            let toca = f64::from(sample_keys(&[novo, k1.clone()], t).unwrap_or(f32::NAN));
            assert!(
                (toca - v).abs() < 1e-3,
                "{interp:?}: ponto {v} × curva {toca} em t={t}"
            );
            let antes = f64::from(sample_keys(&[k0.clone(), k1.clone()], t).unwrap_or(f32::NAN));
            assert!(
                (antes - v).abs() < 1e-3,
                "{interp:?}: a forma mudou ({antes} × {v})"
            );
        }
    }

    #[test]
    fn plano_e_quadrado_sao_inversos() {
        let (k0, k1) = (
            key(1.0, 2.0, Interp::Linear),
            key(3.0, -8.0, Interp::Linear),
        );
        let (t, v) = unit_to_plane(&k0, &k1, [0.25, 1.5]);
        assert_eq!((t, v), (1.5, -13.0));
        assert_eq!(plane_to_unit(&k0, &k1, t, v), [0.25, 1.5]);
    }

    /// Onde os pontos MENTIRIAM ou não têm escala, não há curva editável: retimado, parado, sem
    /// duração, e as famílias sem Bézier exata.
    #[test]
    fn sem_curva_editavel_onde_os_pontos_mentiriam() {
        let k1 = key(2.0, 10.0, Interp::Linear);
        let mut r = key(0.0, 0.0, Interp::Linear);
        assert!(segment_curve(&r, &k1).is_some(), "controlo");
        r.retime = Some(ph2d_anim::Retime::new(0.1, 0.6, 0.5, 0.95));
        assert!(segment_curve(&r, &k1).is_none(), "retimado");
        let parado = key(0.0, 10.0, Interp::Linear);
        assert!(segment_curve(&parado, &k1).is_none(), "parado");
        let mesmo_t = key(2.0, 0.0, Interp::Linear);
        assert!(segment_curve(&mesmo_t, &k1).is_none(), "sem duração");
        assert!(
            segment_curve(&key(0.0, 0.0, Interp::Hold), &k1).is_none(),
            "Hold"
        );
    }
}
