//! Os gates da **FITA** — o `LineKind::Ribbon` do plano 38 W6.
//!
//! O que a feature promete: *o traço PESA*. Os gates perguntam pelo PESO — o atraso que cresce com a
//! velocidade, o chicote que ultrapassa, o pendurar sob a gravidade e a cauda que chega —, nunca
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
    /// Corre para a direita, PARA de repente, e devolve o maior `x` que a tinta alcançou menos o
    /// `x` onde a mão parou. Positivo = ultrapassou.
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
        let mut x = start[0];
        for _ in 0..60 {
            x += 3600.0 * DT;
            s.extend(
                StrokePoint {
                    pos: [x, start[1]],
                    pressure: 1.0,
                },
                &mut out,
            );
            s.tick(DT, &mut out);
        }
        // A mão PARA: o alvo deixa de andar, e só o que tem inércia continua.
        let stop = x;
        let mut far = start[0];
        for _ in 0..180 {
            s.extend(
                StrokePoint {
                    pos: [stop, start[1]],
                    pressure: 1.0,
                },
                &mut out,
            );
            s.tick(DT, &mut out);
            for d in &out {
                far = far.max(d.center[0]);
            }
        }
        far - stop
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

/// **A GRAVIDADE FAZ A FITA PENDER, e a queda é `g·τ²`** — a mão parada, a tinta desce.
///
/// ⚠️ A fórmula é afirmada como NÚMERO, não como forma: um pendurar que só *acontece* passaria com
/// qualquer constante, e é o `g·τ²` que torna os dois knobs previsíveis um contra o outro.
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
    for _ in 0..180 {
        s.extend(
            StrokePoint {
                pos: at,
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

/// **A FITA É INTEGRADA NO RELÓGIO, NÃO NO EVENTO** — o mesmo caminho, no mesmo tempo, entregue em
/// 1 e em 8 eventos por quadro, dá a MESMA fita.
///
/// ⚠️ É a lei que este módulo aprendeu quatro vezes no relevo (*a grandeza é fato do CAMINHO e do
/// RELÓGIO, nunca de quão fino o dispositivo amostrou o caminho*), aplicada à mola: se ela fosse
/// integrada no `extend`, um mouse de 960 Hz desenharia outra fita que um de 125 Hz.
#[test]
fn the_ribbon_is_a_fact_of_the_clock_not_of_the_pointer_rate() {
    fn run_with(sp: BrushSpec, per_frame: usize) -> f32 {
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
        let mut last = start;
        let mut x = start[0];
        #[allow(clippy::cast_precision_loss)]
        let step = 2400.0 * DT / per_frame as f32;
        for _ in 0..120 {
            for _ in 0..per_frame {
                x += step;
                s.extend(
                    StrokePoint {
                        pos: [x, start[1]],
                        pressure: 1.0,
                    },
                    &mut out,
                );
                // ⚠️ **As DUAS portas, e a primeira versão desta fixture lia só a segunda.** Numa
                // fita quem emite é o tique; num traço comum é o `extend` — um helper que lê só um
                // deles mede `x − start` no outro e reporta um número que não é atraso nenhum
                // (medido: o controle dava `4800 contra 4800`, o comprimento do traço).
                if let Some(d) = out.last() {
                    last = d.center;
                }
            }
            s.tick(DT, &mut out);
            if let Some(d) = out.last() {
                last = d.center;
            }
        }
        x - last[0]
    }
    let run = |n| run_with(spec(LineKind::Ribbon, 0.45, 0.30, 0.0), n);
    let sparse = run(1);
    let dense = run(8);
    assert!(
        sparse > 100.0,
        "a fixture não contém fita nenhuma: {sparse:.1} px"
    );
    assert!(
        (sparse - dense).abs() < 10.0,
        "a fita mudou com a taxa do ponteiro: {sparse:.1} px contra {dense:.1}"
    );
    // ⚠️ **CONTROLE NEGATIVO — o estabilizador FALHA esta invariância**, e é isso que a torna uma
    // propriedade e não uma trivialidade da fixture: ele filtra por EVENTO, então oito amostras por
    // quadro dão oito passos de convergência e a mão dele muda com o dispositivo.
    let mut stab = spec(LineKind::None, 0.0, 0.0, 0.0);
    stab.stabilizer = 0.9;
    let (s_sparse, s_dense) = (run_with(stab, 1), run_with(stab, 8));
    assert!(
        (s_sparse - s_dense).abs() > 40.0,
        "controle: o estabilizador DEVERIA mudar com a taxa ({s_sparse:.1} contra {s_dense:.1}) — \
         se ele parou de mudar, esta invariância deixou de dizer algo sobre a fita"
    );
}

/// **A MÃO PARADA ASSENTA** — o piso do amortecimento é o que impede um traço de crescer para sempre.
///
/// ⚠️ **O oráculo é o SILÊNCIO, não a contagem** — quantos dabs a fita deixa ao chegar depende do
/// quanto ela estava atrás, e isso é legítimo; o que não pode acontecer é ela nunca parar. A tabela
/// que escolheu o piso vive no doc do `RIBBON_DAMPING_MIN` (11 840 dabs e nunca em silêncio a `ζ=0`).
#[test]
fn a_parked_ribbon_falls_silent() {
    // O canto mais whippy que o slider alcança: peso máximo, atrito no PISO.
    let mut s = Stroke::new(spec(LineKind::Ribbon, 1.0, 0.0, 0.0), plain(), 7);
    let mut out = Vec::new();
    s.begin(
        StrokePoint {
            pos: [100.0, 300.0],
            pressure: 1.0,
        },
        &mut out,
    );
    for i in 1..=20 {
        #[allow(clippy::cast_precision_loss)]
        let x = 100.0 + (i as f32) * 20.0;
        s.extend(
            StrokePoint {
                pos: [x, 300.0],
                pressure: 1.0,
            },
            &mut out,
        );
        s.tick(DT, &mut out);
    }
    let mut silent_at = None;
    for t in 0..1800 {
        s.tick(DT, &mut out);
        if out.is_empty() {
            if silent_at.is_none() {
                silent_at = Some(t);
            }
        } else {
            silent_at = None;
        }
    }
    let at = silent_at.expect("a fita parada nunca ficou em silêncio");
    #[allow(clippy::cast_precision_loss)]
    let secs = at as f32 * DT;
    assert!(
        secs < 15.0,
        "a fita parada só assentou aos {secs:.1} s — o piso do amortecimento não está a segurar"
    );
}

/// **UMA ENGASGADA NÃO EXPLODE A MOLA** — um quadro de 200 ms sobre a mola mais rígida que o slider
/// alcança, e a fita continua na tela.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE, e o doc do `step_ribbon` já a tinha previsto:**
/// *"um quadro lento entregaria um `dt` grande sobre uma mola rígida e a fita explodiria para fora
/// da tela — com todo gate de unidade verde, porque a unidade nunca vê um quadro de 200 ms"*. Tirar
/// os sub-passos não fazia nenhum dos sete gates sangrar, porque **todos eles ticam a 60 fps**. A
/// previsão estava escrita; faltava a fixture que a contém.
///
/// O peso é o MENOR que ainda arma a fita (`τ` pequeno ⇒ `ω` enorme), que é onde o Euler
/// semi-implícito de passo único diverge.
#[test]
fn a_stalled_frame_does_not_blow_the_spring_up() {
    let (excursion, last) = stalled_excursion(spec(LineKind::Ribbon, 0.02, 0.30, 0.0));
    assert!(
        excursion < STALL_EXCURSION_PX,
        "a fita explodiu num quadro de 200 ms: {excursion:.1} px FORA do gesto"
    );
    assert!(
        (last[0] - 700.0).abs() < ARRIVAL_TOL_PX,
        "a fita nem chegou ao alvo: parou em {:.1}",
        last[0]
    );
}

/// **A CAUDA NÃO CORRE ATÉ O DEDO** — a espícula do report de 2026-08-15.
///
/// ⚠️ **O oráculo é a POSIÇÃO onde a tinta acaba, e não um salto entre dabs.** Uma espícula é uma
/// corrida **reta de dabs normalmente espaçados** numa direção que o gesto não tem, então uma sonda
/// de *salto* mede **zero** sobre ela — foi o que aconteceu, e custou três hipóteses.
///
/// **O que a foto mostrava:** a cauda levava a tinta de `x = 947,2` a `x = 1316,8` — **369 px em
/// 154 dabs**, largura cheia, atravessando o desenho, uma reta por traço. **Mecanismo:** a mola
/// continuava presa ao cursor durante a cauda, e uma mola **CONVERGE** para o alvo; com o alvo
/// parado no dedo, convergir é andar em linha reta até ele.
///
/// ⚠️ **Ele SUBSTITUI o gate `the_tail_arrives_when_the_pen_lifts`, que eu escrevi na mesma wave e
/// que afirmava o DEFEITO como lei** — a mensagem de falha dele era *"a cauda parou LONGE do
/// dedo"*. As duas metades verdadeiras daquele gate (a cauda pinta, e ela ANDA) vivem aqui, com a
/// barra no número medido; a terceira, *"para a menos de 40 px do dedo"*, era a espícula. ⚠️ E o
/// **doc dele contradizia a própria asserção** (*"não é um salto até o dedo"* sobre um `assert` que
/// exigia pousar a 40 px): quando as duas discordam, é o `assert` que shipa.
///
/// ⚠️ **A mola só é cortada no PEN-UP, nunca numa pausa.** Se a mão para sem levantar, a fita
/// continua a ser puxada e alcança o dedo — que é o certo, e é o que o gate irmão
/// `a_stalled_frame_does_not_blow_the_spring_up` afirma ao exigir que ela CHEGUE.
#[test]
fn the_tail_does_not_run_to_the_finger() {
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
    let fim = out.last().map_or(antes, |d| d.center);
    // A tinta acaba onde a FITA parou. Chegar ao dedo é o gancho que a física não produziu.
    assert!(
        fim[0] < x - 100.0,
        "a cauda correu ate o dedo: soltou em {x:.1}, a tinta acabou em {:.1}",
        fim[0]
    );
    // E ela ANDA -- uma cauda que não anda é um corte seco, e o traço perderia a inércia que a
    // feature inteira promete.
    assert!(
        fim[0] > antes[0] + 100.0,
        "a cauda nem andou: {:.1} -> {:.1}",
        antes[0],
        fim[0]
    );
}

/// **UM BREAKPOINT NÃO É UMA ENGASGADA — e só o cap de `dt` cobre o segundo caso.**
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO QUE SOBREVIVEU:** tirar o
/// [`crate::line_kind::RIBBON_MAX_STEP_S`] deixava o
/// `a_stalled_frame_does_not_blow_the_spring_up` **VERDE**, porque num quadro de 200 ms o
/// [`crate::line_kind::RIBBON_MAX_SUBSTEPS`] sozinho já segura (`n` pedido 400, aplicado 134 ⇒
/// `ω · h = 0,75`, ainda estável). São **duas camadas**, cada uma suficiente naquele ponto de
/// operação — e uma defesa em camadas precisa de um gate POR CAMADA
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
///
/// A camada que só o cap compra é o travamento LONGO: com 10 s de quadro e o teto de sub-passos
/// intacto, `h = 10/134 = 74,6 ms` e `ω · h = 37,3` — divergência, com o batente a fazer
/// exactamente o que ele foi escrito para nunca fazer. **O cap não deixa o pedido nascer.**
#[test]
fn a_breakpoint_length_stall_is_capped_by_work_not_by_substeps() {
    // Dez segundos: o artista voltou de um breakpoint. Nenhuma taxa de quadros produz isto, e é
    // por isso que a defesa tem de ser sobre o TRABALHO de um tique, nunca sobre a resolução dele.
    let (excursion, last) = stalled_excursion_at(spec(LineKind::Ribbon, 0.02, 0.30, 0.0), 10.0);
    assert!(
        excursion < STALL_EXCURSION_PX,
        "a fita explodiu num quadro de 10 s: {excursion:.1} px FORA do gesto"
    );
    assert!(
        (last[0] - 700.0).abs() < ARRIVAL_TOL_PX,
        "a fita nem chegou ao alvo: parou em {:.1}",
        last[0]
    );
}

/// Quanto um dab pode pousar FORA da caixa do gesto antes de ser divergência, em px.
///
/// ⚠️ **Frouxo de propósito, e ainda assim afiado:** uma mola instável não sai da caixa por um
/// punhado de pixels — ela vai a `1e78` em dez quadros. O que este número tem de fazer é caber
/// acima do overshoot LEGÍTIMO (o chicote, que é a feature) e muito abaixo de qualquer divergência.
const STALL_EXCURSION_PX: f32 = 100.0;

/// Quão perto do alvo o ÚLTIMO dab tem de pousar, em px.
///
/// ⚠️ **Um ESPAÇAMENTO, não um pixel** — o último dab pousa na última fronteira de espaçamento que o
/// percurso cruzou, nunca na posição exata da ponta da fita. Medido, a quina para a 2,4 px do alvo,
/// que É o passo do pincel de referência: a 1ª versão deste gate pedia `< 1,0` e reprovava produto
/// correto, porque pedir precisão sub-espaçamento a um emissor de DABS é perguntar a coisa errada.
const ARRIVAL_TOL_PX: f32 = 4.0;

/// Dirige a mão a SALTAR 300 px com o processo a ENGASGAR (`dt = 0,2 s`, dez vezes) e devolve
/// `(excursão para fora do gesto, último dab)`.
///
/// ⚠️ **O oráculo é a EXCURSÃO, nunca a distância ao ALVO — e a 1ª versão deste gate usava a
/// distância.** Uma fita ATRASA por construção, então nos primeiros quadros ela está legitimamente a
/// ~300 px do alvo: um limite sobre essa distância reprova o produto CORRETO. Ninguém tinha visto,
/// porque o gate original **nunca chegou ao `assert`** — ele morria a alocar (ver o achado do OOM),
/// e um oráculo que nunca correu verde nunca foi observado. Uma mola que diverge SAI da caixa do
/// gesto; uma que atrasa, fica dentro dela.
fn stalled_excursion(sp: BrushSpec) -> (f32, [f32; 2]) {
    stalled_excursion_at(sp, 0.2)
}

/// O mesmo, com o tamanho do quadro travado como parâmetro — 200 ms é uma engasgada de GC, e
/// segundos é um breakpoint. ⚠️ **Os dois números medem camadas DIFERENTES da mesma defesa**, e
/// mudá-lo é o que separa o teto de sub-passos do cap de `dt`.
fn stalled_excursion_at(sp: BrushSpec, stall_dt: f32) -> (f32, [f32; 2]) {
    let at = [400.0f32, 300.0];
    let to = [700.0f32, 300.0];
    let mut s = Stroke::new(sp, plain(), 7);
    let mut out = Vec::new();
    s.begin(
        StrokePoint {
            pos: at,
            pressure: 1.0,
        },
        &mut out,
    );
    s.extend(
        StrokePoint {
            pos: to,
            pressure: 1.0,
        },
        &mut out,
    );
    let (lo_x, hi_x) = (at[0].min(to[0]), at[0].max(to[0]));
    let (lo_y, hi_y) = (at[1].min(to[1]), at[1].max(to[1]));
    let mut excursion = 0.0f32;
    let mut last = at;
    for _ in 0..10 {
        out.clear();
        s.tick(stall_dt, &mut out);
        for d in &out {
            let dx = (lo_x - d.center[0]).max(d.center[0] - hi_x).max(0.0);
            let dy = (lo_y - d.center[1]).max(d.center[1] - hi_y).max(0.0);
            excursion = excursion.max(dx.max(dy));
            last = d.center;
        }
    }
    (excursion, last)
}

/// **O BATENTE DE SUB-PASSOS NUNCA MORDE** — as duas metades de uma promessa só.
///
/// A [`crate::line_kind::RIBBON_SUBSTEP_FRACTION`] promete `ω · h = 0,25` **sempre**, e o
/// [`crate::line_kind::RIBBON_MAX_SUBSTEPS`] é um BATENTE contra laço em fuga — não um regulador.
/// Se ele morder, a promessa deixou de valer em silêncio e a mola volta a divergir.
///
/// ⚠️ **Este gate existe porque eu escrevi a const contra o `dt` ERRADO.** Derivei `34` de um quadro
/// de 60 fps quando o que ela tem de cobrir é o `dt` que o [`crate::line_kind::RIBBON_MAX_STEP_S`]
/// deixa passar — **quatro** quadros, logo **134**. Com 34 o batente mordia no piso do slider e, com
/// o atrito no topo, o maior autovalor voltava a **3,68 > 1**.
///
/// ⚠️ **Por isso a 1ª metade afirma a ARITMÉTICA das três consts, nunca um literal escrito à mão** —
/// um número à mão erra junto com quem o escreveu, que é exactamente o que aconteceu. E a 2ª metade
/// dirige o PRODUTO na quina mais dura que o artista alcança (peso no piso ⇒ `ω` máximo, atrito no
/// topo ⇒ `c` máximo, quadro engasgado), porque uma aritmética certa sobre um integrador trocado
/// continuaria verde.
#[test]
fn the_substep_ceiling_can_never_bind() {
    use crate::line_kind::{
        RIBBON_LAG_MIN_S, RIBBON_MAX_STEP_S, RIBBON_MAX_SUBSTEPS, RIBBON_SUBSTEP_FRACTION,
    };
    // (1) A ARITMÉTICA: o pior caso alcançável é `dt` no teto sobre `τ` no piso.
    let needed = (RIBBON_MAX_STEP_S / (RIBBON_SUBSTEP_FRACTION * RIBBON_LAG_MIN_S)).ceil();
    assert!(
        needed <= RIBBON_MAX_SUBSTEPS as f32,
        "o batente MORDE: {needed} sub-passos precisos contra {RIBBON_MAX_SUBSTEPS} aplicados \
         -- com ele a mordê-lo, `omega*h` sai de {RIBBON_SUBSTEP_FRACTION} e a mola diverge"
    );

    // (2) O PRODUTO na quina: o menor peso que ainda arma a fita, o maior atrito, um quadro
    // engasgado. `far` mede a distância ao alvo — uma mola que diverge sai da tela em dois passos.
    let sp = spec(LineKind::Ribbon, f32::EPSILON, 1.0, 0.0);
    assert_eq!(
        sp.ribbon_lag_s(),
        RIBBON_LAG_MIN_S,
        "premissa da fixture: este peso tem de pousar no PISO do atraso"
    );
    let (excursion, last) = stalled_excursion(sp);
    assert!(
        excursion < STALL_EXCURSION_PX,
        "a fita divergiu na quina mais dura do slider: {excursion:.1} px FORA do gesto"
    );
    assert!(
        (last[0] - 700.0).abs() < ARRIVAL_TOL_PX,
        "a fita nem chegou ao alvo na quina: parou em {:.1}",
        last[0]
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
