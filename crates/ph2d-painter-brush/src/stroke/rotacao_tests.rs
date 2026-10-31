//! L3 — a ponta que GIRA com inércia ([`BrushSpec::dab_rotation_lag`]): neutra a `0`, atrasada na curva em proporção
//! ao lag, sem salto na inversão; e a ponta chata de ângulo fixo dá larguras diferentes por direcção.

use super::*;
use crate::{DabRotation, Dynamics};

fn pt(x: f32, y: f32) -> StrokePoint {
    StrokePoint {
        pos: [x, y],
        pressure: 1.0,
    }
}

fn spec_seguir(lag: f32) -> BrushSpec {
    let mut s = BrushSpec {
        radius_px: 10.0,
        spacing: 0.25,
        stabilizer: 0.0,
        dab_rotation_lag: lag,
        ..BrushSpec::default()
    };
    s.pen_tilt.rotation = DabRotation::FollowTrajectory;
    s
}

fn tracar(spec: BrushSpec, pts: &[StrokePoint]) -> Vec<Dab> {
    let dyn_ = Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    };
    let mut s = Stroke::new(spec, dyn_, 1);
    let (mut all, mut out) = (Vec::new(), Vec::new());
    s.begin(pts[0], &mut out);
    all.extend_from_slice(&out);
    for &p in &pts[1..] {
        s.extend(p, &mut out);
        all.extend_from_slice(&out);
    }
    s.finish(&mut out);
    all.extend_from_slice(&out);
    all
}

/// Meia volta de raio `r` à volta da origem, no sentido anti-horário, em passos de 1°.
fn arco(r: f32) -> Vec<StrokePoint> {
    (0..=180)
        .map(|g| {
            let a = (g as f32).to_radians();
            pt(r * a.cos(), r * a.sin())
        })
        .collect()
}

/// O atraso médio (graus) do rumo da ponta atrás da tangente do arco, no terço do meio (regime).
fn atraso_medio(lag: f32) -> f32 {
    let dabs = tracar(spec_seguir(lag), &arco(150.0));
    let meio: Vec<_> = dabs
        .iter()
        .filter(|d| {
            let a = d.center[1].atan2(d.center[0]).to_degrees();
            (60.0..120.0).contains(&a)
        })
        .collect();
    assert!(meio.len() > 20, "o arco tem dabs no regime: {}", meio.len());
    let soma: f32 = meio
        .iter()
        .map(|d| {
            let r = (d.center[0].hypot(d.center[1])).max(1e-3);
            let t = [-d.center[1] / r, d.center[0] / r];
            // O rumo fica do lado de FORA da curva (atrasado): o produto vectorial dir × t > 0.
            (d.dir[0] * t[1] - d.dir[1] * t[0]).asin().to_degrees()
        })
        .sum();
    soma / meio.len() as f32
}

#[test]
fn rotacao_lag_zero_e_o_rumo_de_sempre_ao_bit() {
    let ida_e_volta: Vec<_> = (0..=40)
        .map(|k| pt(k as f32 * 4.0, 0.0))
        .chain((0..=40).rev().map(|k| pt(k as f32 * 4.0, 0.5)))
        .collect();
    let a = tracar(spec_seguir(0.0), &ida_e_volta);
    let b = tracar(
        BrushSpec {
            dab_rotation_lag: BrushSpec::default().dab_rotation_lag,
            ..spec_seguir(0.0)
        },
        &ida_e_volta,
    );
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
    // A lei de antes: na inversão o rumo SALTA para −x (o eixo da inércia é que não salta).
    assert!(
        a.iter().any(|d| d.dir[0] < -0.99),
        "sem lag o rumo vira para trás"
    );
}

#[test]
fn rotacao_no_arco_o_rumo_atrasa_em_proporcao_ao_lag() {
    let (a0, a1, a2) = (atraso_medio(0.0), atraso_medio(1.0), atraso_medio(2.0));
    assert!(
        a0 < a1 && a1 < a2,
        "o atraso cresce com o lag: {a0} {a1} {a2}"
    );
    // Regime da média exponencial sobre o arco: atraso ≈ L/R rad, com L = max(0,1·D, 3) + lag·D (D = 20, R = 150).
    for (lag, medido) in [(1.0_f32, a1), (2.0, a2)] {
        let esperado = ((2.0 + lag * 20.0) / 150.0_f32).to_degrees();
        assert!(
            (medido - esperado).abs() < 0.3 * esperado,
            "lag {lag}: atraso {medido}° × esperado {esperado}°"
        );
    }
}

#[test]
fn rotacao_com_lag_a_inversao_nao_salta() {
    let ida_e_volta: Vec<_> = (0..=40)
        .map(|k| pt(k as f32 * 4.0, 0.0))
        .chain((0..=40).rev().map(|k| pt(k as f32 * 4.0, 0.5)))
        .collect();
    let dabs = tracar(spec_seguir(1.0), &ida_e_volta);
    for d in dabs.iter().filter(|d| d.dir != [0.0, 0.0]) {
        assert!(
            d.dir[0] > 0.9,
            "a lâmina fica no eixo x, sem virar: {:?}",
            d.dir
        );
    }
}

#[test]
fn rotacao_ponta_chata_de_angulo_fixo_tem_largura_por_direccao() {
    let chata = BrushSpec {
        radius_px: 10.0,
        dab_quina: 1.0,
        dab_flatten: 0.8,
        dab_angle_deg: 0,
        ..BrushSpec::default()
    };
    // A largura do traço é a meia-extensão da pegada PERPENDICULAR ao rumo (y no horizontal, x no vertical).
    let largura = |spec: &BrushSpec, pts: &[StrokePoint], eixo: usize| {
        let dabs = tracar(*spec, pts);
        let d = dabs[dabs.len() / 2];
        spec.dab_footprint_of(&d).extent()[eixo] * d.radius_px
    };
    let horiz: Vec<_> = (0..=30).map(|k| pt(k as f32 * 4.0, 0.0)).collect();
    let vert: Vec<_> = (0..=30).map(|k| pt(0.0, k as f32 * 4.0)).collect();
    let (h, v) = (largura(&chata, &horiz, 1), largura(&chata, &vert, 0));
    assert!(
        v > 2.0 * h,
        "fixa: fina no horizontal, larga no vertical: {h} × {v}"
    );
    // A seguir o traço (com inércia) a lâmina dá a MESMA largura nas duas direcções.
    let mut segue = chata;
    segue.pen_tilt.rotation = DabRotation::FollowTrajectory;
    segue.dab_rotation_lag = 1.0;
    let (hs, vs) = (largura(&segue, &horiz, 1), largura(&segue, &vert, 0));
    assert!((hs - vs).abs() < 0.05 * hs.max(vs), "a seguir: {hs} × {vs}");
}
