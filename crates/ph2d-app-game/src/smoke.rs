//! ⭐ **O JOGO DE PLATAFORMA do smoke** (`PH2D_GAME_SMOKE`, plano 03): o nível que cresce a cada onda.
//!
//! - **1** (J1): um nível — chão com um vão, duas plataformas, paredes, a meta — e o jogador de
//!   plataforma (setas/A-D andam, ↑/Z salta), a câmara a segui-lo, e cinco moedas com o
//!   `moeda.luau`, que somem quando o jogador lhes toca. A cena nasce PARADA: o Play abre o jogo e o
//!   Stop devolve tudo, moedas incluídas.

use ph2d_core::Vec2;
use ph2d_ecs::{CameraFollow, CameraLimits, GameCamera, Name, Transform};
use ph2d_editor_core::game_stage::GameStage;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_script::{LuauScript, ProjectScripts};
use std::sync::OnceLock;

pub const CENAS: u32 = 1;

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
    Para,
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

/// O horário: joga ao quadro 60, corre para a direita (apanha as duas primeiras moedas) e pára ao 420.
#[must_use]
pub fn gesto_no_quadro(stage: GameStage, quadro: u32) -> Option<Gesto> {
    match quadro {
        60 => Some(Gesto::Joga(stage)),
        120 => Some(Gesto::Direita(true)),
        240 => Some(Gesto::Direita(false)),
        420 => Some(Gesto::Para),
        _ => None,
    }
}

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
pub fn montar(world: &mut ph2d_ecs::World, nivel: u32) -> ph2d_ecs::Entity {
    let _ = nivel.clamp(1, CENAS);
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

#[cfg(test)]
#[path = "smoke_tests.rs"]
mod tests;
