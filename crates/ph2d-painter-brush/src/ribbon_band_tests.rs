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

/// **AS TRAVESSAS DE UM QUADRO NÃO CONVERGEM NUM PONTO** — o gate do leque DENTRO de um tique.
///
/// ⚠️ Um quadro emite várias travessas, e ligá-las todas ao dedo de AGORA faz um punhado de
/// segmentos apontar para o mesmo ponto. As duas pontas de uma travessa têm de ser do MESMO
/// instante, e é a interpolação de [`Stroke::sew_rungs`] que as carimba nele.
///
/// ⚠️ **A fixture é um arrasto RETO a velocidade CONSTANTE, e é de propósito — mas o doc deste gate
/// afirmava cobrir *"as cristas, onde a mão desacelera"*, que ela não tem.** A frase estava certa
/// sobre o fenômeno e errada sobre esta régua: o leque das cristas vive ATRAVÉS de tiques, não
/// dentro de um, e nenhuma interpolação o alcança. Quem o mede é o irmão
/// [`the_rungs_do_not_fan_across_ticks_when_the_two_rails_disagree`]; este continua a guardar a
/// interpolação, que é outra metade e continua load-bearing.
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

/// **O PEN-UP NÃO COSTURA** — no pen-up o trilho do DEDO acabou, e ligar seja o que for ao ponto
/// onde a caneta levantou é literalmente o leque que o gate acima existe para não desenhar.
///
/// ⚠️ **O nome dizia CAUDA e a cauda já não existe** (inibida por ordem do Enio, 2026-08-15): a
/// asserção continua a valer e mudou de dono — ela guardava *"a cauda não costura"* e guarda agora
/// *"o pen-up não costura"*, que é mais forte e o subsume. A metade de DABS da mesma lei vive no
/// [`crate::ribbon_tests`] (`the_pen_up_adds_nothing_because_the_ribbon_ends_where_it_is`); esta é a
/// do canal de FIOS, e são perguntas diferentes — um pen-up que voltasse a percorrer poderia
/// costurar sem carimbar.
#[test]
fn the_pen_up_sews_nothing_because_the_finger_rail_ended() {
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
        "o pen-up costurou {} fios — eles apontariam todos para a caneta parada",
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

/// **O LEQUE ATRAVÉS DE TIQUES** — quando a mão desacelera, a faixa não pode virar um punhado de
/// travessas espetadas no mesmo ponto.
///
/// ⚠️ **É a outra metade do leque, e a interpolação não a alcança.** A irmã
/// [`the_rungs_of_one_tick_do_not_fan_to_a_single_point`] guarda o caso ESPELHO — dedo rápido, fita
/// lenta, várias travessas num quadro — e a fração `f` o resolve. Este é o caso em que a mão
/// desacelera numa crista **enquanto a fita descarrega atraso**: cada quadro emite uma ou duas
/// travessas, todas com a ponta de fora praticamente no mesmo sítio, e o leque cresce quadro a
/// quadro. Nenhuma fração dentro de um tique o vê.
///
/// **A lei que o fecha:** a faixa avança o que os DOIS trilhos avançaram (`min`), e não o arco da
/// fita — duas travessas só são distinguíveis se as duas pontas andaram.
///
/// Medido nesta fixture: a cadência só pelo arco da FITA abre **297,0 px em 12 travessas** no ápice
/// do dedo; só pelo arco do DEDO abre **54,0 px em 3** no ápice da fita; o `max` repete os 297,0. O
/// `min` é o único dos quatro que fecha as duas metades. (Na sonda do gesto ondulado, com o peso que
/// shipa, o leque era **148,5 px em 12 travessas** — 27,0 · 94,5 · 148,5 nos pesos 0,08 · 0,20 ·
/// 0,45: ele cresce com o atraso, que é o que ele tem de descarregar.)
/// ⚠️ **E a fixture TEM de conter o fenômeno**: a asserção de contagem abaixo é o que impede este
/// gate de ficar verde sobre uma faixa que não costurou travessa nenhuma.
#[test]
fn the_rungs_do_not_fan_across_ticks_when_the_two_rails_disagree() {
    let d2 = |a: [f32; 2], b: [f32; 2]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    let mut s = Stroke::new(spec(LineKind::Ribbon, 0.45, 0.5), plain(), 7);
    let (mut out, mut buf) = (Vec::new(), Vec::new());
    let mut x = 100.0f32;
    s.begin(at(x), &mut out);
    let mut travessas: Vec<Thread> = Vec::new();
    for i in 0..80 {
        // ⚠️ **O gesto tem as DUAS assimetrias, e a segunda foi escrita por uma mutação que
        // sobreviveu:** corre · FREIA (fita depressa, dedo devagar ⇒ leque na ponta de FORA) ·
        // ARRANCA (dedo depressa, fita ainda devagar ⇒ leque na ponta de PERTO). Medir a cadência
        // pelo arco do DEDO fecha a primeira e abre a segunda, e passava nas duas réguas anteriores.
        // A freada não PARA: o piso de meio pixel recusaria o tique, e a pergunta aqui é sobre a mão
        // a desacelerar.
        x += if (40..60).contains(&i) { 1.5 } else { 40.0 };
        s.extend(at(x), &mut out);
        s.tick(DT, &mut out);
        s.take_threads(&mut buf);
        for (j, t) in buf.drain(..).enumerate() {
            if j % 2 == 0 {
                travessas.push(t);
            }
        }
    }
    assert!(
        travessas.len() >= 20,
        "a fixture tem de conter o fenômeno: só {} travessas em 80 quadros",
        travessas.len()
    );
    // A corrida de travessas consecutivas que partilham UMA ponta, medida na OUTRA. As duas metades
    // pela mesma régua: um leque é um leque, esteja o ápice no trilho da fita ou no do dedo.
    let corrida_max = |partilha: fn(&Thread) -> [f32; 2], abre: fn(&Thread) -> [f32; 2]| {
        let (mut corrida, mut pior, mut pior_n, mut n) = (0.0f32, 0.0f32, 0usize, 1usize);
        for par in travessas.windows(2) {
            let (a, b) = (&par[0], &par[1]);
            if d2(partilha(a), partilha(b)) < 3.0 {
                corrida += d2(abre(a), abre(b));
                n += 1;
                if corrida > pior {
                    pior = corrida;
                    pior_n = n;
                }
            } else {
                corrida = 0.0;
                n = 1;
            }
        }
        (pior, pior_n)
    };
    let perto = |t: &Thread| [t[0], t[1]];
    let longe = |t: &Thread| [t[2], t[3]];
    let (fora, fora_n) = corrida_max(longe, perto);
    assert!(
        fora <= 40.0,
        "{fora_n} travessas espetadas no mesmo ponto do DEDO, abrindo {fora:.1} px no trilho da \
         fita: a cadência está a ser medida só no arco da fita"
    );
    let (dentro, dentro_n) = corrida_max(perto, longe);
    assert!(
        dentro <= 40.0,
        "{dentro_n} travessas espetadas no mesmo ponto da FITA, abrindo {dentro:.1} px no trilho \
         do dedo: a cadência está a ser medida só no arco do dedo"
    );
}
