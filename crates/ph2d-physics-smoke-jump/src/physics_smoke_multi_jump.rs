//! **A cena 110 — A PRATELEIRA ALTA** (`W-MultiJump`), o pulo do ar.
//!
//! Duas raias IDÊNTICAS, dois personagens IDÊNTICOS, e a única diferença entre
//! eles é um número: quantos pulos têm depois de sair do chão. O da esquerda tem
//! **zero** (o mundo de antes desta wave), o da direita tem **um**.
//!
//! # ⚠️ O teclado dirige os DOIS ao mesmo tempo, e é isso que faz a cena
//!
//! `hand_input_to_players` entrega a entrada a **todo** `PlatformPlayer` da
//! cena, então um único gesto do artista move os dois lado a lado — a
//! comparação não depende de ele repetir o mesmo pulo duas vezes com a mesma
//! mão. O controle não é uma segunda corrida: ele está **dentro do quadro**.
//!
//! # ⚠️ As duas alturas são MEDIDAS, não escolhidas
//!
//! `measure_the_reachable_height` (a sonda desta wave), aperto SEGURADO, acima
//! do repouso:
//!
//! | pulos | alcança (lei: o ápice É a altura pedida, `f3a92736ab`) |
//! |---|---|
//! | um | **`jump_height` = 2,0 m** |
//! | dois | **`jump_height + air_jump_height` = 4,0 m** |
//!
//! (Antes de `f3a92736ab` o ápice passava da altura pedida e a tabela dizia 1,903 /
//! 4,028.) Os PÉS sobem isso a partir de `FEET_GAP`. A **baixa em 1,0 m** cabe num
//! pulo só — prova que os DOIS sabem pular —, e a **alta em 3,6 m** fica acima de um
//! pulo dado da baixa (3,40) e abaixo de dois do chão (4,40).
//!
//! Pular ENCOSTADO na parede, a seta contra ela, sobe o mesmo que na pista: o corpo do player
//! não raspa (Bug #14, curado; até lá subia 0,77 m dos 2,0 e o roteiro pedia o pulo na pista).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::PlatformPlayer;

use ph2d_physics_scene::{slab, spawn_player};

/// O topo da prateleira BAIXA — os pés sobem `FEET_GAP + jump_height` (2,40 m).
pub const LOW_TOP: f32 = 1.0;
/// O topo da prateleira ALTA — acima de um pulo dado da BAIXA (`LOW_TOP + FEET_GAP +
/// jump_height` = 3,40 m) e abaixo de dois do chão (`FEET_GAP + 2·2` = 4,40 m).
pub const HIGH_TOP: f32 = 3.6;
/// Pés acima do chão em repouso: o centro flutua a `FLOAT` e a cápsula desce 0,5 m.
pub const FEET_GAP: f32 = FLOAT - 0.5;

/// A distância entre as duas raias — larga o bastante para nenhum personagem
/// alcançar a geometria do vizinho.
pub const LANE_SPAN: f32 = 16.0;

/// Onde a raia da ESQUERDA começa (o personagem sem pulo do ar).
pub const LANE_A: f32 = 0.0;
/// Onde a raia da DIREITA começa (o personagem com um pulo do ar).
pub const LANE_B: f32 = LANE_A + LANE_SPAN;

/// A altura de flutuação das cenas de player (ver `physics_smoke_player`).
pub const FLOAT: f32 = 0.9;

/// O comprimento do chão de uma raia.
pub const FLOOR_LEN: f32 = 13.0;
/// A prateleira BAIXA, de `x0 + LOW_X0` a `x0 + LOW_X1`.
pub const LOW_X0: f32 = 2.0;
/// O fim da prateleira baixa — o começo da pista até a alta.
pub const LOW_X1: f32 = 4.0;
/// A parede da prateleira ALTA. ⚠️ A pista entre as duas (`HIGH_X0 − LOW_X1`) é
/// longa de propósito: com 3,5 m (a cena até 13/11) o pulo duplo só subia se
/// saísse do chão no primeiro meio metro da pista, e um pulo simples dado na
/// borda da baixa alcançava a alta — a cena aprovava o personagem errado.
pub const HIGH_X0: f32 = 10.0;

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

