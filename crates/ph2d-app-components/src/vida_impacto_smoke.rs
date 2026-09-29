//! ⭐⭐⭐ **Smoke do IMPACTO — o golpe que PESA** (plano 28, W5). `PH2D_VIDA_SMOKE=2`.
//!
//! # A cena: **três inimigos numa FILA, o herói que atira para CIMA, e um espinho**
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
//! ⛔⛔ **A galeria mora TODA à esquerda da COLUNA DOS AVISOS, e o tiro é VERTICAL** (smoke do dono,
//! 25/09: *«não vejo a bala»*). Cada sinal vira um aviso no topo e ao centro do ecrã
//! (`fase_signal_outbox`), e a coluna CRESCE para baixo a cada tiro (`tiro-vida` · `ai` · `morreu`).
//! A 1.ª redacção atirava na horizontal, do herói à esquerda para a coluna à direita — a linha de tiro
//! atravessava o ecrã **por baixo da pilha**, e a bala voava escondida. ⚠️ **A simulação estava
//! certa** (medido: a bala nasce, voa e o extract diz que desenha) e a FOTO com tiro automático
//! mostrou-a debaixo dos avisos. É a mesma lei que a cena do placar já pagou (*«os avisos de sinal
//! empilham-se no topo e tapavam o placar»*). ⇒ a fila, o herói e o espinho cabem entre a
//! [`BORDA_ESQUERDA`] e a [`COLUNA_DOS_AVISOS`], com ERRO DE COMPILAÇÃO a guardá-lo.
//!
//! ⚠️ Se a linha `[vida-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Factory, MasterRoot, Name, Timer, Timers, Transform, Visibility, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, DampMode, DampingOverride, GravityScale, Health,
    HealthBar, ProjectileMotion, RigidBody,
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
    /// Onde a fábrica dele mora, em `x` (a fila está toda em [`Y_INIMIGOS`]).
    pub x: f32,
}

/// ⭐⭐ **Os três inimigos, da esquerda para a direita.**
///
/// ⚠️ **Os `x` cabem entre a [`BORDA_ESQUERDA`] e a [`COLUNA_DOS_AVISOS`]** — ver os `const _`
/// abaixo.
pub const INIMIGOS: [Inimigo; 3] = [
    Inimigo {
        nome: "Leve",
        vida: 20.0,
        aceita: 1.0,
        numeros: true,
        piscar: PISCAR,
        pausa_da_morte: 0.0,
        cor: [0.88, 0.30, 0.28, 1.0],
        x: -4.9,
    },
    Inimigo {
        nome: "Pesado",
        vida: 30.0,
        aceita: 0.25,
        numeros: true,
        piscar: PISCAR,
        pausa_da_morte: PAUSA_DA_MORTE,
        cor: [0.62, 0.38, 0.85, 1.0],
        x: -3.65,
    },
    Inimigo {
        nome: "Controlo (sem impacto)",
        vida: 20.0,
        aceita: 0.0,
        numeros: false,
        piscar: 0.0,
        pausa_da_morte: 0.0,
        cor: [0.55, 0.57, 0.60, 1.0],
        x: -2.4,
    },
];
/// O `y` da fila de inimigos — o LEVE desliza `1,69` m depois de um golpe (medido pela porta do
/// produto no `o_leve_voa_o_pesado_mal_se_mexe_e_o_controlo_fica`), e com a barra por cima ele
/// continua dentro da banda (`1,6 + 1,69 + 0,6 < 4,09`).
pub const Y_INIMIGOS: f32 = 1.6;
/// O lado de um inimigo.
pub const LADO: f32 = 0.9;
/// Meio lado do ESPINHO.
pub const MEIO_ESPINHO: f32 = 0.3;
/// Onde o ESPINHO mora — à ESQUERDA do herói, à mesma altura: o passo (5) é andar para a esquerda.
pub const ESPINHO_XY: [f32; 2] = [-5.75, -0.6];
/// ⚠️ **A borda ESQUERDA da banda visível**, MEDIDA na foto desta cena (`1930×1040`, régua `−600`
/// no px `358` e `−500` no `459`, o canvas a começar no `332`) — a 1.ª redacção punha o herói e o
/// espinho em `x = −6` e a foto mostrou-os CORTADOS pela borda, com os gates todos verdes (eles só
/// mediam o `y`).
pub const BORDA_ESQUERDA: f32 = -6.26;
/// ⛔⛔ **A borda ESQUERDA da coluna dos avisos de sinal** — a medição e o porquê vivem ao lado das
/// duas bordas, em [`crate::vida_smoke::COLUNA_DOS_AVISOS_X`] (uma medida, dois leitores).
pub const COLUNA_DOS_AVISOS: f32 = crate::vida_smoke::COLUNA_DOS_AVISOS_X[0];
/// O empurrão do espinho — mais forte que o da bala, para o herói (que pára na hora) voar à vista.
pub const EMPURRAO_DO_ESPINHO: f32 = 8.0;
/// Onde o herói nasce — por BAIXO do LEVE e virado para CIMA ([`HEROI_RUMO`]): o 1.º tiro não pede
/// pontaria.
pub const HEROI_XY: [f32; 2] = [-4.9, -0.6];
/// O rumo com que o herói nasce — para CIMA, onde a fila está. ⚠️ A fábrica dele aponta a cópia
/// para onde ele aponta (`aim_from_spawner`), logo é este número que decide para onde vai o 1.º tiro.
pub const HEROI_RUMO: f32 = std::f32::consts::FRAC_PI_2;
/// ⭐ **A rapidez da bala desta cena** — mais lenta que a da `=1` (`9 m/s`) porque o tiro vertical é
/// curto (do herói à fila): a `9 m/s` ela vivia `11` tiques à vista antes do golpe, e a `5` vive `20`
/// (medido no `a_bala_sobe_ate_ao_leve_fora_da_coluna_dos_avisos`).
pub const RAPIDEZ_DA_BALA: f32 = 5.0;

