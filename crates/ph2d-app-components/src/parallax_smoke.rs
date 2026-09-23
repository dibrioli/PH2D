//! ⭐⭐⭐ **Smoke da PARALAXE** (plano 24, W7). `PH2D_PARALLAX_SMOKE=1|2`.
//!
//! # A cena `=1`: **o mesmo passo, e a única variável é a DISTÂNCIA**
//!
//! Um herói que anda com as setas, a câmera do jogo atrás dele, e **quatro planos** que só diferem
//! num número: o chão anda com ele, as árvores ficam para trás, as colinas quase não se mexem e o
//! céu parece parado. *É isso, e mais nada, que o olho lê como profundidade.*
//!
//! ⚠️⚠️ **Os POSTES do chão não são decoração — eles são a RÉGUA.** A câmera segue o herói, logo
//! ele fica parado no ecrã; sem nada a `k = 1` para passar por ele o dono não tem contra o quê
//! comparar, e as camadas de fundo leem-se como *«o mundo inteiro anda devagar»*. Há gate a
//! contá-los.
//!
//! ⛔ **E a câmera é PRESA em Y** (`CameraLimits` com a região da altura EXACTA da vista): ela
//! segue o herói nos dois eixos por omissão, e com o céu a `+3,4` bastavam dois passos para cima
//! para ele sair do ecrã. *A cerca não é um truque: é o componente que o artista usaria.*
//!
//! # ⭐ As duas leis que a cena CONTRASTA de propósito
//!
//! As árvores e o céu **REPETEM** (`ScrollRepeat`) e nunca acabam; as colinas **TÊM LIMITES**
//! (`ScrollLimits`) e param na borda da serra. Uma cena com as quatro camadas a repetir ensinaria
//! metade do módulo, e o dono não teria como descobrir a outra.
//!
//! # A cena `=2`: **o DOLLY, que é o que nenhum outro motor deste género dá**
//!
//! As MESMAS camadas, ninguém anda, e a única coisa que se mexe é uma pista do painel. Aproximar a
//! câmera **não é o zoom**: o mundo fica do mesmo tamanho e os fundos ENCOLHEM, que é o que a
//! profundidade faz de verdade.
//!
//! ⚠️ Se a linha `[parallax-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    CameraFollow, CameraLimits, ChildOf, Entity, GameCamera, Name, ScrollFactor, ScrollLimits,
    ScrollMotion, ScrollRepeat, Transform, World,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do corpo do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 2;

/// ⭐ **A meia-vista, em metros** — `height_world / 2` de uma [`GameCamera`] de fábrica.
///
/// ⚠️ Ela é lida pela CERCA que prende a câmera em Y e pelos gates do enquadramento; escrita duas
/// vezes, a cerca e a régua divergiriam no dia em que alguém mexesse no `height_world` da cena.
pub const MEIA_VISTA_Y: f32 = 5.0;

/// ⭐⭐⭐ **O `k` de cada plano, e eles são o ASSUNTO da cena.**
///
/// ⚠️ **Os quatro números são MEDIDOS contra o que o olho separa, não escolhidos em progressão:**
/// com `0,8 / 0,6 / 0,4` os três fundos leem-se como um só borrão a arrastar. O vale está entre
/// `0,65` (nitidamente mais lento que o chão) e `0,12` (nitidamente parado), com as colinas no
/// meio — e o gate exige que dois planos vizinhos difiram por pelo menos `0,2`, senão a cena
/// deixa de ensinar o que diz que ensina.
pub const K_ARVORES: f32 = 0.65;
/// Ver [`K_ARVORES`].
pub const K_COLINAS: f32 = 0.35;
/// Ver [`K_ARVORES`].
pub const K_CEU: f32 = 0.12;

