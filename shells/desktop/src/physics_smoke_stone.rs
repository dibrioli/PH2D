//! **A cena 107 — A PEDRA ESTREITA** (`W-ShapeCast`), irmã de
//! `physics_smoke_player_crouch.rs`.
//!
//! Uma fileira de pedras de **8 cm** penduradas à altura da cabeça, e uma laje
//! larga no fim como CONTROLE. O gesto que a cena existe para julgar é
//! **soltar o agachar debaixo de uma pedra estreita** — e a resposta tem de ser
//! a mesma que debaixo da laje: ele fica agachado.
//!
//! # ⚠️ Por que ESTREITA, e por que várias
//!
//! O sensor do agachar lia o teto com **três raios** nascidos em
//! `−0,20 · 0,00 · +0,20` do centro do corpo — a caixa envolvente mede
//! exactamente 0,20 de meia-largura. Uma pedra de 8 cm cabe **inteira** no vão
//! entre duas dessas amostras, e o preço estava medido: a cabeça subia a
//! **1,267** contra uma face de pedra em **1,25**, com o solver a segurá-la lá
//! dentro.
//!
//! São várias e em posições diferentes porque a grade acompanhava o CORPO: o vão
//! cego movia-se com o personagem, então uma pedra só teria uma janela de `x` em
//! que o defeito aparecia. Com a varredura de corpo **nenhuma** delas deixa
//! passar, e é isso que a fileira torna visível de uma vez.
//!
//! # ⚠️ As alturas são ARITMÉTICA da cápsula
//!
//! | pose | centro | topo |
//! |---|---|---|
//! | de pé | 0,90 | **1,40** |
//! | agachado | 0,55 | **1,05** |
//!
//! A face de baixo das pedras fica em **1,20** — entre os dois, os mesmos
//! 0,15 m de folga para cada lado que o corredor da cena 94 usa. Mais alta e ele
//! passaria de pé (a cena mostraria uma capacidade que não é esta); mais baixa e
//! nem agachado caberia.
//!
//! # ⚠️ O que esta cena NÃO mostra, e porquê
//!
//! A varredura devolveu uma segunda coisa: a **quina da caixa** deixou de ser um
//! teto (a cápsula ali alcança menos que a caixa dela), o que vale **18 cm** de
//! altura de teto medidos. Isso **não é julgável de olho** — não há
//! contrafactual na tela, o artista veria apenas um personagem a levantar-se
//! normalmente. Fica **gateado** (`the_capsule_corner_is_no_longer_a_ceiling`) em
//! vez de encenado, porque uma estação que não se pode julgar é uma que se
//! aprova por cansaço.

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
/// A face de baixo de toda pedra desta cena — entre o topo de pé (1,40) e o
/// agachado (1,05).
pub(crate) const STONE_BOTTOM: f32 = 1.20;
/// Meia-largura de uma pedra ESTREITA: 8 cm de pedra, que cabem inteiros no vão
/// de 0,20 entre duas amostras do sensor antigo.
pub(crate) const NARROW_HALF: f32 = 0.04;
/// Onde cada pedra estreita está.
pub(crate) const NARROW_X: [f32; 4] = [6.0, 10.0, 14.0, 18.0];
/// Onde a laje LARGA de controle começa.
pub(crate) const WIDE_X: f32 = 24.0;

impl App {
    /// **A pedra estreita** — o teto que não cabe num raio.
    pub(crate) fn physics_smoke_stone(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = build_stone_scene(gfx.sim.world_mut());
        eprintln!("{STONE_SMOKE_MESSAGE}");
    }
}

