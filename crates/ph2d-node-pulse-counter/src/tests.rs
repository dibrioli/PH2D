//! Gates do `pulse.counter`. `super` e a raiz da crate — o modulo segue FILHO (via
//! `#[path]`) para `use super::*` alcancar os privados (`step`, `displayed`, `carry_fired`),
//! que sao exatamente o que estes gates medem.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// O registry REAL — o gate do divisor de relógio cozinha o grafo do produto, não um
/// espelho da lei.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

fn fire(v: f32) -> Stream {
    Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
}
/// Um tique SEM reset — o que todo gate anterior a esta wave exercita. A porta
/// desconectada é um `Stream` vazio, exatamente o que o `EvalCtx` entrega.
/// Um tique com o incremento de FÁBRICA (`+1`) e sem reset — a forma que os gates
/// anteriores à wave do carry sempre pediram, para nenhum deles mudar de sentido.
fn tick(pulse: &Stream, state: &Stream, count_max: i64, mode: LimitMode) -> Stream {
    step(pulse, state, &Stream::new(0), count_max, mode, 0.0, 1.0).value
}

/// Um tique com um incremento AUTORADO, e devolve as DUAS saídas.
fn tick_by(
    pulse: &Stream,
    state: &Stream,
    count_max: i64,
    mode: LimitMode,
    step_by: f32,
) -> Counted {
    step(pulse, state, &Stream::new(0), count_max, mode, 0.0, step_by)
}

fn carried(c: &Counted) -> bool {
    match c.carry.get(PULSE_COL).unwrap() {
        Column::Scalar(v) => v[0] > 0.5,
        _ => panic!(),
    }
}

fn value(s: &Stream) -> f32 {
    match s.get(VALUE_COL).unwrap() {
        Column::Scalar(v) => v[0],
        _ => panic!(),
    }
}

/// FALSIFICATION of edge-safety: a pulse HELD high advances the count ONCE
/// (on the rising edge), not once per tick. Counting `pulse > 0.5` every tick
/// would reach 5 after a 5-tick hold.
#[test]
fn a_held_pulse_counts_once_not_once_per_tick() {
    let mut state = Stream::new(1);
    for _ in 0..5 {
        state = tick(&fire(1.0), &state, 16, LimitMode::Wrap);
    }
    assert_eq!(value(&state), 1.0, "one rising edge = one count, not five");
    state = tick(&fire(0.0), &state, 16, LimitMode::Wrap);
    state = tick(&fire(1.0), &state, 16, LimitMode::Wrap);
    assert_eq!(value(&state), 2.0, "the next rising edge counts once more");
}

/// Um tique COM reset, para os gates da porta nova.
fn tick_reset(pulse: &Stream, state: &Stream, reset: f32, count_max: i64, reset_to: f32) -> Stream {
    step(
        pulse,
        state,
        &fire(reset),
        count_max,
        LimitMode::Wrap,
        reset_to,
        1.0,
    )
    .value
}

/// **A porta desconectada é o mundo ANTERIOR, byte a byte.** O neutro não é uma
/// promessa em prosa: a sequência com a porta vazia tem de bater, elemento a
/// elemento, com a mesma corrida em que o reset nunca sobe.
#[test]
fn an_unconnected_reset_is_the_world_before_it() {
    let run = |with_port: bool| {
        let mut state = Stream::new(1);
        let mut seq = Vec::new();
        for k in 0..12 {
            let p = fire(if k % 2 == 0 { 1.0 } else { 0.0 });
            state = if with_port {
                tick_reset(&p, &state, 0.0, 4, 0.0)
            } else {
                tick(&p, &state, 4, LimitMode::Wrap)
            };
            seq.push(value(&state));
        }
        seq
    };
    assert_eq!(
        run(true),
        run(false),
        "porta desconectada e reset em zero descrevem a MESMA contagem"
    );
}

/// **O reset traz a contagem para casa** — a capacidade inteira da wave: uma linha
/// que acumulou pode voltar ao começo sem que o documento seja reconstruído.
#[test]
fn the_reset_returns_the_count_home() {
    let mut state = Stream::new(1);
    for _ in 0..3 {
        state = tick_reset(&fire(1.0), &state, 0.0, 16, 0.0);
        state = tick_reset(&fire(0.0), &state, 0.0, 16, 0.0);
    }
    assert_eq!(value(&state), 3.0, "três bordas, três contagens");
    state = tick_reset(&fire(0.0), &state, 1.0, 16, 0.0);
    assert_eq!(value(&state), 0.0, "o reset devolve a contagem ao começo");
    // …e ela volta a contar dali, em vez de ficar presa em zero.
    state = tick_reset(&fire(1.0), &state, 0.0, 16, 0.0);
    assert_eq!(
        value(&state),
        1.0,
        "e a contagem segue viva depois do reset"
    );
}