/// O ladrilho de cada plano que REPETE, em metros. ⚠️ Ele tem de ser **maior que a peça** e a
/// fileira tem de cobrir `meia-vista + ladrilho/2` de cada lado, senão a costura entra no ecrã.
///
/// ⛔⛔ **A FOTO é que escolheu estes números, com os gates todos verdes:** a `7` m e a `16` m a
/// janela (`~13` m de largura com o Inspector e a Hierarchy abertos) mostrava **UMA árvore e UMA
/// nuvem**, e uma peça sozinha não se lê a andar mais devagar que outra — *a diferença de
/// velocidade vê-se entre VIZINHAS que passam*. Com `3,5` e `8` a janela tem três a quatro de cada.
const TILE_ARVORES: f32 = 3.5;
/// Ver [`TILE_ARVORES`].
const TILE_CEU: f32 = 8.0;

/// ⭐ **Quantas peças por fileira** — ver [`TILE_ARVORES`]. Com `15` a `3,5` m a fileira cobre
/// `±26`, contra os `±13,4` que um ecrã `21:9` mais a janela do embrulho pedem (há gate).
const PECAS_ARVORES: i32 = 15;
/// Ver [`PECAS_ARVORES`]. Com `7` a `8` m ela cobre `±28` contra os `±15,7` precisos.
const PECAS_CEU: i32 = 7;
/// As colinas não repetem: elas ACABAM, e é isso que o `ScrollLimits` ensina.
const PECAS_COLINAS: i32 = 5;
/// O passo da serra, em metros.
const PASSO_COLINAS: f32 = 9.0;

/// ⭐⭐ **A cerca das colinas**, em metros de MUNDO — ver o cabeçalho.
///
/// ⚠️ A lei clampa o centro da vista a `[min + meia, max − meia]`, logo com `±30` e uma meia-vista
/// de `~9,5` a serra **pára quando o dono passa de `~±20`** — que é uma dúzia de passos, e não uma
/// caminhada. *Uma cerca longe de mais não é vista como uma cerca; é vista como uma cena partida.*
const SERRA_MIN_X: f32 = -30.0;
/// Ver [`SERRA_MIN_X`].
const SERRA_MAX_X: f32 = 30.0;

/// ⭐ **A deriva do céu**, em metros por SEGUNDO — as nuvens andam sozinhas.
///
/// ⚠️ **`0,35` e não `2`:** a deriva tem de ser lenta o bastante para o dono a distinguir do próprio
/// passo dele. A `2` m/s ela é mais rápida que a paralaxe do céu e lê-se como *«o céu está a
/// seguir-me ao contrário»*.
const DERIVA_CEU: f32 = 0.35;

/// ⭐⭐ **A altura de cada plano, em metros — e as quatro somam-se a um ORÇAMENTO VERTICAL.**
///
/// ⛔⛔ **A meia-vista é da JANELA e não da banda do canvas** (`aspect_of(surface.size())` na
/// `fase_game_camera`), logo com a régua do transporte ABERTA o dono vê pouco mais de METADE do
/// que a câmera enquadra — e um céu a `+3,4` com o chão a `−3,5` sai do ecrã pelos dois lados.
/// *É a armadilha que três waves desta linha já pagaram, e a cura é dupla:* esta cena **FECHA** a
/// régua no prólogo (nunca «não a abre»: a arrumação vive fora do repositório e pode trazê-la) e
/// arruma as peças em `−3,4 .. +3,7`, que é `71 %` da vista.
const Y_CEU: f32 = 3.2;
/// Ver [`Y_CEU`].
const Y_COLINAS: f32 = 1.2;
/// Ver [`Y_CEU`].
const Y_ARVORES: f32 = -0.9;
/// Ver [`Y_CEU`].
const Y_CHAO: f32 = -2.8;
/// A espessura da faixa do chão, em metros.
const ALTURA_DO_CHAO: f32 = 1.2;
/// O topo da faixa do chão — é onde o herói pousa, e ele é DERIVADO, nunca escrito outra vez.
const Y_TOPO_DO_CHAO: f32 = Y_CHAO + ALTURA_DO_CHAO / 2.0;

const CEU_RGBA: [f32; 4] = [0.74, 0.80, 0.90, 1.0];
const COLINA_RGBA: [f32; 4] = [0.45, 0.52, 0.58, 1.0];
const ARVORE_RGBA: [f32; 4] = [0.20, 0.30, 0.26, 1.0];
const CHAO_RGBA: [f32; 4] = [0.13, 0.14, 0.17, 1.0];
const POSTE_RGBA: [f32; 4] = [0.42, 0.45, 0.52, 1.0];
const HEROI_RGBA: [f32; 4] = [0.35, 0.62, 0.95, 1.0];

