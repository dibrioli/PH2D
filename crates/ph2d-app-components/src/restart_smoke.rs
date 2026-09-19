//! ⭐⭐⭐ **Smoke do FIM DE JOGO** — *perdi, e o jogo recomeça sozinho.* `PH2D_RESTART_SMOKE=1`.
//!
//! # A cena: **o laço inteiro de um jogo, e nenhuma linha de script**
//!
//! Um pátio com três espinhos. O herói anda com as setas; tocar num espinho custa **uma vida**, e as
//! três luzes em cima apagam-se uma a uma. Quando a última se apaga o jogo espera **um segundo** —
//! para o dono ver que perdeu — e **recomeça sozinho**: o herói volta ao sítio de onde partiu, as
//! três luzes acendem, e as vidas voltam a três.
//!
//! # ⭐⭐ O que esta cena mostra que nenhuma outra mostra
//!
//! Ela é a única em que o app **fecha o laço**: até 2026-09-19 o artista conseguia autorar as seis
//! primeiras coisas — andar · nascer · bater · morrer · contar · **perder** — e a sétima, recomeçar,
//! não tinha porta nenhuma (a sonda `mede_o_que_a_composicao_ja_da_ao_fim_de_jogo` mediu-o: com os
//! **nove** verbos ligados ao sinal de fim, um contador de vidas que chegou a `0` fica em `1`, e o
//! princípio dele é `3`).
//!
//! # ⚠️⚠️ A BATIDA é COMPOSIÇÃO, e é de propósito
//!
//! O recomeço não é imediato: a vigia diz *«morri»*, a tabela **arranca um relógio de um segundo**,
//! e é o relógio que diz *«recomeça»*. ⛔ **Nenhuma lei nova** — o `Timer` (#2), a tabela (#5) e a
//! vigia (17/09) já existiam, e a wave acrescenta **um** verbo. *Sem a batida o dono vê a última luz
//! apagar-se e reacender no MESMO quadro, e não fica a saber que perdeu.*
//!
//! # ⚠️ As três LUZES e não um número
//!
//! É a decisão medida da cena irmã do `counter_watch`, com a foto a decidi-la: um `UiLabel` **troca
//! o que um texto MOSTRA, ele não cria o texto** — sem um `VecShape::Text` autorado por baixo o
//! rótulo desenha-se como um anel vazio. ⇒ as luzes, que além disso mostram uma coisa que um número
//! a descer não mostraria: **uma vigia dispara a um LIMIAR**.
//!
//! ⚠️ Se a linha `[restart-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Compare, Counter, CounterRuntime, CounterWatch, CounterWatchRow, Entity, Name, SignalAction,
    SignalActions, SignalVerb, Timer, TimerRuntime, Timers, Transform, Visibility, World,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody, SignalOnHit};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do corpo do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 1;

/// O contador de vidas — o nome é lido pela vigia e pelas linhas da tabela.
pub const VIDAS: &str = "vidas";
/// Quantas vidas a corrida começa com. ⚠️ **TRÊS e não duas:** com duas o dono perde antes de
/// perceber a regra, e com quatro o passo (2) do roteiro fica longo.
pub const VIDAS_INICIAIS: i64 = 3;

/// O sinal que um espinho grita ao ser tocado.
pub const GOLPE: &str = "golpe";
/// O sinal que a vigia diz quando a última vida se vai.
pub const MORRI: &str = "morri";
/// O sinal que o relógio da batida diz — e é ele que recomeça a corrida.
pub const RECOMECA: &str = "recomeca";
/// O nome do relógio da batida.
pub const RELOGIO: &str = "batida";

/// ⭐⭐ **A BATIDA, em microssegundos** — o tempo entre a última luz apagar e o jogo recomeçar.
///
/// ⚠️ **UM segundo, e o número é do PRODUTO e não do motor:** abaixo de meio segundo o dono não
/// distingue *«perdi»* de *«piscou»*, e acima de dois ele pensa que o app parou. É a mesma faixa que
/// todo jogo de arcade usa para o ecrã de fim.
pub const BATIDA_US: u64 = 1_000_000;

