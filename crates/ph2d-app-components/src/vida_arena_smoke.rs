//! ⭐⭐⭐ **Smoke da ARENA — um jogo pequeno com vida, do princípio ao fim** (plano 28, W7).
//! `PH2D_VIDA_SMOKE=4`.
//!
//! # A cena: **tudo o que as waves W2–W6 trouxeram, a jogar ao mesmo tempo**
//!
//! Nenhuma peça é nova: cada uma já tem a sua cena e os seus gates. O que esta cena prova é que
//! elas **compõem** num jogo — com uma vida que se perde, se ganha e acaba, e uma corrida que
//! recomeça sozinha.
//!
//! | peça | o que faz | de onde vem |
//! |---|---|---|
//! | o **herói** (azul) | vida `100`, pisca depois de um golpe, mostra os números | W2 · W5 |
//! | o **placar** (canto de cima à esquerda) | a barra da vida do herói, de longe | W4 |
//! | as **duas armas** (`Q` fogo · `J` gelo) | o fogo deixa QUEIMADURA | W6 |
//! | a **Salamandra** (vermelha, à direita) | vida `60`, **imune** ao fogo, **fraca** ao gelo (`×2`) | W6 |
//! | os **morcegos** (roxos) | nascem de 3 em 3 s e PERSEGUEM o herói à volta do muro; batem `15` e somem | #14 · W2 · nav W8 |
//! | o **muro** (cinzento, à esquerda do herói) | só se passa por BAIXO dele | nav W8 |
//! | a **lava** (laranja, em baixo à esquerda) | não golpeia: QUEIMA enquanto se pisa, e ainda depois | W6 |
//! | o **coração** (rosa) | nasce de 6 em 6 s e CURA `25` — o herói absorve o tipo `cura` | W6 |
//! | a **morte do herói** | um segundo e meio depois a corrida RECOMEÇA sozinha | `RestartRun` |
//!
//! # ⭐⭐ A LAVA não dá golpe nenhum, e é de propósito (medido na lei)
//!
//! O herói tem **invencibilidade** depois de um golpe (a janela contra os morcegos), e um dano por
//! segundo que golpeia é **barrado** por ela: a lava daria `amount × dt` uma vez por janela, com um
//! número `0.1` a subir. ⇒ a lava tem `amount = 0` e é **só** a queimadura: um golpe de zero não
//! arma a invencibilidade (`ph2d_health::Vida::leva` só arma com `d > 0`), e um PULSO passa pela
//! invencibilidade sem a armar — logo pisar a lava renova a queimadura a cada tique e os pulsos
//! caem a cada meio segundo, e sair dela deixa ainda [`LAVA_DEPOIS_S`] de queimadura.
//!
//! # ⭐ O CORAÇÃO é da equipa dos MONSTROS, e é isso que o protege
//!
//! Um dano sem equipa fere toda a gente: um morcego que passasse pelo coração levaria `25` de dano
//! e **gastava-o** (o `Vanish` é de quem toca, cure ou não). Com a equipa dos monstros ele não toca
//! em nenhum monstro — e o herói, que não é monstro, apanha-o. ⚠️ **O coração some mesmo com a vida
//! cheia** (a lei do `Vanish` é o toque, não o efeito) — o roteiro di-lo.
//!
//! # ⭐⭐ (plano 30, W8) Os morcegos NAVEGAM — e a lava, que os queima, eles EVITAM
//!
//! Um morcego é um `NavAgent` sobre um `TopDownPlayer` (a cópia da fábrica leva os dois: estão
//! registados). Ele tem vida e a lava, sem equipa e do tipo fogo, fere-o ⇒ pela regra do
//! `Damage::magoa` («tira» inclui a queimadura que dura) a lava é um furo na malha DELE, e um herói
//! que se refugia nela vê os morcegos ESPERAR na borda. **É o que a cena ensina, de propósito**: é a
//! decisão do dono (plano 30 §11.1), e o tutorial 03 mostra o outro lado — a Salamandra, imune ao
//! fogo, posta a perseguir, atravessa-a.
//!
//! # ⛔ Onde as peças moram
//!
//! Tudo o que o roteiro manda ver fica FORA da coluna dos avisos de sinal
//! ([`crate::vida_smoke::COLUNA_DOS_AVISOS_X`]): à ESQUERDA a lava, o coração, o ninho dos morcegos
//! e o placar; à DIREITA o herói e a Salamandra. O herói atravessa a coluna para ir buscar o
//! coração — atravessar não é morar. ERRO DE COMPILAÇÃO a guardá-lo.
//!
//! ⚠️ Se a linha `[vida-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ActionEdge, ActionTriggerRow, ChildOf, Entity, Factory, MasterRoot, Name, SignalAction,
    SignalActions, SignalFrom, SignalOnAction, SignalTarget, SignalVerb, Timer, Timers, Transform,
    Visibility, World, stable_name_id,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, HealthBar, NavAgent, NavRegion, NavTarget,
    OnHit, ProjectileMotion, Resistance, RigidBody, TopDownPlayer,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

