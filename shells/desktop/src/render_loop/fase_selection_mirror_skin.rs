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
        // ⭐⭐⭐ **OS DOIS SELECTORES, POR UMA PORTA SÓ** — e a cura é do report de 2026-09-19.
        //
        // ⛔⛔⛔ **A lente do painel era mais ESTREITA que o sujeito, e o defeito era MUDO:** o
        // `vector` saía só da lista de caminhos do pen e o `imagem` só do gizmo. ⚠️ **Mas clicar
        // numa linha da HIERARQUIA escreve no GIZMO** (`hero.gizmo.replace_selection`), nunca no
        // pen — logo uma FORMA vectorial escolhida ali lia `{vector: false, imagem: false}`, e o
        // *Expand*, o *Release* e a fileira `Deform By` **nem chegavam a ser pintados**. *O artista
        // não vê um botão morto: vê a ausência de um botão.*
        //
        // ⚠️ É a MESMA forma do report de 2026-09-18 (*«ainda não temos a opção de desconectar a
        // malha do osso»*), que já custou os dois botões de saída — ali a lente não via a imagem,
        // aqui não vê o segundo selector. ⇒ a pergunta passa pela porta da família
        // ([`ph2d_app_skeleton::skin_law::escolhidas`]), que é a **mesma** que o dreno do chip usa:
        // *duas respostas à mesma pergunta divergem no primeiro ajuste.*
        let caminhos: Vec<u64> = self
            .vec
            .pen
            .selected_paths()
            .iter()
            .filter_map(|id| self.vec.entities.get(id).copied())
            .collect();
        let gizmo: Vec<u64> = hero_screen
            .as_ref()
            .map(|h| h.gizmo.iter_selected().collect())
            .unwrap_or_default();
        let presas = ph2d_app_skeleton::skin_law::escolhidas(
            sim,
            caminhos.iter().copied(),
            gizmo.iter().copied(),
        );
        // ⚠️ **A partição é a MÍDIA e não o selector**: o *Expand* não alcança uma imagem (ela não
        // tem geometria autorada), e é essa a única pergunta que ainda precisa de as separar.
        let imagem = presas
            .iter()
            .any(|&e| ph2d_skeleton_live::skin_image::is_skinned_image(sim.world(), e));
        let vector = presas
            .iter()
            .any(|&e| !ph2d_skeleton_live::skin_image::is_skinned_image(sim.world(), e));
        ph2d_panel_skeleton::set_current_skinned(ph2d_panel_skeleton::Skinned { vector, imagem });
        // ⭐⭐⭐ **E POR QUE LEI A SELECÇÃO SE DEFORMA** (ordem do dono, 2026-09-19: *«construa. por
        // desenho»*) — o que a fileira `Deform By` acende.
        ph2d_panel_skeleton::set_current_skin_law_envelope(
            ph2d_app_skeleton::skin_law::alguma_por_alcance(sim, caminhos, gizmo),
        );
        // ⭐⭐⭐ **E SE O ENVELOPE AINDA MANDA EM ALGUMA COISA** (report do dono, 2026-09-18). A lei é
        // pura e vive na crate; aqui só se publica o que ela responde.
        //
        // ⚠️ **A pergunta é da CENA e não da selecção**, porque o `SkinBind` guarda a malha e os
        // pesos e **não** os ossos — logo *«este esqueleto tem forma vectorial?»* não é derivável.
        // A pergunta mais larga erra para o lado conservador: o campo fica à vista enquanto houver
        // uma forma vectorial presa em qualquer sítio.
        // ⚠️ **POR OSSO e não por cena** — a 1.ª redacção perguntava à cena inteira, e numa cena
        // MISTA (imagens de um lado, formas vectoriais do outro) ela acendia o campo nos dois. A
        // premissa que a justificava era minha e caiu: o `SkinBind` **guarda** os ossos, em
        // `Tendon::bone`.
        let osso_do_painel = hero_screen
            .as_ref()
            .and_then(|h| crate::bone_gesture::selected_bone(sim, h.gizmo.iter_selected()))
            .and_then(ph2d_ecs::Entity::try_from_bits);
        ph2d_panel_skeleton::set_current_envelope_manda(
            osso_do_painel.is_none_or(|e| {
                ph2d_skeleton_live::esqueletos::o_envelope_deste_osso_manda(sim, e)
            }),
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
        //
        // ⛔⛔ **O índice sai da PORTA ([`BoneAction::indice`]) e não de uma comparação.** Ele era
        // `usize::from(acao == Transform)`, que está certo com DOIS verbos e mente em silêncio com
        // três: o terceiro lia `0` e acendia o primeiro segmento. *Um índice derivado de uma
        // comparação é uma tabela escrita à mão com outra sintaxe.*
        ph2d_panel_skeleton::set_current_bone_tool(
            ferramenta_osso.then(|| self.vec.draw_config.bone_action.indice()),
        );
        // ⭐ E os dois números do PINCEL DE PESO — publicados SEMPRE, porque o sujeito deles é a
        // ferramenta: é a secção que decide se os pinta, e ela só o faz com o verbo armado.
        ph2d_panel_skeleton::set_current_bone_weight(
            self.vec.draw_config.weight_radius,
            self.vec.draw_config.weight_amount,
        );
    }
}
