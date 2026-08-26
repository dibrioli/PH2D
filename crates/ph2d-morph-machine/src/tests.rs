//! Os gates da lei — a metade que o **runtime do jogo** vai correr.

use super::*;

const A: ShapeId = 10;
const B: ShapeId = 20;
const C: ShapeId = 30;

/// Três formas: `A` é onde se nasce (sem tecla), `jump` leva a `B`, `dash` leva a `C`.
///
/// ⚠️ **Leia a fixtura em voz alta e o modelo aparece:** a tecla é o nome da FORMA, não o nome da
/// passagem. É por isso que a lista tem três entradas e não seis.
fn graph() -> MorphGraph {
    let mut g = MorphGraph {
        states: vec![MorphState::new(A), MorphState::new(B), MorphState::new(C)],
    };
    g.states[1].when = "jump".to_string();
    g.states[2].when = "dash".to_string();
    g
}

/// Anda `n` quadros de 60 fps.
fn frames(m: &mut MorphMachine, g: &MorphGraph, n: usize) {
    for _ in 0..n {
        m.advance(g, 1.0 / 60.0);
    }
}

/// ⭐⭐⭐ **A MESMA TECLA LEVA À MESMA FORMA, DE ONDE QUER QUE SE ESTEJA** — a lei da W10.
///
/// Enio, 2026-08-25: *"se a seta para cima leva ao retângulo azul, independente de que forma
/// estiver ativa no momento, a seta para cima vai levar ao retângulo azul."*
///
/// **Mutação que deve sangrar:** o `reached_by` filtrar por um `from` — a tecla passaria a
/// significar coisas diferentes conforme o sítio, que é exactamente a teia que este modelo apaga.
#[test]
fn the_same_action_reaches_the_same_shape_from_anywhere() {
    let g = graph();
    for origin in [A, B, C] {
        let mut m = MorphMachine::new(&g);
        // Põe a máquina em `origin` sem usar a tecla que estamos a medir.
        if origin != A {
            let ix = g.states.iter().position(|s| s.shape == origin).unwrap();
            assert!(m.travel(&g, ix));
            frames(&mut m, &g, 60);
        }
        assert_eq!(m.current(), origin);
        if origin == B {
            continue; // o `jump` daqui é a própria forma — tem gate próprio abaixo.
        }
        assert!(
            m.fire(&g, "jump"),
            "o jump tem de valer a partir de {origin}"
        );
        assert_eq!(m.current(), B, "e tem de levar SEMPRE a B");
    }
}

/// ⛔ **A TECLA DA FORMA EM QUE JÁ SE ESTÁ NÃO FAZ NADA.**
///
/// ⚠️ Ela seria uma transição de uma forma para ela própria: nem sequer é exprimível (o `VecMorph`
/// guardaria `(X, X)` e o `t` andaria sobre um caminho de comprimento zero), e o artista leria um
/// estremecimento sem causa. *Chegar onde já se está não é chegar.*
///
/// **Mutação que deve sangrar:** o `fire` largar o `st.shape == self.current`.
#[test]
fn the_key_of_the_shape_you_are_already_on_does_nothing() {
    let g = graph();
    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    frames(&mut m, &g, 120);
    assert_eq!(m.current(), B);
    let before = (m.pair(), m.t(), m.current());
    assert!(
        !m.fire(&g, "jump"),
        "ja' estamos em B -- o jump nao faz nada"
    );
    assert_eq!((m.pair(), m.t(), m.current()), before, "e nada se mexeu");
    // O CONTROLE: a OUTRA tecla continua a funcionar daqui.
    assert!(m.fire(&g, "dash"));
    assert_eq!(m.current(), C);
}

/// ⛔ **Uma forma SEM CONDIÇÃO não é alcançada por tecla nenhuma.**
///
/// **Mutação que deve sangrar:** `matches` largar o `!self.when.is_empty()` — toda forma sem tecla
/// passaria a responder a uma acção de nome vazio, e a forma saltava sem ninguém ter pedido.
#[test]
fn a_shape_without_a_condition_is_never_reached_by_a_key() {
    let mut g = MorphGraph {
        states: vec![MorphState::new(A), MorphState::new(B)],
    };
    let mut m = MorphMachine::new(&g);
    assert!(!m.fire(&g, ""), "o nome vazio nao pode casar");
    assert!(!m.fire(&g, "jump"));
    assert_eq!(m.current(), A);
    // O CONTROLE: com nome, a MESMA forma é alcançada.
    g.states[1].when = "jump".to_string();
    assert!(m.fire(&g, "jump"));
    assert_eq!(m.current(), B);
}