use crate::vida_impacto_smoke::BORDA_ESQUERDA;
use crate::vida_smoke::{
    BORDA_DIREITA, COLUNA_DOS_AVISOS_X, HEROIS, MONSTROS, PLACAR, PLACAR_WH, PLACAR_XY, Pendente,
};
use crate::vida_tipos_smoke::{
    ACCAO_FOGO, ACCAO_GELO, FOGO, GELO, QUEIMA_CADA_S, QUEIMA_POR_S, QUEIMA_S, SINAL_FOGO,
    SINAL_GELO, TECLA_FOGO_NOME, TECLA_GELO_NOME,
};

/// O sinal que arranca a Salamandra.
pub(crate) const COMECAR: &str = "comecar-arena";
/// O relógio do ninho — de quanto em quanto tempo nasce um morcego.
pub(crate) const SINAL_MORCEGO: &str = "morcego";
/// O relógio do coração.
pub(crate) const SINAL_CORACAO: &str = "coracao";
/// ⭐ O que o herói grita ao MORRER — e o que arranca o relógio do recomeço.
pub const CAIU: &str = "heroi-caiu";
/// O relógio do recomeço — ele publica este sinal, e a linha `RestartRun` ouve-o.
pub(crate) const RECOMECAR: &str = "recomecar";
/// O nome do relógio do recomeço (o `StartTimer` da tabela aponta-o por aqui).
pub const RELOGIO_RECOMECO: &str = "recomeco";
/// O que a Salamandra grita ao morrer — a vitória.
pub const VENCEU: &str = "venceu";

/// O nome do herói — o placar e os morcegos apontam-no pelo NOME (a lei do repo: nunca os bits).
pub const HEROI: &str = "Heroi";
/// A vida do herói.
pub const VIDA_DO_HEROI: f32 = 100.0;
/// A invencibilidade do herói depois de um golpe — a janela contra um morcego a seguir ao outro.
pub const INVENCIVEL: f32 = 0.8;
/// Quanto dura o recomeço depois da morte, em µs — o tempo de ler o `heroi-caiu` e ver o herói caído.
pub const RECOMECO_US: u64 = 1_500_000;

/// A vida da Salamandra.
pub const VIDA_DA_SALAMANDRA: f32 = 60.0;
/// O golpe de uma bala — o mesmo das armas da cena `=3`.
pub const DANO: f32 = crate::vida_tipos_smoke::DANO;
/// A vida de um morcego — um tiro de qualquer arma mata-o.
pub const VIDA_DO_MORCEGO: f32 = 10.0;
/// Quanto um morcego tira ao herói.
pub const MORDIDA: f32 = 15.0;
/// De quanto em quanto tempo nasce um morcego, em µs.
pub const MORCEGO_CADA_US: u64 = 3_000_000;
/// Quantos morcegos vivos de uma vez.
pub const MORCEGOS_MAX: u32 = 3;
/// ⭐ A rapidez máxima de um morcego — **mais lenta** que a do herói (`4 m/s`): fugir funciona.
pub const MORCEGO_RAPIDEZ: f32 = 2.2;

