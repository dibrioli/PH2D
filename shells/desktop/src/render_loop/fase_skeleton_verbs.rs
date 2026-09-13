//! **Fase do quadro: OS VERBOS DO ESQUELETO** — ligar e soltar a pele e os números do osso. ⚠️ O bloco do osso seleccionado, a seguir, fica
//! no ORQUESTRADOR: ele já só chama as suas fases, e a cola delas é do quadro (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct SkeletonVerbsIntents {
    pub(super) pending_bone_bind: bool,
    pub(super) pending_bone_release: Option<crate::skeleton_live::Keep>,
    pub(super) pending_bone_knob: Option<(bool, f64)>,
    pub(super) osso_selecionado: Option<u64>,
    pub(super) selecao_bits: Vec<u64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_skeleton_verbs(&mut self, intents: SkeletonVerbsIntents) -> Option<usize> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            asset_db,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        let SkeletonVerbsIntents {
            pending_bone_bind,
            pending_bone_release,
            pending_bone_knob,
            osso_selecionado,
            selecao_bits,
        } = intents;
        // ⭐⭐⭐ **O ESQUELETO** (estudo 42 item 5): os três verbos da seção, aplicados aqui como
        // os do envelope — o dreno acima só CAPTURA, e quem mexe no mundo é este bloco.
        if pending_bone_bind {
            let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            let semente = osso_selecionado.map(ph2d_ecs::Entity::from_bits);
            let n = crate::skeleton_live::bind(sim, vec_scene, &self.vec.entities, &ids, semente);
            // ⭐⭐⭐ **E AS IMAGENS ESCOLHIDAS** — a 2.ª mídia (ordem do dono, 2026-09-09).
            //
            // ⚠️ **O MESMO botão, e é o desenho todo:** o estado da arte diz que o artista não
            // deve trabalhar na malha, e o gesto que ele já aprendeu é *escolher e prender*. A
            // malha é traçada da própria tinta e nunca aparece na tela.
            //
            // ⚠️ **O sujeito de uma imagem é a SELECÇÃO do gizmo**, e não a lista de caminhos
            // do pen — são duas famílias com dois selectores, e ler o do vector daria sempre
            // zero imagens.
            let imagens: Vec<(ph2d_ecs::Entity, ph2d_asset::AssetId)> = selecao_bits
                .iter()
                .copied()
                .filter_map(ph2d_ecs::Entity::try_from_bits)
                .filter(|&e| sim.world().get::<ph2d_render::Sprite>(e).is_some())
                .filter_map(|e| {
                    sim.world()
                        .get::<ph2d_ecs::SpritePixels>(e)
                        .map(|p| (e, p.0))
                })
                .collect();
            let mut n_img = 0;
            for (e, id) in imagens {
                let Some(asset) = asset_db.get(&id) else {
                    continue;
                };
                let Some((w, h, cow)) = asset.image_rgba8() else {
                    continue;
                };
                if crate::skeleton_live::bind_image(
                    sim,
                    e,
                    &cow,
                    [w, h],
                    ph2d_poly2d::GridOptions::default(),
                    semente,
                ) {
                    n_img += 1;
                }
            }
            if n_img > 0 {
                eprintln!(
                    "[ph2d-vec] osso: {n_img} imagem(ns) presa(s) -- a malha saiu do recorte da \
                         propria tinta e nao aparece na tela"
                );
            }
            let n = n + n_img;
            if n == 0 {
                eprintln!(
                    "[ph2d-vec] osso: selecione ao menos UMA forma, e desenhe um esqueleto                          antes (ferramenta Bone)"
                );
            } else {
                eprintln!(
                    "[ph2d-vec] osso: {n} forma(s) presa(s) -- no modo Bone: CORPO gira, \
                         bolinha desloca, quadradinho da mancha muda a forca, e o ANEL DUPLO na \
                         ponta da corrente dobra a corrente inteira (IK)"
                );
            }
        }
        if let Some(keep) = pending_bone_release {
            let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            crate::skeleton_live::release(sim, vec_scene, &self.vec.entities, &ids, keep);
        }
        if let Some((forca, v)) = pending_bone_knob
            && let Some(bits) = osso_selecionado
            && let Some(mut osso) = sim
                .world_mut()
                .get_mut::<ph2d_skeleton_ecs::Bone>(ph2d_ecs::Entity::from_bits(bits))
        {
            // ⛔ Os dois são pisos, não tetos: um comprimento negativo viraria o osso do avesso
            // e uma força negativa daria peso negativo. O TETO é o do documento — §0.0: um
            // limite legítimo diz de que recurso é, e não há recurso nenhum a limitar aqui.
            if forca {
                osso.strength = v.max(0.0);
            } else {
                osso.length = v.max(0.0);
            }
        }
        // ⭐⭐⭐ **A ÂNCORA DE IK** — os dois verbos e os três números, aplicados aqui como os do
        // esqueleto: o dreno acima só CAPTURA.
        // ⭐⭐⭐ **QUANTOS CONTROLOS NASCERAM MUDOS ANTES DESTES VERBOS** — a metade de trás da
        // pergunta que o app faz depois deles.
        //
        // ⛔⛔ **Achado da auditoria de 2026-09-08:** o aviso *«este osso é conduzido por uma
        // âncora»* vivia DENTRO do *Add Smart Bone*, logo só disparava na ordem **IK → Smart**.
        // Nas outras duas — pôr a âncora **depois** do controlo, e **alargar o `Chain`** até ele
        // — o app ficava calado sobre exactamente o mesmo facto.
        //
        // ⇒ *um aviso pendurado num VERBO responde por uma ordem; pendurado no FACTO, responde
        // por todas — incluindo as que ninguém enumerou.*
        let mudos_antes = crate::skeleton_smart::governed_controls(sim).len();
        Some(mudos_antes)
    }
}
