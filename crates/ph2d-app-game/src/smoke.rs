//! ⭐ **O JOGO DE PLATAFORMA do smoke** (`PH2D_GAME_SMOKE`, plano 03): o nível que cresce a cada onda.
//!
//! - **1** (J1): um nível — chão com um vão, duas plataformas, paredes, a meta — e o jogador de
//!   plataforma (setas/A-D andam, ↑/Z salta), a câmara a segui-lo, e cinco moedas com o
//!   `moeda.luau`, que somem quando o jogador lhes toca. A cena nasce PARADA: o Play abre o jogo e o
//!   Stop devolve tudo, moedas incluídas.
//! - **2** (J2 + J3, `05_N1_plano.md` §5): o mesmo desenho de nível montado SÓ com os objectos do menu
//!   Add ([`crate::object_add`]) — chão e plataformas são *Static Body*, a parede do fim é uma *Collision
//!   Shape* dentro do chão, o jogador é um *Character* (o boneco anima-se pelo estado: parado, correr,
//!   saltar, cair), as moedas e a bandeira são *Animated Sprite* e o céu é um *Sprite*. A arte é a de
//!   fábrica (`ph2d_render::FACTORY_ART`, plano 06): os blocos repetem o azulejo pelo `UvTransform`.
//! - **3** (plano 09): o nível 2 com a regra «Crouching → fall» no topo e o condutor a agachar sob o
//!   «Low Ceiling» (← e ↓), a soltar ↓ lá debaixo e a sair para a direita.

use ph2d_core::Vec2;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{
    AnimationTag, CameraFollow, CameraLimits, ChildOf, Entity, GameCamera, Name, RepeatMode,
    SimWorld, SpriteAnimations, SpriteAnimator, SpriteGrid, TextureRepeat, Transform, UvTransform,
    ZIndexOverride,
};
use ph2d_editor_core::game_stage::GameStage;
use ph2d_render::{
    COIN_FRAME_MS, COIN_SHEET, DIRT_TILE, FLAG_FRAME_MS, FLAG_SHEET, FactoryArt, GROUND_TILE,
    PLATFORM_TILE, Sprite, WHITE_TILE_KEY,
};
use ph2d_script::{LuauScript, ProjectScripts};
use std::sync::OnceLock;

pub const CENAS: u32 = 3;

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_GAME_SMOKE",
    max_level: CENAS,
}];

#[must_use]
pub fn armed_level_in(get: impl Fn(&str) -> Option<String>) -> Option<u32> {
    get(ROUTERS[0].env)?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=CENAS).contains(n))
}

#[must_use]
pub fn armed_level() -> Option<u32> {
    static LEVEL: OnceLock<Option<u32>> = OnceLock::new();
    *LEVEL.get_or_init(|| armed_level_in(|k| std::env::var(k).ok()))
}

static MONTADA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A cena monta uma vez por processo: `true` na primeira pergunta, `false` depois.
#[must_use]
pub fn monta_agora() -> bool {
    !MONTADA.swap(true, std::sync::atomic::Ordering::Relaxed)
}

/// Um gesto do condutor do smoke — para fotografar o Play numa tela onde a mão não chega.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesto {
    /// Escolhe o palco no menu *Run* e joga.
    Joga(GameStage),
    /// A seta para a direita, premida (`true`) ou largada.
    Direita(bool),
    /// A seta para cima (saltar), premida ou largada.
    Salta(bool),
    Para,
    /// A seta para a esquerda, premida ou largada.
    Esquerda(bool),
    /// A seta para baixo (agachar), premida ou largada.
    Abaixa(bool),
}

/// A seta que um gesto prime ou larga.
/// ⚠️ A shell indexa as teclas pela ordem destas variantes (`janela_do_jogo.rs`): append-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Seta {
    Direita,
    Cima,
    Esquerda,
    Baixo,
}

impl Gesto {
    /// A seta deste gesto e se desce (`true`) ou sobe — `None` se o gesto não é de teclado.
    #[must_use]
    pub fn seta(self) -> Option<(Seta, bool)> {
        match self {
            Gesto::Direita(d) => Some((Seta::Direita, d)),
            Gesto::Salta(d) => Some((Seta::Cima, d)),
            Gesto::Esquerda(d) => Some((Seta::Esquerda, d)),
            Gesto::Abaixa(d) => Some((Seta::Baixo, d)),
            _ => None,
        }
    }
}

