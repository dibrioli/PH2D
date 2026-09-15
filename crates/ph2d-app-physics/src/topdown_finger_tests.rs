//! ⭐⭐⭐ **O DEDO DO TECLADO CHEGA AO MOVER DE VISTA DE CIMA — pela porta do PRODUTO.**
//!
//! # ⛔⛔ Por que este ficheiro existe (report do dono, 2026-09-15: *«nada se move»*)
//!
//! A wave do TOP-20 #13 fechou com **24 provas de mutação** e a suíte verde, e o boneco **não
//! andava no app**. Todos os gates da lei e da ponte entravam pelo canal interno
//! ([`ph2d_physics_ecs::PhysicsBridge::set_player_input`]) ou por uma fita escrita à mão — e a
//! rotura estava **acima** dos dois: a [`crate::bridge::dispatch::dispatch`], que é o ÚNICO ponto
//! por onde o teclado do app entra na ponte, perguntava *«quem são os players?»* varrendo só o
//! [`PlatformPlayer`].
//!
//! ⇒ o mover de vista de cima **nunca recebia entrada** (mapa vazio ⇒ `unwrap_or_default()` ⇒
//! intenção zero) e, como a contagem de players era `0`, a fita **também não gravava** — logo o
//! segundo caminho, o do replay, estava morto pela mesma razão.
//!
//! ⚠️⚠️ **É a lei que a casa já tinha escrita:** *um gesto escrito em DUAS metades aceita a
//! variante nova em SÓ UMA, e a fixtura que chama a porta interna fica verde.* Eu ensinei a metade
//! do **replay** ([`take_taped_input`]) a conhecer o mover novo e não a metade da **entrega**.
//!
//! ⇒ estes gates entram pela porta do produto, com a mesma assinatura que o quadro usa, e a
//! pergunta *«quem lê o teclado?»* passou a ter **uma** porta
//! ([`ph2d_physics_ecs::reads_the_keyboard`]), que as duas metades chamam.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InputTape, PhysicsBridge, PlayerInput, RigidBody,
    TopDownPlayer,
};
use ph2d_timeline::TimelineDoc;

use crate::bridge::dispatch::dispatch;

const DT: f64 = 1.0 / 60.0;
const VELOCIDADE: f32 = 4.0;

/// Um mover de vista de cima sozinho num mundo vazio — **sem parede nenhuma**, porque o que se
/// mede aqui é o CANAL e não o deslize.
fn cena(default_controls: bool) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    // ⚠️ **Escrito nos campos do COMPONENTE**, que é o que o artista autora — e não na lei, que
    // obrigaria esta crate a depender da folha só para montar uma fixtura.
    let cfg = TopDownPlayer {
        speed: VELOCIDADE,
        // ⚠️ **Rampas instantâneas**: o que este gate mede é o dedo a chegar, e uma rampa faria a
        // barra depender de quantos tiques se correm.
        acceleration: 0.0,
        deceleration: 0.0,
        default_controls,
        ..TopDownPlayer::default()
    };
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.35 },
                ..Collider::default()
            },
            cfg,
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, e)
}

/// Roda `frames` quadros **pela porta do produto** com o dedo `input` segurado, e devolve onde o
/// corpo ficou.
fn corre(sim: &mut SimWorld, input: PlayerInput, frames: u64) -> Vec2 {
    let mut bridge = PhysicsBridge::new();
    let mut doc = TimelineDoc::new();
    let mut playhead = Playhead::new(DT);
    let mut tape = InputTape::new();
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    playhead.play();
    for _ in 0..frames {
        playhead.advance();
        dispatch(
            &mut bridge,
            sim,
            &playhead,
            DT,
            &mut doc,
            // ⚠️ **Armado**: é o que a cena de smoke faz, e desarmado o mundo é SEGURADO.
            true,
            input,
            &mut tape,
            &mut drive,
        );
    }
    let mut q = sim.world().try_query::<&Transform>().expect("query");
    let t = q.iter(sim.world()).next().expect("o corpo");
    t.translation
}

/// ⭐⭐⭐ **O gate que teria apanhado o report do dono.**
#[test]
fn o_dedo_do_teclado_chega_ao_mover_de_vista_de_cima() {
    let (mut sim, _) = cena(true);
    let input = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    const TIQUES: u64 = 30;
    let fim = corre(&mut sim, input, TIQUES);
    // Meio segundo a 4 m/s ⇒ 2 m. A folga é generosa de propósito: o que se afirma aqui é
    // *«ele anda»*, e a distância exacta é assunto dos gates da lei.
    let esperado = VELOCIDADE * (TIQUES as f32) * (DT as f32);
    assert!(
        (fim.x - esperado).abs() < 0.2,
        "o dedo NAO chegou ao mover: ele andou {:.4} m em {TIQUES} tiques e devia andar {esperado:.4}.\n\
         ⚠️ Se isto e' ZERO, a pergunta «quem le o teclado?» voltou a ter duas respostas — \
         a `hand_input_to_players` do dispatch e a `take_taped_input` da fita.",
        fim.x
    );
    assert!(
        fim.y.abs() < 1.0e-3,
        "sem intencao vertical ele nao pode derivar em y: {:.6}",
        fim.y
    );
}