/// **O reset GANHA de uma contagem simultânea** — a decisão do TD, pinada. Se a
/// contagem ganhasse, uma linha que recebe pulso e reset no mesmo tique nunca
/// poderia ser zerada por um sinal que chega junto com o metrônomo.
#[test]
fn the_reset_wins_a_simultaneous_count() {
    let mut state = Stream::new(1);
    for _ in 0..4 {
        state = tick_reset(&fire(1.0), &state, 0.0, 16, 0.0);
        state = tick_reset(&fire(0.0), &state, 0.0, 16, 0.0);
    }
    assert_eq!(value(&state), 4.0);
    // Pulso E reset no MESMO tique (a borda subiria de 4 para 5).
    state = tick_reset(&fire(1.0), &state, 1.0, 16, 0.0);
    assert_eq!(value(&state), 0.0, "o reset ganha da borda simultânea");
}

/// **`reset_to` é para ONDE o reset leva** (TD Count CHOP `Reset Value`), e ele
/// atravessa o dobramento do modo como qualquer outro tique.
#[test]
fn the_reset_lands_on_the_authored_value() {
    let mut state = Stream::new(1);
    state = tick_reset(&fire(1.0), &state, 0.0, 6, 3.0);
    assert_eq!(value(&state), 1.0);
    state = tick_reset(&fire(0.0), &state, 1.0, 6, 3.0);
    assert_eq!(value(&state), 3.0, "o reset pousa no valor autorado");
}

/// The reducer emits a VALUE and NEVER a transform channel — the whole point
/// of the pure split. The output stream carries `v` (+ the state columns) and
/// no `P`/`rot`/`size`.
#[test]
fn it_emits_a_value_column_and_no_transform_channel() {
    let s = tick(&fire(1.0), &Stream::new(1), 6, LimitMode::Wrap);
    assert!(s.get(VALUE_COL).is_some(), "emits the value column");
    assert!(s.get("P").is_none(), "no position");
    assert!(
        s.get("rot").is_none() && s.get("size").is_none(),
        "no channel"
    );
}

/// Wrap / Clamp / Zigzag fold the same monotonic tick three ways (TD Count
/// CHOP limit modes), as a VALUE sequence.
#[test]
fn the_three_limit_modes_fold_the_count() {
    let run = |count_max, mode| {
        let mut state = Stream::new(1);
        let mut seq = Vec::new();
        for _ in 0..8 {
            state = tick(&fire(1.0), &state, count_max, mode);
            seq.push(value(&state));
            state = tick(&fire(0.0), &state, count_max, mode);
        }
        seq
    };
    assert_eq!(
        run(4, LimitMode::Wrap),
        vec![1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0, 0.0],
        "wrap returns home"
    );
    assert_eq!(
        run(3, LimitMode::Clamp),
        vec![1.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0],
        "clamp plateaus at N-1"
    );
    assert_eq!(
        run(4, LimitMode::Zigzag),
        vec![1.0, 2.0, 3.0, 2.0, 1.0, 0.0, 1.0, 2.0],
        "zigzag ping-pongs"
    );
}

/// `count_max = 1` never divides by zero and stays home (value 0).
#[test]
fn count_max_one_stays_home_without_dividing_by_zero() {
    let mut state = Stream::new(1);
    for _ in 0..5 {
        state = tick(&fire(1.0), &state, 1, LimitMode::Zigzag);
        state = tick(&fire(0.0), &state, 1, LimitMode::Zigzag);
    }
    assert_eq!(value(&state), 0.0);
}

// ─────────────────────────────────────────────────────────────────────
// O INCREMENTO e o CARRY (folha 12 §P1, 2026-08-10)
// ─────────────────────────────────────────────────────────────────────

