//! Os gates do **Speed Shapes** — o `LineKind::Speed` do plano 38 W2.
//!
//! O que a feature promete (manual do Alchemy, verbatim): *"Accentuates the pen speed to create
//! shapes that throw the line beyond the actual pen position."* Os gates abaixo perguntam pelo
//! ARREMESSO, não pela fórmula.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::{LineKind, MAX_SPEED_AMOUNT, SPEED_LOOKAHEAD_S};
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};

fn spec(spacing: f32, kind: LineKind, amount: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 10.0,
        spacing,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: kind,
        line_speed_amount: amount,
        ..Default::default()
    }
}

fn plain_dynamics() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

fn p(x: f32) -> StrokePoint {
    StrokePoint {
        pos: [x, 0.0],
        pressure: 1.0,
    }
}

/// Percorre uma reta de `total` px em `frames` quadros de `dt`, entregando `per_frame` eventos de
/// ponteiro por quadro, e devolve `(último centro de dab, última posição do ponteiro)`.
///
/// ⚠️ **O tique fecha cada quadro**, que é onde a velocidade é medida — sem ele o `speed_px_s` fica
/// em zero e a fixture não contém o fenômeno.
fn run(sp: BrushSpec, frames: usize, per_frame: usize, total: f32, dt: f32) -> ([f32; 2], f32) {
    let mut s = Stroke::new(sp, plain_dynamics(), 1);
    let mut out = Vec::new();
    let mut last = [0.0f32, 0.0];
    let mut x = 0.0f32;
    s.begin(p(0.0), &mut out);
    if let Some(d) = out.last() {
        last = d.center;
    }
    #[allow(clippy::cast_precision_loss)]
    let step = total / (frames * per_frame) as f32;
    for _ in 0..frames {
        for _ in 0..per_frame {
            x += step;
            s.extend(p(x), &mut out);
            if let Some(d) = out.last() {
                last = d.center;
            }
        }
        s.tick(dt, &mut out);
        if let Some(d) = out.last() {
            last = d.center;
        }
    }
    (last, x)
}

/// **A TINTA É ARREMESSADA À FRENTE DO DEDO** — a frase do manual, medível: com `Speed` a última
/// gota cai ADIANTE da última posição do ponteiro, e sem ele cai onde o dedo estava.
#[test]
fn the_ink_is_thrown_ahead_of_the_finger() {
    let dt = 1.0 / 60.0;
    let (plain, tip) = run(spec(0.1, LineKind::None, 0.0), 8, 4, 400.0, dt);
    let (thrown, tip2) = run(spec(0.1, LineKind::Speed, 4.0), 8, 4, 400.0, dt);
    assert!(
        (tip - tip2).abs() < 1e-3,
        "controle: os dois gestos percorrem o MESMO caminho ({tip} contra {tip2})"
    );
    // Sem tipo, a tinta acompanha o dedo (a folga é o espaçamento de um dab).
    assert!(
        (plain[0] - tip).abs() < 5.0,
        "sem Speed a tinta segue o dedo: dab em {} contra ponteiro em {tip}",
        plain[0]
    );
    // 400 px em 8 quadros = 50 px/quadro; `Amount = 4` arremessa 4 quadros ⇒ ~200 px.
    let throw = thrown[0] - plain[0];
    assert!(
        (150.0..260.0).contains(&throw),
        "o arremesso deveria valer ~4 quadros de percurso (~200 px), mediu {throw}"
    );
}

