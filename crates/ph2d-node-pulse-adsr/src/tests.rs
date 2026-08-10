//! Os gates do `pulse.adsr` — o one-shot, a forma algébrica e o repouso.
//!
//! Vivem num arquivo IRMÃO e seguem sendo módulo **FILHO** (`#[path]`), porque o
//! que eles medem é privado: `step`, `bias` e `Envelope` nunca saem da crate, e um
//! teste de fora só alcançaria a porta do `NodeOp` — que é o produto, não a lei.

use super::*;

/// Um tique de 60 fps, o relógio da maioria das fixtures.
const DT: f32 = 1.0 / 60.0;

/// Um envelope legível a olho: cada trecho é um número redondo e o total é 0,5 s.
fn env() -> Envelope {
    Envelope {
        delay: 0.0,
        attack: 0.1,
        decay: 0.1,
        sustain: 0.5,
        hold: 0.1,
        release: 0.2,
        attack_shape: 0.5,
        release_shape: 0.5,
        retrigger: true,
    }
}

fn pulse(fired: bool) -> Stream {
    Stream::new(1).with(
        PULSE_COL,
        Column::Scalar(vec![if fired { 1.0 } else { 0.0 }]),
    )
}

fn level_of(s: &Stream) -> f32 {
    match s.get(VALUE_COL).unwrap() {
        Column::Scalar(v) => v[0],
        _ => panic!(),
    }
}

/// Dispara no tique `at` e devolve o nível de cada tique de `ticks` tiques, a `dt`.
fn run(e: Envelope, at: usize, ticks: usize, dt: f32) -> Vec<f32> {
    let mut state = Stream::new(1);
    let mut out = Vec::with_capacity(ticks);
    for k in 0..ticks {
        state = step(&pulse(k == at), &state, e, dt);
        out.push(level_of(&state));
    }
    out
}

/// **A curva passa pelos quatro trechos, na ordem e nos níveis autorados.** É a
/// afirmação do produto: um disparo instantâneo virou uma rampa que sobe a 1, cai
/// ao sustain, o segura e volta a zero.
#[test]
fn the_envelope_walks_attack_decay_sustain_release() {
    let e = env();
    // Amostrado direto da lei, que é função PURA da idade — o laço de tiques é
    // testado à parte, e misturar as duas coisas esconderia qual delas errou.
    assert_eq!(e.level(0.0), 0.0, "o disparo começa em zero");
    assert!(
        e.level(0.05) > 0.45 && e.level(0.05) < 0.55,
        "meio do ataque"
    );
    // ⚠️ Amostrar EM CIMA de uma fronteira pede tolerância, e o motivo é aritmético,
    // não folga: `0.3 − 0.1 − 0.1` em `f32` cai do lado de fora do `hold` por 1,4e-8,
    // então o valor lido é o primeiro instante do release. A curva é CONTÍNUA ali —
    // é por isso que o número certo continua sendo o mesmo a menos de um epsílon, e
    // pedir igualdade exata seria pedir que `f32` fosse associativo.
    let close = |a: f32, b: f32| assert!((a - b).abs() < 1e-5, "{a} != {b}");
    close(e.level(0.1), 1.0); // o pico é o fim do ataque
    close(e.level(0.2), 0.5); // o decay aterrissa NO sustain
    close(e.level(0.25), 0.5); // o patamar é plano
    close(e.level(0.3), 0.5); // o release parte do sustain
    close(e.level(0.4), 0.25); // meio do release
    assert_eq!(e.level(0.5), 0.0, "e acaba em zero");
    assert_eq!(e.level(9.0), 0.0, "e fica lá");
}

/// **O one-shot volta sozinho ao repouso, e o repouso é o mesmo par de zeros de um
/// grafo recém-montado.** Sem isso a idade cresceria para sempre e o estado
/// "parado" precisaria de um sentinela — que é exatamente o caso especial que a
/// representação apaga.
#[test]
fn it_returns_to_the_same_rest_a_fresh_graph_has() {
    let e = env();
    let mut state = Stream::new(1);
    // Um envelope inteiro (0,5 s) mais folga.
    for k in 0..45 {
        state = step(&pulse(k == 0), &state, e, DT);
    }
    let age = match state.get(AGE_COL).unwrap() {
        Column::Scalar(v) => v[0],
        _ => panic!(),
    };
    let on = match state.get(ON_COL).unwrap() {
        Column::Scalar(v) => v[0],
        _ => panic!(),
    };
    assert_eq!((age, on), (0.0, 0.0), "parado é (idade 0, desligado)");

    // E o CONTROLE: um grafo que nunca disparou fica em zero, quieto.
    let quiet = run(e, usize::MAX, 30, DT);
    assert!(
        quiet.iter().all(|v| *v == 0.0),
        "sem pulso não há envelope: {quiet:?}"
    );
}

