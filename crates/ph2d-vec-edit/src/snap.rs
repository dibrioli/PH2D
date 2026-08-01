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
//!
//! # Duas espécies de reivindicação (plano 25 §9, a W6)
//!
//! O que está acima é **ALINHAMENTO**: uma restrição 1-D por eixo, e é por isso que ela se
//! decompõe (o X vem de uma vizinha, o Y da grade, e o resultado faz sentido — são duas retas
//! que se cruzam). Encaixar **sobre uma curva** não é disso: é uma **POSIÇÃO**, uma restrição
//! 0-D. "Alinhar meu X com o X do ponto mais próximo daquela curva" não quer dizer nada — todo
//! X dentro da faixa da curva é o X de algum ponto dela. Uma posição vence os dois eixos ou
//! nenhum.
//!
//! ⚠️ **A lei que mantém as quinas alcançáveis:** a curva passa POR CIMA de cada âncora, então
//! perto de um vértice as duas espécies competem — e se a posição vencesse sempre, o nó pousaria
//! a fração de pixel do canto, para sempre, sem gesto que corrigisse. A regra é *vértice vence
//! curva* (a mesma do Inkscape), enunciada aqui como propriedade do **RESULTADO**: se o
//! alinhamento já pousa exactamente sobre UM alvo (os dois eixos vindos do mesmo ponto), isso
//! **é** uma coincidência com um ponto distinto, e a reivindicação 2-D **se retira**.
//!
//! O corolário é o que torna a mudança segura: sem curvas na lista de alvos, esta lei nunca
//! dispara, e o encaixe é **byte-idêntico** ao que já shipava.

use ph2d_vec_scene::curve_probe::{CubicSeg, crossings_near, nearest_on_segs, world_segs};
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
    /// Encaixar **sobre** a geometria — reivindicação de POSIÇÃO (os dois eixos).
    pub to_path: bool,
    /// Encaixar nos **cruzamentos** entre curvas. Também posição, e distinta: um cruzamento
    /// é um ponto que o desenho produziu, não um lugar qualquer do contínuo.
    pub to_crossings: bool,
    /// Distância máxima de encaixe em forma, por eixo. Nas reivindicações de posição ela é
    /// um RAIO — o alinhamento captura num quadrado, a posição num círculo, porque uma diz
    /// respeito a cada eixo sozinho e a outra ao ponto.
    pub threshold: f64,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            to_points: true,
            to_path: false,
            to_crossings: false,
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
    /// Alvos de ALINHAMENTO (1-D por eixo): âncoras e pontos-chave de bbox.
    pub points: Vec<[f64; 2]>,
    /// A geometria, em MUNDO, para as reivindicações de POSIÇÃO. Guardada como segmentos
    /// cúbicos e não como `VecPath` clonada: a projeção nunca lê estilo nem efeitos, e esta
    /// lista é reconstruída por gesto.
    ///
    /// ⚠️ Os **cruzamentos não são um terceiro campo**: eles são derivados daqui na hora da
    /// consulta, localizados em volta do cursor. Guardá-los seria memória que envelhece.
    pub segs: Vec<CubicSeg>,
}

/// De onde veio um encaixe. É o que decide como a guia se desenha, e as quatro espécies
/// dizem coisas diferentes ao artista: *alinhei com aquilo* · *caí na régua* · *estou SOBRE
/// esta linha* · *estou no ponto onde duas linhas se cruzam*.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapSource {
    /// Um ponto de outra forma, por EIXO (âncora ou canto de bbox).
    Shape,
    /// A grade universal do editor.
    Grid,
    /// Um ponto **sobre** a geometria (posição, os dois eixos).
    Curve,
    /// Um **cruzamento** entre curvas (posição, os dois eixos).
    Crossing,
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
    /// De onde veio.
    pub from: SnapSource,
}

