//! **Criar um joint onde se olha** — press no corpo A, arrasta, solta no B (W-J4).
//!
//! Criar um joint já era possível desde a W3, por SELEÇÃO: marque dois corpos,
//! aperte *Join Selected Bodies*. É a rota do Newton, ela fica, e é a rota da
//! corrente. O que faltava é a outra: **apontar**.
//!
//! A diferença não é de conveniência, é de ONDE AS ÂNCORAS NASCEM. Pela seleção
//! não há ponto nenhum a oferecer, então a política de semeadura os põe onde ela
//! sabe — o pivô autorado e, numa mola/corda, o **centro** do corpo B. Pelo
//! gesto há dois pontos, e eles são exatamente o que o artista quis dizer: a
//! mola pendura de onde você apertou até onde você soltou. (E o comprimento de
//! repouso dela é o do gesto — um número que ninguém precisa digitar.)
//!
//! # A recusa também é a feature
//!
//! Soltar fora de um corpo **não** cria um joint preso ao mundo: um `pin-to-world`
//! é outra coisa (o horizonte §8 do plano; o GDevelop o faz com um static
//! escondido) e inventá-lo aqui seria responder uma pergunta que ninguém fez. A
//! recusa vem com toast, e o gesto **segue armado** — o precedente é o eyedropper
//! do §12, que fica armado quando o clique não resolve.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::JointKind;

use crate::App;

/// O gesto em voo: de que corpo saiu, de que ponto, e onde o cursor está.
#[derive(Copy, Clone, Debug)]
pub(crate) struct JointDraw {
    pub(crate) body_a: Entity,
    /// O ponto do PRESS, em mundo — a âncora em A, e o pivô de um Pin/Weld.
    pub(crate) from: [f32; 2],
    /// Onde o cursor está agora, em mundo — a ponta da banda elástica.
    pub(crate) to: [f32; 2],
}

/// O corpo físico mais ao topo sob `world_pos`, ou `None`.
///
/// A MESMA leitura do eyedropper do §12: o pick de sprites, filtrado a quem tem
/// `RigidBody`. Um joint só pode nomear um corpo, então filtrar aqui é o que
/// torna isso verdade por construção em vez de por uma checagem que alguém tem
/// de lembrar de fazer depois.
#[must_use]
fn body_at(gfx: &mut crate::AppGfx, world_pos: [f32; 2]) -> Option<Entity> {
    ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), world_pos)
        .into_iter()
        .map(Entity::from_bits)
        .find(|&e| {
            gfx.sim
                .world()
                .get::<ph2d_physics_ecs::RigidBody>(e)
                .is_some()
        })
}

impl App {
    /// **A PORTA ÚNICA de desarmar** (W-J4b) — o botão, o Esc e qualquer futuro
    /// consumidor passam por aqui.
    ///
    /// Desarmar são DUAS coisas: o modo sai do ar **e** a banda em voo morre. Uma
    /// banda que sobrevivesse ao cancelamento desenharia um gesto que o artista
    /// acabou de recusar, e — pior — o `joint_draw.is_some()` do
    /// `input_dispatch` ainda tomaria o Move/Up seguinte, então o release criaria
    /// o joint que o Esc cancelou. Dois campos, um fato: quem os limpa é uma
    /// função, não dois call sites que precisam lembrar dos dois.
    pub(crate) fn disarm_joint_draw(&mut self) {
        disarm(&mut self.joint_draw_armed, &mut self.joint_draw);
    }

