//! **A cena 108 — O QUE ELE VÊ** (`W-Probes`), a arena dos cinco sensores.
//!
//! Um percurso que acorda cada sensor do player, um de cada vez, para o artista
//! poder olhar o desenho de cada um sem ter de os separar de cabeça: o **túnel**
//! (o teto do agachar), o **beiral** (o perfil da quina) e a **parede** (o
//! flanco). A perna está sempre lá.
//!
//! # ⚠️ O que esta cena existe para julgar não é a física
//!
//! As cinco capacidades já estavam gateadas e smokadas. O que **não existia** é
//! o desenho: até esta wave o artista afinava `float_height`, `cling_distance`,
//! `wall_reach` e `corner_reach` **às cegas**, inferindo o alcance pelo
//! comportamento. Então o roteiro pede para olhar as LINHAS, e o passo que mais
//! importa é o **7**, o controle: a tecla `B` tem de as apagar junto com o
//! contorno — elas são a mesma pergunta.
//!
//! # ⚠️ As alturas são ARITMÉTICA da cápsula, e a cena depende delas
//!
//! | pose | centro | topo |
//! |---|---|---|
//! | de pé | 0,90 | **1,40** |
//! | agachado | 0,55 | **1,05** |
//!
//! A face do **túnel** fica em **1,20** — entre os dois, os mesmos 0,15 m de
//! folga de cada lado que as cenas 94 e 107 usam. O **beiral** fica mais alto
//! (2,60), porque o perfil da quina só é perguntado enquanto a cabeça SOBE:
//! ele tem de ser alcançável num pulo, e não a caminhar.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::PlatformPlayer;

use crate::App;
use crate::physics_smoke_player::{slab, spawn_player};

/// A altura de flutuação agachado — acima do piso geométrico de 0,50 que a
/// cápsula mede do centro ao pé (ver a cena 94).
pub(crate) const CROUCH_HEIGHT: f32 = 0.55;
/// A velocidade de cruzeiro agachado, m/s.
pub(crate) const CROUCH_SPEED: f32 = 2.0;
/// O alcance lateral da correção de quina, em metros.
pub(crate) const CORNER_REACH: f32 = 0.12;
/// O alcance do sensor lateral **além** da borda do corpo, em metros.
pub(crate) const WALL_REACH: f32 = 0.15;

/// A face de baixo do TÚNEL — entre o topo de pé (1,40) e o agachado (1,05).
pub(crate) const TUNNEL_BOTTOM: f32 = 1.20;
/// Onde o túnel começa e acaba.
pub(crate) const TUNNEL_X: [f32; 2] = [6.0, 10.0];
/// A face de baixo do BEIRAL — alcançável a pular, nunca a andar.
pub(crate) const LEDGE_BOTTOM: f32 = 2.60;
/// Onde a QUINA do beiral fica (a borda esquerda dele).
pub(crate) const LEDGE_EDGE_X: f32 = 15.0;
/// A face esquerda da PAREDE.
pub(crate) const WALL_FACE_X: f32 = 22.0;

