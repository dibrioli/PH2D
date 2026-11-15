//! ⭐ **Os pontos da curva Custom no gráfico da timeline** (dono 15/11: «permita criar pontos no
//! gráfico da timeline e editar sem criar keyframes, exatamente igual é feito no easing custom»).
//!
//! A curva de um trecho mora no quadrado unitário (`EaseSpline`); o gráfico pinta-a no plano
//! `(tempo, valor)`: `t = t0 + x·Δt`, `v = v0 + y·Δv`. Estas duas contas e a pergunta «este trecho
//! tem curva editável?» são as únicas que o painel precisa — o resto (criar, mover, apagar, tipos de
//! alça) é a MESMA `EaseSpline` que o cartão da chave edita.

use ph2d_anim::{EaseSpline, Interp, Retime, SplineSide};

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

/// Puxar um braço de uma Custom com PONTOS (em qualquer vista) mexe só na alça da ponta da curva:
/// os pontos e o ritmo ficam. O Override (virar `BezierW`, tirar o ritmo) apagá-los-ia.
#[must_use]
pub fn end_handle_keeps_points(k0: &KeyView, k1: &KeyView) -> bool {
    matches!(k0.interp, Interp::Spline(_)) && segment_curve(k0, k1).is_some()
}

/// O gesto do braço na vista de VELOCIDADE (o do AE: horizontal = influência, vertical =
/// velocidade) numa Custom com pontos: a alça da ponta ganha a influência `x = u` do rato e a
/// inclinação que dá a velocidade `v` que TOCA (`E'·W'·Δv/Δt`, com o ritmo); os pontos e o ritmo
/// ficam. Sem velocidade possível (`W'` nulo ou infinito na ponta) a curva fica como está.
#[must_use]
pub fn spline_with_speed_handle(k0: &KeyView, k1: &KeyView, which: u8, t: f64, v: f64) -> Interp {
    let Interp::Spline(sp) = k0.interp else {
        return k0.interp;
    };
    let (span, dv) = (k1.t_seconds - k0.t_seconds, f64::from(k1.value - k0.value));
    let ws = ph2d_anim::retime::warp_slope(k0.retime, if which == 0 { 0.0 } else { 1.0 });
    if span <= 0.0 || dv.abs() <= f64::EPSILON || !(ws.is_finite() && ws > 0.0) {
        return k0.interp;
    }
    let m = crate::graph::MIN_INFLUENCE;
    let u = (t - k0.t_seconds) / span;
    let declive = v * span / (ws * dv); // E' na ponta
    let (out, inn) = sp.end_handles();
    let f = |x: f64, y: f64| [x as f32, y as f32];
    let nova = if which == 0 {
        let x = u.clamp(m, 1.0); // CLAMP-OK: const bounds, min<max
        sp.with_end_handles(f(x, declive * x), inn)
    } else {
        let x = u.clamp(0.0, 1.0 - m); // CLAMP-OK: const bounds, min<max
        sp.with_end_handles(out, f(x, 1.0 - declive * (1.0 - x)))
    };
    Interp::Spline(nova)
}

/// O braço de VELOCIDADE de um lado de um ponto do meio de uma Custom (dono 15/11: «os pontos
/// criados não deveriam ganhar alça de velocidade em override?»): o ponto é uma chave sem chave,
/// e cada lado dele tem a velocidade que a alça desse lado dá à curva que TOCA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointSpeedArm {
    /// O nó da `EaseSpline` (1..n−1).
    pub node: usize,
    /// `1` = chegada (alça de entrada), `2` = saída.
    pub part: u8,
    /// O instante do ponto (onde o braço nasce).
    pub t: f64,
    /// O instante da ponta do braço: a alça desse lado pelo mapa do ritmo (o comprimento é a
    /// influência, como nas chaves).
    pub tip_t: f64,
    /// A velocidade que toca desse lado (valor por segundo): `E'·W'·Δv/Δt`.
    pub speed: f64,
}

/// `W'` no instante em que o nó `p` do easing toca (`u = W⁻¹(x)`).
fn ritmo_no_no(k0: &KeyView, k1: &KeyView, p: [f32; 2]) -> f64 {
    let span = k1.t_seconds - k0.t_seconds;
    let u = (unit_to_plane(k0, k1, p).0 - k0.t_seconds) / span;
    ph2d_anim::retime::warp_slope(k0.retime, u)
}

/// Os braços de velocidade dos pontos do meio do trecho (nenhum sem pontos). Um lado com a alça
/// vertical (sem influência) ou o ritmo parado/infinito no ponto não tem braço.
#[must_use]
pub fn point_speed_arms(k0: &KeyView, k1: &KeyView) -> Vec<PointSpeedArm> {
    let Some(c) = segment_curve(k0, k1).filter(|_| matches!(k0.interp, Interp::Spline(_))) else {
        return Vec::new();
    };
    let (span, dv) = (k1.t_seconds - k0.t_seconds, f64::from(k1.value - k0.value));
    let mut out = Vec::new();
    for node in 1..c.node_count() - 1 {
        let n = c.node(node);
        let ws = ritmo_no_no(k0, k1, n.p);
        if !(ws.is_finite() && ws > 0.0) {
            continue;
        }
        let t = unit_to_plane(k0, k1, n.p).0;
        for (part, h) in [(1u8, n.h_in), (2, n.h_out)] {
            let dx = f64::from(n.p[0] - h[0]);
            if dx.abs() <= f64::EPSILON {
                continue;
            }
            let declive = f64::from(n.p[1] - h[1]) / dx;
            out.push(PointSpeedArm {
                node,
                part,
                t,
                tip_t: unit_to_plane(k0, k1, h).0,
                speed: declive * dv * ws / span,
            });
        }
    }
    out
}

