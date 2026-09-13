//! **Fase do quadro: A PELE E A FERRAMENTA DO OSSO NO PAINEL** — se a selecção tem forma PRESA ou imagem com pele, a ferramenta do osso e a deformação,
//! publicadas no painel do esqueleto (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_selection_mirror_skin(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);
        // ⭐⭐⭐ **O ESQUELETO** (estudo 42 item 5): as duas perguntas que só a shell
        // responde — *"a selecção tem forma PRESA?"* (decide se as saídas são oferecidas) e
        // *"o que está aceso é um osso, e com que números?"* (decide os dois campos).
        //
        // ⚠️ A 2ª passa pela MESMA porta que o gesto e o overlay usam
        // (`bone_gesture::selected_bone`): QUATRO consumidores, uma resposta — o dedo,
        // o dreno dos verbos, este painel e o desenho do overlay (o quarto entrou na wave
        // do gizmo de limite, e esta conta ficou em três até 2026-09-08).
        let presa = self.vec.pen.selected_paths().iter().any(|id| {
            self.vec.entities.get(id).is_some_and(|&b| {
                sim.world()
                    .get::<ph2d_skeleton_ecs::SkinBind>(ph2d_ecs::Entity::from_bits(b))
                    .is_some()
            })
        });
        ph2d_panel_skeleton::set_current_skinned(presa);
        // ⭐ E a pergunta da fileira *Deform*, que é OUTRA: ela é sobre a CENA, porque a
        // escolha é global. ⛔ Varrer `selected_paths` aqui não a responderia — uma imagem
        // presa é uma sprite, e nunca aparece naquela lista.
        ph2d_panel_skeleton::set_current_skinned_image(
            sim.world()
                .iter_entities()
                .any(|er| crate::render_loop::sim_extract::skinned_image(sim.world(), er.id())),
        );
        // E se a CENA tem esqueleto — é isso que faz a seção aparecer (ou não) fora do modo
        // Osso. ⛔ Sem esta metade ela seria um cabeçalho permanente num app que nunca viu
        // um osso, que é exactamente o report que a tabela de escopo curou em 31/08.
        // ⭐⭐⭐ **QUEM ABRE O PAINEL DE BONES** (ordem do dono, 2026-09-09).
        //
        // ⛔⛔ **A visibilidade deixou de ser DERIVADA da cena.** Enquanto ela era
        // `tem_esqueleto || ferramenta_osso`, escrita em TODO quadro, o menu *Window →
        // Bones* seria um interruptor morto — o quadro seguinte repunha a decisão da shell
        // por cima da do artista. *Duas fontes de verdade para o mesmo bool, e a que o
        // artista toca é a que perde.*
        //
        // ⇒ ficam **duas portas, as duas de ARESTA**: a linha do menu (o `skeleton_toggle`)
        // e a selecção de um osso (mais abaixo). Nenhuma das duas escreve em todo quadro.
        let ferramenta_osso = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bone;
        // ⭐ E o VERBO do arrasto, como ÍNDICE — é o que mantém aquele painel sem depender
        // da crate da ferramenta de vector.
        ph2d_panel_skeleton::set_current_bone_tool(ferramenta_osso.then(|| {
            usize::from(self.vec.draw_config.bone_action == ph2d_tool_vector::BoneAction::Transform)
        }));
        // ⭐ E COMO a pele é desenhada (report das arestas retas, 2026-09-10) — também como
        // ÍNDICE, e ⛔ sem `Option`: esta pergunta tem sempre resposta.
        ph2d_panel_skeleton::set_current_skin_deform(usize::from(
            self.vec.draw_config.skin_deform == ph2d_tool_vector::SkinDeform::Smooth,
        ));
    }
}
