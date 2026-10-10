use super::*;

fn valor(m: &MotionState, nome: &str, col: &str) -> Option<Vec<f32>> {
    match m
        .pump
        .cook
        .externals()
        .get(&input_of(nome))?
        .value
        .get(col)?
    {
        Column::Scalar(v) => Some(v.clone()),
        _ => None,
    }
}

/// ⭐ O que o jogo escreveu chega como `$input:<nome>`; o que deixou de ser escrito sai.
#[test]
fn what_the_game_writes_is_published_and_what_it_stops_writing_leaves() {
    let mut m = MotionState::new();
    m.entradas
        .jogo
        .set("vida", Escritor::Script, Valor::Numero(0.3));
    m.entradas.jogo.fire("dano", Escritor::Sinal);
    publica(&mut m);
    assert_eq!(valor(&m, "vida", INPUT_VALUE), Some(vec![0.3]));
    assert_eq!(valor(&m, "dano", INPUT_COUNT), Some(vec![1.0]));
    m.entradas.jogo.clear();
    publica(&mut m);
    assert!(m.pump.cook.externals().get(&input_of("vida")).is_none());
    assert!(m.pump.cook.externals().get(&input_of("dano")).is_none());
}

/// O disparo de TESTE: cada mudança do `fire` de um cartão Trigger é um disparo no nome dele; o
/// primeiro olhar não dispara.
#[test]
fn flipping_fire_on_a_trigger_card_fires_its_name_once_per_flip() {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let n = g.add_node("value.input");
    g.set_param(n, ph2d_node_value_input::KIND, 2.0);
    g.set_text_param(n, ph2d_node_value_input::NAME_KEY, "boom");
    publica(&mut m);
    assert!(
        valor(&m, "boom", INPUT_COUNT).is_none(),
        "ver não é disparar"
    );
    m.doc.graph.set_param(n, ph2d_node_value_input::FIRE, 1.0);
    publica(&mut m);
    assert_eq!(valor(&m, "boom", INPUT_COUNT), Some(vec![1.0]));
    publica(&mut m);
    assert_eq!(
        valor(&m, "boom", INPUT_COUNT),
        Some(vec![1.0]),
        "sem mudança, sem disparo"
    );
    m.doc.graph.set_param(n, ph2d_node_value_input::FIRE, 0.0);
    publica(&mut m);
    assert_eq!(valor(&m, "boom", INPUT_COUNT), Some(vec![2.0]));
}

/// Republicar o mesmo não suja a bomba (um quadro parado continua sem cozer).
#[test]
fn republishing_the_same_inputs_does_not_dirty_the_pump() {
    let mut m = MotionState::new();
    m.entradas
        .jogo
        .set("vida", Escritor::Script, Valor::Numero(0.3));
    publica(&mut m);
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let sinks = m.sinks.clone();
    m.pump.advance_or_scrub_scoped(
        &m.doc.graph,
        &m.registry,
        &sinks,
        0,
        |t| t as f64 / 60.0,
        m.default_uv_rect,
        m.default_size,
        &scopes,
    );
    assert!(!m.pump.is_dirty());
    publica(&mut m);
    assert!(!m.pump.is_dirty(), "o mesmo valor, nenhuma mudança");
    m.entradas
        .jogo
        .set("vida", Escritor::Script, Valor::Numero(0.4));
    publica(&mut m);
    assert!(
        m.pump.is_dirty(),
        "um valor novo com o relógio parado tem de se ver"
    );
}

/// A linha de comando: números, booleanos e cores; o que não se entende salta-se.
#[test]
fn the_command_line_reads_numbers_booleans_and_colours() {
    let l = le_a_linha(" vida=0.25; ligado=true ;time=0.2,0.5,1; mau=x; sem; alfa=1,0,0,0.5");
    assert_eq!(
        l,
        vec![
            ("vida".to_string(), Valor::Numero(0.25)),
            ("ligado".to_string(), Valor::Numero(1.0)),
            ("time".to_string(), Valor::Cor([0.2, 0.5, 1.0, 1.0])),
            ("alfa".to_string(), Valor::Cor([1.0, 0.0, 0.0, 0.5])),
        ]
    );
    let mut m = MotionState::new();
    m.entradas.da_linha = l;
    publica(&mut m);
    assert_eq!(valor(&m, "vida", INPUT_VALUE), Some(vec![0.25]));
}
