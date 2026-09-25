//! ⭐⭐⭐ **Smoke do IMPACTO — o golpe que PESA** (plano 28, W5). `PH2D_VIDA_SMOKE=2`.
//!
//! # A cena: **três inimigos numa coluna, o herói que atira, e um espinho**
//!
//! A bala tem a mesma vida da cena `=1` e três coisas novas: **empurra** (`Push`), **pausa o jogo**
//! um instante ao entrar (`Hit Pause`) e — nos alvos que o pedem — faz nascer o **número** do dano.
//!
//! | inimigo | o que mostra |
//! |---|---|
//! | vermelho (leve) | voa para trás a cada tiro, **pisca**, e o número sobe |
//! | roxo (pesado) | quase não se mexe (aceita `¼` do empurrão) e a MORTE dele pausa mais tempo |
//! | cinzento (o CONTROLO) | a mesma vida sem nada disto: não voa, não pisca, sem número — só a barra |
//!
//! ⭐ E o **espinho** vermelho à esquerda ensina o impacto do lado de QUEM LEVA: o herói que entra
//! nele é empurrado para trás pelo canal próprio do empurrão (ele pára na hora quando larga a seta,
//! e ainda assim voa) e **pisca** durante a invencibilidade.
//!
//! ⚠️ **Os inimigos são DINÂMICOS sem gravidade e com arrasto** (`DampingOverride`): um corpo
//! estático não sai do sítio com golpe nenhum, e sem arrasto um empurrão levava-o para fora do ecrã.
//!
//! ⚠️ Se a linha `[vida-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Factory, MasterRoot, Name, Timer, Timers, Transform, Visibility, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, DampMode, DampingOverride, GravityScale, Health,
    HealthBar, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::vida_smoke::{AI, HEROIS, MONSTROS, MORREU, Pendente};

/// O sinal que arranca as três fábricas.
const COMECAR: &str = "comecar-impacto";

/// ⭐ **O IMPACTO da bala** — o empurrão, a pausa. A pausa é o número da pesquisa para um golpe
/// comum (`~0,05 s` = 3 tiques do passo fixo, medido no gate da W5).
pub const EMPURRAO: f32 = 5.0;
/// A pausa de um golpe comum, em segundos.
pub const PAUSA: f32 = 0.05;
/// A pausa da MORTE do pesado — o golpe final da pesquisa (`~0,15 s`).
pub const PAUSA_DA_MORTE: f32 = 0.15;
/// Quanto dura cada metade do piscar.
pub const PISCAR: f32 = 0.06;
/// A invencibilidade depois de um golpe — a janela em que o piscar corre.
pub const INVENCIVEL: f32 = 0.4;
/// O arrasto dos inimigos — um empurrão de `5 m/s` desliza `v/k ≈ 1,7 m` e pára.
pub const ARRASTO: f32 = 3.0;

/// Um inimigo da coluna — o que o distingue dos irmãos é o que a cena ensina.
#[derive(Clone, Copy, Debug)]
pub struct Inimigo {
    /// O nome na Hierarquia (e o que os gates procuram).
    pub nome: &'static str,
    /// A vida máxima e inicial.
    pub vida: f32,
    /// Que fracção do empurrão ele aceita (`Push Taken`).
    pub aceita: f32,
    /// Se faz nascer o número do dano.
    pub numeros: bool,
    /// Quanto dura cada metade do piscar (`0` = não pisca).
    pub piscar: f32,
    /// A pausa da morte dele.
    pub pausa_da_morte: f32,
    /// A cor do corpo.
    pub cor: [f32; 4],
    /// Onde a fábrica dele mora, em `y`.
    pub y: f32,
}

