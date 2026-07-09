//! Snap + smart guides (ADR-0108): encaixe de pontos durante arrasto, e as guias
//! de alinhamento que explicam o encaixe. Puro — só world-space e `[f64; 2]`; o
//! shell converte o limiar de pixels para world e desenha as guias.
//!
//! # O modelo
//!
//! O arrasto oferece um punhado de **fontes** (o cursor, ou os 9 pontos-chave da
//! bbox da seleção) e a cena oferece **alvos** (âncoras + pontos-chave de bbox das
//! outras formas). Cada eixo encaixa de forma **independente** — é o que deixa uma
//! forma alinhar o topo com uma vizinha enquanto desliza livremente na horizontal.
//! É o comportamento do Figma, e é o motivo de [`SnapResult`] ter `x` e `y`
//! separados em vez de um único ponto.
//!
//! O módulo vetorial **não tem grade própria**. Quem sabe de grade é o subsistema
//! universal do editor (`GridSnapState`: 9 tipos, magnetismo, subdivisões), e ele
//! entra aqui como uma closure opaca — esta crate segue pura. A grade propõe um
//! ponto de rede INTEIRO (num hex/iso não existe "encaixar só o X"), e depois os
//! pontos das outras formas **sobrescrevem eixo a eixo**: alinhar com o desenho
//! importa mais do que alinhar com a régua.

use ph2d_vec_scene::{VecPathId, VecScene};

/// Configuração de snap. `threshold` em **world-units** (o shell divide o limiar
/// em pixels pelo zoom). A grade tem o raio de magnetismo dela e não usa isto.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SnapConfig {
    /// Desligado = nenhum encaixe, nem em forma nem em grade (o Alt segurado
    /// passa `false` aqui).
    pub enabled: bool,
    /// Encaixar em âncoras e pontos-chave de bbox das outras formas.
    pub to_points: bool,
    /// Distância máxima de encaixe em forma, por eixo.
    pub threshold: f64,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            to_points: true,
            threshold: 0.0,
        }
    }
}

/// A grade do editor, vista desta crate: dado um ponto de mundo, devolve onde ele
/// encaixa — ou `None` se a grade está desligada ou o ponto está fora do raio de
/// magnetismo dela. O shell passa uma closure sobre `GridSnapState::snap_world`.
pub type GridSnapFn<'a> = &'a mut dyn FnMut([f64; 2]) -> Option<[f64; 2]>;

/// Pontos da cena em que se pode encaixar.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapTargets {
    pub points: Vec<[f64; 2]>,
}

/// Um encaixe achado num eixo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SnapAxis {
    /// Quanto somar à coordenada desse eixo para encaixar.
    pub delta: f64,
    /// Ponto de origem (na forma movida), **antes** do encaixe.
    pub source: [f64; 2],
    /// Ponto alvo. Num encaixe de grade é o ponto de rede — não há forma do outro
    /// lado, então a guia degenera numa cruz.
    pub target: [f64; 2],
    /// Encaixou na grade, não num ponto de outra forma.
    pub grid: bool,
}

/// O encaixe de um arrasto: até um por eixo, independentes.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SnapResult {
    pub x: Option<SnapAxis>,
    pub y: Option<SnapAxis>,
}

impl SnapResult {
    /// O deslocamento total a aplicar (`[0, 0]` se nada encaixou).
    #[must_use]
    pub fn delta(&self) -> [f64; 2] {
        [
            self.x.map_or(0.0, |a| a.delta),
            self.y.map_or(0.0, |a| a.delta),
        ]
    }

    /// Encaixou em algum eixo?
    #[must_use]
    pub fn any(&self) -> bool {
        self.x.is_some() || self.y.is_some()
    }

    /// Aplica o encaixe a um ponto.
    #[must_use]
    pub fn apply(&self, p: [f64; 2]) -> [f64; 2] {
        let d = self.delta();
        [p[0] + d[0], p[1] + d[1]]
    }
}

