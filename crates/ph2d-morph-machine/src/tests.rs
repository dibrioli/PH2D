//! Os gates da lei — a metade que o **runtime do jogo** vai correr.

use super::*;

const A: ShapeId = 10;
const B: ShapeId = 20;
const C: ShapeId = 30;

/// `A --jump--> B --dash--> C`, e um atalho `A --dash--> C` para provar a ordem.
fn graph() -> MorphGraph {
    let mut g = MorphGraph {
        start: A,
        edges: Vec::new(),
    };
    for (from, to, when) in [(A, B, "jump"), (B, C, "dash"), (A, C, "dash")] {
        let mut e = MorphEdge::new(from, to);
        e.when = when.to_string();
        g.edges.push(e);
    }
    g
}

/// Anda `n` quadros de 60 fps.
fn frames(m: &mut MorphMachine, g: &MorphGraph, n: usize) {
    for _ in 0..n {
        m.advance(g, 1.0 / 60.0);
    }
}

/// ⭐ **SÓ AS SETAS DO ESTADO CORRENTE.** A correcção nº 1 da pesquisa (State Tree).
///
/// **Mutação que deve sangrar:** `MorphGraph::from` varrer `edges` inteiro em vez de filtrar por
/// `from` — o `dash` em `A` passaria a poder disparar a seta `B --dash--> C`, e a máquina saltava
/// para um estado que nenhuma seta do sítio onde ela está alcança. É a teia que os utilizadores do
/// Animator descrevem, construída por acidente.
#[test]
fn only_the_current_states_arrows_can_fire() {
    let g = graph();
    let mut m = MorphMachine::new(&g);
    // Em `A`, o `dash` existe — mas é o atalho `A -> C`, e nunca o `B -> C`.
    assert!(m.fire(&g, "dash"));
    assert_eq!(m.current(), C, "o dash em A tem de tomar o atalho A->C");

    // Em `C` nao ha' seta nenhuma: nada dispara.
    let mut m = MorphMachine::new(&g);
    assert!(m.fire(&g, "jump"));
    frames(&mut m, &g, 60);
    assert_eq!(m.current(), B);
    assert!(
        !m.fire(&g, "jump"),
        "o `jump` nao parte de B -- nao ha' seta"
    );
    assert_eq!(m.current(), B, "e um disparo recusado nao pode mover nada");
}

/// ⛔ **Uma seta SEM CONDIÇÃO não dispara sozinha.**
///
/// **Mutação que deve sangrar:** `matches` largar o `!self.when.is_empty()` — toda seta
/// recém-desenhada passaria a responder a uma acção de nome vazio, e a forma saltava sem ninguém
/// ter pedido.
#[test]
fn an_arrow_without_a_condition_never_fires_by_itself() {
    let mut g = MorphGraph {
        start: A,
        edges: vec![MorphEdge::new(A, B)],
    };
    let mut m = MorphMachine::new(&g);
    assert!(!m.fire(&g, ""), "o nome vazio nao pode casar");
    assert!(!m.fire(&g, "jump"));
    assert_eq!(m.current(), A);
    // O CONTROLE: com nome, a MESMA seta dispara.
    g.edges[0].when = "jump".to_string();
    assert!(m.fire(&g, "jump"));
    assert_eq!(m.current(), B);
}

/// ⭐ **O `current` SALTA NO LANÇAMENTO** — a meio de `A→B`, as setas que se oferecem são as de `B`.
///
/// **Mutação que deve sangrar:** `current` passar a mudar só na chegada. Encadear `jump` e depois
/// `dash` durante o voo deixaria de casar com seta nenhuma, e o segundo input **desaparecia** — o
/// defeito é silencioso, que é o pior tipo.
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
        m.fire(&g, "dash"),
        "as setas de B tem de estar disponiveis em voo"
    );
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
/// **Mutação que deve sangrar:** o `take_edge` chamar `launch` mesmo em voo — o par trocaria a meio
/// e a forma **saltava** visivelmente.
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
/// máquina ainda não estava — ele não casava com seta nenhuma e o input do jogador **desaparecia
/// em silêncio**. Pior: se casasse, o `launch` poria o par `(from, to)` de uma seta que não parte
/// de onde a máquina aterra, **saltando um estado inteiro**. A cura foi mover o salto do `current`
/// para o lançamento — e é ela que torna "o mais novo ganha" seguro *por construção*, porque todo
/// candidato à fila parte do MESMO sítio.
///
/// **Mutação que deve sangrar:** trocar o `pending` por um `Vec` que empilha.
#[test]
fn the_newest_request_wins_and_the_queue_never_grows() {
    let mut g = graph();
    // Mais uma saida de B, para haver DOIS pedidos possiveis em voo.
    let mut back = MorphEdge::new(B, A);
    back.when = "back".to_string();
    g.edges.push(back);

    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    frames(&mut m, &g, 2);
    m.fire(&g, "dash"); // pede B->C
    m.fire(&g, "back"); // e logo a seguir B->A: este e' o que vale
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
/// Trocar o par ao chegar rebuildaria por nada, todo quadro de chegada.
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
    // E um quadro a mais, parado, nao pode mexer em nada.
    let before = (m.pair(), m.t(), m.current());
    frames(&mut m, &g, 10);
    assert_eq!((m.pair(), m.t(), m.current()), before);
}