    /// **Esc cancela**, e só consome a tecla quando há o que cancelar — o formato
    /// da família de Escapes do shell (Build / Pen / shape do Painter), senão o
    /// Esc pararia de fazer blur de widget no resto do app.
    pub(crate) fn joint_draw_cancel_key(&mut self) -> bool {
        if !self.joint_draw_armed {
            return false;
        }
        self.disarm_joint_draw();
        self.any_input_this_frame = true;
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts
                .push(ph2d_editor::Toast::info("Joint drawing cancelled"));
        }
        true
    }

    /// **O press.** Com o gesto ARMADO, começa a banda no corpo sob o cursor.
    /// Devolve `true` se consumiu o evento.
    pub(crate) fn joint_draw_press(&mut self, sx: f32, sy: f32) -> bool {
        if !self.joint_draw_armed {
            return false;
        }
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let window = gfx.surface.size();
        let world = gfx.camera.screen_to_world((sx, sy), window);
        match body_at(gfx, world) {
            Some(body_a) => {
                self.joint_draw = Some(JointDraw {
                    body_a,
                    from: world,
                    to: world,
                });
            }
            // Nada sob o cursor: o gesto SEGUE armado (o precedente do
            // eyedropper), e o toast diz o que falta em vez de deixar o artista
            // concluir que o botão não fez nada.
            None => {
                gfx.toasts.push(ph2d_editor::Toast::info(
                    "Press ON a physics body to start the joint",
                ));
            }
        }
        true
    }

    /// **O arrasto.** Só move a ponta da banda; nada é autorado até o release.
    pub(crate) fn joint_draw_move(&mut self, sx: f32, sy: f32) {
        let Some(mut d) = self.joint_draw else {
            return;
        };
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        d.to = gfx.camera.screen_to_world((sx, sy), gfx.surface.size());
        self.joint_draw = Some(d);
    }

    /// **O release** — onde o joint nasce, ou onde a recusa é explicada.
    ///
    /// Um joint criado aqui vem com as âncoras NOS pontos do gesto
    /// (`create_joint_at`), e a mola/corda ganha de brinde o comprimento que o
    /// arrasto mediu: o gesto autora a geometria inteira, não só o par.
    pub(crate) fn joint_draw_release(&mut self, sx: f32, sy: f32) {
        let Some(d) = self.joint_draw.take() else {
            return;
        };
        let kind = crate::render_loop::inspector_joint::kind_of(self.join_kind);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window = gfx.surface.size();
        let world = gfx.camera.screen_to_world((sx, sy), window);
        let target = body_at(gfx, world);
        let refusal = match target {
            None => {
                Some("Release ON another body — a joint needs two (world pins are not a thing yet)")
            }
            Some(b) if b == d.body_a => Some("A joint binds two DIFFERENT bodies"),
            _ => None,
        };
        if let Some(msg) = refusal {
            gfx.toasts.push(ph2d_editor::Toast::info(msg));
            return; // segue armado: tente outra vez
        }
        let b = target.expect("refusal covered None");
        let Some(joint) = crate::render_loop::inspector_joint::create_joint_at(
            &mut gfx.sim,
            d.body_a.to_bits(),
            b.to_bits(),
            kind,
            Some((d.from, world)),
        ) else {
            gfx.toasts.push(ph2d_editor::Toast::info(
                "Those two bodies cannot be joined",
            ));
            return;
        };
        // O comprimento que o GESTO mediu. Uma mola criada arrastando 2 m
        // descansa a 2 m; uma corda tem 2 m de máximo. É o número que o §12
        // pediria e que o arrasto já disse — e é por isso que ele não é
        // digitado.
        let span = (world[0] - d.from[0]).hypot(world[1] - d.from[1]);
        if !kind.shares_a_point()
            && span > 1e-3
            && let Some(mut j) = gfx
                .sim
                .world_mut()
                .get_mut::<ph2d_physics_ecs::PhysicsJoint>(joint)
        {
            match kind {
                JointKind::Spring => j.rest_length = span,
                // Uma barra desenhada com 2 m de arrasto MEDE 2 m — o mesmo
                // campo da corda, porque engine-side é o mesmo número autorado.
                JointKind::Rope | JointKind::Rod => j.max_length = span,
                // Um Slider compartilha um ponto, então nunca chega aqui — mas o
                // arrasto DIZ algo para ele, e é o eixo (logo abaixo).
                JointKind::Pin | JointKind::Weld | JointKind::Slider => {}
            }
        }
        // **O arrasto DESENHA O TRILHO.** Para um Slider o gesto não mede um
        // comprimento (ele compartilha um ponto), mede uma DIREÇÃO: o rumo do
        // press até o release é o eixo, escrito na rotação da entidade-joint —
        // que é onde o eixo mora (`JointKind::Slider`). Sem isto, desenhar um
        // trilho na diagonal criava um trilho horizontal e o artista teria de ir
        // digitar o ângulo, que é exatamente o passo que este gesto existe para
        // remover.
        if kind == JointKind::Slider
            && span > 1e-3
            && let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(joint)
        {
            t.rotation = libm::atan2f(world[1] - d.from[1], world[0] - d.from[0]);
        }
        // O gesto terminou: desarma, e SELECIONA o joint novo — a §12 abre no
        // que você acabou de desenhar, exatamente como no botão (W-JointCreate).
        self.joint_draw_armed = false;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.gizmo.selection = Some(joint.to_bits());
            hero.gizmo.extra_selection.clear();
        }
    }
}

