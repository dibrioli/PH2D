//! **O RELÓGIO DA FITA** — o que conta como TEMPO para ela, e o que um quadro lento lhe faz.
//!
//! Irmão dos gates do [`super::ribbon_tests`], que perguntam pelo **PESO** (o atraso, o chicote, o
//! pendurar). Aqui a pergunta é outra, e o corte é de assunto:
//!
//! - **Integrar em SEGUNDOS** — um mouse de 960 Hz desenha o que um de 125 Hz desenha.
//! - **SEM GESTO, SEM TEMPO** — um tique em que o dedo não andou não entrega tempo, porque uma mola
//!   que converge para um alvo parado desenha uma reta de largura cheia: a espícula.
//!
//! ⚠️ As duas **não se contradizem**: a primeira torna o desenho independente do dispositivo, a
//! segunda torna-o função do que a mão fez. Foi confundi-las que me fez tratar a espícula como um
//! defeito do pen-up, quando ela é de QUALQUER instante em que a mão pára.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::LineKind;
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

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

/// **A MÃO PARADA NÃO DEIXA TINTA — e é a lei que mata a espícula na raiz.**
///
/// ⚠️ **Este gate mudou de PREMISSA, e a antiga foi encarada em vez de deixada verde.** Ele afirmava
/// que a fita *acaba por* assentar (o piso do amortecimento existia para um traço não crescer para
/// sempre com a mão parada), e sob *sem gesto, sem tempo* ([`Stroke::tick_ribbon`]) ele passaria a
/// ser **trivialmente verdadeiro no tique 0** — verde sobre uma pergunta que já não pode falhar.
///
/// A pergunta que vale hoje é mais forte e é a do report: **uma mola que converge para um alvo
/// parado desenha uma LINHA RETA de largura cheia**, e ela não pode ser desenhada em instante
/// nenhum. Medido por ablação na porta do produto, a PAUSA acrescentava **18 013 px de tinta, 5 715
/// deles escuros**.
///
/// ⚠️ **E a segunda metade é o que separa CONGELAR de descartar:** a fita guarda posição *e*
/// velocidade, então retomar o gesto continua de onde parou. Sem isso, parar e voltar daria um salto
/// — o artefato que a lei existe para não trocar por outro.
#[test]
fn a_parked_hand_lays_no_ink_and_the_ribbon_keeps_its_state() {
    let mut s = Stroke::new(spec(LineKind::Ribbon, 1.0, 0.0, 0.0), plain(), 7);
    let mut out = Vec::new();
    let em = |x: f32| StrokePoint {
        pos: [x, 300.0],
        pressure: 1.0,
    };
    s.begin(em(100.0), &mut out);
    for i in 1..=20 {
        #[allow(clippy::cast_precision_loss)]
        s.extend(em(100.0 + (i as f32) * 40.0), &mut out);
        s.tick(DT, &mut out);
    }
    let onde = out.last().map_or([0.0, 0.0], |d| d.center);
    // A MÃO PARA, o botão continua preso: cem tiques, e nenhum pode pintar.
    let mut pintou = 0usize;
    for _ in 0..100 {
        s.extend(em(900.0), &mut out);
        s.tick(DT, &mut out);
        pintou += out.len();
    }
    assert_eq!(
        pintou, 0,
        "a mão parada deixou {pintou} dabs — é a espícula: a convergência para um alvo parado é uma reta"
    );
    // E ao RETOMAR ela continua de onde estava, sem salto.
    //
    // ⚠️ **O oráculo é o TRABALHO do tique retomado, não a posição do primeiro dab** — e a mutação
    // pegou a minha primeira versão: o percurso parte do `last_pos`, que a pausa não move, então o
    // primeiro dab depois da pausa cai ao lado de `onde` **mesmo que a fita tenha teleportado para o
    // dedo**. Uma fita que descarta o estado percorre o atraso inteiro num tique só, e isso é
    // visível na CONTAGEM: dezenas de dabs em vez de meia dúzia.
    s.extend(em(940.0), &mut out);
    s.tick(DT, &mut out);
    let retomada = out.len();
    let _ = onde;
    assert!(
        retomada <= 40,
        "o tique retomado carimbou {retomada} dabs: a fita não CONGELOU, ela descartou o estado e \
         percorreu o atraso inteiro de uma vez (medido: 13 congelando, 239 descartando)"
    );
}

/// **UM PINO TREME, E TREMOR NÃO É GESTO** — o piso do [`crate::line_kind::RIBBON_PARK_EPS_PX`].
///
/// ⚠️ **Um rato parado entrega a MESMA coordenada; uma caneta parada, não.** Com a mão pousada, o
/// digitalizador entrega ±0,5 px de ruído a cada amostra, e sem o piso cada um desses tremores conta
/// como gesto: o tique integra, a mola converge para um alvo que não anda, e a **espícula volta —
/// só no tablet**. A fixture de rato (coordenada exata) é CEGA a isto, e foi ela que deixou a
/// mutação do piso sobreviver.
#[test]
fn a_trembling_parked_pen_is_not_a_gesture() {
    let mut s = Stroke::new(spec(LineKind::Ribbon, 1.0, 0.0, 0.0), plain(), 7);
    let mut out = Vec::new();
    let em = |x: f32, y: f32| StrokePoint {
        pos: [x, y],
        pressure: 1.0,
    };
    s.begin(em(100.0, 300.0), &mut out);
    for i in 1..=20 {
        #[allow(clippy::cast_precision_loss)]
        s.extend(em(100.0 + (i as f32) * 40.0, 300.0), &mut out);
        s.tick(DT, &mut out);
    }
    // A mão POUSA e a caneta treme: ruído sub-pixel determinístico, dos dois lados.
    let mut pintou = 0usize;
    for i in 0..100 {
        #[allow(clippy::cast_precision_loss)]
        let n = (i as f32 * 1.7).sin() * 0.3;
        s.extend(em(900.0 + n, 300.0 - n), &mut out);
        s.tick(DT, &mut out);
        pintou += out.len();
    }
    assert_eq!(
        pintou, 0,
        "o tremor da caneta pousada deixou {pintou} dabs — o piso não está a segurar"
    );
}

/// **A CAUDA do pen-up ASSENTA** — o teto de tempo dela e o piso do amortecimento, juntos, são o que
/// impede um traço de crescer depois de o artista o ter terminado.
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
