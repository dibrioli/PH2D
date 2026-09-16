//! ⭐⭐⭐ **Smoke do CÉREBRO AUTORÁVEL** (TOP-20 #15, W4). `PH2D_STATEMACHINE_SMOKE=1`.
//!
//! # A PORTA, e o CONTROLO ao lado dela
//!
//! Duas portas idênticas, lado a lado, a ouvir **o mesmo sinal** (`botao`, que um relógio publica
//! de dois em dois segundos):
//!
//! | | o que tem | o que FAZ |
//! |---|---|---|
//! | **esquerda — «Door»** | uma **máquina de estados** de 3 estados | `Fechada → A abrir → Aberta → Fechada`, um passo por toque |
//! | **direita — «Door (no brain)»** | só a tabela de acções, com as MESMAS duas linhas | fica **sempre escondida**: o mesmo sinal manda mostrar E esconder, e a última ganha |
//!
//! # ⛔⛔ Cada porta é um CORPO com uma FAIXA de sinal por cima — e a partição é load-bearing
//!
//! O cérebro mora no **corpo** (o batente escuro), que é a parte grande e a que o artista carrega
//! para o escolher. A cor vive numa **faixa** por cima dele — três placas empilhadas, uma visível —,
//! e a faixa **não cobre o corpo**: elas encostam e não se sobrepõem.
//!
//! ⚠️ **Isto não é decoração, é o que torna a cena ALCANÇÁVEL** (report do dono, 2026-09-15:
//! *«não apareceu no painel a seção state machine»*). Antes, a entidade com o cérebro **não tinha
//! `Sprite` nenhum**: ela não emitia `RenderInstance`, logo o
//! [`ph2d_render::pick_sprite_at_world`] nunca a devolvia, e um clique na porta escolhia uma das
//! placas — que não tem cérebro. *A cena estava certa como DADOS e era impossível como GESTO*, e o
//! Inspector mostrava, correctamente, a verdade sobre o objecto escolhido.
//!
//! ⛔ E elas não se sobrepõem **por causa do desempate**: o pick devolve *«o último da ordem de
//! iteração»*, que **entre arquétipos diferentes é indefinido** (o corpo carrega `StateMachine`, a
//! placa carrega `Visibility` ⇒ arquétipos diferentes). Encostadas, não há desempate nenhum a fazer.
//!
//! ⭐⭐⭐ **É o CONTROLO que torna a wave legível.** Sem ele, a porta da esquerda parece uma porta
//! que abre — com ele, vê-se *o que um estado compra*: **escolher UMA coisa de cada vez**. É
//! exactamente o que a sonda do §5.0 mediu antes da primeira linha de código (duas linhas
//! contraditórias disparam as duas, e nada escolhe uma).
//!
//! # O que tem de acontecer
//!
//! A faixa da porta da **esquerda** muda de cor num ciclo de três: **vermelha** (fechada) →
//! **amarela** (a abrir) → **verde** (aberta) → vermelha outra vez. A da **direita** fica na mesma
//! cor o tempo todo.
//!
//! ⚠️ **Carregue no CORPO da porta da esquerda** (o batente escuro por baixo da faixa) **e olhe o
//! painel:** a secção *State Machine* diz **`Now: …`** — o estado corrente, a andar com a cena.
//!
//! ⚠️ Se a linha `[statemachine-smoke]` não aparecer, **PARE**: a cena não montou.
//!
//! # ⚠️ Porque as cores, e não um movimento
//!
//! Um estado é uma coisa que não se vê. Mover a porta mostraria a ACÇÃO (que é do #5) e esconderia
//! o ESTADO; três cores num objecto só mostram a máquina a andar, que é o sujeito desta wave. ⛔ E
//! por isso os três `on_enter` são nomes **diferentes**: cada um acende uma linha da tabela.

