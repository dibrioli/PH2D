//! ⭐ **Os pontos da curva Custom no gráfico da timeline** (dono 15/11: «permita criar pontos no
//! gráfico da timeline e editar sem criar keyframes, exatamente igual é feito no easing custom»).
//!
//! A curva de um trecho mora no quadrado unitário (`EaseSpline`); o gráfico pinta-a no plano
//! `(tempo, valor)`: `t = t0 + x·Δt`, `v = v0 + y·Δv`. Estas duas contas e a pergunta «este trecho
//! tem curva editável?» são as únicas que o painel precisa — o resto (criar, mover, apagar, tipos de
//! alça) é a MESMA `EaseSpline` que o cartão da chave edita.

use ph2d_anim::{EaseSpline, Interp, Retime};

use crate::snapshot::KeyView;

/// A curva editável do trecho `k0 → k1` no quadrado unitário, ou `None` quando o gráfico não a
/// mostra: sem duração, PARADO (`Δv = 0`: o eixo do valor não tem escala) ou numa família sem
/// forma exata de Bézier (Hold, Nearest, os presets Ease, Bounce…). A `Linear` é a Bézier
/// `(⅓,⅓)(⅔,⅔)` exata, e entra: é onde o primeiro ponto mais se cria. Um trecho RETIMADO entra:
/// [`unit_to_plane`]/[`plane_to_unit`] levam o ritmo, e os pontos caem em cima de `E(W(u))`.
#[must_use]
pub fn segment_curve(k0: &KeyView, k1: &KeyView) -> Option<EaseSpline> {
    let dv = f64::from(k1.value - k0.value);
    if k1.t_seconds <= k0.t_seconds || dv.abs() <= f64::EPSILON {
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

/// O ponto `[x, y]` do quadrado unitário no plano `(tempo, valor)` do trecho. Num trecho
/// RETIMADO o `x` é o tempo do EASING: toca no instante `u = W⁻¹(x)` (o inverso da Bézier do ritmo
/// é a mesma com as coordenadas trocadas), por isso o ponto fica em cima da curva que toca.
#[must_use]
pub fn unit_to_plane(k0: &KeyView, k1: &KeyView, [x, y]: [f32; 2]) -> (f64, f64) {
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let (span, dv) = (k1.t_seconds - t0, f64::from(k1.value) - v0);
    let u = match k0.retime {
        Some(w) => Retime::new(w.y1, w.x1, w.y2, w.x2).eval(f64::from(x)),
        None => f64::from(x),
    };
    (t0 + u * span, v0 + f64::from(y) * dv)
}

/// O inverso de [`unit_to_plane`] — sem prender o valor: quem guarda os invariantes é a
/// `EaseSpline` (`move_point`/`move_handle`); num trecho retimado o tempo passa pelo ritmo
/// (`x = W(u)`, preso ao trecho). Só faz sentido onde [`segment_curve`] é `Some`.
#[must_use]
pub fn plane_to_unit(k0: &KeyView, k1: &KeyView, t: f64, v: f64) -> [f32; 2] {
    let (t0, v0) = (k0.t_seconds, f64::from(k0.value));
    let (span, dv) = (k1.t_seconds - t0, f64::from(k1.value) - v0);
    let u = (t - t0) / span;
    let x = match k0.retime {
        Some(w) => w.eval(u),
        None => u,
    };
    [x as f32, ((v - v0) / dv) as f32]
}

/// Puxar o braço da PONTA na vista de valor mantém o ritmo? Só numa Custom com pontos retimada:
/// ali o braço é a alça da ponta da `EaseSpline` pelo mesmo mapa dos pontos do meio, e o Override
/// (tirar o ritmo, virar `BezierW`) apagaria os pontos. Sem pontos o braço continua o Override.
#[must_use]
pub fn end_handle_keeps_rhythm(k0: &KeyView, k1: &KeyView) -> bool {
    k0.retime.is_some() && matches!(k0.interp, Interp::Spline(_)) && segment_curve(k0, k1).is_some()
}

/// Onde a vista de VALOR desenha (e agarra) os dois braços do trecho: os de
/// [`crate::segment_handle_points`], salvo numa Custom com pontos retimada, cujas pontas seguem o
/// mapa dos pontos do meio ([`end_handle_keeps_rhythm`]).
#[must_use]
pub fn value_handle_points(k0: &KeyView, k1: &KeyView) -> [(f64, f64); 2] {
    if let (true, Interp::Spline(sp)) = (end_handle_keeps_rhythm(k0, k1), k0.interp) {
        let (out, inn) = sp.end_handles();
        return [unit_to_plane(k0, k1, out), unit_to_plane(k0, k1, inn)];
    }
    crate::graph::segment_handle_points(k0, k1)
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

    const RITMO: Retime = Retime {
        x1: 0.1,
        y1: 0.6,
        x2: 0.5,
        y2: 0.95,
    };

    /// Num trecho RETIMADO o ponto criado e cada ponto movido ficam EM CIMA do que toca
    /// (`E(W(u))`): o oráculo é `sample_keys` no tempo desenhado, com o ritmo ainda lá.
    #[test]
    fn num_trecho_retimado_o_ponto_fica_em_cima_da_curva_que_toca() {
        for interp in [Interp::Linear, Interp::bezier(0.2, 0.0, 0.3, 1.0)] {
            let (mut k0, k1) = (key(1.0, 2.0, interp), key(3.0, 12.0, Interp::Linear));
            k0.retime = Some(RITMO);
            let mut c = segment_curve(&k0, &k1).expect("retimado também é editável");
            let i = c.insert(0.4).expect("cabe");
            let mut novo = k0.clone();
            novo.interp = interp_of_curve(c.clone());
            let keys = [novo.clone(), k1.clone()];
            let toca = |p| {
                let (t, v) = unit_to_plane(&novo, &k1, p);
                (f64::from(sample_keys(&keys, t).unwrap_or(f32::NAN)), v, t)
            };
            let (a, v, t) = toca(c.node(i).p);
            assert!(
                (a - v).abs() < 1e-3,
                "{interp:?}: ponto {v} × toca {a} em {t}"
            );
            let antes = f64::from(sample_keys(&[k0.clone(), k1.clone()], t).unwrap_or(f32::NAN));
            assert!((antes - v).abs() < 1e-3, "{interp:?}: a forma mudou");
            // Arrastado para outro sítio da tela, continua em cima.
            let [x, y] = plane_to_unit(&k0, &k1, 2.3, 9.0);
            c.move_point(i, x, y);
            novo.interp = interp_of_curve(c.clone());
            let keys = [novo.clone(), k1.clone()];
            let (t, v) = unit_to_plane(&novo, &k1, c.node(i).p);
            let a = f64::from(sample_keys(&keys, t).unwrap_or(f32::NAN));
            assert!(
                (t - 2.3).abs() < 1e-4 && (v - 9.0).abs() < 1e-4,
                "não foi ao rato: {t} {v}"
            );
            assert!((a - v).abs() < 1e-3, "{interp:?}: arrastado {v} × toca {a}");
        }
    }

    /// O mapa do ritmo vai e volta: o que se desenha é o que se agarra.
    #[test]
    fn num_trecho_retimado_plano_e_quadrado_sao_inversos() {
        let (mut k0, k1) = (
            key(1.0, 2.0, Interp::Linear),
            key(3.0, -8.0, Interp::Linear),
        );
        k0.retime = Some(RITMO);
        for p in [[0.1, 0.2], [0.5, -0.3], [0.9, 1.4]] {
            let (t, v) = unit_to_plane(&k0, &k1, p);
            let q = plane_to_unit(&k0, &k1, t, v);
            assert!(
                (q[0] - p[0]).abs() < 1e-5 && (q[1] - p[1]).abs() < 1e-5,
                "{p:?} → {q:?}"
            );
        }
        let (t, _) = unit_to_plane(&k0, &k1, [0.6, 0.0]);
        assert!(
            (t - 1.0 - 0.6 * 2.0).abs() > 0.05,
            "o ritmo não entrou no mapa"
        );
    }

    /// O braço da PONTA de uma Custom com pontos retimada segue o mesmo mapa e, puxado, fica sob o
    /// rato sem apagar os pontos nem o ritmo; o braço de uma retimada SEM pontos continua Override.
    #[test]
    fn num_trecho_retimado_o_braco_da_ponta_da_custom_segue_o_mapa_dos_pontos() {
        let mut sp = EaseSpline::from_bezier(0.3, 0.0, 0.7, 1.0);
        sp.insert(0.5);
        let (mut k0, k1) = (
            key(0.0, 0.0, Interp::Spline(sp)),
            key(2.0, 10.0, Interp::Linear),
        );
        k0.retime = Some(Retime::new(0.3, 0.3, 1.0, 0.0)); // chegada vertical no canto
        assert!(end_handle_keeps_rhythm(&k0, &k1));
        assert!(
            crate::segment_handle_drawn(&k0, &k1, 1),
            "com pontos o braço fica"
        );
        let [out, _] = value_handle_points(&k0, &k1);
        assert_eq!(out, unit_to_plane(&k0, &k1, sp.end_handles().0));
        let mut novo = k0.clone();
        novo.interp = crate::weighted_with_handle(&k0, &k1, 0, 0.5, 4.0);
        let Interp::Spline(c) = novo.interp else {
            panic!("virou {:?}", novo.interp);
        };
        assert_eq!(c.points().len(), 1);
        let [out, inn] = value_handle_points(&novo, &k1);
        assert!(
            (out.0 - 0.5).abs() < 1e-4 && (out.1 - 4.0).abs() < 1e-4,
            "{out:?}"
        );
        assert_eq!(inn, value_handle_points(&k0, &k1)[1], "a outra ponta andou");
        let mut sem = k0.clone();
        sem.interp = Interp::bezier(0.3, 0.1, 0.6, 0.9);
        assert!(!end_handle_keeps_rhythm(&sem, &k1), "sem pontos = Override");
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
        assert!(
            segment_curve(&key(0.0, 0.0, Interp::Linear), &k1).is_some(),
            "controlo"
        );
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
