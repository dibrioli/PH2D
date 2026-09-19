//! ⭐⭐⭐ **Smoke do GATILHO** (suplente #24). `PH2D_TRIGGER_SMOKE=1`.
//!
//! # A cena: **a arma que aponta, e a que não aponta**
//!
//! Um herói que anda com as setas e **roda para onde anda**, e duas armas que ouvem a MESMA tecla:
//!
//! | quem | o que ele demonstra |
//! |---|---|
//! | **o herói** (azul) | a arma dele tem `Aim from spawner` **ligado** — a bala sai para onde ele está virado |
//! | a **torreta** (cinzenta) | o CONTROLO: o mesmo gatilho, a mesma receita, e o `Aim from spawner` **desligado** — as balas dela saem sempre para o mesmo lado |
//!
//! ⚠️⚠️ **As duas ouvem a mesma tecla de propósito, e é isso que torna a wave legível:** uma tecla,
//! dois tiros, e a única diferença entre eles é **uma caixa de marcar**. *Duas teclas não ensinariam
//! nada — o dono não saberia se o que mudou foi a mira ou o gesto.*
//!
//! # ⛔ E a acção `fire` NÃO vem de fábrica
//!
//! O `InputMap::with_player_defaults` tem sete acções e **nenhuma é disparar** (medido). O prólogo
//! da shell cria-a e liga-a ao **espaço**, porque *um gatilho cuja acção o mapa não conhece fica
//! calado* — e o smoke mostraria a ferramenta a parecer partida, que é a espécie que o `CLAUDE.md`
//! §5.0 chama de **pior que uma cena ausente**.
//!
//! ⚠️ Se a linha `[trigger-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ActionEdge, ActionTriggerRow, Entity, Factory, Lifetime, MasterRoot, Name, SignalOnAction,
    Transform, Visibility, World,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, ProjectileMotion, RigidBody};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 1;

/// O nome da acção que o prólogo cria — ⚠️ **lido nos DOIS sítios pela mesma const**, senão a cena
/// liga uma tecla a uma acção e o gatilho ouve outra.
pub const ACCAO: &str = "fire";
/// O nome do sinal — idem: quem publica e quem ouve leem daqui.
pub const SINAL: &str = "tiro";

/// ⭐⭐⭐ **A TECLA — o `Q`, e ela foi MEDIDA e não escolhida** (report do dono, 2026-09-18).
///
/// ⛔⛔ **A 1.ª redacção usava o ESPAÇO e afirmava, sem medir, que era *«a tecla que ninguém do
/// editor usa no canvas»*.** É FALSO: o espaço é o **Play/Pause do transporte**
/// ([`ph2d_editor_core`], `dispatch::key`: `KEY_SPACE if !cmd => GraphKey::TogglePlay`), logo um
/// toque fazia **duas** coisas — parava a corrida E disparava. *Uma afirmação sobre um atalho é
/// uma MEDIÇÃO que eu não fiz.*
///
/// # ⚠️ A medição (2026-09-18), e porque a lista velha não servia
///
/// Um estudo de 2026-08-12 mediu *«das 26 letras, só NOVE estão livres: `H I J M N P U V Y`»* — e
/// ela **envelheceu para falsa**: o `P` foi tomado pelo menu radial (E4) e o `U` pelo detalhe da
/// escultura, no mês seguinte. ⇒ re-medido contra os braços REAIS do teclado de canvas, sobram
/// **três** letras sem braço simples: **`H`** (que é o *Bypass* do grafo do Motion), **`J`** e
/// **`Q`**.
///
/// ⚠️⚠️ **E a 1.ª re-medição também mentiu**: ela contou a tabela do NORMALIZADOR
/// (`KeyCode::KeyQ => 0x51`, que traduz e não trata) como se fossem atalhos, e devolveu *«nenhuma
/// letra está livre»*. *Uma régua que varre `KeyCode::Key<X> =>` apanha o tradutor junto com os
/// consumidores.*
///
/// ⭐⭐ **O `Q` ganha das outras duas porque não é só livre: é a tecla que o MAPA DE FÁBRICA deste
/// app já escolheu para uma acção de JOGADOR** (`PLAYER_DASH`), com a regra escrita ao lado dela —
/// *«o `W` desta shell abre o painel de mundo, e um default que briga com um atalho que já existe é
/// uma armadilha que só o artista descobre»*. Esta cena tinha exactamente essa armadilha.
///
/// ⚠️ **Partilhar tecla com o `dash` é DELIBERADO e tem precedente escrito** no mesmo mapa (o
/// `jump` e o `move_up` partilham a seta de cima): *as duas leis nunca correm na mesma cena* — o
/// `dash` é do mover de PLATAFORMA e aqui o herói é de vista de cima.
///
/// ⛔ E há gate: `a_tecla_do_gatilho_nao_e_reclamada_pelo_editor`, com o ESPAÇO como **controlo
/// positivo** — senão a régua mede o nada e fica verde.
pub const TECLA: u32 = 0x51;

