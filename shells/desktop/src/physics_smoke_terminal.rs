//! **A cena 116 — O POÇO** (`W-Fall`), o teto de queda.
//!
//! Três raias IDÊNTICAS, três personagens IDÊNTICOS, largados da MESMA altura —
//! e a única diferença entre eles são dois números do Inspector. O da
//! **ESQUERDA** não tem teto (o mundo de antes desta wave, o **CONTROLE**), o do
//! **MEIO** tem `Max Fall` e o da **DIREITA** tem `Max Fall` **e** `Glide Fall`.
//!
//! # ⚠️ Ninguém toca em nada, e é essa a metade que a cena existe para mostrar
//!
//! O planeio precisa do dedo — é um regime, e dura o que o dedo durar. O teto de
//! queda **não pergunta nada ao jogador**: dar Play basta. É por isso que os
//! três nascem NO AR em vez de correrem de um patamar, e é isso que separa esta
//! cena da irmã 112.
//!
//! # ⚠️ E a terceira raia é a COMPOSIÇÃO, que é onde um `max` acidental morde
//!
//! Os dois tetos passam pela MESMA porta e vence o MENOR
//! (`ph2d_platformer::descent_ceiling`). Com o dedo em baixo manda o planeio
//! (mais apertado); soltando, sobra o teto — que continua vivo. Uma composição
//! invertida daria ao planeio o poder de **acelerar** uma queda que o teto já
//! tinha limitado, e é exactamente isso que o passo 4 do roteiro procura.
//!
//! # ⚠️ Os números do roteiro saem da sonda, não do olho
//!
//! `the_cap_decides_who_lands_first` mede ESTA geometria com ESTES números.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// O topo do chão de cada raia.
pub(crate) const GROUND_TOP: f32 = 0.0;
/// De quanto acima do chão os três são largados.
///
/// ⚠️ **ARITMÉTICA, não gosto:** uma queda livre percorre `½·g·t²`, então estes
/// dezasseis metros custam **1,52 s** MEDIDOS ao que não tem teto, contra
/// **4,05 s** com o teto de 4 m/s. Mais alto e o
/// personagem sai do quadro antes de o artista o ver; mais baixo e as três
/// aterragens ficam demasiado juntas para se distinguirem a olho.
pub(crate) const DROP_TOP: f32 = 16.0;

/// O teto de queda das raias do meio e da direita, m/s.
pub(crate) const CAP: f32 = 4.0;
/// O planeio da raia da direita, m/s — **mais apertado que o teto**, senão a
/// composição não teria o que mostrar (o menor é que vence).
pub(crate) const GLIDE: f32 = 1.5;

/// A largura do chão de cada raia.
pub(crate) const GROUND_END: f32 = 6.0;
/// A distância entre raias — ⚠️ maior que `GROUND_END`, para a geometria de uma
/// nunca alcançar a outra (gate).
pub(crate) const LANE_SPAN: f32 = 9.0;

/// A altura de flutuação das cenas de player (ver `physics_smoke_player`).
pub(crate) const FLOAT: f32 = 0.9;

/// As três raias: o rótulo, o teto de queda e o planeio.
///
/// ⚠️ **A primeira carrega zero nos dois, e é o CONTROLE** — sem ela as outras
/// duas seriam duas quedas sem régua.
pub(crate) const LANES: [(&str, f32, f32); 3] = [
    ("No Cap", 0.0, 0.0),
    ("Capped", CAP, 0.0),
    ("Capped+Glide", CAP, GLIDE),
];

/// Onde a raia `i` começa.
#[must_use]
pub(crate) fn lane_x(i: usize) -> f32 {
    i as f32 * LANE_SPAN
}

