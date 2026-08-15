//! **Os gates do `Rough`** (plano 38 W6) — o tipo que faz a linha VAGUEAR.
//!
//! As duas propriedades load-bearing são as duas que o doc do [`crate::stroke::roughen`] enuncia, e
//! nenhuma delas é sobre *"o traço mudou"*: **ele é idempotente sob re-carimbo** (senão a figura
//! ferve enquanto o artista a ajusta) e **ele não é o jitter** (senão a row é a segunda porta para
//! um controle que já existe).

use crate::stroke::roughen::offset_at;
use crate::line_kind::LineKind;
use crate::{BrushSpec, Dab, Dynamics, Stroke, StrokeMethod, StrokePoint};

/// Um pincel com o `Rough` armado nas duas oitavas.
fn rough_spec() -> BrushSpec {
    BrushSpec {
        radius_px: 8.0,
        spacing: 0.10,
        stroke_method: StrokeMethod::Space,
        line_kind: LineKind::Rough,
        rough_amount: 0.5,
        rough_bowing: 0.5,
        rough_passes: 2,
        ..BrushSpec::default()
    }
}

/// Carimba um traço reto e devolve os dabs.
///
/// ⚠️ **O `extend` LIMPA o `out` a cada chamada** (é o contrato dele), então acumular é do chamador —
/// a primeira versão deste helper devolvia só o último lote e o CONTROLE (`len > 20`) foi quem
/// disse, em vez de a suíte passar sobre uma fixture de três dabs.
fn run(spec: BrushSpec, n: usize) -> Vec<Dab> {
    let mut st = Stroke::new(spec, Dynamics::default(), 7);
    let mut out = Vec::new();
    st.begin(
        StrokePoint {
            pos: [20.0, 100.0],
            pressure: 1.0,
        },
        &mut out,
    );
    let mut all = std::mem::take(&mut out);
    for i in 1..=n {
        #[allow(clippy::cast_precision_loss)]
        let x = 20.0 + i as f32 * 12.0;
        st.extend(
            StrokePoint {
                pos: [x, 100.0],
                pressure: 1.0,
            },
            &mut out,
        );
        all.extend_from_slice(&out);
    }
    all
}

/// **O MESMO CAMINHO DÁ OS MESMOS DABS, AO BIT** — a propriedade de que os shape editors dependem.
///
/// ⚠️ **Ela é load-bearing e não higiene:** um shape editor re-carimba a figura INTEIRA a cada
/// quadro enquanto o artista a ajusta, então um desvio semeado num contador por-dab faria a figura
/// **FERVER enquanto ele só olha**. É a doença que este módulo nomeia desde o sculpt, e a cura é o
/// desvio ser função PURA do ARCO.
///
/// **Mutação que sangra:** dar ao [`offset_at`] uma semente do RNG do traço em vez do arco (as duas
/// corridas divergem no primeiro dab que sorteia).
#[test]
fn the_same_path_lays_the_same_ink_to_the_bit() {
    let a = run(rough_spec(), 12);
    let b = run(rough_spec(), 12);
    assert_eq!(a.len(), b.len(), "duas corridas do mesmo caminho divergiram");
    assert!(a.len() > 20, "controle: o traço tem de deixar dabs");
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            x.center, y.center,
            "o dab {i} caiu em lugares diferentes em duas corridas do MESMO caminho"
        );
    }
}

/// **O DESVIO VAGUEIA, NÃO TREME** — a linha que separa o `Rough` do jitter.
///
/// ⚠️ **O oráculo é o PASSO entre dabs vizinhos contra a AMPLITUDE**, e não um `assert_ne!` sobre a
/// linha reta: um campo de ruído branco por-dab também moveria a tinta, e passaria num gate que só
/// pedisse *"o traço não é reto"*. O que faz a mão é a COERÊNCIA — dois dabs a um espaçamento de
/// distância têm de estar quase no mesmo lugar do desvio.
///
/// **Mutação que sangra:** trocar o `noise1` por um hash puro do índice (o passo passa a ser da
/// ordem da amplitude, e o traço vira poeira).
#[test]
fn the_wander_is_coherent_not_a_per_dab_tremor() {
    let spec = rough_spec();
    let amp = spec.rough_amount_px() + spec.rough_bowing_px();
    let step_px = spec.spacing * 2.0 * spec.radius_px;
    let mut worst = 0.0f32;
    let mut span = 0.0f32;
    let mut arc = 0.0f32;
    let mut prev = offset_at(&spec, 0.0, 0);
    while arc < 400.0 {
        arc += step_px;
        let d = offset_at(&spec, arc, 0);
        worst = worst.max((d - prev).abs());
        span = span.max(d.abs());
        prev = d;
    }
    assert!(
        span > amp * 0.25,
        "controle: o campo tem de VAGUEAR de verdade (excursão {span:.3} px contra amplitude {amp:.3})"
    );
    // Um espaçamento é 1/10 do diâmetro e o menor comprimento de onda são 4 diâmetros, então o passo
    // não pode passar de uma fração pequena da amplitude — é isto que separa vaguear de tremer.
    assert!(
        worst < amp * 0.20,
        "o desvio salta {worst:.3} px entre dabs vizinhos (amplitude {amp:.3}) -- isto e' JITTER, nao vaguear"
    );
}

