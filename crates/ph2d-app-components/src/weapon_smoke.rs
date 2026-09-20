//! ⭐⭐⭐ **Smoke da ARMA.** `PH2D_WEAPON_SMOKE=1`.
//!
//! # A cena: **três torretas, uma tecla, e uma diferença cada**
//!
//! As três ouvem o MESMO gatilho na MESMA tecla, e as três atiram para cima. O que muda de coluna
//! para coluna é **uma** coisa — e cada uma é um dos três buracos que a medição do §5.0 achou:
//!
//! | coluna | o que ela demonstra | o buraco que ela fecha |
//! |---|---|---|
//! | **esquerda** (azul) | segurar dá **quatro** tiros por segundo, o pente acaba aos seis e ela recarrega sozinha | o **RITMO** e o **PENTE** |
//! | **meio** (cinzenta) | o CONTROLO: o gatilho liga **direto** à fábrica, sem arma — segurar dá uma MANGUEIRA | *é isto que a composição de hoje dá* |
//! | **direita** (laranja) | cinco chumbos num leque, com duas cartuchas e uma recarga lenta | o **ESPALHAMENTO** |
//!
//! ⚠️⚠️ **O CONTROLO é o que torna a wave legível**, e é a lei desta linha desde o `#13`: sem ele o
//! dono vê três torretas a disparar e não tem como saber o que a arma comprou. *Com ele, a
//! diferença entre «um componente» e «nenhum» está lado a lado, com a mesma tecla.*
//!
//! ⛔ **Nenhuma delas anda**, e é deliberado: o sujeito desta cena é a ARMA, e um herói que se move
//! poria a MIRA na tela — que é outra wave (o gatilho, #24) e já está paga.
//!
//! # ⚠️ A acção e a tecla vêm da cena do GATILHO, pela mesma const
//!
//! ⛔ Re-declará-las aqui seria a segunda resposta a *«que tecla dispara neste app?»*, e as duas
//! divergiriam no dia em que uma fosse afinada. A medição que escolheu o `Q` — e o report do dono
//! que rejeitou o espaço — vivem lá, uma vez.
//!
//! ⚠️ Se a linha `[weapon-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ActionEdge, ActionTriggerRow, Counter, CounterRuntime, Entity, Factory, Lifetime, MasterRoot,
    Name, SignalOnAction, Transform, Visibility, WeaponFire, World,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, ProjectileMotion, RigidBody};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 1;

/// A acção e a tecla são as do GATILHO — ver o cabeçalho.
pub use crate::trigger_smoke::{ACCAO, TECLA, TECLA_NOME};

/// O sinal que o gatilho publica — o que a ARMA ouve, e o que o CONTROLO liga direto à fábrica.
pub const GATILHO: &str = "fire";
/// O sinal que a arma da esquerda publica a cada tiro.
pub const TIRO: &str = "tiro";
/// O da caçadeira.
pub const CHUMBO: &str = "chumbo";
/// O nome do pente da arma da esquerda.
pub const PENTE: &str = "ammo";
/// ⚠️ **O da caçadeira é OUTRO**, de propósito: dois pentes com o mesmo nome fariam as duas armas
/// partilhar munição, e o dono veria a esquerda ficar seca por causa de um tiro da direita.
pub const CARTUCHAS: &str = "shells";

/// Quantas balas a arma da esquerda leva. ⚠️ **Seis, e não uma**: com um pente de um, «acabou» e
/// «a cadência» leem-se iguais.
pub const PENTE_N: i64 = 6;
/// Quantas cartuchas a caçadeira leva.
pub const CARTUCHAS_N: i64 = 2;

/// ⭐⭐⭐ **O DEPÓSITO da caçadeira** — o contador de que a recarga TIRA.
///
/// ⚠️ **Ele vive numa CAIXA e não na arma**, e não por gosto: uma entidade tem **um** `Counter` e o
/// pente já o ocupa. ⭐ De graça, isso torna-o um objecto da cena que o dono vê na Hierarquia e
/// cujo número desce à frente dele.
pub const RESERVA: &str = "shell box";

