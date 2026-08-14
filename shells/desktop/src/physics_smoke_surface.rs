//! **A cena 115 — A SUPERFÍCIE FALA** (`W-Surface`).
//!
//! Quatro raias IDÊNTICAS, quatro personagens IDÊNTICOS, e a única diferença
//! entre elas é de que o CHÃO é feito. Da esquerda para a direita: **gelo**
//! (`grip 0,15`), o chão de sempre (**sem componente**, o controle), **borracha**
//! (`grip 4`) e uma **esteira** (`belt 3`).
//!
//! # ⚠️ Uma raia é o CONTROLE, e sem ela a cena não diz nada
//!
//! A do meio não carrega `WalkSurface` nenhum — é o mundo de antes desta wave,
//! byte a byte. Sem ela as outras três seriam três números sem régua.
//!
//! # ⚠️ O gelo tem CONSEQUÊNCIA, e ela vale para os DOIS sentidos
//!
//! O `grip` multiplica arrancar **e** parar, que é o que faz do gelo gelo — e é
//! a razão de esta cena ter um POÇO no fim: quem não pára, cai. A mesma raia
//! mostra a outra metade sem um passo a mais, porque o artista sente a demora a
//! arrancar antes de chegar à marca.
//!
//! # ⚠️ E a esteira leva por TRAÇÃO — a propriedade EMERGENTE
//!
//! Nada no código diz que uma correia sem `grip` não leva nada; cai da
//! composição. O roteiro faz o artista **autorar isso pelo Inspector**, que é
//! como a quarta condição de UI do módulo é mostrada a uma pessoa em vez de só
//! gateada.
//!
//! # ⚠️ A aceleração desta cena é BAIXA, a mesma nota da irmã 114
//!
//! Com o perfil de partida (`accel = 60`) o arranque inteiro cabe em poucos
//! tiques mesmo com um quarto da tração, e a diferença some. **Não é um default
//! de produto** — é o que torna a wave mensurável a olho.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::{PlatformPlayer, WalkSurface};

use crate::App;
use crate::physics_smoke_brake::{DECK_END, DECK_START, DECK_TOP, FLOAT, MARK_X, PIT_TOP};
use crate::physics_smoke_player::{slab, spawn_player};

/// A velocidade de cruzeiro desta cena, m/s — a da irmã 114.
pub(crate) const RUN_SPEED: f32 = 8.0;
/// A aceleração desta cena, m/s² — baixa de propósito, ver o topo do módulo.
pub(crate) const RUN_ACCEL: f32 = 8.0;

/// A distância entre raias — a da irmã, que já foi medida contra o POÇO e não
/// contra o deck (o poço avança 8 m além de `DECK_END`).
pub(crate) const LANE_SPAN: f32 = 32.0;

/// As quatro superfícies da cena.
///
/// ⚠️ **O controle carrega `None`, não um `WalkSurface` neutro** — a raia do
/// meio tem de ser o mundo SEM o componente, senão ela deixaria de ser controle
/// e passaria a testar que o neutro é neutro (o que é outro gate, e já existe).
pub(crate) const SURFACES: [(Option<WalkSurface>, &str); 4] = [
    (
        Some(WalkSurface {
            grip: 0.15,
            belt: 0.0,
        }),
        "Ice",
    ),
    (None, "Normal"),
    (
        Some(WalkSurface {
            grip: 4.0,
            belt: 0.0,
        }),
        "Rubber",
    ),
    (
        Some(WalkSurface {
            grip: 1.0,
            belt: 3.0,
        }),
        "Belt",
    ),
];

/// Onde a raia `i` começa.
#[must_use]
pub(crate) fn lane_x(i: usize) -> f32 {
    i as f32 * LANE_SPAN
}

/// Um bloco de `x0` a `x1` com o topo em `top`, nascendo de `floor`.
fn block(
    world: &mut bevy_ecs::world::World,
    name: &str,
    x0: f32,
    x1: f32,
    top: f32,
    floor: f32,
    tint: [f32; 4],
) -> Entity {
    let half_w = (x1 - x0) * 0.5;
    let half_h = (top - floor) * 0.5;
    slab(
        world,
        name,
        Vec2::new(x0 + half_w, top - half_h),
        [half_w, half_h],
        0.0,
        tint,
    )
}

/// Uma raia: o poço, a plataforma (com a superfície), a marca — e o personagem.
fn lane(
    world: &mut bevy_ecs::world::World,
    x0: f32,
    tag: &str,
    surface: Option<WalkSurface>,
) -> (Entity, Entity) {
    // ⚠️ **O poço é PRIMEIRO**, para a plataforma ficar desenhada por cima dele.
    block(
        world,
        &format!("{tag} Pit"),
        x0 + DECK_END,
        x0 + DECK_END + 8.0,
        PIT_TOP,
        PIT_TOP - 1.0,
        [0.20, 0.18, 0.22, 1.0],
    );
    // A cor diz de que o chão é feito, para o olho não ter de decorar a ordem.
    let tint = match tag {
        "Ice" => [0.55, 0.75, 0.92, 1.0],
        "Rubber" => [0.42, 0.30, 0.30, 1.0],
        "Belt" => [0.28, 0.42, 0.40, 1.0],
        _ => [0.35, 0.35, 0.4, 1.0],
    };
    // ⚠️ A superfície vive no DECK — a face que o pé encontra — e o `block`
    // devolve a entidade que acabou de criar. Re-descobri-la por NOME seria uma
    // segunda resposta a *"qual destes blocos é o chão?"*.
    let deck = block(
        world,
        &format!("{tag} Deck"),
        x0 + DECK_START,
        x0 + DECK_END,
        DECK_TOP,
        PIT_TOP,
        tint,
    );
    if let Some(s) = surface {
        world.entity_mut(deck).insert(s);
    }
    // A MARCA — uma faixa fina embutida no topo, para o olho ter de onde medir.
    // ⚠️ Ela é DESENHO e não colisão: um degrau ali mudaria o que a cena mede.
    block(
        world,
        &format!("{tag} Mark"),
        x0 + MARK_X - 0.06,
        x0 + MARK_X + 0.06,
        DECK_TOP + 0.02,
        DECK_TOP - 0.4,
        [0.95, 0.75, 0.25, 1.0],
    );

    // ⚠️ Os quatro nascem pela MESMA porta, então nada da geometria do corpo
    // pode divergir entre eles. Só o NOME e os dois números de caminhada são
    // escritos por cima — a superfície não é do personagem, é do CHÃO.
    let p = spawn_player(world, Vec2::new(x0 + 1.0, DECK_TOP + FLOAT));
    world.entity_mut(p).insert(Name::new(tag.to_string()));
    {
        let mut e = world.entity_mut(p);
        let mut cfg = e.get_mut::<PlatformPlayer>().expect("player");
        cfg.speed = RUN_SPEED;
        cfg.acceleration = RUN_ACCEL;
    }
    (deck, p)
}

