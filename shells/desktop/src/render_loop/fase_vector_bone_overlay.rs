//! **Fase do quadro: O OVERLAY DOS OSSOS** (estudo 42 item 5) — os ossos, as articulações e o realce de
//! pele desenhados sobre a câmera do quadro enquanto o plano de overlay o pede (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_bone_overlay(
        &mut self,
        vec_px_to_world: f64,
        cam_affine: ph2d_vector::Affine,
        overlay: ph2d_app_vec::overlay::VecOverlayPlan,
    ) -> Option<ph2d_vector::Affine> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vector_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // ⭐⭐⭐ **OS OSSOS** (estudo 42 item 5): desenhados enquanto a ferramenta de VETOR está
        // na mão, e só então.
        //
        // ⚠️ **FORA do `overlay.edit`, e é a mesma razão da linha de corte abaixo:** aquele
        // portão fecha em Select/Build/Bucket porque *âncoras* ali são ruído — mas um osso não
        // é uma âncora, é o corpo do rig. Escondê-lo no Select tiraria da tela a única coisa
        // que diz onde o esqueleto está enquanto se mexe nas formas dele.
        //
        // ⛔ Não é uma forma da cena: nada disto entra no documento, no SVG ou no z-order.
        if overlay.bones {
            // ⭐⭐ **O CORPO, não as duas pontas** — desde a F8 um osso pode DOBRAR, e o que se
            // desenha é a polilinha dele. ⛔ Um osso recto devolve dois nós e sai byte a byte como
            // saía; e quem o AGARRA (`bone_pick`) lê a mesma porta, senão ele seria um controlo
            // morto sob o dedo.
            let ossos = crate::skeleton_live::bone_polylines(sim);
            if !ossos.is_empty() {
                // ⭐⭐⭐ **A REGIÃO DE INFLUÊNCIA do osso em foco** — o *Bone Strength* do Moho.
                // Ela entra ANTES dos ossos: é um fundo, e o rig desenha-se por cima dela.
                //
                // ⚠️ **O foco é a SELECÇÃO, e é a mesma pergunta que o dedo faz** — a alça só é
                // agarrável onde ela é pintada (`bone_pick::hover` recebe o mesmo `foco`).
                //
                // ⚠️ A selecção CRUA basta e filtra-se sozinha: `influence_region` devolve
                // `None` para o que não é osso, então não há aqui uma segunda pergunta
                // *"isto é um osso?"* a divergir da que o `hover` faz.
                // ⭐⭐⭐ **O OSSO EM FOCO SAI DA MESMA PORTA QUE O DEDO USA**
                // ([`crate::bone_gesture::selected_bone`]), e não do primário do gizmo.
                //
                // ⛔⛔ **Eram DUAS respostas para «qual osso está em foco», e o doc de uma delas
                // afirmava serem a mesma.** O dedo lê a selecção INTEIRA (o `selected_bone`
                // explica porquê: prender uma forma a um esqueleto entre vários faz-se
                // escolhendo os dois, e aí **o primário é a forma**); o desenho lia só o
                // primário. ⇒ com uma forma seleccionada ao lado do osso, o dedo oferecia as
                // alças de um osso e o canvas pintava-as noutro sítio — ou em sítio nenhum.
                // Report do dono (2026-09-08): *«gizmo não mantém ângulo fixo em relação ao
                // osso»*.
                //
                // ⚠️ O doc do `selected_bone_bits` já prescrevia isto: *«no laço de desenho o
                // `gfx` está emprestado mutável de ponta a ponta, e ali chama-se a função livre
                // acima — a lei é a mesma, e é por isso que ela vive numa função só»*.
                let osso_focado =
                    crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected());
                ph2d_skeleton_render::draw_influence(
                    osso_focado.and_then(|b| crate::skeleton_live::influence_region(sim, b)),
                    matches!(
                        self.skeleton.bone_hover,
                        Some(h) if h.part == ph2d_skeleton_render::BonePart::Influence
                    ),
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
                // ⭐⭐⭐ **O ARCO DE LIMITE do osso em foco** — o setor por onde a ponta dele
                // pode passar, mais as duas paredes agarráveis.
                //
                // ⚠️ **Depois da influência e ANTES dos ossos**: os dois são fundo, e o arco
                // vive por cima da mancha (é por isso que o véu dele é mais fraco). O rig
                // desenha-se por cima dos dois.
                //
                // ⚠️ **A mesma pergunta que o dedo faz** — `bone_pick::hover` só oferece as
                // paredes do osso em FOCO, e é esta linha que decide de quem elas são.
                ph2d_skeleton_render::draw_limit(
                    osso_focado
                        .and_then(|b| {
                            // ⚠️ **O MESMO zoom que o dedo usa** (`bone_pick::hover` recebe
                            // este `vec_px_to_world`): a folga das alças é uma grandeza de TELA
                            // sobre geometria de MUNDO, e dois zooms diferentes poriam a alça
                            // pintada num sítio e a agarrável noutro.
                            crate::bone_limit::arc(
                                sim,
                                ph2d_ecs::Entity::from_bits(b),
                                vec_px_to_world,
                            )
                        })
                        .as_ref(),
                    self.skeleton.bone_hover.map(|h| h.part),
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
                // ⚠️ **Que pontas recebem anel é um CORPO e mora na família**
                // ([`ph2d_app_skeleton::goal::ring_targets`]): aqui decide-se a ORDEM dos passes,
                // não o que cada um desenha.
                let criar = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bone
                    && self.vec.draw_config.bone_action == ph2d_tool_vector::BoneAction::Create;
                let pontas = ph2d_app_skeleton::goal::ring_targets(sim, criar);
                // ⭐⭐⭐ **A FAIXA DA CORRENTE GOVERNADA, por BAIXO dos ossos** (report do dono,
                // 2026-09-14: *«não temos uma linha indicativa do IK Chain»*). O `Chain` é um
                // número no painel e o que ele significa é **quais ossos obedecem** — sem isto,
                // mudá-lo de `2` para `4` não tem efeito visível nenhum até se arrastar o alvo.
                // ⚠️ **Antes** do `draw_bones`, e é o que a torna um realce: o corpo do osso
                // continua a ser o que se vê e o que se agarra.
                ph2d_skeleton_render::draw_chains(
                    &crate::skeleton_goal::chains(sim),
                    hero.gizmo.selection,
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
                ph2d_skeleton_render::draw_bones(
                    &ossos,
                    hero.gizmo.selection,
                    self.skeleton.bone_hover,
                    &pontas,
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
                // ⭐⭐⭐ **AS ÂNCORAS DE IK** — o losango do alvo e o tracejado até à ponta. Elas
                // vêm DEPOIS dos ossos porque o alvo é o que a mão agarra: ele fica por cima.
                // ⚠️ E a FAIXA da corrente governada já foi desenhada ANTES dos ossos, por baixo
                // deles (`draw_chains`): ela é um realce, não um desenho novo.
                ph2d_skeleton_render::draw_goals(
                    &crate::skeleton_goal::anchors(sim),
                    hero.gizmo.selection,
                    ph2d_app_skeleton::goal::goal_hover(criar, self.skeleton.bone_hover),
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
            // ⭐⭐⭐ **O OSSO QUE ESTÁ A NASCER** (Enio, 2026-09-07: *«deve aparecer logo no
            // mouse down e crescer conforme o usuário arrasta»*). ⛔ Ele fica FORA do `if
            // !ossos.is_empty()` de propósito: o PRIMEIRO osso de uma cena nasce quando não há
            // osso nenhum, e era exactamente esse que o artista desenhava às cegas.
            if let Some((origem, ponta, arma)) = self.skeleton.bone_preview {
                ph2d_skeleton_render::draw_bone_preview(
                    origem,
                    ponta,
                    arma,
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
        }
        Some(cam_affine)
    }
}