/// ⭐⭐ **Os três inimigos, de cima para baixo.**
///
/// ⚠️ **Os `y` cabem na banda que a régua deixa visível** (`+4,09` / `−1,19` m, medida na foto da
/// cena da arma).
pub const INIMIGOS: [Inimigo; 3] = [
    Inimigo {
        nome: "Leve",
        vida: 20.0,
        aceita: 1.0,
        numeros: true,
        piscar: PISCAR,
        pausa_da_morte: 0.0,
        cor: [0.88, 0.30, 0.28, 1.0],
        y: 3.0,
    },
    Inimigo {
        nome: "Pesado",
        vida: 30.0,
        aceita: 0.25,
        numeros: true,
        piscar: PISCAR,
        pausa_da_morte: PAUSA_DA_MORTE,
        cor: [0.62, 0.38, 0.85, 1.0],
        y: 1.5,
    },
    Inimigo {
        nome: "Controlo (sem impacto)",
        vida: 20.0,
        aceita: 0.0,
        numeros: false,
        piscar: 0.0,
        pausa_da_morte: 0.0,
        cor: [0.55, 0.57, 0.60, 1.0],
        y: 0.0,
    },
];
/// O `x` da coluna de inimigos.
pub const X_INIMIGOS: f32 = 3.0;
/// O lado de um inimigo.
pub const LADO: f32 = 0.9;
/// Onde o ESPINHO mora — abaixo do herói, à esquerda, longe da linha de tiro.
pub const ESPINHO_XY: [f32; 2] = [-4.5, 0.0];
/// ⚠️ **A borda ESQUERDA da banda visível**, MEDIDA na foto desta cena (`1930×1040`, régua `−600`
/// no px `358` e `−500` no `459`, o canvas a começar no `332`) — a 1.ª redacção punha o herói e o
/// espinho em `x = −6` e a foto mostrou-os CORTADOS pela borda, com os gates todos verdes (eles só
/// mediam o `y`).
pub const BORDA_ESQUERDA: f32 = -6.26;
/// O empurrão do espinho — mais forte que o da bala, para o herói (que pára na hora) voar à vista.
pub const EMPURRAO_DO_ESPINHO: f32 = 8.0;
/// Onde o herói nasce — à altura do LEVE, o 1.º tiro não pede pontaria.
pub const HEROI_XY: [f32; 2] = [-4.5, 3.0];

// ⚠️ O herói (meia largura `0,45`) e o espinho (`0,4`) cabem dentro da borda esquerda MEDIDA, com
// folga — ERRO DE COMPILAÇÃO e não um teste: duas constantes comparadas o compilador dobra-as.
const _: () = assert!(HEROI_XY[0] - 0.45 >= BORDA_ESQUERDA + 0.2);
const _: () = assert!(ESPINHO_XY[0] - 0.4 >= BORDA_ESQUERDA + 0.2);

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const ESPINHO_RGBA: [f32; 4] = [0.85, 0.2, 0.2, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];

/// A receita de um inimigo — dinâmico, sem gravidade, com arrasto e a VIDA do impacto.
fn receita_do_inimigo(world: &mut World, i: Inimigo) -> Entity {
    world
        .spawn((
            Name::new(i.nome),
            MasterRoot,
            Transform::from_translation(Vec2::ZERO),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], i.cor),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: LADO / 2.0,
                    half_y: LADO / 2.0,
                },
                ..Collider::default()
            },
            GravityScale(0.0),
            DampingOverride {
                linear: ARRASTO,
                angular: ARRASTO,
                mode: DampMode::Replace,
            },
            Health {
                max: i.vida,
                start: i.vida,
                team: MONSTROS.to_owned(),
                on_damage: AI.to_owned(),
                on_death: MORREU.to_owned(),
                invincible_s: if i.piscar > 0.0 { INVENCIVEL } else { 0.0 },
                blink_s: i.piscar,
                knockback_taken: i.aceita,
                numbers: i.numeros,
                death_hitstop_s: i.pausa_da_morte,
                ..Health::default()
            },
            HealthBar {
                offset_y: 0.6,
                height: 0.1,
                ..HealthBar::default()
            },
        ))
        .id()
}

