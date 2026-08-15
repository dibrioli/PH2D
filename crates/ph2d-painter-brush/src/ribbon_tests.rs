//! Os gates da **FITA** — o `LineKind::Ribbon` do plano 38 W6.
//!
//! O que a feature promete: *o traço PESA*. Os gates perguntam pelo PESO — o atraso que cresce com a
//! velocidade, o chicote que ultrapassa, o pendurar sob a gravidade e o pen-up que NÃO acrescenta
//! nada (a cauda foi inibida por ordem do Enio em 2026-08-15) —, nunca
//! pela fórmula.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::{Dab, Stroke, StrokePoint};

const DT: f32 = 1.0 / 60.0;

fn spec(kind: LineKind, weight: f32, friction: f32, gravity: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: kind,
        ribbon_weight: weight,
        ribbon_friction: friction,
        ribbon_gravity: gravity,
        ..Default::default()
    }
}

fn plain() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// Um traço RETO a `speed` px/s por `frames` quadros, com o tique do produto entre os eventos.
///
/// ⚠️ **Ela ACUMULA, e tem de acumular** — o `extend`/`tick` começam por `out.clear()`, então ler o
/// buffer no fim devolve só o último evento. É a mesma armadilha de fixture que a wave do Spray
/// pagou, e a razão de este helper existir.
///
/// ⚠️ **E ela TICA**, que é o que separa esta fixture das vinte da W3/W4: a fita é integrada no
/// relógio, então sem tique ela nunca se move e todo gate ficaria verde por vácuo.
fn straight(sp: BrushSpec, speed: f32, frames: usize) -> (Vec<Dab>, [f32; 2]) {
    let mut s = Stroke::new(sp, plain(), 7);
    let mut out = Vec::new();
    let mut all: Vec<Dab> = Vec::new();
    let start = [100.0f32, 300.0];
    s.begin(
        StrokePoint {
            pos: start,
            pressure: 1.0,
        },
        &mut out,
    );
    all.extend(out.iter().copied());
    let mut x = start[0];
    for _ in 0..frames {
        x += speed * DT;
        s.extend(
            StrokePoint {
                pos: [x, start[1]],
                pressure: 1.0,
            },
            &mut out,
        );
        all.extend(out.iter().copied());
        s.tick(DT, &mut out);
        all.extend(out.iter().copied());
    }
    (all, [x, start[1]])
}

/// O quanto a tinta ficou atrás do dedo no fim de um traço reto.
fn lag_of(sp: BrushSpec, speed: f32) -> f32 {
    let (dabs, tip) = straight(sp, speed, 120);
    let ink = dabs.last().map_or(tip[0], |d| d.center[0]);
    tip[0] - ink
}

/// **O TRAÇO PESA — e NENHUMA marca fica adiante do atraso**, não só a última.
///
/// ⚠️ **O oráculo é a marca MAIS ADIANTADA, e a primeira versão lia a ÚLTIMA.** A mutação que a
/// derrubou é *o `extend` volta a percorrer*: com as duas portas a percorrer, o caminho passa a
/// ziguezaguear entre o dedo e a ponta da fita — visualmente catastrófico — e a ÚLTIMA marca de cada
/// quadro continua a ser a do tique, então o gate ficava **verde sobre um traço destruído**. Perguntar
/// pela marca mais adiantada afirma a propriedade inteira: *a tinta desta fita vive atrás do dedo*.
#[test]
fn the_ribbon_leaves_the_ink_behind_the_finger() {
    fn farthest_lag(sp: BrushSpec) -> f32 {
        let (dabs, tip) = straight(sp, 2400.0, 120);
        let far = dabs.iter().fold(f32::MIN, |a, d| a.max(d.center[0]));
        tip[0] - far
    }
    let plain_lag = farthest_lag(spec(LineKind::None, 0.45, 0.30, 0.0));
    let ribbon_lag = farthest_lag(spec(LineKind::Ribbon, 0.45, 0.30, 0.0));
    assert!(
        plain_lag.abs() < 5.0,
        "controle: sem fita a tinta segue o dedo, mediu {plain_lag:.1} px"
    );
    assert!(
        ribbon_lag > 100.0,
        "a marca mais adiantada da fita deveria viver atrás do dedo: {ribbon_lag:.1} px"
    );
}

