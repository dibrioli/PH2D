//! ⭐⭐⭐ **Smoke da VIDA** (plano 28, W2). `PH2D_VIDA_SMOKE=1`.
//!
//! # A cena: **quatro alvos numa coluna, e o herói que atira**
//!
//! O herói azul atira com o **`Q`** (a mesma arma da cena do golpe). Cada alvo tem uma VIDA
//! ([`ph2d_physics_ecs::Health`]) e cada bala um DANO ([`ph2d_physics_ecs::Damage`]) de `10`:
//!
//! | alvo | vida | o que acontece |
//! |---|---|---|
//! | vermelho | `10` | morre ao **1.º** tiro |
//! | laranja | `20` | morre ao **2.º** |
//! | roxo | `30` | morre ao **3.º** |
//! | cinzento (o CONTROLO) | `10`, **mesma equipa do herói** | **nunca** morre — as balas dele não o ferem |
//!
//! ⚠️ **Nenhum alvo precisa de tabela para MORRER**: a morte é da VIDA, e quem os tira da cena é o
//! dreno da shell (`mortes_anunciadas`). Na cena do golpe (#24) era uma linha `Destroy` por alvo;
//! aqui é **zero** — *é essa a diferença que a W2 compra*.
//!
//! # ⭐⭐ E o roxo tem uma tabela de TRÊS linhas, que é a W2b
//!
//! O `J` publica `veneno` e o roxo responde com `Damage 5` e arranca um relógio dele próprio, que um
//! segundo depois publica `cura-lenta` — e ele responde com `Heal 5`, o primeiro produtor do
//! `On Heal`.
//! ⛔ **Nenhuma das duas linhas é um `Destroy`**, e o gate afirma-o: a tabela passou a FERIR e a
//! CURAR, nunca a matar.
//!
//! # ⛔ Os alvos são SÓLIDOS, e a razão foi MEDIDA
//!
//! A bala é um mover que **pára rente** ao obstáculo e só vê formas SÓLIDAS (a sonda
//! `mede_o_golpe_que_chega`, casos E e G): um alvo-sensor seria atravessado sem ser visto, e o dano
//! chegaria pelo canal do mover só se o alvo fosse sólido. Na cena do golpe os alvos eram sensores
//! porque quem reportava era o `SignalOnHit`.
//!
//! # ⛔ E os alvos NASCEM, não são postos à mão
//!
//! Só sai da cena quem nasceu numa corrida (`ph2d_ecs::is_transient`): um alvo de DOCUMENTO morre
//! (deixa de levar tiros, grita o sinal) e **fica**. ⇒ cada alvo sai da sua **fábrica** (TOP-20
//! #11), que um **relógio** (#2) arranca ao entrar a corrida — a lição da cena do golpe.
//!
//! # ⭐⭐ E cada alvo tem a sua BARRA DE VIDA, e um PLACAR mostra a do roxo de longe (W4)
//!
//! A barra verde de cada alvo encolhe na hora do tiro, e o pedaço perdido fica BRANCO um instante
//! antes de escorrer — o rasto. O placar no alto é uma barra larga num objecto SEM vida que mostra a
//! do roxo **pelo nome** (`Target`), a forma do HUD: a mesma porta serve as duas.
//!
//! ⚠️ Se a linha `[vida-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, Factory, Lifetime, MasterRoot, Name, SignalAction, SignalActions, SignalFrom,
    SignalOnAction, SignalTarget, SignalVerb, Timer, Timers, Transform, Visibility, World,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, HealthBar, OnHit, ProjectileMotion,
    RigidBody,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — CONTADO do `match` do [`montar`].
pub const CENAS: u32 = 1;

/// A acção e a tecla — as MESMAS da cena do golpe, que é a fonte (a tecla foi medida lá).
pub const ACCAO: &str = crate::dano_smoke::ACCAO;
/// O nome da tecla, para o roteiro.
pub const TECLA_NOME: &str = crate::dano_smoke::TECLA_NOME;
/// O sinal do gatilho.
const SINAL: &str = "tiro-vida";
/// O sinal que arranca as quatro fábricas.
const COMECAR: &str = "comecar-vida";
/// O que um alvo grita ao levar dano, e ao morrer.
pub const AI: &str = "ai";
/// O que um alvo grita ao morrer.
pub const MORREU: &str = "morreu";
/// A equipa do herói — e do aliado cinzento.
pub const HEROIS: &str = "herois";
/// A equipa dos três alvos que morrem.
pub const MONSTROS: &str = "monstros";
/// O dano de uma bala.
pub const DANO: f32 = 10.0;

