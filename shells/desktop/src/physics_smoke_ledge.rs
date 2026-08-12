//! **A cena 111 — O PARAPEITO** (`W-Ledge`), a beirada.
//!
//! Duas raias IDÊNTICAS, dois personagens IDÊNTICOS, e a única diferença entre
//! eles é um número: o alcance do braço. O da esquerda tem **zero** (o mundo de
//! antes desta wave), o da direita tem **0,60 m**.
//!
//! # ⚠️ O teclado dirige os DOIS ao mesmo tempo, e é isso que faz a cena
//!
//! `hand_input_to_players` entrega a entrada a **todo** `PlatformPlayer` da
//! cena, então um único gesto do artista move os dois lado a lado — o controle
//! está **dentro do quadro**, e não numa segunda corrida.
//!
//! # ⚠️ As alturas são MEDIDAS, e a do meio é a que faz a cena
//!
//! `measure_the_reachable_height` (a sonda da wave anterior), aperto SEGURADO:
//! um pulo alcança **1,903 m**. Daí:
//!
//! | patamar | topo | quem chega |
//! |---|---|---|
//! | o BAIXO | **1,0 m** | os dois, a pular — é o controlo |
//! | o ALTO | **2,4 m** | ninguém, a pular; só quem agarra a beirada |
//!
//! ⚠️ **E o número que de facto decide a cena é OUTRO, medido aqui:** pular
//! **colado à parede** — que é o gesto do passo 3 — alcança **0,745 m**, contra
//! os 1,903 do ar livre. O atrito contra a face come **61%** da subida
//! (`measure_what_a_jump_against_the_wall_reaches`), e o topo do corpo pica em
//! **2,145 m**. A primeira versão desta cena pôs o patamar alto em 2,60 usando o
//! número do ar livre, e o corpo **nunca chegava à janela**.
//!
//! Com o lábio em 2,40 e o braço em 0,60 a janela é `[1,80 · 2,40)`, e o corpo
//! atravessa-a de 1,80 a 2,145 — perto do ápice, onde ele é lento. E 2,40 fica
//! acima de **2,303**, que é onde os PÉS chegam num pulo de ar livre perfeito:
//! ninguém pousa nele sem agarrar.
//!
//! # ⚠️ E os dois patamares têm PAREDE, porque uma beirada é uma quina
//!
//! Um bloco que nasce do chão dá as duas metades de uma vez: a face vertical por
//! onde se sobe e o topo em que se pousa. Um patamar a pairar teria um lábio de
//! cada lado e o gesto deixaria de ser o que a cena quer mostrar.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O topo do patamar BAIXO — ⚠️ **1,0 e não 1,5**, e o número vem do pulo
/// COLADO À PAREDE: os pés dele chegam a **1,145 m** (medido), então um degrau
/// de 1,5 é intransponível para quem chega encostado à face — que é como se
/// chega a um patamar.
pub(crate) const LOW_TOP: f32 = 1.0;
/// O topo do patamar ALTO — acima do que um pulo alcança, e dentro do braço.
pub(crate) const HIGH_TOP: f32 = 2.4;

/// O alcance autorado na raia da direita.
pub(crate) const GRAB: f32 = 0.6;

/// A distância entre as duas raias — larga o bastante para nenhum personagem
/// alcançar a geometria do vizinho.
pub(crate) const LANE_SPAN: f32 = 16.0;

/// Onde a raia da ESQUERDA começa (o personagem sem beirada).
pub(crate) const LANE_A: f32 = 0.0;
/// Onde a raia da DIREITA começa (o personagem com o braço).
pub(crate) const LANE_B: f32 = LANE_A + LANE_SPAN;

/// A altura de flutuação das cenas de player (ver `physics_smoke_player`).
pub(crate) const FLOAT: f32 = 0.9;

/// Um bloco de `x0` a `x1` com o topo em `top` — o chão dele é `y = -0,5`, então
/// ele nasce do solo em vez de pairar.
fn block(world: &mut bevy_ecs::world::World, name: &str, x0: f32, x1: f32, top: f32) {
    let half_w = (x1 - x0) * 0.5;
    let half_h = (top + 0.5) * 0.5;
    slab(
        world,
        name,
        Vec2::new(x0 + half_w, top - half_h),
        [half_w, half_h],
        0.0,
        [0.35, 0.35, 0.4, 1.0],
    );
}