/// A cena `=2`. Devolve **quem nasce ESCOLHIDO** — o herói.
pub(crate) fn cena_dois(world: &mut World) -> Entity {
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
    for inimigo in INIMIGOS {
        let receita = receita_do_inimigo(world, inimigo);
        world.spawn((
            Name::new(format!("Fabrica: {}", inimigo.nome)),
            Transform::from_translation(Vec2::new(X_INIMIGOS, inimigo.y)),
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
    // ⭐ O ESPINHO — um sensor estático que fere e EMPURRA quem lhe toca.
    world.spawn((
        Name::new("Espinho"),
        Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.8], ESPINHO_RGBA),
        Transform::from_translation(Vec2::new(ESPINHO_XY[0], ESPINHO_XY[1])),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.4,
                half_y: 0.4,
            },
            is_sensor: true,
            ..Collider::default()
        },
        Damage {
            amount: 5.0,
            team: MONSTROS.to_owned(),
            knockback: EMPURRAO_DO_ESPINHO,
            hitstop_s: PAUSA,
            ..Damage::default()
        },
    ));
    // A BALA: a da cena `=1`, com o impacto.
    let bala = crate::vida_smoke::receita_da_bala(world);
    if let Some(mut d) = world.get_mut::<Damage>(bala) {
        d.knockback = EMPURRAO;
        d.hitstop_s = PAUSA;
    }
    // ⭐ O HERÓI da cena `=1`, com uma VIDA que pisca e mostra os números.
    let heroi = crate::vida_smoke::heroi(world, HEROI_XY, HEROI_RGBA, bala);
    world.entity_mut(heroi).insert(Health {
        max: 100.0,
        start: 100.0,
        team: HEROIS.to_owned(),
        on_damage: AI.to_owned(),
        invincible_s: 1.0,
        blink_s: 0.08,
        numbers: true,
        ..Health::default()
    });
    heroi
}

/// O roteiro da cena `=2`.
pub(crate) fn roteiro() {
    let tecla = crate::vida_smoke::TECLA_NOME;
    println!(
        "[vida-smoke] cena=2  o golpe que PESA  tecla={tecla}\n\
         (0) o quadrado de contorno verde no MEIO do ecra' sao os MOLDES: e' deles que cada \
         inimigo e cada bala nascem. Nao levam tiros e nao se mexem\n\
         (1) espere um instante: nascem TRES quadrados numa coluna a' direita — vermelho (LEVE), \
         roxo (PESADO) e, em baixo, cinzento (o CONTROLO) — cada um com a barra verde por cima\n\
         (2) carregue no {tecla}: o heroi azul ja' nasce a' altura do VERMELHO. No instante do \
         tiro o jogo PARA um nada (a pausa do golpe), o vermelho VOA para tras e desliza ate' \
         parar, PISCA um bocadinho, e um NUMERO amarelo `10` sobe por cima dele e desaparece\n\
         (3) desca ate' ao ROXO e atire: ele quase NAO se mexe (aceita so' um quarto do empurrao). \
         Ao 3.o tiro ele morre — e o jogo PARA um pouco MAIS do que num tiro normal\n\
         (4) o CONTROLO: atire no CINZENTO — a barra desce, mas ele nao voa, nao pisca e nao \
         aparece numero nenhum\n\
         (5) agora leve o heroi para BAIXO ate' ao quadrado VERMELHO pequeno (o ESPINHO) a' \
         esquerda: o heroi leva um golpe, e' EMPURRADO para tras mesmo parado, aparece `5` por \
         cima dele, e ele PISCA durante um segundo (nesse segundo o espinho nao o fere outra vez)\n\
         (6) na barra de CIMA carregue em `Reset` e depois em `Play`: tudo volta ao principio\n\
         (7) clique no VERMELHO: no Inspector, na seccao `Health`, estao `Death Pause`, `Blink`, \
         `Push Taken` e `Hit Numbers`. Clique na BALA (o molde amarelo): na seccao `Damage` estao \
         `Hit Pause`, `Push` e `Push Up`\n\
         (8) deu errado se: o vermelho nao voa · o cinzento voa ou pisca · nao aparece numero · o \
         heroi atravessa o espinho sem ser empurrado · um objecto fica invisivel depois de piscar · \
         ou o jogo nao para nada no tiro"
    );
}

#[cfg(test)]
#[path = "vida_impacto_smoke_tests.rs"]
mod tests;