/// ⭐ **O `current` SALTA NO LANÇAMENTO** — a meio de `A→B`, é de `B` que se parte a seguir.
///
/// **Mutação que deve sangrar:** `current` passar a mudar só na chegada. Encadear `jump` e depois
/// `jump` outra vez durante o voo deixaria o segundo a ser lido a partir de `A` — e ele **passaria**
/// (em A o jump é válido), pondo a máquina a voar para B a partir de B.
#[test]
fn the_current_lands_where_the_flight_lands_so_a_chain_never_loses_an_input() {
    let g = graph();
    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    assert_eq!(
        m.current(),
        B,
        "o voo aterra em B, logo e' de B que se parte a seguir"
    );
    frames(&mut m, &g, 2); // ainda em voo
    assert!(m.is_flying());
    assert!(
        !m.fire(&g, "jump"),
        "o jump em voo PARA B tem de ser recusado -- a maquina ja' se comprometeu com B"
    );
    assert!(m.fire(&g, "dash"), "e a outra tecla tem de entrar na fila");
    assert_eq!(
        m.current(),
        B,
        "mas um pedido em FILA nao move o current -- senao o proximo pedido seria lido a partir \
         de um sitio onde a maquina ainda nao esta'"
    );
    frames(&mut m, &g, 120);
    assert_eq!(m.current(), C, "e o pedido em fila arranca na chegada");
}

/// ⭐⭐ **UM PEDIDO A MEIO DO VOO ESPERA A CHEGADA** — e o par nunca é de três formas.
///
/// ⛔ As duas alternativas foram fechadas por argumento, e ficam escritas: **ignorar** o pedido
/// perde o input do jogador; **saltar** para o par novo não é exprimível, porque o `VecMorph` guarda
/// **um par** e sair do meio de `(A,B)` para `(B,C)` precisaria de uma mistura de três.
///
/// **Mutação que deve sangrar:** o `take` chamar `launch` mesmo em voo — o par trocaria a meio e a
/// forma **saltava** visivelmente.
#[test]
fn a_request_mid_flight_waits_for_the_arrival() {
    let g = graph();
    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    frames(&mut m, &g, 2);
    m.fire(&g, "dash");
    assert_eq!(m.pair(), (A, B), "o par NAO pode trocar a meio do voo");
    frames(&mut m, &g, 60);
    assert_eq!(m.pair(), (B, C), "chegou a B e o pedido guardado arrancou");
    frames(&mut m, &g, 60);
    assert!(!m.is_flying());
    assert_eq!((m.pair(), m.t()), ((B, C), 1.0));
}

/// ⚠️ **O MAIS NOVO GANHA, e a fila não cresce.** Uma fila funda reproduziria, um segundo depois,
/// teclas que o jogador já esqueceu.
///
/// ⛔⛔ **Este gate nasceu VERMELHO e o defeito era de DESENHO, não de código** (2026-08-25): com o
/// `current` a saltar também na fila, o segundo pedido era lido a partir de um estado onde a
/// máquina ainda não estava. A cura foi mover o salto do `current` para o lançamento — e é ela que
/// torna "o mais novo ganha" seguro *por construção*, porque todo candidato à fila parte do MESMO
/// sítio.
///
/// **Mutação que deve sangrar:** trocar o `pending` por um `Vec` que empilha.
#[test]
fn the_newest_request_wins_and_the_queue_never_grows() {
    let mut g = graph();
    // Uma tecla para VOLTAR a A, para haver DOIS pedidos possiveis em voo.
    g.states[0].when = "back".to_string();

    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    frames(&mut m, &g, 2);
    m.fire(&g, "dash"); // pede -> C
    m.fire(&g, "back"); // e logo a seguir -> A: este e' o que vale
    frames(&mut m, &g, 60);
    assert_eq!(m.pair(), (B, A), "o pedido mais NOVO tem de ganhar");
    frames(&mut m, &g, 60);
    assert!(!m.is_flying(), "e nao pode sobrar um terceiro voo na fila");
    assert_eq!(m.current(), A);
}

/// ⭐ **CHEGAR NÃO TROCA O PAR** — `t = 1` em `(A,B)` já É a forma B.
///
/// ⚠️ Isto não é elegância: o cache de `Plan` do `morph_live` é chaveado pela geometria em MUNDO
/// das duas fontes, e a busca de fase custa os **5,9 ms** que o `Plan` foi inventado para matar.
///
/// **Mutação que deve sangrar:** `pair()` devolver `(current, current)` em repouso.
#[test]
fn arriving_never_rebuilds_the_pair() {
    let g = graph();
    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    frames(&mut m, &g, 120);
    assert!(!m.is_flying());
    assert_eq!(m.pair(), (A, B), "o par tem de FICAR o do voo que acabou");
    assert_eq!(m.t(), 1.0, "e o t saturado, que ja' e' a forma B");
    let before = (m.pair(), m.t(), m.current());
    frames(&mut m, &g, 10);
    assert_eq!((m.pair(), m.t(), m.current()), before);
}

