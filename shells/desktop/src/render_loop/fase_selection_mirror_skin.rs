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
        let FrameGfx {
            sim, hero_screen, ..
        } = FrameGfx::of(gfx);
        // ⭐⭐⭐ **O ESQUELETO** (estudo 42 item 5): as duas perguntas que só a shell
        // responde — *"a selecção tem forma PRESA?"* (decide se as saídas são oferecidas) e
        // *"o que está aceso é um osso, e com que números?"* (decide os dois campos).
        //
        // ⚠️ A 2ª passa pela MESMA porta que o gesto e o overlay usam
        // (`bone_gesture::selected_bone`): QUATRO consumidores, uma resposta — o dedo,
        // o dreno dos verbos, este painel e o desenho do overlay (o quarto entrou na wave
        // do gizmo de limite, e esta conta ficou em três até 2026-09-08).
        let vector = self.vec.pen.selected_paths().iter().any(|id| {
            self.vec.entities.get(id).is_some_and(|&b| {
                sim.world()
                    .get::<ph2d_skeleton_ecs::SkinBind>(ph2d_ecs::Entity::from_bits(b))
                    .is_some()
            })
        });
        // ⭐⭐⭐ **E A IMAGEM PRESA, que este cabeçalho já prometia e o código não lia** (report do
        // dono, 2026-09-18: *«ainda não temos a opção de desconectar a malha do osso»*). Sem esta
        // metade, com uma imagem presa escolhida o painel lia `false` e os botões de saída **nem
        // eram pintados** — *o artista não via um botão morto, via a ausência de um botão*.
        //
        // ⚠️ **O sujeito de uma imagem é a SELECÇÃO DO GIZMO**, nunca a lista de caminhos do pen —
        // são duas famílias com dois selectores, e é exactamente a nota que o verbo do *Bind* já
        // carrega por escrito desde a wave da 2.ª mídia.
        let imagem = hero_screen.as_ref().is_some_and(|h| {
            h.gizmo.iter_selected().any(|b| {
                ph2d_ecs::Entity::try_from_bits(b).is_some_and(|e| {
                    ph2d_skeleton_live::skin_image::is_skinned_image(sim.world(), e)
                })
            })
        });
        ph2d_panel_skeleton::set_current_skinned(ph2d_panel_skeleton::Skinned { vector, imagem });
        // ⭐⭐⭐ **E SE O ENVELOPE AINDA MANDA EM ALGUMA COISA** (report do dono, 2026-09-18). A lei é
        // pura e vive na crate; aqui só se publica o que ela responde.
        //
        // ⚠️ **A pergunta é da CENA e não da selecção**, porque o `SkinBind` guarda a malha e os
        // pesos e **não** os ossos — logo *«este esqueleto tem forma vectorial?»* não é derivável.
        // A pergunta mais larga erra para o lado conservador: o campo fica à vista enquanto houver
        // uma forma vectorial presa em qualquer sítio.
        ph2d_panel_skeleton::set_current_envelope_manda(
            ph2d_skeleton_live::esqueletos::ha_forma_vectorial_presa(sim),
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
    }
}
