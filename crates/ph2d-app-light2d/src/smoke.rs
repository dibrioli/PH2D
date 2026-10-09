//! **As cenas de smoke da LUZ 2D** — `PH2D_LIGHT2D_SMOKE=<n>` (plano `docs/Luz2D/01_plano.md` §6).
//!
//! | n | o que ensina |
//! |---|---|
//! | 1 | **Uma luz** — uma sala escura (luz ambiente fraca), uma lâmpada à esquerda, uma parede que BLOQUEIA a luz e três quadrados coloridos: o Vermelho (à frente) e o Azul (rente à borda da sombra) acendem, o Verde (atrás da parede) fica na sombra, e a sombra alarga longe da parede |
//!
//! ⚠️ A W3 deixa só a cena 1, para o roteador ser ALCANÇÁVEL (o gate das famílias); o roteiro ao dono,
//! a fotografia e as cenas 2–4 são da W4.

use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, Name, Transform};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, TipoDeLuz};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O roteador desta família — o maior nível que [`encenar`] responde.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_LIGHT2D_SMOKE",
    max_level: 1,
}];

/// O nível armado por variável de ambiente (`None` = inerte).
#[must_use]
pub fn nivel_armado() -> Option<u32> {
    std::env::var("PH2D_LIGHT2D_SMOKE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
}

/// Povoa `world` com a cena `nivel`. Devolve se havia cena para esse nível.
pub fn encenar(world: &mut World, nivel: u32) -> bool {
    match nivel {
        1 => {
            uma_luz(world);
            true
        }
        _ => {
            eprintln!("[luz2d] PH2D_LIGHT2D_SMOKE={nivel}: não há esta cena (1..=1)");
            false
        }
    }
}

fn pose(x: f32, y: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    (t, GlobalTransform::from_transform(t))
}

fn quadrado(
    world: &mut World,
    nome: &str,
    centro: [f32; 2],
    lado: [f32; 2],
    cor: [f32; 4],
) -> bevy_ecs::entity::Entity {
    let (t, g) = pose(centro[0], centro[1]);
    world
        .spawn((
            Name::new(nome),
            Sprite::atlas(WHITE_TILE_KEY, lado, cor),
            t,
            g,
        ))
        .id()
}

/// Cena 1 — **Uma luz.** Unidades do mundo em metros; a vista do editor começa centrada na origem.
fn uma_luz(world: &mut World) {
    // O chão da sala (recebe a luz) e a escuridão ambiente: `I = 2π·0,06` ⇒ 6 % do forno branco.
    quadrado(
        world,
        "Chão",
        [0.0, 0.0],
        [9.0, 5.0],
        [0.85, 0.85, 0.85, 1.0],
    );
    let (t, g) = pose(0.0, 0.0);
    world.spawn((
        Name::new("Luz ambiente"),
        Luz {
            cor: [0.6, 0.7, 1.0],
            intensidade: std::f32::consts::TAU * 0.06,
            tamanho_m: 0.0,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ambiente,
        },
        t,
        g,
    ));
    // A lâmpada: `I = 2π·1,5` ⇒ branco a 1,5 m (plano §2.3).
    let (t, g) = pose(-3.0, 0.4);
    world.spawn((
        Name::new("Lâmpada"),
        Luz {
            cor: [1.0, 0.9, 0.7],
            intensidade: std::f32::consts::TAU * 1.5,
            tamanho_m: 0.25,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    let parede = quadrado(
        world,
        "Parede",
        [-0.8, 0.6],
        [0.3, 1.6],
        [0.35, 0.3, 0.3, 1.0],
    );
    world
        .entity_mut(parede)
        .insert(BloqueiaLuz { opacidade: 1.0 });
    quadrado(
        world,
        "Vermelho",
        [-1.8, -1.4],
        [0.6, 0.6],
        [0.9, 0.2, 0.2, 1.0],
    );
    quadrado(world, "Verde", [1.4, 0.3], [0.6, 0.6], [0.2, 0.8, 0.3, 1.0]);
    quadrado(
        world,
        "Azul",
        [2.6, -1.3],
        [0.6, 0.6],
        [0.25, 0.4, 0.95, 1.0],
    );
}