impl SnapAxis {
    /// Encaixou na grade? (o predicado que o desenho das guias fazia com o antigo campo
    /// booleano — mantido como pergunta, não como estado.)
    #[must_use]
    pub fn is_grid(&self) -> bool {
        self.from == SnapSource::Grid
    }
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
///
/// `curves` liga a coleta da GEOMETRIA (os alvos de posição). É um parâmetro e não sempre-
/// ligado porque este recolhimento roda por movimento do gizmo: quem não ligou "Path" nem
/// "Crossings" no painel não paga por percorrer os contornos da cena.
#[must_use]
pub fn collect_targets(
    scene: &VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    skip_paths: &[VecPathId],
    skip_verts: &[(VecPathId, usize)],
    curves: bool,
) -> SnapTargets {
    let mut points = Vec::new();
    let mut segs = Vec::new();
    for path in scene.paths() {
        if skip_paths.contains(&path.id) {
            continue;
        }
        // Alvos de snap são pontos que o usuário VÊ: mundo, não local (ADR-0111).
        let xf = ph2d_vec_scene::xform_of(xforms, path.id);
        let mut deformed = false;
        for (i, v) in path.verts_all().enumerate() {
            if skip_verts.contains(&(path.id, i)) {
                deformed = true;
            } else {
                points.push(xf.apply(v.anchor));
            }
        }
        if deformed {
            // Uma forma cuja geometria está mudando debaixo do gesto não é referência —
            // nem a caixa dela, nem as curvas.
            continue;
        }
        if let Some((lo, hi)) = scene.path_curve_bbox(path.id) {
            // Os 9 pontos-chave são do bbox LOCAL; sobem um a um (uma forma girada
            // dá um quadrilátero, e os pontos dele seguem sendo os cantos/meios).
            for kp in bbox_key_points(lo, hi) {
                points.push(xf.apply(kp));
            }
        }
        if curves {
            world_segs(path, &xf, &mut segs);
        }
    }
    SnapTargets { points, segs }
}

/// A reivindicação de POSIÇÃO: o ponto 2-D que vence os dois eixos, se houver.
///
/// **Cruzamento antes de curva**, e não por gosto: perto de um cruzamento as duas curvas
/// passam por ali, então a distância à curva e a distância ao cruzamento são iguais a menos
/// de ruído de `f64` — decidir por proximidade seria cara-ou-coroa, e metade das vezes o nó
/// pousaria ao lado do ponto que o artista mirava.
#[must_use]
fn position_claim(
    sources: &[[f64; 2]],
    targets: &SnapTargets,
    cfg: SnapConfig,
) -> Option<(SnapSource, [f64; 2], [f64; 2])> {
    if targets.segs.is_empty() || cfg.threshold <= 0.0 {
        return None;
    }
    let mut best: Option<(f64, [f64; 2], [f64; 2])> = None;
    if cfg.to_crossings {
        for &s in sources {
            for x in crossings_near(&targets.segs, s, cfg.threshold) {
                keep_nearest(&mut best, s, x);
            }
        }
        if let Some((_, s, t)) = best {
            return Some((SnapSource::Crossing, s, t));
        }
    }
    if cfg.to_path {
        for &s in sources {
            if let Some(p) = nearest_on_segs(&targets.segs, s, cfg.threshold) {
                keep_nearest(&mut best, s, p);
            }
        }
        if let Some((_, s, t)) = best {
            return Some((SnapSource::Curve, s, t));
        }
    }
    None
}

/// Guarda o par `(fonte, alvo)` de menor distância. Empate fica com o primeiro, como no
/// alinhamento — determinístico.
fn keep_nearest(best: &mut Option<(f64, [f64; 2], [f64; 2])>, s: [f64; 2], t: [f64; 2]) {
    let d2 = (t[0] - s[0]).powi(2) + (t[1] - s[1]).powi(2);
    if best.is_none_or(|(bd, _, _)| d2 < bd) {
        *best = Some((d2, s, t));
    }
}

/// Resolve o encaixe de `sources` contra `targets` e, opcionalmente, contra a
/// grade do editor.
///
/// A grade propõe **os dois eixos de uma vez** (um ponto de rede é 2D; decompor
/// por eixo só faria sentido numa grade quadrada, e existem nove tipos). Os pontos
/// das outras formas então sobrescrevem eixo a eixo — cada um escolhendo o
/// candidato de **menor** deslocamento dentro do limiar, independente do outro.
/// Empate fica com o primeiro (determinístico).
///
/// A reivindicação de POSIÇÃO (curva/cruzamento) entra por último e vence os dois eixos de
/// uma vez — **exceto** quando o alinhamento já pousou exactamente sobre um alvo, que é a
/// lei *vértice vence curva* do cabeçalho do módulo.
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
                    from: SnapSource::Grid,
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
                            from: SnapSource::Shape,
                        });
                    }
                }
            }
        }
    }

    // 3. A reivindicação de POSIÇÃO. Ela se retira quando o alinhamento já pousou
    //    exactamente sobre um alvo — *vértice vence curva*, enunciado sobre o resultado.
    let coincident = match (px, py) {
        (Some(a), Some(b)) => a.target == b.target,
        _ => false,
    };
    if !coincident && let Some((from, s, t)) = position_claim(sources, targets, cfg) {
        let axis = |k: usize| {
            Some(SnapAxis {
                delta: t[k] - s[k],
                source: s,
                target: t,
                from,
            })
        };
        return SnapResult {
            x: axis(0),
            y: axis(1),
        };
    }

    SnapResult {
        x: px.or(gx),
        y: py.or(gy),
    }
}

#[cfg(test)]
#[path = "snap_tests.rs"]
mod tests;
