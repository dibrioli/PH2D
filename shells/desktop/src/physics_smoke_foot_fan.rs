//! **A cena 109 — A FENDA** (`W-Probes2`), a perna que é um leque.
//!
//! Dois personagens IDÊNTICOS parados sobre fendas IDÊNTICAS, e a única
//! diferença entre eles é um número: quantos raios a perna casta. O da
//! esquerda tem **um**, o da direita tem **três**.
//!
//! # ⚠️ O que esta cena existe para julgar é uma AUSÊNCIA
//!
//! A cura desta wave é *nada acontecer* — o personagem fica de pé onde o corpo
//! dele tem apoio —, e uma cena que mostrasse só isso seria indistinguível de
//! uma cena que não faz nada. Por isso o **controle está DENTRO do quadro**: o
//! vizinho da esquerda, com a perna de um raio só, **afunda**, e ele é a
//! fotografia do mundo de antes.
//!
//! # ⚠️ A largura das fendas é ARITMÉTICA do corpo, não gosto
//!
//! O corpo mede **0,4 m** de largura (`radius = 0,2` ⇒ meia-largura 0,2), e os
//! pés de fora nascem em `±meia-largura × spread` = **±0,2**.
//!
//! | fenda | meia-largura | os pés de fora caem em | veredito |
//! |---|---|---|---|
//! | estreita (**0,30**) | 0,15 | 0,05 m **de chão** | o leque SEGURA |
//! | larga (**0,60**) | 0,30 | dentro do buraco | ninguém segura |
//!
//! A segunda não é uma limitação a esconder: a perna não é levitação, e um
//! personagem que ficasse de pé sobre um buraco maior que ele seria pior que
//! um que cai.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// A fenda que o CORPO atravessa — mais estreita que a distância entre os pés.
pub(crate) const GAP_NARROW: f32 = 0.30;
/// A fenda mais larga que o corpo — nenhum pé alcança chão.
pub(crate) const GAP_WIDE: f32 = 0.60;

/// Onde o personagem de UM raio fica parado.
pub(crate) const ONE_RAY_X: f32 = 3.0;
/// Onde o personagem de TRÊS raios (o default) fica parado.
pub(crate) const FAN_X: f32 = 7.0;
/// A fenda larga, sem ninguém em cima — o artista anda até ela.
pub(crate) const WIDE_X: f32 = 11.0;

/// A altura de flutuação das cenas de player (ver `physics_smoke_player`).
pub(crate) const FLOAT: f32 = 0.9;

/// Um trecho de chão de `x0` a `x1`, com o topo em `y = 0`.
fn floor_span(world: &mut bevy_ecs::world::World, name: &str, x0: f32, x1: f32) {
    let half = (x1 - x0) * 0.5;
    slab(
        world,
        name,
        Vec2::new(x0 + half, -0.5),
        [half, 0.5],
        0.0,
        [0.35, 0.35, 0.4, 1.0],
    );
}

impl App {
    /// **A fenda** — a perna é um leque.
    pub(crate) fn physics_smoke_foot_fan(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_foot_fan_scene(gfx.sim.world_mut());
        eprintln!("{FOOT_FAN_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 109**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
///
/// Devolve `(o de um raio, o de três)`.
pub(crate) fn build_foot_fan_scene(world: &mut bevy_ecs::world::World) -> (Entity, Entity) {
    // O chão, em quatro trechos: as três fendas são o que falta entre eles.
    let n = GAP_NARROW * 0.5;
    let w = GAP_WIDE * 0.5;
    floor_span(world, "Floor A", -2.0, ONE_RAY_X - n);
    floor_span(world, "Floor B", ONE_RAY_X + n, FAN_X - n);
    floor_span(world, "Floor C", FAN_X + n, WIDE_X - w);
    floor_span(world, "Floor D", WIDE_X + w, 17.0);

    // ⚠️ Os dois nascem pela MESMA porta (`spawn_player`), então a geometria do
    // corpo — de que a aritmética das fendas depende — não pode divergir entre
    // eles. Só o NOME e a contagem de raios são escritos por cima.
    let one = spawn_player(world, Vec2::new(ONE_RAY_X, FLOAT + 0.6));
    world.entity_mut(one).insert(Name::new("One Ray"));
    world
        .entity_mut(one)
        .get_mut::<PlatformPlayer>()
        .expect("player")
        .foot_samples = 1;

    let fan = spawn_player(world, Vec2::new(FAN_X, FLOAT + 0.6));
    world.entity_mut(fan).insert(Name::new("Fan"));

    (one, fan)
}

/// O roteiro da cena 109.
pub(crate) const FOOT_FAN_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 109] A FENDA (W-Probes2). Dois personagens iguais, duas\n",
    "fendas iguais de 0.30 m -- e o corpo mede 0.40 m, entao ele ATRAVESSA as\n",
    "duas. O da ESQUERDA tem a perna de UM raio; o da DIREITA, de tres.\n",
    "A terceira fenda (x=11) mede 0.60: mais larga que o corpo.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play. O da DIREITA fica DE PE'. O da\n",
    "    ESQUERDA afunda ~0.41 m -- quase metade da altura de flutuacao -- num\n",
    "    buraco que o corpo dele cobre. Ele e' a fotografia do mundo de antes.\n",
    " 2. Aperte B. Debaixo do da direita ha' TRES linhas para baixo: a do meio\n",
    "    passa pela fenda SEM tique (nao achou nada) e as de fora tem tique no\n",
    "    chao. Debaixo do da esquerda ha' UMA, e ela mergulha no buraco.\n",
    " 3. OS AJUSTES: selecione o da esquerda e, no Inspector, card PERNA, suba\n",
    "    'Foot Rays' de 1 para 3. Ele SOBE na hora, e ganha os outros dois pes.\n",
    " 4. Baixe 'Foot Spread' para 0.2: os pes juntam-se ao centro, caem dentro\n",
    "    da fenda, e ele afunda de novo. Devolva para 1.\n",
    " 5. Numeros PARES sao arredondados para CIMA (4 vira 5): o raio do meio e'\n",
    "    quem desempata, entao ele tem de existir.\n",
    " 6. CONTROLE: ande com o da direita ate' a fenda de x=11, que e' mais larga\n",
    "    que o corpo. Ele CAI -- a perna nao e' levitacao, e um personagem que\n",
    "    ficasse de pe' sobre um buraco maior que ele seria pior que um que cai.\n",
    "\n",
    "O QUE ISTO CORRIGE: a perna castava UM raio no centro, entao o personagem\n",
    "nao achava chao onde o corpo dele tinha apoio. Medido: 0.411 m de queda\n",
    "parado sobre 10 cm de fenda; a 40 cm ele saia do mundo (113 m).\n",
);

#[cfg(test)]
#[path = "physics_smoke_foot_fan_tests.rs"]
mod tests;