/// O nome que o `Q` tem na tela, para o roteiro e para o painel.
pub const TECLA_NOME: &str = "Q";

/// ⭐⭐⭐ **Uma acção que EXISTE e NÃO tem tecla** — o sujeito do segundo aviso.
///
/// ⛔⛔ **Ela tem de ser CRIADA pela cena, e isso foi medido:** as **sete** acções do
/// `InputMap::with_player_defaults` têm todas ligação (`←/A · →/D · ↑/Z · ↓/S · Q · R`), logo
/// nenhuma delas serve de exemplo. *Um passo de roteiro que manda escrever `grab` ensinaria o
/// contrário do que acontece* — a espécie que o `CLAUDE.md` §5.0 chama de pior que uma cena
/// ausente.
///
/// ⚠️ **E o estado dela não é um erro:** é o que qualquer artista alcança ao criar uma acção e
/// esquecer-se de lhe ligar uma tecla. O gatilho fica calado, e até 2026-09-18 o painel dizia que
/// estava tudo bem.
pub const ACCAO_SEM_TECLA: &str = "reload";

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const TORRETA_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];
const BALA_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];
const BALA_CTRL_RGBA: [f32; 4] = [0.70, 0.72, 0.75, 1.0];

/// **A receita que esta fábrica ainda vai apontar** — o marcador de MONTAGEM da irmã do `#11`.
///
/// ⚠️ Ele existe porque a identidade só é atribuída depois: o `Factory::master` é um `StableId`, e
/// no instante em que a cena monta o mestre ainda não tem um. ⛔ Semear o campo com os bits da
/// entidade seria escrever no componente a coisa que o `CLAUDE.md` proíbe.
#[derive(bevy_ecs::component::Component, Clone, Copy)]
struct Pendente(Entity);

/// A RECEITA de uma bala: um mestre escondido, com a lei do projéctil e a higiene do ciclo de vida.
///
/// ⚠️ **O `Lifetime` não é enfeite** — sem ele cada tiro fica na cena para sempre, e a `=1` enche-se
/// de balas paradas no fim do alcance (a lei do `#12`: sem ela o `#11` e o `#14` VAZAM).
fn receita(world: &mut World, nome: &str, cor: [f32; 4]) -> Entity {
    world
        .spawn((
            Name::new(nome),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.12 },
                ..Collider::default()
            },
            // ⚠️ **Um rectângulo COMPRIDO** e não um quadrado: é ele que torna a MIRA visível — uma
            // bala redonda aponta para todo o lado.
            Sprite::atlas(WHITE_TILE_KEY, [0.55, 0.18], cor),
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 9.0,
                    range: 7.0,
                    face_velocity: true,
                    ..ProjectileLaw::default()
                },
                0,
            ),
            // ⚠️ `duration_us` e não segundos: o relógio desta casa é o passo fixo, e um `f32` de
            // segundos aqui seria a segunda unidade a atravessar a mesma fronteira.
            Lifetime {
                duration_us: 2_000_000,
                ..Lifetime::default()
            },
        ))
        .id()
}

/// **A arma:** o gatilho que ouve a tecla e a fábrica que nasce ao sinal dele.
///
/// ⚠️ **Os dois no MESMO objecto**, e é o que faz a mira funcionar: o ângulo que a cópia herda é o
/// de MUNDO da FÁBRICA (`aim_from_spawner`), logo ela tem de viver em quem aponta.
fn arma(world: &mut World, e: Entity, mestre: Entity, mira: bool) {
    world.entity_mut(e).insert((
        SignalOnAction(vec![ActionTriggerRow {
            action: ACCAO.to_owned(),
            edge: ActionEdge::Press,
            signal: SINAL.to_owned(),
        }]),
        Factory {
            master: 0, // resolvido em `resolver_receitas` — o mestre ainda não tem identidade
            on_signal: SINAL.to_owned(),
            burst: 1,
            aim_from_spawner: mira,
            ..Factory::default()
        },
    ));
    world.entity_mut(e).insert(Pendente(mestre));
}

