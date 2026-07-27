//! **A cena da FERRAMENTA DE INTERAÇÃO** (`PH2D_PHYSICS_SMOKE=53`, W-Hand).
//!
//! A cena 52 provou que a MÃO existe. Esta é sobre o que o artista pediu depois:
//! *"merece uma seção de ajustes e tipos — segurar RÍGIDO, tipo Rope, a mola está
//! dura e merece parâmetros manuais"*, mais **explosão** e **campo de atração**.
//!
//! ⚠️ **Ela abre TOCANDO e com o painel de física ABERTO** — as três ferramentas
//! existem precisamente enquanto o solver corre, e a seção Interaction é onde se
//! escolhe qual delas está na mão.
//!
//! Quatro estações, cada uma para uma pergunta:
//!
//! - **A PRANCHA**: pegar uma barra pela PONTA e comparar os três modos. Spring
//!   deixa balançar, Rigid a segura NIVELADA, Rope a deixa pendurar a `slack` de
//!   distância.
//! - **A TORRE**: o alvo do estouro. Um clique no pé dela a espalha.
//! - **O ENXAME**: oito bolinhas espalhadas, o alvo do campo de atração — segurar
//!   o botão as JUNTA no cursor; com força negativa ele as espalha.
//! - **O MURO MÓVEL**: um corpo ESTÁTICO. Arrastá-lo com o relógio andando move o
//!   collider junto — era o **bug do collider fantasma** que o artista reportou.
//!
//! Os números da mensagem saíram da sonda `probe_smoke_53`, rodada sobre ESTAS
//! peças antes de a mensagem ser escrita.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const GREY: [f32; 4] = [0.75, 0.75, 0.8, 1.0];
const HOT: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
const COOL: [f32; 4] = [0.4, 0.8, 0.95, 1.0];
const LIME: [f32; 4] = [0.55, 0.9, 0.35, 1.0];

/// As quatro estações **com o chão** — a MESMA construção que a sonda headless
/// mede, para os números serem sobre a cena que o artista abre.
///
/// ⚠️ **Chão PRÓPRIO e largo**, pela lição que a cena 52 pagou: o
/// `physics_smoke::spawn_floor` compartilhado mede `half_x = 4` e estas estações
/// vão de −8 a +8, então elas nasceriam FORA dele e cairiam — com um sintoma que
/// parece um bug de ferramenta.
pub(crate) fn spawn_props(world: &mut World) {
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, -0.25)),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 0.5], [0.40, 0.42, 0.48, 1.0]),
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 11.0,
                half_y: 0.25,
            },
            ..Collider::default()
        },
    ));

    // ── A PRANCHA (esquerda): comprida e fina, para a ATITUDE ser legível. É o
    // corpo em que os três modos de segurar mostram três coisas diferentes.
    crate_(
        world,
        "Plank",
        [-5.0, 1.2],
        [1.2, 0.12],
        1.0,
        BodyKind::Dynamic,
        LIME,
    );

    // ── O MURO MÓVEL (ponta esquerda): ESTÁTICO. Arrastá-lo tocando leva o
    // collider junto — o bug reportado. A bolinha em cima dele é a testemunha: ela
    // desce quando o muro desce.
    //
    // ⚠️ **Longe da prancha, e a sonda pagou por isto:** com o muro em −8,0 o
    // extremo esquerdo da prancha (−7,2) pousava EM CIMA dele, inclinado, e os
    // três modos de segurar mostravam duas atitudes iguais em vez de três
    // diferentes — o contato do muro decidia o ângulo, não a lei da mão.
    crate_(
        world,
        "Ledge",
        [-8.5, 1.0],
        [1.0, 0.2],
        1.0,
        BodyKind::Static,
        GREY,
    );
    ball(world, "Witness", [-8.5, 1.6], 0.25, 1.0, HOT);

    // ── A TORRE (meio): o alvo do estouro. Seis caixotes, para o espalhamento
    // ser óbvio (a lição da cena 30: um impacto sem consequência não mostra o
    // efeito).
    for i in 0..6u16 {
        crate_(
            world,
            &format!("Tower {}", i + 1),
            [0.0, 0.55 + f32::from(i) * 0.62],
            [0.3, 0.3],
            0.5,
            BodyKind::Dynamic,
            if i % 2 == 0 { HOT } else { COOL },
        );
    }

    // ── O ENXAME (direita): oito bolinhas espalhadas, o alvo do campo. Leves, e
    // separadas o bastante para "juntou" ser visível.
    for i in 0..8u16 {
        let col = f32::from(i % 4);
        let row = f32::from(i / 4);
        ball(
            world,
            &format!("Bit {}", i + 1),
            [5.0 + col * 1.1, 0.4 + row * 1.1],
            0.22,
            1.0,
            if i % 2 == 0 { COOL } else { LIME },
        );
    }
}

