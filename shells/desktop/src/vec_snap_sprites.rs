//! **Os alvos de snap que vêm do RASTER** — irmão de `vec_snap.rs` pelo teto de 600 LOC da
//! shell, e o corte é por responsabilidade: lá moram os alvos da CENA VETORIAL (âncoras, caixas
//! de forma, geometria), aqui os do outro lado da árvore.
//!
//! ⚠️ **Nenhum concorrente tem isto, e a razão é estrutural:** nenhum deles mistura raster e
//! vetor na mesma hierarquia. Nós misturamos (ADR-0110), então alinhar uma forma à borda de um
//! sprite é um gesto tão comum quanto alinhá-la a outra forma — e não havia alvo nenhum.

use crate::app_state::App;
use ph2d_vec_edit::snap::bbox_key_points;

/// Os nove pontos-chave da caixa de um sprite, em MUNDO.
///
/// A caixa LOCAL é `anchor ± half` — a convenção que o `gizmo_anchor_half` fala e que o
/// `picking.rs` testa (`|local − anchor| <= half`). Cada ponto sobe pelo afim **um a um**, e
/// não a caixa inteira: sob rotação o resultado é um quadrilátero girado, e os pontos dele
/// seguem sendo os cantos e os meios — exactamente o que `collect_targets` já faz com a bbox
/// de uma forma vetorial. Re-encaixotar depois daria a caixa alinhada aos eixos, que é maior
/// que o sprite e cujos cantos não estão em cima de nada.
///
/// Pura para ser gateável sem janela — `sprite_snap_points` precisa de `gfx`, que só existe
/// com um `winit` vivo.
#[must_use]
fn sprite_box_points(
    anchor: [f32; 2],
    half: [f32; 2],
    xf: &ph2d_vec_scene::Xform,
) -> [[f64; 2]; 9] {
    let lo = [
        f64::from(anchor[0] - half[0]),
        f64::from(anchor[1] - half[1]),
    ];
    let hi = [
        f64::from(anchor[0] + half[0]),
        f64::from(anchor[1] + half[1]),
    ];
    bbox_key_points(lo, hi).map(|p| xf.apply(p))
}

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
            let xf = crate::vec_transform::xform_of_transform(
                crate::vec_transform::world_transform(&gfx.sim, e),
            );
            out.extend(sprite_box_points(anchor, half, &xf));
        }
        out
    }
}

#[cfg(test)]
mod sprite_box_tests {
    use super::sprite_box_points;
    use ph2d_vec_scene::Xform;

    /// A caixa local é `anchor ± half` — a MESMA que o gizmo desenha. Um alvo derivado por
    /// outra convenção (por exemplo `0 ± half`, tratando a âncora como pivô normalizado)
    /// pousaria o ponto onde o retângulo de seleção não está.
    #[test]
    fn the_sprite_box_is_anchor_plus_minus_half() {
        let p = sprite_box_points([5.0, 0.0], [2.0, 1.0], &Xform::IDENTITY);
        assert!(p.contains(&[3.0, -1.0]), "canto inferior-esquerdo: {p:?}");
        assert!(p.contains(&[7.0, 1.0]), "canto superior-direito");
        assert!(p.contains(&[5.0, 0.0]), "centro");
    }

    /// **Cada ponto sobe pelo afim, e não a caixa.** Re-encaixotar depois daria a caixa
    /// alinhada aos eixos — maior que o sprite, e com cantos que não estão em cima de nada.
    ///
    /// ⚠️ **A rotação TEM de ser fora dos múltiplos de 90°**, e isto custou uma mutação: um giro
    /// de 90° leva caixa alinhada em caixa alinhada, então ali re-encaixotar é no-op e o gate
    /// fica verde sobre o defeito. A 45° o quadrilátero girado tem canto em `(c, 3c)` enquanto
    /// a caixa dele teria canto em `(3c, 3c)` — dois pontos distintos, e um deles não existe.
    #[test]
    fn under_rotation_the_points_are_the_turned_quads_corners() {
        let c = std::f64::consts::FRAC_1_SQRT_2;
        let rot = Xform([c, c, -c, c, 0.0, 0.0]);
        let p = sprite_box_points([0.0, 0.0], [2.0, 1.0], &rot);
        let has = |q: [f64; 2]| {
            p.iter()
                .any(|r| (r[0] - q[0]).abs() < 1e-9 && (r[1] - q[1]).abs() < 1e-9)
        };
        assert!(has([c, 3.0 * c]), "o canto (2,1) girado 45 graus: {p:?}");
        assert!(has([3.0 * c, c]), "o canto (2,-1) girado");
        assert!(
            !has([3.0 * c, 3.0 * c]),
            "a CAIXA do quadrilatero nao e' um alvo: {p:?}"
        );
    }

    /// A translação chega (é o caso de todo sprite que não está na origem).
    #[test]
    fn the_world_offset_reaches_the_points() {
        let shifted = Xform([1.0, 0.0, 0.0, 1.0, 100.0, 50.0]);
        let p = sprite_box_points([0.0, 0.0], [1.0, 1.0], &shifted);
        assert!(
            p.contains(&[99.0, 49.0]) && p.contains(&[101.0, 51.0]),
            "{p:?}"
        );
    }
}
