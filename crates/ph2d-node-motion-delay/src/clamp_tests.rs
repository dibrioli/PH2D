//! **O TETO DA TAXA** — os gates de `max_step` e `max_accel` (folha 03, linha 67).
//!
//! O que estes gates têm de separar não é *"o número faz alguma coisa"*: é que
//! ele é um **teto** (a saída nunca o passa), que a régua é a **magnitude** do
//! elemento e não a componente, e que **desligado o nó é o de antes, ao BIT**.

use crate::tests::{Ops, Src, rig, run, run_any};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;

/// A maior mudança entre dois tiques consecutivos de uma trajetória escalar.
fn worst_step(v: &[f32]) -> f32 {
    v.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max)
}

/// Um degrau: o alvo salta de 0 para 10 no tique 3 e fica lá.
fn step_path(n: usize) -> Vec<f32> {
    (0..n).map(|k| if k < 3 { 0.0 } else { 10.0 }).collect()
}

/// Roda a cadeia com um alvo **2-D** e devolve o `P` emitido a cada tique — o
/// `run` do irmão só devolve o `y`, e a régua da magnitude precisa dos dois.
fn run_xy(params: &[(&str, f32)], path: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let (g, _src, dly) = rig(params);
    let ops = Ops(Src {
        pos: std::sync::Mutex::new(vec![path[0]]),
        ids: std::sync::Mutex::new(None),
    });
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for (t, p) in path.iter().enumerate() {
        *ops.0.pos.lock().expect("test") = vec![*p];
        let cooked = cook.cook(&g, &ops, dly, t as f64).expect("coze");
        match cooked[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => out.push(v[0]),
            _ => panic!("o delay emite P"),
        }
        cook.advance_tick(&g, &ops, t as f64).expect("tique");
    }
    out
}

/// **O TETO DE PASSO É UM TETO.**
///
/// O CONTROLE é a primeira metade: sem ele *"a saída anda pouco"* seria
/// satisfeito por um lag mais lento, que é o que o `ticks` já faz. A pergunta é
/// se existe um número que a saída **não passa**.
#[test]
fn the_step_ceiling_is_a_ceiling_and_the_lag_alone_is_not() {
    let path = step_path(40);
    let free = run(&[("mode", 2.0), ("ticks", 4.0), ("channel", 1.0)], &path);
    let capped = run(
        &[
            ("mode", 2.0),
            ("ticks", 4.0),
            ("channel", 1.0),
            ("max_step", 0.25),
        ],
        &path,
    );
    let w_free = worst_step(&free);
    let w_cap = worst_step(&capped);
    assert!(
        w_free > 1.0,
        "o controle TEM de dar um passo grande, senão a cena não contém o \
         fenômeno; maior passo {w_free:.4}"
    );
    assert!(
        w_cap <= 0.25 + 1e-5,
        "nenhum passo pode passar do teto de 0,25; o maior mede {w_cap:.4}"
    );
    // E ele ainda CHEGA — um teto que travasse a saída seria outro defeito.
    assert!(
        *capped.last().expect("tiques") > 5.0,
        "a saída limitada tem de continuar a subir; parou em {:.4}",
        capped.last().expect("tiques")
    );
}

/// **O TETO DE ACELERAÇÃO É O ARRANQUE, NÃO O TETO.**
///
/// Com ele o primeiro passo é pequeno e os seguintes crescem; sem ele o primeiro
/// já é o maior. É a diferença entre *partir devagar* e *ir devagar*.
#[test]
fn the_accel_ceiling_ramps_the_start_instead_of_capping_the_whole_move() {
    let path = step_path(40);
    let plain = run(&[("mode", 2.0), ("ticks", 4.0), ("channel", 1.0)], &path);
    let ramped = run(
        &[
            ("mode", 2.0),
            ("ticks", 4.0),
            ("channel", 1.0),
            ("max_accel", 0.1),
        ],
        &path,
    );
    let first_plain = (plain[3] - plain[2]).abs();
    let first_ramp = (ramped[3] - ramped[2]).abs();
    assert!(
        first_ramp < first_plain * 0.25,
        "o arranque limitado tem de ser MUITO menor: {first_ramp:.4} contra \
         {first_plain:.4}"
    );
    // ...e a velocidade tem de CRESCER, senão isto é um teto de passo disfarçado.
    let later = (ramped[9] - ramped[8]).abs();
    assert!(
        later > first_ramp * 2.0,
        "a aceleração limitada ainda acelera: passo {later:.4} no tique 9 contra \
         {first_ramp:.4} no arranque"
    );
}