/// O gesto do braço de velocidade de um ponto (o do AE: horizontal = influência, vertical =
/// velocidade): a alça desse lado vai para o `x` do rato (pelo ritmo) com a inclinação que dá a
/// velocidade `v`. O tipo do ponto manda no outro lado (Smooth: a mesma velocidade; Corner:
/// livre), como no gráfico de valor. `None` onde o ponto não tem braço.
#[must_use]
pub fn spline_with_point_speed(
    k0: &KeyView,
    k1: &KeyView,
    (node, part): (usize, u8),
    t: f64,
    v: f64,
) -> Option<EaseSpline> {
    let mut c = segment_curve(k0, k1).filter(|_| matches!(k0.interp, Interp::Spline(_)))?;
    if node == 0 || node + 1 >= c.node_count() {
        return None;
    }
    let (span, dv) = (k1.t_seconds - k0.t_seconds, f64::from(k1.value - k0.value));
    let p = c.node(node).p;
    let ws = ritmo_no_no(k0, k1, p);
    if !(ws.is_finite() && ws > 0.0) {
        return None;
    }
    let m = crate::graph::MIN_INFLUENCE as f32;
    let x = plane_to_unit(k0, k1, t, 0.0)[0];
    let (lado, x) = if part == 1 {
        let antes = c.node(node - 1).p[0];
        (SplineSide::In, x.clamp(antes, p[0] - m)) // CLAMP-OK: entre o nó anterior e este
    } else {
        let depois = c.node(node + 1).p[0];
        (SplineSide::Out, x.clamp(p[0] + m, depois)) // CLAMP-OK: entre este e o seguinte
    };
    let declive = v * span / (ws * dv);
    let y = f64::from(p[1]) + declive * f64::from(x - p[0]);
    c.move_handle(node, lado, x, y as f32);
    Some(c)
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

/// Os valores de tudo o que a vista de valor desenha de uma Custom com pontos: cada nó e as duas
/// alças dele (as das pontas são os braços das chaves). A faixa da banda tem de os conter.
pub(crate) fn custom_drawn_values(k0: &KeyView, k1: &KeyView) -> Vec<f64> {
    let Some(c) = segment_curve(k0, k1) else {
        return Vec::new();
    };
    (0..c.node_count())
        .flat_map(|i| {
            let n = c.node(i);
            [n.p, n.h_in, n.h_out]
        })
        .map(|p| unit_to_plane(k0, k1, p).1)
        .collect()
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

    /// Smoke do dono 15/11: a ponta do braço de uma Custom com pontos acima da curva (`y = 1,3`)
    /// ficava FORA da banda — nem se via nem se agarrava. A faixa que o gráfico ajusta contém
    /// cada nó e cada alça, com ou sem ritmo.
    #[test]
    fn a_faixa_contem_cada_alca_da_custom_com_pontos() {
        let mut sp = EaseSpline::from_bezier(0.3, 0.0, 0.7, 1.0);
        let i = sp.insert(0.5).expect("cabe");
        sp.move_handle(i, ph2d_anim::SplineSide::Out, 0.7, -0.4);
        let sp = sp.with_end_handles([0.06, 1.3], sp.end_handles().1);
        for retime in [None, Some(RITMO)] {
            let (mut k0, k1) = (
                key(0.0, 0.0, Interp::Spline(sp)),
                key(2.0, 10.0, Interp::Linear),
            );
            k0.selected = true;
            k0.retime = retime;
            let (lo, hi) = crate::drawn_extent(&[k0.clone(), k1.clone()], 0.0, 2.0, 64).unwrap();
            for v in custom_drawn_values(&k0, &k1) {
                assert!(
                    v as f32 >= lo - 1e-4 && v as f32 <= hi + 1e-4,
                    "{retime:?}: {v} fora de {lo}..{hi}"
                );
            }
            assert!(hi >= 12.9, "{retime:?}: a ponta do braço (13) fora: {hi}");
            assert!(lo <= -3.9, "{retime:?}: a alça do ponto (−4) fora: {lo}");
        }
    }

    /// Na vista de VELOCIDADE, puxar o braço de uma Custom com pontos dá à ponta a velocidade do
    /// rato (o oráculo é a velocidade que TOCA, `segment_endpoint_speed`, com o ritmo) e a
    /// influência do rato — sem apagar os pontos nem o ritmo.
    #[test]
    fn na_vista_de_velocidade_o_braco_da_custom_com_pontos_mantem_pontos_e_ritmo() {
        let mut sp = EaseSpline::from_bezier(0.3, 0.0, 0.7, 1.0);
        sp.insert(0.5);
        for retime in [None, Some(RITMO)] {
            for (which, t, v) in [(0u8, 0.4, 7.0), (1, 1.5, 3.0)] {
                let (mut k0, k1) = (
                    key(0.0, 0.0, Interp::Spline(sp)),
                    key(2.0, 10.0, Interp::Linear),
                );
                k0.retime = retime;
                assert!(end_handle_keeps_points(&k0, &k1));
                let mut novo = k0.clone();
                novo.interp = spline_with_speed_handle(&k0, &k1, which, t, v);
                let Interp::Spline(c) = novo.interp else {
                    panic!("{retime:?}: virou {:?}", novo.interp);
                };
                assert_eq!(c.points().len(), 1, "{retime:?}: os pontos sumiram");
                let toca = crate::speed::segment_endpoint_speed(&novo, &k1, which);
                assert!(
                    (toca - v).abs() < 1e-3,
                    "{retime:?} {which}: toca {toca} × rato {v}"
                );
                if retime.is_none() {
                    let (tt, _) = crate::speed_handle_tip(&novo, &k1, which).unwrap();
                    assert!((tt - t).abs() < 1e-4, "a influência não foi ao rato: {tt}");
                }
            }
        }
        let sem = key(0.0, 0.0, Interp::bezier(0.3, 0.0, 0.7, 1.0));
        assert!(!end_handle_keeps_points(
            &sem,
            &key(2.0, 10.0, Interp::Linear)
        ));
    }

    /// Os pontos da Custom ganham braços de VELOCIDADE (dono 15/11): cada lado diz a velocidade que
    /// TOCA ali (oráculo: diferença finita de um lado sobre `sample_keys`, com e sem ritmo); puxar o
    /// braço leva a ponta ao rato (influência e velocidade); Smooth arrasta a outra velocidade
    /// junto, Corner deixa-a onde estava.
    #[test]
    fn os_pontos_tem_bracos_de_velocidade_que_tocam_e_se_puxam() {
        let mut sp = EaseSpline::from_bezier(0.3, 0.0, 0.7, 1.0);
        let i = sp.insert(0.5).expect("cabe");
        for retime in [None, Some(RITMO)] {
            let (mut k0, k1) = (
                key(0.0, 0.0, Interp::Spline(sp)),
                key(2.0, 10.0, Interp::Linear),
            );
            k0.retime = retime;
            let arms = point_speed_arms(&k0, &k1);
            assert_eq!(arms.len(), 2, "{retime:?}: chegada e saída");
            let keys = [k0.clone(), k1.clone()];
            let s = |t: f64| f64::from(sample_keys(&keys, t).unwrap());
            let h = 1e-4;
            for a in &arms {
                let fd = if a.part == 1 {
                    (s(a.t) - s(a.t - h)) / h
                } else {
                    (s(a.t + h) - s(a.t)) / h
                };
                assert!(
                    (fd - a.speed).abs() < 0.02 * a.speed.abs().max(1.0),
                    "{retime:?} {a:?}: toca {fd}"
                );
            }
            let saida = arms.iter().find(|a| a.part == 2).unwrap();
            let alvo_t = saida.t + 0.6 * (saida.tip_t - saida.t) + 0.05;
            let c = spline_with_point_speed(&k0, &k1, (i, 2), alvo_t, 3.0).expect("tem braço");
            assert_eq!(c.points().len(), 1);
            let mut novo = k0.clone();
            novo.interp = Interp::Spline(c);
            let depois = point_speed_arms(&novo, &k1);
            let (ent, sai) = (
                depois.iter().find(|a| a.part == 1).unwrap(),
                depois.iter().find(|a| a.part == 2).unwrap(),
            );
            assert!(
                (sai.speed - 3.0).abs() < 1e-3,
                "{retime:?}: saída {}",
                sai.speed
            );
            assert!(
                (sai.tip_t - alvo_t).abs() < 1e-3,
                "{retime:?}: ponta {}",
                sai.tip_t
            );
            assert!(
                (ent.speed - 3.0).abs() < 1e-3,
                "{retime:?}: Smooth — a chegada acompanha"
            );
            // Corner: a outra velocidade fica.
            let mut cc = c;
            assert!(cc.set_kind(i, ph2d_anim::HandleKind::Corner));
            novo.interp = Interp::Spline(cc);
            let antes = point_speed_arms(&novo, &k1)[0].speed;
            let c2 = spline_with_point_speed(&novo, &k1, (i, 2), alvo_t, 6.0).unwrap();
            novo.interp = Interp::Spline(c2);
            let d2 = point_speed_arms(&novo, &k1);
            assert!(
                (d2[0].speed - antes).abs() < 1e-3,
                "{retime:?}: Corner mexeu na chegada"
            );
            assert!((d2[1].speed - 6.0).abs() < 1e-3);
        }
        let sem = key(0.0, 0.0, Interp::Linear);
        assert!(point_speed_arms(&sem, &key(2.0, 10.0, Interp::Linear)).is_empty());
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
