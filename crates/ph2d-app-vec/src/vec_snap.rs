//! ⭐ **A LEI do snap vectorial, fora da shell** — W2/L4, A2 (desfazer o acoplamento).
//!
//! Aqui mora o que o encaixe DECIDE; na shell fica só o que ele precisa de PERGUNTAR à janela.
//! O corte é exactamente esse, e é verificável: nada neste ficheiro menciona `App`, `gfx`,
//! painel ou câmera.
//!
//! # ⚠️ O módulo chama-se `vec_snap` de propósito
//!
//! Ele era `shells/desktop/src/vec_snap.rs`, e o nome do módulo entra no **nome de cada teste**.
//! Mantê-lo faz a prova de que nenhum teste se perdeu (`scripts/nextest-list-diff.py`) passar
//! mesmo a `--depth 2` (`módulo::fn`), e não só no `--depth 1` por omissão. *Renomear um módulo
//! de testes durante uma extracção é indistinguível, na lista, de perder os testes dele.*
//!
//! # O que FICOU na shell, e porquê (a lista que o substrato tem de cobrir)
//!
//! | método | o que ele precisa e não é da família |
//! |---|---|
//! | `vec_px_to_world` | `gfx.surface` + `gfx.camera` — e ⚠️ ele não é do vector: **11 ficheiros** de outras famílias o chamam |
//! | `vec_snap_point` · `vec_snap_move` | `gfx.hero_screen.grid` — estado de PAINEL |
//! | `vec_rebuild_snap_targets` | `gfx.guides` + a orquestração do gesto |
//! | `dragged_entity_bits` · `dragged_vec_path_ids` · `snap_dragged_vec_during_drag` | `gfx.hero_screen.gizmo` |
//! | `sprite_snap_points` · `vec_move_sources` | `crate::vec_transform`, que **não pôde sair** (declara um `#[path]` de um teste que precisa de `crate::profile_live`) |

use ph2d_ecs::{Entity, SimWorld, VecPathRef};
use ph2d_editor::grid_snap::GridSnapState;
use ph2d_vec_edit::snap::{SnapConfig, SnapResult, SnapSource};
use ph2d_vec_render::{Guide, GuideKind};
use ph2d_vec_scene::VecPathId;

/// Distância máxima de encaixe EM FORMA, em pixels de tela (convertida para world
/// pelo zoom). A grade tem o raio de magnetismo dela. 10px dá um ímã confortável
/// (8 era tímido p/ mouse); Alt ignora o snap quando atrapalha.
const SNAP_PX: f64 = 10.0;

/// ⭐⭐ **A FOLGA COM QUE SOLDAR DECIDE QUE DUAS PONTAS SÃO O MESMO NÓ** — em unidades de MUNDO.
///
/// ⚠️ **É o MESMO ímã do encaixe, e a razão é uma só resposta por pergunta.** O app já decide, a
/// cada traço desenhado, que duas coisas a menos de [`SNAP_PX`] na tela são para ficar no mesmo
/// sítio; soldar reusa esse veredito em vez de inventar um segundo número que divergiria dele no
/// dia em que um dos dois mudasse. Report do Enio (2026-09-01): *"ainda não consegue conectar as
/// duas curvas … as linhas não compartilham o mesmo nó"*.
///
/// ⛔ **Não é o `WELD_TOL`** (`1e-6`, exacto), que responde a outra pergunta: *estas duas pontas JÁ
/// são o mesmo nó?* — depois de a solda as ter fundido numa coordenada só. Uma delas mede intenção,
/// a outra mede um facto.
#[must_use]
pub fn vec_weld_tolerance(px_to_world: f64) -> f64 {
    SNAP_PX * px_to_world
}

/// Ajustes de snap do módulo vetorial. Só o encaixe em FORMAS — a grade é do
/// subsistema universal.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VecSnapSettings {
    /// Encaixar em âncoras / bbox das outras formas.
    pub on: bool,
    /// Encaixar **sobre** a geometria.
    pub path: bool,
    /// Encaixar nos cruzamentos entre curvas.
    pub crossings: bool,
    /// Encaixar nas **guias** do documento.
    ///
    /// ⚠️ Nasce LIGADO, ao contrário das duas de posição: uma guia só existe porque o artista
    /// a arrastou até lá, então o ímã dela não muda o app debaixo de ninguém — num documento
    /// sem guias este interruptor é inerte por construção.
    pub guides: bool,
}

