//! Os gates da **FAIXA** — as travessas que fazem da fita o *Ribbon Shapes* do Alchemy, e não só a
//! linha atrasada (plano 38 W6).
//!
//! ⚠️ **A premissa da W6 estava ERRADA e a referência a desmentiu**, o que é a razão de este arquivo
//! existir: o plano citava o Alchemy e descrevia o *Dyna* do Krita. A saída do Alchemy é uma **FAIXA
//! com travessas** — dois trilhos, o do dedo e o atrasado, ligados por riscos que se abrem quando a
//! mão acelera. A linha atrasada é **metade** disso, e é o trilho de tinta.
//!
//! **O que estes gates perguntam é a APARÊNCIA da faixa**, nunca a fórmula: *ela existe?* · *ela
//! abre com a velocidade?* · *as travessas ligam instantes CORRESPONDENTES (não um leque)?* · *a
//! cadência é do caminho e não da taxa de eventos?* · *o trilho de fora é uma aresta e não uma fila
//! de pontos?*

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::threads::Thread;
use crate::stroke::{Stroke, StrokePoint};

const DT: f32 = 1.0 / 60.0;

fn spec(kind: LineKind, weight: f32, rungs: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: kind,
        ribbon_weight: weight,
        ribbon_friction: 0.30,
        ribbon_rungs: rungs,
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

fn at(x: f32) -> StrokePoint {
    StrokePoint {
        pos: [x, 300.0],
        pressure: 1.0,
    }
}

/// Um traço RETO a `speed` px/s por `frames` quadros, com `per_frame` eventos por quadro e o tique
/// do produto entre eles. Devolve TODOS os fios costurados durante o gesto (a cauda fica de fora).
///
/// ⚠️ **`take_threads` LIMPA o buffer** (é um dreno), então o helper acumula — ler no fim devolveria
/// só o último quadro, a mesma armadilha de fixture que o `straight` do irmão documenta.
fn sew_straight(sp: BrushSpec, speed: f32, frames: usize, per_frame: usize) -> Vec<Thread> {
    sew_at(sp, speed, frames, per_frame, DT)
}

/// O mesmo gesto com o passo de relógio explícito — é ele que separa *cadência de ARCO* de
/// *cadência de TIQUE*.
fn sew_at(sp: BrushSpec, speed: f32, frames: usize, per_frame: usize, dt: f32) -> Vec<Thread> {
    let mut s = Stroke::new(sp, plain(), 7);
    let mut out = Vec::new();
    let mut buf = Vec::new();
    let mut all = Vec::new();
    let mut x = 100.0f32;
    s.begin(at(x), &mut out);
    #[allow(clippy::cast_precision_loss)]
    let step = speed * dt / per_frame as f32;
    for _ in 0..frames {
        for _ in 0..per_frame {
            x += step;
            s.extend(at(x), &mut out);
        }
        s.tick(dt, &mut out);
        s.take_threads(&mut buf);
        all.append(&mut buf);
    }
    all
}

fn len(t: Thread) -> f32 {
    ((t[2] - t[0]).powi(2) + (t[3] - t[1]).powi(2)).sqrt()
}

/// **A LARGURA DA FAIXA É O ATRASO** — a promessa central do desenho, e é ela que faz a faixa abrir
/// quando a mão acelera e fechar quando ela trava, que é o que se vê na saída do Alchemy.
///
/// ⚠️ **O oráculo é uma RAZÃO entre duas velocidades**, e não um comprimento absoluto: o atraso em
/// regime vale `v · τ`, então dobrar a mão tem de dobrar a travessa. Um número absoluto teria de ser
/// re-calibrado a cada mexida no `RIBBON_LAG_MAX_S` e não afirmaria a lei.
#[test]
fn the_band_is_the_lag_so_it_opens_with_the_hand() {
    let sp = spec(LineKind::Ribbon, 0.45, 0.5);
    // As travessas do FIM do gesto, onde o atraso já está em regime (as do começo ainda crescem).
    let tail = |speed: f32| {
        let all = sew_straight(sp, speed, 90, 1);
        let n = all.len();
        let ult: Vec<f32> = all[n * 3 / 4..].iter().map(|&t| len(t)).collect();
        #[allow(clippy::cast_precision_loss)]
        let m = ult.iter().sum::<f32>() / ult.len() as f32;
        m
    };
    let lento = tail(600.0);
    let rapido = tail(1200.0);
    let razao = rapido / lento;
    assert!(
        (1.6..=2.4).contains(&razao),
        "a faixa tem de abrir com a mão: {lento:.1} px a 600 px/s contra {rapido:.1} a 1200 \
         (razão {razao:.2}, esperada ~2)"
    );
}

/// **AS TRAVESSAS DE UM QUADRO NÃO CONVERGEM NUM PONTO** — o gate do LEQUE.
///
/// ⚠️ Um quadro emite várias travessas, e ligá-las todas ao dedo de AGORA faz um punhado de
/// segmentos apontar para o mesmo ponto: o leque que aparece nas cristas, onde a mão desacelera e o
/// quadro cobre mais arco da fita. As duas pontas de uma travessa têm de ser do MESMO instante, e é
/// a interpolação de [`Stroke::sew_rungs`] que as carimba nele.
#[test]
fn the_rungs_of_one_tick_do_not_fan_to_a_single_point() {
    let mut s = Stroke::new(spec(LineKind::Ribbon, 0.45, 1.0), plain(), 7);
    let mut out = Vec::new();
    let mut buf = Vec::new();
    let mut x = 100.0f32;
    s.begin(at(x), &mut out);
    // Aquece: sem isto a fita ainda está no pen-down e o quadro não emite travessa nenhuma.
    for _ in 0..40 {
        x += 2400.0 * DT;
        s.extend(at(x), &mut out);
        s.tick(DT, &mut out);
        s.take_threads(&mut buf);
        buf.clear();
    }
    x += 2400.0 * DT;
    s.extend(at(x), &mut out);
    s.tick(DT, &mut out);
    s.take_threads(&mut buf);
    let longe: Vec<f32> = buf.iter().map(|t| t[2]).collect();
    assert!(
        longe.len() >= 3,
        "a fixture tem de conter o fenômeno: um quadro com VÁRIAS travessas (tem {})",
        longe.len()
    );
    let (lo, hi) = longe
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(
        hi - lo > 4.0,
        "as pontas de um quadro convergiram num ponto (leque): espalhamento {:.2} px",
        hi - lo
    );
}

/// **A CADÊNCIA É DO CAMINHO, NUNCA DA TAXA DE EVENTOS** — a mesma lei do relógio que a fita já
/// carrega, agora nas travessas.
///
/// ⚠️ Uma travessa por QUADRO faria o número de travessas mudar com a taxa de quadros, e uma por DAB
/// preencheria SÓLIDO (um dab sai a cada ~1,2 px — a primeira tentativa desta wave). O resíduo de
/// arco atravessa os quadros, então o mesmo gesto costura a mesma faixa.
#[test]
fn the_rung_cadence_is_a_fact_of_the_path_not_of_the_pointer_rate() {
    let sp = spec(LineKind::Ribbon, 0.45, 0.5);
    let um = sew_straight(sp, 2400.0, 60, 1);
    let quatro = sew_straight(sp, 2400.0, 60, 4);
    assert!(!um.is_empty(), "o controle não pode ser vazio");
    assert_eq!(
        um.len(),
        quatro.len(),
        "um mouse de 960 Hz tem de costurar a mesma faixa que um de 240 Hz"
    );
    // ⚠️ **E a outra metade é o RELÓGIO, que é a que o resíduo de arco compra.** O mesmo caminho no
    // mesmo tempo, com o dobro dos tiques: sem o `ribbon_rung_accum` a atravessar os quadros, cada
    // tique recomeçaria a contagem do zero e a faixa perderia travessas com a taxa de quadros — o
    // desenho passaria a depender de quão rápido a máquina desenha.
    let denso = sew_at(sp, 2400.0, 120, 1, DT / 2.0);
    let d = (i64::try_from(um.len()).unwrap_or(0) - i64::try_from(denso.len()).unwrap_or(0)).abs();
    assert!(
        d <= 2,
        "a 120 Hz a faixa mudou: {} travessas contra {} a 60 Hz",
        denso.len(),
        um.len()
    );
}

/// **O TRILHO DE FORA É UMA ARESTA, NÃO UMA FILA DE PONTOS** — sem o segmento entre duas pontas
/// consecutivas a borda externa da faixa é pontilhada, e o desenho deixa de ler como faixa.
///
/// O oráculo é a CONECTIVIDADE: uma ponta interior é extremo de três segmentos (a sua travessa e os
/// dois trechos do trilho que a ladeiam). Sem o trilho ela seria extremo de um só.
#[test]
fn the_far_rail_joins_consecutive_rungs_into_an_edge() {
    let all = sew_straight(spec(LineKind::Ribbon, 0.45, 0.5), 2400.0, 60, 1);
    assert!(
        all.len() >= 9,
        "a fixture precisa de várias travessas (tem {})",
        all.len()
    );
    let mut contagem: std::collections::BTreeMap<(u32, u32), usize> =
        std::collections::BTreeMap::new();
    for t in &all {
        for p in [(t[0], t[1]), (t[2], t[3])] {
            *contagem.entry((p.0.to_bits(), p.1.to_bits())).or_default() += 1;
        }
    }
    let interiores = contagem.values().filter(|&&n| n >= 3).count();
    assert!(
        interiores >= all.len() / 4,
        "as pontas não estão ligadas entre si: só {interiores} de {} extremos aparecem 3× \
         (o trilho de fora não foi costurado)",
        contagem.len()
    );
}

/// **A CAUDA NÃO COSTURA** — no pen-up o trilho do DEDO acabou, e ligar a cauda ao ponto onde a
/// caneta levantou é literalmente o leque que o gate acima existe para não desenhar.
#[test]
fn the_tail_sews_nothing_because_the_finger_rail_ended() {
    let mut s = Stroke::new(spec(LineKind::Ribbon, 1.0, 0.5), plain(), 7);
    let mut out = Vec::new();
    let mut buf = Vec::new();
    let mut x = 100.0f32;
    s.begin(at(x), &mut out);
    for _ in 0..60 {
        x += 2400.0 * DT;
        s.extend(at(x), &mut out);
        s.tick(DT, &mut out);
    }
    s.take_threads(&mut buf);
    assert!(
        !buf.is_empty(),
        "controle: o gesto tem de ter costurado a faixa"
    );
    buf.clear();
    s.finish(&mut out);
    s.take_threads(&mut buf);
    assert!(
        buf.is_empty(),
        "a cauda costurou {} fios — eles apontariam todos para a caneta parada",
        buf.len()
    );
}

/// **QUEM COSTURA É O TIPO **E** O KNOB DAQUELE TIPO** — a porta única, e o CONTROLE que prova que
/// `Rungs = 0` degenera na linha atrasada sozinha (o pincel de arrasto), sem custo de fio nenhum.
///
/// ⚠️ **Este gate nasceu de um defeito MEDIDO:** havia duas respostas para *"este pincel costura?"* —
/// o portão do enum e o do spec — e o doc do segundo dizia consultar o primeiro sem o consultar.
/// Ligar a fita no portão do enum deixou o motor a costurar **343 travessas por traço** com o
/// depósito **mudo**. O portão do enum não existe mais.
#[test]
fn only_a_kind_with_its_own_knob_armed_sews_threads() {
    for (kind, rungs, deve) in [
        (LineKind::None, 0.5, false),
        (LineKind::Speed, 0.5, false),
        (LineKind::Ribbon, 0.0, false),
        (LineKind::Ribbon, 0.5, true),
    ] {
        let sp = spec(kind, 0.45, rungs);
        assert_eq!(
            sp.sews_threads(),
            deve,
            "{kind:?} com Rungs {rungs}: a porta respondeu {} e devia responder {deve}",
            sp.sews_threads()
        );
        let all = sew_straight(sp, 2400.0, 40, 1);
        assert_eq!(
            !all.is_empty(),
            deve,
            "{kind:?} com Rungs {rungs}: o motor costurou {} fios",
            all.len()
        );
    }
}

/// **UMA FITA SEM ATRASO NÃO TEM FAIXA** — a largura da faixa É o atraso, então sem ele os dois
/// trilhos coincidem e cada travessa é um segmento de comprimento zero: fios gastos a pintar nada.
#[test]
fn a_ribbon_with_no_lag_has_no_band_to_draw() {
    let sp = spec(LineKind::Ribbon, 0.0, 1.0);
    assert!(
        !sp.ribbon_band_active(),
        "sem atraso não há faixa a costurar"
    );
    let all = sew_straight(sp, 2400.0, 40, 1);
    assert!(
        all.is_empty(),
        "costurou {} fios de comprimento zero",
        all.len()
    );
}
