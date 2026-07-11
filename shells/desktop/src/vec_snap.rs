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
use ph2d_ecs::{Entity, SimWorld, VecPathRef};
use ph2d_editor::grid_snap::GridSnapState;
use ph2d_vec_edit::snap::{
    SnapConfig, SnapResult, SnapTargets, bbox_key_points, collect_targets, snap,
};
use ph2d_vec_render::Guide;
use ph2d_vec_scene::VecPathId;

/// Distância máxima de encaixe EM FORMA, em pixels de tela (convertida para world
/// pelo zoom). A grade tem o raio de magnetismo dela. 10px dá um ímã confortável
/// (8 era tímido p/ mouse); Alt ignora o snap quando atrapalha.
const SNAP_PX: f64 = 10.0;

/// Ajustes de snap do módulo vetorial. Só o encaixe em FORMAS — a grade é do
/// subsistema universal.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct VecSnapSettings {
    /// Encaixar em âncoras / bbox das outras formas.
    pub on: bool,
}

impl Default for VecSnapSettings {
    fn default() -> Self {
        Self { on: true }
    }
}

/// Pergunta à grade universal onde `p` encaixa. `None` = grade desligada, ou o
/// ponto está fora do raio de magnetismo dela (o arrasto segue liso entre pontos
/// de rede). `sprite_half_size` é `[0, 0]`: encaixamos pontos, não sprites.
pub(crate) fn ask_grid(state: &mut GridSnapState, p: [f64; 2]) -> Option<[f64; 2]> {
    if !state.snap_enabled {
        return None;
    }
    let w = [p[0] as f32, p[1] as f32];
    let s = state.snap_world(w, [0.0, 0.0]);
    (s != w).then(|| [f64::from(s[0]), f64::from(s[1])])
}

/// As guias de um encaixe. Para um eixo X encaixado, a fonte deslocada divide o
/// `x` com o alvo → o segmento sai vertical, e é a linha que o usuário vê. Encaixe
/// de grade não tem forma do outro lado: vira só a cruz no ponto de rede.
#[must_use]
pub(crate) fn guides_of(r: &SnapResult) -> Vec<Guide> {
    let d = r.delta();
    let mut out = Vec::new();
    for axis in [r.x, r.y] {
        let Some(a) = axis else { continue };
        let moved = [a.source[0] + d[0], a.source[1] + d[1]];
        let guide = Guide {
            a: moved,
            b: if a.grid { moved } else { a.target },
            grid: a.grid,
        };
        // A grade reivindica os dois eixos com o MESMO ponto de rede: uma cruz basta.
        if !out.contains(&guide) {
            out.push(guide);
        }
    }
    out
}

/// O que `snap_dragged_vec_during_drag` faz com as guias, por tipo de arraste do
/// gizmo. Pura p/ ser testável sem gfx — trava a política de snap-por-gesto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum DragSnap {
    /// Translate: o snap desliza a forma inteira pelo delta (feito aqui no shell).
    SlideTranslate,
    /// Scale: o encaixe acontece DENTRO do gizmo (`advance_gizmo_drag` alimenta o
    /// `world_snap_fn` com os alvos vetoriais e já publica as guias) — não mexemos.
    GizmoOwnsGuides,
    /// Rotate/MovePivot: sem snap espacial de posição — as guias saem da tela.
    ClearGuides,
}

/// Roteia o gesto do gizmo para a disposição de snap correspondente.
#[must_use]
pub(crate) fn drag_snap_kind(kind: ph2d_editor::GizmoDragKind) -> DragSnap {
    use ph2d_editor::GizmoDragKind::{MovePivot, Rotate, ScaleCorner, ScaleEdge, Translate};
    match kind {
        Translate => DragSnap::SlideTranslate,
        ScaleCorner { .. } | ScaleEdge { .. } => DragSnap::GizmoOwnsGuides,
        Rotate | MovePivot => DragSnap::ClearGuides,
    }
}

