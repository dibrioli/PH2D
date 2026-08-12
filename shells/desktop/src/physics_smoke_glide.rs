//! **A cena 112 — O VÃO** (`W-Glide`), o planeio.
//!
//! Duas raias IDÊNTICAS, dois personagens IDÊNTICOS, e a única diferença entre
//! eles é um número: o teto de descida. O da esquerda tem **zero** (o mundo de
//! antes desta wave), o da direita tem **2,00 m/s**.
//!
//! # ⚠️ O teclado dirige os DOIS ao mesmo tempo, e é isso que faz a cena
//!
//! `hand_input_to_players` entrega a entrada a **todo** `PlatformPlayer` da
//! cena, então um único gesto do artista move os dois lado a lado — o controle
//! está **dentro do quadro**, e não numa segunda corrida.
//!
//! # ⚠️ O vão é MEDIDO, e a medição teve de mudar de forma
//!
//! A sonda `measure_the_gap_a_glide_crosses` larga o personagem **parado** e
//! mede quanto ele atravessa enquanto cai — mas esta cena o faz **correr de um
//! patamar**, e quem sai a correr já leva a velocidade toda. São números
//! diferentes, e usar o primeiro para dimensionar a segunda seria a repetição
//! exata do erro que a cena da beirada cometeu (ela pôs o patamar alto numa
//! altura calculada com o número do ar livre, e o corpo nunca lá chegava).
//!
//! O número que dimensiona esta cena sai de
//! `the_gap_is_between_what_each_one_reaches`, que mede **esta geometria**.
//!
//! # ⚠️ E há um POÇO, em vez de uma queda sem fim
//!
//! Falhar tem de ser visível e tem de acabar: o chão do poço fica bem abaixo, e
//! quem não atravessa aterra nele em vez de sumir do quadro para sempre.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O topo do patamar de onde se salta.
pub(crate) const TAKEOFF_TOP: f32 = 3.0;
/// Onde ele acaba (a beira de onde se corre).
pub(crate) const TAKEOFF_END: f32 = 4.0;
/// A largura do vão — ⚠️ **MEDIDA nesta geometria e com este gesto**, pela
/// sonda `where_each_one_crosses_the_landing_level`: correndo do patamar com o
/// pulo preso, o **sem planeio** atravessa **7,18 m** e o **planador** atravessa
/// **18,47 m**. Doze fica entre os dois com folga dos dois lados (4,8 m de
/// margem para quem cai, 6,5 m para quem passa).
///
/// ⚠️ **A primeira versão deste número era 7,00 e vinha da fixture ERRADA** — a
/// sonda que larga o personagem **parado** (`measure_the_gap_a_glide_crosses`,
/// 4,18 m sem planeio). Quem corre de um patamar **e pula** leva a velocidade
/// toda, e com 7,00 os DOIS atravessavam: a cena não mostrava falha nenhuma.
pub(crate) const GAP: f32 = 12.0;
/// O topo do patamar em que se aterra.
pub(crate) const LANDING_TOP: f32 = 0.0;
/// Onde o patamar de aterragem acaba — ⚠️ **para além de onde o planador cruza
/// o nível** (22,47 medidos), senão ele atravessaria o vão e cairia na ponta.
pub(crate) const LANDING_END: f32 = 30.0;
/// O fundo do poço.
pub(crate) const PIT_TOP: f32 = -4.0;

/// O teto de descida autorado na raia da direita, m/s.
pub(crate) const GLIDE: f32 = 2.0;

/// A distância entre as duas raias — ⚠️ maior que `LANDING_END`, para a
/// geometria de uma nunca alcançar a outra (gate).
pub(crate) const LANE_SPAN: f32 = 36.0;
/// Onde a raia da ESQUERDA começa (o personagem sem planeio).
pub(crate) const LANE_A: f32 = 0.0;
/// Onde a raia da DIREITA começa (o personagem que plana).
pub(crate) const LANE_B: f32 = LANE_A + LANE_SPAN;

/// A altura de flutuação das cenas de player (ver `physics_smoke_player`).
pub(crate) const FLOAT: f32 = 0.9;

/// Um bloco de `x0` a `x1` com o topo em `top`, nascendo de `floor`.
fn block(
    world: &mut bevy_ecs::world::World,
    name: &str,
    x0: f32,
    x1: f32,
    top: f32,
    floor: f32,
    tint: [f32; 4],
) {
    let half_w = (x1 - x0) * 0.5;
    let half_h = (top - floor) * 0.5;
    slab(
        world,
        name,
        Vec2::new(x0 + half_w, top - half_h),
        [half_w, half_h],
        0.0,
        tint,
    );
}