/// **A geometria da cena 107**, separada do `App` de propósito.
///
/// ⚠️ **É esta função que os gates dirigem**, e não uma reconstrução deles: a
/// mensagem manda o artista fazer dois gestos, e a única forma de os afirmar
/// antes de ele os ler é correr a MESMA cena pela ponte real. Uma segunda cópia
/// da geometria seria verde sobre uma cena que ninguém abre.
pub(crate) fn build_stone_scene(world: &mut bevy_ecs::world::World) -> ph2d_ecs::Entity {
    {
        slab(
            world,
            "Floor",
            Vec2::new(16.0, -0.5),
            [34.0, 0.5],
            0.0,
            [0.35, 0.35, 0.4, 1.0],
        );

        for (i, x) in NARROW_X.iter().enumerate() {
            stone(
                world,
                &format!("Stone {}", i + 1),
                *x,
                NARROW_HALF,
                [0.46, 0.30, 0.32, 1.0],
            );
        }

        // O CONTROLE: a mesma pergunta sobre uma laje que qualquer raio via.
        stone(
            world,
            "Wide Slab",
            WIDE_X + 3.0,
            3.0,
            [0.42, 0.32, 0.34, 1.0],
        );

        // Uma parede no fim, para o personagem não sair de quadro a correr.
        slab(
            world,
            "Backstop",
            Vec2::new(34.5, 1.5),
            [0.5, 2.0],
            0.0,
            [0.30, 0.34, 0.42, 1.0],
        );

        let player = spawn_player(world, Vec2::new(0.0, 1.4));

        // ⚠️ **A capacidade é ARMADA aqui**, e não herdada: o `PlatformPlayer`
        // nasce com o agachar desligado, e uma cena que herdasse o default
        // mostraria pedras debaixo das quais nada acontece.
        let mut q = world.query::<(&mut PlatformPlayer, &Transform)>();
        for (mut p, _) in q.iter_mut(world) {
            p.crouch_height = CROUCH_HEIGHT;
            p.crouch_speed = CROUCH_SPEED;
        }
        player
    }
}

/// Uma pedra pendurada, com a face de baixo em [`STONE_BOTTOM`].
fn stone(world: &mut bevy_ecs::world::World, name: &str, x: f32, half_x: f32, tint: [f32; 4]) {
    const HALF_Y: f32 = 1.0;
    slab(
        world,
        name,
        Vec2::new(x, STONE_BOTTOM + HALF_Y),
        [half_x, HALF_Y],
        0.0,
        tint,
    );
}

/// O roteiro da cena 107 — o gesto é SOLTAR O AGACHAR debaixo de uma pedra que
/// não cabe num raio.
pub(crate) const STONE_SMOKE_MESSAGE: &str = concat!(
    "[physics-smoke 107] A PEDRA ESTREITA (W-ShapeCast). Quatro pedras de 8 cm\n",
    "penduradas com a face de baixo em 1.20 m (o topo do personagem de pe' mede\n",
    "1.40, agachado 1.05), e uma laje LARGA no fim como controle.\n",
    "\n",
    "⚠️ Se a linha acima nao aparecer, pare: a cena nao montou.\n",
    "\n",
    "CONTROLE: setas <- / -> (ou A / D) andam. SETA PARA BAIXO (ou S) agacha.\n",
    "\n",
    "O QUE JULGAR, nesta ordem:\n",
    " 1. Marque Physics no transporte e de Play.\n",
    " 2. Ande para a direita. De pe' ele BATE na primeira pedra -- ela esta' a\n",
    "    altura da cabeca dele.\n",
    " 3. Segure BAIXO e passe por baixo dela.\n",
    " 4. PARE debaixo de uma pedra e solte o botao: ele NAO se levanta. Ande\n",
    "    alguns passos e experimente debaixo de cada uma das quatro -- a recusa\n",
    "    tem de acontecer em TODAS, e nao so' nalgumas.\n",
    " 5. ENTRE duas pedras, solte o botao: ali ele levanta-se na hora. E' isso\n",
    "    que separa 'o sensor ve' a pedra' de 'o sensor recusa sempre'.\n",
    " 6. Va' ate' a laje larga e repita o passo 4: a resposta e' a MESMA. Ela e' o\n",
    "    controle -- se a laje se comportasse diferente da pedra, o sensor estaria\n",
    "    a medir o tamanho do teto e nao o corpo.\n",
    "\n",
    "O QUE ISTO CORRIGE: ate' esta wave o sensor lia o teto com tres raios, e uma\n",
    "pedra de 8 cm cabia inteira no vao entre duas amostras. Medido, a cabeca\n",
    "subia a 1.267 contra uma face de pedra em 1.25 -- ele levantava-se PARA\n",
    "DENTRO da rocha, e o solver segurava-o la'.\n",
);

#[cfg(test)]
#[path = "physics_smoke_stone_tests.rs"]
mod tests;