/// O que o prólogo precisa de saber da cena montada.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// Quem nasce ESCOLHIDO — ver [`montar`].
    pub escolhido: u64,
}

/// ⭐ **Uma fileira de peças iguais, penduradas num pai que carrega a lei.**
///
/// ⚠️ **A lei mora no PAI e as peças são filhas**, e não o contrário: a paralaxe escreve UM
/// `Transform` e a propagação da casa leva as `n` peças de graça. Com a lei em cada peça o passe
/// pagaria `n` vezes o mesmo trabalho e uma peça esquecida ficaria para trás em silêncio.
fn fileira(
    world: &mut World,
    nome: &str,
    y: f32,
    peca: [f32; 2],
    rgba: [f32; 4],
    passo: f32,
    n: i32,
) -> Entity {
    let pai = world
        .spawn((
            Name::new(nome.to_owned()),
            Transform::from_translation(Vec2::new(0.0, y)),
        ))
        .id();
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let x = (i - n / 2) as f32 * passo;
        world.spawn((
            Name::new(format!("{nome} {i}")),
            Sprite::atlas(WHITE_TILE_KEY, peca, rgba),
            Transform::from_translation(Vec2::new(x, 0.0)),
            ChildOf(pai),
        ));
    }
    pai
}

/// **Os quatro planos.** ⚠️ **O CÉU PRIMEIRO**, e isto não é estilo: a ordem das raízes é a ordem
/// de CRIAÇÃO, logo quem nasce primeiro desenha por baixo (a lei que o report de 15/09 pagou).
fn cenario(world: &mut World) {
    let ceu = fileira(
        world,
        "Ceu",
        Y_CEU,
        [3.0, 0.8],
        CEU_RGBA,
        TILE_CEU,
        PECAS_CEU,
    );
    world.entity_mut(ceu).insert((
        ScrollFactor { k: [K_CEU, K_CEU] },
        ScrollRepeat {
            tile: [TILE_CEU, 0.0],
        },
        ScrollMotion {
            velocity: [DERIVA_CEU, 0.0],
        },
    ));

    let colinas = fileira(
        world,
        "Colinas",
        Y_COLINAS,
        [7.0, 2.4],
        COLINA_RGBA,
        PASSO_COLINAS,
        PECAS_COLINAS,
    );
    world.entity_mut(colinas).insert((
        ScrollFactor {
            k: [K_COLINAS, K_COLINAS],
        },
        // ⚠️ **SEM `ScrollRepeat`, de propósito** — ver o cabeçalho: é o contraste que ensina.
        ScrollLimits {
            min: [SERRA_MIN_X, -100.0],
            max: [SERRA_MAX_X, 100.0],
        },
    ));

    let arvores = fileira(
        world,
        "Arvores",
        Y_ARVORES,
        [0.5, 2.2],
        ARVORE_RGBA,
        TILE_ARVORES,
        PECAS_ARVORES,
    );
    world.entity_mut(arvores).insert((
        ScrollFactor {
            k: [K_ARVORES, K_ARVORES],
        },
        ScrollRepeat {
            tile: [TILE_ARVORES, 0.0],
        },
    ));

    // ⭐ O CHÃO e os POSTES vivem a `k = 1` — eles NÃO têm `ScrollFactor`, e a ausência é a lei:
    // o mundo é o plano de referência, e um `ScrollFactor { k: [1,1] }` seria a mesma coisa escrita
    // duas vezes (o passe salta o neutro por construção).
    world.spawn((
        Name::new("Chao"),
        Sprite::atlas(WHITE_TILE_KEY, [400.0, ALTURA_DO_CHAO], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, Y_CHAO)),
    ));
    // ⚠️ A `2` m e não a `4`: a régua tem de ter vizinhas na janela, pela mesma razão das árvores.
    // ⚠️ E penduradas num pai SEM `ScrollFactor`: a foto mostrou 121 linhas soltas na Hierarchy, e o
    // roteiro manda o dono procurar lá o «Ceu» e as «Colinas».
    let postes = world
        .spawn((Name::new("Postes"), Transform::from_translation(Vec2::ZERO)))
        .id();
    for i in -60..=60 {
        #[allow(clippy::cast_precision_loss)]
        let x = f32::from(i16::try_from(i).unwrap_or(0)) * 2.0;
        world.spawn((
            Name::new(format!("Poste {i}")),
            Sprite::atlas(WHITE_TILE_KEY, [0.35, 0.7], POSTE_RGBA),
            Transform::from_translation(Vec2::new(x, Y_TOPO_DO_CHAO + 0.35)),
            ChildOf(postes),
        ));
    }
}

