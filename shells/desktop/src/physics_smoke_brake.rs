//! **A cena 114 — A DERRAPADA** (`W-Brake`), o peso.
//!
//! Três raias IDÊNTICAS, três personagens IDÊNTICOS, e a única diferença entre
//! eles é um número: quanto do orçamento de aceleração eles gastam a FREAR. O da
//! esquerda tem **0** (gelo), o do meio **1** (o mundo de antes desta wave) e o
//! da direita **2**.
//!
//! # ⚠️ O teclado dirige os TRÊS ao mesmo tempo
//!
//! `hand_input_to_players` entrega a entrada a **todo** `PlatformPlayer` da cena,
//! então um único gesto move os três lado a lado: o controle está **dentro do
//! quadro**, e não numa segunda corrida.
//!
//! # ⚠️ A aceleração desta cena é BAIXA, e é ela que torna a wave VISÍVEL
//!
//! Com o perfil de partida (`accel = 60`) a paragem inteira mede **0,17 m** e
//! cabe em cinco tiques — a diferença entre freio 1 e freio 2 é de **onze
//! centímetros**, e um personagem tem quarenta de largura. A cena seria um
//! contraste que ninguém consegue ver.
//!
//! ⇒ a cena autora [`RUN_ACCEL`] baixo de propósito, e a sonda
//! `measure_the_scene_brake` é quem escolheu o número: ela varre a config até as
//! três paragens se separarem em METROS. ⚠️ **Isto não é um default de produto**
//! — é a mesma nota do `WalkConfig::STARTING_POINT`: aqui o número existe para o
//! olho medir, não para jogar.
//!
//! # ⚠️ E o gelo tem CONSEQUÊNCIA, em vez de só deslizar
//!
//! Falhar tem de ser visível: a plataforma acaba num POÇO. O da esquerda não
//! pára, então ele **cai** — que é exatamente o que `brake = 0` promete, e é a
//! forma de a promessa não ser confundida com *"o knob está quebrado"*.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name};
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// A altura de flutuação das cenas de player.
pub(crate) const FLOAT: f32 = 0.9;

/// A velocidade de cruzeiro desta cena, m/s.
pub(crate) const RUN_SPEED: f32 = 8.0;
/// A aceleração desta cena, m/s² — ⚠️ **MEDIDA**, ver o topo do módulo.
pub(crate) const RUN_ACCEL: f32 = 8.0;

/// O topo da plataforma.
pub(crate) const DECK_TOP: f32 = 0.0;
/// Onde a plataforma começa.
pub(crate) const DECK_START: f32 = 0.0;
/// Onde o artista larga o direcional — a marca no chão.
pub(crate) const MARK_X: f32 = 12.0;
/// Onde a plataforma acaba (a beira do poço) — ⚠️ **para além de onde o freio
/// mais fraco que AINDA PÁRA consegue parar**, senão a cena mostraria os três a
/// cair e não diria nada sobre nenhum.
pub(crate) const DECK_END: f32 = 20.0;
/// O fundo do poço.
pub(crate) const PIT_TOP: f32 = -6.0;

/// Os três freios da cena.
pub(crate) const BRAKES: [(f32, &str); 3] = [(0.0, "Ice"), (1.0, "Normal"), (2.0, "Hard")];

/// A distância entre raias.
///
/// ⚠️ **Ela tem de passar do POÇO, e não do deck** — a primeira versão desta cena
/// media 26 contra um `DECK_END` de 20 e parecia folgada, mas o poço vai até
/// `DECK_END + 8 = 28`: as duas últimas unidades de cada poço ficavam **debaixo
/// do deck da raia seguinte**, e o do gelo aterrava lá. Quem pegou não foi a
/// aritmética — foi o gate reescrito para MEDIR a geometria montada em vez de
/// comparar duas constantes (o clippy chamou àquilo *"assertion has a constant
/// value"*, e tinha razão duas vezes).
pub(crate) const LANE_SPAN: f32 = 32.0;

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