/// ⭐ **A queimadura da LAVA** — pontos por segundo e o intervalo dos pulsos.
pub const LAVA_POR_S: f32 = 6.0;
/// O intervalo dos pulsos da lava.
pub const LAVA_CADA_S: f32 = 0.5;
/// Quanto a queimadura dura depois de sair da lava.
pub const LAVA_DEPOIS_S: f32 = 2.0;
/// O tipo da lava — o do fogo: a Salamandra, se lá fosse, não sentiria nada.
pub const LAVA_TIPO: &str = FOGO;

/// ⭐ **O CORAÇÃO** — quanto cura, de quanto em quanto tempo nasce, e o tipo que o herói absorve.
pub const CURA: f32 = 25.0;
/// De quanto em quanto tempo nasce um coração (se não houver já um), em µs.
pub const CORACAO_CADA_US: u64 = 6_000_000;
/// O tipo do coração — o herói tem uma resistência a ele que ABSORVE.
pub const CURA_TIPO: &str = "cura";

// ── A geometria (metros de mundo) ───────────────────────────────────────────
/// Onde o herói nasce — à DIREITA do muro, à altura da Salamandra, virado para ela.
pub const HEROI_XY: [f32; 2] = [2.85, 2.1];
/// ⭐ (plano 30, W8) **O MURO**: vertical, logo à direita da coluna dos avisos, do alto até
/// [`MURO_BAIXO`] — a única passagem entre o ninho e o herói é por BAIXO dele.
pub const MURO_X: f32 = 2.05;
/// A meia largura do muro.
pub const MURO_MEIO: f32 = 0.15;
/// Onde o muro acaba em baixo.
pub const MURO_BAIXO: f32 = 0.5;
/// Onde o muro acaba em cima — acima da banda visível, para ninguém o contornar por cima.
pub const MURO_CIMA: f32 = 6.0;
/// ⚠️ O fundo da região andável — o do CANVAS (px `765` ↔ `−2,6` m na foto da W8 a `1930×1040`),
/// e não o [`FUNDO`]: com ele o chão de baixo via-se e nenhum morcego lá entrava.
pub const REGIAO_FUNDO: f32 = -2.5;
/// Onde a Salamandra nasce.
pub const SALAMANDRA_XY: [f32; 2] = [5.6, 2.1];
/// O lado da Salamandra.
pub const LADO_DA_SALAMANDRA: f32 = 1.0;
/// Onde o ninho dos morcegos mora — no canto de cima À ESQUERDA, debaixo do placar.
pub const NINHO_XY: [f32; 2] = [-5.4, 2.9];
/// O lado de um morcego (o corpo é um círculo deste diâmetro).
pub const LADO_DO_MORCEGO: f32 = 0.45;
/// O centro da lava.
pub const LAVA_XY: [f32; 2] = [-4.0, -0.35];
/// O tamanho da lava.
pub const LAVA_WH: [f32; 2] = [2.2, 1.2];
/// Onde o coração nasce — à esquerda, entre o ninho e a lava.
pub const CORACAO_XY: [f32; 2] = [-3.0, 1.4];
/// O lado do coração.
pub const LADO_DO_CORACAO: f32 = 0.4;
/// A banda visível na vertical, medida na foto da cena da arma (a timeline aberta).
pub const TOPO: f32 = 4.09;
/// Idem, o fundo.
pub const FUNDO: f32 = -1.19;