/// Uma raia: o chão, e o personagem largado no alto.
fn lane(world: &mut bevy_ecs::world::World, x0: f32, tag: &str, cap: f32, glide: f32) -> Entity {
    let half_w = GROUND_END * 0.5;
    slab(
        world,
        &format!("{tag} Ground"),
        Vec2::new(x0 + half_w, GROUND_TOP - 0.5),
        [half_w, 0.5],
        0.0,
        [0.35, 0.35, 0.4, 1.0],
    );

    // ⚠️ Os três nascem pela MESMA porta (`spawn_player`), então a geometria do
    // corpo — de que a aritmética das alturas depende — não pode divergir entre
    // eles. Só o NOME e os dois tetos são escritos por cima.
    let p = spawn_player(world, Vec2::new(x0 + half_w, DROP_TOP + FLOAT));
    world.entity_mut(p).insert(Name::new(tag.to_string()));
    {
        let mut e = world.entity_mut(p);
        let mut cfg = e.get_mut::<PlatformPlayer>().expect("player");
        cfg.max_fall_speed = cap;
        cfg.glide_fall_speed = glide;
    }
    p
}

impl App {
    /// **O poço** — o teto de queda.
    pub(crate) fn physics_smoke_terminal(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_terminal_scene(gfx.sim.world_mut());
        eprintln!("{TERMINAL_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 116**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
pub(crate) fn build_terminal_scene(world: &mut bevy_ecs::world::World) -> Vec<Entity> {
    LANES
        .iter()
        .enumerate()
        .map(|(i, (tag, cap, glide))| lane(world, lane_x(i), tag, *cap, *glide))
        .collect()
}

/// O roteiro da cena 116 — ⚠️ **os números saem da sonda**
/// (`the_cap_decides_who_lands_first`), e não do olho.
pub(crate) const TERMINAL_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 116] O POCO (W-Fall). Tres raias iguais, tres personagens\n",
    "iguais, largados dos MESMOS 16.00 m -- so' os numeros diferem.\n",
    "ESQUERDA sem teto (o controle) - MEIO com Max Fall 4.00 m/s - DIREITA com\n",
    "Max Fall 4.00 e Glide Fall 1.50.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play. NAO toque em nada -- e' essa a\n",
    "    metade que esta cena existe para mostrar. O da ESQUERDA despenca e\n",
    "    aterra em 1.52 s; os outros dois descem LISOS e chegam em 4.05 s.\n",
    "    (Se os tres chegarem juntos, PARE: o teto nao esta' a agir.)\n",
    " 2. Olhe a QUEDA, nao so' a aterragem: o da esquerda ACELERA (a distancia\n",
    "    entre ele e os outros cresce a cada instante) e os outros dois descem\n",
    "    sempre ao mesmo ritmo. E' isso que uma velocidade terminal e'.\n",
    " 3. Reset. Repita SEGURANDO o pulo desde o inicio. As duas primeiras raias\n",
    "    caem exactamente igual -- o teto nao pergunta nada ao jogador -- e so'\n",
    "    a DIREITA fica mais lenta, chegando em 9.22 s.\n",
    " 4. Ainda a segurar, SOLTE o botao a meio da descida: a da direita volta ao\n",
    "    ritmo do MEIO no mesmo instante, e nao ao da esquerda. Vence o teto\n",
    "    MENOR dos dois, e soltar o dedo deixa o outro vivo -- nunca o\n",
    "    ilimitado. (Se ela despencar como a da esquerda, PARE.)\n",
    " 5. No chao, pule. A altura do pulo tem de ser a MESMA nas tres raias -- um\n",
    "    teto de DESCIDA nunca empurra para baixo, entao nao pode encolher uma\n",
    "    subida. (Se alguma pular mais baixo, PARE.)\n",
    " 6. OS AJUSTES: selecione o da esquerda e, no Inspector, card FALL, suba\n",
    "    'Max Fall (m/s)' de 0 para 4.00. Ele passa a cair como o do meio.\n",
    " 7. Ponha 40.00: quase nao ajuda -- caindo 16 m ele nunca chega perto disso.\n",
    "    Ponha 1.00: desce como uma pena. E volte a 0: o teto desliga.\n",
    "\n",
    "O QUE ISTO ACRESCENTA: ate' esta wave nao existia velocidade terminal\n",
    "nenhuma. Medido, uma queda de mil metros chega a 142.57 m/s aos 8 s e\n",
    "continua a crescer -- um personagem que caia de alto o bastante atravessa o\n",
    "cenario a velocidades que nenhum colisor discreto resolve, e o artista nao\n",
    "tinha numero nenhum para dizer 'nao mais depressa que isto'.\n",
);

#[cfg(test)]
#[path = "physics_smoke_terminal_tests.rs"]
mod tests;
