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
    AnimationTag, CameraFollow, CameraLimits, ChildOf, Entity, GameCamera, Name, ScrollFactor,
    SimWorld, SpriteAnimations, SpriteAnimator, SpriteGrid, TiledDraw, Transform, ZIndexOverride,
};
use ph2d_editor_core::game_stage::GameStage;
use ph2d_render::{
    CLOUDS_TILE, COIN_FRAME_MS, COIN_SHEET, DIRT_TILE, FLAG_FRAME_MS, FLAG_SHEET, FactoryArt,
    GROUND_TILE, HILLS_TILE, PLATFORM_TILE, Sprite, WHITE_TILE_KEY,
};
use ph2d_script::{LuauScript, ProjectScripts};
use std::sync::OnceLock;

pub const CENAS: u32 = 4;

/// A cena 4 (plano 12): o nível 2 com duas imagens na biblioteca do projecto para trocar pelo Inspector — a
/// folha do herói azul e os montes ao pôr do sol (`docs/GameEngineFuncional/ferramentas/arte_trocar.py`).
pub const IMAGENS_4: [&str; 2] = [
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/GameEngineFuncional/arte/heroi_azul.png"
    ),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/GameEngineFuncional/arte/montes_por_do_sol.png"
    ),
];

/// As imagens que a cena `nivel` traz para a biblioteca.
#[must_use]
pub fn imagens_da_cena(nivel: u32) -> &'static [&'static str] {
    if nivel == 4 { &IMAGENS_4 } else { &[] }
}

// As fotos do dono (`PH2D_GAME_SMOKE_FOTO`), irmãs pelo tecto de LOC.
#[path = "smoke_foto.rs"]
mod foto;
pub use foto::{
    CENTRO_DAS_FOTOS_13, CENTRO_DOS_USOS, Foto, FotoDaCamera, ONDE_LARGA, PassoDoDono, PlanoDaFoto,
    QUADROS_POR_USO, estado_do_player, foto, plano_da_foto, uso_no_quadro, usos_do_dono,
};

/// O herói das cenas de smoke — nome de objecto da cena, não texto da interface.
#[must_use]
pub fn player(sim: &mut SimWorld) -> Option<Entity> {
    por_nome(sim, "Player")
}