/// ⭐⭐ **O VENENO — a acção, a tecla e o sinal** (plano 28, W2b): o verbo `Damage` da tabela.
///
/// ⚠️ **O `J` foi MEDIDO e não escolhido** — é uma das três letras sem braço no teclado do editor
/// (`H` · `J` · `Q`, ver [`crate::trigger_smoke::TECLA`]); o `Q` já é o tiro, e o `H` é o *Bypass*
/// do grafo do Motion. Gate `a_tecla_do_veneno_nao_e_reclamada_pelo_editor`, com controlo.
pub const ACCAO_VENENO: &str = "veneno";
/// O keycode do `J`.
pub const TECLA_VENENO: u32 = 0x4A;
/// ⭐ **As acções que o prólogo cria, com a tecla de cada uma** — o tiro e o veneno. ⚠️ Uma lista
/// e não duas chamadas soltas: a shell percorre-a, logo uma terceira acção entra por aqui e não por
/// uma linha nova de composição.
pub const ACCOES: [(&str, u32); 2] = [
    (ACCAO, crate::trigger_smoke::TECLA),
    (ACCAO_VENENO, TECLA_VENENO),
];
/// O nome que o `J` tem na tela, para o roteiro.
pub const TECLA_VENENO_NOME: &str = "J";
/// O sinal do veneno — o herói publica-o, a tabela do roxo ouve-o.
pub const SINAL_VENENO: &str = "veneno";
/// Quanto tira um toque de veneno — **metade** de uma bala, para a descida ler-se diferente do tiro.
pub const VENENO: f64 = 5.0;
/// ⭐⭐ **A CURA** — um relógio do próprio roxo que o veneno ARRANCA e que o cura um segundo depois
/// (o verbo `Heal`).
///
/// ⛔⛔ **Ele NÃO é um relógio a repetir, e a razão foi uma FOTO** (2026-09-24): a 1.ª redacção curava
/// a cada segundo, sempre — e cada disparo é um aviso de sinal no topo do canvas. O molde **e** a
/// cópia correm o relógio (um molde reage a sinais, lei que esta cena não muda), logo aos `3 s` havia
/// **cinco** avisos `cura-lenta` empilhados a tapar os que o roteiro nomeia (`ai`, `morreu`,
/// `curou`). Arrancado pelo veneno, o relógio só fala depois de um toque.
pub const SINAL_CURA: &str = "cura-lenta";
/// O nome do relógio — o `StartTimer` da tabela aponta-o por aqui.
pub const RELOGIO_CURA: &str = "cura";
/// O atraso da cura, em µs.
pub const CURA_US: u64 = 1_000_000;
/// Quanto cura cada tique — igual ao veneno, para um toque ser desfeito por um segundo.
pub const CURA: f64 = 5.0;
/// O que o roxo grita ao ser curado — o `On Heal`, cujo primeiro produtor é esta wave.
pub const CUROU: &str = "curou";
/// O índice do alvo que leva o veneno e a cura — o ROXO, o de vida `30`, que é o que o passo (7)
/// manda escolher no Inspector: a vida a subir e a descer vê-se num sítio só.
pub const ROXO: usize = 2;

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const BALA_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];

