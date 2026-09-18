//! **Fase do quadro: OS VERBOS DO ESQUELETO** — ligar e soltar a pele e os números do osso. ⚠️ O bloco do osso seleccionado, a seguir, fica
//! no ORQUESTRADOR: ele já só chama as suas fases, e a cola delas é do quadro (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct SkeletonVerbsIntents {
    pub(super) pending_bone_bind: bool,
    pub(super) pending_bone_release: Option<crate::skeleton_live::Keep>,
    pub(super) pending_bone_knob: Option<(ph2d_app_skeleton::knobs::BoneKnob, f64)>,
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
            hero_screen,
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
        // ⚠️ **Bloco ROTULADO e não um `return`:** a recusa é do BIND, e um retorno cedo levaria
        // com ela o soltar e os knobs, que são verbos independentes do mesmo quadro.
        'bind: {
            if !pending_bone_bind {
                break 'bind;
            }
            let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            let semente = osso_selecionado.map(ph2d_ecs::Entity::from_bits);
            // ⛔⛔⛔ **O BIND PERGUNTA ANTES DE PRENDER** (2026-09-18, defeito MEDIDO): sem osso
            // escolhido a semente é `None`, e o `skeleton_of` responde a `None` com **todos os
            // ossos da cena** — com dois esqueletos a forma ficava presa aos SEIS, em silêncio, e
            // esta linha do log dizia *«1 imagem presa»* como se estivesse tudo bem.
            //
            // ⚠️ **A lei é PURA e vive na crate** (`recusa_do_bind`): ela devolve a razão em vez de
            // a imprimir, senão a metade que interessa — *a razão certa para o facto certo* — fica
            // fora de qualquer teste, e a decisão precisaria de um mundo desenhado para ser medida.
            if let Some(r) = ph2d_skeleton_live::recusa_do_bind::recusa_do_bind(sim, semente) {
                eprintln!("[ph2d-vec] {}", r.frase());
                break 'bind;
            }
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
            // ⚠️ **O `ppm` do PROJECTO, lido como a `fase_extract_inputs` o lê:** a régua da imagem
            // resolve a âncora ao prender e o quad ao desenhar, e os dois têm de dar a mesma.
            let ppm = hero_screen
                .as_ref()
                .map(|h| h.project.pixels_per_meter)
                .unwrap_or(ph2d_editor_core::project::DEFAULT_PIXELS_PER_METER);
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
                    ppm,
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
            let n = crate::skeleton_live::release(sim, vec_scene, &self.vec.entities, &ids, keep);
            // ⭐⭐⭐ **E AS IMAGENS SOLTAM-SE TAMBÉM** (report do dono, 2026-09-18: *«ainda não temos
            // a opção de desconectar a malha do osso»*). O *Bind* alcançava as duas mídias desde a
            // wave da 2.ª mídia e o *Release* alcançava **uma** — a lei da imagem
            // (`skin_image::release_image`) existia e o **único** chamador de produto dela era
            // automático (uma ferramenta que muda a moldura solta o osso sozinho). *Uma lei sem
            // gesto é uma lei que o artista não tem.*
            //
            // ⛔ **Só no `Keep::Source`:** o `Keep::Deformed` é o *Expand*, que troca o desenho
            // autorado pela geometria de agora — e uma imagem **não tem geometria autorada**. O
            // painel já não pinta o *Expand* para uma imagem; esta cerca é a segunda metade, para o
            // caso de o comando chegar por outra porta.
            let mut n_img = 0;
            if keep == ph2d_skeleton_live::skin_live::Keep::Source {
                for bits in &selecao_bits {
                    if crate::skeleton_skin_image::release_image(sim, *bits) {
                        n_img += 1;
                    }
                }
            }
            if n + n_img == 0 {
                eprintln!(
                    "[ph2d-vec] osso: nada a soltar -- escolha a forma ou a imagem que esta' \
                     presa ao esqueleto"
                );
            } else {
                eprintln!(
                    "[ph2d-vec] osso: {n} forma(s) e {n_img} imagem(ns) solta(s) do esqueleto"
                );
            }
        }
        if let Some((knob, v)) = pending_bone_knob
            && let Some(bits) = osso_selecionado
            && let Some(mut osso) = sim
                .world_mut()
                .get_mut::<ph2d_skeleton_ecs::Bone>(ph2d_ecs::Entity::from_bits(bits))
        {
            ph2d_app_skeleton::knobs::apply(&mut osso, knob, v);
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