// ⚠️ **Nada mora na coluna dos avisos** e tudo cabe na banda — ERRO DE COMPILAÇÃO (um `assert!`
// sobre constantes num teste o compilador dobra-o). O herói virado para a direita tem meia largura
// `0,45` e meia altura `0,2`.
const _: () = assert!(HEROI_XY[0] - 0.45 >= COLUNA_DOS_AVISOS_X[1] + 0.1);
const _: () = assert!(SALAMANDRA_XY[0] + LADO_DA_SALAMANDRA / 2.0 <= BORDA_DIREITA - 0.2);
const _: () = assert!(HEROI_XY[0] + 0.45 + 1.0 <= SALAMANDRA_XY[0] - LADO_DA_SALAMANDRA / 2.0);
const _: () = assert!(LAVA_XY[0] - LAVA_WH[0] / 2.0 >= BORDA_ESQUERDA + 0.2);
const _: () = assert!(LAVA_XY[0] + LAVA_WH[0] / 2.0 <= COLUNA_DOS_AVISOS_X[0] - 0.1);
const _: () = assert!(LAVA_XY[1] - LAVA_WH[1] / 2.0 >= FUNDO);
const _: () = assert!(CORACAO_XY[0] + LADO_DO_CORACAO / 2.0 <= COLUNA_DOS_AVISOS_X[0] - 0.1);
const _: () = assert!(NINHO_XY[0] - LADO_DO_MORCEGO / 2.0 >= BORDA_ESQUERDA + 0.2);
// O ninho fica debaixo do placar e o coração entre o ninho e a lava — nenhum toca no outro.
const _: () = assert!(NINHO_XY[1] + LADO_DO_MORCEGO / 2.0 < PLACAR_XY[1] - PLACAR_WH[1] / 2.0);
const _: () = assert!(CORACAO_XY[1] - LADO_DO_CORACAO / 2.0 > LAVA_XY[1] + LAVA_WH[1] / 2.0);
const _: () = assert!(CORACAO_XY[1] + LADO_DO_CORACAO / 2.0 < NINHO_XY[1] - LADO_DO_MORCEGO / 2.0);
// A barra da Salamandra cabe por baixo do topo.
const _: () = assert!(SALAMANDRA_XY[1] + BARRA_Y + BARRA_H / 2.0 <= TOPO);
// O morcego é mais lento que o herói — fugir tem de funcionar.
const _: () = assert!(MORCEGO_RAPIDEZ < RAPIDEZ_DO_HEROI);
// O muro fica fora da coluna, o herói (meia largura `0,45`) à direita dele, e a passagem de baixo
// deixa passar a Salamandra (o maior corpo que o tutorial põe a andar: meia diagonal `0,71`).
const _: () = assert!(MURO_X - MURO_MEIO >= COLUNA_DOS_AVISOS_X[1] + 0.1);
const _: () = assert!(HEROI_XY[0] - 0.45 >= MURO_X + MURO_MEIO + 0.1);
const _: () = assert!(MURO_BAIXO - FUNDO > 2.0 * 0.71 + 0.2 && REGIAO_FUNDO < FUNDO);
const _: () = assert!(MURO_BAIXO < HEROI_XY[1] - 1.0 && MURO_CIMA > TOPO);

/// A rapidez do herói (a do molde das três cenas desta família).
pub const RAPIDEZ_DO_HEROI: f32 = 4.0;
/// Onde a barra da Salamandra mora, acima do centro dela.
pub const BARRA_Y: f32 = 0.7;
/// A altura da barra da Salamandra.
pub const BARRA_H: f32 = 0.12;

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const SALAMANDRA_RGBA: [f32; 4] = [0.88, 0.30, 0.22, 1.0];
const MORCEGO_RGBA: [f32; 4] = [0.62, 0.38, 0.85, 1.0];
const LAVA_RGBA: [f32; 4] = [0.95, 0.45, 0.10, 1.0];
const CORACAO_RGBA: [f32; 4] = [0.98, 0.45, 0.65, 1.0];
const MURO_RGBA: [f32; 4] = [0.38, 0.40, 0.46, 1.0];
const FOGO_RGBA: [f32; 4] = [1.0, 0.45, 0.15, 1.0];
const GELO_RGBA: [f32; 4] = [0.45, 0.85, 1.0, 1.0];