/// O condutor (`PH2D_GAME_SMOKE_DRIVE=window|fullscreen|editor`), lido pela porta injetável.
#[must_use]
pub fn conducao_in(get: impl Fn(&str) -> Option<String>) -> Option<GameStage> {
    match get("PH2D_GAME_SMOKE_DRIVE")?.trim() {
        "window" => Some(GameStage::Window),
        "fullscreen" => Some(GameStage::Fullscreen),
        "editor" => Some(GameStage::Editor),
        _ => None,
    }
}

#[must_use]
pub fn conducao() -> Option<GameStage> {
    static C: OnceLock<Option<GameStage>> = OnceLock::new();
    *C.get_or_init(|| conducao_in(|k| std::env::var(k).ok()))
}

/// O horário. Nível 1: joga ao quadro 60, corre para a direita (apanha as duas primeiras moedas) e pára
/// ao 420. Nível 2: corre, salta para a plataforma baixa, sai dela a cair, aterra no chão e pára — as
/// quatro animações por esta ordem.
#[must_use]
pub fn gesto_no_quadro(nivel: u32, stage: GameStage, quadro: u32) -> Option<Gesto> {
    let horario: &[(u32, Gesto)] = match nivel {
        2 => &HORARIO_2,
        3 => &HORARIO_3,
        _ => &HORARIO_1,
    };
    match quadro {
        60 => Some(Gesto::Joga(stage)),
        _ => horario.iter().find(|(q, _)| *q == quadro).map(|(_, g)| *g),
    }
}

const HORARIO_1: [(u32, Gesto); 3] = [
    (120, Gesto::Direita(true)),
    (240, Gesto::Direita(false)),
    (420, Gesto::Para),
];

/// Quadros do condutor do nível 2 (o Play entra ao 60; a física começa no quadro seguinte).
pub const HORARIO_2: [(u32, Gesto); 5] = [
    (120, Gesto::Direita(true)),
    (QUADRO_DO_SALTO, Gesto::Salta(true)),
    (QUADRO_DO_SALTO + 12, Gesto::Salta(false)),
    (330, Gesto::Direita(false)),
    (480, Gesto::Para),
];
/// O nível 3: ← e ↓ até à parede (sob o tecto baixo), solta ↓ lá debaixo (continua agachado), sai para
/// a direita (levanta) e pára.
pub const HORARIO_3: [(u32, Gesto); 7] = [
    (120, Gesto::Esquerda(true)),
    (125, Gesto::Abaixa(true)),
    (240, Gesto::Esquerda(false)),
    (250, Gesto::Abaixa(false)),
    (330, Gesto::Direita(true)),
    (420, Gesto::Direita(false)),
    (480, Gesto::Para),
];
/// Onde o salto do nível 2 sai (afinado pelo gate `o_nivel_2_anda_salta_cai_e_anima`).
pub const QUADRO_DO_SALTO: u32 = 190;