/// Uma raia: chão, o patamar baixo, o alto, e o personagem no começo.
fn lane(world: &mut bevy_ecs::world::World, x0: f32, tag: &str, grab: f32) -> Entity {
    block(world, &format!("{tag} Floor"), x0, x0 + 12.0, 0.0);
    block(world, &format!("{tag} Low"), x0 + 3.0, x0 + 5.0, LOW_TOP);
    block(world, &format!("{tag} High"), x0 + 8.5, x0 + 12.0, HIGH_TOP);

    // ⚠️ Os dois nascem pela MESMA porta (`spawn_player`), então a geometria do
    // corpo — de que a aritmética das alturas depende — não pode divergir entre
    // eles. Só o NOME e o alcance são escritos por cima.
    let p = spawn_player(world, Vec2::new(x0 + 1.0, FLOAT + 0.3));
    world.entity_mut(p).insert(Name::new(tag.to_string()));
    {
        let mut e = world.entity_mut(p);
        let mut cfg = e.get_mut::<PlatformPlayer>().expect("player");
        cfg.ledge_grab = grab;
        // ⚠️ **Os dois DECLARADOS, e não herdados** (`W-LedgeSensor`): a janela
        // igual ao alcance é o mundo de antes da wave, quando um número fazia os
        // dois eixos, e é ela que o roteiro do passo 3 mede. Uma cena que
        // herdasse o default mediria outra janela sem que nada o dissesse.
        cfg.ledge_reach_y = GRAB;
        // ⚠️ **Um raio, como antes** — a extensão é o que o passo 9 liga.
        cfg.ledge_span = 0.0;
    }
    p
}

impl App {
    /// **O parapeito** — a beirada.
    pub(crate) fn physics_smoke_ledge(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_ledge_scene(gfx.sim.world_mut());
        eprintln!("{LEDGE_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 111**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
///
/// Devolve `(o sem beirada, o com braço)`.
pub(crate) fn build_ledge_scene(world: &mut bevy_ecs::world::World) -> (Entity, Entity) {
    let plain = lane(world, LANE_A, "No Ledge", 0.0);
    let ledge = lane(world, LANE_B, "Ledge Grab", GRAB);
    (plain, ledge)
}

/// O roteiro da cena 111.
pub(crate) const LEDGE_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 111] O PARAPEITO (W-Ledge). Duas raias iguais, dois\n",
    "personagens iguais -- e o teclado dirige os DOIS ao mesmo tempo.\n",
    "O da ESQUERDA nao tem beirada; o da DIREITA alcanca 0.60 m.\n",
    "Patamar baixo: topo 1.00 m. Patamar alto: topo 2.40 m.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play.\n",
    " 2. Ande para a DIREITA e suba no patamar BAIXO. Os DOIS conseguem -- ele\n",
    "    cabe num pulo so'. E' isto que torna a falha do passo 3 uma FALTA DE\n",
    "    BRACO, e nao um personagem quebrado.\n",
    " 3. Desca e siga ate' o patamar ALTO (2.40 m). Pule contra ele SEGURANDO a\n",
    "    direcao: o da ESQUERDA bate e cai. O da DIREITA fica PENDURADO, com o\n",
    "    topo do corpo no labio -- e fica la' enquanto o dedo segurar.\n",
    " 4. Pendurado, aperte o PULO: ele sobe e fica DE PE' em cima do patamar.\n",
    "    (Sobe primeiro, atravessa depois -- a diagonal cortaria a quina.)\n",
    " 5. Pendure-se de novo e SOLTE a direcao: ele larga e cai. Nao ha' botao de\n",
    "    largar, e nao deve haver.\n",
    " 6. OS AJUSTES: selecione o da esquerda e, no Inspector, card LEDGE, suba\n",
    "    'Ledge Grab (m)' de 0 para 0.60. Ele passa a agarrar o mesmo patamar.\n",
    " 7. Baixe 'Ledge Speed (m/s)' para 1: a subida fica lenta e deliberada.\n",
    "    Suba para 8: ele salta por cima. Devolva para 3.\n",
    " 8. Com a beirada armada, ande contra o patamar BAIXO no chao, sem pular:\n",
    "    ele NAO se pendura nele -- um degrau que se sobe a pe' nao e' beirada.\n",
    " 9. OS TRES CONTROLES DO SENSOR (W-LedgeSensor). No card LEDGE:\n",
    "    * 'Grab Height (m)' e' a JANELA acima da cabeca. Baixe para 0.20: ele\n",
    "      passa a raspar o labio sem o apanhar. Devolva para 0.60.\n",
    "    * 'Ledge Grab (m)' e' so' o X agora -- quao a' FRENTE ele procura.\n",
    "    * 'Grab Span (m)' da' EXTENSAO ao sensor: em 0 e' um raio so' (o mundo\n",
    "      de antes, ao bit); suba para 0.60 e o sensor vira um leque de cinco.\n",
    "      Aperte B: as cinco marcas aparecem, e SO' A QUE ACHOU acende.\n",
    "\n",
    "O QUE ISTO ACRESCENTA: o labio e' achado por UM raio para baixo, a' frente\n",
    "da cabeca, e o `x` em que ele bate E' o alvo da subida. Nem o pendurar nem a\n",
    "subida escrevem pose: os dois sao velocidade, como o arranque.\n",
);

#[cfg(test)]
#[path = "physics_smoke_ledge_tests.rs"]
mod tests;
