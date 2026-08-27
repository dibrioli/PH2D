//! **O REALCE DE PROVENIÊNCIA** (estudo de UI viva, C2) — *passar sobre uma coisa acende a mesma
//! coisa do outro lado*, para **qualquer objecto** e em **qualquer modo** (Enio, 2026-08-23).
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
/// **TODOS OS OBJECTOS SOB UM PONTO**, em ordem de z (o de cima primeiro) — a porta ÚNICA do
/// *pick de objecto* deste app.
///
/// Três fontes, e a ordem é a que o artista vê: as formas **vetoriais** desenham por cima, depois
/// a arte do **Flip**, depois as **sprites**.
///
/// ⚠️ **Ela existia DUAS vezes copiada dentro do `input_dispatch`** (o clique com modificador e o
/// clique simples), e o realce ia ser a terceira. Três cópias de *"o que este ponto pega"* é como
/// o realce acaba a acender uma coisa e o clique a pegar outra — e nenhuma das três fica vermelha,
/// porque cada uma está certa sozinha.
///
/// ⚠️ **Função LIVRE, e é o que a torna alcançável dos três sítios.** Um método de `App` seria
/// inalcançável de dentro do `input_dispatch`, onde o `gfx` já é um `&mut` do campo — e os campos
/// vizinhos entram por si, que é o empréstimo disjunto que o Rust permite. *Uma porta que um dos
/// chamadores não alcança é uma porta que ele contorna.*
///
/// ⚠️ **`&mut AppGfx` não é descuido:** o pick de sprites consulta a `PresentWorld`, e a query do
/// bevy pede mundo mutável.
/// Os pedaços do mundo que o pick precisa — e o `hero_screen` fica **de fora de propósito**: é
/// justamente ele que os chamadores do `input_dispatch` seguram (`gfx.hero_screen.as_mut()`)
/// enquanto pedem o pick. Um `&mut AppGfx` seria um segundo empréstimo mutável do mesmo `gfx`, e a
/// porta ficaria inalcançável dos dois sítios que ela existe para unificar.
pub(crate) struct PickWorld<'a> {
    pub(crate) sim: &'a ph2d_ecs::SimWorld,
    pub(crate) vec_scene: &'a ph2d_vec_scene::VecScene,
    pub(crate) flip: &'a ph2d_flip::FlipDoc,
    pub(crate) present: &'a mut ph2d_ecs::PresentWorld,
    pub(crate) camera: &'a ph2d_render::Camera2d,
    pub(crate) window_size: ph2d_host::WindowSize,
    /// A régua do projeto — o anel de um objeto vazio é medido em pixels de arte
    /// ([`crate::group_gizmo_view::marker_world_radius`]).
    pub(crate) pixels_per_meter: f32,
}

pub(crate) fn pick_objects_at(
    w: &mut PickWorld,
    vec_entities: &crate::vec_entities::VecEntityMap,
    vec_view_derived: &ph2d_vec_scene::VecViewState,
    vec_live_drawn: &ph2d_vec_render::LiveGeometry,
    flip_entities: &crate::flip_entities::FlipEntityMap,
    pointer: (f32, f32),
) -> Vec<u64> {
    let world = w.camera.screen_to_world(pointer, w.window_size);
    let stroke_r = crate::vec_gizmo_view::stroke_hit_r(w.camera, w.window_size);
    let flip_r = crate::flip_gizmo_view::stroke_hit_r(w.camera, w.window_size);
    let view = crate::vec_entities::view_state_for_pick(w.sim, vec_entities, vec_view_derived);
    let mut hits = crate::vec_gizmo_view::pick_all_at_world(
        w.sim,
        w.vec_scene,
        vec_live_drawn,
        &view,
        vec_entities,
        world,
        stroke_r,
    );
    // ADR-0114/ADR-0111: a arte Flip também compõe por cima dos sprites — entra na lista do
    // clique-cíclico, sob o vetor.
    hits.extend(crate::flip_gizmo_view::pick_all_at_world(
        w.sim,
        w.flip,
        flip_entities,
        world,
        flip_r,
    ));
    hits.extend(ph2d_render::pick_sprites_at_world(
        w.present.world_mut(),
        world,
    ));
    // ⭐⭐ **E o ANEL de um objeto VAZIO pega** (Enio, 2026-08-26: *«não consigo transformar o
    // objeto total a partir do centro do objeto vazio»*).
    //
    // ⚠️ **Por ÚLTIMO, e é a metade que importa:** um objeto vazio é quase sempre o PAI da arte que
    // está por baixo do anel, e a lista é depois reordenada por `pick_order::descendants_first` —
    // que ADIA o ancestral. Um clique sobre a arte pega a arte; um clique no anel onde não há arte
    // pega o grupo; e o segundo clique no mesmo sítio cicla para ele. *O contêiner não rouba o
    // clique dos filhos.*
    hits.extend(crate::group_gizmo_view::pick_empty_at_world(
        w.sim,
        world,
        w.pixels_per_meter,
    ));
    hits
}