/// **A FITA ULTRAPASSA; O ESTABILIZADOR NUNCA** — o chicote, e é ele que separa os dois.
///
/// ⚠️ **Este gate substitui um que eu escrevi ERRADO, e o erro fica registado porque a próxima
/// pessoa vai ter a mesma ideia.** A primeira versão afirmava *"o atraso da fita cresce com a
/// velocidade e o do estabilizador não"* — medido, **o do estabilizador cresce igual** (50,4 →
/// 386,4 px para 8× a velocidade). Claro que cresce: um lag de primeira ordem em regime também vale
/// `v · τ`. *A distinção não estava onde eu a tinha posto.*
///
/// **Onde ela está de facto:** o estabilizador é uma média corrida e **converge por baixo** — ele
/// nunca passa do alvo, por construção. A fita tem MASSA: com `ζ < 1` ela ultrapassa e volta, e é
/// esse trecho que o artista lê como *chicote*. Um filtro de primeira ordem não sabe fazer isto com
/// nenhum ajuste de intensidade.
#[test]
fn the_ribbon_overshoots_the_stop_and_the_stabilizer_never_does() {
    /// Corre para a direita e VIRA para baixo em `x = 1000`, sem nunca parar. Devolve o quanto a
    /// tinta passou da quina no eixo `x` — positivo = ultrapassou.
    ///
    /// ⚠️ **A fixture parada MORREU e a propriedade não** — desde *sem gesto, sem tempo* uma mão
    /// parada não entrega tempo à física, então medir o chicote *depois* da mão parar mede zero em
    /// qualquer ajuste. O chicote nunca foi sobre a mão parar: é a MASSA a levar a ponta para fora
    /// da curva, e a quina é onde ele se vê. *A distinção mudou de sítio, não de existência.*
    fn overshoot(sp: BrushSpec) -> f32 {
        let mut s = Stroke::new(sp, plain(), 7);
        let mut out = Vec::new();
        let start = [100.0f32, 300.0];
        s.begin(
            StrokePoint {
                pos: start,
                pressure: 1.0,
            },
            &mut out,
        );
        let quina = 1000.0f32;
        let mut x = start[0];
        while x < quina {
            x = (x + 3600.0 * DT).min(quina);
            s.extend(
                StrokePoint {
                    pos: [x, start[1]],
                    pressure: 1.0,
                },
                &mut out,
            );
            s.tick(DT, &mut out);
        }
        // A mão VIRA e continua a andar — nunca pára.
        let mut y = start[1];
        let mut far = start[0];
        for _ in 0..90 {
            y += 3600.0 * DT;
            s.extend(
                StrokePoint {
                    pos: [quina, y],
                    pressure: 1.0,
                },
                &mut out,
            );
            s.tick(DT, &mut out);
            for d in &out {
                far = far.max(d.center[0]);
            }
        }
        far - quina
    }
    // A fita no canto whippy (atrito no PISO) tem de PASSAR do ponto onde a mão parou.
    let whip = overshoot(spec(LineKind::Ribbon, 0.45, 0.0, 0.0));
    assert!(
        whip > 5.0,
        "a fita não chicoteou: passou {whip:.1} px do ponto de parada"
    );
    // CONTROLE 1: o estabilizador, na MESMA fixture, converge por baixo e nunca passa.
    let mut stab = spec(LineKind::None, 0.0, 0.0, 0.0);
    stab.stabilizer = 0.9;
    let s_over = overshoot(stab);
    assert!(
        s_over <= 0.5,
        "controle: o estabilizador não pode ultrapassar ({s_over:.1} px)"
    );
    // CONTROLE 2: a fita SUPER-amortecida também não — o chicote é do `ζ`, não do tipo.
    let heavy = overshoot(spec(LineKind::Ribbon, 0.45, 1.0, 0.0));
    assert!(
        heavy <= 0.5,
        "controle: `Friction` no topo não pode chicotear ({heavy:.1} px)"
    );
}