/// ⭐ **A câmera do jogo, presa em Y** — ver o cabeçalho.
///
/// ⚠️ `damping` a zero de propósito: com amortecimento o dono não sabe se o que vê é a paralaxe ou
/// a câmera a apanhar o herói.
fn camera(world: &mut World, alvo: &str) -> Entity {
    world
        .spawn((
            Name::new("Camera"),
            Transform::from_translation(Vec2::ZERO),
            GameCamera::default(),
            CameraFollow {
                target: alvo.to_owned(),
                damping: [0.0, 0.0],
                ..CameraFollow::default()
            },
            // ⚠️ **A região tem a ALTURA EXACTA da vista** ⇒ `lo > hi` em Y e a lei devolve o meio,
            // que é `0` — a câmera fica pregada. Em X ela é larga o bastante para a caminhada.
            CameraLimits {
                min: [-400.0, -MEIA_VISTA_Y],
                max: [400.0, MEIA_VISTA_Y],
            },
        ))
        .id()
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — as ÁRVORES, que é onde o roteiro manda olhar.
fn cena_um(world: &mut World) -> Entity {
    cenario(world);

    // ⭐ O HERÓI: anda com as setas, e a câmera segue-o ⇒ **andar é mover a VISTA**.
    //
    // ⛔⛔ **O CORPO CINEMÁTICO NÃO É DECORAÇÃO** (report do dono, 19/09): a ponte do mover varre
    // `self.bodies`, logo quem não tem corpo nunca entra no laço — e um corpo `Dynamic` é do
    // SOLVER, logo a gravidade leva-o para fora do ecrã em dois segundos.
    world.spawn((
        Name::new("Heroi"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.45 },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.9, 0.9], HEROI_RGBA),
        Transform::from_translation(Vec2::new(0.0, Y_TOPO_DO_CHAO + 0.45)),
        ph2d_physics_ecs::TopDownPlayer::from_law(TopDownLaw {
            speed: 6.0,
            direction: DirectionMode::Free,
            ..TopDownLaw::default()
        }),
    ));

    camera(world, "Heroi");

    // ⛔ **As ÁRVORES nascem escolhidas, e é o ROTEIRO que o exige:** o passo (3) manda ver a
    // secção *Parallax* no painel da direita, e com ninguém escolhido o Inspector diz *«Select an
    // entity in the Hierarchy»*. *Um passo que nomeia uma secção AFIRMA que ela está na tela.*
    entidade_por_nome(world, "Arvores")
}

/// A cena `=2` — o DOLLY. **Ninguém anda**, e a câmera nasce escolhida.
fn cena_dois(world: &mut World) -> Entity {
    cenario(world);
    // ⚠️ **Sem herói e sem alvo**: o dolly é a ÚNICA coisa que se mexe, e é isso que o torna
    // legível. Com a câmera a seguir alguém o dono não sabe o que é o dolly e o que é o passo.
    camera(world, "")
}