/// Uma raia: a plataforma, a marca de onde largar, o poço — e o personagem.
fn lane(world: &mut bevy_ecs::world::World, x0: f32, tag: &str, brake: f32) -> Entity {
    let stone = [0.35, 0.35, 0.4, 1.0];
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
    block(
        world,
        &format!("{tag} Deck"),
        x0 + DECK_START,
        x0 + DECK_END,
        DECK_TOP,
        PIT_TOP,
        stone,
    );
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

    // ⚠️ Os três nascem pela MESMA porta (`spawn_player`), então nada da
    // geometria do corpo pode divergir entre eles. Só o NOME e os três números
    // de caminhada são escritos por cima.
    let p = spawn_player(world, Vec2::new(x0 + 1.0, DECK_TOP + FLOAT));
    world.entity_mut(p).insert(Name::new(tag.to_string()));
    {
        let mut e = world.entity_mut(p);
        let mut cfg = e.get_mut::<PlatformPlayer>().expect("player");
        cfg.speed = RUN_SPEED;
        cfg.acceleration = RUN_ACCEL;
        cfg.brake_scale = brake;
    }
    p
}

impl App {
    /// **A derrapada** — o peso.
    pub(crate) fn physics_smoke_brake(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_brake_scene(gfx.sim.world_mut());
        eprintln!("{BRAKE_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 114**, separada do `App` de propósito — é ela que os
/// gates dirigem, e não uma reconstrução deles.
pub(crate) fn build_brake_scene(world: &mut bevy_ecs::world::World) -> Vec<Entity> {
    BRAKES
        .iter()
        .enumerate()
        .map(|(i, (brake, tag))| lane(world, lane_x(i), tag, *brake))
        .collect()
}

/// O roteiro da cena 114 — ⚠️ **os números saem da sonda**
/// (`measure_the_scene_brake`), e não do olho.
pub(crate) const BRAKE_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 114] A DERRAPADA (W-Brake). Tres raias iguais, tres\n",
    "personagens iguais -- e o teclado dirige os TRES ao mesmo tempo.\n",
    "So' um numero difere: quanto do orcamento de aceleracao eles gastam a\n",
    "FREAR, ao largar o direcional. ESQUERDA 0 (gelo) - MEIO 1 (o mundo de\n",
    "antes desta wave) - DIREITA 2.\n",
    "A faixa AMBAR no chao (x = 12.00 na raia) e' de onde medir.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. A tecla B\n",
    "liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de' Play.\n",
    " 2. Corra para a DIREITA e SOLTE a seta ao passar da faixa ambar. Medido:\n",
    "    o do MEIO derrapa 2.95 m e para; o da DIREITA derrapa 1.43 m -- metade\n",
    "    do caminho. O da ESQUERDA derrapa 9.26 m, nao para, e CAI no poco.\n",
    "    (Cair e' o que freio 0 promete. Se ele parasse, PARE.)\n",
    " 3. Reset. Agora corra e, em vez de soltar, aperte a seta CONTRARIA. Os\n",
    "    tres viram no MESMO tempo -- inverter e' o fator de viragem, que esta\n",
    "    wave nao toca. (Se o da esquerda demorar mais a virar, PARE.)\n",
    " 4. Corra e PULE. No ar os tres se comportam igual: o freio e' do CHAO, e\n",
    "    quem responde no ar e' 'Air Acceleration'. (Se o do gelo flutuar de\n",
    "    lado diferente dos outros, PARE.)\n",
    " 5. OS AJUSTES: selecione o da ESQUERDA e, no Inspector, card WALK, suba\n",
    "    'Brake' de 0 para 1: ele passa a derrapar 2.95 m, como o do meio.\n",
    "    Para 0.50 sao 5.97 m -- ele ainda para, na ultima curva da beira.\n",
    " 6. Baixe para 0.25: derrapa 8.46 m e volta a CAIR. E suba para 40: ele\n",
    "    para NA HORA, no tique em que voce solta, sem recuar um centimetro.\n",
    "    (40 nao e' magia: e' onde a sobra inteira cabe num tique NESTA config,\n",
    "    `speed / (turn*accel*dt)`. Com o perfil de partida seriam 4.)\n",
    "\n",
    "O QUE ISTO ACRESCENTA: ate' esta wave 'Acceleration' respondia por arrancar\n",
    "E por parar, e o fator de viragem so' cobre INVERTER. Um personagem que\n",
    "arranca rapido era obrigado a parar rapido -- e gelo era inexprimivel.\n",
    "\n",
    "⚠️ A aceleracao desta cena e' BAIXA de proposito (8 m/s^2 contra os 60 do\n",
    "perfil de partida): com 60 a paragem inteira mede 17 cm, e a diferenca\n",
    "entre freio 1 e 2 seria de onze centimetros -- invisivel. Nao e' um default\n",
    "de produto; e' o que torna a wave mensuravel a olho.\n",
);

#[cfg(test)]
#[path = "physics_smoke_brake_tests.rs"]
mod tests;
