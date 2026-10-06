//! ⭐⭐⭐ **Smoke dos TIPOS de dano e do dano que DURA** (plano 28, W6). `PH2D_VIDA_SMOKE=3`.
//!
//! # A cena: **três alvos numa coluna, e um herói com DUAS armas**
//!
//! O herói azul atira **FOGO** com o `Q` e **GELO** com o `J`. As duas balas tiram `10`; a de fogo
//! deixa ainda uma **QUEIMADURA** — `3` pontos por segundo durante `3` segundos, um pulso por
//! segundo — que continua a morder depois de a bala sumir.
//!
//! | alvo | resistências | fogo (`Q`) | gelo (`J`) |
//! |---|---|---|---|
//! | **Salamandra** (vermelha) | fogo `0` · gelo `2` | **nada** — nem o golpe, nem a queimadura | **`20`** — o dobro |
//! | **Controlo** (cinzento) | nenhuma | `10`, e depois `3` · `3` · `3` | `10` |
//! | **Elemental de fogo** (amarelo) | fogo **absorve** | a vida **SOBE** `10`, e a queimadura cura `3` · `3` · `3` | `10` |
//!
//! ⭐ **É o CONTROLO ao lado que torna a wave legível**: sem ele, «a salamandra não sofre» leria-se
//! como uma bala que não acerta. O elemental nasce com METADE da vida, senão a cura não teria para
//! onde subir (a porta da cura não passa do máximo).
//!
//! ⚠️ **As duas armas são DUAS fábricas FILHAS do herói**: um objecto tem UMA `Factory`, e uma
//! fábrica filha aponta para onde o PAI aponta (a mira sai do `world_transform`). ⛔ A alternativa —
//! uma fábrica que troca de molde — seria uma segunda porta para o que a tabela já exprime.
//!
//! ⛔ **A geometria é a da cena `=1`** (a coluna à direita da COLUNA DOS AVISOS, o herói à altura do
//! alvo de cima, o tiro horizontal): foi ela que o dono aprovou depois do *«não vejo a bala»*.
//!
//! ⚠️ Se a linha `[vida-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ActionEdge, ActionTriggerRow, ChildOf, Entity, Factory, Lifetime, MasterRoot, Name,
    SignalOnAction, Timer, Timers, Transform, Visibility, World,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, HealthBar, OnHit, ProjectileMotion,
    Resistance, RigidBody,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

use crate::vida_smoke::{
    AI, BARRA_H, BARRA_Y, CUROU, HEROI_X, HEROIS, LADO, MONSTROS, MORREU, Pendente, X_ALVOS,
};

/// A acção do FOGO — no `Q`, a tecla da arma de todas as cenas desta família (medida no gatilho).
pub const ACCAO_FOGO: &str = "fogo";
/// A acção do GELO — no `J`, a outra letra livre do teclado do editor (a do veneno da `=1`).
pub const ACCAO_GELO: &str = "gelo";
/// ⭐ **As acções que o prólogo cria nesta cena**, com a tecla de cada uma — a MESMA forma da lista
/// da `=1` ([`crate::vida_smoke::ACCOES`]), lida pela porta [`crate::vida_smoke::accoes`].
pub const ACCOES: [(&str, u32); 2] = [
    (ACCAO_FOGO, crate::trigger_smoke::TECLA),
    (ACCAO_GELO, crate::vida_smoke::TECLA_VENENO),
];
/// Os nomes das teclas, para o roteiro.
pub const TECLA_FOGO_NOME: &str = crate::dano_smoke::TECLA_NOME;
/// Idem, o do gelo.
pub const TECLA_GELO_NOME: &str = crate::vida_smoke::TECLA_VENENO_NOME;
/// Os sinais das duas armas.
pub(crate) const SINAL_FOGO: &str = "tiro-fogo";
pub(crate) const SINAL_GELO: &str = "tiro-gelo";
/// O sinal que arranca as três fábricas.
pub(crate) const COMECAR: &str = "comecar-tipos";

