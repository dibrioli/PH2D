//! ⭐⭐⭐ **Smoke do GOLPE** (suplente #24). `PH2D_DANO_SMOKE=1`.
//!
//! # A cena: **duas fileiras de alvos, e uma caixa de diferença**
//!
//! O herói azul atira com o **`Q`** (o gatilho de 18/09). Cada fileira de alvos reage ao próprio
//! golpe, e a única diferença entre elas é a **cerca**:
//!
//! | fileira | a cerca da linha | o que acontece a UM tiro |
//! |---|---|---|
//! | **de cima** | `From Myself` | morre **o alvo que levou o tiro**, e a bala com ele |
//! | **de baixo** (o CONTROLO) | `From Anyone` | morrem **os três**, porque o sinal é um nome global |
//!
//! ⚠️⚠️ **Os dois sinais têm nomes DIFERENTES de propósito, e não é a diferença que se demonstra:**
//! com o mesmo nome as duas experiências misturavam-se — um tiro na fileira de cima apagaria a de
//! baixo, porque a linha sem cerca ouve **tudo o que se chame assim**. *O nome isola as duas
//! fileiras; o que se mede é a cerca.*
//!
//! # ⛔ E os alvos NASCEM, não são postos à mão
//!
//! O `Destroy` só tira quem nasceu numa corrida ([`ph2d_ecs::is_transient`]) — apagar um objecto do
//! DOCUMENTO durante a corrida tira-o do documento, e o `Ctrl+Z` herdaria a remoção. ⇒ as duas
//! fileiras saem de duas **fábricas** (TOP-20 #11) que um **relógio** (#2) arranca ao entrar a
//! corrida. *Uma cena com alvos de documento ensinaria o contrário do que acontece* — a espécie que
//! o `CLAUDE.md` §5.0 chama de pior que uma cena ausente.
//!
//! ⚠️ **Os alvos são SENSORES**: o que se quer é que eles **reportem** o toque, não que travem a
//! bala. É a mesma forma da armadilha do smoke das Tags.
//!
//! # ⛔⛔ E a 1.ª redacção desta cena SUICIDOU-SE, com a foto a dizê-lo
//!
//! Os alvos nasciam espalhados numa **faixa** (`SpawnAt::Area`), e a `seed` pôs dois deles a
//! **tocarem-se**. Cada um é um sensor que grita `golpe`, cada um ouve o seu, e a cerca `Myself`
//! fez **exactamente o que promete**: eles mataram-se um ao outro no primeiro quadro. O log deu a
//! causa em duas linhas (*«golpe <- fisica, X tocou Y»* e *«Y tocou X»*) e a foto deu o efeito —
//! `0 entities`.
//!
//! ⇒ **duas curas, e as duas são composição que esta linha já shipou:** os alvos nascem em **pontos
//! MARCADOS** (a recusa medida do `SpawnPoint`, TOP-20 #11) em vez de ao acaso, e cada um leva um
//! **`SignalTagFilter`** (#9) que só deixa passar o toque de uma **bala**. *Sem o filtro, o herói a
//! caminhar sobre um alvo matava-o.*
//!
//! ⚠️ Se a linha `[dano-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::tags::Tags;
use ph2d_ecs::{
    Entity, Factory, Lifetime, MasterRoot, Name, Pick, SignalAction, SignalActions, SignalFrom,
    SignalOnAction, SignalTarget, SignalVerb, SpawnAt, Timer, Timers, Transform, Visibility, World,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, ProjectileMotion, RigidBody, SignalOnHit, SignalTagFilter,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_tags::{TagId, TagTree};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — CONTADO do `match` do [`montar`].
pub const CENAS: u32 = 1;

/// A acção que o prólogo cria no Input Map, e a tecla dela — as mesmas do gatilho de 18/09.
pub const ACCAO: &str = "fire";
/// O nome do sinal do gatilho.
pub const SINAL: &str = "tiro";
/// O `Q` — ⚠️ **medida e não escolhida**; ver [`crate::trigger_smoke::TECLA`], que é a fonte.
pub const TECLA_NOME: &str = "Q";

/// O sinal que a fileira de CIMA grita ao ser tocada.
const GOLPE_CERCADO: &str = "golpe";
/// O da fileira de BAIXO — ⚠️ outro nome, para as duas experiências não se misturarem (ver o
/// cabeçalho). *Ele não é a diferença que se demonstra.*
const GOLPE_SOLTO: &str = "golpe-solto";
/// O sinal que arranca as duas fábricas.
const COMECAR: &str = "comecar";

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const ALVO_CERCADO_RGBA: [f32; 4] = [0.85, 0.32, 0.30, 1.0];
const ALVO_SOLTO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];
const BALA_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];