impl App {
    /// Configuração de snap EM FORMA para este frame. `px_to_world` = world-units
    /// por pixel. O Alt segurado desliga tudo (forma e grade).
    pub(crate) fn vec_snap_cfg(&self, px_to_world: f64) -> SnapConfig {
        let bypass = self.modifiers.alt_key();
        SnapConfig {
            enabled: !bypass,
            to_points: self.vec_snap.on,
            threshold: SNAP_PX * px_to_world,
        }
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
        self.vec_snap_targets = match self.gfx.as_ref() {
            Some(gfx) => {
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                collect_targets(&gfx.vec_scene, &xf, skip_paths, skip_verts)
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
        let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
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
        self.gfx.as_ref().map_or_else(Vec::new, |g| {
            bits.iter()
                .filter_map(|&b| {
                    g.sim
                        .world()
                        .get::<VecPathRef>(Entity::from_bits(b))
                        .map(|v| v.0)
                })
                .collect()
        })
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

#[cfg(test)]
mod tests {
    use super::{DragSnap, drag_snap_kind, guides_of};
    use ph2d_editor::GizmoDragKind;
    use ph2d_vec_edit::snap::{SnapAxis, SnapResult};

    /// A política de snap-por-gesto: Translate desliza a forma (aqui), Scale deixa o
    /// gizmo encaixar (não mexe nas guias), Rotate/MovePivot limpam. Se scale voltasse
    /// a "SlideTranslate" o canto âncora saltaria; se limpasse, a guia do scale sumiria.
    #[test]
    fn drag_snap_kind_routes_by_gesture() {
        assert_eq!(
            drag_snap_kind(GizmoDragKind::Translate),
            DragSnap::SlideTranslate
        );
        assert_eq!(drag_snap_kind(GizmoDragKind::Rotate), DragSnap::ClearGuides);
        assert_eq!(
            drag_snap_kind(GizmoDragKind::MovePivot),
            DragSnap::ClearGuides
        );
        assert_eq!(
            drag_snap_kind(GizmoDragKind::ScaleCorner {
                dx_sign: 1.0,
                dy_sign: -1.0,
            }),
            DragSnap::GizmoOwnsGuides,
        );
        assert_eq!(
            drag_snap_kind(GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 }),
            DragSnap::GizmoOwnsGuides,
        );
    }

    /// Um encaixe em X faz a fonte deslocada dividir o `x` com o alvo → a guia sai
    /// VERTICAL. É o que torna a linha legível: ela liga os dois pontos alinhados.
    #[test]
    fn an_x_snap_draws_a_vertical_segment_between_the_two_points() {
        let r = SnapResult {
            x: Some(SnapAxis {
                delta: 0.5,
                source: [9.5, 100.0],
                target: [10.0, 20.0],
                grid: false,
            }),
            y: None,
        };
        let g = guides_of(&r);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].a, [10.0, 100.0], "a fonte, já encaixada");
        assert_eq!(g[0].b, [10.0, 20.0], "o alvo");
        assert_eq!(g[0].a[0], g[0].b[0], "vertical");
        assert!(!g[0].grid);
    }

    /// Os dois eixos encaixados: a fonte carrega AMBOS os deltas, então cada guia
    /// continua axis-aligned (uma vertical, uma horizontal).
    #[test]
    fn both_axes_produce_two_axis_aligned_guides() {
        let r = SnapResult {
            x: Some(SnapAxis {
                delta: 1.0,
                source: [9.0, 5.0],
                target: [10.0, 50.0],
                grid: false,
            }),
            y: Some(SnapAxis {
                delta: -2.0,
                source: [9.0, 5.0],
                target: [80.0, 3.0],
                grid: false,
            }),
        };
        let g = guides_of(&r);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].a, [10.0, 3.0]);
        assert_eq!(g[0].a[0], g[0].b[0], "guia de X é vertical");
        assert_eq!(g[1].a[1], g[1].b[1], "guia de Y é horizontal");
    }

    /// A grade reivindica os dois eixos com o MESMO ponto de rede: uma cruz, não
    /// duas sobrepostas. E ela não tem forma do outro lado — a guia degenera.
    #[test]
    fn a_grid_snap_is_one_cross_not_two_lines() {
        let lattice = SnapAxis {
            delta: 0.4,
            source: [9.6, 20.4],
            target: [10.0, 20.0],
            grid: true,
        };
        let r = SnapResult {
            x: Some(lattice),
            y: Some(SnapAxis {
                delta: -0.4,
                ..lattice
            }),
        };
        let g = guides_of(&r);
        assert_eq!(g.len(), 1, "uma cruz só");
        assert_eq!(g[0].a, [10.0, 20.0], "no ponto de rede");
        assert_eq!(g[0].a, g[0].b, "degenerado: sem linha");
        assert!(g[0].grid);
    }

    /// Forma num eixo, grade no outro: duas guias distintas — a linha da forma e a
    /// cruz da grade, esta última no ponto onde a forma de fato parou.
    #[test]
    fn a_shape_snap_and_a_grid_snap_coexist() {
        let r = SnapResult {
            x: Some(SnapAxis {
                delta: -0.3,
                source: [9.8, 20.4],
                target: [9.5, 99.0],
                grid: false,
            }),
            y: Some(SnapAxis {
                delta: -0.4,
                source: [9.8, 20.4],
                target: [10.0, 20.0],
                grid: true,
            }),
        };
        let g = guides_of(&r);
        assert_eq!(g.len(), 2);
        assert!(
            !g[0].grid && g[0].a[0] == g[0].b[0],
            "linha vertical da forma"
        );
        assert!(g[1].grid && g[1].a == g[1].b, "cruz da grade");
        assert_eq!(g[1].a, [9.5, 20.0], "onde a forma parou de verdade");
    }
}