/// **O PESO É UM TEMPO** — dobrar o `Weight` dobra o atraso, na mesma velocidade.
#[test]
fn the_weight_is_the_lag_time() {
    let half = lag_of(spec(LineKind::Ribbon, 0.25, 0.30, 0.0), 2400.0);
    let full = lag_of(spec(LineKind::Ribbon, 0.50, 0.30, 0.0), 2400.0);
    let ratio = full / half;
    assert!(
        (1.6..2.4).contains(&ratio),
        "o dobro do peso deveria dar o dobro do atraso: {half:.1} → {full:.1} ({ratio:.2}×)"
    );
}

/// **A GRAVIDADE FAZ A FITA PENDER, e a queda é `g·τ²`** — a tinta desce ABAIXO do caminho.
///
/// ⚠️ A fórmula é afirmada como NÚMERO, não como forma: um pendurar que só *acontece* passaria com
/// qualquer constante, e é o `g·τ²` que torna os dois knobs previsíveis um contra o outro.
///
/// ⚠️ **A FIXTURE MUDOU e o NÚMERO não** — e a distinção é o que impede este gate de ser silenciado.
/// Ele media a mão PARADA (180 tiques sobre o mesmo ponto), e desde *sem gesto, sem tempo*
/// ([`Stroke::tick_ribbon`]) uma mão parada não entrega tempo à física: aquele fixture passou a
/// medir zero. **Não é o pendurar que morreu — é o pingar com a mão parada**, que era a espícula
/// vertical do report. Em regime, sob arrasto horizontal, o equilíbrio da mola sob gravidade
/// constante é `k·Δy = g` ⇒ `Δy = g/ω² = g·τ²`: **o mesmo número**, medido onde a feature se usa.
#[test]
fn the_gravity_makes_the_ribbon_hang_by_g_tau_squared() {
    let sp = spec(LineKind::Ribbon, 1.0, 0.50, 1.0);
    let predicted = sp.ribbon_gravity_px_s2() * sp.ribbon_lag_s() * sp.ribbon_lag_s();
    let mut s = Stroke::new(sp, plain(), 7);
    let mut out = Vec::new();
    let at = [200.0f32, 300.0];
    s.begin(
        StrokePoint {
            pos: at,
            pressure: 1.0,
        },
        &mut out,
    );
    let mut last = at;
    for i in 1..=600 {
        // Arrasto HORIZONTAL lento e longo, até o transiente vertical assentar no equilíbrio.
        #[allow(clippy::cast_precision_loss)]
        let x = at[0] + (i as f32) * 4.0;
        s.extend(
            StrokePoint {
                pos: [x, at[1]],
                pressure: 1.0,
            },
            &mut out,
        );
        s.tick(DT, &mut out);
        if let Some(d) = out.last() {
            last = d.center;
        }
    }
    let drop = last[1] - at[1];
    assert!(
        (drop - predicted).abs() < 2.5,
        "a fita deveria pender `g·τ²` = {predicted:.1} px, mediu {drop:.1}"
    );
    // CONTROLE: sem gravidade ela chega ao dedo e fica.
    let mut nog = sp;
    nog.ribbon_gravity = 0.0;
    let mut s2 = Stroke::new(nog, plain(), 7);
    s2.begin(
        StrokePoint {
            pos: at,
            pressure: 1.0,
        },
        &mut out,
    );
    let mut last2 = at;
    for _ in 0..180 {
        s2.extend(
            StrokePoint {
                pos: at,
                pressure: 1.0,
            },
            &mut out,
        );
        s2.tick(DT, &mut out);
        if let Some(d) = out.last() {
            last2 = d.center;
        }
    }
    assert!(
        (last2[1] - at[1]).abs() < 1.0,
        "controle: sem gravidade a fita não pende ({:.1} px)",
        last2[1] - at[1]
    );
}

