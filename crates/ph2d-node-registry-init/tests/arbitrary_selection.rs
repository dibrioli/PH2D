//! **A SELEÇÃO ARBITRÁRIA JÁ É EXPRIMÍVEL, e a folha 08 diz que não é.**
//!
//! O P0 dela — *"predicado / SELEÇÃO arbitrária"* no `motion.cull` — traz a cadeia
//! tentada: `motion.expression(pred) → ?`, com o veredito *"morre no §0"* (nada
//! escreve coluna nomeada) e a nota *"a cura não é um param no `cull`, é o
//! escritor do §0"*.
//!
//! ⚠️ **A cadeia morria por um elo, não por dois:** a `motion.expression` produz
//! um VALOR por elemento e o que faltava era onde pousá-lo — e o `motion.drive`
//! **escreve o canal `Falloff`** (o 6º da lista dele) em modo `Set`, que é
//! exatamente a coluna que o `cull(mode = Falloff)` compara. Três nós, todos já
//! shipados, e nenhum escritor de coluna nomeada a construir:
//!
//! ```text
//! motion.expression("<predicado>")  ->  motion.drive(Falloff, Set)  ->  motion.cull(Falloff, 0.5)
//! ```
//!
//! Este arquivo é a MEDIÇÃO dessa afirmação, não um argumento sobre ela: monta a
//! cadeia pela porta do cook e conta o que sobrevive. Se algum dia um dos três
//! elos mudar de forma, isto fica vermelho e a refutação da folha volta a ser uma
//! pergunta em aberto, em vez de uma frase que ninguém reconfere.
//!
//! ⚠️ **E a medição achou o que faltava DE VERDADE, que era outra coisa:** o
//! vocabulário da fórmula é `i`/`n`/`t`/`f` mais qualquer coluna **ESCALAR**, e
//! `P` é `Vec2` — então um predicado sobre a POSIÇÃO, o primeiro que qualquer um
//! escreve, avaliava **silenciosamente a zero**. As lanes `x`/`y` fecham isso, e
//! sem elas esta refutação teria um buraco exactamente onde o artista olha
//! primeiro. (Medido de passagem: `%` não existe no parser — `i % 2` devolve a
//! coluna toda zerada.)

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// O canal `Falloff` do `motion.drive` (o 6º rótulo, índice 5) e o modo `Set`
/// (o 2º, índice 1) — os dois números que fazem a cadeia existir.
const DRIVE_CHANNEL_FALLOFF: f32 = 5.0;
const DRIVE_MODE_SET: f32 = 1.0;
/// O modo `Falloff` do `motion.cull` (índice 1).
const CULL_MODE_FALLOFF: f32 = 1.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Monta `grid → expression(pred) → drive(Falloff, Set) → cull(Falloff, 0.5)` e
/// devolve as posições que sobreviveram.
fn survivors(pred: &str) -> Vec<[f32; 2]> {
    let reg = registry();
    let mut g = Graph::new();

    // Uma grade 5x1: cinco elementos com `x` distinto, para um predicado sobre a
    // posição ter algo que distinguir.
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 5.0);
    g.set_param(grid, "gap_x", 1.0);

    // O PREDICADO, escrito pelo artista como fórmula.
    let expr = g.add_node("motion.expression");
    g.set_text_param(expr, "expr", pred.to_string());
    g.connect(Edge {
        from: (grid, 0),
        to: (expr, 0),
        delayed: false,
    })
    .expect("expression in");

    // ⚠️ O ELO QUE A FOLHA NÃO TENTOU: o valor vira a coluna `falloff`.
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", DRIVE_CHANNEL_FALLOFF);
    g.set_param(drive, "mode", DRIVE_MODE_SET);
    g.connect(Edge {
        from: (grid, 0),
        to: (drive, 0),
        delayed: false,
    })
    .expect("drive in");
    g.connect(Edge {
        from: (expr, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("drive value");

    let cull = g.add_node("motion.cull");
    g.set_param(cull, "mode", CULL_MODE_FALLOFF);
    g.set_param(cull, "amount", 0.5);
    g.connect(Edge {
        from: (drive, 0),
        to: (cull, 0),
        delayed: false,
    })
    .expect("cull in");

    g.validate(&reg).expect("a cadeia inteira e bem-tipada");
    positions(&g, &reg, cull)
}

fn positions(g: &Graph, reg: &NodeRegistry, node: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, node, 0.0).expect("cook");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    match Stream::get(s, "P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **O predicado arbitrário CULLA, e a folha 08 tem um P0 a menos.**
///
/// A fórmula não é um dos modos que o `cull` conhece — é aritmética por elemento,
/// e é exactamente isso que a `Fraction` (contagem líder) e a `Falloff` de uma
/// FORMA espacial não conseguiam exprimir.
#[test]
fn an_arbitrary_formula_selects_which_elements_survive() {
    // Cinco colunas com gap 1, centradas: os `x` são -2, -1, 0, 1, 2.
    let all = survivors("1");
    assert_eq!(all.len(), 5, "o predicado constante 1 nao culla ninguem");

    // Fica quem esta a DIREITA do centro — um predicado sobre a POSIÇÃO.
    let right = survivors("x > 0");
    assert_eq!(right.len(), 2, "as duas da direita sobrevivem: {right:?}");
    assert!(
        right.iter().all(|p| p[0] > 0.0),
        "e sao as da direita, nao duas quaisquer: {right:?}"
    );

    // O COMPLEMENTAR devolve o resto — a prova de que quem decide e a formula, e
    // nao alguma ordem de lista que so parecia o predicado.
    let left = survivors("x < 0");
    assert_eq!(left.len(), 2, "e as duas da esquerda: {left:?}");
    assert!(left.iter().all(|p| p[0] < 0.0), "{left:?}");

    // ⚠️ O caso DECISIVO: um predicado NÃO-MONOTÔNICO na ordem da lista — só as
    // pontas, com o meio removido. Nenhuma regra por CONTAGEM produz este
    // conjunto, e e por isso que ele e a refutacao e nao os dois de cima.
    let ends = survivors("abs(x) > 1.5");
    assert_eq!(ends.len(), 2, "as duas pontas: {ends:?}");
    assert!(
        ends.iter().all(|p| p[0].abs() > 1.5),
        "e sao as pontas, com o meio removido: {ends:?}"
    );

    // E um predicado sobre o ÍNDICE, que e o outro eixo do vocabulario.
    let odd = survivors("i > 2");
    assert_eq!(odd.len(), 2, "os dois ultimos por indice: {odd:?}");
}

/// **A posição chega à fórmula, e ela é o motivo de a refutação valer.**
///
/// ⚠️ Antes disto o vocabulário só via colunas ESCALARES, e `P` é `Vec2` — então
/// `x > 0` não era um predicado falso, era um predicado **mudo**: a coluna inteira
/// saía zero e o `cull` removia tudo, sem erro e sem aviso. Um gate que só usasse
/// `i` ficaria verde sobre exactamente o predicado que um artista escreve primeiro.
#[test]
fn a_formula_can_read_where_the_element_is() {
    // Sem as lanes, TODOS os cinco eram cullados por `x > 0` (a coluna zerada nao
    // alcanca o limiar 0.5), e o gate acima passaria com `len() == 0` se a barra
    // fosse "algum foi removido" em vez do CONJUNTO.
    let right = survivors("x > 0");
    assert!(!right.is_empty(), "uma coluna muda cullaria TODOS");

    // As duas lanes sao distintas: um predicado em `y` sobre uma linha (todo
    // `y = 0`) nao pode coincidir com um em `x`.
    assert_eq!(
        survivors("y > 0").len(),
        0,
        "nesta fila todo y e zero, entao o predicado em y nao guarda ninguem"
    );
    assert_eq!(survivors("y < 1").len(), 5, "e o complementar guarda todos");
}

/// **O elo que a folha declarou ausente EXISTE, e este gate o nomeia sozinho.**
///
/// Sem ele o teste acima ainda poderia passar por alguma outra rota, e a lição
/// ficaria escrita no lugar errado. Aqui o `drive` é dirigido por um valor e o que
/// se lê é a COLUNA que ele escreveu — a peça que o §0 dava por inexistente.
#[test]
fn motion_drive_writes_the_named_column_a_value_field_carries() {
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.0);

    let expr = g.add_node("motion.expression");
    // Um valor por elemento que nenhuma forma espacial produziria: uma rampa
    // pelo indice, normalizada. ⚠️ NAO `i % 2` — medido, o parser nao tem `%`, e
    // a formula cai no campo zerado sem dizer nada, que era como este proprio
    // gate nasceu verde-sobre-errado na primeira escrita.
    g.set_text_param(expr, "expr", "i * 0.25".to_string());
    g.connect(Edge {
        from: (grid, 0),
        to: (expr, 0),
        delayed: false,
    })
    .expect("expression in");

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", DRIVE_CHANNEL_FALLOFF);
    g.set_param(drive, "mode", DRIVE_MODE_SET);
    g.connect(Edge {
        from: (grid, 0),
        to: (drive, 0),
        delayed: false,
    })
    .expect("drive in");
    g.connect(Edge {
        from: (expr, 0),
        to: (drive, 1),
        delayed: false,
    })
    .expect("drive value");
    g.validate(&reg).expect("bem-tipada");

    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, drive, 0.0).expect("cook");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    let Some(Column::Scalar(f)) = Stream::get(s, "falloff") else {
        panic!("o drive escreveu a coluna `falloff`")
    };
    assert_eq!(
        f,
        &vec![0.0, 0.25, 0.5, 0.75],
        "a coluna carrega o valor POR ELEMENTO que a formula produziu"
    );
}