/// Uma batida completa: o tique que SOBE (onde a contagem anda e o carry vive) mais o
/// tique baixo que re-arma a borda seguinte.
///
/// ⚠️ **A primeira versão devolvia o `Counted` do tique BAIXO** e os dois gates de carry
/// nasceram com ele mudo em toda batida — o carry mora no tique da borda, e um tique
/// quieto não carrega por lei (é o que `um_tique_quieto_nunca_carrega` afirma). O estado
/// vem do tique baixo, o carry vem do de cima: *a fixture tem de conter o fenômeno, e
/// aqui ela o descartava*.
fn beat(state: &Stream, n: i64, mode: LimitMode, by: f32) -> Counted {
    let up = tick_by(&fire(1.0), state, n, mode, by);
    let settled = tick_by(&fire(0.0), &up.value, n, mode, by);
    Counted {
        value: settled.value,
        carry: up.carry,
    }
}

/// **A CONTAGEM CORRE PARA TRÁS** — a capacidade que o incremento negativo compra, e a
/// razão de o `Clamp` ter ganho piso.
///
/// Sob `Wrap` o módulo euclidiano já respondia certo a um tique negativo (`-1 mod 6 = 5`),
/// então descer é *"subir ao contrário"* de graça. É o `Clamp` que precisava do piso: sem
/// ele a contagem exibida iria a **-1, -2, -3** — números que uma contagem não tem.
#[test]
fn a_contagem_corre_para_tras() {
    let mut state = Stream::new(1);
    let mut vistos = Vec::new();
    for _ in 0..4 {
        let c = beat(&state, 6, LimitMode::Wrap, -1.0);
        vistos.push(value(&c.value));
        state = c.value;
    }
    assert_eq!(
        vistos,
        vec![5.0, 4.0, 3.0, 2.0],
        "conta para trás e dá a volta"
    );

    let mut state = Stream::new(1);
    let mut chao = Vec::new();
    for _ in 0..3 {
        let c = beat(&state, 6, LimitMode::Clamp, -1.0);
        chao.push(value(&c.value));
        state = c.value;
    }
    assert_eq!(
        chao,
        vec![0.0, 0.0, 0.0],
        "o Clamp tem PISO: nunca uma contagem negativa"
    );
}

/// **O piso do `Clamp` é byte-idêntico no mundo que já shipava.** A afirmação do
/// doc-comment não é prosa: com o incremento de fábrica o tique só sabe crescer a partir
/// de zero, então `clamp(0, n-1)` e `min(n-1)` concordam em TODO tique alcançável.
#[test]
fn o_piso_do_clamp_nao_move_o_mundo_de_hoje() {
    for t in 0..64i64 {
        for n in 1..12i64 {
            assert_eq!(
                displayed(t, n, LimitMode::Clamp),
                t.min(n - 1),
                "tique {t}, n {n}: o piso só morde abaixo de zero"
            );
        }
    }
}

/// **O CARRY DISPARA QUANDO O CICLO DÁ A VOLTA** — e em mais nenhuma batida.
///
/// A metade *"e em mais nenhuma"* é o gate: sem ela um carry cravado em `1.0` passaria,
/// e o divisor de relógio que ele existe para permitir dividiria por um.
#[test]
fn o_carry_dispara_quando_o_ciclo_da_a_volta() {
    let mut state = Stream::new(1);
    let mut fogo = Vec::new();
    for _ in 0..9 {
        let c = beat(&state, 4, LimitMode::Wrap, 1.0);
        fogo.push(carried(&c));
        state = c.value;
    }
    // Exibido 1,2,3,0,1,2,3,0,1 — o carry cai na batida em que a contagem CHEGA
    // em casa (a 4ª e a 8ª), não na anterior: é a fronteira do ciclo que ele marca.
    assert_eq!(
        fogo,
        vec![false, false, false, true, false, false, false, true, false],
        "uma volta a cada 4 batidas, nem uma a mais"
    );
}