/// Uma raia: o patamar de onde se salta, o vão, o de aterragem, e o poço.
fn lane(world: &mut bevy_ecs::world::World, x0: f32, tag: &str, glide: f32) -> Entity {
    let stone = [0.35, 0.35, 0.4, 1.0];
    // ⚠️ **O poço é PRIMEIRO**, para os patamares ficarem desenhados por cima
    // dele — ele atravessa o vão inteiro, e é onde quem não atravessa aterra.
    block(
        world,
        &format!("{tag} Pit"),
        x0,
        x0 + LANDING_END,
        PIT_TOP,
        PIT_TOP - 1.0,
        [0.20, 0.18, 0.22, 1.0],
    );
    block(
        world,
        &format!("{tag} Takeoff"),
        x0,
        x0 + TAKEOFF_END,
        TAKEOFF_TOP,
        PIT_TOP,
        stone,
    );
    block(
        world,
        &format!("{tag} Landing"),
        x0 + TAKEOFF_END + GAP,
        x0 + LANDING_END,
        LANDING_TOP,
        PIT_TOP,
        stone,
    );

    // ⚠️ Os dois nascem pela MESMA porta (`spawn_player`), então a geometria do
    // corpo — de que a aritmética das alturas depende — não pode divergir entre
    // eles. Só o NOME e o teto de descida são escritos por cima.
    let p = spawn_player(world, Vec2::new(x0 + 1.0, TAKEOFF_TOP + FLOAT));
    world.entity_mut(p).insert(Name::new(tag.to_string()));
    world
        .entity_mut(p)
        .get_mut::<PlatformPlayer>()
        .expect("player")
        .glide_fall_speed = glide;
    p
}

impl App {
    /// **O vão** — o planeio.
    pub(crate) fn physics_smoke_glide(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_glide_scene(gfx.sim.world_mut());
        eprintln!("{GLIDE_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 112**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
///
/// Devolve `(o sem planeio, o que plana)`.
pub(crate) fn build_glide_scene(world: &mut bevy_ecs::world::World) -> (Entity, Entity) {
    let plain = lane(world, LANE_A, "No Glide", 0.0);
    let glide = lane(world, LANE_B, "Glide", GLIDE);
    (plain, glide)
}

/// O roteiro da cena 112.
pub(crate) const GLIDE_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 112] O VAO (W-Glide). Duas raias iguais, dois personagens\n",
    "iguais -- e o teclado dirige os DOIS ao mesmo tempo.\n",
    "O da ESQUERDA nao plana; o da DIREITA desce no maximo a 2.00 m/s.\n",
    "Patamar de saida: topo 3.00 m. Vao: 12.00 m. Aterragem: topo 0.00 m.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play.\n",
    " 2. Corra para a DIREITA ate' sair do patamar, SEM segurar o pulo. Os dois\n",
    "    caem igual e aterram no POCO. E' o mundo de antes desta wave.\n",
    " 3. Reset. Agora corra e SEGURE o pulo ao sair da beira. O da esquerda cai\n",
    "    no poco na mesma; o da DIREITA desce devagar e ATRAVESSA o vao.\n",
    "    (E' so' para isto que um planeio existe: atravessar.)\n",
    " 4. Pendurado no ar a planar, SOLTE o botao: a queda volta a ser a de\n",
    "    sempre no mesmo instante. Nao ha' inercia de planeio, e nao deve haver.\n",
    " 5. No chao, SEGURE o pulo e pule. A altura do pulo tem de ser a MESMA nas\n",
    "    duas raias -- o planeio nunca empurra para baixo, entao ele nao pode\n",
    "    encolher uma subida. (Se o da direita pular mais baixo, PARE.)\n",
    " 6. OS AJUSTES: selecione o da esquerda e, no Inspector, card GLIDE, suba\n",
    "    'Glide Fall (m/s)' de 0 para 2.00. Ele passa a atravessar o mesmo vao.\n",
    " 7. Baixe para 0.50: ele quase nao desce e passa MUITO longe. Suba para\n",
    "    8.00: quase nao ajuda -- 8 m/s ja' e' mais rapido do que ele cai.\n",
    "\n",
    "O QUE ISTO ACRESCENTA: o planeio e' um TETO de descida, nao uma escala de\n",
    "gravidade. Uma escala nunca assenta (a velocidade cresce com a profundidade)\n",
    "e um alvo inverteria quem sobe. As tres formas estao medidas lado a lado em\n",
    "`measure_glide.rs`.\n",
);

#[cfg(test)]
#[path = "physics_smoke_glide_tests.rs"]
mod tests;
