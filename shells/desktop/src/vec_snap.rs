//! Cola do snap vetorial (ADR-0108): converte o limiar de pixels para world, monta
//! os alvos a partir da cena, e transforma um [`SnapResult`] nas guias que o frame
//! desenha. O motor puro mora em `ph2d_vec_edit::snap`.
//!
//! Os alvos são recolhidos **uma vez por gesto** (no Down), não por movimento: a
//! cena não muda de forma durante o arrasto — só a coisa arrastada, que é
//! justamente o que sai da lista de alvos.
//!
//! **Grade.** O Vector não tem grade própria. O editor já tem o subsistema
//! universal ([`GridSnapState`]: nove tipos, magnetismo, subdivisões, overlay e
//! painel próprios), e o mesmo `snap_world` que o gizmo de sprite e o Painter já
//! usam. Aqui ele entra como closure ([`App::vec_grid_fn`]); ligar/desligar a
//! grade é no painel de Grid Snap, não no do Vector.
//!
//! **Alt segurado ignora o snap** (forma e grade), como no Figma. Alt já significa
//! "quebrar a tangente" ao AGARRAR um handle, mas isso é decidido no Down e handles
//! nunca encaixam, então os dois usos não se cruzam.

use crate::Transform;
use crate::app_state::App;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_vec_edit::snap::{SnapConfig, SnapTargets, bbox_key_points, collect_targets, snap};
use ph2d_vec_scene::VecPathId;

// ⭐ **A LEI mora em `ph2d_app_vec::vec_snap`; aqui fica o que PERGUNTA à janela** (W2/L4, A2).
//
// A re-exportação é o que mantém `crate::vec_snap::ask_grid` / `::guides_of` / `::VecSnapSettings`
// / `::vec_weld_tolerance` a resolver nos cinco ficheiros da shell que os chamam — mover a lei
// custou ZERO alterações neles. ⚠️ **O módulo do outro lado chamava-se `vec_snap` e na Fase B passou a `snap`** — o prefixo sai
// porque dentro da crate tudo é a família (HOWTO §1.3), e isso também desfez uma colisão de
// BASENAME real com `crates/ph2d-ecs/src/vec_bindings.rs`. O nome do módulo entra no nome de cada
// teste, então a prova (`nextest-list-diff`) corre a `--depth 1`, que compara a FUNÇÃO.
pub(crate) use ph2d_app_vec::snap::{
    DragSnap, VecSnapSettings, ask_grid, drag_snap_kind, guides_of, ids_of_bits, snap_cfg,
    vec_weld_tolerance, wants_curves,
};

impl App {
    /// Configuração de snap EM FORMA para este frame. `px_to_world` = world-units
    /// por pixel. O Alt segurado desliga tudo (forma e grade).
    pub(crate) fn vec_snap_cfg(&self, px_to_world: f64) -> SnapConfig {
        snap_cfg(&self.vec_snap, self.modifiers.alt_key(), px_to_world)
    }

    /// A cena precisa oferecer a GEOMETRIA neste gesto? Porta única: quem recolhe os alvos e
    /// quem os resolve perguntam à mesma função, senão o recolhimento pararia de trazer as
    /// curvas no dia em que um terceiro interruptor de posição nascer.
    pub(crate) fn vec_snap_wants_curves(&self) -> bool {
        wants_curves(&self.vec_snap)
    }

    /// world-units por pixel de tela (delta de 1 px na horizontal).
    pub(crate) fn vec_px_to_world(&self) -> f64 {
        let Some(gfx) = self.gfx.as_ref() else {
            return 0.0;
        };
        let win = gfx.surface.size();
        let a = gfx.camera.screen_to_world((0.0, 0.0), win);
        let b = gfx.camera.screen_to_world((1.0, 0.0), win);
        f64::from(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt())
    }