/// ⭐ **Acha uma raiz pelo nome** — a cena escolhe por NOME e nunca por bits guardados a meio da
/// montagem, que é a lei desta casa para toda referência durável.
///
/// ⚠️ **Por uma QUERY e não por `iter_entities`** — medido: a varredura de `EntityRef` não achava
/// um `Name` que a query acha, e o fallback do `from_bits(0)` rebentava a cena inteira.
///
/// # Panics
/// Se a cena não tiver o nome — é um defeito do `cenario`, e ele tem de falar alto.
fn entidade_por_nome(world: &mut World, nome: &str) -> Entity {
    world
        .query::<(Entity, &Name)>()
        .iter(world)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena da paralaxe não tem «{nome}»"))
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
#[must_use]
pub fn montar(world: &mut World, nivel: u32) -> Montada {
    let nivel = if nivel == 2 { 2 } else { 1 };
    let escolhido = if nivel == 2 {
        cena_dois(world)
    } else {
        cena_um(world)
    };
    ph2d_ecs::assign_missing_stable_ids(world);
    // ⚠️⚠️ **Os nomes que o roteiro manda procurar saem da MESMA porta que o painel pinta** (`tr`) e
    // nunca de um literal: *um passo que nomeia uma fileira AFIRMA que ela está na tela com aquele
    // nome*, e um literal aqui envelhece no dia em que alguém renomear a chave — com o dono a
    // aprovar o smoke com o passo impossível dentro. Há gate.
    let t = ph2d_i18n::tr;
    let (camera, dolly) = (
        t("panel.inspector.camera.camera"),
        t("panel.inspector.camera.dolly"),
    );
    let (secao, factor, repeat, drift, lmax) = (
        t("panel.inspector.parallax.parallax"),
        t("panel.inspector.parallax.scroll_factor"),
        t("panel.inspector.parallax.repeat_m"),
        t("panel.inspector.parallax.drift_m_s"),
        t("panel.inspector.parallax.limit_max_m"),
    );
    if nivel == 2 {
        println!(
            "[parallax-smoke] cena=2 — o DOLLY\n\
             (1) a «Camera» ja' esta' escolhida: no painel da direita procure a seccao `{camera}` \
             e a fileira `{dolly}`, que nasce em 0\n\
             (2) arraste o numero do `{dolly}` para a DIREITA: o chao e os postes ficam do MESMO \
             tamanho e os tres fundos ENCOLHEM — as arvores pouco, as colinas mais, o ceu muito\n\
             (3) arraste-o para a ESQUERDA: o contrario, os fundos CRESCEM\n\
             (4) isto NAO e' o zoom: um zoom mexia tambem no chao. E' a camera a APROXIMAR-SE do \
             plano do mundo, e cada fundo obedece a' distancia dele\n\
             (5) deu errado se: o chao mudar de tamanho · algum fundo NAO mudar · ou a imagem \
             saltar em vez de crescer devagar"
        );
    } else {
        println!(
            "[parallax-smoke] cena=1 — o FUNDO\n\
             (1) ande para a DIREITA com as SETAS e olhe para os QUATRO planos: os postes cinzentos \
             do chao passam por si, as arvores escuras ficam para tras, as colinas mal se mexem e o \
             ceu parece parado. E' UM numero que os separa\n\
             (2) continue a andar bastante: as arvores e o ceu NUNCA acabam — eles repetem. As \
             COLINAS, essas, PARAM: elas tem borda, e a cena ensina as duas leis lado a lado\n\
             (3) no painel da direita (as «Arvores» ja' estao escolhidas) procure a seccao \
             `{secao}` e a fileira `{factor}`: ponha o primeiro numero em 1 e ande — agora as \
             arvores andam COM o chao e a profundidade desaparece. Ponha-o em 0 e elas ficam \
             presas ao ecra\n\
             (4) escolha o «Ceu» na Hierarchy: alem do factor ele tem `{repeat}` (o tamanho do \
             ladrilho) e `{drift}` (a deriva). Pare de andar e repare que as nuvens continuam a \
             passar sozinhas\n\
             (5) escolha as «Colinas»: elas nao tem `{repeat}`, tem a cerca — e' ela que as faz \
             parar no passo (2). Suba o primeiro numero de `{lmax}` de 30 para 80 e ande outra \
             vez: a serra vai mais longe\n\
             (6) deu errado se: todos os planos andarem a mesma velocidade · aparecer uma emenda no \
             ceu ou nas arvores · as colinas nunca pararem · ou o fundo SALTAR quando o heroi muda \
             de direccao"
        );
    }
    Montada {
        nivel,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "parallax_smoke_tests.rs"]
mod tests;