/// ⭐⭐ **O CONTROLO — e ele é o que mantém o «motor puro» honesto.**
///
/// Com `default_controls = false` a entidade tem de ficar de fora das DUAS metades: a que entrega
/// e a que grava. Sem este gate, a cura do irmão acima podia ser *«entrega a toda a gente»*.
#[test]
fn e_com_os_controlos_de_fabrica_desligados_ele_ignora_o_teclado() {
    let (mut sim, _) = cena(false);
    let input = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    let fim = corre(&mut sim, input, 30);
    assert!(
        fim.x.abs() < 1.0e-4 && fim.y.abs() < 1.0e-4,
        "um motor PURO nao pode ouvir o teclado, e ele andou para ({:.6}, {:.6})",
        fim.x,
        fim.y
    );
}

/// ⚠️ **E o eixo VERTICAL é meia pergunta separada** — o `drive_y` nasceu nesta wave e viaja na
/// mesma struct, então uma entrega que só copiasse `drive` deixaria o boneco a andar só na
/// horizontal, que é um report diferente e igualmente mudo.
#[test]
fn e_o_eixo_vertical_viaja_na_mesma_entrega() {
    let (mut sim, _) = cena(true);
    let input = PlayerInput {
        drive_y: 1.0,
        ..PlayerInput::default()
    };
    let fim = corre(&mut sim, input, 30);
    assert!(
        fim.y > 1.8,
        "o eixo vertical nao chegou: y = {:.4} (esperado ~2,0)",
        fim.y
    );
}

/// Uma cena de **4 direcções** — o modo em que a ordem das setas manda.
fn cena_quatro_direccoes() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut cfg = TopDownPlayer {
        speed: VELOCIDADE,
        acceleration: 0.0,
        deceleration: 0.0,
        ..TopDownPlayer::default()
    };
    cfg.direction_mode =
        ph2d_topdown::direction::to_wire(ph2d_topdown::direction::DirectionMode::FourWay);
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.35 },
            ..Collider::default()
        },
        cfg,
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    sim
}

/// ⭐⭐⭐ **A ÚLTIMA SETA MANDA, e a memória SOBREVIVE entre tiques** (ordem do dono, 2026-09-15).
///
/// # ⛔⛔ O que só ESTE gate pode reprovar
///
/// Os gates da lei ([`ph2d_topdown`]) provam que, **dada** a memória, a última seta ganha. O que
/// eles não alcançam é a ponte **carregar** essa memória de um tique para o seguinte: se ela
/// nascesse fresca a cada tique, as duas setas pareceriam chegar sempre ao mesmo tempo e o corpo
/// ficaria preso no eixo declarado para sempre — com a suíte da lei inteira verde.
///
/// ⚠️ **A sequência é o discriminador, e ela foi escolhida por isso:** segurar as duas desde o
/// princípio dá a MESMA resposta com e sem memória. Só *«uma primeiro, a outra depois»* separa os
/// dois programas.
#[test]
fn a_ultima_seta_manda_e_a_memoria_atravessa_os_tiques() {
    let mut sim = cena_quatro_direccoes();
    let mut bridge = PhysicsBridge::new();
    let mut doc = TimelineDoc::new();
    let mut playhead = Playhead::new(DT);
    let mut tape = InputTape::new();
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    playhead.play();

    let direita = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    let ambas = PlayerInput {
        drive: 1.0,
        drive_y: 1.0,
        ..PlayerInput::default()
    };
    let mut passo = |sim: &mut SimWorld, entrada: PlayerInput, tiques: u64| {
        for _ in 0..tiques {
            playhead.advance();
            dispatch(
                &mut bridge,
                sim,
                &playhead,
                DT,
                &mut doc,
                true,
                entrada,
                &mut tape,
                &mut drive,
            );
        }
    };

    passo(&mut sim, direita, 15);
    let meio = pos(&sim);
    assert!(
        meio.x > 0.9,
        "so' com a `→` ele tem de andar em x: {meio:?}"
    );

    passo(&mut sim, ambas, 15);
    let fim = pos(&sim);
    let andou_x = fim.x - meio.x;
    let andou_y = fim.y - meio.y;
    assert!(
        andou_y > 0.9,
        "a `↑` chegou DEPOIS e tem de mandar: ele andou {andou_y:.4} em y.\n\
         ⚠️ Se isto e' ~0 e o x cresceu, a ponte esta' a criar a memoria FRESCA a cada tique — \
         as duas setas parecem chegar juntas e a dominancia cai no eixo declarado para sempre."
    );
    assert!(
        andou_x.abs() < 1.0e-3,
        "e o `→` tem de CALAR-SE enquanto a `↑` manda: ele andou {andou_x:.6} em x"
    );
}

/// A pose do (único) corpo da cena.
fn pos(sim: &SimWorld) -> Vec2 {
    let mut q = sim.world().try_query::<&Transform>().expect("query");
    q.iter(sim.world()).next().expect("o corpo").translation
}
