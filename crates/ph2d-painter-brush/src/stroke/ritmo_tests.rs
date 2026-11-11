//! B22.2 (decisão 8) — o relógio do traço parado não segue o monitor: o spray parado e o aerógrafo pousam a mesma
//! densidade a 30, 60 e 144 q/s, e um quadro preso não despeja o atraso.

use super::*;
use crate::falloff::Falloff;

fn spec(parado_s: f32, method: StrokeMethod, airbrush_rate_s: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 10.0,
        spacing: 0.5,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        stroke_method: method,
        parado_s,
        airbrush_rate_s,
        ..Default::default()
    }
}

fn sem_dinamica() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// Os dabs que o tique pousa com a caneta parada, `dts` a `dts`.
fn parado(spec: BrushSpec, dts: &[f32]) -> Vec<usize> {
    let mut s = Stroke::new(spec, sem_dinamica(), 1);
    let mut out = Vec::new();
    s.begin(
        StrokePoint {
            pos: [50.0, 50.0],
            pressure: 1.0,
        },
        &mut out,
    );
    dts.iter()
        .map(|&dt| {
            s.tick(dt, &mut out);
            out.len()
        })
        .collect()
}

fn um_segundo(spec: BrushSpec, fps: usize) -> usize {
    parado(spec, &vec![1.0 / fps as f32; fps]).iter().sum()
}

/// O Spray Can (`parado_s = 0,0035`): ~286 nuvens por segundo parado a qualquer ritmo de quadro. Antes: 240 a 30 q/s
/// (o batente de 8 por tique), 286 a 60 e a 144.
#[test]
fn spray_parado_a_30_e_144_fps_da_a_mesma_densidade() {
    let sp = spec(0.0035, StrokeMethod::Space, 0.1);
    let alvo = (1.0 / 0.0035_f32).floor() as usize;
    for fps in [30, 60, 144] {
        let n = um_segundo(sp.clone(), fps);
        assert!(
            n.abs_diff(alvo) <= 1,
            "{fps} q/s: {n} nuvens num segundo parado, o alvo é {alvo}"
        );
    }
}

/// O aerógrafo a 1 ms: 1000 dabs por segundo a 30 e a 144 q/s. Antes: 240 a 30 q/s.
#[test]
fn aerografo_a_30_e_144_fps_da_a_mesma_densidade() {
    let sp = spec(0.0, StrokeMethod::Airbrush, 0.001);
    for fps in [30, 60, 144] {
        let n = um_segundo(sp.clone(), fps);
        assert!(
            n.abs_diff(1000) <= 2,
            "{fps} q/s: {n} dabs num segundo, o alvo é 1000"
        );
    }
}

/// Um quadro preso (5 s) conta só até [`DT_TIQUE_MAX_S`]: o atraso cai, não é despejado depois.
#[test]
fn spray_parado_nao_acumula() {
    let p = 0.0035;
    let teto = (DT_TIQUE_MAX_S / p).ceil() as usize + 1;
    for sp in [
        spec(p, StrokeMethod::Space, 0.1),
        spec(0.0, StrokeMethod::Airbrush, p),
    ] {
        let n = parado(sp, &[5.0, 1.0 / 60.0, 1.0 / 60.0]);
        assert!(n[0] <= teto, "o quadro preso pousou {} (teto {teto})", n[0]);
        assert!(
            n[1] <= 6 && n[2] <= 6,
            "o atraso foi despejado depois: {n:?}"
        );
    }
}