/// **O ARREMESSO É FATO DO CAMINHO E DO RELÓGIO, NUNCA DO ESPAÇAMENTO** — o mesmo gesto, o mesmo
/// relógio e dois espaçamentos de dab dão o MESMO arremesso.
///
/// ⚠️ É a lei que este módulo aprendeu quatro vezes no relevo, aplicada à velocidade: se ela fosse
/// medida por EVENTO ou por DAB, este gate sangraria — a W0.1 mediu 73× de variação no primeiro e
/// zero informação no segundo.
#[test]
fn the_throw_is_a_fact_of_the_path_and_the_clock_not_of_the_dab_spacing() {
    let dt = 1.0 / 60.0;
    let (fine, tip) = run(spec(0.05, LineKind::Speed, 4.0), 8, 4, 400.0, dt);
    let (coarse, _) = run(spec(0.8, LineKind::Speed, 4.0), 8, 4, 400.0, dt);
    // Os dois carimbam em grades diferentes, então o dab final não cai no mesmo lugar; o que TEM de
    // bater é o quanto ele passou do dedo, e a tolerância é um passo de dab do mais grosso (16 px).
    let (a, b) = (fine[0] - tip, coarse[0] - tip);
    // ⚠️ **A NÃO-VACUIDADE primeiro:** uma invariância é verdadeira de graça quando a grandeza é
    // ZERO, e foi assim que este gate ficou VERDE sob a mutação que mata a medição de velocidade.
    // Sem esta linha ele afirma *"dois arremessos ausentes são iguais"*.
    assert!(
        a > 100.0,
        "a fixture não contém arremesso nenhum: {a:.1} px"
    );
    assert!(
        (a - b).abs() < 17.0,
        "o arremesso mudou com o espaçamento: {a:.1} px contra {b:.1} px"
    );
}

/// **E ELE TAMBÉM NÃO É FATO DA TAXA DE EVENTOS** — o mesmo caminho no mesmo tempo, entregue em 4 e
/// em 64 eventos por quadro, arremessa igual.
#[test]
fn the_throw_does_not_move_with_the_pointer_polling_rate() {
    let dt = 1.0 / 60.0;
    let (few, tip) = run(spec(0.1, LineKind::Speed, 4.0), 8, 4, 400.0, dt);
    let (many, _) = run(spec(0.1, LineKind::Speed, 4.0), 8, 64, 400.0, dt);
    let (a, b) = (few[0] - tip, many[0] - tip);
    assert!(
        a > 100.0,
        "a fixture não contém arremesso nenhum: {a:.1} px"
    );
    assert!(
        (a - b).abs() < 5.0,
        "o arremesso seguiu o DISPOSITIVO: {a:.1} px com 4 eventos/quadro contra {b:.1} com 64"
    );
}

/// **`Amount = 0` É BYTE-IDÊNTICO A `None`** — a rede que torna a wave reversível, e a que prova que
/// escolher o tipo sem mexer no slider não move um pixel.
#[test]
fn amount_zero_is_byte_identical_to_the_neutral_type() {
    let dt = 1.0 / 60.0;
    let (none, _) = run(spec(0.1, LineKind::None, 0.0), 8, 4, 400.0, dt);
    let (zero, _) = run(spec(0.1, LineKind::Speed, 0.0), 8, 4, 400.0, dt);
    assert_eq!(none, zero, "Amount 0 moveu a tinta");
    // E o outro lado: um `Amount` alto com o tipo NEUTRO também não move nada.
    let (armed_but_none, _) = run(spec(0.1, LineKind::None, MAX_SPEED_AMOUNT), 8, 4, 400.0, dt);
    assert_eq!(none, armed_but_none, "o tipo None honrou um Amount");
}