/// Os 9 pontos-chave de uma bbox: 4 cantos, 4 meios de aresta, centro. São as
/// fontes de um arrasto de objeto e os alvos que as outras formas oferecem.
#[must_use]
pub fn bbox_key_points(lo: [f64; 2], hi: [f64; 2]) -> [[f64; 2]; 9] {
    let (mx, my) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    [
        [lo[0], lo[1]],
        [mx, lo[1]],
        [hi[0], lo[1]],
        [lo[0], my],
        [mx, my],
        [hi[0], my],
        [lo[0], hi[1]],
        [mx, hi[1]],
        [hi[0], hi[1]],
    ]
}

/// Coleta os alvos de snap da cena.
///
/// `skip_paths` sai inteiro (a seleção que está sendo movida). `skip_verts` são
/// âncoras individuais em movimento (índices PLANOS): elas saem, e a bbox do path
/// que as contém também — uma bbox que está sendo deformada não é referência.
#[must_use]
pub fn collect_targets(
    scene: &VecScene,
    skip_paths: &[VecPathId],
    skip_verts: &[(VecPathId, usize)],
) -> SnapTargets {
    let mut points = Vec::new();
    for path in scene.paths() {
        if skip_paths.contains(&path.id) {
            continue;
        }
        let mut deformed = false;
        for (i, v) in path.verts_all().enumerate() {
            if skip_verts.contains(&(path.id, i)) {
                deformed = true;
            } else {
                points.push(v.anchor);
            }
        }
        if !deformed && let Some((lo, hi)) = scene.path_curve_bbox(path.id) {
            points.extend_from_slice(&bbox_key_points(lo, hi));
        }
    }
    SnapTargets { points }
}