/// A cena `=1` — a arma que aponta, e a que não aponta. Devolve **quem nasce ESCOLHIDO**.
fn cena_um(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**, e isto não é estilo: desde a cura de 15/09 a ordem das raízes é a
    // ordem de CRIAÇÃO, logo quem nasce primeiro desenha por baixo. *Antes era o contrário, e
    // ninguém sabia.*
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [20.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));

    let bala = receita(world, "Bala", BALA_RGBA);
    let bala_ctrl = receita(world, "Bala (controlo)", BALA_CTRL_RGBA);

    // ⭐ O HERÓI: anda com as setas e **roda para onde anda** — é essa rotação que a mira lê.
    let heroi = world
        .spawn((
            Name::new("Heroi"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.4 },
                ..Collider::default()
            },
            // ⚠️ Comprido, como as balas, e pela mesma razão: é assim que se vê para onde ele está
            // virado quando está parado.
            Sprite::atlas(WHITE_TILE_KEY, [1.1, 0.5], HEROI_RGBA),
            Transform::from_translation(Vec2::new(-3.0, 0.0)),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::Free,
                // ⭐ **É esta linha que a mira lê**: o herói vira para onde anda, e a bala sai para
                // onde ele está virado.
                rotation: ph2d_topdown::rotation::RotationMode::ToMovement,
                ..TopDownLaw::default()
            }),
        ))
        .id();
    arma(world, heroi, bala, true);

    // ⭐⭐ O CONTROLO: o MESMO gatilho e a MESMA receita, com a mira DESLIGADA.
    //
    // ⚠️ Ela está rodada `90°` de propósito — com a rotação a zero, «herda a rotação da fábrica» e
    // «fica com a do molde» dariam a MESMA imagem, e o controlo não controlaria nada.
    let mut pose = Transform::from_translation(Vec2::new(3.5, 0.0));
    pose.rotation = std::f32::consts::FRAC_PI_2;
    let torreta = world
        .spawn((
            Name::new("Torreta (sem mira)"),
            Sprite::atlas(WHITE_TILE_KEY, [1.1, 0.5], TORRETA_RGBA),
            pose,
        ))
        .id();
    arma(world, torreta, bala_ctrl, false);
    // ⭐⭐ **O HERÓI nasce escolhido, e é a FOTO que o exige:** o roteiro manda ver a secção
    // *Trigger* «no painel da direita», e com ninguém escolhido o Inspector diz *«Select an entity
    // in the Hierarchy»*. *Um passo que nomeia uma secção AFIRMA que ela está na tela* — a lei que
    // o `#15` pagou com o report *«não apareceu no painel a seção state machine»*.
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
    /// O HERÓI, que nasce escolhido — ver [`cena_um`].
    pub escolhido: u64,
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
///
/// ⚠️ Separada do prólogo pela razão das irmãs: é a metade que um gate consegue correr — o resto
/// pede o relógio e o Input Map, que não são o mundo.
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    resolver_receitas(world);
    // ⚠️ **O roteiro é da CENA e imprime-se AQUI**, como o das irmãs (`particles_smoke`): ele fala
    // do que a cena montou e da tecla que ela declara, e nada disso é da shell. ⛔ Ele viveu no
    // prólogo até a catraca `the_shell_only_shrinks` o mandar para casa — e as duas respostas
    // coincidem, que é o sinal de que o corte é por responsabilidade.
    println!(
        "[trigger-smoke] cena=1  accao=«{ACCAO}» (tecla {TECLA_NOME})  sinal=«{SINAL}»\n\
         (1) carregue no {TECLA_NOME}: sai uma bala AMARELA do retangulo AZUL, para onde ele \
         aponta\n\
         (2) ande com as SETAS — o azul RODA para onde anda; carregue no {TECLA_NOME} outra vez e \
         a bala sai para o lado NOVO\n\
         (3) o retangulo CINZENTO da direita ouve a MESMA tecla e atira SEMPRE para cima: e' o \
         CONTROLO, e a unica diferenca entre os dois e' uma caixa de marcar\n\
         (4) role o painel da direita ate' ao fim (o «Heroi» ja' esta' escolhido): a seccao TRIGGER \
         e' quem ouve a tecla, e a caixa «Aim from spawner» da seccao FACTORY e' a unica diferenca \
         para o cinzento\n\
         (5) na barra de CIMA carregue em `Pause`: o {TECLA_NOME} deixa de disparar e volta a ser \
         do editor. `Play`, ao lado, devolve-o\n\
         (6) na seccao TRIGGER troque o nome da accao de `{ACCAO}` para `{ACCAO_SEM_TECLA}`: ela \
         EXISTE e nao tem tecla, e o painel diz isso com OUTRA frase — porque a cura e' outra \
         (ligar uma tecla, nao criar a accao). Escreva `fier` e compare: o aviso muda\n\
         (7) deu errado se: nada sai ao carregar no {TECLA_NOME} · as DUAS balas saem para o mesmo \
         lado · alguma coisa sai com o relogio PARADO · ou o passo (6) nao muda o aviso"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "trigger_smoke_tests.rs"]
mod tests;