static QUADRO: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// O quadro do condutor desde que a cena montou (um por chamada).
#[must_use]
pub fn proximo_quadro() -> u32 {
    QUADRO.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub const MOEDA_NAME: &str = "moeda.luau";

/// O script de cada moeda: some quando o jogador lhe toca.
pub const MOEDA_LUAU: &str = r#"-- moeda.luau — cada MOEDA amarela corre ESTE script.
-- Quando o jogador lhe toca, ela some. O Stop devolve-a: o jogo corre sobre uma cópia da cena.

ph2d.property("reach", 0.6, { min = 0.1, max = 3 }) -- a distância a que o jogador a apanha

function update(self, dt)
  local p = ph2d.find_by_name("Player")
  if not p then return end
  local dx = (ph2d.get(p, "x") or 0) - (ph2d.get(self.id, "x") or 0)
  local dy = (ph2d.get(p, "y") or 0) - (ph2d.get(self.id, "y") or 0)
  if dx * dx + dy * dy < self.reach * self.reach then
    print("coin!")
    ph2d.despawn(self.id)
  end
end
"#;

/// Os blocos do nível: nome, centro, meia-extensão (metros).
const BLOCOS: [(&str, [f32; 2], [f32; 2]); 6] = [
    ("Ground Left", [-5.0, -0.5], [7.0, 0.5]),
    ("Ground Right", [11.0, -0.5], [6.0, 0.5]),
    ("Platform Low", [3.5, 1.6], [1.2, 0.25]),
    ("Platform High", [9.0, 3.0], [1.5, 0.25]),
    ("Wall Left", [-12.5, 3.0], [0.5, 4.0]),
    ("Wall Right", [17.5, 3.0], [0.5, 4.0]),
];

const MOEDAS: [[f32; 2]; 5] = [
    [-4.0, 0.7],
    [-1.5, 0.7],
    [3.5, 2.4],
    [9.0, 3.8],
    [14.0, 0.7],
];

const CHAO: [f32; 4] = [0.35, 0.35, 0.40, 1.0];
const OURO: [f32; 4] = [0.98, 0.80, 0.18, 1.0];
const META: [f32; 4] = [0.30, 0.80, 0.40, 1.0];

/// Monta o nível `nivel` no mundo. Devolve o jogador.
pub fn montar(sim: &mut SimWorld, registry: &ComponentRegistry, nivel: u32) -> Entity {
    match nivel {
        2 => nivel_2(sim, registry),
        3 => {
            let jogador = nivel_2(sim, registry);
            let mut regras = vec![ph2d_physics_ecs::AnimRule {
                when: ph2d_physics_ecs::AnimWhen::Pose(ph2d_physics_ecs::CharacterPose::Crouch),
                play: "fall".into(),
            }];
            regras.extend(ph2d_physics_ecs::AnimRules::of_character(&Default::default()).0);
            sim.world_mut()
                .entity_mut(jogador)
                .insert(ph2d_physics_ecs::AnimRules(regras));
            println!(
                "[game-smoke] =3 o nível 2 com a regra «Crouching → fall» no topo (não há desenho de \
                 agachar): com ↓ e ← o boneco entra agachado sob o tecto baixo da esquerda, solta ↓ \
                 lá debaixo e continua baixo, e ao sair para a direita levanta."
            );
            jogador
        }
        _ => nivel_1(sim.world_mut()),
    }
}

fn nivel_1(world: &mut ph2d_ecs::World) -> Entity {
    for (nome, c, h) in BLOCOS {
        ph2d_app_physics::common::slab(world, nome, Vec2::new(c[0], c[1]), h, 0.0, CHAO);
    }
    world.spawn((
        Name::new("Goal"),
        Transform::from_translation(Vec2::new(15.5, 1.0)),
        Sprite::atlas(WHITE_TILE_KEY, [0.3, 2.0], META),
    ));
    let moeda = world
        .get_resource_or_init::<ProjectScripts>()
        .insert(MOEDA_NAME, MOEDA_LUAU);
    for (i, at) in MOEDAS.into_iter().enumerate() {
        world.spawn((
            Name::new(format!("Coin {}", i + 1)),
            Transform::from_translation(Vec2::new(at[0], at[1])),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 0.4], OURO),
            LuauScript::library(moeda),
        ));
    }
    let jogador = ph2d_app_physics::common::spawn_player(world, Vec2::new(-8.0, 1.5));
    world.spawn((
        Transform::from_translation(Vec2::new(-8.0, 2.0)),
        Name::new("Camera"),
        GameCamera::default(),
        CameraFollow {
            target: "Player".into(),
            damping: [5.0, 5.0],
            dead_zone: [0.25, 0.25],
            lookahead: [0.0, 0.0],
            offset: [0.0, 1.0],
        },
        CameraLimits {
            min: [-13.0, -1.0],
            max: [18.0, 9.0],
        },
    ));
    ph2d_ecs::assign_missing_stable_ids(world);
    println!(
        "[game-smoke] =1 o nível: chão com um vão, duas plataformas, a meta verde e cinco moedas. \
         A cena está PARADA. Run ▸ Play in a Window (ou F5) abre o jogo numa janela; setas andam, \
         ↑ salta, as moedas somem ao tocar. Shift+F5 (ou fechar a janela) pára — e tudo volta."
    );
    jogador
}

/// Um objecto do menu Add, pela porta do menu (as sementes do app incluídas).
fn do_menu(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    entry: ph2d_editor_core::object_add::AddEntry,
) -> Entity {
    let seeds = ph2d_app_physics::physics_seed::COMPONENT_SEEDS;
    let bits = crate::object_add::add(entry, sim, registry, seeds)
        .or_else(|| ph2d_app_components::object_add::add(entry, sim, registry, seeds))
        .expect("a entrada é de uma família do menu")
        .unwrap_or_else(|e| panic!("{}: {e}", entry.key.key()));
    Entity::from_bits(bits)
}

