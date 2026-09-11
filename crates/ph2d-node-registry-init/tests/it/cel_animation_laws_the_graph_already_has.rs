//! **DUAS DAS TRÊS LEIS DE ANIMAÇÃO QUADRO-A-QUADRO JÁ EXISTEM** — medido, não opinado.
//!
//! # Por que este ficheiro é um gate e não uma nota
//!
//! O plano [93](../../../docs/Motion%20Nodes/93_plano_lsystem_datasource_celanim.md) §3
//! nomeou três coisas em falta no `motion.sub_uv` para ele ser a *cel animation* da casa:
//! **inverso**, **ping-pong** e **tocar uma vez**. A medição de 2026-08-28 refuta duas:
//!
//! | lei | o que o plano dizia | o que a medição diz |
//! |---|---|---|
//! | **inverso** | falta | ⛔ o `speed` do `sub_uv` **já vai a negativo** (`min: -MAX_CELL_SPEED`). Zero nós |
//! | **ping-pong** | falta | ⛔ o `value.wrap` **já tem `Mirror`** (`MirroredRepeat`, período `2w`). Um nó |
//! | **tocar uma vez** | falta | ⛔ o `value.wrap` **já tem `Clamp`** — segura na última célula. Um nó |
//! | duração desigual por quadro | falta | ⏳ **essa falta mesmo** — nada mapeia `t` numa escada não-uniforme sem autorar ponto a ponto |
//!
//! ⇒ Construir `direction` e `play` como params do `sub_uv` seria pôr no painel botões que
//! **o app já tem** — o que a §5.0 do `CLAUDE.md` proíbe pelo nome: *"antes de construir um
//! item de lista aberta, MEÇA se a composição já o exprime"*.
//!
//! # A régua é a SEQUÊNCIA DE CÉLULAS, não a coluna
//!
//! Cada gate coze o grafo em vários instantes e lê a célula que a `uv_cell` nomeia. Uma
//! afirmação sobre a coluna sozinha ficaria verde com um `sub_uv` que escrevesse qualquer
//! coisa; o que se mede é **que célula da folha o artista vê**, ao longo do tempo, que é a
//! pergunta que a feature faz.
//!
//! ⚠️ Este ficheiro é a **prova** que sustenta uma recusa medida. Quem quiser reabrir o item
//! começa por o correr, e argumenta contra os números.

use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Quantas células a folha de teste tem — uma tira de 6, como um ciclo de passo.
const CELLS: u32 = 6;

/// UM elemento na origem — a fixtura mais barata que tem cardinalidade.
///
/// ⚠️ **`motion.grid` e não `motion.make_point`**: o `make_point` sem entrada nem campos
/// ligados emite ZERO elementos (a cardinalidade dele vem de fora), e uma folha de sprite
/// sobre um stream vazio não tem célula nenhuma para mostrar — o gate morria a ler o
/// elemento `0` de uma coluna de comprimento `0`, que é a fixtura a estar errada e não o app.
fn one_element(g: &mut Graph) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", 1.0);
    g.set_param(n, "cols", 1.0);
    n
}

fn registry() -> ph2d_node_registry::NodeRegistry {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16) {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .expect("a ligacao e' de tipos compativeis");
}

/// A célula que o elemento `0` mostra, lida da `uv_cell` que o `sub_uv` escreve.
///
/// A coluna é `[escala_u, escala_v, desloc_u, desloc_v]`; numa tira de `CELLS × 1` a célula é
/// `desloc_u / escala_u`, arredondada — a conta inversa da que o nó fez.
///
/// ⚠️ **O cozinhador vem de FORA e é reusado.** Um `Cook::new()` por instante nunca devolve
/// nada de velho, e foi por isso que a 1.ª redacção destes quatro ficou cega ao
/// `motion.sub_uv` estar **congelado** por se declarar `Pure` a ler o relógio (2026-08-28).
/// *A régua tem de ser a do app, e o app reusa o cozinhador.*
fn cell_at(
    g: &Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    sink: NodeId,
    cook: &mut Cook,
    t: f64,
) -> i32 {
    let CookValue::Instances(s) = &cook.cook(g, reg, sink, t).expect("coze")[0] else {
        panic!("instancias")
    };
    let Some(Column::Vec4(v)) = s.get(ph2d_node_motion_sub_uv::CELL_COLUMN) else {
        panic!("o sub_uv tem de escrever a uv_cell")
    };
    (v[0][2] / v[0][0]).round() as i32
}

/// Uma tira de `CELLS` células sobre um elemento, com o `speed` dado. Devolve `(grafo, sink)`.
fn strip(speed: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = one_element(&mut g);
    let suv = g.add_node("motion.sub_uv");
    g.set_param(suv, "cols", CELLS as f32);
    g.set_param(suv, "rows", 1.0);
    g.set_param(suv, "speed", speed);
    wire(&mut g, src, 0, suv, 0);
    (g, suv)
}

/// A sequência de células nos `n` primeiros passos de meio segundo.
/// ⚠️ **UM cozinhador para a caminhada inteira** — é a régua do APP, e a única que vê um memo
/// a devolver um stream velho.
fn walk(g: &Graph, reg: &ph2d_node_registry::NodeRegistry, sink: NodeId, n: usize) -> Vec<i32> {
    let mut cook = Cook::new();
    (0..n)
        .map(|k| cell_at(g, reg, sink, &mut cook, k as f64 * 0.5))
        .collect()
}