/// Os dois TIPOS — o nome que as balas levam e que as resistências procuram.
pub const FOGO: &str = "fogo";
/// Idem, o gelo.
pub const GELO: &str = "gelo";
/// O golpe de uma bala.
pub const DANO: f32 = 10.0;
/// ⭐ **A QUEIMADURA** — pontos por segundo, duração e intervalo. `3 × 3 = 9` no total, em três
/// pulsos que se CONTAM a olho (um por segundo).
pub const QUEIMA_POR_S: f32 = 3.0;
/// A duração da queimadura, em segundos.
pub const QUEIMA_S: f32 = 3.0;
/// Cada quanto tempo ela pulsa.
pub const QUEIMA_CADA_S: f32 = 1.0;
/// A vida dos três alvos.
pub const VIDA: f32 = 40.0;

/// Um alvo da coluna.
#[derive(Clone, Copy, Debug)]
pub struct Alvo {
    /// O nome na Hierarquia (e o que os gates procuram).
    pub nome: &'static str,
    /// A vida com que nasce (o máximo é sempre [`VIDA`]).
    pub comeca: f32,
    /// `(tipo, taxa, absorve)` — as resistências dele.
    pub resistencias: &'static [(&'static str, f32, bool)],
    /// A cor do corpo.
    pub cor: [f32; 4],
    /// O `y` da fábrica dele.
    pub y: f32,
}

/// ⭐⭐ **Os três alvos, de cima para baixo** — os `y` da cena `=1` (cabem na banda da régua).
pub const ALVOS: [Alvo; 3] = [
    Alvo {
        nome: "Salamandra",
        comeca: VIDA,
        resistencias: &[(FOGO, 0.0, false), (GELO, 2.0, false)],
        cor: [0.88, 0.30, 0.22, 1.0],
        y: 3.4,
    },
    Alvo {
        nome: "Controlo (sem resistencias)",
        comeca: VIDA,
        resistencias: &[],
        cor: [0.55, 0.57, 0.60, 1.0],
        y: 2.1,
    },
    Alvo {
        nome: "Elemental de fogo",
        comeca: VIDA / 2.0,
        resistencias: &[(FOGO, 1.0, true)],
        cor: [0.97, 0.80, 0.25, 1.0],
        y: 0.8,
    },
];
// A coluna separa-se (`1,3` m entre filas) e o herói nasce à altura da de cima.
const _: () = assert!(ALVOS[0].y - ALVOS[1].y > LADO);
const _: () = assert!(ALVOS[1].y - ALVOS[2].y > LADO);
// ⚠️ **Fora da coluna dos avisos** (a lição do *«não vejo a bala»*, §13 do plano 28): o herói e a
// coluna de alvos herdam o `x` da `=1`, que já o prova em `crate::vida_smoke`; aqui fica só o que
// esta cena acrescenta — a barra do alvo de CIMA não sai da banda (`+4,09`, medida na foto).
// ⭐ Erro de COMPILAÇÃO e não um gate: um `assert!` sobre constantes num teste é dobrado pelo
// compilador (o clippy di-lo), e um `const { … }` dentro de uma função o `cargo check` não avalia.
const _: () = assert!(ALVOS[0].y + BARRA_Y + BARRA_H / 2.0 <= 4.09);

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];
const FOGO_RGBA: [f32; 4] = [1.0, 0.45, 0.15, 1.0];
const GELO_RGBA: [f32; 4] = [0.45, 0.85, 1.0, 1.0];