impl App {
    /// **A superfície fala** — de que o chão é feito.
    pub(crate) fn physics_smoke_surface(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_surface_scene(gfx.sim.world_mut());
        eprintln!("{SURFACE_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 115**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
pub(crate) fn build_surface_scene(world: &mut bevy_ecs::world::World) -> Vec<(Entity, Entity)> {
    SURFACES
        .iter()
        .enumerate()
        .map(|(i, (surface, tag))| lane(world, lane_x(i), tag, *surface))
        .collect()
}

/// O roteiro da cena 115 — ⚠️ **os números saem da sonda**
/// (`measure_the_scene_surface`), e não do olho.
pub(crate) const SURFACE_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 115] A SUPERFICIE FALA (W-Surface). Quatro raias iguais,\n",
    "quatro personagens iguais -- e o teclado dirige os QUATRO ao mesmo tempo.\n",
    "So' o CHAO difere. ESQUERDA gelo (azul) - MEIO nada (o controle, cinza) -\n",
    "depois borracha (castanho) - DIREITA uma esteira (verde).\n",
    "A faixa AMBAR no chao (x = 12.00 na raia) e' de onde medir.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play. NAO toque em nada: o da\n",
    "    DIREITA anda sozinho -- a esteira o leva 3.00 m em 1 s. Os outros tres\n",
    "    ficam onde estao. (A seta VERDE-AGUA no chao dela e' para que lado ela\n",
    "    corre; e' a unica forma de ler isso com a cena parada.)\n",
    " 2. Reset. Corra para a DIREITA e conte quanto cada um demora a arrancar.\n",
    "    Medido em 1 s: gelo 0.89 m - controle 5.05 - borracha 7.32. O gelo\n",
    "    custa a sair do lugar. (Se os tres sairem juntos, PARE.)\n",
    " 3. Corra ate' passar da faixa ambar e SOLTE a seta. Medido a partir da\n",
    "    marca: o controle derrapa 2.95 m, a borracha 0.87, e o GELO 8.27 -- ele\n",
    "    nao para e CAI no poco. (Cair e' o que grip 0.15 promete nos dois\n",
    "    sentidos: quem nao arranca tambem nao para. Se ele parasse, PARE.)\n",
    "    ⚠️ A ESTEIRA cai tambem, e por OUTRO motivo: ela continua a levar\n",
    "    depois de voce soltar (10.62 m). Largar o direcional nao desliga uma\n",
    "    correia -- as duas quedas nao sao a mesma coisa.\n",
    " 4. A ESTEIRA, de pe: volte para a raia da direita e fique parado sobre ela.\n",
    "    Ela leva. Ande CONTRA ela e voce anda mais devagar; a favor, mais\n",
    "    depressa -- a lei mede tudo relativo ao chao.\n",
    " 5. OS AJUSTES (e a parte que so' o Inspector mostra): selecione o DECK da\n",
    "    esteira -- o bloco, nao o personagem -- e no card do corpo fisico veja\n",
    "    'Grip' e 'Belt (m/s)' ja' com os valores autorados (1 e 3). Baixe o\n",
    "    Grip para 0: a esteira deixa de levar QUALQUER COISA, e a seta continua\n",
    "    la'. Uma correia leva por ATRITO -- sem tracao ela nao tem por onde\n",
    "    puxar. (Se ela continuar a levar, PARE.)\n",
    " 6. Ponha o Belt em -3: a seta inverte e ela leva para o outro lado. Ponha\n",
    "    Grip 1 e Belt 0 nos dois: a row DETACHA o componente e a raia volta a\n",
    "    ser um chao comum -- identica a' do meio.\n",
    " 7. E o CONTROLE: selecione o deck do MEIO. Ele nao carrega superficie\n",
    "    nenhuma, e as duas rows mostram o neutro (1 e 0).\n",
    "\n",
    "O QUE ISTO ACRESCENTA: ate' esta wave todo chao era o mesmo chao. Gelo,\n",
    "borracha e esteira eram inexprimiveis -- e uma esteira so' existiria se a\n",
    "plataforma de facto andasse, que ninguem constroi assim.\n",
    "\n",
    "⚠️ So' o PLAYER le' a superficie: um caixote sobre a esteira nao e' levado\n",
    "por ela. E' uma limitacao NOMEADA, nao um descuido -- a amostra de chao e' o\n",
    "canal do personagem.\n",
);

#[cfg(test)]
#[path = "physics_smoke_surface_tests.rs"]
mod tests;
