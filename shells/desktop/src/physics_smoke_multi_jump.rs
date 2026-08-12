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
//! | pulos | alcança |
//! |---|---|
//! | um | **1,903 m** |
//! | dois | **4,028 m** |
//!
//! Daí as duas prateleiras: a **baixa em 1,5 m** cabe num pulo só — é ela que
//! prova que os DOIS personagens sabem pular, sem o que a falha da alta leria
//! como *"o da esquerda está quebrado"* — e a **alta em 3,0 m** cai no vão entre
//! os dois números, então ela separa exatamente a capacidade desta wave.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O topo da prateleira BAIXA — cabe num pulo só (1,903 medido).
pub(crate) const LOW_TOP: f32 = 1.5;
/// O topo da prateleira ALTA — entre um pulo (1,903) e dois (4,028).
pub(crate) const HIGH_TOP: f32 = 3.0;

/// A distância entre as duas raias — larga o bastante para nenhum personagem
/// alcançar a geometria do vizinho.
pub(crate) const LANE_SPAN: f32 = 16.0;

/// Onde a raia da ESQUERDA começa (o personagem sem pulo do ar).
pub(crate) const LANE_A: f32 = 0.0;
/// Onde a raia da DIREITA começa (o personagem com um pulo do ar).
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

/// Uma raia: chão, a prateleira baixa, a alta, e o personagem no começo.
fn lane(world: &mut bevy_ecs::world::World, x0: f32, tag: &str, air_jumps: u32) -> Entity {
    block(world, &format!("{tag} Floor"), x0, x0 + 12.0, 0.0);
    block(world, &format!("{tag} Low"), x0 + 3.0, x0 + 5.0, LOW_TOP);
    block(world, &format!("{tag} High"), x0 + 8.5, x0 + 12.0, HIGH_TOP);

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

impl App {
    /// **A prateleira alta** — o pulo do ar.
    pub(crate) fn physics_smoke_multi_jump(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_multi_jump_scene(gfx.sim.world_mut());
        eprintln!("{MULTI_JUMP_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 110**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
///
/// Devolve `(o sem pulo do ar, o com um)`.
pub(crate) fn build_multi_jump_scene(world: &mut bevy_ecs::world::World) -> (Entity, Entity) {
    let ground = lane(world, LANE_A, "One Jump", 0);
    let air = lane(world, LANE_B, "Double Jump", 1);
    (ground, air)
}

/// O roteiro da cena 110.
pub(crate) const MULTI_JUMP_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 110] A PRATELEIRA ALTA (W-MultiJump). Duas raias iguais,\n",
    "dois personagens iguais -- e o teclado dirige os DOIS ao mesmo tempo.\n",
    "O da ESQUERDA tem zero pulos no ar; o da DIREITA tem um.\n",
    "Prateleira baixa: topo 1.50 m. Prateleira alta: topo 3.00 m.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play.\n",
    " 2. Ande para a DIREITA e suba na prateleira BAIXA. Os DOIS conseguem --\n",
    "    ela cabe num pulo so' (1.90 m de alcance medido). E' isto que torna a\n",
    "    falha do passo 3 uma FALTA DE PULO, e nao um personagem quebrado.\n",
    " 3. Desca e siga ate' a prateleira ALTA (3.00 m). Segure o pulo: o da\n",
    "    ESQUERDA bate na parede e cai. Com o da direita, SEGURE, solte, e\n",
    "    aperte de novo no alto do arco: ele sobe outra vez e POUSA em cima.\n",
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