/// O objecto da raiz com este nome.
#[must_use]
pub fn por_nome(sim: &mut SimWorld, nome: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &ph2d_ecs::Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e)
}

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
        2 | 4 => &HORARIO_2,
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
            println!(
                "[game-smoke] =3 o nível 2 com o condutor do agachar (plano 10): com ↓ e ← o boneco \
                 anda agachado («crouch_walk») sob o tecto baixo da esquerda, solta ↓ lá debaixo e \
                 continua baixo («crouch»), e ao sair para a direita levanta."
            );
            jogador
        }
        4 => {
            let jogador = nivel_2(sim, registry);
            println!(
                "[game-smoke] =4 o nível 2 com «heroi_azul» e «montes_por_do_sol» na biblioteca (plano 12): \
                 escolha a imagem dos montes e a folha do Player pela ranhura Image do Inspector."
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

/// O fundo do nível 2 (plano 11): o céu parado no ecrã, maior que a vista mais larga, na cor do céu do fundo
/// azul do Kenney (`#DFF6F5`).
pub const CEU_TAMANHO: [f32; 2] = [40.0, 16.0];
/// ⚠️ O tingimento é LINEAR: `#DFF6F5` pela curva sRGB (IEC 61966-2-1). Escrito como `223/255` saía `#F0FBFA`
/// (medido na foto do plano 11).
pub const CEU_COR: [f32; 4] = [0.7379, 0.9216, 0.9131, 1.0];
/// As camadas de fundo: (nome, fracção da câmara que guardam, centro no MUNDO na vista do início, ordem).
pub const CAMADAS_2: [(&str, f32, [f32; 2], i32); 2] = [
    ("Clouds", 0.15, [-5.9, 5.6], -2),
    ("Hills", crate::object_add::CAMADA_K, [-5.9, 0.44], -1),
];
/// O centro da câmara quando o nível começa (o jogador em `[-8, 1,5]` encostado ao limite da esquerda, 16:9).
pub const CENTRO_DO_INICIO: [f32; 2] = [-5.9, 3.0];

/// A pose AUTORADA que põe uma camada com fracção `k` em `no_mundo` quando a câmara está em
/// [`CENTRO_DO_INICIO`] (a ponte desloca-a de `centro·(1 − k)`).
#[must_use]
pub fn autorada(no_mundo: [f32; 2], k: f32) -> [f32; 2] {
    [
        no_mundo[0] - CENTRO_DO_INICIO[0] * (1.0 - k),
        no_mundo[1] - CENTRO_DO_INICIO[1] * (1.0 - k),
    ]
}

/// Veste um bloco com o azulejo de fábrica do seu papel em *Tiled* ao tamanho do bloco, escala 1 e a caixa do
/// collider igual ao desenho (plano GEF 14 D6: o que o artista faz no Inspector — a imagem, Draw Mode Tiled e
/// a Largura/Altura).
fn azulejo(sim: &mut SimWorld, e: Entity, nome: &str, meia: [f32; 2]) {
    let art = match nome {
        "Platform Low" | "Platform High" => PLATFORM_TILE,
        "Wall Left" => DIRT_TILE,
        _ => GROUND_TILE,
    };
    let [cw, ch] = art.cell_px();
    let lado_m = [1.0, ch as f32 / cw as f32];
    let w = sim.world_mut();
    w.entity_mut(e).insert((
        Sprite::atlas(art.key, [2.0 * meia[0], 2.0 * meia[1]], [1.0; 4]),
        TiledDraw { tile: lado_m },
    ));
    caixa(w, e, meia);
}

/// A caixa do collider de `e` com meia-extensão `meia` (m).
fn caixa(w: &mut ph2d_ecs::World, e: Entity, meia: [f32; 2]) {
    use ph2d_physics_ecs::{Collider, ColliderShape};
    if let Some(mut c) = w.get_mut::<Collider>(e) {
        c.shape = ColliderShape::Cuboid {
            half_x: meia[0],
            half_y: meia[1],
        };
    }
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
    use crate::object_add::{ANIMATED_SPRITE, CHARACTER, COLLISION_SHAPE, SPRITE, STATIC_BODY};
    let mut chao_direito = None;
    for (nome, c, h) in BLOCOS_2 {
        let e = do_menu(sim, registry, STATIC_BODY);
        arruma(sim, e, nome, c, [1.0, 1.0]);
        azulejo(sim, e, nome, h);
        if nome == "Ground Right" {
            chao_direito = Some((e, c, h));
        }
    }
    // A parede do fim: uma forma DENTRO do chão, de escala 1 — o tamanho é a caixa do collider.
    let (chao, c, _) = chao_direito.expect("o chão da direita");
    let parede = do_menu(sim, registry, COLLISION_SHAPE);
    let ([px, py], meia) = PAREDE_INVISIVEL;
    arruma(
        sim,
        parede,
        "Invisible Wall",
        [px - c[0], py - c[1]],
        [1.0, 1.0],
    );
    let w = sim.world_mut();
    caixa(w, parede, meia);
    w.entity_mut(parede).insert(ChildOf(chao));
    // O céu é uma Sprite com a Parallax a 0 (parada no ecrã); as nuvens e os montes são Background Layers.
    let ceu = do_menu(sim, registry, SPRITE);
    arruma(sim, ceu, "Sky", [0.0, 0.0], CEU_TAMANHO);
    sim.world_mut()
        .entity_mut(ceu)
        .insert((ZIndexOverride(-3), ScrollFactor { k: [0.0, 0.0] }));
    if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(ceu) {
        s.tint = CEU_COR;
    }
    for (nome, k, no_mundo, ordem) in CAMADAS_2 {
        let e = do_menu(sim, registry, crate::object_add::BACKGROUND_LAYER);
        arruma(sim, e, nome, autorada(no_mundo, k), [1.0, 1.0]);
        let art = if nome == "Clouds" {
            CLOUDS_TILE
        } else {
            HILLS_TILE
        };
        let w = sim.world_mut();
        crate::object_add::veste_camada(w, e, art, crate::object_add::CAMADA_LADRILHOS, k);
        w.entity_mut(e).insert(ZIndexOverride(ordem));
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
        // Da borda de fora da parede ao fim do chão da direita: a vista nunca mostra o fim do mundo.
        CameraLimits {
            min: [-13.0, -1.0],
            max: [17.0, 9.0],
        },
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    println!(
        "[game-smoke] =2 o mesmo nível feito SÓ com o menu Add (Add ▸ Game), com a arte do Kenney \
         redesenhada em vetor: chão e plataformas de relva (Static Body), a parede invisível do fim \
         (Collision Shape dentro do chão), um tecto baixo à esquerda (segure ↓ para passar agachado), o jogador (Character), as moedas a girar e a bandeira a \
         ondular (Animated Sprite), o céu parado (Sprite com Parallax 0) e, ao longe, as nuvens e os \
         montes (Background Layer), que andam mais devagar que o chão quando a câmara segue o jogador. Run ▸ Play in a Window: o boneco fica parado, \
         corre, salta e cai — a animação troca sozinha — e as moedas somem ao tocar."
    );
    jogador
}

#[cfg(test)]
#[path = "smoke_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "camara_tests.rs"]
mod camara_tests;