/// **A forma linear é a IDENTIDADE, não uma aproximação dela.** `0.5` faz o termo
/// `(1/b − 2)` valer zero exato, então o denominador é `1` e o *bias* devolve `u`
/// bit a bit — é o que permite chamar o default de "reta" sem ressalva.
#[test]
fn the_linear_shape_is_the_identity_bit_for_bit() {
    for i in 0..=100 {
        let u = i as f32 / 100.0;
        assert_eq!(bias(u, 0.5), u, "bias({u}, 0.5) tem de ser u");
    }
    // E as duas direções fazem o que o rótulo promete, no meio do curso.
    assert!(bias(0.5, 0.2) < 0.5, "abaixo de 0,5 a rampa sai rápida");
    assert!(bias(0.5, 0.8) > 0.5, "acima de 0,5 ela sai lenta");
    // Os extremos são presos: nenhum `inf`/`NaN` chega à tela.
    for b in [0.0f32, 1.0, -5.0, 9.0, f32::NAN] {
        assert!(bias(0.5, b).is_finite(), "bias(_, {b}) tem de ser finito");
    }
}

/// **Cada shape governa exatamente os trechos que o cabeçalho promete** — o
/// `attack_shape` a subida, o `release_shape` as DUAS quedas. Sem este gate a
/// decisão de ter dois knobs em vez de três seria uma frase num doc-comment, e
/// trocar um pelo outro no `decay` passaria despercebido: com os dois no default
/// linear eles são indistinguíveis.
#[test]
fn each_shape_bends_exactly_the_segments_it_claims() {
    let mut e = env();
    e.release_shape = 0.2;
    // A subida NÃO se move: ela é do outro knob.
    let linear = env();
    assert_eq!(
        e.level(0.05),
        linear.level(0.05),
        "o release_shape não pode tocar o ataque"
    );
    // As duas QUEDAS se movem, e para o mesmo lado.
    assert_ne!(e.level(0.15), linear.level(0.15), "o decay curva");
    assert_ne!(e.level(0.4), linear.level(0.4), "e o release também");

    // E o espelho: o attack_shape move só a subida.
    let mut a = env();
    a.attack_shape = 0.2;
    assert_ne!(a.level(0.05), linear.level(0.05), "o ataque curva");
    assert_eq!(a.level(0.15), linear.level(0.15), "e não alcança o decay");
    assert_eq!(a.level(0.4), linear.level(0.4), "nem o release");
}

/// **`retrigger` decide o que um pulso faz com um envelope que já corre.** Ligado,
/// reinicia; desligado, é ignorado — e a diferença é observável porque o segundo
/// pulso cai no meio do decay, onde reiniciar leva o nível de volta a zero.
#[test]
fn retrigger_restarts_and_without_it_the_pulse_is_ignored() {
    let mut e = env();
    // Segundo disparo no tique 9 (0,15 s) — dentro do decay.
    e.retrigger = true;
    let restarted = run_two(e, 0, 9);
    e.retrigger = false;
    let ignored = run_two(e, 0, 9);

    assert_eq!(restarted[9], 0.0, "reiniciar leva o nível de volta a zero");
    assert!(
        ignored[9] > 0.5,
        "ignorado, o envelope segue no decay: {}",
        ignored[9]
    );
}

fn run_two(e: Envelope, a: usize, b: usize) -> Vec<f32> {
    let mut state = Stream::new(1);
    let mut out = Vec::new();
    for k in 0..40 {
        state = step(&pulse(k == a || k == b), &state, e, DT);
        out.push(level_of(&state));
    }
    out
}

