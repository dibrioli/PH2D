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

use crate::app_state::App;
use ph2d_editor::grid_snap::GridSnapState;
use ph2d_vec_edit::snap::{SnapConfig, SnapResult, SnapTargets, collect_targets, snap};
use ph2d_vec_render::Guide;
use ph2d_vec_scene::VecPathId;

/// Distância máxima de encaixe EM FORMA, em pixels de tela (convertida para world
/// pelo zoom). A grade tem o raio de magnetismo dela.
const SNAP_PX: f64 = 8.0;

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
}

#[cfg(test)]
mod tests {
    use super::guides_of;
    use ph2d_vec_edit::snap::{SnapAxis, SnapResult};

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
