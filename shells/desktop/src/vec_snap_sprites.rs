//! **Os alvos de snap que vêm do RASTER** — irmão de `vec_snap.rs` pelo teto de 600 LOC da
//! shell, e o corte é por responsabilidade: lá moram os alvos da CENA VETORIAL (âncoras, caixas
//! de forma, geometria), aqui os do outro lado da árvore.
//!
//! ⚠️ **Nenhum concorrente tem isto, e a razão é estrutural:** nenhum deles mistura raster e
//! vetor na mesma hierarquia. Nós misturamos (ADR-0110), então alinhar uma forma à borda de um
//! sprite é um gesto tão comum quanto alinhá-la a outra forma — e não havia alvo nenhum.

use crate::app_state::App;

// ⭐ A caixa (pura) mora em `ph2d_app_vec::snap_sprites` — W2/L4, A2.
use ph2d_app_vec::snap_sprites::sprite_box_points;

impl App {
    /// As entidades que este gesto está MOVENDO — a primária do gizmo mais os membros do grupo.
    ///
    /// Mesma fonte que o `snap_dragged_vec_during_drag` já consulta: derivar aqui em vez de
    /// receber por parâmetro mantém as cinco chamadas de [`Self::vec_rebuild_snap_targets`]
    /// intactas **e** impede que as duas listas divirjam. Sem gesto em curso ela é vazia, que é
    /// o certo — durante um traço de caneta nenhum sprite está se movendo.
    pub(crate) fn dragged_entity_bits(&self) -> Vec<u64> {
        let mut bits: Vec<u64> = self
            .group_drag_starts
            .iter()
            .map(|s| s.entity_bits)
            .collect();
        if let Some(d) = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.gizmo.drag)
        {
            bits.push(d.entity_bits);
        }
        bits
    }

    /// Os pontos-chave da caixa de cada SPRITE, em mundo.
    ///
    /// ⚠️ **Nenhum concorrente tem isto**, e a razão é estrutural: nenhum deles mistura raster e
    /// vetor na mesma árvore. Nós misturamos (ADR-0110), então alinhar uma forma à borda de um
    /// sprite é um gesto tão comum quanto alinhá-la a outra forma — e não havia alvo nenhum.
    ///
    /// ⚠️ A caixa vem da **porta que o GIZMO usa** (`anchor ± half`, o par que
    /// `gizmo_anchor_half` fala): um alvo derivado por outra via pousaria o ponto onde o
    /// retângulo de seleção não está, e a única testemunha seria o olho do artista.
    pub(crate) fn sprite_snap_points(&mut self, skip: &[u64]) -> Vec<[f64; 2]> {
        let Some(gfx) = self.gfx.as_mut() else {
            return Vec::new();
        };
        let mut boxes: Vec<(ph2d_ecs::Entity, [f32; 2], [f32; 2])> = Vec::new();
        {
            let mut q = gfx
                .sim
                .world_mut()
                .query::<(ph2d_ecs::Entity, &ph2d_render::Sprite)>();
            let world = gfx.sim.world();
            for (e, s) in q.iter(world) {
                if skip.contains(&e.to_bits()) {
                    continue;
                }
                boxes.push((e, s.anchor, [s.size[0] * 0.5, s.size[1] * 0.5]));
            }
        }
        let mut out = Vec::with_capacity(boxes.len() * 9);
        for (e, anchor, half) in boxes {
            let xf = ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(&gfx.sim, e),
            );
            out.extend(sprite_box_points(anchor, half, &xf));
        }
        out
    }
}