/// O prefixo do nome de cada luz — o alvo que a tabela nomeia.
pub const LUZ: &str = "Vida ";
/// ⚠️ **A altura das luzes, e ela é MEDIDA e não escolhida:** a 1.ª redacção punha-as a `3,2` m e a
/// cena media `5,70` m de alto contra a **BANDA** de `5,00` — o que sobra do canvas com a timeline
/// aberta é ~metade da janela, e as luzes ficavam FORA DO ECRÃ. Há gate.
const LUZ_Y: f32 = 1.8;

/// Quantos espinhos o pátio tem. ⚠️ **Tantos quantas as vidas**, para o dono poder perder andando
/// em frente uma vez por espinho — sem ter de voltar a nenhum.
pub const ESPINHOS: i64 = VIDAS_INICIAIS;
/// O passo entre espinhos, em metros.
const ESPINHO_PASSO: f32 = 3.0;

/// Onde o herói nasce. ⚠️ **Abaixo da fileira de espinhos**, para o passo (1) do roteiro ser uma
/// tecla: a seta para cima leva-o ao primeiro.
pub const HEROI_Y: f32 = -2.0;

const CHAO_RGBA: [f32; 4] = [0.13, 0.14, 0.17, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const ESPINHO_RGBA: [f32; 4] = [0.95, 0.35, 0.30, 1.0];
const LUZ_RGBA: [f32; 4] = [0.45, 0.92, 0.55, 1.0];

/// O que o prólogo precisa de saber da cena montada.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// O HERÓI, que nasce escolhido — ver [`montar`].
    pub escolhido: u64,
}

/// Uma linha *«ao ouvir `on`, faz `verb` em `target`»*. Alvo vazio = **este objecto**.
fn linha(on: &str, target: &str, verb: SignalVerb, arg: &str) -> SignalAction {
    SignalAction {
        on: on.to_owned(),
        target: target.to_owned(),
        verb,
        arg: arg.to_owned(),
        ..SignalAction::default()
    }
}

/// Uma regra *«quando as vidas chegarem a `limiar`, diz `sinal`»*.
fn regra(limiar: i64, sinal: &str) -> CounterWatchRow {
    CounterWatchRow {
        counter: VIDAS.to_owned(),
        compare: Compare::AtMost,
        value: limiar,
        signal: sinal.to_owned(),
        once: true,
    }
}

/// As três luzes de vida, em cima do pátio.
///
/// ⛔⛔⛔ **A `Visibility` é obrigatória e a sua ausência é MUDA** — o gate desta cena apanhou-a na
/// 1.ª corrida: o `set_visible` da ponte lê `get::<Visibility>` e **devolve `false` a quem não a
/// tem**, logo um `Hide` sobre uma luz sem ela é **inerte** (contado, não gritado). *O dono tocaria
/// no espinho e a luz não se apagava, com a tabela, a vigia e o contador todos certos.*
fn luzes(world: &mut World) {
    for i in 0..VIDAS_INICIAIS {
        #[allow(clippy::cast_precision_loss)]
        let x = (i as f32 - 1.0) * 1.2;
        world.spawn((
            Name::new(format!("{LUZ}{}", i + 1)),
            Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.8], LUZ_RGBA),
            Visibility::default(),
            Transform::from_translation(Vec2::new(x, LUZ_Y)),
        ));
    }
}