/// **A receita da Salamandra** — a da cena `=3`, com mais vida e o grito da vitória.
///
/// ⚠️ (W8) **O corpo é CINEMÁTICO, não estático**: parada é a mesma parede para os morcegos (a malha
/// conta o cinemático parado), e o tutorial 03 fá-la perseguir com UM gesto — *Add Component → Nav
/// Agent* — porque a semente do mover nunca rebaixa um `Static` que o artista pôs.
fn receita_da_salamandra(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Salamandra"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [LADO_DA_SALAMANDRA, LADO_DA_SALAMANDRA],
                SALAMANDRA_RGBA,
            ),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO_DA_SALAMANDRA / 2.0,
                    half_y: LADO_DA_SALAMANDRA / 2.0,
                },
                ..Collider::default()
            },
            Health {
                max: VIDA_DA_SALAMANDRA,
                start: VIDA_DA_SALAMANDRA,
                team: MONSTROS.to_owned(),
                on_death: VENCEU.to_owned(),
                numbers: true,
                resistances: vec![
                    Resistance {
                        kind: FOGO.to_owned(),
                        rate: 0.0,
                        absorbs: false,
                    },
                    Resistance {
                        kind: GELO.to_owned(),
                        rate: 2.0,
                        absorbs: false,
                    },
                ],
                ..Health::default()
            },
            HealthBar {
                offset_y: BARRA_Y,
                height: BARRA_H,
                ..HealthBar::default()
            },
        ))
        .id()
}

/// **A receita de um MORCEGO** — um agente de navegação que PERSEGUE o herói pelo nome à volta do
/// muro, com uma vida (um tiro mata-o) e um dano que o gasta ao morder.
///
/// ⚠️ (W8) **Era um projéctil teleguiado, e batia no muro** (o CONTROLO do gate
/// `um_morcego_da_a_volta_ao_muro`). ⚠️ A chegada é a de fábrica (`0,1` m entre CENTROS): os corpos
/// encostam muito antes, logo ele empurra até morder — com a chegada da cena `=1` (encostar com
/// folga) ele parava ao lado do herói e nunca mordia.
fn receita_do_morcego(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Morcego"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [LADO_DO_MORCEGO, LADO_DO_MORCEGO],
                MORCEGO_RGBA,
            ),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball {
                    radius: LADO_DO_MORCEGO / 2.0,
                },
                ..Collider::default()
            },
            TopDownPlayer::from_law(TopDownLaw {
                speed: MORCEGO_RAPIDEZ,
                direction: DirectionMode::Free,
                default_controls: false,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Named(stable_name_id(HEROI)),
                ..NavAgent::default()
            },
            Health {
                max: VIDA_DO_MORCEGO,
                start: VIDA_DO_MORCEGO,
                team: MONSTROS.to_owned(),
                numbers: true,
                ..Health::default()
            },
            Damage {
                amount: MORDIDA,
                team: MONSTROS.to_owned(),
                on_hit: OnHit::Vanish,
                ..Damage::default()
            },
        ))
        .id()
}

/// **A receita do CORAÇÃO** — um sensor que «fere» com o tipo `cura`, que o herói absorve.
fn receita_do_coracao(world: &mut World) -> Entity {
    world
        .spawn((
            Name::new("Coracao"),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [LADO_DO_CORACAO, LADO_DO_CORACAO],
                CORACAO_RGBA,
            ),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO_DO_CORACAO / 2.0,
                    half_y: LADO_DO_CORACAO / 2.0,
                },
                is_sensor: true,
                ..Collider::default()
            },
            Damage {
                amount: CURA,
                team: MONSTROS.to_owned(),
                on_hit: OnHit::Vanish,
                kind: CURA_TIPO.to_owned(),
                ..Damage::default()
            },
        ))
        .id()
}

/// **A receita de uma bala** do tipo `kind` — a da cena `=3` (a de fogo leva a queimadura).
fn receita_da_bala(world: &mut World, kind: &str, rgba: [f32; 4], queima: bool) -> Entity {
    world
        .spawn((
            Name::new(format!("Bala de {kind}")),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.16], rgba),
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
            ph2d_ecs::Lifetime {
                duration_us: 3_000_000,
                ..ph2d_ecs::Lifetime::default()
            },
            Damage {
                amount: DANO,
                team: HEROIS.to_owned(),
                on_hit: OnHit::Vanish,
                kind: kind.to_owned(),
                over_time_per_s: if queima { QUEIMA_POR_S } else { 0.0 },
                over_time_s: QUEIMA_S,
                over_time_every_s: QUEIMA_CADA_S,
                ..Damage::default()
            },
        ))
        .id()
}

