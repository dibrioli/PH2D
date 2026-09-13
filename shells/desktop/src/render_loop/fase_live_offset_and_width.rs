//! **Fase do quadro: O OFFSET E A LARGURA VIVOS** — o offset ao vivo e o perfil de largura vivo (ADR-0148) (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct LiveOffsetAndWidthIntents {
    pub(super) pending_width_preset: Option<usize>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_live_offset_and_width(&mut self, intents: LiveOffsetAndWidthIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let LiveOffsetAndWidthIntents {
            pending_width_preset,
        } = intents;
        // ── Offset AO VIVO ───────────────────────────────────────────────────
        // *"os botões Miter, Round e Bevel são previsualizações em tempo real dos efeitos,
        // mas para consolidar a curva deve-se apertar Apply Offset ou Convert to Curves"*
        // (Enio, 2026-07-21). O documento guarda a curva AUTORADA o tempo todo — o que se
        // vê é a geometria derivada, cozida por `offset_live::recook` e desenhada no z da
        // forma. Aqui só se ARMA a relação (`ph2d_ecs::VecOffset`): o slider dá o `d`, os
        // chips de Corner/Side dão a quina e o lado.
        {
            let knobs = crate::vec_expand::expand_knobs();
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            // **O painel espelha o que está SELECIONADO.** Sem isto, escolher uma forma com
            // offset vivo mostraria os knobs globais do painel e o chip mentiria sobre a
            // forma que está na tela. A borda é a SELEÇÃO (não o clique), e ela corre ANTES
            // da borda dos chips — publicar depois faria o espelho parecer um clique novo e
            // reescreveria a forma com os valores que acabaram de sair dela.
            let mirror = (sel.len() == 1)
                .then(|| sel[0])
                .filter(|id| crate::offset_live::spec_of(sim, &self.vec.entities, *id).is_some());
            let knobs = if mirror != self.vec.offset_mirrored {
                self.vec.offset_mirrored = mirror;
                match mirror.and_then(|id| crate::offset_live::spec_of(sim, &self.vec.entities, id))
                {
                    Some(spec) => {
                        ph2d_panel_vector::set_expand_join(spec.join);
                        ph2d_panel_vector::set_expand_side(spec.side);
                        let scale = crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &{
                            ph2d_vec_entities::transform::build(sim, &self.vec.entities)
                        });
                        hero.store.set_slider_value(
                            ph2d_panel_vector::ids::VECTOR_EXPAND_OFFSET,
                            ph2d_tool_vector::params::offset_frac_to_slider(spec.d / scale),
                        );
                        (spec.join, spec.side)
                    }
                    None => knobs,
                }
            } else {
                knobs
            };
            let offset_grabbed = matches!(
                hero.store.active_id(),
                Some(id) if id == ph2d_panel_vector::ids::VECTOR_EXPAND_OFFSET
            );
            // O slider fala FRAÇÃO do tamanho da forma (−100%..+100%); o `d` de mundo nasce
            // de `fração × escala` (a porta única `vec_expand::offset_scale`, `ada45fac`).
            // ⚠️ A escala NÃO precisa mais ser congelada no grab: o preview deixou de
            // churnar a cena, então a bbox das FONTES não se move durante o arrasto.
            let frac = hero
                .store
                .slider(ph2d_panel_vector::ids::VECTOR_EXPAND_OFFSET)
                .map_or(ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC, |(_, v)| {
                    ph2d_tool_vector::params::slider_to_offset_frac(v)
                });
            if offset_grabbed && !sel.is_empty() {
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                let d = frac * crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
                crate::offset_live::arm(sim, &self.vec.entities, &sel, d, knobs.0, knobs.1);
                self.vec.offset_mirrored = (sel.len() == 1).then(|| sel[0]);
            }
            // Um chip de Corner/Side clicado RETUNA os offsets vivos da seleção — e só
            // eles: sem offset armado, o chip arma o próximo arrasto e não inventa
            // geometria de lugar nenhum. Comparar com o quadro anterior é o que distingue
            // "o artista clicou" de "o painel está no valor de sempre".
            if knobs != self.vec.expand_knobs.0 {
                self.vec.expand_knobs.0 = knobs;
                crate::offset_live::retune(sim, &self.vec.entities, &sel, knobs);
            }
        }
        // ── A LARGURA VIVA (ADR-0148) ────────────────────────────────────────
        // Os quatro sliders `W Start/Mid/End/Pos` deixaram de ser parâmetros de um comando
        // e passaram a AUTORAR um perfil vivo: o traço engrossa e afina enquanto o slider
        // anda, e o botão *Power Stroke* MATERIALIZA — o mesmo par que o Offset já tinha.
        {
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            // **O painel espelha o que está SELECIONADO** — a mesma lei (e a mesma ordem) do
            // offset acima: a borda é a SELEÇÃO, e ela corre ANTES de o arrasto ser lido.
            let mirror = (sel.len() == 1)
                .then(|| sel[0])
                .filter(|id| crate::profile_live::spec_of(sim, &self.vec.entities, *id).is_some());
            if mirror != self.vec.profile_mirrored {
                self.vec.profile_mirrored = mirror;
                if let Some(p) = mirror
                    .and_then(|id| crate::profile_live::spec_of(sim, &self.vec.entities, id))
                    .as_ref()
                    .and_then(crate::profile_live::preset_of)
                {
                    crate::profile_live::write_preset_to_store(&mut hero.store, &p);
                }
            }
            // **O catálogo de perfis** (W2b) — escolher uma FORMA pelo nome. Corre ANTES do
            // arrasto de propósito: escrever os quatro sliders é o que faz a fileira acender
            // e os knobs mostrarem o que a forma passou a ser, e o armamento abaixo o
            // relê pela mesma porta. As duas metades — os sliders e o documento — têm de
            // andar juntas, senão o painel diria uma coisa e a tela outra.
            if let Some(p) = pending_width_preset
                .and_then(|i| ph2d_vec_scene::WIDTH_PRESETS.get(i))
                .filter(|_| !sel.is_empty())
            {
                crate::profile_live::write_preset_to_store(&mut hero.store, &p.profile);
                crate::profile_live::arm(sim, &self.vec.entities, &sel, &p.profile.to_stops());
                // O espelho da seleção acabou de ser ESCRITO por nós: sem isto o bloco do
                // frame seguinte veria o `mirror` inalterado e não reescreveria nada — mas
                // com uma seleção de uma forma só ele passaria a divergir na primeira troca.
                self.vec.profile_mirrored = (sel.len() == 1).then(|| sel[0]);
            }
            let grabbed = matches!(
                hero.store.active_id(),
                Some(id) if id == ph2d_panel_vector::ids::VECTOR_EXPAND_W_START
                    || id == ph2d_panel_vector::ids::VECTOR_EXPAND_W_MID
                    || id == ph2d_panel_vector::ids::VECTOR_EXPAND_W_END
                    || id == ph2d_panel_vector::ids::VECTOR_EXPAND_W_POS
            );
            if grabbed && !sel.is_empty() {
                let stops = crate::profile_live::preset_from_store(&hero.store).to_stops();
                crate::profile_live::arm(sim, &self.vec.entities, &sel, &stops);
                self.vec.profile_mirrored = (sel.len() == 1).then(|| sel[0]);
            }
        }
    }
}