/// Põe nome, posição e escala — o que o artista faz depois de escolher no menu.
fn arruma(sim: &mut SimWorld, e: Entity, nome: &str, at: [f32; 2], escala: [f32; 2]) {
    let mut em = sim.world_mut().entity_mut(e);
    em.insert(Name::new(nome));
    let mut t = em.get_mut::<Transform>().expect("todo objecto tem pose");
    t.translation = Vec2::new(at[0], at[1]);
    t.scale = Vec2::new(escala[0], escala[1]);
}

/// Os blocos do nível 2 (*Static Body*): o desenho do nível 1 com as plataformas à altura do salto do
/// boneco de fábrica (sobe ~1,4 m; a baixa tem o topo a 1,1 m, a alta alcança-se da baixa).
pub const BLOCOS_2: [(&str, [f32; 2], [f32; 2]); 6] = [
    ("Ground Left", [-5.0, -0.5], [7.0, 0.5]),
    ("Ground Right", [11.0, -0.5], [6.0, 0.5]),
    ("Platform Low", [3.5, 0.85], [1.2, 0.25]),
    ("Platform High", [8.5, 2.0], [1.5, 0.25]),
    ("Wall Left", [-12.5, 3.0], [0.5, 4.0]),
    // Fundo a 0,60 m: de pé (topo 0,80) não cabe, agachado (0,50) passa — plano 09.
    ("Low Ceiling", [-10.5, 0.85], [1.5, 0.25]),
];

/// A parede invisível do fim do nível: centro e meia-extensão no mundo (m).
pub const PAREDE_INVISIVEL: ([f32; 2], [f32; 2]) = ([17.5, 3.0], [0.5, 4.0]);

/// As moedas do nível 2: sobre o chão, sobre cada plataforma e antes da meta.
pub const MOEDAS_2: [[f32; 2]; 5] = [
    [-4.0, 0.6],
    [-1.5, 0.6],
    [3.5, 1.65],
    [8.5, 2.8],
    [13.5, 0.6],
];

/// O céu do nível 2 (centro, tamanho em m) e a cor do fundo do pacote do Kenney (`#C2E3E8`).
const CEU: ([f32; 2], [f32; 2]) = ([2.5, 4.0], [40.0, 16.0]);
const CEU_COR: [f32; 4] = [194.0 / 255.0, 227.0 / 255.0, 232.0 / 255.0, 1.0];

/// Veste um bloco com o azulejo de fábrica do seu papel, repetido ao tamanho do bloco (o que o artista faz
/// no Inspector: a imagem, *Repeat* e a escala do UV).
fn azulejo(sim: &mut SimWorld, e: Entity, nome: &str, meia: [f32; 2]) {
    let art = match nome {
        "Platform Low" | "Platform High" => PLATFORM_TILE,
        "Wall Left" => DIRT_TILE,
        _ => GROUND_TILE,
    };
    let [cw, ch] = art.cell_px();
    let lado_m = [1.0, ch as f32 / cw as f32];
    sim.world_mut().entity_mut(e).insert((
        Sprite::atlas(art.key, crate::object_add::BLOCO, [1.0; 4]),
        SpriteGrid {
            hframes: art.grid[0],
            vframes: art.grid[1],
            frame: art.middle_frame(),
        },
        UvTransform {
            scale: [2.0 * meia[0] / lado_m[0], 2.0 * meia[1] / lado_m[1]],
            offset: [0.0, 0.0],
        },
        TextureRepeat(RepeatMode::Enabled),
    ));
}

/// Troca a folha de um *Animated Sprite* por uma folha de fábrica de uma linha, a tocar em laço.
fn folha(sim: &mut SimWorld, e: Entity, art: FactoryArt, tag: &str, ms: u32) {
    let mut tags = SpriteAnimations::new();
    let mut t = AnimationTag::new(tag, 0, art.grid[0] - 1);
    t.frame_ms = ms;
    tags.insert(t).expect("uma tag");
    let mut animator = SpriteAnimator::new(tag);
    animator.autoplay = true;
    sim.world_mut().entity_mut(e).insert((
        Sprite::atlas(art.key, [1.0, 1.0], [1.0; 4]),
        SpriteGrid {
            hframes: art.grid[0],
            vframes: art.grid[1],
            frame: 0,
        },
        tags,
        animator,
    ));
}

