//! **As cenas de smoke da LUZ 2D** — `PH2D_LIGHT2D_SMOKE=<n>` (plano `docs/Luz2D/01_plano.md` §6).
//!
//! | n | o que ensina |
//! |---|---|
//! | 1 | **Uma luz** — uma sala escura (luz ambiente fraca), uma lâmpada à esquerda, uma parede que BLOQUEIA a luz e três quadrados coloridos: o Vermelho (à frente) e o Azul (rente à borda da sombra) acendem, o Verde (atrás da parede) fica na sombra, e a sombra alarga longe da parede |
//! | 2 | **Vaga-lumes** — 1 000 sprites que BRILHAM, em quatro enxames nos cantos de um chão escuro, cada um com uma árvore ao lado: cada enxame acende o chão em volta e a árvore faz sombra; o custo da luz é o de uma lâmpada só (a sonda mede 1 × 1 000) |
//! | 3 | **Rebote** (W5c) — uma sala fechada: chão e teto brancos, parede esquerda VERMELHA, direita VERDE, uma lâmpada branca perto do teto, um fundo que só RECEBE luz e uma caixa branca no chão: a luz que bate nas paredes volta tingida — o fundo avermelha junto à vermelha e esverdeia junto à verde, e cada face da caixa toma a cor da parede que a olha (`PH2D_LIGHT2D_REBOTE=0` desliga o rebote, para comparar) |
//! | 4 | **Tamanho da lâmpada** — duas salas iguais separadas por um muro: à esquerda uma lâmpada PEQUENA, à direita uma GRANDE, cada uma atrás da mesma parede: a pequena faz sombra nítida, a grande macia (a sonda mede a penumbra × a geometria) |
//!
//! Os vaga-lumes estão PARADOS: o que a cena ensina é a luz (e o custo), não o voo.

use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, Name, Transform};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, NaoRecebeLuz, TipoDeLuz};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// ⭐ O que faz de um quadrado uma PAREDE: bloqueia a luz e NÃO a recebe. ⚠️ Medido (W4): o interior
/// de um corpo opaco recebe zero (a lei da W1, no gabarito e na placa) — uma parede que recebesse a
/// luz sairia PRETA. «Não recebe luz» é o que o artista faz hoje; a face acesa é da W5.
const PAREDE: (BloqueiaLuz, NaoRecebeLuz) = (BloqueiaLuz { opacidade: 1.0 }, NaoRecebeLuz);