/// **Uma fábrica** que nasce `receita` em `xy` quando ouve `sinal`, com um tecto de vivos.
fn fabrica(world: &mut World, nome: &str, xy: [f32; 2], sinal: &str, vivos: u32, receita: Entity) {
    world.spawn((
        Name::new(format!("Fabrica: {nome}")),
        Transform::from_translation(Vec2::new(xy[0], xy[1])),
        Factory {
            master: 0,
            on_signal: sinal.to_owned(),
            burst: 1,
            alive_max: vivos,
            ..Factory::default()
        },
        Pendente(receita),
    ));
}

/// **Um relógio** que publica `sinal` de `cada_us` em `cada_us` (ou uma vez, com `repete = false`).
fn relogio(world: &mut World, nome: &str, sinal: &str, cada_us: u64, repete: bool) {
    world.spawn((
        Name::new(nome.to_owned()),
        Transform::from_translation(Vec2::new(0.0, 5.0)),
        Timers(vec![Timer {
            duration_us: cada_us,
            signal: sinal.to_owned(),
            autostart: true,
            repeat: repete,
            ..Timer::default()
        }]),
    ));
}

/// ⭐⭐ **As duas linhas do herói** — a morte ARRANCA o relógio do recomeço, e o relógio, um segundo
/// e meio depois, RECOMEÇA a corrida.
///
/// ⚠️ **As duas ouvem `From Myself`**: o `heroi-caiu` é do próprio herói, e o relógio que publica o
/// `recomecar` vive nele. ⛔ E o intervalo não é enfeite: um recomeço no mesmo quadro da morte
/// apagava a morte antes de ela se ver.
#[must_use]
pub fn tabela_do_heroi() -> SignalActions {
    let linha = |on: &str, verb, arg: &str| SignalAction {
        on: on.to_owned(),
        target: String::new(),
        verb,
        arg: arg.to_owned(),
        target_by: SignalTarget::Named,
        from: SignalFrom::Myself,
    };
    SignalActions(vec![
        linha(CAIU, SignalVerb::StartTimer, RELOGIO_RECOMECO),
        linha(RECOMECAR, SignalVerb::RestartRun, ""),
    ])
}

