//! As provas do número autorado.

use super::*;

/// **O MODO NÚMERO ENTREGA O NÚMERO** — a razão de o nó existir.
#[test]
fn the_number_mode_emits_the_authored_number() {
    for v in [-720.0_f32, -1.5, 0.0, 1.0, 42.0, 22_000.0] {
        assert_eq!(value_of(0.0, v, 0.0), v, "modo Number com {v}");
        // O `state` não participa no modo Number — senão a caixa escondida moveria o valor.
        assert_eq!(value_of(0.0, v, 1.0), v);
    }
}

/// **O MODO BOOLEANO ENTREGA `0` OU `1`, EXACTOS** — nunca o `state` cru.
///
/// ⚠️ O destino é um param que outro nó lê como número: um booleano que chegasse `0,7` seria um
/// terceiro estado que a caixa não consegue exprimir nem desfazer.
#[test]
fn the_boolean_mode_emits_exactly_zero_or_one() {
    assert_eq!(value_of(1.0, 99.0, 0.0), 0.0);
    assert_eq!(value_of(1.0, 99.0, 1.0), 1.0);
    // Um valor intermédio (um `state` vindo de um documento antigo ou de um fio) COLAPSA.
    assert_eq!(value_of(1.0, 99.0, 0.49), 0.0);
    assert_eq!(value_of(1.0, 99.0, 0.5), 1.0);
    assert_eq!(value_of(1.0, 99.0, 7.0), 1.0);
    // E o `value` não vaza para o modo booleano.
    assert_eq!(value_of(1.0, -1234.0, 1.0), 1.0);
}

/// **UM `kind` LIXO CAI NO MODO NÚMERO** — o default, e o caminho de sempre.
#[test]
fn a_junk_kind_falls_back_to_the_number() {
    for k in [f32::NAN, f32::INFINITY, -3.0, 99.0, 0.4] {
        assert_eq!(value_of(k, 7.0, 1.0), 7.0, "kind {k}");
    }
}

/// **O NÓ NASCE A EMITIR `1`, e não `0`.**
///
/// ⚠️ `0` é o neutro da soma **e** o absorvente do produto: um `Number` acabado de largar num
/// `value.math` não mudaria nada e leria como partido. É o mesmo raciocínio do `debug.const`,
/// que emitia `1` — e é a diferença entre um nó que se explica ao ser largado e um que precisa
/// de manual.
#[test]
fn a_freshly_dropped_node_already_does_something() {
    let d = |name: &str| {
        MANIFEST
            .params
            .iter()
            .find(|p| p.name == name)
            .expect("declarado")
            .default
    };
    assert_eq!(value_of(d(KIND), d(VALUE_PARAM), d(STATE)), 1.0);
}

/// **CADA MODO MOSTRA O SEU CONTROLE, E SÓ O SEU** — e os dois gates existem.
///
/// ⚠️ Sem isto o painel pinta a caixa e o slider ao mesmo tempo, e um deles é um controle sobre
/// nada. A régua é o REGISTRY (é quem o painel consulta), não a tabela estática: apagar a
/// chamada de registo deixaria o `static` intacto e o painel errado — foi exactamente essa a
/// mutação que sobreviveu no `fx.glow` (auditoria de 2026-08-27).
#[test]
fn the_registry_carries_a_gate_for_each_mode() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("regista");
    let gates = reg
        .param_gates(MANIFEST.id)
        .expect("o registry tem de carregar os gates");
    let has = |param: &str, mode: i32| {
        gates
            .iter()
            .any(|g| g.param == param && g.when == KIND && g.values.contains(&mode))
    };
    assert!(has(VALUE_PARAM, 0), "o slider aparece no modo Number");
    assert!(has(STATE, 1), "a caixa aparece no modo Boolean");
    assert!(!has(VALUE_PARAM, 1), "o slider NAO aparece no Boolean");
    assert!(!has(STATE, 0), "a caixa NAO aparece no Number");
}

/// **O TECTO DIGITÁVEL É MAIS LARGO QUE O CURSO DA MÃO, e o número tem RECURSO.**
///
/// ⚠️ A barra não é uma opinião: com passo `0,01`, um `f32` só soma o passo enquanto
/// `ULP(v) ≤ 0,01`. Este gate afirma as duas metades — que `TYPED_LIMIT` ainda move, e que o
/// dobro dele **já não move**. *Um tecto que diz de que recurso é pode ser verificado; um que
/// diz «por segurança» não.*
#[test]
fn the_typed_ceiling_is_where_the_step_stops_moving_the_number() {
    let step = PARAM_HINTS
        .iter()
        .find(|h| h.param == VALUE_PARAM)
        .expect("o slider")
        .step;
    // As DUAS metades, e é o par que torna o tecto uma medição em vez de um palpite: no tecto
    // o passo ainda move, e no degrau seguinte (a potência de dois acima) já não.
    assert!(
        TYPED_LIMIT + step > TYPED_LIMIT,
        "no tecto o passo ainda move"
    );
    let beyond = TYPED_LIMIT * 2.0;
    assert_eq!(beyond + step, beyond, "no degrau seguinte o passo evapora");
    let hand = PARAM_HINTS
        .iter()
        .find(|h| h.param == VALUE_PARAM)
        .expect("o slider")
        .max;
    assert!(
        HARD_MAX[0].max > hand,
        "o tecto digitavel tem de ser mais largo que o curso da mao"
    );
    assert_eq!(HARD_MIN[0].min, -HARD_MAX[0].max, "e simetrico");
    // ⚠️ E ele tem de cobrir o maior slider do CATÁLOGO (`22 000`, o `sim.spawn::rate`) — senão
    // este nó não consegue conduzir o param que mais precisa dele.
    assert!(HARD_MAX[0].max > 22_000.0);
}