impl Default for VecSnapSettings {
    fn default() -> Self {
        // ⚠️ As duas reivindicações de POSIÇÃO nascem DESLIGADAS, e é decisão de produto: um
        // ímã que agarra a linha inteira (e não só os pontos notáveis dela) muda como todo
        // gesto de desenho se comporta, e ligá-lo por default mudaria o app debaixo de quem
        // não pediu. Ficam a um clique, na seção que já existe.
        Self {
            on: true,
            path: false,
            crossings: false,
            guides: true,
        }
    }
}

/// Pergunta à grade universal onde `p` encaixa. `None` = grade desligada, ou o
/// ponto está fora do raio de magnetismo dela (o arrasto segue liso entre pontos
/// de rede). `sprite_half_size` é `[0, 0]`: encaixamos pontos, não sprites.
pub fn ask_grid(state: &mut GridSnapState, p: [f64; 2]) -> Option<[f64; 2]> {
    if !state.snap_enabled {
        return None;
    }
    let w = [p[0] as f32, p[1] as f32];
    let s = state.snap_world(w, [0.0, 0.0]);
    (s != w).then(|| [f64::from(s[0]), f64::from(s[1])])
}

/// As guias de um encaixe. Para um eixo X encaixado, a fonte deslocada divide o
/// `x` com o alvo → o segmento sai vertical, e é a linha que o usuário vê.
///
/// As três espécies que **não** são alinhamento não têm forma do outro lado a que ligar: a
/// guia degenera num ponto (`a == b`) e o desenho vira a marca da espécie. A grade e as duas
/// reivindicações de posição reclamam os dois eixos com o MESMO alvo, então a deduplicação
/// deixa uma marca só — não duas sobrepostas.
#[must_use]
pub fn guides_of(r: &SnapResult) -> Vec<Guide> {
    let d = r.delta();
    let mut out = Vec::new();
    for axis in [r.x, r.y] {
        let Some(a) = axis else { continue };
        let moved = [a.source[0] + d[0], a.source[1] + d[1]];
        let kind = match a.from {
            SnapSource::Shape => GuideKind::Align,
            SnapSource::Grid => GuideKind::Grid,
            SnapSource::Curve => GuideKind::Curve,
            SnapSource::Crossing => GuideKind::Crossing,
            SnapSource::Guide => GuideKind::GuideHit,
        };
        let guide = Guide {
            a: moved,
            b: if kind == GuideKind::Align {
                a.target
            } else {
                moved
            },
            kind,
        };
        if !out.contains(&guide) {
            out.push(guide);
        }
    }
    out
}

/// O que `snap_dragged_vec_during_drag` faz com as guias, por tipo de arraste do
/// gizmo. Pura p/ ser testável sem gfx — trava a política de snap-por-gesto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DragSnap {
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
pub fn drag_snap_kind(kind: ph2d_editor::GizmoDragKind) -> DragSnap {
    use ph2d_editor::GizmoDragKind::{MovePivot, Rotate, ScaleCorner, ScaleEdge, Translate};
    match kind {
        Translate => DragSnap::SlideTranslate,
        ScaleCorner { .. } | ScaleEdge { .. } => DragSnap::GizmoOwnsGuides,
        Rotate | MovePivot => DragSnap::ClearGuides,
    }
}

/// A configuração de snap EM FORMA para este quadro — **pura**, e por isso gateável.
///
/// `bypass` é o Alt segurado, que desliga tudo (forma e grade) como no Figma. Ele chega como
/// **booleano** e não como `&Modifiers`: o estado do teclado é da shell, a política não.
///
/// ⚠️ Era um método de `App` (`vec_snap_cfg`) e, por isso, **não tinha um único teste** — para o
/// exercitar era preciso uma `App`, que precisa de uma janela. Desacoplar não é arrumação aqui:
/// é o que torna a lei alcançável de um gate.
#[must_use]
pub fn snap_cfg(settings: &VecSnapSettings, bypass: bool, px_to_world: f64) -> SnapConfig {
    SnapConfig {
        enabled: !bypass,
        to_points: settings.on,
        to_path: settings.path,
        to_crossings: settings.crossings,
        to_guides: settings.guides,
        threshold: SNAP_PX * px_to_world,
    }
}

/// A cena precisa oferecer a GEOMETRIA neste gesto?
///
/// ⚠️ **Porta única**: quem RECOLHE os alvos e quem os RESOLVE perguntam a esta mesma função. Sem
/// ela, o recolhimento pararia de trazer as curvas no dia em que um terceiro interruptor de
/// posição nascesse — e o sintoma seria um ímã que existe no painel e não agarra nada.
#[must_use]
pub fn wants_curves(settings: &VecSnapSettings) -> bool {
    settings.path || settings.crossings
}

