//! **As cenas de smoke da LUZ 2D** — `PH2D_LIGHT2D_SMOKE=<n>` (plano `docs/Luz2D/01_plano.md` §6).
//!
//! | n | o que ensina |
//! |---|---|
//! | 1 | **Uma luz** — uma sala escura (luz ambiente fraca), uma lâmpada à esquerda, uma parede que BLOQUEIA a luz e três quadrados coloridos: o Vermelho (à frente) e o Azul (rente à borda da sombra) acendem, o Verde (atrás da parede) fica na sombra, e a sombra alarga longe da parede |
//! | 2 | **Vaga-lumes** — 1 000 sprites que BRILHAM, em quatro enxames nos cantos de um chão escuro, cada um com uma árvore ao lado: cada enxame acende o chão em volta e a árvore faz sombra; o custo da luz é o de uma lâmpada só (a sonda mede 1 × 1 000) |
//! | 3 | **Rebote** (W5c) — uma sala fechada: chão e teto claros, parede esquerda VERMELHA, direita VERDE, uma lâmpada branca perto do teto e um fundo que só RECEBE luz: a luz que bate nas paredes volta tingida — a sala clareia, o fundo avermelha junto à vermelha e esverdeia junto à verde (`PH2D_LIGHT2D_REBOTE=0` desliga o rebote, para comparar) |
//! | 4 | **Tamanho da lâmpada** — duas salas iguais separadas por um muro: à esquerda uma lâmpada PEQUENA, à direita uma GRANDE, cada uma atrás da mesma parede: a pequena faz sombra nítida, a grande macia (a sonda mede a penumbra × a geometria) |
//! | 5 | **Volume** — uma lâmpada no meio de um chão e seis corpos à volta que bloqueiam E recebem a luz (disco, elipse, losango, barra inclinada, «L» e cruz): o interior de cada um mostra a luz do exterior mais próximo, logo cada lado vira uma FACE — acesa a virada para a lâmpada, escura a de trás — e as faces encontram-se em arestas (o disco vira um cone que aponta a luz; o «L» e a cruz dobram as faces no canto, porque a lei olha a borda do CONJUNTO) |
//! | 6 | **Formas desenhadas** (W5d) — uma lâmpada no meio, um CÍRCULO e uma ELIPSE desenhados (vetor, no documento do editor) em cima e os mesmos dois como IMAGENS embaixo, com as mesmas cores: o desenho bloqueia e recebe a luz como a imagem — a face virada para a lâmpada acesa, a de trás escura, a sombra atrás de cada um |
//!
//! Os vaga-lumes estão PARADOS: o que a cena ensina é a luz (e o custo), não o voo.

use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, Name, Transform};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, NaoRecebeLuz, TipoDeLuz};
use ph2d_render::{DISC_TILE_KEY, Sprite, WHITE_TILE_KEY};

/// ⭐ O que faz de um quadrado uma PAREDE: bloqueia a luz e NÃO a recebe. ⚠️ Medido (W4): o interior
/// de um corpo opaco recebe zero (a lei da W1, no gabarito e na placa) — uma parede que recebesse a
/// luz sairia PRETA. «Não recebe luz» é o que o artista faz hoje; a face acesa é da W5.
const PAREDE: (BloqueiaLuz, NaoRecebeLuz) = (BloqueiaLuz { opacidade: 1.0 }, NaoRecebeLuz);

/// ⭐ W5e — a cena 7, o traço do Flip (ficheiro irmão).
#[path = "smoke_flip.rs"]
mod flip;
pub use flip::{
    COR_DA_ONDA, DISCOS_DE_TINTA, DiscoDeTinta, NOME_DO_TRACO, marcar_flip, onda_a_mao, semear_flip,
};
/// ⭐ W5e — a cena 8, as formas do Motion (ficheiro irmão).
#[path = "smoke_motion.rs"]
mod motion;
pub use motion::{LAMPADA_DAS_FORMAS, marcar_motion};