/// ⚠️ **`6` são TRÊS recargas** de um pente de [`CARTUCHAS_N`], que é o mínimo que mostra uma
/// escada: com uma o dono não distingue *«acabou»* de *«nunca funcionou»*.
pub const RESERVA_N: i64 = 6;
/// A cadência da arma da esquerda, em ms — quatro tiros por segundo.
pub const CADENCIA_MS: u64 = 250;
/// Quanto ela demora a recarregar.
pub const RECARGA_MS: u64 = 800;
/// Quantos chumbos a caçadeira cospe de cada vez.
pub const CHUMBOS: u32 = 5;
/// A abertura do leque, em graus.
pub const LEQUE_GRAUS: f32 = 30.0;

/// ⭐⭐⭐ **Onde as três se põem, e a linha É medida.**
///
/// ⚠️ **O `±4` sai da BANDA VISÍVEL MEDIDA NA FOTO** desta cena (`1930×1012`, 100 % de zoom, com a
/// régua do transporte aberta): a régua do canvas põe o `0` no ecrã em `y = 491 px` e o `−100` em
/// `588`, logo `1 m` mede `98 px` e a banda vai de **`+4,09 m`** a **`−1,19 m`**; de lado, de
/// `−7,4` a `+6,2`. *Uma altura não diz ONDE* — por isso o gate mede a CAIXA de cada peça, e não a
/// envergadura da cena.
const COLUNA_X: [f32; 3] = [-4.0, 0.0, 4.0];
/// ⭐⭐⭐ **A altura das três torretas, e ela foi CORRIGIDA POR UMA FOTO.**
///
/// ⛔ A 1.ª redacção pôs `−0,8` e as três saíram **CORTADAS pela borda de baixo**: com meia altura
/// de `0,5 m`, o pé delas ficava em `−1,3` e a banda acaba em `−1,19`. ⚠️ *Os sete gates da cena
/// estavam VERDES* — eles mediam o CENTRO de cada peça, e um centro dentro da banda não diz que a
/// peça inteira cabe. É a mesma família do gate do `#25` que media a ALTURA em vez da POSIÇÃO.
const TORRETA_Y: f32 = -0.4;
/// ⭐ **O alcance de uma bala, em metros** — `4` e não `7`: a `4` ela morre DENTRO da banda
/// (`−0,4 + 4,0 = 3,6`, contra o tecto medido de `4,09`), e o dono vê o voo inteiro. Com mais, ela
/// sai por cima e ele vê só a partida.
const ALCANCE: f32 = 4.0;

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const ARMA_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const CTRL_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];
const CACADEIRA_RGBA: [f32; 4] = [0.95, 0.55, 0.20, 1.0];
const BALA_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];
const BALA_CTRL_RGBA: [f32; 4] = [0.70, 0.72, 0.75, 1.0];
const CHUMBO_RGBA: [f32; 4] = [0.99, 0.72, 0.42, 1.0];
/// A caixa de munições. ⚠️ **A COR da caçadeira, mais escura** — é isso que diz a quem olha que
/// eles são o mesmo assunto, sem uma linha de texto.
const CAIXA_RGBA: [f32; 4] = [0.62, 0.36, 0.13, 1.0];
/// Onde a caixa fica. ⚠️ **MEDIDO contra a banda** (`|x| + meia-largura <= 6,2`): `5,7 + 0,35`
/// dá `6,05`, e é o gate `cada_peca_cabe_na_banda_visivel` que o afirma. ⛔ E ela fica FORA do
/// leque da caçadeira (que a `4` m de alcance abre até `x ≈ 5,03`), senão o chumbo dela ricochetava
/// no próprio depósito.
const CAIXA_XY: [f32; 2] = [5.7, TORRETA_Y];

/// **A receita que esta fábrica ainda vai apontar** — o marcador de MONTAGEM da irmã do `#11`.
///
/// ⚠️ Ele existe porque a identidade só é atribuída depois: o `Factory::master` é um `StableId`, e
/// no instante em que a cena monta o mestre ainda não tem um.
#[derive(bevy_ecs::component::Component, Clone, Copy)]
struct Pendente(Entity);

/// A RECEITA de uma bala.
///
/// ⚠️ **O `Lifetime` não é enfeite** — sem ele cada tiro fica na cena para sempre, e a coluna do
/// meio (que cospe sessenta por segundo) enche o mundo em dez segundos.
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
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            // ⚠️ Um rectângulo COMPRIDO: é ele que torna o LEQUE da caçadeira visível — cinco bolas
            // redondas num cone leem-se como cinco bolas.
            Sprite::atlas(WHITE_TILE_KEY, [0.14, 0.42], cor),
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 7.0,
                    range: ALCANCE,
                    face_velocity: true,
                    ..ProjectileLaw::default()
                },
                0,
            ),
            Lifetime {
                duration_us: 2_000_000,
                ..Lifetime::default()
            },
        ))
        .id()
}