/// ⭐⭐ **Os quatro alvos, de cima para baixo** — `(nome, vida, equipa, cor, y)`.
///
/// ⚠️ **Os `y` cabem na banda que a régua deixa visível** (`+4,09` / `−1,19` m, MEDIDA na foto da
/// cena da arma): com a timeline aberta a vista não é centrada na origem, e uma coluna simétrica
/// punha os dois de baixo fora do ecrã.
pub const ALVOS: [(&str, f32, &str, [f32; 4], f32); 4] = [
    (
        "Alvo de 1 tiro",
        10.0,
        MONSTROS,
        [0.88, 0.30, 0.28, 1.0],
        3.4,
    ),
    (
        "Alvo de 2 tiros",
        20.0,
        MONSTROS,
        [0.95, 0.60, 0.22, 1.0],
        2.1,
    ),
    (
        "Alvo de 3 tiros",
        30.0,
        MONSTROS,
        [0.62, 0.38, 0.85, 1.0],
        0.8,
    ),
    (
        "Aliado (nao morre)",
        10.0,
        HEROIS,
        [0.55, 0.57, 0.60, 1.0],
        -0.5,
    ),
];
/// O `x` da coluna de alvos.
pub const X_ALVOS: f32 = 4.0;
/// O lado de um alvo. ⚠️ Com `1,3` m entre filas, a janela de mira do herói é `LADO/2 + 0,1` =
/// `0,6` m para cada lado — o dobro do raio do herói.
pub const LADO: f32 = 1.0;
/// ⭐ **Onde a barra de cada alvo mora** (plano 28, W4) — o centro dela acima do centro do alvo.
///
/// ⚠️ **Não é o de fábrica (`0,75`), e a razão é a coluna:** com `1,3` m entre filas o quadrado de
/// cima começa `0,8` m acima deste centro, e a barra de fábrica (até `0,82`) tocaria-o.
///
/// ⚠️⚠️ **E a FOTO corrigiu a 1.ª redacção** (`0,62` com a altura de fábrica): a barra do alvo de
/// cima acabava EXACTAMENTE no topo da banda visível (`+4,09`), cortada pela borda. A `0,57` com
/// [`BARRA_H`] ela ocupa `0,52..0,62` — livre do próprio alvo, do de cima e da borda, com gate.
pub const BARRA_Y: f32 = 0.57;
/// A altura da barra de cada alvo — mais fina que a de fábrica, para caber na coluna.
pub const BARRA_H: f32 = 0.1;
/// O nome do objecto do PLACAR — uma barra larga no alto, sem vida própria.
pub const PLACAR: &str = "Placar";
/// ⚠️ **O nome que a CÓPIA do roxo recebe** — a porta de cópia dá a cada cópia um nome livre
/// (`ph2d_unique_name`), e o molde já tem o nome sem sufixo. ⇒ o placar nomeia a CÓPIA, e um gate
/// prova que a cópia que a fábrica faz é esta (senão o placar diria «ninguém com esse nome»).
pub const ALVO_DO_PLACAR: &str = "Alvo de 3 tiros (1)";
/// Onde o placar mora — dentro da banda que a régua deixa visível (topo `+4,09`), longe da coluna.
pub const PLACAR_XY: [f32; 2] = [0.0, 3.8];
/// O tamanho do placar — a barra de um HUD é larga e grossa, de propósito.
pub const PLACAR_WH: [f32; 2] = [4.0, 0.3];
// ⚠️ **O placar cabe na banda visível** (topo `+4,09`, medido na foto da cena da arma) — ERRO DE
// COMPILAÇÃO e não um gate: um `assert!` sobre constantes é dobrado pelo compilador.
const _: () = assert!(PLACAR_XY[1] + PLACAR_WH[1] / 2.0 <= 4.09);

/// **A receita que esta fábrica ainda vai apontar** — o marcador de MONTAGEM (o molde da cena do
/// golpe): a identidade só é atribuída depois.
#[derive(bevy_ecs::component::Component, Clone, Copy)]
struct Pendente(Entity);

/// **As três linhas do roxo** — o veneno FERE e ARRANCA o relógio; o relógio, um segundo depois,
/// CURA.
///
/// ⚠️ **A cura ouve `From Myself`, e é isso que a torna robusta:** o relógio vive no molde E em
/// cada cópia, e com `From Anyone` o relógio de um curaria o outro. O veneno ouve `From Anyone`
/// porque quem o publica é o herói. ⚠️ Arrancar um relógio que já anda **recomeça-o**
/// (`timer::start`), logo a cura chega um segundo depois do ÚLTIMO toque.
pub fn tabela_do_roxo() -> SignalActions {
    let linha = |on: &str, verb, arg: String, from| SignalAction {
        on: on.to_owned(),
        target: String::new(),
        verb,
        arg,
        target_by: SignalTarget::Named,
        from,
    };
    SignalActions(vec![
        linha(
            SINAL_VENENO,
            SignalVerb::Damage,
            VENENO.to_string(),
            SignalFrom::Anyone,
        ),
        linha(
            SINAL_VENENO,
            SignalVerb::StartTimer,
            RELOGIO_CURA.into(),
            SignalFrom::Anyone,
        ),
        linha(
            SINAL_CURA,
            SignalVerb::Heal,
            CURA.to_string(),
            SignalFrom::Myself,
        ),
    ])
}