use ph2d_core::Vec2;
use ph2d_ecs::{
    MachineState, Name, SignalAction, SignalActions, SignalVerb, StateMachine, StateTransition,
    Timer, Timers, Transform, Visibility, World,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 1;

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const PORTA_RGBA: [f32; 4] = [0.30, 0.33, 0.40, 1.0];
const FECHADA_RGBA: [f32; 4] = [0.85, 0.30, 0.28, 1.0];
const A_ABRIR_RGBA: [f32; 4] = [0.92, 0.76, 0.26, 1.0];
const ABERTA_RGBA: [f32; 4] = [0.36, 0.78, 0.44, 1.0];

/// ⭐ **A geometria de uma porta, num sítio só** — o corpo em baixo, a faixa em cima, encostados.
///
/// ⚠️ **Elas TOCAM-SE e não se sobrepõem**, e a aritmética tem de o dizer: o [`JUNTA_Y`] é ao mesmo
/// tempo o topo do corpo e o fundo da faixa. Escrever as duas caixas à mão seria a segunda resposta à mesma pergunta,
/// e o ponto que o gate carrega deixaria de ser o ponto que o dono carrega.
mod porta {
    /// Meia-largura das duas peças.
    pub(super) const MEIA_LARGURA: f32 = 1.3;
    /// O `y` onde o corpo acaba e a faixa começa.
    pub(super) const JUNTA_Y: f32 = 0.8;
    /// O `y` do fundo do corpo.
    pub(super) const BASE_Y: f32 = -2.2;
    /// O `y` do topo da faixa.
    pub(super) const TOPO_Y: f32 = 2.2;

    /// `(tamanho, centro_y)` do CORPO — a peça que carrega o cérebro e o que o dedo apanha.
    pub(super) const fn corpo() -> ([f32; 2], f32) {
        (
            [MEIA_LARGURA * 2.0, JUNTA_Y - BASE_Y],
            (BASE_Y + JUNTA_Y) * 0.5,
        )
    }

    /// `(tamanho, centro_y)` da FAIXA — a peça que a máquina acende.
    pub(super) const fn faixa() -> ([f32; 2], f32) {
        (
            [MEIA_LARGURA * 2.0, TOPO_Y - JUNTA_Y],
            (JUNTA_Y + TOPO_Y) * 0.5,
        )
    }
}

/// Uma placa colorida da FAIXA — as três de uma porta, empilhadas, e a máquina escolhe qual se vê.
///
/// ⚠️ **Três objectos e não um objecto que muda de cor**, e é a decisão que torna a cena honesta:
/// a tabela de acções (#5) sabe **mostrar e esconder**, e não sabe pintar. Inventar um verbo de cor
/// só para o smoke seria medir um app que não existe.
fn placa(world: &mut World, nome: &str, x: f32, cor: [f32; 4], visivel: bool) {
    let (tamanho, y) = porta::faixa();
    world.spawn((
        Name::new(nome),
        Sprite::atlas(WHITE_TILE_KEY, tamanho, cor),
        Visibility { hidden: !visivel },
        Transform::from_translation(Vec2::new(x, y)),
    ));
}

/// O SPRITE do corpo de uma porta — o batente escuro, sempre visível, que carrega os componentes.
///
/// ⛔ **Ele é a razão de o `Transform` dos dois «donos» ter deixado de estar na origem:** uma
/// entidade sem sprite podia ficar em qualquer sítio porque não se via nem se apanhava; esta **é** o
/// alvo do dedo, logo tem de estar debaixo da faixa que ela comanda.
fn corpo_da_porta(x: f32) -> (Sprite, Transform) {
    let (tamanho, y) = porta::corpo();
    (
        Sprite::atlas(WHITE_TILE_KEY, tamanho, PORTA_RGBA),
        Transform::from_translation(Vec2::new(x, y)),
    )
}

/// As duas linhas que uma porta tem para CADA estado: mostra a placa dele, esconde as outras duas.
fn acende(alvo: &str, sinal: &str, apaga: [&str; 2]) -> Vec<SignalAction> {
    let mut v = vec![SignalAction {
        on: sinal.to_string(),
        target: alvo.to_string(),
        verb: SignalVerb::Show,
        arg: String::new(),
        target_by: ph2d_ecs::SignalTarget::default(),
    }];
    for outro in apaga {
        v.push(SignalAction {
            on: sinal.to_string(),
            target: outro.to_string(),
            verb: SignalVerb::Hide,
            arg: String::new(),
            target_by: ph2d_ecs::SignalTarget::default(),
        });
    }
    v
}

/// A cena `=1` — a porta e o controlo.
fn cena_um(world: &mut World) {
    // ⚠️ **O chão é a PRIMEIRA raiz criada**, e desde 2026-09-15 isso quer dizer que ele desenha
    // ATRÁS de tudo (a varredura que numera as raízes invertia a ordem de criação até essa data).
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 14.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));

    // ── O BOTÃO: um relógio que publica `botao` de 2 em 2 segundos ───────────
    //
    // ⚠️ **Um relógio e não uma tecla**, e é a decisão que faz a cena ensinar sozinha: com uma
    // tecla, quem não a carrega lê *«não faz nada»*. O ritmo vem do `Timer` (#2), que é o que a
    // fábrica também faz — *um componente novo apoia-se nos que já existem*.
    world.spawn((
        Name::new("Button"),
        Timers(vec![Timer {
            name: "tick".into(),
            duration_us: 2_000_000,
            repeat: true,
            autostart: true,
            signal: "botao".into(),
        }]),
        Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.8], [0.90, 0.90, 0.95, 1.0]),
        Transform::from_translation(Vec2::new(0.0, 5.2)),
    ));

    // ── A PORTA da esquerda: o corpo + as três placas + o cérebro + a tabela ─
    //
    // ⚠️ **O corpo nasce ANTES das placas** — a ordem de criação é a ordem de desenho (ver o chão,
    // acima), e o batente é o que fica por baixo.
    let esq = -4.5;
    let mut linhas = acende("Left Closed", "door_closed", ["Left Opening", "Left Open"]);
    linhas.extend(acende(
        "Left Opening",
        "door_opening",
        ["Left Closed", "Left Open"],
    ));
    linhas.extend(acende(
        "Left Open",
        "door_open",
        ["Left Closed", "Left Opening"],
    ));
    world.spawn((
        Name::new("Door"),
        StateMachine {
            states: vec![
                MachineState {
                    name: "Closed".into(),
                    on_enter: "door_closed".into(),
                    on_exit: String::new(),
                },
                MachineState {
                    name: "Opening".into(),
                    on_enter: "door_opening".into(),
                    on_exit: String::new(),
                },
                MachineState {
                    name: "Open".into(),
                    on_enter: "door_open".into(),
                    on_exit: String::new(),
                },
            ],
            // ⚠️ **As três ouvem o MESMO nome**, e é isso que a torna um ciclo de um passo por
            // toque: um sinal é **gasto** por quem o ouve, logo o estado de chegada não o ouve
            // outra vez no mesmo tique (a lei que um gate vermelho impôs a esta wave).
            transitions: vec![
                StateTransition {
                    from: 0,
                    on: "botao".into(),
                    to: 1,
                },
                StateTransition {
                    from: 1,
                    on: "botao".into(),
                    to: 2,
                },
                StateTransition {
                    from: 2,
                    on: "botao".into(),
                    to: 0,
                },
            ],
            initial: 0,
        },
        SignalActions(linhas),
        corpo_da_porta(esq),
    ));
    placa(world, "Left Closed", esq, FECHADA_RGBA, true);
    placa(world, "Left Opening", esq, A_ABRIR_RGBA, false);
    placa(world, "Left Open", esq, ABERTA_RGBA, false);

    // ── O CONTROLO: as MESMAS acções, SEM cérebro ────────────────────────────
    //
    // ⭐⭐⭐ Ele ouve o `botao` directamente — e as três linhas disparam **todas**, na ordem em que
    // estão escritas. A última ganha, e a porta fica presa. *É a medição do §5.0 posta no ecrã.*
    //
    // ⚠️ **Ele tem CORPO pelo mesmo motivo que a irmã** — e é isso que faz o controlo controlar
    // alguma coisa: o dono escolhe as duas portas e o painel diz, de uma, que ela tem cérebro, e da
    // outra que não. Sem corpo, «a que não tem» era inalcançável e a comparação não existia.
    let dir = 4.5;
    let mut controlo = acende("Right Closed", "botao", ["Right Opening", "Right Open"]);
    controlo.extend(acende(
        "Right Opening",
        "botao",
        ["Right Closed", "Right Open"],
    ));
    controlo.extend(acende(
        "Right Open",
        "botao",
        ["Right Closed", "Right Opening"],
    ));
    world.spawn((
        Name::new("Door (no brain)"),
        SignalActions(controlo),
        corpo_da_porta(dir),
    ));
    placa(world, "Right Closed", dir, FECHADA_RGBA, true);
    placa(world, "Right Opening", dir, A_ABRIR_RGBA, false);
    placa(world, "Right Open", dir, ABERTA_RGBA, false);
}

/// Monta a cena pedida e devolve o nível montado.
///
/// ⚠️ **Uma cena só, logo SEM `match`** — o `clippy` recusa um `match` de braço único, e com razão.
/// ⭐ O que mantém o [`CENAS`] honesto é o **gate** `o_roteador_nunca_devolve_acima_do_tecto`, que
/// varre níveis e compara: *o número conta-se do produto, nunca de uma nota* (`CLAUDE.md` §5.0).
/// ⛔ Uma cena nova traz de volta o `match` **e** um braço no gate.
pub fn montar(world: &mut World, _nivel: u32) -> u32 {
    cena_um(world);
    println!(
        "[statemachine-smoke] =1 a FAIXA da porta da ESQUERDA cicla vermelho -> amarelo -> verde \
         (um passo por toque do botao); a da DIREITA tem as MESMAS accoes SEM cerebro e fica \
         presa. Carregue no CORPO da porta da esquerda (o batente escuro por baixo da faixa) e \
         veja «Now: …» na seccao State Machine do painel"
    );
    1
}

#[cfg(test)]
#[path = "statemachine_smoke_tests.rs"]
mod tests;