/// Os espinhos: colisores **SENSOR** que gritam ao serem tocados.
fn espinhos(world: &mut World) {
    for i in 0..ESPINHOS {
        #[allow(clippy::cast_precision_loss)]
        let x = (i as f32 - 1.0) * ESPINHO_PASSO;
        world.spawn((
            Name::new(format!("Espinho {}", i + 1)),
            RigidBody {
                kind: BodyKind::Static,
            },
            // ⚠️ **`is_sensor`**: ele avisa e não empurra — um espinho que travasse o herói faria
            // o passo (2) do roteiro depender de pontaria.
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.5,
                },
                is_sensor: true,
                ..Collider::default()
            },
            SignalOnHit(GOLPE.to_owned()),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], ESPINHO_RGBA),
            Transform::from_translation(Vec2::new(x, 0.0)),
        ));
    }
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO**.
fn cena_um(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**, e isto não é estilo: desde a cura de 15/09 a ordem das raízes é a
    // ordem de CRIAÇÃO, logo quem nasce primeiro desenha por baixo.
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [24.0, 16.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
    luzes(world);
    espinhos(world);

    // ⭐⭐⭐ **O HERÓI carrega o jogo inteiro** — o contador, a vigia, o relógio da batida e a
    // tabela que liga os quatro. *Nenhuma linha de script.*
    world
        .spawn((
            Name::new("Heroi"),
            // ⛔⛔ **O CORPO CINEMÁTICO não é decoração** (report do dono, 19/09): a ponte do mover
            // varre `self.bodies`, logo quem não tem corpo **nunca entra no laço** e as setas não
            // fazem nada. E `Kinematic` é LEI — um corpo dinâmico é do SOLVER e a gravidade leva-o.
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.45 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.9], HEROI_RGBA),
            Transform::from_translation(Vec2::new(0.0, HEROI_Y)),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: 5.0,
                direction: DirectionMode::Free,
                ..TopDownLaw::default()
            }),
            Counter {
                name: VIDAS.to_owned(),
                start: VIDAS_INICIAIS,
            },
            CounterRuntime {
                value: VIDAS_INICIAIS,
            },
            // ⭐ **A BATIDA**: um relógio que NÃO arranca sozinho — quem o arranca é a tabela, ao
            // ouvir «morri». ⛔ Com `autostart` ele recomeçaria a corrida um segundo depois de ela
            // começar, sem ninguém ter perdido.
            Timers(vec![Timer {
                name: RELOGIO.to_owned(),
                duration_us: BATIDA_US,
                repeat: false,
                autostart: false,
                signal: RECOMECA.to_owned(),
            }]),
            TimerRuntime::default(),
            // ⭐⭐⭐ **A VIGIA**: três limiares para as luzes, e o quarto é o fim.
            CounterWatch(vec![
                regra(2, "luz3"),
                regra(1, "luz2"),
                regra(0, "luz1"),
                regra(0, MORRI),
            ]),
            // ⭐⭐⭐ **A TABELA — o jogo inteiro em seis linhas.**
            SignalActions(vec![
                // tocar num espinho custa uma vida
                linha(GOLPE, "", SignalVerb::AddToCounter, "-1"),
                // cada limiar apaga a sua luz
                linha("luz3", &format!("{LUZ}3"), SignalVerb::Hide, ""),
                linha("luz2", &format!("{LUZ}2"), SignalVerb::Hide, ""),
                linha("luz1", &format!("{LUZ}1"), SignalVerb::Hide, ""),
                // ⭐ e a última vida ARRANCA A BATIDA, em vez de recomeçar já
                linha(MORRI, "", SignalVerb::StartTimer, RELOGIO),
                // ⭐⭐⭐ **O VERBO DESTA WAVE** — e ele não tem alvo: o sujeito é a CORRIDA.
                linha(RECOMECA, "", SignalVerb::RestartRun, ""),
            ]),
        ))
        .id()
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    ph2d_ecs::assign_missing_stable_ids(world);
    println!(
        "[restart-smoke] cena=1  {VIDAS_INICIAIS} vidas · {ESPINHOS} espinhos · batida de {:.1} s\n\
         (1) ande com as SETAS para CIMA: o quadrado azul toca no espinho VERMELHO do meio e uma \
         das tres luzes verdes APAGA-SE\n\
         (2) toque nos outros dois espinhos: as luzes apagam uma a uma. Ao apagar a ULTIMA o jogo \
         espera um segundo — e RECOMECA SOZINHO\n\
         (3) o que tem de acontecer no recomeco: o heroi volta ao sitio de onde partiu, as TRES \
         luzes acendem, e o painel da direita mostra as vidas de volta a {VIDAS_INICIAIS}\n\
         (4) o heroi ja' esta' escolhido: role o painel da direita ate' `Signal Actions` — o jogo \
         inteiro sao SEIS linhas, e a ultima diz `Restart Run`\n\
         (5) na mesma seccao, repare que a linha do `Restart Run` NAO pergunta a quem: o sujeito \
         dela e' a corrida, nao um objecto\n\
         (6) role ate' `Counter Watch`: e' a regra que liga o contador ao fim. A linha `Now` mostra \
         as vidas a descer enquanto joga\n\
         (7) deu errado se: as setas nao moverem o quadrado azul · tocar num espinho nao apagar \
         luz · o jogo nao recomecar sozinho · ou recomecar e as luzes ficarem APAGADAS",
        BATIDA_US as f64 / 1e6
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "restart_smoke_tests.rs"]
mod tests;