/// Resolve o encaixe de `sources` contra `targets` e, opcionalmente, contra a
/// grade do editor.
///
/// A grade propõe **os dois eixos de uma vez** (um ponto de rede é 2D; decompor
/// por eixo só faria sentido numa grade quadrada, e existem nove tipos). Os pontos
/// das outras formas então sobrescrevem eixo a eixo — cada um escolhendo o
/// candidato de **menor** deslocamento dentro do limiar, independente do outro.
/// Empate fica com o primeiro (determinístico).
#[must_use]
pub fn snap(
    sources: &[[f64; 2]],
    targets: &SnapTargets,
    cfg: SnapConfig,
    grid: Option<GridSnapFn<'_>>,
) -> SnapResult {
    if !cfg.enabled {
        return SnapResult::default();
    }

    // 1. A grade, se houver: a fonte de menor deslocamento reivindica os 2 eixos.
    let (mut gx, mut gy) = (None, None);
    if let Some(grid) = grid {
        let mut best: Option<(f64, [f64; 2], [f64; 2])> = None;
        for &s in sources {
            let Some(g) = grid(s) else { continue };
            let d2 = (g[0] - s[0]).powi(2) + (g[1] - s[1]).powi(2);
            if best.is_none_or(|(bd, _, _)| d2 < bd) {
                best = Some((d2, s, g));
            }
        }
        if let Some((_, s, g)) = best {
            let axis = |k: usize| {
                Some(SnapAxis {
                    delta: g[k] - s[k],
                    source: s,
                    target: g,
                    grid: true,
                })
            };
            (gx, gy) = (axis(0), axis(1));
        }
    }

    // 2. Pontos de outras formas — sobrescrevem a grade no eixo que reivindicarem.
    let (mut px, mut py) = (None, None);
    if cfg.to_points && cfg.threshold > 0.0 {
        for &s in sources {
            for &t in &targets.points {
                for axis in 0..2 {
                    let delta = t[axis] - s[axis];
                    if delta.abs() > cfg.threshold {
                        continue;
                    }
                    let slot = if axis == 0 { &mut px } else { &mut py };
                    if slot.is_none_or(|b: SnapAxis| delta.abs() < b.delta.abs()) {
                        *slot = Some(SnapAxis {
                            delta,
                            source: s,
                            target: t,
                            grid: false,
                        });
                    }
                }
            }
        }
    }

    SnapResult {
        x: px.or(gx),
        y: py.or(gy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::rectangle;

    fn targets(pts: &[[f64; 2]]) -> SnapTargets {
        SnapTargets {
            points: pts.to_vec(),
        }
    }

    fn cfg() -> SnapConfig {
        SnapConfig {
            threshold: 1.0,
            ..SnapConfig::default()
        }
    }

    /// Uma grade quadrada de passo `step`, com um raio de magnetismo — o mesmo
    /// contrato do `GridSnapState` real (fora do raio ela não reivindica o ponto).
    fn square_grid(step: f64, radius: f64) -> impl FnMut([f64; 2]) -> Option<[f64; 2]> {
        move |p: [f64; 2]| {
            let g = [(p[0] / step).round() * step, (p[1] / step).round() * step];
            let d2 = (g[0] - p[0]).powi(2) + (g[1] - p[1]).powi(2);
            (d2 <= radius * radius).then_some(g)
        }
    }

    #[test]
    fn a_point_inside_the_threshold_snaps_exactly_onto_the_target() {
        let r = snap(&[[9.5, 20.4]], &targets(&[[10.0, 20.0]]), cfg(), None);
        assert_eq!(r.apply([9.5, 20.4]), [10.0, 20.0]);
        assert!(!r.x.unwrap().grid);
    }

    /// O ponto do lema: os eixos são independentes. Aqui só o X está no limiar —
    /// o Y desliza livre. Sem isso não dá pra alinhar bordas sem colar o objeto.
    #[test]
    fn each_axis_snaps_on_its_own() {
        let r = snap(&[[9.8, 50.0]], &targets(&[[10.0, 20.0]]), cfg(), None);
        assert_eq!(r.apply([9.8, 50.0]), [10.0, 50.0]);
        assert!(r.x.is_some() && r.y.is_none());
    }

    #[test]
    fn nothing_snaps_beyond_the_threshold_or_when_disabled() {
        assert!(
            snap(&[[8.0, 20.0]], &targets(&[[10.0, 20.4]]), cfg(), None)
                .x
                .is_none()
        );
        let off = SnapConfig {
            enabled: false,
            ..cfg()
        };
        // Desligado (Alt segurado) ignora forma E grade.
        let mut grid = square_grid(5.0, 1.0);
        let r = snap(
            &[[9.9, 20.0]],
            &targets(&[[10.0, 20.0]]),
            off,
            Some(&mut grid),
        );
        assert_eq!(r.delta(), [0.0, 0.0]);
    }

    #[test]
    fn the_nearest_candidate_wins_each_axis() {
        let r = snap(
            &[[10.0, 10.0]],
            &targets(&[[10.7, 9.4], [10.2, 10.9]]),
            cfg(),
            None,
        );
        assert_eq!(r.x.unwrap().target, [10.2, 10.9], "dx = 0.2 < 0.7");
        assert_eq!(r.y.unwrap().target, [10.7, 9.4], "dy = -0.6, |−0.6| < 0.9");
    }

    /// Vários pontos-fonte (a bbox de uma seleção): o melhor de todos vence.
    #[test]
    fn many_sources_offer_the_best_of_all() {
        let bbox = bbox_key_points([0.0, 0.0], [10.0, 10.0]);
        // Alvo colado no canto superior direito.
        let r = snap(&bbox, &targets(&[[10.3, 10.0]]), cfg(), None);
        assert!((r.x.unwrap().delta - 0.3).abs() < 1e-9);
        assert_eq!(
            r.x.unwrap().source,
            [10.0, 0.0],
            "o canto inferior-direito acha primeiro (empate → 1º)"
        );
        assert_eq!(r.y.unwrap().delta, 0.0);
    }

    /// A grade reivindica os DOIS eixos de uma vez — um ponto de rede é 2D, e
    /// decompor por eixo só faria sentido num grid quadrado (existem nove tipos).
    #[test]
    fn the_grid_claims_both_axes_as_one_lattice_point() {
        let mut grid = square_grid(5.0, 1.0);
        let r = snap(
            &[[9.6, 20.4]],
            &SnapTargets::default(),
            cfg(),
            Some(&mut grid),
        );
        assert_eq!(r.apply([9.6, 20.4]), [10.0, 20.0]);
        assert!(r.x.unwrap().grid && r.y.unwrap().grid);
        assert_eq!(
            r.x.unwrap().target,
            r.y.unwrap().target,
            "o mesmo ponto de rede"
        );
    }

    /// Fora do raio de magnetismo a grade não reivindica nada — o arrasto segue
    /// liso entre pontos de rede (comportamento Figma/Blender).
    #[test]
    fn outside_the_magnetism_radius_the_grid_stays_quiet() {
        let mut grid = square_grid(5.0, 1.0);
        let r = snap(
            &[[12.5, 12.5]],
            &SnapTargets::default(),
            cfg(),
            Some(&mut grid),
        );
        assert!(!r.any());
    }

    /// Forma vence régua, **por eixo**: o X encaixa na âncora vizinha e o Y fica
    /// com o ponto de rede. É o que deixa alinhar com o desenho sem perder a grade.
    #[test]
    fn shape_points_override_the_grid_axis_by_axis() {
        let mut grid = square_grid(5.0, 1.0);
        // X: âncora em 9.9 (delta −0.1) bate o grid 10.0 (delta +0.1)? Não — quem
        // decide não é a distância, é a precedência: o ponto SEMPRE sobrescreve.
        let r = snap(
            &[[9.8, 20.4]],
            &targets(&[[9.5, 99.0]]),
            cfg(),
            Some(&mut grid),
        );
        assert_eq!(r.apply([9.8, 20.4]), [9.5, 20.0]);
        assert!(!r.x.unwrap().grid, "X veio da forma");
        assert!(r.y.unwrap().grid, "Y veio da grade");
    }

    /// Com "Shapes" desligado no painel, só a grade opera.
    #[test]
    fn with_points_off_only_the_grid_speaks() {
        let mut grid = square_grid(5.0, 1.0);
        let c = SnapConfig {
            to_points: false,
            ..cfg()
        };
        let r = snap(&[[9.6, 20.4]], &targets(&[[9.5, 20.5]]), c, Some(&mut grid));
        assert_eq!(r.apply([9.6, 20.4]), [10.0, 20.0]);
    }

    #[test]
    fn bbox_key_points_are_corners_mids_and_center() {
        let k = bbox_key_points([0.0, 0.0], [10.0, 4.0]);
        assert_eq!(k.len(), 9);
        assert!(k.contains(&[0.0, 0.0]) && k.contains(&[10.0, 4.0]));
        assert!(k.contains(&[5.0, 2.0]), "centro");
        assert!(
            k.contains(&[5.0, 0.0]) && k.contains(&[0.0, 2.0]),
            "meios de aresta"
        );
    }

    #[test]
    fn collect_targets_skips_the_moving_paths_and_the_moving_anchors() {
        let mut scene = VecScene::new();
        let a = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
        let b = scene.push_path(rectangle([20.0, 0.0], [30.0, 10.0]));

        // Nada excluído: 4 âncoras + 9 pontos de bbox, por path.
        let all = collect_targets(&scene, &[], &[]);
        assert_eq!(all.points.len(), 2 * (4 + 9));

        // `a` inteiro fora (é ele que está sendo movido).
        let no_a = collect_targets(&scene, &[a], &[]);
        assert_eq!(no_a.points.len(), 4 + 9);
        assert!(!no_a.points.contains(&[0.0, 0.0]));

        // Uma âncora de `b` em movimento: ela sai E a bbox de `b` sai junto —
        // uma caixa que está sendo deformada não serve de referência.
        let deforming = collect_targets(&scene, &[], &[(b, 0)]);
        assert_eq!(deforming.points.len(), (4 + 9) + 3);
        assert!(!deforming.points.contains(&[20.0, 0.0]));
    }
}