/// **O PASSO É INVIOLÁVEL, E É A ORDEM QUE O GARANTE — mas só onde ela é
/// OBSERVÁVEL, e a medição corrigiu a minha primeira afirmação.**
///
/// ⚠️ Com a velocidade a partir de ZERO as duas ordens respeitam o teto **por
/// indução**: o alvo do passo já cabe nele, e chegar mais perto de um alvo que
/// cabe nunca sai. A ordem só é observável quando `|prev_vel|` **passa** do teto
/// — e isso acontece por um gesto REAL: o artista **APERTA** o slider com o
/// canal já em movimento. É esse o mundo que este gate monta.
///
/// FALSIFICADO se a aceleração fosse aplicada por ÚLTIMO: partindo de uma
/// velocidade grande, ela reduz para `prev_vel ± max_accel`, que continua muito
/// acima do número que o artista acabou de digitar.
#[test]
fn the_step_ceiling_binds_immediately_even_when_it_is_tightened_mid_move() {
    let out = run_any(
        &[
            ("mode", 2.0),
            ("ticks", 2.0),
            ("channel", 1.0),
            ("max_accel", 0.05),
        ],
        24,
        |g, dly, t| {
            // Os doze primeiros tiques correm SEM teto de passo: a saída ganha
            // velocidade de verdade. No tique 12 o teto aperta.
            g.set_param(dly, "max_step", if t < 12 { 0.0 } else { 0.2 });
            // ⚠️ Um DEGRAU, não um alvo constante: um elemento recém-nascido
            // já nasce NO alvo, então uma fonte parada tem a saída convergida
            // desde o tique 0 e o controle mede zero.
            let y = if t < 3 { 0.0 } else { 100.0 };
            Stream::new(1).with("P", Column::Vec2(vec![[0.0, y]]))
        },
    );
    let y = |s: &Stream| match s.get("P") {
        Some(Column::Vec2(v)) => v[0][1],
        _ => panic!("P"),
    };
    let before = y(&out[11]) - y(&out[10]);
    // O CONTROLE mede a propriedade EXACTA de que a ordem depende: a
    // velocidade acumulada tem de PASSAR do teto que está prestes a ser
    // digitado. Um número maior seria arbitrário; um menor deixaria as duas
    // ordens concordarem por indução, e o gate não separaria nada.
    assert!(
        before > 0.2,
        "o CONTROLE: sem teto a saída TEM de estar a correr mais depressa que o \
         teto que vem aí, senão as duas ordens concordam; passo medido {before:.4}"
    );
    let worst = out
        .windows(2)
        .skip(12)
        .map(|w| (y(&w[1]) - y(&w[0])).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst <= 0.2 + 1e-5,
        "assim que o teto é digitado ele VALE, por mais depressa que a saída \
         estivesse; o maior passo depois do aperto mede {worst:.4}"
    );
}

/// **A RÉGUA É A MAGNITUDE, NÃO A COMPONENTE.**
///
/// O mesmo teto, o mesmo comprimento de percurso, duas direções: uma ao longo do
/// eixo e outra a 45°. As duas têm de andar o MESMO por tique. FALSIFICADO por um
/// clamp por componente, que deixa a diagonal ser `√2` vezes mais rápida — uma
/// taxa que depende da orientação dos eixos.
#[test]
fn the_ceiling_measures_the_elements_magnitude_not_its_components() {
    const D: f32 = 10.0;
    let axial: Vec<[f32; 2]> = (0..30)
        .map(|k| if k < 3 { [0.0, 0.0] } else { [D, 0.0] })
        .collect();
    let diag: Vec<[f32; 2]> = (0..30)
        .map(|k| {
            if k < 3 {
                [0.0, 0.0]
            } else {
                [
                    D * std::f32::consts::FRAC_1_SQRT_2,
                    D * std::f32::consts::FRAC_1_SQRT_2,
                ]
            }
        })
        .collect();
    let p = [
        ("mode", 2.0),
        ("ticks", 4.0),
        ("channel", 4.0),
        ("max_step", 0.4),
    ];
    let a = run_xy(&p, &axial);
    let d = run_xy(&p, &diag);
    let dist = |v: &[[f32; 2]]| -> f32 {
        v.windows(2)
            .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
            .fold(0.0, f32::max)
    };
    let (ma, md) = (dist(&a), dist(&d));
    assert!(
        ma <= 0.4 + 1e-5 && md <= 0.4 + 1e-5,
        "nenhuma das duas passa do teto: axial {ma:.4}, diagonal {md:.4}"
    );
    assert!(
        (ma - md).abs() < 1e-4,
        "o teto é ISOTRÓPICO: axial {ma:.4} contra diagonal {md:.4}"
    );
}

/// **DESLIGADO É O NÓ DE ANTES, AO BIT — E O ESTADO TAMBÉM.**
///
/// A segunda metade é a que um gate de trajetória não vê: com os tetos em zero o
/// stream de estado **não ganha a coluna da velocidade**, então um grafo salvo
/// antes destes params cozinha o mesmo objeto e não só a mesma imagem.
#[test]
fn with_both_ceilings_off_the_node_is_byte_identical_state_included() {
    let path = step_path(30);
    let before = run(&[("mode", 2.0), ("ticks", 5.0), ("channel", 1.0)], &path);
    let after = run(
        &[
            ("mode", 2.0),
            ("ticks", 5.0),
            ("channel", 1.0),
            ("max_step", 0.0),
            ("max_accel", 0.0),
        ],
        &path,
    );
    assert_eq!(before, after, "sem teto, os MESMOS bits");

    // E a coluna de estado não nasce.
    let (g, _src, dly) = rig(&[("mode", 2.0), ("ticks", 5.0), ("channel", 1.0)]);
    let ops = Ops(Src {
        pos: std::sync::Mutex::new(vec![[0.0, 1.0]]),
        ids: std::sync::Mutex::new(None),
    });
    let mut cook = Cook::new();
    let cooked = cook.cook(&g, &ops, dly, 0.0).expect("coze");
    assert!(
        cooked[0].as_stream().get("dl_vel").is_none(),
        "sem teto armado, o estado do teto não existe"
    );
}