/// ⛔ **O INVERSO JÁ EXISTE, e custa ZERO nós**: `speed` negativo.
///
/// O controlo é o mesmo grafo com `speed` positivo — a sequência tem de ser a inversa. Sem
/// ele, um `sub_uv` que ignorasse o sinal daria as duas iguais e o gate ficaria verde a
/// afirmar que uma feature em falta já existe, que é o pior tipo de verde que há.
#[test]
fn playing_backwards_is_already_a_knob_the_node_has() {
    let reg = registry();
    let (fwd, a) = strip(2.0);
    let (rev, b) = strip(-2.0);
    let f = walk(&fwd, &reg, a, 7);
    let r = walk(&rev, &reg, b, 7);
    assert_eq!(f, vec![0, 1, 2, 3, 4, 5, 0], "para a frente: {f:?}");
    assert_eq!(r, vec![0, 5, 4, 3, 2, 1, 0], "para tras: {r:?}");
}

/// ⛔ **O PING-PONG JÁ EXISTE, e custa UM nó**: `value.wrap` no modo `Mirror`.
///
/// A porta `cell` do `sub_uv` aceita um `value.*` por elemento, e o `Mirror` é o
/// `MirroredRepeat` — uma triangular de período `2w`, que é exactamente *ping-pong sem
/// repetir as pontas*.
#[test]
fn ping_pong_is_already_one_existing_node_away() {
    let reg = registry();
    let mut g = Graph::new();
    let src = one_element(&mut g);
    let clock = g.add_node("value.time");
    g.set_param(clock, "rate", 2.0);
    let wrap = g.add_node("value.wrap");
    g.set_param(wrap, "lo", 0.0);
    // ⚠️ `CELLS - 1`: a triangular sobe até `hi` e desce, então o cume é a ÚLTIMA célula.
    g.set_param(wrap, "hi", (CELLS - 1) as f32);
    g.set_param(wrap, "mode", 2.0); // Mirror
    let suv = g.add_node("motion.sub_uv");
    g.set_param(suv, "cols", CELLS as f32);
    g.set_param(suv, "rows", 1.0);
    wire(&mut g, src, 0, clock, 0);
    wire(&mut g, clock, 0, wrap, 0);
    wire(&mut g, src, 0, suv, 0);
    wire(&mut g, wrap, 0, suv, 1);

    let seq = walk(&g, &reg, suv, 11);
    assert_eq!(
        seq,
        vec![0, 1, 2, 3, 4, 5, 4, 3, 2, 1, 0],
        "sobe e desce SEM repetir as pontas: {seq:?}"
    );
}

/// ⛔ **TOCAR UMA VEZ JÁ EXISTE, e é o mesmo nó noutro modo**: `value.wrap` em `Clamp`.
///
/// E ele pára onde a §11 do Sprite diz que tem de parar — **na última célula**, não na
/// primeira.
#[test]
fn playing_once_and_holding_the_last_cell_is_the_same_node_in_another_mode() {
    let reg = registry();
    let mut g = Graph::new();
    let src = one_element(&mut g);
    let clock = g.add_node("value.time");
    g.set_param(clock, "rate", 2.0);
    let wrap = g.add_node("value.wrap");
    g.set_param(wrap, "lo", 0.0);
    g.set_param(wrap, "hi", (CELLS - 1) as f32);
    g.set_param(wrap, "mode", 0.0); // Clamp
    let suv = g.add_node("motion.sub_uv");
    g.set_param(suv, "cols", CELLS as f32);
    g.set_param(suv, "rows", 1.0);
    wire(&mut g, src, 0, clock, 0);
    wire(&mut g, clock, 0, wrap, 0);
    wire(&mut g, src, 0, suv, 0);
    wire(&mut g, wrap, 0, suv, 1);

    let seq = walk(&g, &reg, suv, 9);
    assert_eq!(
        seq,
        vec![0, 1, 2, 3, 4, 5, 5, 5, 5],
        "toca uma vez e SEGURA na ultima: {seq:?}"
    );
}

/// ⏳ **A que falta MESMO: a duração desigual por quadro.**
///
/// Este gate não afirma que é impossível — afirma que a rota que existe é **uniforme por
/// construção**, que é o que torna o item legítimo em vez de já-feito. Com `speed` constante
/// cada célula dura exactamente o mesmo, e nenhum dos três modos do `value.wrap` muda isso:
/// eles dobram e cortam o eixo, nunca o esticam por troços.
///
/// ⚠️ A rota que *tecnicamente* o exprimiria é um `field.remap` com uma curva de degraus
/// `Hold` autorada ponto a ponto — o que quer dizer desenhar oito pontos num editor de
/// quadrado unitário para dizer *"o terceiro quadro dura o triplo"*. Exprimível não é
/// alcançável, que é a frase que abriu o `value.number`.
#[test]
fn unequal_frame_duration_is_the_one_that_is_genuinely_missing() {
    let reg = registry();
    let (g, sink) = strip(2.0);
    // Amostrado FINO: cada célula tem de ocupar exactamente a mesma fatia de tempo.
    let mut fine_cook = Cook::new();
    let fine: Vec<i32> = (0..60)
        .map(|k| cell_at(&g, &reg, sink, &mut fine_cook, k as f64 * 0.05))
        .collect();
    let mut runs = Vec::new();
    let mut cur = (fine[0], 0usize);
    for c in &fine {
        if *c == cur.0 {
            cur.1 += 1;
        } else {
            runs.push(cur.1);
            cur = (*c, 1);
        }
    }
    let inner = &runs[1..runs.len().saturating_sub(1)];
    assert!(
        !inner.is_empty(),
        "a varredura tem de cobrir varias celulas"
    );
    assert!(
        inner.iter().all(|r| *r == inner[0]),
        "a rota que existe e' UNIFORME por construcao — todas as celulas duram o mesmo: {runs:?}"
    );
}