/// A cena `=4`. Devolve **quem nasce ESCOLHIDO** — o herói.
pub fn cena_quatro(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**, depois a LAVA: a ordem das raízes é a de CRIAÇÃO, e a lava
    // desenha-se por cima do chão e por baixo de quem a pisa.
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
    world.spawn((
        Name::new("Lava"),
        Sprite::atlas(WHITE_TILE_KEY, LAVA_WH, LAVA_RGBA),
        Transform::from_translation(Vec2::new(LAVA_XY[0], LAVA_XY[1])),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: LAVA_WH[0] / 2.0,
                half_y: LAVA_WH[1] / 2.0,
            },
            is_sensor: true,
            ..Collider::default()
        },
        Damage {
            amount: 0.0,
            per_second: true,
            kind: LAVA_TIPO.to_owned(),
            over_time_per_s: LAVA_POR_S,
            over_time_s: LAVA_DEPOIS_S,
            over_time_every_s: LAVA_CADA_S,
            ..Damage::default()
        },
    ));
    // ⭐ (W8) O muro e a região andável — a banda visível inteira; a malha recua-se pelo raio de
    // cada agente e a lava, que magoa um morcego, é um furo SÓ na malha dele.
    world.spawn((
        Name::new("Muro"),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [MURO_MEIO * 2.0, MURO_CIMA - MURO_BAIXO],
            MURO_RGBA,
        ),
        Transform::from_translation(Vec2::new(MURO_X, (MURO_CIMA + MURO_BAIXO) / 2.0)),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: MURO_MEIO,
                half_y: (MURO_CIMA - MURO_BAIXO) / 2.0,
            },
            ..Collider::default()
        },
    ));
    world.spawn((
        Name::new("Regiao"),
        NavRegion {
            half_extents: [
                (BORDA_DIREITA - BORDA_ESQUERDA) / 2.0,
                (TOPO - REGIAO_FUNDO) / 2.0,
            ],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(
            (BORDA_DIREITA + BORDA_ESQUERDA) / 2.0,
            (TOPO + REGIAO_FUNDO) / 2.0,
        )),
    ));
    relogio(world, "Arranque", COMECAR, 250_000, false);
    relogio(
        world,
        "Relogio do ninho",
        SINAL_MORCEGO,
        MORCEGO_CADA_US,
        true,
    );
    relogio(
        world,
        "Relogio do coracao",
        SINAL_CORACAO,
        CORACAO_CADA_US,
        true,
    );
    let salamandra = receita_da_salamandra(world);
    world.spawn((
        Name::new("Fabrica: Salamandra"),
        Transform::from_translation(Vec2::new(SALAMANDRA_XY[0], SALAMANDRA_XY[1])),
        Factory {
            master: 0,
            on_signal: COMECAR.to_owned(),
            burst: 1,
            total_max: 1,
            ..Factory::default()
        },
        Pendente(salamandra),
    ));
    let morcego = receita_do_morcego(world);
    fabrica(
        world,
        "Ninho",
        NINHO_XY,
        SINAL_MORCEGO,
        MORCEGOS_MAX,
        morcego,
    );
    let coracao = receita_do_coracao(world);
    fabrica(world, "Coracao", CORACAO_XY, SINAL_CORACAO, 1, coracao);
    world.spawn((
        Name::new(PLACAR),
        Transform::from_translation(Vec2::new(PLACAR_XY[0], PLACAR_XY[1])),
        HealthBar {
            target: HEROI.to_owned(),
            width: PLACAR_WH[0],
            height: PLACAR_WH[1],
            offset_y: 0.0,
            ..HealthBar::default()
        },
    ));
    let bala_fogo = receita_da_bala(world, FOGO, FOGO_RGBA, true);
    let bala_gelo = receita_da_bala(world, GELO, GELO_RGBA, false);
    let heroi = world
        .spawn((
            Name::new(HEROI),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.4], HEROI_RGBA),
            Transform::from_translation(Vec2::new(HEROI_XY[0], HEROI_XY[1])),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: RAPIDEZ_DO_HEROI,
                direction: DirectionMode::Free,
                rotation: ph2d_topdown::rotation::RotationMode::ToMovement,
                ..TopDownLaw::default()
            }),
            SignalOnAction(vec![
                ActionTriggerRow {
                    action: ACCAO_FOGO.to_owned(),
                    edge: ActionEdge::Press,
                    signal: SINAL_FOGO.to_owned(),
                },
                ActionTriggerRow {
                    action: ACCAO_GELO.to_owned(),
                    edge: ActionEdge::Press,
                    signal: SINAL_GELO.to_owned(),
                },
            ]),
            Health {
                max: VIDA_DO_HEROI,
                start: VIDA_DO_HEROI,
                team: HEROIS.to_owned(),
                on_death: CAIU.to_owned(),
                invincible_s: INVENCIVEL,
                blink_s: 0.08,
                numbers: true,
                resistances: vec![Resistance {
                    kind: CURA_TIPO.to_owned(),
                    rate: 1.0,
                    absorbs: true,
                }],
                ..Health::default()
            },
            tabela_do_heroi(),
            Timers(vec![Timer {
                name: RELOGIO_RECOMECO.to_owned(),
                duration_us: RECOMECO_US,
                signal: RECOMECAR.to_owned(),
                autostart: false,
                repeat: false,
            }]),
        ))
        .id();
    for (nome, sinal, bala) in [
        ("Arma de fogo", SINAL_FOGO, bala_fogo),
        ("Arma de gelo", SINAL_GELO, bala_gelo),
    ] {
        world.spawn((
            Name::new(nome.to_owned()),
            Transform::from_translation(Vec2::ZERO),
            ChildOf(heroi),
            Factory {
                master: 0,
                on_signal: sinal.to_owned(),
                burst: 1,
                aim_from_spawner: true,
                ..Factory::default()
            },
            Pendente(bala),
        ));
    }
    heroi
}