/// **DUAS PASSADAS DIVERGEM E SE CRUZAM** — o contorno duplo do Excalidraw.
///
/// ⚠️ **Ele afirma as DUAS metades, e uma sozinha não descreve o desenho:** se as passadas nunca se
/// afastam, é uma linha só; se nunca se cruzam, são duas linhas paralelas (um contorno grosso).
///
/// **Mutação que sangra:** semear as passadas com `pass` cru em vez do primo (os dois campos
/// partilham metade dos argumentos do hash e ficam correlacionados).
#[test]
fn two_passes_diverge_and_cross() {
    let spec = rough_spec();
    let mut apart = 0.0f32;
    let mut saw_left = false;
    let mut saw_right = false;
    let mut arc = 0.0f32;
    while arc < 600.0 {
        let d = offset_at(&spec, arc, 1) - offset_at(&spec, arc, 0);
        apart = apart.max(d.abs());
        if d > 0.5 {
            saw_right = true;
        }
        if d < -0.5 {
            saw_left = true;
        }
        arc += 4.0;
    }
    assert!(
        apart > spec.rough_amount_px() * 0.5,
        "as duas passadas nunca se afastam ({apart:.3} px) -- e' uma linha so'"
    );
    assert!(
        saw_left && saw_right,
        "as duas passadas nunca se CRUZAM (esq={saw_left} dir={saw_right}) -- sao duas paralelas"
    );
}

/// **O NEUTRO É BYTE-IDÊNTICO** — com o tipo desarmado nada deste módulo alcança um dab.
///
/// ⚠️ **As duas metades do desarmado são testadas:** o tipo `None` (outro tipo escolhido) e o tipo
/// `Rough` com as duas amplitudes em zero — o segundo é o que prova que o `rough_active` é a porta,
/// e não o `line_kind` sozinho.
#[test]
fn the_neutral_is_byte_identical() {
    let plain = BrushSpec {
        radius_px: 8.0,
        spacing: 0.10,
        stroke_method: StrokeMethod::Space,
        ..BrushSpec::default()
    };
    let base = run(plain, 12);

    let armed_but_zero = BrushSpec {
        line_kind: LineKind::Rough,
        rough_amount: 0.0,
        rough_bowing: 0.0,
        rough_passes: 2,
        ..plain
    };
    let zero = run(armed_but_zero, 12);
    assert_eq!(
        base.len(),
        zero.len(),
        "o Rough com amplitude zero emitiu um numero diferente de dabs -- as passadas escaparam do `rough_active`"
    );
    for (i, (a, b)) in base.iter().zip(zero.iter()).enumerate() {
        assert_eq!(a.center, b.center, "o dab {i} moveu com o Rough no neutro");
    }
}

/// **O DESVIO É PERPENDICULAR AO TRAÇO** — nunca ao longo dele.
///
/// ⚠️ **Um desvio radial só re-espaçaria os dabs**, e o espaçamento já tem dono; o que faz uma linha
/// parecer desenhada à mão é ela sair do lugar **de lado**. Num traço horizontal isso é medível
/// direto: o `y` tem de vaguear e o `x` tem de continuar a ser o que a mão fez.
///
/// **Mutação que sangra:** trocar a perpendicular pelo próprio heading.
#[test]
fn the_wander_is_perpendicular_to_the_stroke() {
    let dabs = run(rough_spec(), 12);
    let mut ys = (f32::MAX, f32::MIN);
    for d in &dabs {
        ys.0 = ys.0.min(d.center[1]);
        ys.1 = ys.1.max(d.center[1]);
    }
    assert!(
        ys.1 - ys.0 > 2.0,
        "o traco nao vagueou de lado (excursao em y = {:.3} px)",
        ys.1 - ys.0
    );
    // E o x continua a andar monotonicamente: o desvio não re-ordenou o caminho.
    let mut back = 0;
    for w in dabs.windows(2) {
        if w[1].center[0] + 1e-3 < w[0].center[0] {
            back += 1;
        }
    }
    assert!(
        back <= dabs.len() / 4,
        "{back} de {} dabs andaram para TRAS -- o desvio esta' ao longo do traco, nao de lado",
        dabs.len()
    );
}
