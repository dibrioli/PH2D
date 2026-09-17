//! **Fase do quadro: O ENVELOPE** — criar, expandir e soltar o envelope, o gesto da gaiola e os presets com o bend (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct EnvelopeIntents {
    pub(super) pending_create_envelope: bool,
    pub(super) pending_expand_envelope: bool,
    pub(super) pending_release_envelope: bool,
    pub(super) pending_envelope_kind: Option<ph2d_ecs::EnvelopeKind>,
    pub(super) pending_clear_pins: bool,
    pub(super) pending_envelope_preset: Option<usize>,
    pub(super) pending_envelope_bend: Option<f64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_envelope(&mut self, intents: EnvelopeIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let EnvelopeIntents {
            pending_create_envelope,
            pending_expand_envelope,
            pending_release_envelope,
            pending_envelope_kind,
            pending_clear_pins,
            pending_envelope_preset,
            pending_envelope_bend,
        } = intents;
        if pending_create_envelope {
            let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            match ph2d_app_vec::envelope_live::create(sim, vec_scene, &self.vec.entities, &ids) {
                Some(_) => {
                    // O artista SELECIONOU a forma e SÓ ENTÃO clicou Envelope: enveloparr
                    // re-parenteia o filho sem tocar o pen, então o `sync_selection` deste
                    // frame não reroda a promoção filho→container (nem o pen mudou, nem o
                    // conjunto vetorial do gizmo) — e o gizmo ficaria no FILHO, sem gaiola para
                    // desenhar (alças de nó em vez da gaiola). Invalidar a memória do sync força
                    // a promoção no `sync_selection` logo abaixo. Gate:
                    // `enveloping_a_selected_shape_promotes_the_gizmo_to_the_container`.
                    self.vec.sel.invalidate();
                    eprintln!(
                        "[ph2d-vec] envelope: {} forma(s) envolvida(s) -- va' para o modo Node \
                             e arraste os CANTOS da gaiola",
                        ids.len()
                    );
                }
                None => eprintln!("[ph2d-vec] envelope: selecione ao menos UMA forma"),
            }
        }
        // ADR-0129: **Expand** (a deformada vira o desenho) e **Release** (a fonte autorada
        // volta). O MESMO `dissolve` — muda só QUAL geometria fica.
        if pending_expand_envelope
            && ph2d_app_vec::envelope_live::dissolve(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.pen,
                ph2d_app_vec::envelope_live::Keep::Deformed,
            )
        {
            eprintln!("[ph2d-vec] envelope: expandido (a deformacao virou o desenho)");
        }
        if pending_release_envelope
            && ph2d_app_vec::envelope_live::dissolve(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.pen,
                ph2d_app_vec::envelope_live::Keep::Authored,
            )
        {
            eprintln!("[ph2d-vec] envelope: solto (a forma original voltou)");
        }
        // ADR-0129 Fatia D: trocar o GESTO da gaiola. O container vem da MESMA porta que
        // decide a selecao e alimenta os chips -- o clique nunca acerta outro envelope que
        // nao o desenhado. Trocar re-cozinha no frame seguinte: em repouso os dois mapas
        // coincidem, entao numa gaiola intocada a troca nao move um pixel.
        if pending_envelope_kind.is_some() || pending_clear_pins {
            let sel: Vec<u64> = self
                .vec
                .pen
                .selected_paths()
                .iter()
                .filter_map(|id| self.vec.entities.get(id).copied())
                .collect();
            if let Some(bits) = ph2d_app_vec::envelope_live::sole_container(sim, &sel) {
                if let Some(kind) = pending_envelope_kind
                    && ph2d_app_vec::envelope_gesture::set_kind(sim, bits, kind)
                {
                    eprintln!("[ph2d-vec] envelope: gesto {kind:?}");
                }
                if pending_clear_pins && ph2d_app_vec::envelope_gesture::clear_pins(sim, bits) {
                    eprintln!("[ph2d-vec] envelope: pinos apagados");
                }
            }
        }
        // ADR-0129 Fatia C: o preset carimba a gaiola inteira; o Bend re-carimba o preset ATIVO.
        // Os dois passam pela MESMA `apply_preset`, entao clicar "Arc" e arrastar o Bend nao
        // podem produzir gaiolas diferentes para os mesmos numeros.
        if pending_envelope_preset.is_some() || pending_envelope_bend.is_some() {
            let sel: Vec<u64> = self
                .vec
                .pen
                .selected_paths()
                .iter()
                .filter_map(|id| self.vec.entities.get(id).copied())
                .collect();
            if let Some(bits) = ph2d_app_vec::envelope_live::sole_container(sim, &sel)
                && let Some((cur_warp, cur_bend)) =
                    ph2d_app_vec::envelope_gesture::warp_of(sim, bits)
            {
                let warp = pending_envelope_preset
                    .and_then(|i| ph2d_ecs::EnvelopeWarp::ALL.get(i).copied())
                    .or(cur_warp);
                let bend = pending_envelope_bend.unwrap_or(cur_bend);
                // Sem preset na mao E sem preset ativo, o Bend nao tem o que re-carimbar --
                // e' o caso da gaiola promovida a manual pelo arrasto.
                if let Some(warp) = warp
                    && ph2d_app_vec::envelope_live::apply_preset(sim, bits, warp, bend)
                {
                    eprintln!("[ph2d-vec] envelope: preset {warp:?} bend {bend:.2}");
                }
            }
        }
    }
}
