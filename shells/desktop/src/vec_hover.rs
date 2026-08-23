//! **O REALCE DE PROVENIÊNCIA** (estudo de UI viva, C2) — *passar sobre uma coisa acende a mesma
//! coisa do outro lado*.
//!
//! # O que o artista ganha
//!
//! Uma booleana viva de cinco formas é uma lista de cinco linhas na Hierarquia e um borrão só no
//! canvas. Perguntar *"qual destas linhas é aquele lobo?"* — ou o inverso — custava um clique e
//! desfazer o clique. Com o realce, custa passar o rato.
//!
//! # ⭐ UMA PORTA, e é essa a razão de este módulo existir
//!
//! O realce tem **dois produtores** (o ponteiro sobre o canvas · o ponteiro sobre uma linha) e
//! **dois consumidores** (a linha acende · a forma ganha contorno). Quatro metades. Se cada
//! consumidor derivasse a própria resposta, bastaria um deles usar a ordem de z e o outro a ordem
//! da árvore para a linha acesa e a forma contornada serem **objectos diferentes** — e a UI
//! passaria a mentir sobre a proveniência sem uma linha vermelha em lado nenhum.
//!
//! É a mesma disciplina do [`crate::vec_bool_shape`], e a lição que esta linha pagou três vezes em
//! 2026-08-23: *pintar e despachar têm de ler a MESMA fonte*.
//!
//! # A PRECEDÊNCIA é a lei, e ela sai de uma pergunta que a shell já responde
//!
//! O ponteiro está sobre um painel? Então ele **não** está sobre o canvas, e picar o canvas
//! responderia por uma forma que está por baixo da janela. `panel_at` é quem sabe, e é a mesma
//! guarda que o clique usa ([`crate::input_dispatch`]) — usar outra faria o realce acender o que o
//! clique não pega.
//!
//! # ⚠️ E o pick corre a cada quadro, com a conta na mão
//!
//! Medido em 2026-08-23 (release, esta workstation), `pick_all_at_world` sobre uma cena de:
//!
//! | formas | por pick | de um quadro de 16,67 ms |
//! |---|---|---|
//! | 10 | 0,0050 ms | 0,03% |
//! | 50 | 0,0265 ms | 0,16% |
//! | 200 | 0,1066 ms | 0,64% |
//! | 800 | 0,5423 ms | **3,25%** |
//!
//! É **linear** e cabe folgado. ⛔ Não construí cache nenhum: um cache aqui seria estado vivo a
//! invalidar contra a câmara, a cena, o mapa vivo e o ponteiro — quatro fontes para poupar meio
//! por cento de um quadro. *Medir antes de otimizar* ([`project_m5_perf_validated`]).

/// A porta que RESOLVE, e a que DESENHA — as duas neste módulo de propósito: elas têm de
/// concordar sobre que objecto é o assunto.
impl crate::App {
    /// **O OBJETO SOB O PONTEIRO**, venha ele do canvas ou de uma linha da Hierarquia — em bits de
    /// `Entity`, ou `None` quando o ponteiro não aponta a objecto nenhum.
    ///
    /// ⚠️ **A resposta do CANVAS é a mesma que o clique daria** (o mapa vivo FUNDIDO
    /// `vec_live_drawn`, a mesma `view_state_for_pick`, o mesmo raio de traço), e não *"a forma
    /// mais próxima"*: um realce que acendesse outra coisa que a que o clique pega seria pior que
    /// não haver realce nenhum.
    ///
    /// ⚠️ **O primeiro da lista, e não o último.** O `pick_all_at_world` devolve do topo para o
    /// fundo, com quem está mesmo sob o dedo à frente de quem é alcançável pela porta do grupo —
    /// que é exactamente a ordem em que o clique cicla.
    #[must_use]
    pub(crate) fn pick_hovered_object(
        &self,
        hero: &ph2d_editor::HeroScreen,
        pointer: (f32, f32),
    ) -> Option<u64> {
        let gfx = self.gfx.as_ref()?;
        // Sobre um painel: a resposta é a LINHA, se for uma; nunca o canvas por baixo dele.
        if hero.store.panel_at(pointer.0, pointer.1).is_some() {
            let node = hero.store.hot_id()?;
            return gfx
                .hero_live
                .as_ref()
                .and_then(|live| live.bridge.entity_for(node));
        }
        let window_size = gfx.surface.size();
        let view = crate::vec_entities::view_state_for_pick(
            &gfx.sim,
            &self.vec_entities,
            &self.vec_view_derived,
        );
        crate::vec_gizmo_view::pick_all_at_world(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec_live_drawn,
            &view,
            &self.vec_entities,
            gfx.camera.screen_to_world(pointer, window_size),
            crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
        )
        .first()
        .copied()
    }
}

/// **A GEOMETRIA QUE O CONTORNO DESENHA** — em MUNDO, para o objecto apontado.
///
/// ⚠️ **Ela responde o mesmo que o clique vê, e há um caso em que as duas coisas divergem:** um
/// operando **absorvido** por uma booleana viva tem entrada VAZIA no mapa vivo (o resultado do
/// grupo pousa na base, e os demais desenham nada). Contornar essa entrada desenharia **nada** —
/// justamente no caso em que o artista mais precisa de saber qual dos cinco lobos é aquela linha.
///
/// ⇒ a regra é uma só: **o que o mapa vivo diz, ou — se ele nada diz — a pegada PRÓPRIA da
/// forma**, que é exactamente o que a linha da Hierarquia nomeia.
#[must_use]
pub(crate) fn hover_outline_world(
    sim: &ph2d_ecs::SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    map: &crate::vec_entities::VecEntityMap,
    live: &ph2d_vec_render::LiveGeometry,
    bits: u64,
) -> Vec<ph2d_vec_scene::VecPath> {
    use ph2d_vec_scene::{bake_xform, xform_of};
    let ids = crate::vec_entities::subtree_paths(sim, scene, ph2d_ecs::Entity::from_bits(bits));
    let xf = crate::vec_transform::build(sim, map);
    let mut out = Vec::new();
    for id in ids {
        match live.get(&id) {
            Some(items) if !items.is_empty() => out.extend(items.iter().cloned()),
            _ => {
                if let Some(p) = scene.path(id) {
                    let mut world = p.cooked().into_owned();
                    bake_xform(&mut world, &xform_of(&xf, id));
                    out.push(world);
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "vec_hover_tests.rs"]
mod tests;