/// **O ZIGZAG carrega na volta COMPLETA, não na dobra do meio** — a mesma lei, outro
/// modo, sem um `if` por modo em lugar nenhum.
///
/// ⚠️ Este gate nasceu com a expectativa *"carrega em cada dobra"* e foi ele que derrubou
/// a primeira lei (ver [`carry_fired`]). A dobra do meio é a trajetória do zigzag, não uma
/// fronteira de ciclo: o contador só volta ao ponto de partida depois da ida E da volta, e
/// é ISSO que um contador a jusante tem de contar.
#[test]
fn o_carry_marca_a_volta_completa_do_zigzag() {
    let mut state = Stream::new(1);
    let mut vistos = Vec::new();
    for _ in 0..6 {
        let c = beat(&state, 4, LimitMode::Zigzag, 1.0);
        vistos.push((value(&c.value), carried(&c)));
        state = c.value;
    }
    // 1,2,3,2,1,0 — seis batidas fecham o período `2(n-1)`, e o carry cai na última.
    assert_eq!(
        vistos,
        vec![
            (1.0, false),
            (2.0, false),
            (3.0, false),
            (2.0, false),
            (1.0, false),
            (0.0, true),
        ],
        "a fronteira do ciclo é a volta INTEIRA, não a dobra"
    );

    // E o CLAMP nunca carrega, porque não tem ciclo: uma contagem que para nunca
    // completa uma volta. Sem esta metade, um carry cravado em Wrap passaria por aqui.
    let mut state = Stream::new(1);
    for k in 0..8 {
        let c = beat(&state, 4, LimitMode::Clamp, 1.0);
        assert!(!carried(&c), "o Clamp não tem ciclo a fechar (batida {k})");
        state = c.value;
    }
}

/// **Um tique QUIETO nunca carrega** — a cláusula (a), e a versão ingênua da lei falha
/// aqui de forma espetacular: sem o `advanced`, *"a contagem não andou os `step`"* é
/// trivialmente verdade num tique sem pulso, e o carry vira um pulso CONTÍNUO.
#[test]
fn um_tique_quieto_nunca_carrega() {
    let mut state = Stream::new(1);
    // Leva a contagem até a beira da volta…
    for _ in 0..3 {
        state = beat(&state, 4, LimitMode::Wrap, 1.0).value;
    }
    // …e depois deixa quieto: dez tiques sem pulso nenhum.
    for k in 0..10 {
        let c = tick_by(&fire(0.0), &state, 4, LimitMode::Wrap, 1.0);
        assert!(!carried(&c), "tique quieto {k} não pode carregar");
        state = c.value;
    }
}

/// **Um RESET não carrega** — a cláusula (b). Ele salta a contagem para um lugar
/// arbitrário, e sem esta cláusula acertaria o carry por acidente sempre que o destino
/// não fosse `anterior + step`.
#[test]
fn um_reset_nao_carrega() {
    let mut state = Stream::new(1);
    // ⚠️ **CINCO batidas, não duas.** A primeira versão parava no tique 2 e resetava para
    // 0 — os dois no MESMO ciclo (`div_euclid(_, 4) == 0`), então o salto não cruzava
    // fronteira nenhuma e a mutação que apaga a cláusula do reset **sobrevivia**. O gate
    // só mede o que a cláusula faz se o salto de facto atravessar um ciclo.
    for _ in 0..5 {
        state = beat(&state, 4, LimitMode::Wrap, 1.0).value;
    }
    // Pulso E reset no mesmo tique: o reset ganha a contagem (a lei do `tick_after_reset`)
    // e NÃO acende o carry, por mais que a contagem tenha saltado.
    // Tique 5 (ciclo 1) → reset para 0 (ciclo 0): o salto CRUZA a fronteira.
    let c = step(&fire(1.0), &state, &fire(1.0), 4, LimitMode::Wrap, 0.0, 1.0);
    assert_eq!(value(&c.value), 0.0, "o reset ganha a contagem simultânea");
    assert!(
        match c.carry.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => v[0] <= 0.5,
            _ => panic!(),
        },
        "um comando de fora não é o ciclo virando"
    );
}