/// **O PEN-UP NÃO ACRESCENTA NADA** — a fita acaba exactamente onde o último tique a deixou.
///
/// ⚠️ **Este gate SUBSTITUI o `the_tail_does_not_run_to_the_finger`, e a substituição é a segunda
/// desta linhagem.** O primeiro (`the_tail_arrives_when_the_pen_lifts`) afirmava o DEFEITO como lei
/// — exigia que a tinta pousasse a 40 px do dedo, que era a espícula. O segundo curou o mecanismo
/// (a coleira cortada no pen-up) e passou a exigir que a cauda **andasse** ao menos 100 px. Agora o
/// Enio inibiu o resíduo inteiro (2026-08-15: *"no mouse up o fim do traço cresce um segmento
/// indesejado"*), e a metade *"ela anda"* daquele gate passou a afirmar o que o produto não faz
/// mais.
///
/// **As duas metades que sobrevivem, e são as que importam:** a tinta acaba ATRÁS do dedo (a fita
/// nunca salta para o cursor, ao contrário do estabilizador) **e** o pen-up não move um pixel. A
/// segunda é a lei nova; a primeira é a que sempre separou uma fita de um gancho.
///
/// ⚠️ **A PREMISSA é conferida**, senão o gate mede o vazio: a fita tem de estar de facto atrasada
/// quando a caneta levanta.
#[test]
fn the_pen_up_adds_nothing_because_the_ribbon_ends_where_it_is() {
    let dt = 1.0 / 60.0;
    let mut s = Stroke::new(spec(LineKind::Ribbon, 0.45, 0.30, 0.0), plain(), 7);
    let mut out = Vec::new();
    s.begin(
        StrokePoint {
            pos: [100.0, 300.0],
            pressure: 1.0,
        },
        &mut out,
    );
    let mut x = 100.0f32;
    let mut antes = [100.0f32, 300.0];
    for _ in 0..30 {
        out.clear();
        x += 40.0;
        s.extend(
            StrokePoint {
                pos: [x, 300.0],
                pressure: 1.0,
            },
            &mut out,
        );
        s.tick(dt, &mut out);
        if let Some(d) = out.last() {
            antes = d.center;
        }
    }
    // PREMISSA: a fita tem de estar de facto atrasada, senão o gate não contém o fenômeno.
    let atraso = x - antes[0];
    assert!(
        atraso > 100.0,
        "premissa: a fita nem estava atrasada ({atraso:.1} px) -- o gate mede o vazio"
    );
    out.clear();
    s.finish(&mut out);
    assert!(
        out.is_empty(),
        "o pen-up acrescentou {} dabs: o traço cresce depois de a mão soltar",
        out.len()
    );
    // E a tinta acaba ATRÁS do dedo — a fita nunca é encerrada num salto até o cursor.
    assert!(
        antes[0] < x - 100.0,
        "a tinta acabou no dedo: soltou em {x:.1}, a fita estava em {:.1}",
        antes[0]
    );
}

/// **O NEUTRO É BYTE-IDÊNTICO** — `weight = 0` desarma a fita, e o traço é o de sempre.
#[test]
fn a_weightless_ribbon_is_the_plain_stroke_to_the_byte() {
    let (plain_dabs, _) = straight(spec(LineKind::None, 0.0, 0.0, 0.0), 2400.0, 60);
    let (zero, _) = straight(spec(LineKind::Ribbon, 0.0, 0.30, 1.0), 2400.0, 60);
    assert_eq!(
        plain_dabs, zero,
        "uma fita sem peso deveria ser o traço de sempre, ao bit"
    );
    // CONTROLE: com peso ela DIFERE — senão este gate afirmaria que a fita nunca faz nada.
    let (armed, _) = straight(spec(LineKind::Ribbon, 0.45, 0.30, 0.0), 2400.0, 60);
    assert_ne!(
        plain_dabs, armed,
        "controle: a fita armada tem de mudar a tinta"
    );
}