/// **Um envelope por LINHA** — o campo é unário, cada instância dispara o seu. Duas
/// linhas, só uma pulsa: a outra tem de ficar em zero, e não herdar o vizinho.
#[test]
fn it_runs_one_envelope_per_row() {
    let e = env();
    let two = |a: bool, b: bool| {
        Stream::new(2).with(
            PULSE_COL,
            Column::Scalar(vec![if a { 1.0 } else { 0.0 }, if b { 1.0 } else { 0.0 }]),
        )
    };
    let mut state = step(&two(false, true), &Stream::new(2), e, DT);
    for _ in 0..3 {
        state = step(&two(false, false), &state, e, DT);
    }
    match state.get(VALUE_COL).unwrap() {
        Column::Scalar(v) => {
            assert_eq!(v[0], 0.0, "a linha que não disparou fica parada");
            assert!(v[1] > 0.0, "a que disparou está subindo: {}", v[1]);
        }
        _ => panic!(),
    }
}

/// **A duração é a AUTORADA em qualquer relógio** — o envelope anda em segundos,
/// não em tiques. ⚠️ A tolerância é DERIVADA: a saída só é amostrada em fronteira
/// de tique, então o fim medido pode passar do autorado por até um tique do relógio
/// mais grosso.
#[test]
fn the_duration_is_the_authored_one_at_any_clock() {
    let e = env();
    let total = e.total();
    for dt in [1.0f32 / 60.0, 1.0 / 15.0, 1.0 / 240.0] {
        let ticks = (2.0 * total / dt) as usize;
        let levels = run(e, 0, ticks, dt);
        let last_live = levels
            .iter()
            .rposition(|v| *v > 0.0)
            .expect("o envelope acendeu");
        let measured = (last_live + 1) as f32 * dt;
        // ⚠️ A margem é DERIVADA e tem dois termos, cada um de um tique: a saída só
        // é amostrada em fronteira de tique, e a idade é uma SOMA repetida de `dt`
        // (não um produto), então ela pode cruzar o total um tique atrasada.
        assert!(
            measured >= total && measured - total <= 2.0 * dt,
            "a dt={dt} o envelope durou {measured} s contra {total} autorados"
        );
    }
}

/// **O teto digitável dos tempos é o último que ainda faz a idade ANDAR**, e o
/// recurso é precisão de representação — o gêmeo exato do `debounce` do
/// `pulse.threshold`. Uma idade que para de crescer congela o envelope no nível em
/// que estava, em silêncio.
#[test]
fn the_hard_max_is_the_last_age_that_still_advances() {
    let ceiling = PARAM_HARD_MAX
        .iter()
        .find(|h| h.param == "release")
        .expect("os tempos têm teto digitável")
        .max;
    // O relógio mais RÁPIDO é o teste mais duro: o menor incremento.
    let dt = 1.0f32 / 240.0;
    assert!(ceiling + dt > ceiling, "{ceiling} s ainda avança a 240 fps");
    assert!(
        ceiling * 2.0 + dt <= ceiling * 2.0,
        "e {} s não — o teto senta NO penhasco",
        ceiling * 2.0
    );
    // E todos os tempos partilham o mesmo teto: é o mesmo `f32` a andar.
    for p in ["delay", "attack", "decay", "hold", "release"] {
        assert_eq!(
            PARAM_HARD_MAX
                .iter()
                .find(|h| h.param == p)
                .unwrap_or_else(|| panic!("{p} tem teto"))
                .max,
            ceiling
        );
    }
}

/// **Todo param declarado tem rótulo E seção** — a lei anti-param-mudo do doc 88,
/// afirmada aqui porque a lista deste nó é a maior da família e a nona linha é
/// exatamente a que alguém esquece.
#[test]
fn every_param_is_labelled_and_grouped() {
    for spec in MANIFEST.params {
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == spec.name),
            "{} não tem hint",
            spec.name
        );
        assert!(
            PARAM_GROUPS.iter().any(|g| g.param == spec.name),
            "{} não tem seção",
            spec.name
        );
    }
    // E o contrário: nada de hint órfão apontando para um param que não existe.
    for h in PARAM_HINTS {
        assert!(
            MANIFEST.params.iter().any(|p| p.name == h.param),
            "hint órfã: {}",
            h.param
        );
    }
}

#[test]
fn registers_and_resolves() {
    use ph2d_nodegraph::cook::OpResolver;
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