/// **A VELOCIDADE É MEDIDA EM TODO PINCEL, NÃO SÓ NO AIRBRUSH** — o tique mede o gesto ANTES de
/// perguntar qual é o método de traço.
///
/// ⚠️ Sem esta ordem o `Speed` nasceria morto em todo pincel que não fosse o Airbrush, e os gates de
/// unidade da fórmula ficariam **todos verdes** sobre isso — é este gate, e só ele, que a pega.
#[test]
fn the_speed_is_measured_before_the_airbrush_branch() {
    let mut s = Stroke::new(spec(0.1, LineKind::Speed, 1.0), plain_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(p(0.0), &mut out);
    assert_eq!(s.speed_px_s(), 0.0, "traço novo: a mão ainda não andou");
    for k in 1..=8 {
        #[allow(clippy::cast_precision_loss)]
        s.extend(p(k as f32 * 6.0), &mut out);
    }
    s.tick(1.0 / 60.0, &mut out);
    // 48 px num quadro de 60 fps = 2880 px/s.
    let v = s.speed_px_s();
    assert!(
        (2700.0..3000.0).contains(&v),
        "o tique de um pincel COMUM tem de medir a velocidade: {v} px/s"
    );
}

/// **A JANELA DE ANTECIPAÇÃO É UM TEMPO** — `Amount` quadros a `SPEED_LOOKAHEAD_S` cada; dobrar o
/// `Amount` dobra o arremesso.
#[test]
fn the_amount_counts_frames_of_look_ahead() {
    let dt = 1.0 / 60.0;
    let (a1, tip) = run(spec(0.1, LineKind::Speed, 2.0), 8, 4, 400.0, dt);
    let (a2, _) = run(spec(0.1, LineKind::Speed, 4.0), 8, 4, 400.0, dt);
    let (t1, t2) = (a1[0] - tip, a2[0] - tip);
    assert!(
        t1 > 50.0,
        "a fixture não contém arremesso nenhum: {t1:.1} px"
    );
    // O arremesso é proporcional; a folga é um passo de dab (2 px neste espaçamento).
    assert!(
        (t2 - 2.0 * t1).abs() < 6.0,
        "dobrar o Amount tem de dobrar o arremesso: {t1:.1} → {t2:.1}"
    );
    // E a constante é o que o doc diz que é: um quadro de 60 fps.
    assert!(
        (SPEED_LOOKAHEAD_S - 1.0 / 60.0).abs() < 1e-6,
        "a janela deixou de ser um quadro de 60 fps"
    );
}

/// **A LINHA CONTINUA UMA LINHA** — o defeito que o Enio reprovou em 2026-08-13 (*"speed não é igual
/// o Alchemy"*), com o número ao lado.
///
/// O arremesso medido por TIQUE é uma ESCADA: um valor por quadro, constante dentro dele, saltando na
/// fronteira ⇒ a tinta saía como uma fileira de arcos deslocados e **desconectados**, um por quadro.
/// Medido num arco rápido, o maior vão entre dabs vizinhos valia **25× o passo nominal em `Amount=1`
/// e 99× em `Amount=4`**. O Alchemy arremessa por ponto gravado; a cura aqui é a RAMPA sobre o arco
/// do próprio quadro (`super::stroke::speed`).
///
/// ⚠️ **O oráculo é o DIÂMETRO, não o passo:** o que faz uma linha ser sólida é os dabs se
/// sobreporem, e um arremesso que estica o caminho **aumenta o passo de propósito** (é a feature).
/// O gate pergunta o que o olho pergunta — *dá para ver buraco?* —, e a resposta é `vão < diâmetro`.
///
/// **Mutação que sangra:** trocar a rampa por um degrau (o `throw` lendo a medida do tique direto).
#[test]
fn the_thrown_line_stays_solid_it_does_not_break_into_shifted_arcs() {
    let dt = 1.0 / 60.0;
    let diameter = 2.0 * 10.0; // o raio do `spec()` acima
    for amount in [1.0f32, 4.0, MAX_SPEED_AMOUNT] {
        let mut s = Stroke::new(spec(0.1, LineKind::Speed, amount), plain_dynamics(), 1);
        let mut out = Vec::new();
        let mut centres: Vec<[f32; 2]> = Vec::new();
        // Um gesto que ACELERA: cada quadro percorre mais que o anterior, então a escada tem degraus
        // grandes — sem ela na fixture o gate não conteria o fenômeno.
        let mut x = 0.0f32;
        s.begin(p(x), &mut out);
        centres.extend(out.iter().map(|d| d.center));
        for f in 1..=8 {
            #[allow(clippy::cast_precision_loss)]
            let per = 6.0 * f as f32; // 6, 12, 18 … px por evento
            for _ in 0..4 {
                x += per;
                s.extend(p(x), &mut out);
                centres.extend(out.iter().map(|d| d.center));
            }
            s.tick(dt, &mut out);
            centres.extend(out.iter().map(|d| d.center));
        }
        let mut worst = 0.0f32;
        for w in centres.windows(2) {
            worst = worst.max((w[1][0] - w[0][0]).abs().max((w[1][1] - w[0][1]).abs()));
        }
        // Não-vacuidade: sem arremesso nenhum um traço qualquer passa este gate de graça.
        let span = centres.last().unwrap()[0] - centres[0][0];
        assert!(
            span > 700.0,
            "a fixture não contém arremesso: o traço mede {span:.0} px contra os ~672 do gesto"
        );
        assert!(
            worst < diameter,
            "a linha se partiu em Amount={amount}: maior vão {worst:.1} px contra um diâmetro de {diameter}"
        );
    }
}
