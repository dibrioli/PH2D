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
                // ⭐ **O FUNDO do osso em foco** — a mancha de influência e o arco de limite, os
                // dois por BAIXO do rig. Ver [`fundo_do_osso_focado`]: aqui decide-se a ORDEM
                // dos passes, não o que cada um desenha.
                fundo_do_osso_focado(
                    sim,
                    osso_focado,
                    self.skeleton.bone_hover,
                    vec_px_to_world,
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
                // ⭐⭐⭐ **O PESO À VISTA, por BAIXO dos ossos** — cada ponto da arte presa colorido
                // pela influência do osso em foco. Ele só existe com o verbo `Weight` armado, e é
                // a razão de o pincel deixar de ser cego (`ph2d_skeleton_render::peso`).
                //
                // ⚠️ **ANTES do rig, como a mancha e o arco:** ele é uma leitura sobre o desenho, e
                // o corpo do osso continua a ser o que se vê e o que se agarra.
                //
                // ⛔ **O alvo é o que está sob o cursor, e não a selecção** — o pincel escolhe a
                // arte pela ponta do dedo (é o que o pen-down faz), logo a pré-visualização tem de
                // responder à MESMA pergunta. *Mostrar os pesos de outra arte é pior que não
                // mostrar nenhum.*
                if self.vec.draw_config.bone_action == ph2d_tool_vector::BoneAction::Weight {
                    // ⚠️ **O raio do painel e' de ECRA.** A lei quer MUNDO (converte-se com o
                    // mesmo factor do pick) e o anel quer PIXEIS — e e' por eles falarem unidades
                    // diferentes que os dois numeros aparecem aqui lado a lado.
                    let raio_px = self.vec.draw_config.weight_radius;
                    let raio = raio_px * vec_px_to_world;
                    // ⚠️ **Qual arte é o sujeito é LEI e mora na crate**
                    // ([`ph2d_skeleton_live::peso_a_mao::pontos_do_indicador`]): aqui decide-se a
                    // ORDEM dos passes, não de quem se mostram os pesos.
                    let pontos = ph2d_skeleton_live::peso_a_mao::pontos_do_indicador(
                        sim,
                        hero.project.pixels_per_meter,
                        osso_focado.map(ph2d_ecs::Entity::from_bits),
                        self.skeleton.weight_drag.map(ph2d_ecs::Entity::from_bits),
                        self.skeleton.weight_cursor,
                        raio,
                    );
                    ph2d_skeleton_render::draw_weights(
                        &pontos,
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                    // ⚠️ **A escala vem do MESMO número que o zoom dá ao dedo** — o anel é uma
                    // distância de MUNDO, e o `vec_px_to_world` é o factor inverso.
                    ph2d_skeleton_render::draw_weight_brush(
                        self.skeleton.weight_cursor,
                        raio_px,
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                }
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
                // ⭐⭐⭐⭐ **AS DUAS ALÇAS DE CURVATURA — POR CIMA DE TUDO O QUE É RIG.**
                //
                // ⛔⛔ **Report do dono (2026-09-16): *«os handles não estão por cima (z-index).
                // Handles com Z-index maior que ossos»*.** Elas pintavam-se ANTES do `draw_bones`,
                // com a justificação de que *«as hastes ligam-se à junta e à ponta, e o osso por
                // cima deixa a bolinha da junta inteira»* — uma razão de ACABAMENTO a decidir a
                // ordem de um CONTROLO.
                //
                // ⚠️⚠️ **E a ordem do desenho contradizia a do DEDO**, que é o defeito a sério: no
                // `bone_pick::hover` as alças do osso em foco competem por proximidade e ganham ao
                // CORPO (a de curvatura chega a ignorar a distância ao osso, senão seria
                // inalcançável no ponto neutro) — *o artista agarrava o que não via*. A lei é a
                // inversa da do pick: **o que o dedo apanha primeiro pinta-se por último.**
                //
                // ⭐ **E esta é a população inteira do defeito, não metade dele:** a mancha da
                // influência e o arco do limite são FUNDO, e as alças deles vivem FORA do eixo do
                // osso; as de curvatura nascem **em cima do eixo** (o ponto de controlo no terço),
                // logo são as únicas que se sobrepõem ao corpo por construção.
                ph2d_skeleton_render::draw_bend(
                    osso_focado.and_then(|b| crate::skeleton_live::bend_handles(sim, b)),
                    self.skeleton.bone_hover.map(|h| h.part),
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

/// ⭐⭐⭐ **O FUNDO DO OSSO EM FOCO** — a mancha de influência e o arco de limite.
///
/// ⚠️ **Os dois vêm ANTES do rig e nesta ordem**, e é a ordem que diz o que eles são: a mancha é
/// fundo, o arco vive por cima dela (é por isso que o véu dele é mais fraco), e o corpo do osso
/// desenha-se por cima dos dois — é ele que se vê e que se agarra.
///
/// ⚠️ **Função LIVRE e não método**, e não é estilo: no laço de desenho o `gfx` está emprestado
/// mutável de ponta a ponta (`sim` e o alvo saem dele), logo um `&mut self` aqui não compila. É a
/// mesma razão que já põe o [`crate::bone_gesture::selected_bone`] numa função livre.
fn fundo_do_osso_focado(
    sim: &SimWorld,
    osso_focado: Option<u64>,
    hover: Option<ph2d_skeleton_render::BoneHover>,
    vec_px_to_world: f64,
    cam_affine: ph2d_vector::Affine,
    theme: ph2d_tokens::Theme,
    target: &mut ph2d_vector::VectorScene,
) {
    // ⭐⭐⭐ **A REGIÃO DE INFLUÊNCIA do osso em foco** — o *Bone Strength* do Moho.
    // Ela entra ANTES dos ossos: é um fundo, e o rig desenha-se por cima dela.
    //
    // ⚠️ **O foco é a SELECÇÃO, e é a mesma pergunta que o dedo faz** — a alça só é
    // agarrável onde ela é pintada (`bone_pick::hover` recebe o mesmo `foco`).
    //
    // ⚠️ A selecção CRUA basta e filtra-se sozinha: `influence_region` devolve
    // `None` para o que não é osso, então não há aqui uma segunda pergunta
    // *"isto é um osso?"* a divergir da que o `hover` faz.
    ph2d_skeleton_render::draw_influence(
        osso_focado.and_then(|b| crate::skeleton_live::influence_region(sim, b)),
        matches!(
            hover,
            Some(h) if h.part == ph2d_skeleton_render::BonePart::Influence
        ),
        cam_affine,
        theme,
        target,
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
                crate::bone_limit::arc(sim, ph2d_ecs::Entity::from_bits(b), vec_px_to_world)
            })
            .as_ref(),
        hover.map(|h| h.part),
        cam_affine,
        theme,
        target,
    );
}
