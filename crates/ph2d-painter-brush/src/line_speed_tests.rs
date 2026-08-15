//! Os gates do **Speed Shapes** — o `LineKind::Speed` do plano 38 W2.
//!
//! O que a feature promete (manual do Alchemy, verbatim): *"Accentuates the pen speed to create
//! shapes that throw the line beyond the actual pen position."* Os gates abaixo perguntam pelo
//! ARREMESSO, não pela fórmula.
//!
//! ⚠️ **Não há `Amount`** (Enio 2026-08-13: *"em alchemy o slider não é necessário"*): a antecipação
//! é UMA constante de tempo ([`SPEED_LOOKAHEAD_S`]), então o produto tem de acertar de fábrica e
//! estes gates são o que a mantêm honesta.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::{LineKind, SPEED_LOOKAHEAD_S};
use crate::spec::BrushSpec;
use crate::stroke::{Dab, Stroke, StrokePoint};

fn spec(spacing: f32, kind: LineKind) -> BrushSpec {
    BrushSpec {
        radius_px: 10.0,
        spacing,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: kind,
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
/// ponteiro por quadro, e devolve `(dabs emitidos, última posição do ponteiro)`.
///
/// ⚠️ **O tique fecha cada quadro**, que é onde a velocidade é medida — sem ele o `speed_px_s` fica
/// em zero e a fixture não contém o fenômeno. `ticking = false` é o CONTROLE: um relógio que nunca
/// anda.
fn run_raw(
    sp: BrushSpec,
    frames: usize,
    per_frame: usize,
    total: f32,
    dt: f32,
    ticking: bool,
) -> (Vec<Dab>, f32) {
    let mut s = Stroke::new(sp, plain_dynamics(), 1);
    let mut out = Vec::new();
    let mut all: Vec<Dab> = Vec::new();
    let mut x = 0.0f32;
    s.begin(p(0.0), &mut out);
    all.extend(out.iter().copied());
    #[allow(clippy::cast_precision_loss)]
    let step = total / (frames * per_frame) as f32;
    for _ in 0..frames {
        for _ in 0..per_frame {
            x += step;
            s.extend(p(x), &mut out);
            all.extend(out.iter().copied());
        }
        if ticking {
            s.tick(dt, &mut out);
            all.extend(out.iter().copied());
        }
    }
    (all, x)
}

/// O caso comum: relógio andando, devolve `(último centro, ponta do ponteiro)`.
fn run(sp: BrushSpec, frames: usize, per_frame: usize, total: f32, dt: f32) -> ([f32; 2], f32) {
    let (dabs, tip) = run_raw(sp, frames, per_frame, total, dt, true);
    (dabs.last().map_or([0.0, 0.0], |d| d.center), tip)
}

/// **A TINTA É ARREMESSADA À FRENTE DO DEDO** — a frase do manual, medível: com `Speed` a última
/// gota cai ADIANTE da última posição do ponteiro, e sem ele cai onde o dedo estava.
///
/// A lei é `arremesso = velocidade × antecipação`: 400 px em 8 quadros de 60 fps são **3 000 px/s**,
/// e a `SPEED_LOOKAHEAD_S` de um sexto de segundo pede **~500 px**. A banda é larga por baixo porque
/// a RAMPA (que mantém a linha contínua) faz a velocidade usada por um dab perseguir a medida.
#[test]
fn the_ink_is_thrown_ahead_of_the_finger() {
    let dt = 1.0 / 60.0;
    let (plain, tip) = run(spec(0.1, LineKind::None), 8, 4, 400.0, dt);
    let (thrown, tip2) = run(spec(0.1, LineKind::Speed), 8, 4, 400.0, dt);
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
    let throw = thrown[0] - plain[0];
    let want = 3000.0 * SPEED_LOOKAHEAD_S;
    assert!(
        (0.7 * want..1.15 * want).contains(&throw),
        "o arremesso deveria valer ~{want:.0} px (velocidade × antecipação), mediu {throw:.1}"
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
    let (fine, tip) = run(spec(0.05, LineKind::Speed), 8, 4, 400.0, dt);
    let (coarse, _) = run(spec(0.8, LineKind::Speed), 8, 4, 400.0, dt);
    let (a, b) = (fine[0] - tip, coarse[0] - tip);
    // ⚠️ **A NÃO-VACUIDADE primeiro:** uma invariância é verdadeira de graça quando a grandeza é
    // ZERO, e foi assim que este gate ficou VERDE sob a mutação que mata a medição de velocidade.
    // Sem esta linha ele afirma *"dois arremessos ausentes são iguais"*.
    assert!(
        a > 100.0,
        "a fixture não contém arremesso nenhum: {a:.1} px"
    );
    assert!(
        (a - b).abs() < 20.0,
        "o arremesso mudou com o espaçamento: {a:.1} px contra {b:.1} px"
    );
}

/// **E ELE TAMBÉM NÃO É FATO DA TAXA DE EVENTOS** — o mesmo caminho no mesmo tempo, entregue em 4 e
/// em 64 eventos por quadro, arremessa igual.
#[test]
fn the_throw_does_not_move_with_the_pointer_polling_rate() {
    let dt = 1.0 / 60.0;
    let (few, tip) = run(spec(0.1, LineKind::Speed), 8, 4, 400.0, dt);
    let (many, _) = run(spec(0.1, LineKind::Speed), 8, 64, 400.0, dt);
    let (a, b) = (few[0] - tip, many[0] - tip);
    assert!(
        a > 100.0,
        "a fixture não contém arremesso nenhum: {a:.1} px"
    );
    assert!(
        (a - b).abs() < 6.0,
        "o arremesso seguiu o DISPOSITIVO: {a:.1} px com 4 eventos/quadro contra {b:.1} com 64"
    );
}

/// **O TIPO NEUTRO IGNORA O RELÓGIO INTEIRO** — com `None`, o mesmo gesto com e sem tique produz a
/// MESMA lista de dabs, ao bit: nem o arremesso, nem a rampa, nem o preenchimento alcançam um dab.
///
/// ⚠️ É a rede que torna a wave reversível, e o **CONTROLE ao lado é o que a torna não-vácua**: sob
/// `Speed` as duas listas TÊM de divergir, senão este gate estaria afirmando que o relógio nunca
/// importa para ninguém.
#[test]
fn the_neutral_type_ignores_the_clock_entirely() {
    let dt = 1.0 / 60.0;
    let (ticked, _) = run_raw(spec(0.1, LineKind::None), 8, 4, 400.0, dt, true);
    let (still, _) = run_raw(spec(0.1, LineKind::None), 8, 4, 400.0, dt, false);
    assert_eq!(ticked, still, "o tipo None honrou o relógio");
    let (s_ticked, _) = run_raw(spec(0.1, LineKind::Speed), 8, 4, 400.0, dt, true);
    let (s_still, _) = run_raw(spec(0.1, LineKind::Speed), 8, 4, 400.0, dt, false);
    assert_ne!(
        s_ticked, s_still,
        "controle: sob Speed o relógio TEM de mudar a tinta"
    );
}

/// **SÓ O `Speed` ARREMESSA** — escolher um tipo que costura fios não pode jogar a tinta além do
/// dedo.
///
/// ⚠️ **Este gate nasceu VERMELHO sobre o produto que shipava**, e o número é a razão de ele existir:
/// a guarda do arremesso era *"o tipo não é o neutro"*, correta enquanto o `Speed` era o único tipo,
/// e a W3/W4 a herdaram em silêncio. Medido nesta mesma fixture, num traço em que a mão vai a
/// `x = 400`: `Sketchy` e `Wire` punham a última gota **onde o `Speed` a punha**, ~180 px adiante.
///
/// ⚠️ **A fixture TEM de TICAR, e é por isso que nenhum dos vinte gates das W3/W4 viu isto:** o
/// arremesso é `velocidade × antecipação`, a velocidade só existe depois de um tique, e as fixtures
/// daquelas waves nunca chamam [`Stroke::tick`]. O produto tica todo quadro.
///
/// ⚠️ **O CONTROLE é o `Speed` na mesma corrida** — sem ele, o dia em que o arremesso morrer por
/// inteiro deixa este gate **verde**, afirmando que três tipos que não arremessam não arremessam.
#[test]
fn only_the_speed_type_throws_the_ink() {
    let dt = 1.0 / 60.0;
    let (plain, tip) = run(spec(0.1, LineKind::None), 8, 4, 400.0, dt);
    let (thrown, _) = run(spec(0.1, LineKind::Speed), 8, 4, 400.0, dt);
    assert!(
        thrown[0] - plain[0] > 100.0,
        "controle: a fixture não contém arremesso nenhum ({:.1} px)",
        thrown[0] - plain[0]
    );
    for kind in [LineKind::Sketchy, LineKind::Wire, LineKind::Ribbon] {
        let (ink, _) = run(spec(0.1, kind), 8, 4, 400.0, dt);
        let delta = ink[0] - plain[0];
        // O SUJEITO deste gate é o arremesso, e um arremesso põe a tinta ADIANTE.
        assert!(
            delta < 1.0,
            "{kind:?} arremessou a tinta: {delta:.1} px ADIANTE do neutro (o dedo parou em {tip:.1})"
        );
        // ⚠️ **O outro lado só vale para quem não tem modelo de atraso, e a 1ª versão media
        // `.abs()`** — o que conta ATRASO como se fosse arremesso. A fita mede **−314 px** nesta
        // fixture, e isso é literalmente a feature dela (com gate próprio em `ribbon_tests`);
        // colapsar os dois sinais num módulo faz este gate reprovar o produto correto.
        if kind != LineKind::Ribbon {
            assert!(
                delta > -1.0,
                "{kind:?} não tem modelo de atraso e ficou {delta:.1} px atrás do neutro"
            );
        }
    }
}

/// **A VELOCIDADE É MEDIDA EM TODO PINCEL, NÃO SÓ NO AIRBRUSH** — o tique mede o gesto ANTES de
/// perguntar qual é o método de traço.
///
/// ⚠️ Sem esta ordem o `Speed` nasceria morto em todo pincel que não fosse o Airbrush, e os gates de
/// unidade da fórmula ficariam **todos verdes** sobre isso — é este gate, e só ele, que a pega.
#[test]
fn the_speed_is_measured_before_the_airbrush_branch() {
    let mut s = Stroke::new(spec(0.1, LineKind::Speed), plain_dynamics(), 1);
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

/// **A ANTECIPAÇÃO É UM TEMPO, e é por isso que ela AUTO-ESCALA** — dobrar a velocidade do gesto
/// dobra o arremesso, sem ninguém tocar num controle.
///
/// ⚠️ Este gate substituiu o que pinava o valor da constante contra ela mesma: um teste que lê
/// `SPEED_LOOKAHEAD_S` e afirma que ela vale `SPEED_LOOKAHEAD_S` **não pode falhar**. A propriedade
/// que importa é a PROPORCIONALIDADE, e ela é observável na tinta.
///
/// **Mutação que sangra:** trocar a antecipação por um comprimento fixo em pixels.
#[test]
fn the_look_ahead_is_a_time_so_a_faster_gesture_throws_further() {
    let dt = 1.0 / 60.0;
    // O MESMO caminho percorrido a duas velocidades numa razão de QUATRO (mais quadros ⇒ mais
    // devagar). ⚠️ **A razão larga é parte da fixture:** com duas velocidades vizinhas, um
    // comprimento de rampa fixo em pixels acerta as duas por acidente e a mutação sobrevive.
    let (slow, tip_s) = run(spec(0.1, LineKind::Speed), 48, 4, 800.0, dt);
    let (fast, tip_f) = run(spec(0.1, LineKind::Speed), 12, 4, 800.0, dt);
    assert!((tip_s - tip_f).abs() < 1e-3, "controle: o mesmo caminho");
    let (a, b) = (slow[0] - tip_s, fast[0] - tip_f);
    assert!(a > 10.0, "a fixture não contém arremesso nenhum: {a:.1} px");
    assert!(
        (b - 4.0 * a).abs() < 0.25 * b,
        "quadruplicar a velocidade tem de quadruplicar o arremesso: {a:.1} → {b:.1}"
    );
}

/// **A LINHA CONTINUA UMA LINHA** — o defeito que o Enio reprovou DUAS vezes (2026-08-13: *"speed
/// não é igual o Alchemy"*, depois *"ainda fica pontilhado"*, com a foto), com o número ao lado.
///
/// O arremesso **estica o caminho da tinta**: o motor emite um dab a cada `spacing × diâmetro` do
/// caminho da MÃO, e entre dois dabs a tinta anda `passo × (1 + Δv/v)`. Medido no gesto de um
/// artista (sonda `measure_the_gap_on_a_gesture_that_changes_speed`, pincel de raio 4), o maior vão
/// entre dabs vizinhos valia **1,61 diâmetro** com a contagem de dabs **idêntica** à do traço sem
/// arremesso — os mesmos dabs espalhados por um caminho mais longo. A cura é percorrer o caminho
/// ARREMESSADO (`super::stroke::speed::fill_thrown_gap`).
///
/// ⚠️ **O oráculo é o DIÂMETRO, não o passo:** o que faz uma linha ser sólida é os dabs se
/// sobreporem, e um arremesso que estica o caminho **aumenta o passo de propósito** (é a feature).
/// O gate pergunta o que o olho pergunta — *dá para ver buraco?*
///
/// **Mutações que sangram:** desligar o preenchimento · medir o vão no caminho da MÃO em vez do da
/// tinta · usar o passo NOMINAL em vez do diâmetro que de fato é carimbado.
#[test]
fn the_thrown_line_stays_solid_it_does_not_break_into_beads() {
    let dt = 1.0 / 60.0;
    // ⚠️ **A fixture varre o PINCEL FINO**, que é onde o defeito foi reportado: o vão é medido em
    // diâmetros, então um pincel grosso o esconde. Raio 4 é a espessura do desenho do Enio.
    for radius in [4.0f32, 10.0, 25.0] {
        let mut sp = spec(0.1, LineKind::Speed);
        sp.radius_px = radius;
        let mut s = Stroke::new(sp, plain_dynamics(), 1);
        let mut out = Vec::new();
        let mut centres: Vec<[f32; 2]> = Vec::new();
        let mut x = 0.0f32;
        s.begin(p(x), &mut out);
        centres.extend(out.iter().map(|d| d.center));
        // Um gesto que ACELERA E FREIA — é a MUDANÇA de velocidade que estica o caminho, então uma
        // fixture de velocidade constante não conteria o fenômeno.
        for f in 0..24u32 {
            let ph = f32::from(u8::try_from(f % 8).unwrap_or(0)) / 8.0;
            let u = if ph < 0.5 { ph * 2.0 } else { (1.0 - ph) * 2.0 };
            let per = (200.0 + 2800.0 * u) / 60.0 / 4.0;
            for _ in 0..4 {
                x += per;
                s.extend(p(x), &mut out);
                centres.extend(out.iter().map(|d| d.center));
            }
            s.tick(dt, &mut out);
            centres.extend(out.iter().map(|d| d.center));
        }
        let mut worst = 0.0f32;
        let mut ink = 0.0f32;
        for w in centres.windows(2) {
            let d = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
            worst = worst.max(d);
            ink += d;
        }
        // ⚠️ **Não-vacuidade, e ela mede o que o defeito É:** o arremesso ESTICA o caminho da tinta,
        // então a fixture só contém o fenômeno se a tinta percorrer mais que a mão. Sem esta linha
        // um traço sem arremesso nenhum passa o gate de graça.
        assert!(
            ink > 1.3 * x,
            "a fixture não estica o caminho: a tinta anda {ink:.0} px contra {x:.0} da mão"
        );
        let diameter = 2.0 * radius;
        assert!(
            worst < diameter,
            "a linha se partiu em contas no raio {radius}: maior vão {worst:.1} px contra um diâmetro de {diameter}"
        );
    }
}

/// **A TINTA É DEPOSITADA NO ESPAÇAMENTO, AO LONGO DO CAMINHO QUE A TINTA PERCORRE** — a lei da
/// wave enunciada como propriedade, e a que impede a cura de virar *"emita dabs até parar de doer"*.
///
/// O vão MÉDIO entre dabs vizinhos tem de valer um passo (`spacing × diâmetro`): abaixo disso o
/// motor está desperdiçando tinta e tempo em todo traço do app; acima, a linha tem buraco. É a
/// mesma promessa que o `Space` sempre fez para o caminho da MÃO, agora feita onde a tinta cai.
///
/// **Mutações que sangram:** preencher com metade do passo (o vão médio despenca) · não preencher
/// (ele dispara).
#[test]
fn the_ink_is_laid_at_the_spacing_along_the_path_the_ink_takes() {
    let dt = 1.0 / 60.0;
    let step = 0.1 * 2.0 * 10.0; // spacing × diâmetro do `spec()`
    let (dabs, _) = run_raw(spec(0.1, LineKind::Speed), 8, 4, 800.0, dt, true);
    let gaps: Vec<f32> = dabs
        .windows(2)
        .map(|w| (w[1].center[0] - w[0].center[0]).hypot(w[1].center[1] - w[0].center[1]))
        .collect();
    assert!(gaps.len() > 100, "a fixture emitiu tinta de menos");
    #[allow(clippy::cast_precision_loss)]
    let mean = gaps.iter().sum::<f32>() / gaps.len() as f32;
    assert!(
        (0.5 * step..1.2 * step).contains(&mean),
        "o vão MÉDIO deveria valer um passo ({step:.1} px), mediu {mean:.2}"
    );
}
/// **E A LINHA CONTINUA LISA** — a outra metade do report do Enio (*"nosso traço não é idêntico na
/// FORMA"*, 2026-08-13), que o preenchimento sozinho não responde.
///
/// ⚠️ **São DUAS causas, e a primeira hipótese estava errada.** A velocidade é medida por TIQUE,
/// logo é uma ESCADA — mas suavizá-la (a EMA de [`super::stroke::speed`]) só levou a razão de 25,1
/// para 24,4×: *quase nada*. Olhando ONDE a virada estava, ela não era uma quina de quadro e sim uma
/// **ondulação de ±5 px numa reta**. O arremesso é uma **ALAVANCA**: ele desloca a tinta por `k` px
/// ao longo do heading, então o tremor residual de **0,6°** que o heading do Rake carrega — ele é
/// suavizado sobre 1,6 px, porque um carimbo tem de ACOMPANHAR o traço — vira ±5 px de ondulação
/// quando `k = 470`. A cura é a MIRA com janela própria, longa como o próprio arremesso.
///
/// ⚠️ **O oráculo é o CONTROLE, não um número escolhido:** a régua é o que a MÃO desenhou. A tinta
/// pode voar para fora do giro (é a feature), mas a curva que ela descreve não pode virar mais
/// bruscamente que a que o artista fez — e, sendo uma curva MAIOR, ela deve virar menos (medido:
/// **0,9× e 1,6×**). A medição é à escala de um **DIÂMETRO** de percurso: abaixo disso uma virada é
/// ruído sub-pixel entre dabs que se sobrepõem dez vezes, e o gate julgaria o que ninguém vê.
///
/// **Mutações que sangram** (razão no arco de 3 quadros, contra **0,9× do produto**): a mira ser o
/// heading CRU ⇒ **24,4×** · a mira usar a janela do RAKE ⇒ **3,9×** · a EMA da velocidade virar um
/// degrau ⇒ **3,4×**.
///
/// ⚠️ **E uma mutação SOBREVIVE, medida e documentada em vez de gateada:** trocar a janela da EMA de
/// velocidade (hoje o arco do próprio quadro) por uma constante de 40 px dá **0,9× e 0,9×** — ela
/// passa, e no arco lento é até melhor que o produto. O valor daquela janela **não é observável** nos
/// regimes medidos depois que a MIRA passou a fazer o trabalho pesado; o auto-escalar fica por ser a
/// escolha derivada (*alcançar em um quadro de percurso*) contra um número mágico em pixels.
#[test]
fn the_thrown_line_is_smooth_it_does_not_kink_once_per_frame() {
    let dt = 1.0 / 60.0;
    let mut sp = spec(0.1, LineKind::Speed);
    sp.radius_px = 8.0;
    // ⚠️ **DUAS velocidades, e é o que torna o gate honesto sobre a AUTO-ESCALA:** o mesmo arco em 3
    // e em 12 quadros (78 e 20 px de percurso por quadro). Um comprimento de rampa fixo em pixels
    // acerta UMA delas por acidente e erra a outra.
    for nf in [3usize, 12] {
        let (mut control, mut thrown) = (0.0f32, 0.0f32);
        for kind in [LineKind::None, LineKind::Speed] {
            sp.line_kind = kind;
            let mut s = Stroke::new(sp, plain_dynamics(), 1);
            let mut out = Vec::new();
            let mut c: Vec<[f32; 2]> = Vec::new();
            let r = 150.0f32;
            let pf = 8usize;
            let pt = |i: usize| {
                let t = i as f32 / (nf * pf) as f32;
                let (mut u, mut v) = ([1.0f32, 0.0f32], [0.0f32, 1.0f32]);
                let (mut lo, mut hi) = (0.0f32, 1.0f32);
                for _ in 0..20 {
                    let m = 0.5 * (lo + hi);
                    let mid = [u[0] + v[0], u[1] + v[1]];
                    let l = mid[0].hypot(mid[1]);
                    let mid = [mid[0] / l, mid[1] / l];
                    if t < m {
                        v = mid;
                        hi = m;
                    } else {
                        u = mid;
                        lo = m;
                    }
                }
                [u[0] * r, u[1] * r]
            };
            s.begin(
                StrokePoint {
                    pos: pt(0),
                    pressure: 1.0,
                },
                &mut out,
            );
            c.extend(out.iter().map(|d| d.center));
            for f in 0..nf {
                for k in 1..=pf {
                    s.extend(
                        StrokePoint {
                            pos: pt(f * pf + k),
                            pressure: 1.0,
                        },
                        &mut out,
                    );
                    c.extend(out.iter().map(|d| d.center));
                }
                s.tick(dt, &mut out);
                c.extend(out.iter().map(|d| d.center));
            }
            // Reamostra o caminho da TINTA a cada DIÂMETRO: é a escala em que uma quina deixa de ser
            // ruído sub-pixel e vira um canto que o olho vê num traço desta espessura.
            let diam = 16.0f32;
            let mut marks: Vec<[f32; 2]> = vec![c[0]];
            let mut acc = 0.0f32;
            for w in c.windows(2) {
                acc += (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
                if acc >= diam {
                    marks.push(w[1]);
                    acc = 0.0;
                }
            }
            let mut worst = 0.0f32;
            for w in marks.windows(3) {
                let a = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
                let b = [w[2][0] - w[1][0], w[2][1] - w[1][1]];
                let (la, lb) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
                if la < 1e-4 || lb < 1e-4 {
                    continue;
                }
                let cos = ((a[0] * b[0] + a[1] * b[1]) / (la * lb)).clamp(-1.0, 1.0);
                worst = worst.max(1.0 - cos);
            }
            if kind == LineKind::None {
                control = worst;
            } else {
                thrown = worst;
            }
        }
        assert!(
            control > 1e-4,
            "controle: a fixture tem de ter curvatura ({control:.6}) em {nf} quadros"
        );
        assert!(
            thrown < 3.0 * control,
            "a tinta virou mais bruscamente que a mão em {nf} quadros: {thrown:.6} contra {control:.6} do controle"
        );
    }
}