/// Os `VecPathId` das entidades `bits` que são vectoriais (o resto é ignorado em silêncio, que
/// é o correcto: um gizmo arrasta sprites e formas na mesma árvore — ADR-0110).
///
/// Precisa do MUNDO e de mais nada, e é por isso que pôde sair da `App`.
#[must_use]
pub fn ids_of_bits(sim: &SimWorld, bits: &[u64]) -> Vec<VecPathId> {
    bits.iter()
        .filter_map(|&b| {
            sim.world()
                .get::<VecPathRef>(Entity::from_bits(b))
                .map(|v| v.0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{DragSnap, drag_snap_kind, guides_of};
    use ph2d_editor::GizmoDragKind;
    use ph2d_vec_edit::snap::{SnapAxis, SnapResult, SnapSource};
    use ph2d_vec_render::GuideKind;

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
                from: SnapSource::Shape,
            }),
            y: None,
        };
        let g = guides_of(&r);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].a, [10.0, 100.0], "a fonte, já encaixada");
        assert_eq!(g[0].b, [10.0, 20.0], "o alvo");
        assert_eq!(g[0].a[0], g[0].b[0], "vertical");
        assert_eq!(g[0].kind, GuideKind::Align);
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
                from: SnapSource::Shape,
            }),
            y: Some(SnapAxis {
                delta: -2.0,
                source: [9.0, 5.0],
                target: [80.0, 3.0],
                from: SnapSource::Shape,
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
            from: SnapSource::Grid,
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
        assert_eq!(g[0].kind, GuideKind::Grid);
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
                from: SnapSource::Shape,
            }),
            y: Some(SnapAxis {
                delta: -0.4,
                source: [9.8, 20.4],
                target: [10.0, 20.0],
                from: SnapSource::Grid,
            }),
        };
        let g = guides_of(&r);
        assert_eq!(g.len(), 2);
        assert!(
            g[0].kind == GuideKind::Align && g[0].a[0] == g[0].b[0],
            "linha vertical da forma"
        );
        assert!(
            g[1].kind == GuideKind::Grid && g[1].a == g[1].b,
            "cruz da grade"
        );
        assert_eq!(g[1].a, [9.5, 20.0], "onde a forma parou de verdade");
    }
}

#[cfg(test)]
mod cfg_tests {
    use super::{VecSnapSettings, snap_cfg, wants_curves};

    /// **O Alt desliga o encaixe INTEIRO, não os interruptores.** A distinção importa: se o bypass
    /// apagasse os campos em vez de `enabled`, largar o Alt devolveria uma configuração com tudo
    /// desligado — e o artista perderia os ajustes por ter usado o atalho uma vez.
    #[test]
    fn o_bypass_desliga_o_encaixe_e_preserva_os_interruptores() {
        let s = VecSnapSettings::default();
        let com = snap_cfg(&s, true, 1.0);
        assert!(!com.enabled, "Alt segurado tem de desligar o encaixe");
        assert_eq!(
            (com.to_points, com.to_guides),
            (s.on, s.guides),
            "o bypass nao pode reescrever os interruptores do painel"
        );
    }

    /// **O limiar é em PIXELS de tela, convertido pelo zoom.** Um limiar fixo em mundo faria o ímã
    /// agarrar meia tela quando afastado e nada quando aproximado.
    #[test]
    fn o_limiar_segue_o_zoom() {
        let s = VecSnapSettings::default();
        let perto = snap_cfg(&s, false, 0.5).threshold;
        let longe = snap_cfg(&s, false, 2.0).threshold;
        assert!(
            (longe / perto - 4.0).abs() < 1e-12,
            "o limiar tem de ser linear no world-por-pixel: {perto} vs {longe}"
        );
    }

    /// **A porta única responde a QUALQUER das duas reivindicações de posição.** Um `&&` aqui
    /// deixaria o encaixe sobre a curva morto enquanto os cruzamentos estivessem desligados —
    /// o modo de falha exacto que o doc-comment da porta nomeia.
    #[test]
    fn qualquer_reivindicacao_de_posicao_pede_a_geometria() {
        let mut s = VecSnapSettings::default();
        assert!(!wants_curves(&s), "por omissao as duas estao desligadas");
        s.path = true;
        assert!(wants_curves(&s), "so' `path` basta");
        s.path = false;
        s.crossings = true;
        assert!(wants_curves(&s), "so' `crossings` basta");
    }
}