/// **A RECEITA de um alvo** — um corpo SÓLIDO com uma VIDA; o ROXO leva também a tabela e o relógio
/// da cura ([`tabela_do_roxo`]).
fn receita_do_alvo(
    world: &mut World,
    (nome, vida, equipa, cor, _): (&str, f32, &str, [f32; 4], f32),
) -> Entity {
    let e = world
        .spawn((
            Name::new(nome),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], cor),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO / 2.0,
                    half_y: LADO / 2.0,
                },
                ..Collider::default()
            },
            Health {
                max: vida,
                start: vida,
                team: equipa.to_owned(),
                on_damage: AI.to_owned(),
                on_death: MORREU.to_owned(),
                on_heal: CUROU.to_owned(),
                ..Health::default()
            },
            // ⭐ A barra do PRÓPRIO alvo (`Target` vazio) — a cópia leva-a com o resto.
            HealthBar {
                offset_y: BARRA_Y,
                height: BARRA_H,
                ..HealthBar::default()
            },
        ))
        .id();
    if nome == ALVOS[ROXO].0 {
        world.entity_mut(e).insert((
            tabela_do_roxo(),
            Timers(vec![Timer {
                name: RELOGIO_CURA.to_owned(),
                duration_us: CURA_US,
                signal: SINAL_CURA.to_owned(),
                autostart: false,
                repeat: false,
            }]),
        ));
    }
    e
}

/// **A RECEITA da bala** — o projéctil da cena do golpe, com um DANO em vez de uma tag.
fn receita_da_bala(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Bala"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.16], BALA_RGBA),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 9.0,
                    range: 14.0,
                    face_velocity: true,
                    ..ProjectileLaw::default()
                },
                0,
            ),
            // ⚠️ Sem ele cada tiro que não acerta fica parado no fim do alcance (a lei do #12).
            Lifetime {
                duration_us: 3_000_000,
                ..Lifetime::default()
            },
            Damage {
                amount: DANO,
                team: HEROIS.to_owned(),
                on_hit: OnHit::Vanish,
                ..Damage::default()
            },
        ))
        .id()
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — o herói.
fn cena_um(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**: a ordem das raízes é a de CRIAÇÃO (a cura de 15/09).
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
    // O RELÓGIO que arranca as fábricas — `autostart`, sem repetição (o molde da cena do golpe).
    world.spawn((
        Name::new("Arranque"),
        Transform::from_translation(Vec2::new(0.0, 5.0)),
        Timers(vec![Timer {
            duration_us: 250_000,
            signal: COMECAR.to_owned(),
            autostart: true,
            repeat: false,
            ..Timer::default()
        }]),
    ));
    for alvo in ALVOS {
        let receita = receita_do_alvo(world, alvo);
        world.spawn((
            Name::new(format!("Fabrica: {}", alvo.0)),
            Transform::from_translation(Vec2::new(X_ALVOS, alvo.4)),
            Factory {
                master: 0,
                on_signal: COMECAR.to_owned(),
                burst: 1,
                total_max: 1,
                ..Factory::default()
            },
            Pendente(receita),
        ));
    }
    let bala = receita_da_bala(world);
    // ⭐ O PLACAR (plano 28, W4) — um objecto SEM vida cuja barra mostra a do roxo pelo nome.
    world.spawn((
        Name::new(PLACAR),
        Transform::from_translation(Vec2::new(PLACAR_XY[0], PLACAR_XY[1])),
        HealthBar {
            target: ALVO_DO_PLACAR.to_owned(),
            width: PLACAR_WH[0],
            height: PLACAR_WH[1],
            offset_y: 0.0,
            ..HealthBar::default()
        },
    ));

    // ⭐ O HERÓI: anda com as setas, roda para onde anda, e a arma aponta para onde ele aponta.
    // ⚠️ Nasce À ALTURA do alvo de cima — o 1.º tiro não pede pontaria.
    let heroi = world
        .spawn((
            Name::new("Heroi"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.4], HEROI_RGBA),
            Transform::from_translation(Vec2::new(-6.0, ALVOS[0].4)),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::Free,
                rotation: ph2d_topdown::rotation::RotationMode::ToMovement,
                ..TopDownLaw::default()
            }),
            SignalOnAction(vec![
                ph2d_ecs::ActionTriggerRow {
                    action: ACCAO.to_owned(),
                    edge: ph2d_ecs::ActionEdge::Press,
                    signal: SINAL.to_owned(),
                },
                ph2d_ecs::ActionTriggerRow {
                    action: ACCAO_VENENO.to_owned(),
                    edge: ph2d_ecs::ActionEdge::Press,
                    signal: SINAL_VENENO.to_owned(),
                },
            ]),
            Factory {
                master: 0,
                on_signal: SINAL.to_owned(),
                burst: 1,
                aim_from_spawner: true,
                ..Factory::default()
            },
        ))
        .id();
    world.entity_mut(heroi).insert(Pendente(bala));
    heroi
}