/// O roteador desta família — o maior nível que [`encenar`] responde.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_LIGHT2D_SMOKE",
    max_level: 8,
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
        5 => {
            volume(world);
            true
        }
        6 => {
            redondas(world);
            true
        }
        7 => {
            flip::traco_do_flip(world);
            true
        }
        8 => {
            motion::formas_do_motion(world);
            true
        }
        _ => {
            eprintln!("[luz2d] PH2D_LIGHT2D_SMOKE={nivel}: não há esta cena (1–8)");
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

/// Cena 3 — o interior da sala (metros): `x ∈ [−3, 3]`, `y ∈ [−1,8, 1,8]`.
pub const SALA_DO_REBOTE: [f32; 2] = [6.0, 3.6];
/// A espessura das paredes da sala do rebote (metros).
pub const PAREDE_DO_REBOTE_M: f32 = 0.2;

/// Cena 3 — **Rebote** (W5c, a lei do plano §2.4). As paredes BLOQUEIAM e RECEBEM a luz — é isso que
/// as faz devolvê-la, × a cor delas. ⚠️ Medido na foto: com albedos de 0,9 a sala fechada multiplica
/// a luz ~8× e satura a branco (a cor some); com os de uma sala real (≈ 0,75) e a lâmpada fraca o
/// rebote clareia e tinge sem estourar. Sem objeto no meio: o interior de um opaco que recebe luz
/// mostra a face mais próxima, cortada no eixo medial (ABERTO 2 da W5a) — uma caixa lia-se como
/// cunhas pretas.
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
            intensidade: std::f32::consts::TAU * 0.35,
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
            [0.75, 0.75, 0.75, 1.0],
        ),
        (
            "Teto",
            [0.0, 0.5 * (h + e)],
            [w + 2.0 * e, e],
            [0.75, 0.75, 0.75, 1.0],
        ),
        (
            "Parede vermelha",
            [-0.5 * (w + e), 0.0],
            [e, h],
            [0.8, 0.08, 0.08, 1.0],
        ),
        (
            "Parede verde",
            [0.5 * (w + e), 0.0],
            [e, h],
            [0.08, 0.7, 0.12, 1.0],
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

/// Cena 5 — a lâmpada (no centro do chão).
pub const LAMPADA_DO_VOLUME: [f32; 2] = [0.0, 0.0];

/// Cena 5 — um corpo: a imagem (o quadrado ou o disco do atlas), o centro e o lado (metros), a
/// rotação (radianos) e a cor.
pub struct Corpo {
    pub nome: &'static str,
    pub imagem: u32,
    pub centro: [f32; 2],
    pub lado: [f32; 2],
    pub rotacao: f32,
    pub cor: [f32; 3],
}

const fn corpo(
    nome: &'static str,
    imagem: u32,
    centro: [f32; 2],
    lado: [f32; 2],
    rotacao: f32,
    cor: [f32; 3],
) -> Corpo {
    Corpo {
        nome,
        imagem,
        centro,
        lado,
        rotacao,
        cor,
    }
}

/// Cena 5 — os corpos. O «L» e a cruz são dois retângulos sobrepostos cada (a lei do interior olha a
/// borda do CONJUNTO).
pub const CORPOS_DO_VOLUME: [Corpo; 8] = [
    corpo(
        "Disco",
        DISC_TILE_KEY,
        [-3.0, 1.7],
        [1.4, 1.4],
        0.0,
        [0.85, 0.25, 0.2],
    ),
    corpo(
        "Elipse",
        DISC_TILE_KEY,
        [0.0, 2.6],
        [2.6, 1.1],
        0.0,
        [0.9, 0.75, 0.2],
    ),
    corpo(
        "Losango",
        WHITE_TILE_KEY,
        [3.0, 1.7],
        [1.2, 1.2],
        std::f32::consts::FRAC_PI_4,
        [0.25, 0.45, 0.9],
    ),
    corpo(
        "Barra",
        WHITE_TILE_KEY,
        [3.1, -1.7],
        [2.4, 0.55],
        0.5,
        [0.2, 0.75, 0.35],
    ),
    corpo(
        "L — base",
        WHITE_TILE_KEY,
        [-2.8, -2.3],
        [1.8, 0.6],
        0.0,
        [0.75, 0.4, 0.85],
    ),
    corpo(
        "L — haste",
        WHITE_TILE_KEY,
        [-3.4, -1.6],
        [0.6, 2.0],
        0.0,
        [0.75, 0.4, 0.85],
    ),
    corpo(
        "Cruz — braço",
        WHITE_TILE_KEY,
        [0.0, -2.6],
        [2.0, 0.55],
        0.0,
        [0.9, 0.9, 0.9],
    ),
    corpo(
        "Cruz — tronco",
        WHITE_TILE_KEY,
        [0.0, -2.6],
        [0.55, 2.0],
        0.0,
        [0.9, 0.9, 0.9],
    ),
];

/// Cena 5 — **Volume.** Os corpos RECEBEM a luz (sem «não recebe luz»): o interior de cada um é o
/// exterior mais próximo, e cada lado vira uma face com a luz que bate nele.
pub fn volume(world: &mut World) {
    quadrado(
        world,
        "Chão",
        [0.0, 0.0],
        [11.5, 8.5],
        [0.6, 0.6, 0.62, 1.0],
    );
    ambiente(world, 0.03, [0.6, 0.7, 1.0]);
    let (t, g) = pose(LAMPADA_DO_VOLUME[0], LAMPADA_DO_VOLUME[1]);
    world.spawn((
        Name::new("Lâmpada"),
        Luz {
            cor: [1.0, 0.92, 0.8],
            intensidade: std::f32::consts::TAU * 1.2,
            tamanho_m: 0.2,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    for c in CORPOS_DO_VOLUME {
        let mut t = Transform::IDENTITY;
        t.translation.x = c.centro[0];
        t.translation.y = c.centro[1];
        t.rotation = c.rotacao;
        world.spawn((
            Name::new(c.nome),
            Sprite::atlas(c.imagem, c.lado, [c.cor[0], c.cor[1], c.cor[2], 1.0]),
            t,
            GlobalTransform::from_transform(t),
            BloqueiaLuz { opacidade: 1.0 },
        ));
    }
}

/// Cena 6 — uma forma redonda: o nome, o centro e os semieixos (metros), a cor (sRGB, a do
/// documento vetorial; a imagem gêmea leva a mesma em linear).
pub struct Redonda {
    pub nome: &'static str,
    pub centro: [f64; 2],
    pub semi: [f64; 2],
    pub cor: [u8; 3],
}

/// Cena 6 — as formas DESENHADAS (no documento vetorial) e, no espelho de cada uma em relação à
/// lâmpada ([`ABAIXO_DA_IMAGEM`] abaixo), a IMAGEM gêmea: a mesma luz chega às duas.
pub const REDONDAS: [Redonda; 2] = [
    Redonda {
        nome: "Círculo",
        centro: [-2.6, 1.6],
        semi: [0.9, 0.9],
        cor: [51, 179, 191],
    },
    Redonda {
        nome: "Elipse",
        centro: [2.5, 1.6],
        semi: [1.4, 0.65],
        cor: [230, 140, 50],
    },
];

/// Cena 6 — a distância da imagem gêmea abaixo de cada forma desenhada (metros): o dobro da altura
/// delas sobre a lâmpada.
pub const ABAIXO_DA_IMAGEM: f64 = 3.2;

/// sRGB (u8) → linear: a cor que a imagem gêmea leva para sair igual ao desenho.
fn linear(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Cena 6 — **Formas desenhadas.** O mundo: o chão, a luz ambiente, a lâmpada e as IMAGENS gêmeas; as
/// formas desenhadas entram pelo documento ([`semear_documento`]).
fn redondas(world: &mut World) {
    let gemeas: Vec<_> = REDONDAS
        .iter()
        .map(|r| {
            (
                r.nome,
                [r.centro[0], r.centro[1] - ABAIXO_DA_IMAGEM],
                r.semi,
                r.cor,
            )
        })
        .collect();
    chao_lampada_e_gemeas(world, &gemeas);
}

/// Uma imagem gêmea: o nome, o centro, os semi-eixos e a cor sRGB.
type Gemea = (&'static str, [f64; 2], [f64; 2], [u8; 3]);

/// As cenas 6 e 7: o chão no fundo, a luz ambiente, a lâmpada e as IMAGENS gêmeas `(nome, centro,
/// semi-eixos, cor sRGB)` das formas desenhadas (o disco do atlas, «bloqueia a luz»).
fn chao_lampada_e_gemeas(world: &mut World, gemeas: &[Gemea]) {
    let chao = quadrado(
        world,
        "Chão",
        [0.0, 0.0],
        [11.5, 8.5],
        [0.6, 0.6, 0.62, 1.0],
    );
    // O chão no FUNDO: a raiz sem `RootOrder` desenha na frente, e os caminhos que o editor sincroniza
    // depois recebem a ordem seguinte à maior que houver (o chão cobria os desenhos).
    world.entity_mut(chao).insert(ph2d_ecs::RootOrder(0));
    ambiente(world, 0.03, [0.6, 0.7, 1.0]);
    let (t, g) = pose(LAMPADA_DO_VOLUME[0], LAMPADA_DO_VOLUME[1]);
    world.spawn((
        Name::new("Lâmpada"),
        Luz {
            cor: [1.0, 0.92, 0.8],
            intensidade: std::f32::consts::TAU * 1.2,
            tamanho_m: 0.2,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
    for &(nome, centro, semi, cor) in gemeas {
        let (t, g) = pose(centro[0] as f32, centro[1] as f32);
        world.spawn((
            Name::new(format!("{nome} (imagem)")),
            Sprite::atlas(
                DISC_TILE_KEY,
                [2.0 * semi[0] as f32, 2.0 * semi[1] as f32],
                [linear(cor[0]), linear(cor[1]), linear(cor[2]), 1.0],
            ),
            t,
            g,
            BloqueiaLuz { opacidade: 1.0 },
        ));
    }
}

/// ⭐ Cena 6 — põe no DOCUMENTO vetorial as formas desenhadas do nível (só a 6 tem); devolve o
/// caminho e o nome de cada uma. A entidade de cada caminho nasce na sincronização do editor (o
/// `entities::sync`, no fim do quadro): [`marcar_semeadas`] dá-lhe o nome e o «bloqueia a luz».
pub fn semear_documento(
    doc: &mut ph2d_vec_scene::VecScene,
    nivel: u32,
) -> Vec<(ph2d_vec_scene::VecPathId, &'static str)> {
    use ph2d_vec_scene::{Paint, Rgba8, ellipse};
    if nivel != 6 {
        return Vec::new();
    }
    REDONDAS
        .iter()
        .map(|r| {
            let mut p = ellipse(r.centro, r.semi[0], r.semi[1]);
            p.fill = Some(Paint::Solid(Rgba8::new(r.cor[0], r.cor[1], r.cor[2], 255)));
            (doc.push_path(p), r.nome)
        })
        .collect()
}

/// As formas semeadas cuja entidade o editor já criou recebem o nome e o «bloqueia a luz»; devolve as
/// que faltam (as que saíram do documento — um projeto aberto por cima — esquecem-se).
pub fn marcar_semeadas(
    world: &mut World,
    doc: &ph2d_vec_scene::VecScene,
    pendentes: Vec<(ph2d_vec_scene::VecPathId, &'static str)>,
) -> Vec<(ph2d_vec_scene::VecPathId, &'static str)> {
    let mut faltam = Vec::new();
    for (id, nome) in pendentes {
        if doc.path(id).is_none() {
            continue;
        }
        let e = world
            .query::<(bevy_ecs::entity::Entity, &ph2d_ecs::VecPathRef)>()
            .iter(world)
            .find(|(_, r)| r.0 == id)
            .map(|(e, _)| e);
        match e {
            Some(e) => {
                world
                    .entity_mut(e)
                    .insert((Name::new(nome), BloqueiaLuz { opacidade: 1.0 }));
            }
            None => faltam.push((id, nome)),
        }
    }
    faltam
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