/// ⚠️ **Uma seta INSTANTÂNEA chega, em vez de dividir por zero.** Um corte é uma escolha legítima
/// do artista, e recusá-la faria o slider ter um valor proibido no meio da faixa.
#[test]
fn a_cut_arrives_instead_of_dividing_by_zero() {
    let mut g = graph();
    g.edges[0].duration_s = 0.0;
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
    g.edges[0].spring = Some(ph2d_spring::Spring::default());
    let mut m = MorphMachine::new(&g);
    m.fire(&g, "jump");
    // Generoso de proposito: o gate mede que ela ASSENTA, nao quanto tempo leva.
    frames(&mut m, &g, 600);
    assert!(!m.is_flying(), "a mola nunca assentou em 10 s");
    assert_eq!(m.t(), 1.0, "assentar poe a forma EXACTA, nunca 0,997");
}

/// ⭐ **QUEM LÊ O MEU INPUT** — a correcção nº 2 da pesquisa, a cura do medo do Animator.
///
/// **Mutação que deve sangrar:** `live_actions` varrer o grafo inteiro — o painel passaria a
/// prometer, no estado `A`, uma acção que só faz alguma coisa em `B`.
#[test]
fn live_actions_names_only_what_does_something_from_here() {
    let g = graph();
    let mut m = MorphMachine::new(&g);
    assert_eq!(m.live_actions(&g), vec!["jump", "dash"]);
    m.fire(&g, "jump");
    frames(&mut m, &g, 60);
    assert_eq!(m.live_actions(&g), vec!["dash"], "de B so' o dash faz algo");
    m.fire(&g, "dash");
    frames(&mut m, &g, 60);
    assert!(m.live_actions(&g).is_empty(), "C nao tem saida");
}

/// **A porta da PRÉ-VISUALIZAÇÃO ignora a condição — mas não o estado corrente.**
///
/// ⚠️ Ela existe porque o artista tem de poder VER a seta que acabou de desenhar antes de lhe dar
/// nome; sem ela, uma seta sem condição seria indemonstrável.
///
/// **Mutação que deve sangrar:** o `travel` largar a guarda `e.from != self.current` — o botão do
/// painel passaria a pôr a máquina num par que não parte de onde ela está.
#[test]
fn preview_travels_an_arrow_without_its_condition() {
    let g = MorphGraph {
        start: A,
        edges: vec![MorphEdge::new(A, B), MorphEdge::new(B, C)],
    };
    let mut m = MorphMachine::new(&g);
    assert!(
        !m.travel(&g, 1),
        "a seta 1 parte de B, e a maquina esta' em A"
    );
    assert!(m.travel(&g, 0));
    assert_eq!(m.current(), B);
    assert!(
        !m.travel(&g, 9),
        "um indice que nao existe recusa em vez de entrar em panico"
    );
}

/// **As formas do grafo são DERIVADAS das setas**, com o `start` à frente.
#[test]
fn the_shapes_are_derived_from_the_arrows() {
    assert_eq!(graph().shapes(), vec![A, B, C]);
    let empty = MorphGraph {
        start: A,
        edges: Vec::new(),
    };
    assert_eq!(
        empty.shapes(),
        vec![A],
        "uma maquina de uma forma so' e' valida"
    );
}

/// **O grafo atravessa o ficheiro** — ele é conteúdo autorado.
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