/// **A RECEITA de um alvo** — um corpo SÓLIDO (a bala é um mover e só vê formas sólidas — a lição
/// da `=1`), com a vida, as resistências, a barra e os NÚMEROS de dano ligados: é o número que
/// mostra o `20` do gelo na salamandra, e a AUSÊNCIA dele que mostra o fogo a não morder.
fn receita_do_alvo(world: &mut World, a: Alvo) -> Entity {
    world
        .spawn((
            Name::new(a.nome),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], a.cor),
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
                max: VIDA,
                start: a.comeca,
                team: MONSTROS.to_owned(),
                on_damage: AI.to_owned(),
                on_death: MORREU.to_owned(),
                on_heal: CUROU.to_owned(),
                numbers: true,
                resistances: a
                    .resistencias
                    .iter()
                    .map(|&(kind, rate, absorbs)| Resistance {
                        kind: kind.to_owned(),
                        rate,
                        absorbs,
                    })
                    .collect(),
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

/// **A RECEITA de uma bala** do tipo `kind` — o projéctil da `=1` com o TIPO; a de fogo leva a
/// queimadura.
fn receita_da_bala(world: &mut World, kind: &str, rgba: [f32; 4], queima: bool) -> Entity {
    world
        .spawn((
            Name::new(format!("Bala de {kind}")),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            crate::smoke_desenho::disco(0.1, rgba),
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

/// **Uma arma** — uma fábrica FILHA do herói que ouve `sinal` e atira `bala` para onde ele aponta.
fn arma(world: &mut World, heroi: Entity, nome: &str, sinal: &str, bala: Entity) {
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

/// A cena `=3`. Devolve **quem nasce ESCOLHIDO** — o herói.
pub fn cena_tres(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**: a ordem das raízes é a de CRIAÇÃO.
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
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
    for a in ALVOS {
        let receita = receita_do_alvo(world, a);
        world.spawn((
            Name::new(format!("Fabrica: {}", a.nome)),
            Transform::from_translation(Vec2::new(X_ALVOS, a.y)),
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
    let bala_fogo = receita_da_bala(world, FOGO, FOGO_RGBA, true);
    let bala_gelo = receita_da_bala(world, GELO, GELO_RGBA, false);
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
            crate::smoke_desenho::disco(0.3, HEROI_RGBA),
            Transform::from_translation(Vec2::new(HEROI_X, ALVOS[0].y)),
            ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
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
        ))
        .id();
    crate::smoke_desenho::rumo(world, heroi, 0.3, HEROI_RGBA);
    arma(world, heroi, "Arma de fogo", SINAL_FOGO, bala_fogo);
    arma(world, heroi, "Arma de gelo", SINAL_GELO, bala_gelo);
    heroi
}

/// O roteiro da cena `=3` — impresso no terminal.
pub fn roteiro() {
    println!(
        "[vida-smoke] cena=3  fogo={TECLA_FOGO_NOME}  gelo={TECLA_GELO_NOME}  golpe={DANO}  \
         queimadura={QUEIMA_POR_S}/s por {QUEIMA_S} s\n\
         (0) os quadrados de contorno verde no MEIO do ecra' sao os MOLDES (os dos alvos e os das \
         duas balas): e' deles que tudo nasce. Nao levam tiros\n\
         (1) espere um instante: nascem TRES quadrados numa coluna a' direita — VERMELHO (a \
         Salamandra), CINZENTO (o Controlo) e AMARELO (o Elemental de fogo), cada um com uma \
         barra verde. A do amarelo nasce a MEIO\n\
         (2) o heroi azul ja' nasce a' altura da SALAMANDRA. Carregue no {TECLA_FOGO_NOME} (bala \
         LARANJA, de fogo): a bala some ao bater e NADA acontece — nem numero, nem barra, nem \
         queimadura. Ela e' imune ao fogo\n\
         (3) carregue no {TECLA_GELO_NOME} (bala AZUL-CLARA, de gelo): sobe o numero 20 e a barra \
         dela desce para metade. Ela e' fraca ao gelo: o dobro dos 10\n\
         (4) o CONTROLO: desca ate' ao CINZENTO (seta para baixo, depois seta para a direita para \
         o heroi se virar) e atire FOGO: sobe 10, e depois, um de cada vez, 3 · 3 · 3 — um por \
         segundo, com a bala ja' longe. E' a QUEIMADURA. O GELO tira-lhe so' 10\n\
         (5) desca ate' ao AMARELO e atire FOGO: a barra dele SOBE (o fogo cura-o) e sobe mais \
         tres vezes com a queimadura, ate' encher. O GELO fere-o normalmente\n\
         (6) clique na SALAMANDRA: no painel da direita, na seccao `Health`, ha' a lista \
         `Resistances` com `fogo  immune` e `gelo  x2`. Clique numa linha e ela abre por baixo \
         (`Type`, `Rate`, `Absorbs`). Mude o `Rate` do fogo para 1 e atire fogo outra vez: \
         agora ela sofre\n\
         (7) na barra de CIMA carregue em `Reset` e depois em `Play`: tudo volta ao inicio\n\
         (8) deu errado se: a salamandra perde vida com o fogo · o gelo tira-lhe so' 10 · o \
         cinzento nao leva os tres 3 depois do fogo · a barra do amarelo desce com o fogo · o \
         {TECLA_FOGO_NOME} ou o {TECLA_GELO_NOME} nao atiram · ou a lista `Resistances` nao aparece"
    );
}

#[cfg(test)]
#[path = "vida_tipos_smoke_tests.rs"]
mod tests;