/// O nível 2 — ver o cabeçalho do módulo.
fn nivel_2(sim: &mut SimWorld, registry: &ComponentRegistry) -> Entity {
    use crate::object_add::{
        ANIMATED_SPRITE, BLOCO, CHARACTER, COLLISION_SHAPE, SPRITE, STATIC_BODY,
    };
    let mut chao_direito = None;
    for (nome, c, h) in BLOCOS_2 {
        let e = do_menu(sim, registry, STATIC_BODY);
        arruma(
            sim,
            e,
            nome,
            c,
            [2.0 * h[0] / BLOCO[0], 2.0 * h[1] / BLOCO[1]],
        );
        azulejo(sim, e, nome, h);
        if nome == "Ground Right" {
            chao_direito = Some((e, c, h));
        }
    }
    // A parede do fim: uma forma DENTRO do chão — a pose local desfaz a escala do pai.
    let (chao, c, h) = chao_direito.expect("o chão da direita");
    let escala = [2.0 * h[0] / BLOCO[0], 2.0 * h[1] / BLOCO[1]];
    let parede = do_menu(sim, registry, COLLISION_SHAPE);
    let ([px, py], [hx, hy]) = PAREDE_INVISIVEL;
    arruma(
        sim,
        parede,
        "Invisible Wall",
        [(px - c[0]) / escala[0], (py - c[1]) / escala[1]],
        [2.0 * hx / escala[0], 2.0 * hy / escala[1]],
    );
    sim.world_mut().entity_mut(parede).insert(ChildOf(chao));
    let ceu = do_menu(sim, registry, SPRITE);
    arruma(sim, ceu, "Sky", CEU.0, CEU.1);
    sim.world_mut().entity_mut(ceu).insert(ZIndexOverride(-1));
    if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(ceu) {
        s.tint = CEU_COR;
    }
    let meta = do_menu(sim, registry, ANIMATED_SPRITE);
    arruma(sim, meta, "Goal", [15.5, 1.0], [1.0, 2.0]);
    folha(sim, meta, FLAG_SHEET, "wave", FLAG_FRAME_MS);
    let moeda = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert(MOEDA_NAME, MOEDA_LUAU);
    for (i, at) in MOEDAS_2.into_iter().enumerate() {
        let e = do_menu(sim, registry, ANIMATED_SPRITE);
        arruma(sim, e, &format!("Coin {}", i + 1), at, [0.5, 0.5]);
        folha(sim, e, COIN_SHEET, "spin", COIN_FRAME_MS);
        sim.world_mut()
            .entity_mut(e)
            .insert(LuauScript::library(moeda));
    }
    let jogador = do_menu(sim, registry, CHARACTER);
    arruma(sim, jogador, "Player", [-8.0, 1.5], [1.0, 1.0]);
    let camara = do_menu(sim, registry, ph2d_app_components::object_add::CAMERA);
    arruma(sim, camara, "Camera", [-8.0, 2.0], [1.0, 1.0]);
    // Mais perto que os 10 m de fábrica: o boneco desenhado lê-se (~110 px de altura numa janela de 900).
    if let Some(mut c) = sim.world_mut().get_mut::<GameCamera>(camara) {
        c.height_world = 8.0;
    }
    sim.world_mut().entity_mut(camara).insert((
        CameraFollow {
            target: "Player".into(),
            damping: [5.0, 5.0],
            dead_zone: [0.25, 0.25],
            lookahead: [0.0, 0.0],
            offset: [0.0, 1.0],
        },
        CameraLimits {
            min: [-13.0, -1.0],
            max: [18.0, 9.0],
        },
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    println!(
        "[game-smoke] =2 o mesmo nível feito SÓ com o menu Add (Add ▸ Game), com a arte do Kenney \
         redesenhada em vetor: chão e plataformas de relva (Static Body), a parede invisível do fim \
         (Collision Shape dentro do chão), um tecto baixo à esquerda (segure ↓ para passar agachado), o jogador (Character), as moedas a girar e a bandeira a \
         ondular (Animated Sprite) e o céu (Sprite). Run ▸ Play in a Window: o boneco fica parado, \
         corre, salta e cai — a animação troca sozinha — e as moedas somem ao tocar."
    );
    jogador
}

#[cfg(test)]
#[path = "smoke_tests.rs"]
mod tests;