/// **O `falloff` GATEIA O TETO, como gateia tudo o mais neste nó.**
///
/// Um elemento de peso zero é transparente — e um teto que ainda o limitasse
/// seria o nó a agir onde o artista disse que ele não age. O elemento livre ao
/// lado, mesmo teto, é o CONTROLE: sem ele *"o de peso zero está no alvo"* seria
/// satisfeito por um teto que não faz nada.
#[test]
fn a_zero_falloff_element_is_transparent_even_with_a_ceiling_armed() {
    let out = run_any(
        &[
            ("mode", 2.0),
            ("ticks", 6.0),
            ("channel", 1.0),
            ("max_step", 0.05),
        ],
        20,
        |_g, _dly, t| {
            let y = if t < 3 { 0.0 } else { 10.0 };
            Stream::new(2)
                .with("P", Column::Vec2(vec![[0.0, y], [0.0, y]]))
                .with("falloff", Column::Scalar(vec![0.0, 1.0]))
        },
    );
    let last = out.last().expect("tiques");
    let Some(Column::Vec2(v)) = last.get("P") else {
        panic!("P")
    };
    assert!(
        (v[0][1] - 10.0).abs() < 1e-5,
        "peso zero: transparente, no alvo cru; está em {:.4}",
        v[0][1]
    );
    assert!(
        v[1][1] < 2.0,
        "o vizinho de peso cheio TEM de estar preso pelo teto, senão o teto não \
         está armado; está em {:.4}",
        v[1][1]
    );
}

/// **UM RECÉM-NASCIDO ENTRA COM VELOCIDADE ZERO.**
///
/// A linha que chega para um elemento que o estado não conhece é a dele — mas a
/// VELOCIDADE não: herdá-la faria um elemento novo arrancar com o impulso de um
/// estranho, que é o oposto do que um teto de aceleração existe para fazer.
#[test]
fn a_newborn_starts_with_no_speed_of_its_own() {
    let state = Stream::new(0);
    let rows = vec![None, None];
    let v = crate::ring::prev_vel(&state, &rows, 4, 2);
    assert_eq!(v, vec![0.0; 4], "sem linha, sem velocidade");
}

/// **O TETO VALE SEM LAG NENHUM — `Rise = 0` E' UMA POSICAO DO SLIDER.**
///
/// ⚠️ *"So' quero um limite de velocidade, sem atraso"* e' o caso de uso mais
/// obvio dos dois params de taxa, e e' o que o *Max Slope* do Lag CHOP do
/// TouchDesigner entrega por desenho: ele limita o sinal haja ou nao haja lag.
/// Com o early-out antigo (`ticks <= 0` devolvia o valor vivo antes de o passe
/// do teto correr) os dois ficavam **pintados, registrados, vivos sob o rato e
/// INERTES** — o controle morto que nenhum gate de UI enxerga, porque a UI
/// estava toda certa.
///
/// ⚠️ **O CONTROLE e' a segunda metade e ele e' o que da' sentido a' primeira:**
/// sem teto, `ticks = 0` tem de seguir o alvo **AO BIT** — senao *"a saida anda
/// pouco"* teria sido satisfeito por o no' ter passado a atrasar sozinho.
#[test]
fn the_rate_ceiling_holds_with_no_lag_at_all() {
    let path = step_path(40);

    let free = run(&[("mode", 2.0), ("ticks", 0.0), ("channel", 1.0)], &path);
    let capped = run(
        &[
            ("mode", 2.0),
            ("ticks", 0.0),
            ("channel", 1.0),
            ("max_step", 0.25),
        ],
        &path,
    );

    // O CONTROLE: sem teto e sem lag o no' e' transparente, ao BIT.
    for (k, (a, b)) in free.iter().zip(path.iter()).enumerate() {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "sem teto e sem lag o no' e' transparente (tique {k}): {a} contra {b}"
        );
    }

    // E com o teto armado a saida NAO passa dele, mesmo com lag zero.
    let worst = worst_step(&capped);
    assert!(
        worst <= 0.25 + 1e-5,
        "o teto tem de valer com `ticks = 0`; o pior passo mede {worst:.4} contra 0,25"
    );
    // ...e o teto tem de estar de facto MORDENDO nesta fixture, senao a
    // assercao acima e' verde por vacuo sobre um degrau que ninguem limitou.
    assert!(
        worst_step(&free) > 0.25 * 4.0,
        "o CONTROLE tem de pedir mais taxa que o teto; ele mede {:.4}",
        worst_step(&free)
    );
}