/// **DOIS CONTADORES ENCADEADOS SÃO UM DIVISOR DE RELÓGIO** — o pagamento inteiro da
/// saída nova, medido pelo grafo REAL.
///
/// `beat → counter(4) → carry → counter(4)`: o segundo só anda quando o primeiro dá a
/// volta, então ele conta de quatro em quatro batidas. ⚠️ E este é o **primeiro grafo do
/// repo a ler a porta de saída 1 de alguma coisa** — o motor sempre a indexou
/// (`Cook::cur_output`), mas nada a exercitava.
#[test]
fn dois_contadores_encadeados_sao_um_divisor_de_relogio() {
    let reg = registry();
    let mut g = Graph::new();
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 1.0);
    let bt = g.add_node("pulse.beat");
    g.set_param(bt, "period", 0.1);
    let baixo = g.add_node("pulse.counter");
    g.set_param(baixo, "count_max", 4.0);
    let alto = g.add_node("pulse.counter");
    g.set_param(alto, "count_max", 16.0);
    for (a, ap, b, bp, delayed) in [
        (src, 0u16, bt, 0u16, false),
        (bt, 0, bt, 1, true),
        (bt, 0, baixo, 0, false),
        (baixo, 0, baixo, 1, true),
        // A PORTA 1: o carry do de baixo é o pulso do de cima.
        (baixo, 1, alto, 0, false),
        (alto, 0, alto, 1, true),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed,
        })
        .expect("fio");
    }
    assert!(
        g.validate(&reg).is_ok(),
        "o carry é um PULSO e casa com a porta de pulso"
    );

    let mut cook = Cook::new();
    let mut baixo_v = Vec::new();
    let mut alto_v = Vec::new();
    for k in 0..=96u64 {
        let t = k as f64 / 60.0;
        let out = cook.cook(&g, &reg, alto, t).expect("cozinha");
        let s = out[0].as_stream();
        if let Some(Column::Scalar(v)) = s.get(VALUE_COL) {
            alto_v.push(v[0]);
        }
        let ob = cook.cook(&g, &reg, baixo, t).expect("cozinha");
        if let Some(Column::Scalar(v)) = ob[0].as_stream().get(VALUE_COL) {
            baixo_v.push(v[0]);
        }
        cook.advance_tick(&g, &reg, t).expect("fecha o tique");
    }
    let b_max = baixo_v.iter().cloned().fold(0.0f32, f32::max);
    let a_fim = *alto_v.last().unwrap();
    assert!(b_max >= 3.0, "o de baixo dá a volta: máximo {b_max}");
    assert!(
        a_fim >= 3.0,
        "o de cima conta as VOLTAS do de baixo, não as batidas: {a_fim}"
    );
    // A prova de que ele DIVIDE: o de cima anda estritamente menos que o de baixo andou.
    let b_voltas = baixo_v.windows(2).filter(|w| w[1] != w[0]).count();
    let a_voltas = alto_v.windows(2).filter(|w| w[1] != w[0]).count();
    assert!(
        a_voltas * 3 <= b_voltas,
        "divisor por 4: o de cima anda {a_voltas} contra {b_voltas} do de baixo"
    );
}

/// **O param `step` CHEGA à lei, pela porta do produto.**
///
/// ⚠️ Este gate existe porque a mutação *"o `eval` ignora o param e usa 1.0"* **sobreviveu
/// a todos os outros**: eles chamam [`step`] direto com o incremento explícito, então
/// nenhum passava pelo `ctx.param("step")`. Um param pode estar declarado, ter hint, ter
/// lei gateada — e não chegar nela. É a costura, e ela custa um gate próprio.
#[test]
fn o_incremento_autorado_chega_a_lei() {
    let reg = registry();
    let mut g = Graph::new();
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 1.0);
    let bt = g.add_node("pulse.beat");
    g.set_param(bt, "period", 0.1);
    let ct = g.add_node("pulse.counter");
    g.set_param(ct, "count_max", 6.0);
    g.set_param(ct, "step", -1.0); // conta PARA TRÁS
    for (a, ap, b, bp, delayed) in [
        (src, 0u16, bt, 0u16, false),
        (bt, 0, bt, 1, true),
        (bt, 0, ct, 0, false),
        (ct, 0, ct, 1, true),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed,
        })
        .expect("fio");
    }
    assert!(g.validate(&reg).is_ok());

    let mut cook = Cook::new();
    let mut vistos = Vec::new();
    for k in 0..=40u64 {
        let t = k as f64 / 60.0;
        let out = cook.cook(&g, &reg, ct, t).expect("cozinha");
        if let Some(Column::Scalar(v)) = out[0].as_stream().get(VALUE_COL) {
            vistos.push(v[0]);
        }
        cook.advance_tick(&g, &reg, t).expect("fecha o tique");
    }
    // Com incremento −1 e `Wrap`, a primeira batida leva a contagem a 5 (o módulo
    // euclidiano dá a volta por baixo). Com o param ignorado ela iria a 1.
    assert!(
        vistos.contains(&5.0),
        "o incremento negativo tem de ATRAVESSAR o eval: {vistos:?}"
    );
    assert!(
        !vistos.contains(&1.0)
            || vistos.iter().position(|v| *v == 5.0) < vistos.iter().position(|v| *v == 1.0),
        "a contagem desce (5,4,3…), não sobe: {vistos:?}"
    );
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