impl App {
    /// **O que ele vê** — a arena dos cinco sensores.
    pub(crate) fn physics_smoke_probes(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_probe_scene(gfx.sim.world_mut());
        eprintln!("{PROBES_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 108**, separada do `App` de propósito.
///
/// ⚠️ **É esta função que os gates dirigem**, e não uma reconstrução deles: a
/// mensagem manda o artista fazer três gestos, e a única forma de afirmar que a
/// cena os permite é correr a MESMA cena pela ponte real.
pub(crate) fn build_probe_scene(world: &mut bevy_ecs::world::World) -> ph2d_ecs::Entity {
    slab(
        world,
        "Floor",
        Vec2::new(14.0, -0.5),
        [22.0, 0.5],
        0.0,
        [0.35, 0.35, 0.4, 1.0],
    );

    // O TÚNEL — o teto do agachar.
    let tunnel_half = (TUNNEL_X[1] - TUNNEL_X[0]) * 0.5;
    slab(
        world,
        "Tunnel",
        Vec2::new(TUNNEL_X[0] + tunnel_half, TUNNEL_BOTTOM + 1.0),
        [tunnel_half, 1.0],
        0.0,
        [0.46, 0.30, 0.32, 1.0],
    );

    // O BEIRAL — a quina que a assistência de canto existe para livrar. Ele se
    // estende para a DIREITA da quina, então o gesto é pular junto dela.
    slab(
        world,
        "Ledge",
        Vec2::new(LEDGE_EDGE_X + 3.0, LEDGE_BOTTOM + 0.4),
        [3.0, 0.4],
        0.0,
        [0.32, 0.44, 0.36, 1.0],
    );

    // A PAREDE — o flanco.
    slab(
        world,
        "Wall",
        Vec2::new(WALL_FACE_X + 0.5, 4.0),
        [0.5, 4.0],
        0.0,
        [0.30, 0.34, 0.42, 1.0],
    );

    let player = spawn_player(world, Vec2::new(0.0, 1.4));

    // ⚠️ **As capacidades são ARMADAS aqui**, e não herdadas: parede e agachar
    // nascem desligados no produto, e uma cena que herdasse o default mostraria
    // sensores que nunca acendem — lido como *"o desenho está quebrado"*.
    let mut q = world.query::<(&mut PlatformPlayer, &Transform)>();
    for (mut p, _) in q.iter_mut(world) {
        p.crouch_height = CROUCH_HEIGHT;
        p.crouch_speed = CROUCH_SPEED;
        p.corner_reach = CORNER_REACH;
        p.wall_reach = WALL_REACH;
        p.wall_slide_speed = 3.0;
        p.wall_jump_height = 2.0;
    }
    player
}

/// O roteiro da cena 108 — o que se julga são as LINHAS, não a física.
pub(crate) const PROBES_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 108] O QUE ELE VE (W-Probes). Um percurso com TUNEL (face em\n",
    "1.20), BEIRAL (face em 2.60, quina em x=15) e PAREDE (face em x=22).\n",
    "O topo do personagem de pe' mede 1.40; agachado, 1.05.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. CIMA (ou Z) pula. BAIXO (ou S)\n",
    "agacha. A tecla B liga/desliga o desenho da fisica.\n",
    "\n",
    "O QUE JULGAR, nesta ordem -- o assunto sao as LINHAS de sensor:\n",
    " 1. Marque Physics no transporte e de Play. Debaixo do personagem ha' TRES\n",
    "    linhas para BAIXO, cada uma com um TIQUE atravessado: e' a perna, que e'\n",
    "    um LEQUE (W-FootFan), e cada tique diz onde AQUELE pe' achou o chao.\n",
    " 2. Ande. As linhas acompanham, e os tiques ficam colados no chao.\n",
    " 3. Ao lado do corpo ha' tres tracinhos com TIQUE na ponta (o flanco) e uma\n",
    "    barra com as duas pontas marcadas sobre a cabeca (o vao da quina).\n",
    "    Apagados = ARMADO e nao perguntado. ⚠️ Eles comecam na BORDA do corpo,\n",
    "    nao no centro: o raio nasce no centro (o cast precisa disso) e 20 dos\n",
    "    35 px ficavam por baixo do contorno do collider.\n",
    " 4. TUNEL: segure BAIXO, entre, e SOLTE o botao parado la' dentro. Aparece\n",
    "    uma SILHUETA do corpo um pouco acima dele, ACESA: e' a varredura do\n",
    "    agachar a dizer 'nao cabe'. Ande para fora e solte: a silhueta apaga e\n",
    "    ele levanta-se.\n",
    " 5. BEIRAL: pule junto da quina (x=15). Enquanto ele SOBE, a barra sobre a\n",
    "    cabeca GANHA ALTURA (o leque mede `rel_up x dt x Look-ahead`, entao ele\n",
    "    e' zero parado -- e' honesto: um sensor parado olha mesmo zero para\n",
    "    cima) e ganha hastes ACESAS do lado tapado.\n",
    " 6. PAREDE: pule contra ela SEGURANDO a direcao. Os tres tracinhos do lado\n",
    "    acendem e ganham tique. Solte a direcao: eles voltam a apagar.\n",
    " 7. PARADO: com o transporte PAUSADO, arraste o personagem pelo canvas. Os\n",
    "    sensores TEM de acompanhar (a leitura re-deriva a geometria com o\n",
    "    relogio parado). Eles ficam todos APAGADOS: a lei nao correu, entao\n",
    "    nenhum RESPONDE -- so' o alcance e' desenhado, que e' o que se afina.\n",
    " 8. OS AJUSTES: no Inspector, cards Forgiveness e Walls, ha' quatro numeros\n",
    "    novos -- Corner Rays / Corner Look-ahead / Wall Rays / Wall Ray Spread.\n",
    "    Suba Wall Rays para 9: aparecem NOVE tracinhos de cada lado. Baixe o\n",
    "    Wall Ray Spread para 0.5: eles juntam-se para a cintura.\n",
    " 9. CONTROLE: aperte B. Os sensores TEM de sumir junto com o contorno dos\n",
    "    colliders -- e' a mesma pergunta ('mostre-me a fisica que nao se ve'),\n",
    "    e um segundo interruptor para ela seria a segunda porta.\n",
    "\n",
    "O QUE ISTO CORRIGE: ate' esta wave o overlay desenhava collider, joint,\n",
    "zona, seta, anel, linha d'agua e cruz de contato -- e dos cinco sensores do\n",
    "player, nada. O alcance da perna, do flanco e da quina era afinado as cegas.\n",
);

#[cfg(test)]
#[path = "physics_smoke_probes_tests.rs"]
mod tests;