/// ⚠️ **Uma transição INSTANTÂNEA chega, em vez de dividir por zero.** Um corte é uma escolha
/// legítima do artista, e recusá-la faria o slider ter um valor proibido no meio da faixa.
#[test]
fn a_cut_arrives_instead_of_dividing_by_zero() {
    let mut g = graph();
    g.states[1].duration_s = 0.0;
    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    frames(&mut m, &g, 1);
    assert!(!m.is_flying());
    assert_eq!((m.pair(), m.t()), ((A, B), 1.0));
    assert!(m.t().is_finite(), "e o t nunca pode ser NaN");
}

/// **A MOLA também chega** — e assenta sem duração, que é a razão de ela existir.
#[test]
fn the_spring_settles_and_lands_exactly_on_the_shape() {
    let mut g = graph();
    g.states[1].spring = Some(ph2d_spring::Spring::default());
    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    frames(&mut m, &g, 600);
    assert!(!m.is_flying(), "a mola nunca assentou em 10 s");
    assert_eq!(m.t(), 1.0, "assentar poe a forma EXACTA, nunca 0,997");
}

/// ⭐ **QUEM LÊ O MEU INPUT** — a correcção nº 2 da pesquisa, a cura do medo do Animator.
///
/// ⚠️ **Sob o modelo por-forma são TODAS menos a de onde já se está**, e é essa a resposta certa:
/// a tecla que leva à forma corrente não faz nada, e listá-la prometeria um efeito que não
/// acontece.
///
/// **Mutação que deve sangrar:** `live_actions` largar o `s.shape != self.current` — o painel
/// passaria a prometer, em B, uma tecla que em B não faz nada.
#[test]
fn live_actions_names_only_what_does_something_from_here() {
    let g = graph();
    let mut m = MorphMachine::new(&g);
    assert_eq!(m.live_actions(&g), vec!["jump", "dash"]);
    m.fire(&g, "jump");
    frames(&mut m, &g, 60);
    assert_eq!(
        m.live_actions(&g),
        vec!["dash"],
        "em B o jump ja' nao faz nada -- ele leva a B"
    );
    m.fire(&g, "dash");
    frames(&mut m, &g, 60);
    assert_eq!(m.live_actions(&g), vec!["jump"], "e em C e' o inverso");
}

/// **A porta da PRÉ-VISUALIZAÇÃO ignora a condição — mas não a forma corrente.**
///
/// ⚠️ Ela existe porque o artista tem de poder VER a forma antes de lhe dar tecla; sem ela, um
/// estado sem condição seria indemonstrável.
///
/// **Mutação que deve sangrar:** o `travel` largar a guarda `st.shape != self.current` — o botão
/// do painel poria a máquina a voar de uma forma para ela própria.
#[test]
fn preview_travels_to_a_shape_without_its_condition() {
    let g = MorphGraph {
        states: vec![MorphState::new(A), MorphState::new(B), MorphState::new(C)],
    };
    let mut m = MorphMachine::new(&g);
    assert!(!m.travel(&g, 0), "ja' estamos em A");
    assert!(m.travel(&g, 1), "e nenhuma delas tem condicao");
    assert_eq!(m.current(), B);
    assert!(
        !m.travel(&g, 9),
        "um indice que nao existe recusa em vez de entrar em panico"
    );
}

/// **As formas são a própria lista, na ordem em que o artista as escolheu.**
///
/// ⚠️ E o `start` é **derivado** dela — um campo ao lado podia apontar para uma forma que a lista
/// não tem, e essa discordância passa muda por uma fusão.
#[test]
fn the_shapes_are_the_list_and_the_start_is_the_first() {
    assert_eq!(graph().shapes(), vec![A, B, C]);
    assert_eq!(graph().start(), Some(A));
    let empty = MorphGraph::default();
    assert!(empty.shapes().is_empty());
    assert_eq!(empty.start(), None, "sem formas nao ha' onde nascer");
    // ⛔ E uma máquina sobre a lista vazia é INERTE, nunca um pânico.
    let mut m = MorphMachine::new(&empty);
    assert!(!m.fire(&empty, "jump"));
    assert!(!m.travel(&empty, 0));
}

/// **A máquina autorada atravessa o ficheiro** — ela é conteúdo autorado.
#[test]
fn the_graph_survives_the_round_trip() {
    let g = graph();
    let bytes = postcard_like(&g);
    let back: MorphGraph = serde_json::from_str(&bytes).expect("volta");
    assert_eq!(back, g);
}

/// O `.ph2dproj` é postcard, mas o gate aqui só precisa de provar que os derives existem e casam —
/// e o `serde_json` é `dev-dependency`, então não alcança consumidor nenhum.
fn postcard_like(g: &MorphGraph) -> String {
    serde_json::to_string(g).expect("serializa")
}