/// Quantos alvos por fileira.
const POR_FILEIRA: u32 = 3;
/// O `y` de cada fileira.
const Y_CIMA: f32 = 2.2;
const Y_BAIXO: f32 = -2.2;
/// O vão entre dois alvos, em metros — ver [`fileira`].
const ESPACO: f32 = 2.2;
/// ⭐⭐ **O LADO de um alvo, e ele é a JANELA DE MIRA do passo 3 — medido, não escolhido.**
///
/// O herói nasce à altura da fileira de cima, logo o passo 2 não pede pontaria nenhuma; o passo 3
/// pede que ele desça `4,4` m e pare **à altura** dos cinzentos. A janela é `LADO/2 + raio da bala`
/// = `0,80` m de cada lado (`0,55` com um alvo de `0,9`), e a `4` m/s isso é `0,45` s de tecla
/// contra `0,27` — *o dobro do tempo para o dedo acertar*.
///
/// ⛔ **E o vão FICA visível:** `ESPACO − LADO = 0,8` m, quatro vezes o diâmetro da bala — os alvos
/// não se tocam, que é a metade geométrica da cura do cabeçalho.
const LADO: f32 = 1.4;

/// **A receita que esta fábrica ainda vai apontar** — o marcador de MONTAGEM da irmã do `#11`: a
/// identidade só é atribuída depois, e semear o campo com os bits da entidade seria escrever no
/// componente a coisa que o `CLAUDE.md` proíbe.
#[derive(bevy_ecs::component::Component, Clone, Copy)]
struct Pendente(Entity);

/// Uma linha de tabela.
fn linha(on: &str, verb: SignalVerb, from: SignalFrom, target_by: SignalTarget) -> SignalAction {
    SignalAction {
        on: on.to_owned(),
        target: String::new(),
        verb,
        arg: String::new(),
        target_by,
        from,
    }
}

/// **A RECEITA de um alvo** — um sensor que grita ao ser tocado, e uma tabela com DUAS linhas.
///
/// ⚠️ **As duas linhas são o par que fecha o golpe:** uma tira o ALVO da cena, a outra tira a BALA
/// (`Who Hit`, o outro lado do contacto). Sem a segunda a bala atravessa e continua a voar.
fn receita_do_alvo(
    world: &mut World,
    nome: &str,
    sinal: &str,
    from: SignalFrom,
    bala: TagId,
) -> Entity {
    world
        .spawn((
            Name::new(nome),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], alvo_cor(from)),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                shape: ColliderShape::Cuboid {
                    half_x: LADO / 2.0,
                    half_y: LADO / 2.0,
                },
                ..Collider::default()
            },
            SignalOnHit(sinal.to_owned()),
            // ⛔⛔ **SÓ uma BALA o acorda** (TOP-20 #9). Sem este filtro, o herói a caminhar sobre
            // um alvo matava-o — e dois alvos que se tocassem matavam-se um ao outro, que foi o que
            // a 1.ª redacção desta cena de facto fez.
            SignalTagFilter(bala.0),
            SignalActions(vec![
                linha(sinal, SignalVerb::Destroy, from, SignalTarget::Named),
                linha(sinal, SignalVerb::Destroy, from, SignalTarget::Other),
            ]),
        ))
        .id()
}