/// Uma raia: chão, a prateleira baixa, a alta, e o personagem no começo.
fn lane(world: &mut bevy_ecs::world::World, x0: f32, tag: &str, air_jumps: u32) -> Entity {
    block(world, &format!("{tag} Floor"), x0, x0 + FLOOR_LEN, 0.0);
    block(
        world,
        &format!("{tag} Low"),
        x0 + LOW_X0,
        x0 + LOW_X1,
        LOW_TOP,
    );
    block(
        world,
        &format!("{tag} High"),
        x0 + HIGH_X0,
        x0 + FLOOR_LEN,
        HIGH_TOP,
    );

    // ⚠️ Os dois nascem pela MESMA porta (`spawn_player`), então a geometria do
    // corpo — de que a aritmética das alturas depende — não pode divergir entre
    // eles. Só o NOME e a contagem de pulos são escritos por cima.
    let p = spawn_player(world, Vec2::new(x0 + 1.0, FLOAT + 0.3));
    world.entity_mut(p).insert(Name::new(tag.to_string()));
    world
        .entity_mut(p)
        .get_mut::<PlatformPlayer>()
        .expect("player")
        .air_jumps = air_jumps;
    p
}

/// **A prateleira alta** — o pulo do ar.
pub fn physics_smoke_multi_jump(ctx: &mut ph2d_physics_smoke_scenes::SceneCtx<'_>) {
    let _ = build_multi_jump_scene(ctx.world);
    eprintln!("{MULTI_JUMP_SMOKE_MESSAGE}");
}

/// **A geometria da cena 110**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
///
/// Devolve `(o sem pulo do ar, o com um)`.
pub fn build_multi_jump_scene(world: &mut bevy_ecs::world::World) -> (Entity, Entity) {
    let ground = lane(world, LANE_A, "One Jump", 0);
    let air = lane(world, LANE_B, "Double Jump", 1);
    (ground, air)
}

/// O roteiro da cena 110.
pub const MULTI_JUMP_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 110] A PRATELEIRA ALTA (W-MultiJump). Duas raias iguais,\n",
    "dois personagens iguais -- e o teclado dirige os DOIS ao mesmo tempo.\n",
    "O da ESQUERDA tem zero pulos no ar; o da DIREITA tem um.\n",
    "Prateleira baixa: topo 1.00 m. Prateleira alta: topo 3.60 m.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play.\n",
    " 2. Ande para a DIREITA e suba na prateleira BAIXA. Os DOIS conseguem --\n",
    "    ela cabe num pulo so' (2.00 m, a Jump Height). E' isto que torna a\n",
    "    falha do passo 3 uma FALTA DE PULO, e nao um personagem quebrado.\n",
    " 3. Desca para a pista e corra ate' a prateleira ALTA (3.60 m); pule junto\n",
    "    dela, encostado ou nao. Segure o pulo: o da ESQUERDA bate na parede e\n",
    "    cai -- nem pulando da beira da baixa ele chega. Com o da direita,\n",
    "    SEGURE, solte, e aperte de novo no alto do arco: ele POUSA em cima.\n",
    " 4. Ja' em cima, tente o segundo pulo de novo antes de pousar -- nao ha'\n",
    "    terceiro. A carga volta ao TOCAR o chao, nunca no ar.\n",
    " 5. OS AJUSTES: selecione o da esquerda e, no Inspector, card PULAR, suba\n",
    "    'Air Jumps' de 0 para 1. Ele passa a alcancar a mesma prateleira.\n",
    " 6. Baixe 'Air Jump Height (m)' para 0.5: o segundo pulo vira um degrauzinho\n",
    "    -- e' o Hollow Knight em vez do Celeste. Devolva para 2.\n",
    " 7. Ponha 'Air Jumps' em 3 e de' UM aperto so': ele da' UM pulo. Um aperto\n",
    "    e' um pulo; as outras cargas ficam la' para os apertos seguintes.\n",
    "\n",
    "O QUE ISTO ACRESCENTA: o `air actions counter` do tnua. A carga recarrega\n",
    "no CHAO, pela mesma porta que o coyote e o arranque ja' perguntam.\n",
);

#[cfg(test)]
#[path = "physics_smoke_multi_jump_tests.rs"]
mod tests;