/// O gatilho — o MESMO nas três. ⚠️ **`Hold` e não `Press`**, e é o que torna a cena legível: a
/// cadência só se vê a SEGURAR. *Com `Press`, as três dariam um tiro por toque e a arma não teria
/// o que demonstrar.*
fn gatilho() -> SignalOnAction {
    SignalOnAction(vec![ActionTriggerRow {
        action: ACCAO.to_owned(),
        edge: ActionEdge::Hold,
        signal: GATILHO.to_owned(),
    }])
}

/// Uma torreta: o corpo, o gatilho e a fábrica. A ARMA (quando existe) entra por fora.
fn torreta(
    world: &mut World,
    nome: &str,
    cor: [f32; 4],
    x: f32,
    fabrica: Factory,
    mestre: Entity,
) -> Entity {
    // ⚠️ **Rodada 90°**: as três apontam para CIMA, e é isso que faz as três colunas serem
    // comparáveis — com rumos diferentes, o dono leria o rumo como a diferença.
    let mut pose = Transform::from_translation(Vec2::new(x, TORRETA_Y));
    pose.rotation = std::f32::consts::FRAC_PI_2;
    world
        .spawn((
            Name::new(nome),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 0.45], cor),
            pose,
            gatilho(),
            fabrica,
            Pendente(mestre),
        ))
        .id()
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — a arma da esquerda.
fn cena_um(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**, e isto não é estilo: desde a cura de 15/09 a ordem das raízes é a
    // ordem de CRIAÇÃO, logo quem nasce primeiro desenha por baixo.
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));

    let bala = receita(world, "Bala", BALA_RGBA);
    let bala_ctrl = receita(world, "Bala (controlo)", BALA_CTRL_RGBA);
    let chumbo = receita(world, "Chumbo", CHUMBO_RGBA);

    // ⭐ (1) A ARMA — a fábrica ouve o sinal DELA, nunca o gatilho.
    let arma = torreta(
        world,
        "Arma",
        ARMA_RGBA,
        COLUNA_X[0],
        Factory {
            master: 0,
            on_signal: TIRO.to_owned(),
            burst: 1,
            aim_from_spawner: true,
            ..Factory::default()
        },
        bala,
    );
    world.entity_mut(arma).insert((
        WeaponFire {
            on_signal: GATILHO.to_owned(),
            cooldown_ms: CADENCIA_MS,
            ammo_counter: PENTE.to_owned(),
            reload_ms: RECARGA_MS,
            reload_on: String::new(),
            on_fire: TIRO.to_owned(),
            on_empty: String::new(),
            on_reloaded: String::new(),
            // ⚠️ **Sem depósito: reserva INFINITA**, que é o comportamento que o dono aprovou.
            reserve_counter: String::new(),
        },
        // ⭐⭐ **O PENTE é um contador**, e é isso que o põe no Inspector e no HUD de graça.
        Counter {
            name: PENTE.to_owned(),
            start: PENTE_N,
            keep_on_restart: false,
        },
        CounterRuntime { value: PENTE_N },
    ));

    // ⭐⭐ (2) O CONTROLO — **sem arma**: o gatilho liga DIRETO à fábrica.
    torreta(
        world,
        "Sem arma (controlo)",
        CTRL_RGBA,
        COLUNA_X[1],
        Factory {
            master: 0,
            on_signal: GATILHO.to_owned(),
            burst: 1,
            aim_from_spawner: true,
            ..Factory::default()
        },
        bala_ctrl,
    );

    // ⭐ (3) A CAÇADEIRA — o leque vive na FÁBRICA, ao lado do `burst`.
    let cacadeira = torreta(
        world,
        "Cacadeira",
        CACADEIRA_RGBA,
        COLUNA_X[2],
        Factory {
            master: 0,
            on_signal: CHUMBO.to_owned(),
            burst: CHUMBOS,
            spread_deg: LEQUE_GRAUS,
            aim_from_spawner: true,
            ..Factory::default()
        },
        chumbo,
    );
    world.entity_mut(cacadeira).insert((
        WeaponFire {
            on_signal: GATILHO.to_owned(),
            cooldown_ms: 700,
            ammo_counter: CARTUCHAS.to_owned(),
            reload_ms: 1_200,
            reload_on: String::new(),
            on_fire: CHUMBO.to_owned(),
            on_empty: String::new(),
            on_reloaded: String::new(),
            // ⭐⭐⭐ **A caçadeira é a única com DEPÓSITO, e a escolha é medida:** as outras duas
            // foram aprovadas pelo dono com reserva infinita, e a lição delas é o RITMO. Aqui a
            // lição já é *«poucos tiros, devagar»* — é a única coluna em que um depósito que ACABA
            // se lê, e as outras ficam byte-idênticas a servir de CONTROLO.
            reserve_counter: RESERVA.to_owned(),
        },
        Counter {
            name: CARTUCHAS.to_owned(),
            start: CARTUCHAS_N,
            keep_on_restart: false,
        },
        CounterRuntime { value: CARTUCHAS_N },
    ));

    // ⭐⭐⭐ **A CAIXA DE MUNIÇÕES** — o depósito é um objecto da CENA, não um campo escondido.
    //
    // ⚠️ **Sem `Sprite` ela seria um número invisível**: o dono precisa de a poder escolher no
    // canvas para ver o `Counter` dela descer, e a lei do #15 já o pagou (*uma cena certa como
    // DADOS e impossível como GESTO*).
    world.spawn((
        Name::new("Shell Box"),
        Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], CAIXA_RGBA),
        Transform::from_translation(Vec2::new(CAIXA_XY[0], CAIXA_XY[1])),
        Counter {
            name: RESERVA.to_owned(),
            start: RESERVA_N,
            keep_on_restart: false,
        },
        CounterRuntime { value: RESERVA_N },
    ));

    arma
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
    /// A ARMA, que nasce escolhida — o roteiro manda ver a secção *Weapon* no painel da direita.
    pub escolhido: u64,
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
///
/// ⚠️ Separada do prólogo pela razão das irmãs: é a metade que um gate consegue correr — o resto
/// pede o relógio e o Input Map, que não são o mundo.
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    resolver_receitas(world);
    // ⚠️ **CONTADO e não escrito à mão:** quantas recargas o depósito paga sai dos dois números
    // que a cena declara — *um literal aqui envelhece no dia em que um deles mudar*.
    let recargas = RESERVA_N / CARTUCHAS_N;
    println!(
        "[weapon-smoke] cena=1  tecla={TECLA_NOME}  accao=«{ACCAO}»  pentes: {PENTE}={PENTE_N} · \
         {CARTUCHAS}={CARTUCHAS_N}\n\
         (1) SEGURE o {TECLA_NOME} e olhe as tres colunas ao mesmo tempo\n\
         (2) a AZUL da esquerda atira ~4 vezes por segundo, para DEPOIS de {PENTE_N} tiros, e volta \
         sozinha ~1 segundo depois: e' o RITMO e o PENTE\n\
         (3) a CINZENTA do meio nao tem arma — o gatilho liga direto a' fabrica, e segurar da' uma \
         MANGUEIRA de balas: e' o que a composicao de hoje da'\n\
         (4) a LARANJA da direita cospe {CHUMBOS} chumbos num LEQUE, tem so' {CARTUCHAS_N} \
         cartuchas e demora mais a recarregar\n\
         (4-bis) e SO' ELA tem DEPO'SITO: a CAIXA castanha ao lado dela tem {RESERVA_N} cartuchas, \
         e cada recarga tira de la'. Escolha a «Shell Box» no canvas e veja o numero descer no \
         Inspector; ao fim de {recargas} recargas ela fica seca PARA SEMPRE — as outras duas nao, \
         porque nao tem deposito nenhum\n\
         (5) a «Arma» ja' esta' escolhida: role o painel da direita ate' a' seccao WEAPON e veja a \
         linha «{PENTE_N} of {PENTE_N} rounds» descer enquanto voce segura a tecla, e dizer \
         «Reloading…» quando o pente acaba\n\
         (6) na barra de CIMA carregue em `Pause`: nenhuma das tres dispara, e o {TECLA_NOME} volta \
         a ser do editor. `Play` devolve-o\n\
         (7) deu errado se: a azul cuspir tao depressa como a cinzenta · nunca ficar sem municao · \
         os {CHUMBOS} chumbos sairem todos no mesmo rumo · a caixa castanha nao descer · a LARANJA \
         continuar a disparar com a caixa a zero · ou alguma coisa sair com o relogio PARADO"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "weapon_smoke_tests.rs"]
mod tests;
