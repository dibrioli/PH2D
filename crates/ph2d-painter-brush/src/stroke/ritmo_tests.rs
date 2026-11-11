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

thread_local! {
    /// O oráculo da prancha B22.2: `true` faz o [`Stroke::tick_parado`] correr o relógio VELHO (verbatim, abaixo).
    pub(super) static RELOGIO_ANTIGO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// O relógio velho do spray parado, verbatim (antes do c7a8208d0c): o batente de 8 por tique sem descartar o atraso.
pub(super) fn pulsos_antigos(acum: &mut f32, dt: f32, periodo: f32) -> u32 {
    *acum += dt.max(0.0);
    let mut n = 0;
    while *acum >= periodo && n < 8 {
        *acum -= periodo;
        n += 1;
    }
    n
}

/// Os dabs, tique a tique, de um segundo parado a `dt`, pelo relógio velho ou pelo novo.
fn dabs_parado(antigo: bool, dt: f32, ticks: usize) -> Vec<Vec<Dab>> {
    RELOGIO_ANTIGO.set(antigo);
    let mut sp = spec(0.0035, StrokeMethod::Space, 0.1);
    sp.spray_count = 16;
    sp.jitter = 0.7;
    let mut s = Stroke::new(sp, sem_dinamica(), 7);
    let mut out = Vec::new();
    s.begin(
        StrokePoint {
            pos: [50.0, 50.0],
            pressure: 1.0,
        },
        &mut out,
    );
    let v = (0..ticks)
        .map(|_| {
            s.tick(dt, &mut out);
            out.clone()
        })
        .collect();
    RELOGIO_ANTIGO.set(false);
    v
}

/// A prancha B22.2 pinta o «antes» a 30 q/s pelo motor NOVO com `dt = 8 × período + ε` (o tool não vê o oráculo
/// `cfg(test)` desta crate). A prova: o relógio velho a 1/30 s e o novo a esse `dt` pousam os MESMOS dabs, tique a
/// tique, ao bit.
#[test]
fn o_antes_a_30_qs_e_o_motor_novo_a_8_periodos_por_tique() {
    let velho = dabs_parado(true, 1.0 / 30.0, 30);
    let novo = dabs_parado(false, DT_ANTES_30_QS, 30);
    assert!(velho.iter().all(|t| !t.is_empty()));
    assert_eq!(velho, novo);
    // E o controlo: o novo a 1/30 s NÃO é o velho (a cura muda o desenho a 30 q/s).
    assert_ne!(dabs_parado(false, 1.0 / 30.0, 30), velho);
}

/// O `dt` com que o motor novo repete o velho a 30 q/s: 8 períodos + um ε que em 30 tiques não chega a um período.
const DT_ANTES_30_QS: f32 = 8.0 * 0.0035 + 1e-5;