fn ball(world: &mut World, name: &str, at: [f32; 2], r: f32, density: f32, rgba: [f32; 4]) {
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Sprite::atlas(WHITE_TILE_KEY, [r * 2.0, r * 2.0], rgba),
        Name::new(name.to_string()),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: r },
            density,
            ..Collider::default()
        },
    ));
}

fn crate_(
    world: &mut World,
    name: &str,
    at: [f32; 2],
    half: [f32; 2],
    density: f32,
    kind: BodyKind,
    rgba: [f32; 4],
) {
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Sprite::atlas(WHITE_TILE_KEY, [half[0] * 2.0, half[1] * 2.0], rgba),
        Name::new(name.to_string()),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: half[0],
                half_y: half[1],
            },
            density,
            ..Collider::default()
        },
    ));
}

impl crate::App {
    /// **Cena 53 (W-Hand).** Quatro estações, TOCANDO, com o painel de física
    /// aberto.
    pub(crate) fn physics_smoke_interact(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_props(gfx.sim.world_mut());
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 53] A cena esta TOCANDO e o painel PHYSICS esta aberto (tecla W).\n  \
               Abra a secao INTERACTION -- e la que se escolhe o que o ponteiro faz.\n\n  \
               1. Aperte B (mostra os colliders). Com uma ferramenta de PONTO em maos o\n     \
                  cursor ganha um ANEL verde-limao apagado: e o alcance dela.\n  \
               2. HAND / Spring: pegue a PRANCHA (barra verde) pela PONTA e levante. Ela\n     \
                  BALANCA e vem atras do cursor -- e uma mola.\n     \
                  (medido: 0,445 rad de giro, ponto de pega a 0,119 m do cursor)\n     \
                  Stiffness governa o atraso num arrasto rapido (4 m/s): 50 -> 1,012 m,\n     \
                  400 (default) -> 0,369 m, 1600 -> 0,169 m, 6400 -> 0,069 m.\n     \
                  Damping e uma RAZAO: 1,00 e o critico e nao passa do cursor; 0,25 chega\n     \
                  antes (0,071 m de atraso) mas PASSA 0,132 m; 0,00 nunca assenta.\n  \
               3. HAND / Rigid: pegue a prancha pela ponta de novo. Ela sobe NIVELADA, sem\n     \
                  atraso e sem balancar -- na atitude que tinha.\n     \
                  (medido: 0,000 rad de giro, ponto de pega a 0,000 m do cursor)\n     \
                  ATENCAO, e o preco da palavra: um hold rigido ATRAVESSA parede. Tente\n     \
                  arrastar a prancha para dentro do muro cinza -- ela passa. So a mola\n     \
                  respeita geometria.\n  \
               4. HAND / Rope, Slack 1,5: a prancha fica PENDURADA e gira livre dentro da\n     \
                  coleira. Levante MENOS que 1,5 m e ela nao sai do chao (a corda esta\n     \
                  frouxa -- isso e a corda funcionando).\n     \
                  (medido levantando 2,5 m: -2,00 rad de giro, ponto de pega a 1,19 m)\n  \
               5. BLAST: escolha 'Blast' e clique no PE DA TORRE. Ela EXPLODE.\n     \
                  (medido: raio 3, impulso 10 -> 6 caixotes atingidos, e a pilha abre de\n      \
                   0,90 para 5,61 m de espalhamento medio)\n     \
                  Clique FORA do anel: 0 corpos, nada acontece. Radius desenha o anel,\n     \
                  Impulse e a forca no centro (o falloff pesa dai para a borda).\n  \
               6. PULL: escolha 'Pull' e SEGURE o botao sobre o ENXAME (direita). As oito\n     \
                  bolinhas se JUNTAM no cursor e FICAM la.\n     \
                  (medido: forca 50, raio 4 -> o raio medio da nuvem cai de 1,28 para\n      \
                   0,43 m em 1 s)\n     \
                  Force NEGATIVA repele: -20 abre a nuvem para 9,23 m em 1 s (elas saem\n     \
                  de quadro; -50 as manda a 14,63 m). Repelir espalha para longe -- e\n     \
                  forca sustentada, nao um estalo.\n  \
               7. O MURO MOVEL (ponta esquerda): 'Ledge' e ESTATICO e tem uma bolinha em\n     \
                  cima. Escolha Hand, selecione o muro e ARRASTE-O PARA BAIXO com o\n     \
                  relogio ANDANDO: o collider vai junto e a bolinha DESCE com ele.\n     \
                  (medido: descendo 0,80 m a testemunha desce 0,800 m)\n     \
                  Era o bug do collider fantasma: antes desta wave o desenho ia e o\n     \
                  collider ficava onde estava.\n  \
               8. Desmarque 'Physics' no transporte: nenhuma das tres ferramentas dispara\n     \
                  (sem passo de solver nada se move) e o anel de mira SOME.\n  \
               9. Arraste a regua para TRAS: nenhum cutucao volta -- eles nao estao no\n     \
                  documento, e a cena re-simula da pose autorada."
        );
    }
}

#[cfg(test)]
#[path = "physics_smoke_interact_tests.rs"]
mod tests;