/// O roteador desta família — o maior nível que [`encenar`] responde.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_LIGHT2D_SMOKE",
    max_level: 4,
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
        2 => {
            vaga_lumes(world, VAGA_LUMES);
            true
        }
        3 => {
            rebote(world);
            true
        }
        4 => {
            tamanho_da_lampada(world);
            true
        }
        _ => {
            eprintln!("[luz2d] PH2D_LIGHT2D_SMOKE={nivel}: não há esta cena (1–4)");
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
    world.entity_mut(parede).insert(PAREDE);
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

/// Quantos vaga-lumes a cena 2 tem (plano §6).
pub const VAGA_LUMES: usize = 1000;
/// Quantos enxames — os vaga-lumes repartem-se igualmente. ⚠️ QUATRO, nos cantos, e é medido: no
/// plano a luz cai só com `1/r`, e com dez enxames espalhados o chão inteiro acendia por igual
/// (`luz_em` 1,5–1,7 em todo lado) — nem poça nem sombra se viam.
pub const ENXAMES: usize = 4;
/// O raio de um enxame (metros).
pub const RAIO_DO_ENXAME_M: f32 = 1.1;
/// O brilho de cada vaga-lume (`SpriteEmissive`): radiância `cor·k`.
pub const BRILHO_DO_VAGA_LUME: f32 = 1.0;

/// Os centros dos enxames (cada árvore 1,6 m à direita do seu enxame). ⚠️ A cena cabe na vista de
/// fábrica do editor a 1920×1080 (≈ 12 × 9 m, medido na foto): fora dela os enxames não se viam.
pub const CENTROS_DOS_ENXAMES: [[f32; ENXAMES]; 2] =
    [[-4.3, 3.0, -4.3, 3.0], [2.2, 2.2, -2.2, -2.2]];

/// Cena 2 — **Vaga-lumes.** `n` vaga-lumes em [`ENXAMES`] enxames (posições por um gerador
/// determinístico), um chão escuro, uma árvore à direita de cada enxame e a ambiente bem baixa.
pub fn vaga_lumes(world: &mut World, n: usize) {
    quadrado(
        world,
        "Chão",
        [0.0, 0.0],
        [11.5, 8.5],
        [0.45, 0.5, 0.42, 1.0],
    );
    ambiente(world, 0.03, [0.5, 0.6, 1.0]);
    let mut semente: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut aleatorio = move || {
        semente = semente
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((semente >> 40) as f32) / ((1u64 << 24) as f32)
    };
    for (k, (&cx, &cy)) in CENTROS_DOS_ENXAMES[0]
        .iter()
        .zip(&CENTROS_DOS_ENXAMES[1])
        .enumerate()
    {
        let arvore = quadrado(
            world,
            &format!("Árvore {}", k + 1),
            [cx + 1.6, cy],
            // Mais alta que o enxame (2,2 m): atrás dela há sombra inteira, não só meia.
            [0.35, 2.6],
            [0.18, 0.14, 0.1, 1.0],
        );
        world.entity_mut(arvore).insert(PAREDE);
    }
    for i in 0..n {
        let k = i % ENXAMES;
        let (cx, cy) = (CENTROS_DOS_ENXAMES[0][k], CENTROS_DOS_ENXAMES[1][k]);
        // Um ponto uniforme no disco do enxame.
        let (r, a) = (
            RAIO_DO_ENXAME_M * aleatorio().sqrt(),
            std::f32::consts::TAU * aleatorio(),
        );
        let e = quadrado(
            world,
            &format!("Vaga-lume {}", i + 1),
            [cx + r * a.cos(), cy + r * a.sin()],
            [0.1, 0.1],
            [1.0, 0.9, 0.35, 1.0],
        );
        world
            .entity_mut(e)
            .insert(ph2d_ecs::SpriteEmissive(BRILHO_DO_VAGA_LUME));
    }
}

/// Cena 3 — o interior da sala (metros): `x ∈ [−3, 3]`, `y ∈ [−1,8, 1,8]`; as paredes têm 0,3 m.
pub const SALA_DO_REBOTE: [f32; 2] = [6.0, 3.6];
/// A espessura das paredes da sala do rebote (metros).
pub const PAREDE_DO_REBOTE_M: f32 = 0.3;

/// Cena 3 — **Rebote** (W5c, a lei do plano §2.4). As paredes BLOQUEIAM e RECEBEM a luz — é isso que
/// as faz devolvê-la, × a cor delas.
pub fn rebote(world: &mut World) {
    let [w, h] = SALA_DO_REBOTE;
    let e = PAREDE_DO_REBOTE_M;
    // O fundo da sala: só recebe a luz (não bloqueia) — é nele que se vê a cor das paredes a escorrer.
    quadrado(world, "Fundo", [0.0, 0.0], [w, h], [0.8, 0.8, 0.8, 1.0]);
    ambiente(world, 0.01, [0.6, 0.7, 1.0]);
    let (t, g) = pose(0.0, 0.5 * h - 0.6);
    world.spawn((
        Name::new("Lâmpada"),
        Luz {
            cor: [1.0, 1.0, 1.0],
            intensidade: std::f32::consts::TAU * 1.2,
            tamanho_m: 0.15,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    let opaca = BloqueiaLuz { opacidade: 1.0 };
    for (nome, centro, lado, cor) in [
        (
            "Chão",
            [0.0, -0.5 * (h + e)],
            [w + 2.0 * e, e],
            [0.95, 0.95, 0.95, 1.0],
        ),
        (
            "Teto",
            [0.0, 0.5 * (h + e)],
            [w + 2.0 * e, e],
            [0.95, 0.95, 0.95, 1.0],
        ),
        (
            "Parede vermelha",
            [-0.5 * (w + e), 0.0],
            [e, h],
            [0.9, 0.1, 0.1, 1.0],
        ),
        (
            "Parede verde",
            [0.5 * (w + e), 0.0],
            [e, h],
            [0.1, 0.8, 0.15, 1.0],
        ),
        (
            "Caixa branca",
            [0.0, -0.5 * h + 0.5],
            [0.8, 1.0],
            [0.95, 0.95, 0.95, 1.0],
        ),
    ] {
        let q = quadrado(world, nome, centro, lado, cor);
        world.entity_mut(q).insert(opaca);
    }
}

/// Cena 4 — os dois raios de lâmpada (metros).
pub const LAMPADA_PEQUENA_M: f32 = 0.15;
/// Ver [`LAMPADA_PEQUENA_M`].
pub const LAMPADA_GRANDE_M: f32 = 0.6;
/// O centro de cada sala (`x`): a pequena à esquerda, a grande à direita.
pub const SALAS_X: [f32; 2] = [-2.9, 2.9];
/// Na sala de centro `cx`: a lâmpada, o canto de cima-direita da parede (o que decide a borda da
/// sombra) e o `x` da linha onde a sonda mede a penumbra.
#[must_use]
pub fn geometria_da_sala(cx: f32) -> ([f32; 2], [f32; 2], f32) {
    ([cx - 1.8, 1.5], [cx - 0.3, 0.5], cx + 1.4)
}

/// Cena 4 — **Tamanho da lâmpada.** Duas salas iguais separadas por um muro; em cada uma, uma
/// lâmpada (pequena à esquerda, grande à direita, o MESMO brilho a 1 m) e a mesma parede.
pub fn tamanho_da_lampada(world: &mut World) {
    quadrado(
        world,
        "Chão",
        [0.0, 0.0],
        [11.5, 8.5],
        [0.85, 0.85, 0.85, 1.0],
    );
    ambiente(world, 0.02, [0.6, 0.7, 1.0]);
    let muro = quadrado(world, "Muro", [0.0, 0.0], [0.2, 8.5], [0.3, 0.3, 0.32, 1.0]);
    world.entity_mut(muro).insert(PAREDE);
    for (cx, (nome, raio)) in SALAS_X.into_iter().zip([
        ("Lâmpada pequena", LAMPADA_PEQUENA_M),
        ("Lâmpada grande", LAMPADA_GRANDE_M),
    ]) {
        let (lampada, canto, _) = geometria_da_sala(cx);
        let (t, g) = pose(lampada[0], lampada[1]);
        world.spawn((
            Name::new(nome),
            Luz {
                cor: [1.0, 0.95, 0.85],
                intensidade: std::f32::consts::TAU * 2.0,
                tamanho_m: raio,
                alcance_m: f32::INFINITY,
                tipo: TipoDeLuz::Ponto,
            },
            t,
            g,
        ));
        // A parede: 0,2 × 3 m, o canto de cima-direita em `canto`.
        let parede = quadrado(
            world,
            if cx < 0.0 {
                "Parede da pequena"
            } else {
                "Parede da grande"
            },
            [canto[0] - 0.1, canto[1] - 1.5],
            [0.2, 3.0],
            [0.35, 0.3, 0.3, 1.0],
        );
        world.entity_mut(parede).insert(PAREDE);
    }
}

/// A luz ambiente de uma cena: `fracao` do forno branco, na cor dada.
fn ambiente(world: &mut World, fracao: f32, cor: [f32; 3]) {
    let (t, g) = pose(0.0, 0.0);
    world.spawn((
        Name::new("Luz ambiente"),
        Luz {
            cor,
            intensidade: std::f32::consts::TAU * fracao,
            ..Luz::ambiente()
        },
        t,
        g,
    ));
}
