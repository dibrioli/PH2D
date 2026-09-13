//! **Fase do quadro: A MOLDURA, O LAYOUT, O Z E AS ÂNCORAS DA SELECÇÃO** — o que o painel vectorial honra e
//! publica sobre a selecção: a moldura, o auto layout, o z, as âncoras e os componentes (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! **Corte um nível abaixo:** o bloco do painel vectorial é UM statement de 467 linhas; esta fase é um grupo dos statements do CORPO dele, e a chamada fica dentro do bloco, no sítio do corpo.

use super::*;

/// Os pedidos de moldura, layout, z e âncoras que o dreno do barramento recolheu neste quadro.
pub(super) struct FrameLayoutIntents {
    pub(super) pending_frame_clip: Option<bool>,
    pub(super) pending_layout_edit: Option<crate::vec_layout_edit::LayoutEdit>,
    pub(super) pending_anchor_edit: Option<crate::vec_anchor_edit::AnchorEdit>,
    pub(super) pending_layout_field: Option<(crate::vec_layout_edit::LayoutField, f64)>,
    pub(super) pending_vec_z: Option<f64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_selection_frame_panel(
        &mut self,
        intents: FrameLayoutIntents,
    ) -> Option<Vec<ph2d_vec_scene::VecPathId>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let FrameLayoutIntents {
            pending_frame_clip,
            pending_layout_edit,
            pending_anchor_edit,
            pending_layout_field,
            pending_vec_z,
        } = intents;
        let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
        // **A MOLDURA da seleção** (plano UI/UX W0): honra o chip e publica o estado. O
        // clique é honrado ANTES da publicação para o painel mostrar, no mesmo frame, o
        // valor que o artista acabou de escolher — publicar primeiro deixaria o chip a
        // piscar de volta ao valor antigo por um quadro.
        //
        // ⚠️ **O RECORTE deixou de ser a moldura** (2026-08-21): o chip vale para
        // qualquer forma FECHADA, então o sujeito sai do `vec_clip_edit` — que precisa da
        // cena para ler o `closed`, coisa que a pergunta da moldura nunca precisou.
        if let Some(clip) = pending_frame_clip {
            crate::vec_clip_edit::set_selected_clip(sim, vec_scene, &self.vec.entities, &sel, clip);
        }
        ph2d_panel_vector::state::set_frame_clip(crate::vec_clip_edit::selected_clip(
            sim,
            vec_scene,
            &self.vec.entities,
            &sel,
        ));
        // ⚠️ **A outra metade, e ela tem outro sujeito.** A seção Frame (Show as Panel +
        // presets de dispositivo) pergunta pela MOLDURA, que continua a sair do
        // `frame_of_selection` — publicar o recorte para as duas ofereceria os presets de
        // telefone sobre uma elipse que o artista mandou recortar.
        ph2d_panel_vector::state::set_frame_present(
            crate::vec_frame_edit::frame_of_selection(sim, &self.vec.entities, &sel).is_some(),
        );
        // ⚠️ O chip *Show as Panel* le' a visibilidade REAL do painel autorado, e nao uma
        // copia: o X do painel escreve o MESMO mapa, entao fechar por la' apaga o chip
        // sozinho. Um bool proprio aqui seria a segunda resposta que fica acesa.
        ph2d_panel_vector::state::set_frame_panel_open(
            hero.is_panel_visible(ph2d_panel_authored::visibility_key()),
        );
        // **O AUTO LAYOUT** (plano UI/UX W2) — honra o clique ANTES de publicar, pela
        // mesma razao do recorte acima: publicar primeiro deixaria o chip a piscar de
        // volta ao valor antigo por um quadro.
        if let Some(e) = pending_layout_edit {
            crate::vec_layout_edit::apply_layout_edit(sim, &self.vec.entities, &sel, e);
        }
        if let Some((f, v)) = pending_layout_field {
            // ⚠️ **A VOLTA da fronteira de display, e ela pergunta ao TIPO.** No mesmo
            // painel viajam três naturezas de número: o vão / o recuo / os limites são
            // comprimentos, `Columns` é uma CONTAGEM e `Grow`/`Shrink` são razões do
            // flexbox. Converter os três dividiria "três colunas" por cem, em silêncio —
            // todos são `f64`. Quem responde é `LayoutField::is_length`, cujo `match` o
            // compilador cobra quando uma variante nova entra.
            let v = if f.is_length() {
                ph2d_editor_core::LengthDisplay::of(&hero.project).to_world(v)
            } else {
                v
            };
            crate::vec_layout_edit::apply_layout_field(sim, &self.vec.entities, &sel, f, v);
        }
        // **O Z-INDEX**, honrado ANTES de publicar (a ordem dos vizinhos): publicar
        // primeiro deixaria o campo a mostrar o valor ANTERIOR por um quadro.
        if let Some(v) = pending_vec_z
            && let Some(id) = sel.first()
        {
            ph2d_vec_entities::entities::zorder::set_authored_z(
                sim,
                &self.vec.entities,
                *id,
                // O clamp e' o do COMPONENTE, e mora na porta que escreve — nao no widget.
                v.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
            );
        }
        // AS ÂNCORAS (plano UI/UX W3): aplicar, e só depois publicar — publicar antes
        // deixaria o chip a mostrar a regra ANTERIOR por um frame, e o artista veria a
        // escolha "não pegar" (a mesma ordem que os tokens abaixo).
        if let Some(e) = pending_anchor_edit {
            crate::vec_anchor_edit::apply_anchor_edit(sim, vec_scene, &self.vec.entities, &sel, e);
        }
        ph2d_panel_vector::state::set_anchor_state(crate::vec_anchor_edit::selected_anchors(
            sim,
            &self.vec.entities,
            &sel,
        ));
        // OS COMPONENTES (plano UI/UX W5): publicar DEPOIS de o produtor ter cozido —
        // é dele que vem a resposta *"esta instância está órfã?"*, e perguntá-la aqui
        // outra vez seria a segunda porta.
        //
        // ⭐⭐⭐ **UM motor, e por isso UMA leitura** (F4.6c fechada, 2026-09-07). Aqui
        // viveu o outro lado do interruptor — sem a env var a secção descrevia o
        // `VecInstance` — e com ele morreram as DUAS listas que só aquele motor enchia:
        // as PEÇAS e os VARIANTS da instância vetorial.
        //
        // ⚠️ **Elas já saíam vazias no caminho de omissão desde 2026-09-06, por
        // DECLARAÇÃO** — o `VecInstance` era componente registado e viajava na cópia
        // profunda do *Make*, então uma leitura ingénua enchia-as e o painel pintava
        // controlos que o dreno geral recusava **em silêncio**. Apagar o produtor apaga a
        // pergunta: *o que não existe não precisa de ser publicado vazio.*
        //
        // ⚠️ **As duas capacidades NÃO se perderam, e a régua é anterior a este corte:**
        // esconder uma peça de UMA cópia e pintá-la são hoje o `Visibility`/`Sprite` da
        // própria peça (`ph2d_app_components::instance_structure::instance_piece_override_tests`, escrito como
        // pré-condição desta fatia), e os variants vivem no cartão do Inspector (F5).
        ph2d_panel_vector::state::set_component_state(crate::vec_component_general::state_of(
            sim,
            &self.vec.entities,
            &sel,
            // ⭐ **O objecto único na mão** — é o que faz a secção aparecer sobre um
            // GRUPO, que não é path nenhum. Ver [`vec_component_general::subject_of`].
            (hero.gizmo.selected_len() == 1)
                .then_some(hero.gizmo.selection)
                .flatten(),
            // ⚠️ O rótulo do botão troca enquanto o gesto de duas mãos está aberto, e
            // é ele que diz ao artista que o app está à espera do segundo clique.
            matches!(
                self.vec.path_pick,
                Some(crate::vec_pick::PathPick::InstanceMain(_))
            ),
        ));
        Some(sel)
    }
}