/// Troca cada [`Pendente`] pelo `StableId` do mestre. ⚠️ Corre DEPOIS de a identidade existir.
fn resolver_receitas(world: &mut World) {
    ph2d_ecs::assign_missing_stable_ids(world);
    let mut q = world.query::<(Entity, &Pendente)>();
    let pares: Vec<(Entity, Entity)> = q.iter(world).map(|(e, p)| (e, p.0)).collect();
    for (fab, mestre) in pares {
        let id = world.get::<ph2d_ecs::StableId>(mestre).map_or(0, |s| s.0);
        if let Some(mut f) = world.get_mut::<Factory>(fab) {
            f.master = id;
        }
        world.entity_mut(fab).remove::<Pendente>();
    }
}

/// O que o prólogo precisa de saber da cena montada.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// O HERÓI, que nasce escolhido.
    pub escolhido: u64,
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    resolver_receitas(world);
    println!(
        "[vida-smoke] cena=1  tecla={TECLA_NOME}  dano de uma bala={DANO}\n\
         (0) o quadrado de contorno verde no MEIO do ecra' sao os MOLDES (o do alvo e o da bala): \
         e' deles que cada alvo e cada bala nascem. Nao levam tiros e nao se mexem\n\
         (1) espere um instante: nascem QUATRO quadrados numa coluna a' direita — vermelho, \
         laranja, roxo e, em baixo, cinzento — cada um com uma BARRA verde por cima. No alto, a \
         barra larga e' o PLACAR: mostra a vida do ROXO de longe\n\
         (2) carregue no {TECLA_NOME}: o heroi azul ja' nasce a' altura do VERMELHO. Ele some ao \
         1.o tiro, e aparece o aviso `{AI}` e depois `{MORREU}`\n\
         (3) segure a seta para BAIXO ate' ficar a' altura do LARANJA, depois a seta para a \
         DIREITA (o heroi vira-se para onde anda) e atire: o 1.o tiro so' mostra `{AI}`, e a barra \
         dele encolhe para metade NA HORA — o pedaco perdido fica BRANCO um instante e depois \
         escorre (o rasto). O 2.o tiro mata-o\n\
         (4) faca o mesmo no ROXO: precisa de TRES tiros. O PLACAR no alto desce junto com a \
         barra dele\n\
         (5) o CONTROLO: atire no CINZENTO — ele e' da mesma equipa do heroi, e as balas batem nele \
         e somem sem o ferir. Nunca aparece `{AI}`\n\
         (6) na barra de CIMA carregue em `Reset` e depois em `Play`: os quatro voltam, cada um \
         com a vida inteira\n\
         (7) clique no ROXO: no painel da direita (Inspector) aparece a seccao `Health` com \
         `Now: 30 of 30`. Atire nele e o numero desce 10 por tiro. A bala tem a seccao `Damage`\n\
         (8) com o ROXO escolhido carregue no {TECLA_VENENO_NOME} (o veneno): `Now` desce {VENENO} \
         e aparece `{AI}`. Um segundo depois do ULTIMO toque ele sobe {CURA} e aparece `{CUROU}` — \
         a cura do proprio roxo. Toque varias vezes seguidas: desce a cada toque e sobe so' uma \
         vez. Na CURA a barra sobe de uma vez, sem rasto\n\
         (9) deu errado se: o laranja ou o roxo morrem ao 1.o tiro · o cinzento some · a bala \
         atravessa um quadrado · carregar no {TECLA_NOME} nao faz nada · nenhum quadrado some \
         depois de muitos tiros · o {TECLA_VENENO_NOME} nao mexe no `Now` · o roxo nunca \
         volta a subir · uma barra fica cheia depois de um tiro · o pedaco branco nunca escorre · \
         ou o PLACAR fica vazio ou parado"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "vida_smoke_tests.rs"]
mod tests;