/// **Desarmar são DUAS coisas** — o modo sai do ar e a banda em voo morre.
///
/// Função LIVRE sobre os dois campos, não método: o sítio de ação da
/// `render_loop` tem o `gfx` emprestado de dentro do `self`, então um
/// `&mut self` ali é E0499 — a mesma razão pela qual o `join_chain` é livre. É
/// esta função (e não dois call sites) que sabe que o desarme tem duas metades.
pub(crate) fn disarm(armed: &mut bool, draw: &mut Option<JointDraw>) {
    *armed = false;
    *draw = None;
}

/// **O botão é um TOGGLE** — apertar armado cancela, pela porta acima.
///
/// A alternativa (só armar) deixava o artista sem saída: o gesto é modal e come o
/// press no canvas, então uma vez armado o único jeito de sair era completar um
/// joint que ele não queria (Enio, smoke da W-J4).
pub(crate) fn toggle(armed: &mut bool, draw: &mut Option<JointDraw>) {
    if *armed {
        disarm(armed, draw);
    } else {
        *armed = true;
    }
}

/// **A CORRENTE** (P9): N corpos, em ordem, viram N−1 joints. Devolve
/// `(quantos, o último)`.
///
/// A rota por seleção sempre soube ligar DOIS; a corrente é a razão de ela
/// sobreviver ao gesto de desenhar — sete elos à mão são sete gestos, marcá-los
/// e apertar um botão é um. A ordem é a da SELEÇÃO (primário primeiro, extras na
/// ordem em que entraram), que é o que o artista construiu clicando.
///
/// ⚠️ **Um passo de undo, e de graça:** os N−1 spawns caem no MESMO frame, e o
/// undo global é por DIFF de fim de frame — ele vê um estado, não N operações.
/// Nada aqui abre bracket.
///
/// ⚠️ **Função livre sobre `&mut SimWorld`**, não método de `App`: o laço de
/// ações do `render_loop` já tem o `sim` destruturado do `AppGfx`, então um
/// `&mut self` ali não compila — e livre ela é gateável headless.
pub(crate) fn join_chain(
    sim: &mut SimWorld,
    order: &[u64],
    kind: JointKind,
) -> (usize, Option<Entity>) {
    let mut made = 0;
    let mut last = None;
    for pair in order.windows(2) {
        if let Some(j) =
            crate::render_loop::inspector_joint::create_joint(sim, pair[0], pair[1], kind)
        {
            made += 1;
            last = Some(j);
        }
    }
    (made, last)
}

/// A âncora do gesto em voo, para a banda elástica do overlay: `(de, para)` em
/// mundo, ou `None` sem gesto.
#[must_use]
pub(crate) fn band(draw: Option<JointDraw>) -> Option<([f32; 2], [f32; 2])> {
    draw.map(|d| (d.from, d.to))
}

/// O corpo A do gesto ainda existe? (Um corpo apagado sob o gesto o invalida.)
#[must_use]
pub(crate) fn body_alive(sim: &SimWorld, draw: Option<JointDraw>) -> bool {
    draw.is_none_or(|d| sim.world().get::<Transform>(d.body_a).is_some())
}

#[cfg(test)]
#[path = "joint_draw_tests.rs"]
mod tests;