// ⚠️ Tudo cabe entre a borda esquerda e a coluna dos avisos, com folga — ERRO DE COMPILAÇÃO e não um
// teste: duas constantes comparadas o compilador dobra-as. (O herói virado para cima tem meia largura
// `0,2` e meia altura `0,45`.)
const _: () = assert!(HEROI_XY[0] - 0.45 >= BORDA_ESQUERDA + 0.2);
const _: () = assert!(ESPINHO_XY[0] - MEIO_ESPINHO >= BORDA_ESQUERDA + 0.2);
const _: () = assert!(INIMIGOS[0].x - LADO / 2.0 >= BORDA_ESQUERDA + 0.2);
const _: () = assert!(INIMIGOS[2].x + LADO / 2.0 <= COLUNA_DOS_AVISOS - 0.1);
const _: () = assert!(INIMIGOS[0].x + LADO / 2.0 < INIMIGOS[1].x - LADO / 2.0);
const _: () = assert!(INIMIGOS[1].x + LADO / 2.0 < INIMIGOS[2].x - LADO / 2.0);
// O herói virado para cima e o espinho cabem por BAIXO da banda visível (`−1,19` m).
const _: () = assert!(HEROI_XY[1] - 0.45 >= -1.19);
const _: () = assert!(ESPINHO_XY[1] - MEIO_ESPINHO >= -1.19);
// O espinho não toca o herói ao nascer, e o herói nasce debaixo do LEVE.
const _: () = assert!(ESPINHO_XY[0] + MEIO_ESPINHO < HEROI_XY[0] - 0.2);
const _: () = assert!(HEROI_XY[0] == INIMIGOS[0].x);

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
            Transform::from_translation(Vec2::new(inimigo.x, Y_INIMIGOS)),
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
        Sprite::atlas(
            WHITE_TILE_KEY,
            [2.0 * MEIO_ESPINHO, 2.0 * MEIO_ESPINHO],
            ESPINHO_RGBA,
        ),
        Transform::from_translation(Vec2::new(ESPINHO_XY[0], ESPINHO_XY[1])),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: MEIO_ESPINHO,
                half_y: MEIO_ESPINHO,
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
    if let Some(mut m) = world.get_mut::<ProjectileMotion>(bala) {
        m.initial_speed = RAPIDEZ_DA_BALA;
    }
    // ⭐ O HERÓI da cena `=1`, com uma VIDA que pisca e mostra os números.
    let heroi = crate::vida_smoke::heroi(world, HEROI_XY, HEROI_RGBA, bala);
    if let Some(mut t) = world.get_mut::<Transform>(heroi) {
        t.rotation = HEROI_RUMO;
    }
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
         inimigo e cada bala nascem. Nao levam tiros e nao se mexem. E os avisos `Signal: ...` \
         que vao aparecendo no TOPO, ao centro, sao os sinais da cena (um por tiro e um por golpe) \
         — por isso tudo o que ha' para ver fica a' ESQUERDA deles\n\
         (1) espere um instante: nascem TRES quadrados numa FILA em cima, a' esquerda — vermelho \
         (LEVE), roxo (PESADO) e, mais a' direita, cinzento (o CONTROLO) — cada um com a barra \
         verde por cima\n\
         (2) carregue no {tecla}: o heroi azul ja' nasce POR BAIXO do VERMELHO, virado para CIMA. \
         A bala AMARELA sobe; no instante em que acerta o jogo PARA um nada (a pausa do golpe), \
         o vermelho VOA para cima e desliza ate' parar, PISCA um bocadinho, e um NUMERO amarelo \
         `10` sobe por cima dele e desaparece\n\
         (3) ande com a seta para a DIREITA ate' ficar por baixo do ROXO, carregue UM INSTANTE na \
         seta para CIMA (o heroi vira-se para cima) e atire: ele quase NAO se mexe (aceita so' um \
         quarto do empurrao). Ao 3.o tiro ele morre — e o jogo PARA um pouco MAIS do que num \
         tiro normal\n\
         (4) o CONTROLO: faca o mesmo por baixo do CINZENTO — a barra desce, mas ele nao voa, nao \
         pisca e nao aparece numero nenhum\n\
         (5) agora leve o heroi para a ESQUERDA ate' ao quadrado VERMELHO pequeno (o ESPINHO), \
         ao lado de onde ele nasceu: o heroi leva um golpe, e' EMPURRADO para tras mesmo parado, \
         aparece `5` por cima dele, e ele PISCA durante um segundo (nesse segundo o espinho nao o \
         fere outra vez)\n\
         (6) na barra de CIMA carregue em `Reset` e depois em `Play`: tudo volta ao principio\n\
         (7) clique no VERMELHO: no Inspector, na seccao `Health`, estao `Death Pause`, `Blink`, \
         `Push Taken` e `Hit Numbers`. Clique na BALA (o molde amarelo): na seccao `Damage` estao \
         `Hit Pause`, `Push` e `Push Up`\n\
         (8) deu errado se: nao ve a bala amarela a subir · o vermelho nao voa · o cinzento voa \
         ou pisca · nao aparece numero · o heroi atravessa o espinho sem ser empurrado · um \
         objecto fica invisivel depois de piscar · ou o jogo nao para nada no tiro"
    );
}

#[cfg(test)]
#[path = "vida_impacto_smoke_tests.rs"]
mod tests;