const fn alvo_cor(from: SignalFrom) -> [f32; 4] {
    match from {
        SignalFrom::Myself => ALVO_CERCADO_RGBA,
        SignalFrom::Anyone => ALVO_SOLTO_RGBA,
    }
}

/// **Uma fileira**: três pontos MARCADOS e a fábrica que nasce neles ao [`COMECAR`].
///
/// ⛔⛔ **Pontos marcados e não uma faixa**, e a razão é MEDIDA: com `SpawnAt::Area` a semente pôs
/// dois alvos a tocarem-se, cada um gritou o golpe dele, e eles mataram-se no primeiro quadro (ver
/// o cabeçalho). *Um ponto marcado é uma posição que o artista escolheu; uma faixa é um sorteio.*
///
/// ⚠️ **`Pick::Cycle` com `burst = POR_FILEIRA`** dá exactamente um por ponto, em roda-viva.
fn fileira(world: &mut World, nome: &str, y: f32, receita: Entity, ponto: TagId) {
    // ⚠️ **O que separa dois alvos é `ESPACO − LADO`** (ver o [`LADO`]): tem de ser visível ao olho
    // E maior do que o colisor, senão a cura do cabeçalho é uma coincidência de números.
    for (i, x) in [0.0f32, ESPACO, 2.0 * ESPACO].into_iter().enumerate() {
        world.spawn((
            Transform::from_translation(Vec2::new(2.0 + x, y)),
            Name::new(format!("{nome} {}", i + 1)),
            Visibility::visible(),
            Tags::from_ids([ponto]),
        ));
    }
    world.spawn((
        Name::new(nome),
        Transform::from_translation(Vec2::new(2.0, y)),
        Factory {
            master: 0, // resolvido em `resolver_receitas`
            on_signal: COMECAR.to_owned(),
            at: SpawnAt::Tagged {
                tag: ponto.0,
                pick: Pick::Cycle,
            },
            burst: POR_FILEIRA,
            // ⚠️ **Uma vez só** — o relógio não repete, e o `Max Total` torna a intenção legível
            // para quem abre o painel.
            total_max: POR_FILEIRA,
            ..Factory::default()
        },
        Pendente(receita),
    ));
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — o herói, porque o roteiro fala da secção dele.
fn cena_um(world: &mut World, tree: &mut TagTree) -> Entity {
    // ⭐ As duas tags que a cena autora: a que marca os POSTOS e a que diz *«isto é uma bala»*.
    let posto_cima = tree.create("Posto/Cima").expect("cria");
    let posto_baixo = tree.create("Posto/Baixo").expect("cria");
    let bala_tag = tree.create("Bala").expect("cria");
    // ⚠️⚠️ **O CHÃO PRIMEIRO**, e isto não é estilo: a ordem das raízes é a de CRIAÇÃO, logo quem
    // nasce primeiro desenha por baixo (a cura de 15/09).
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));

    // ⭐ O RELÓGIO que arranca as duas fábricas — ⚠️ `autostart` e SEM repetição: os alvos nascem
    // uma vez, ao entrar a corrida, e o `Home` devolve a cena ao princípio.
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

    let cercado = receita_do_alvo(
        world,
        "Alvo (com cerca)",
        GOLPE_CERCADO,
        SignalFrom::Myself,
        bala_tag,
    );
    let solto = receita_do_alvo(
        world,
        "Alvo (sem cerca)",
        GOLPE_SOLTO,
        SignalFrom::Anyone,
        bala_tag,
    );
    fileira(world, "Fileira de cima", Y_CIMA, cercado, posto_cima);
    fileira(world, "Fileira de baixo", Y_BAIXO, solto, posto_baixo);

    // ⭐ A RECEITA da bala — ⚠️ o `Lifetime` não é enfeite: sem ele cada tiro que não acerta fica
    // parado no fim do alcance (a lei do #12, *sem ela o #11 e o #14 VAZAM*).
    let bala = world
        .spawn((
            Name::new("Bala"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.16], BALA_RGBA),
            // ⭐ **A bala LEVA a tag**, e é ela que o filtro dos alvos lê. ⚠️ A cópia herda-a: uma
            // tag é um componente como outro qualquer, e a cópia profunda leva-o.
            Tags::from_ids([bala_tag]),
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
            Lifetime {
                duration_us: 3_000_000,
                ..Lifetime::default()
            },
        ))
        .id();

    // ⭐ O HERÓI: anda com as setas, roda para onde anda, e a arma dele aponta para onde ele aponta.
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
            Sprite::atlas(WHITE_TILE_KEY, [1.1, 0.5], HEROI_RGBA),
            // ⭐⭐⭐ **À ALTURA da fileira de cima, e isso é o passo 2 do roteiro.** Com o herói em
            // `y = 0` o tiro recto passava **entre** as duas fileiras e o dono tinha de descobrir
            // a pontaria antes de ver a lei; nascendo alinhado, o passo 2 é **uma tecla**. ⚠️ E o
            // alcance chega: `14` m contra os `8,5` até ao 1.º alvo.
            Transform::from_translation(Vec2::new(-6.5, Y_CIMA)),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::Free,
                rotation: ph2d_topdown::rotation::RotationMode::ToMovement,
                ..TopDownLaw::default()
            }),
            SignalOnAction(vec![ph2d_ecs::ActionTriggerRow {
                action: ACCAO.to_owned(),
                edge: ph2d_ecs::ActionEdge::Press,
                signal: SINAL.to_owned(),
            }]),
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
pub fn montar(world: &mut World, tree: &mut TagTree, _nivel: u32) -> Montada {
    let escolhido = cena_um(world, tree);
    resolver_receitas(world);
    println!(
        "[dano-smoke] cena=1  tecla={TECLA_NOME}  alvos={POR_FILEIRA} por fileira\n\
         (1) espere um instante: as duas fileiras NASCEM — tres alvos VERMELHOS em cima e tres \
         CINZENTOS em baixo (eles vem de fabricas, e por isso podem sair da cena)\n\
         (2) carregue no {TECLA_NOME}: o heroi azul ja' nasce A' ALTURA dos vermelhos, logo nao \
         precisa de mirar. Morre O QUE LEVOU O TIRO, e a bala com ele — os outros dois ficam\n\
         (3) carregue com o rato num alvo VERMELHO que sobrou e depois num CINZENTO: no painel da \
         direita, a seccao `Signal Actions` de cada um. A diferenca entre as duas fileiras e' UMA \
         caixa — `From Myself` nos vermelhos, `From Anyone` nos cinzentos. As duas linhas fazem \
         `Destroy`, e a segunda aponta a `Who Hit`, que e' a bala\n\
         (4) agora segure a seta para BAIXO ate' o heroi ficar a' altura dos CINZENTOS, depois a \
         seta para a DIREITA (ele vira-se para onde anda) e carregue no {TECLA_NOME}: morrem OS \
         TRES. E' o CONTROLO — a mesma tabela sem a cerca\n\
         (5) carregue em STOP na regua de baixo: o {TECLA_NOME} deixa de disparar. `Home` rebobina \
         e os alvos voltam — eles nasceram na corrida, logo nao estao no ficheiro\n\
         (6) deu errado se: os alvos nao nascem · o tiro de cima leva mais do que um · o de baixo \
         leva so' um · a bala atravessa o alvo e continua · ou carregar no {TECLA_NOME} nao faz nada"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "dano_smoke_tests.rs"]
mod tests;