    /// Recolhe os alvos de snap da cena, excluindo o que está em movimento.
    /// Chamado no Down de cada gesto.
    pub(crate) fn vec_rebuild_snap_targets(
        &mut self,
        skip_paths: &[VecPathId],
        skip_verts: &[(VecPathId, usize)],
    ) {
        // Os alvos são pontos de MUNDO; a geometria é local (ADR-0111).
        let curves = self.vec_snap_wants_curves();
        let dragged = self.dragged_entity_bits();
        let sprites = self.sprite_snap_points(&dragged);
        self.vec_snap_targets = match self.gfx.as_ref() {
            Some(gfx) => {
                let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec_entities);
                let mut t = collect_targets(&gfx.vec_scene, &xf, skip_paths, skip_verts, curves);
                t.points.extend(sprites);
                // ⚠️ As guias entram INTEIRAS, sem filtro de gesto: elas não pertencem a forma
                // nenhuma, então mover o que quer que seja nunca tira uma da lista. É o que as
                // torna a única referência que sobrevive a arrastar tudo o que há na cena.
                t.guides = gfx.guides.iter().copied().collect();
                t
            }
            None => SnapTargets::default(),
        };
    }

    /// Encaixa um único ponto (cursor) contra formas + grade, e registra as guias.
    /// Usado pelas ferramentas de forma.
    pub(crate) fn vec_snap_point(&mut self, p: [f64; 2], cfg: SnapConfig) -> [f64; 2] {
        let targets = std::mem::take(&mut self.vec_snap_targets);
        let mut hero = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut());
        let mut grid = |q: [f64; 2]| {
            let h = hero.as_mut()?;
            ask_grid(&mut h.grid.snap_state, q)
        };
        let r = snap(&[p], &targets, cfg, Some(&mut grid));
        self.vec_snap_targets = targets;
        self.vec_snap_guides = guides_of(&r);
        r.apply(p)
    }

    /// Zera as guias (fim de gesto / gesto sem encaixe).
    pub(crate) fn vec_clear_snap_guides(&mut self) {
        self.vec_snap_guides.clear();
    }

    /// Os pontos-FONTE de um arrasto de forma/grupo: os 9 pontos-chave da bbox de
    /// curva COMBINADA (mundo) mais cada âncora dos membros. Um grupo alinha como um
    /// bloco rígido só (a bbox combinada), e cada vértice ainda pode encaixar.
    fn vec_move_sources(&self, ids: &[VecPathId]) -> Vec<[f64; 2]> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec_entities);
        let mut pts = Vec::new();
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        let mut any = false;
        for &id in ids {
            if let Some(p) = gfx.vec_scene.paths().iter().find(|p| p.id == id) {
                let x = ph2d_vec_scene::xform_of(&xf, id);
                for v in p.verts_all() {
                    pts.push(x.apply(v.anchor));
                }
            }
            if let Some((l, h)) = gfx.vec_scene.path_world_curve_bbox(&xf, id) {
                lo[0] = lo[0].min(l[0]);
                lo[1] = lo[1].min(l[1]);
                hi[0] = hi[0].max(h[0]);
                hi[1] = hi[1].max(h[1]);
                any = true;
            }
        }
        if any {
            pts.extend(bbox_key_points(lo, hi));
        }
        pts
    }

    /// Encaixa a FORMA/GRUPO arrastado (`ids`) contra a cena + grade pelo motor puro
    /// `ph2d_vec_edit::snap`, registra as GUIAS, e devolve o delta de MUNDO a somar aos
    /// `Transform` de todos os membros. `[0, 0]` se nada encaixou. É o análogo de
    /// [`Self::vec_snap_point`] para um arrasto de objeto (várias fontes, um delta).
    fn vec_snap_move(&mut self, ids: &[VecPathId]) -> [f64; 2] {
        let sources = self.vec_move_sources(ids);
        if sources.is_empty() {
            self.vec_snap_guides.clear();
            return [0.0, 0.0];
        }
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        let targets = std::mem::take(&mut self.vec_snap_targets);
        let mut hero = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut());
        let mut grid = |q: [f64; 2]| {
            let h = hero.as_mut()?;
            ask_grid(&mut h.grid.snap_state, q)
        };
        let r = snap(&sources, &targets, cfg, Some(&mut grid));
        self.vec_snap_targets = targets;
        self.vec_snap_guides = guides_of(&r);
        r.delta()
    }

    /// Os `VecPathId` das entidades `bits` que são vetoriais (o resto é ignorado).
    pub(crate) fn vec_ids_of_bits(&self, bits: &[u64]) -> Vec<VecPathId> {
        self.gfx
            .as_ref()
            .map_or_else(Vec::new, |g| ids_of_bits(&g.sim, bits))
    }

    /// As formas vetoriais arrastadas por um gizmo: a primária (`primary_bits`) mais
    /// os membros do grupo. Usado para EXCLUIR o que se move dos alvos de snap (uma
    /// forma não encaixa em si mesma), tanto no translate quanto no scale.
    pub(crate) fn dragged_vec_path_ids(&self, primary_bits: u64) -> Vec<VecPathId> {
        let mut bits = vec![primary_bits];
        bits.extend(self.group_drag_starts.iter().map(|s| s.entity_bits));
        self.vec_ids_of_bits(&bits)
    }

    /// Snap vetorial em TEMPO REAL durante um arraste de gizmo: chamado a cada
    /// `CursorMoved`, logo depois de `advance_gizmo_drag` seguir o cursor. Encaixa a
    /// forma/grupo (bordas/centros/vértices, X e Y independentes, mais a grade) e
    /// desliza TODOS os membros pelo mesmo delta — um grupo alinha como bloco rígido.
    /// As guias aparecem no frame. Só o gesto Translate desliza aqui (scale encaixa
    /// dentro do gizmo; rotate/move-pivot não têm snap de posição).
    pub(crate) fn snap_dragged_vec_during_drag(&mut self) {
        let drag = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.gizmo.drag);
        let Some(drag) = drag else {
            self.vec_snap_guides.clear();
            return;
        };
        // O snap de translação (deslizar a forma pelo delta) é feito AQUI. Scale
        // encaixa dentro do gizmo (advance_gizmo_drag, que já publica as guias) — só
        // saímos. Rotate/MovePivot não têm snap de posição — as guias saem.
        match drag_snap_kind(drag.kind) {
            DragSnap::SlideTranslate => {}
            DragSnap::GizmoOwnsGuides => return,
            DragSnap::ClearGuides => {
                self.vec_snap_guides.clear();
                return;
            }
        }
        let mut bits = vec![drag.entity_bits];
        bits.extend(self.group_drag_starts.iter().map(|s| s.entity_bits));
        let ids = self.vec_ids_of_bits(&bits);
        if ids.is_empty() {
            return;
        }
        // Alvos = tudo menos o que se move (a cena é estável durante o arraste; só o
        // skip muda). Recolher por Move é barato p/ a contagem de formas típica.
        self.vec_rebuild_snap_targets(&ids, &[]);
        let delta = self.vec_snap_move(&ids);
        if delta == [0.0, 0.0] {
            return;
        }
        if let Some(gfx) = self.gfx.as_mut() {
            for &b in &bits {
                slide_entity_world(&mut gfx.sim, b, delta);
            }
        }
    }
}

/// Desliza a entidade `bits` por um delta de MUNDO, convertido para o frame local do
/// pai e somado à `translation`. No-op se o delta é nulo. Recebe só o `SimWorld` (não
/// o `AppGfx`) para não colidir com um borrow vivo de `hero_screen` no chamador.
pub(crate) fn slide_entity_world(sim: &mut SimWorld, bits: u64, delta: [f64; 2]) {
    if delta == [0.0, 0.0] {
        return;
    }
    let entity = Entity::from_bits(bits);
    let pw = ph2d_ecs::parent_world_transform(sim.world(), entity);
    let parent = ph2d_editor::TransformSnapshot {
        translation: [pw.translation.x, pw.translation.y],
        rotation: pw.rotation,
        scale: [pw.scale.x, pw.scale.y],
    };
    let [dx, dy] = ph2d_editor::world_delta_to_local(parent, delta[0] as f32, delta[1] as f32);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
        t.translation.x += dx;
        t.translation.y += dy;
    }
}