/// O roteiro da cena `=4` — impresso no terminal.
pub fn roteiro() {
    println!(
        "[vida-smoke] cena=4  a ARENA  fogo={TECLA_FOGO_NOME}  gelo={TECLA_GELO_NOME}  vida do \
         heroi={VIDA_DO_HEROI}\n\
         (0) os quadrados de contorno verde no MEIO do ecra' sao os MOLDES (Salamandra, morcego, \
         coracao e as duas balas): e' deles que tudo nasce. Nao levam tiros. Os avisos `Signal: \
         ...` no TOPO ao centro sao os sinais da cena\n\
         (1) o heroi azul esta' a' DIREITA, virado para a SALAMANDRA (quadrado VERMELHO com barra). \
         A barra larga no canto de cima a' ESQUERDA e' o PLACAR: a vida do HEROI\n\
         (2) carregue no {TECLA_FOGO_NOME} (bala LARANJA, fogo): nada acontece a' Salamandra — e' \
         imune ao fogo. Carregue no {TECLA_GELO_NOME} (bala AZUL-CLARA, gelo): sobe o numero 20 e a \
         barra dela desce. Mais duas balas de gelo e ela morre: aparece `{VENCEU}`\n\
         (3) de 3 em 3 segundos nasce um MORCEGO roxo no canto de cima a' esquerda e vem atras do \
         heroi (no maximo tres): o MURO cinzento so' se passa por BAIXO, e ele da' a volta. Se \
         tocar no heroi, some, tira 15 (o numero sobe por cima do heroi), o PLACAR desce e o heroi \
         PISCA. Mate-o com um tiro de qualquer arma (vire-se para ele com as setas). Ele e' mais \
         lento que o heroi: fugir funciona. A tecla B mostra o caminho que cada um planeou\n\
         (4) leve o heroi para a LAVA (o retangulo LARANJA em baixo a' esquerda): enquanto a pisa, \
         a cada meio segundo sobe um 3 e o placar desce. Saia dela: ainda queima mais dois \
         segundos. Enquanto estiver la' dentro os morcegos ESPERAM na borda: a lava queima-os, e \
         eles evitam-na sozinhos\n\
         (5) de 6 em 6 segundos nasce um CORACAO rosa a' esquerda (so' um de cada vez). Passe por \
         cima dele com o heroi ferido: ele some e o PLACAR sobe 25. Com a vida cheia ele some na \
         mesma (quem apanha, gasta)\n\
         (6) deixe o heroi morrer (fique na lava ou deixe os morcegos morder): o placar esvazia, \
         aparece `{CAIU}`, e um segundo e meio depois TUDO recomeca sozinho — o heroi com a vida \
         cheia, a Salamandra de volta, os morcegos e o coracao do principio\n\
         (7) clique no HEROI: no painel da direita (Inspector) a seccao `Health` tem `Now`, a lista \
         `Resistances` com a linha `cura` e a seccao das acoes (`Signal Actions`) com as duas \
         linhas do recomeco. Clique na LAVA: a seccao `Damage` mostra o dano a zero e a \
         queimadura por segundo\n\
         (8) deu errado se: o fogo fere a Salamandra · o gelo nao a mata ao 3.o tiro · os \
         morcegos nao seguem o heroi, ficam presos no muro ou entram na lava · um morcego que \
         morde nao some · a lava nao queima ou nao \
         continua depois de sair · o coracao nao cura · o heroi morre e nada recomeca · ou, depois \
         do recomeco, o heroi nasce ferido ou a queimar"
    );
}

#[cfg(test)]
#[path = "vida_arena_smoke_tests.rs"]
mod tests;