impl crate::App {
    /// **O OBJECTO SOB O PONTEIRO**, venha ele do canvas ou de uma linha da Hierarquia — em bits de
    /// `Entity`, ou `None` quando o ponteiro não aponta a objecto nenhum.
    ///
    /// ⚠️ **A resposta do CANVAS é a que o clique daria**, e não *"a forma mais próxima"*: ela sai
    /// da MESMA porta que o clique usa ([`Self::pick_objects_at`]). Um realce que acendesse outra
    /// coisa que a que o clique pega seria pior que não haver realce nenhum.
    ///
    /// ⚠️ **O primeiro da lista, e não o último** — a ordem de z, que é a ordem em que o clique
    /// cicla.
    pub(crate) fn pick_hovered_object(&mut self, pointer: (f32, f32)) -> Option<u64> {
        // Os dois factos do store, lidos antes de qualquer borrow mutável.
        let (over_panel, hot, ppm) = {
            let hero = self.gfx.as_ref()?.hero_screen.as_ref()?;
            (
                hero.store.panel_at(pointer.0, pointer.1).is_some(),
                hero.store.hot_id(),
                hero.project.pixels_per_meter,
            )
        };
        // Sobre um painel: a resposta é a LINHA, se for uma; nunca o canvas por baixo dele.
        if over_panel {
            let node = hot?;
            return self
                .gfx
                .as_ref()?
                .hero_live
                .as_ref()
                .and_then(|live| live.bridge.entity_for(node));
        }
        let gfx = self.gfx.as_mut()?;
        let mut w = PickWorld {
            window_size: gfx.surface.size(),
            sim: &gfx.sim,
            vec_scene: &gfx.vec_scene,
            flip: &gfx.flip,
            present: &mut gfx.present,
            camera: &gfx.camera,
            pixels_per_meter: ppm,
        };
        pick_objects_at(
            &mut w,
            &self.vec_entities,
            &self.vec_view_derived,
            &self.vec_live_drawn,
            &self.flip_entities,
            pointer,
        )
        .first()
        .copied()
    }

    /// **O QUE O CONTORNO DESENHA para o objecto apontado** — resolvido no mesmo instante em que o
    /// objecto é escolhido, e guardado no quadro.
    ///
    /// ⚠️ **Duas famílias, uma regra: o contorno diz ONDE o objecto está.** Uma forma vetorial tem
    /// caminhos, e o contorno é o desenho dela; uma **sprite** e a arte do **Flip** não têm caminho
    /// nenhum, e o que "aquele objecto" significa é a **caixa de mundo** dele — a mesma que o gizmo
    /// desenha. ⛔ Inventar um contorno de silhueta para uma sprite seria uma segunda resposta a
    /// *"que forma tem este objecto"*, e ela divergiria do gizmo no primeiro objecto girado.
    pub(crate) fn resolve_hover_outline(&mut self, bits: u64) -> Vec<ph2d_vec_scene::VecPath> {
        let Some(gfx) = self.gfx.as_mut() else {
            return Vec::new();
        };
        let vector = hover_outline_world(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec_entities,
            &self.vec_live_drawn,
            bits,
        );
        if !vector.is_empty() {
            return vector;
        }
        ph2d_render::selection_bbox_world(gfx.present.world_mut(), bits)
            .map(|b| {
                let (c, h) = b.center_half();
                vec![ph2d_vec_scene::rectangle(
                    [f64::from(c[0] - h[0]), f64::from(c[1] - h[1])],
                    [f64::from(c[0] + h[0]), f64::from(c[1] + h[1])],
                )]
            })
            .unwrap_or_default()
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
#[path = "hover_highlight_tests.rs"]
mod tests;
