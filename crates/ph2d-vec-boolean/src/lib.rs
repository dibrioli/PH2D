#![forbid(unsafe_code)]
//! ph2d-vec-boolean — operações booleanas **edit-time** da pipeline vetorial nova
//! (ADR-0108, Fase 1). União / interseção / subtração / xor de regiões fechadas
//! via o motor exato `linesweeper` (MIT/Apache). É destrutivo por design: o
//! resultado é um path editável, não um nó vivo — sem boolean em runtime
//! (ADR-0108 §2 D4).
//!
//! **Buracos são de primeira classe.** O `linesweeper` devolve os contornos já
//! agrupados por containment (`Contours::grouped`): o de fora primeiro, os de
//! dentro depois. Cada grupo vira UM [`VecPath`] compound (`subpaths` +
//! [`FillRule::EvenOdd`]) — então subtrair um disco de outro dá uma rosquinha de
//! verdade, e não dois discos sólidos empilhados. `EvenOdd` é o que torna o
//! aninhamento robusto: alterna dentro/fora a cada contorno cruzado, sem depender
//! da orientação de nenhum deles.
//!
//! kurbo/linesweeper são internos a esta crate (não cruzam pro render, que fala a
//! kurbo do vello) → zero risco de version-skew.

use kurbo::{BezPath, PathEl, Point};
/// **O arranjo planar** — as FACES que o Shape Builder pinta. Módulo irmão: ele monta as
/// regiões COM as ops daqui, e é por isso que mora nesta crate (o motor é o mesmo).
pub mod arrangement;
pub use arrangement::{Arrangement, FaceId, MAX_BUILD_SHAPES, Membership};

use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{Contour, FillRule, VecPath, VecVertex, VertexKind};

/// Operação booleana entre duas regiões.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BoolOp {
    Union,
    Intersect,
    Subtract,
    /// Exclude (exclusive-or): a região coberta por exatamente UMA das duas
    /// (a sobreposição vira buraco). O 4º op canônico de Pathfinder.
    Exclude,
}

impl BoolOp {
    fn to_linesweeper(self) -> BinaryOp {
        match self {
            BoolOp::Union => BinaryOp::Union,
            BoolOp::Intersect => BinaryOp::Intersection,
            BoolOp::Subtract => BinaryOp::Difference,
            BoolOp::Exclude => BinaryOp::Xor,
        }
    }
}

/// Aplica `op` entre `a` e `b` (fechados). Atalho de [`apply_many`] para dois
/// paths: `a` é a base (a que sobra num `Subtract`) e `b`, sendo o do topo, doa o
/// estilo.
#[must_use]
pub fn apply(a: &VecPath, b: &VecPath, op: BoolOp) -> Vec<VecPath> {
    apply_many(&[a, b], op)
}

/// Booleana **N-ária** sobre `paths` em ordem de **z (fundo → topo)**.
///
/// O acumulador começa no path de trás e dobra sequencialmente sobre os demais —
/// o que dá exatamente a semântica do Pathfinder do Illustrator para as quatro
/// ops (`((a−b)−c) == a − (b∪c)`, e Union/Intersect/Xor são associativas). Ou
/// seja: **Subtract preserva o de trás e remove tudo que está por cima.** O estilo
/// (fill + stroke) vem do path do **topo**, também como no Illustrator.
///
/// Devolve os paths-resultado (compound quando há buraco); vazio se degenerar
/// (interseção de disjuntos) ou se `paths` tem < 2 entradas.
#[must_use]
pub fn apply_many(paths: &[&VecPath], op: BoolOp) -> Vec<VecPath> {
    let ([first, rest @ ..], Some(style)) = (paths, paths.last()) else {
        return Vec::new();
    };
    if rest.is_empty() {
        return Vec::new();
    }
    let lop = op.to_linesweeper();
    let mut acc = to_bez(first);
    // A regra de entrada do 1º fold precisa ler os contornos das FONTES. Só um
    // compound depende de EvenOdd (é o que faz seu contorno interno ser buraco);
    // um contorno único lê igual sob as duas regras — exceto se auto-intersectar,
    // caso em que NonZero (o default do modelo) é o que o render mostra.
    let mut acc_rule = first.fill_rule;
    let mut grouped: Vec<Vec<BezPath>> = Vec::new();
    for other in rest {
        let rule = if acc_rule == FillRule::EvenOdd || other.fill_rule == FillRule::EvenOdd {
            LsFillRule::EvenOdd
        } else {
            LsFillRule::NonZero
        };
        let Ok(contours) = linesweeper::binary_op(&acc, &to_bez(other), rule, lop) else {
            return Vec::new();
        };
        // O `grouped()` já separa "de fora + os que estão dentro dele". O último
        // agrupamento do fold vira o resultado; os intermediários viram o próximo
        // acumulador — cujos contornos o linesweeper já orientou, então dali em
        // diante NonZero e EvenOdd concordam.
        grouped = contours
            .grouped()
            .iter()
            .map(|g| g.iter().map(|&i| contours[i].path.clone()).collect())
            .collect();
        acc = BezPath::new();
        for c in grouped.iter().flatten() {
            acc.extend(c.iter());
        }
        acc_rule = FillRule::NonZero;
    }
    grouped
        .iter()
        .filter_map(|group| compound_from(group, style))
        .collect()
}

/// Um grupo de contornos (o 1º é o de fora, os demais estão dentro dele) vira um
/// `VecPath` compound com o estilo de `style`. `None` se o contorno externo
/// degenerar.
fn compound_from(group: &[BezPath], style: &VecPath) -> Option<VecPath> {
    let mut it = group.iter().filter_map(verts_from_bez);
    let outer = it.next()?;
    let subpaths: Vec<Contour> = it.map(Contour::new_closed).collect();
    let compound = !subpaths.is_empty();
    Some(VecPath {
        verts: outer,
        closed: true,
        fill: style.fill.clone(),
        stroke: style.stroke,
        subpaths,
        // Contorno único: as duas regras coincidem — mantém o default histórico.
        fill_rule: if compound {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        },
        ..VecPath::default()
    })
}

/// Comprimento abaixo do qual um handle (ou uma aresta) conta como degenerado.
const COINCIDENT_EPS: f64 = 1e-9;

/// Tolerância do **seno** do ângulo entre duas direções unitárias para chamá-las
/// colineares. Escala-livre: vale igual num ícone de 1 unidade e num layout de 10⁴.
const COLLINEAR_EPS: f64 = 1e-9;

#[inline]
fn kp(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

/// O segmento `a → b` é uma RETA? (os dois handles colapsados nas âncoras).
fn is_line(a: &VecVertex, b: &VecVertex) -> bool {
    close_enough(a.out_handle, a.anchor) && close_enough(b.in_handle, b.anchor)
}

/// `VecPath` → kurbo `BezPath`, **todos os contornos**. Um compound entra na
/// booleana com os buracos que já tinha.
///
/// Segmento reto vira `line_to`, **não** uma cúbica degenerada `(P0,P0,P3,P3)`.
/// Parece equivalente e não é: ao cortar essa cúbica num ponto de interseção, o
/// `linesweeper` devolve duas metades cujos pontos de controle caem a 1/3 e 2/3
/// da aresta. A geometria continua reta, mas cada quina volta como vértice
/// *Smooth* com handles pendurados no meio das arestas — pontos de controle que o
/// usuário nunca pediu. Com `line_to`, o corte de uma reta devolve retas.
fn to_bez(path: &VecPath) -> BezPath {
    // A booleana come a geometria COZIDA — ela corta a forma que está na tela. E o
    // resultado sai com as quinas ASSADAS (os verts novos nascem com raio 0): uma
    // booleana é destrutiva por natureza, e um raio vivo não sobrevive a ela — o
    // vértice que ele arredondava pode nem existir mais depois do corte. É o que a
    // Figma faz, e é o único contrato honesto.
    let path = &*path.cooked();
    let mut bp = BezPath::new();
    for c in 0..path.contour_count() {
        let Some((verts, _)) = path.contour(c) else {
            continue;
        };
        let Some(first) = verts.first() else {
            continue;
        };
        bp.move_to(kp(first.anchor));
        for w in verts.windows(2) {
            if is_line(&w[0], &w[1]) {
                bp.line_to(kp(w[1].anchor));
            } else {
                bp.curve_to(kp(w[0].out_handle), kp(w[1].in_handle), kp(w[1].anchor));
            }
        }
        if let Some(last) = verts.last().filter(|_| verts.len() >= 2) {
            // `close_path` já implica a reta de volta ao início — só o fechamento
            // CURVO precisa de um elemento próprio.
            if !is_line(last, first) {
                bp.curve_to(kp(last.out_handle), kp(first.in_handle), kp(first.anchor));
            }
        }
        bp.close_path();
    }
    bp
}

/// kurbo `BezPath` (um contorno) → vértices (âncoras + handles reconstruídos dos
/// elementos), **podado** por [`prune_contour`]. `None` se degenerar em < 3.
fn verts_from_bez(bp: &BezPath) -> Option<Vec<VecVertex>> {
    let mut verts: Vec<VecVertex> = Vec::new();
    for el in bp.elements() {
        match *el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => verts.push(VecVertex::corner([p.x, p.y])),
            PathEl::QuadTo(c, e) => {
                if let Some(prev) = verts.last().copied() {
                    let a = prev.anchor;
                    let c1 = [
                        a[0] + 2.0 / 3.0 * (c.x - a[0]),
                        a[1] + 2.0 / 3.0 * (c.y - a[1]),
                    ];
                    let c2 = [e.x + 2.0 / 3.0 * (c.x - e.x), e.y + 2.0 / 3.0 * (c.y - e.y)];
                    if let Some(pm) = verts.last_mut() {
                        pm.out_handle = c1;
                    }
                    verts.push(smooth([e.x, e.y], c2, [e.x, e.y]));
                }
            }
            PathEl::CurveTo(c1, c2, e) => {
                if let Some(pm) = verts.last_mut() {
                    pm.out_handle = [c1.x, c1.y];
                }
                verts.push(smooth([e.x, e.y], [c2.x, c2.y], [e.x, e.y]));
            }
            PathEl::ClosePath => {}
        }
    }
    let verts = prune_contour(verts);
    (verts.len() >= 3).then_some(verts)
}

/// Piso de vértices de um contorno — nunca podamos abaixo de um triângulo.
const MIN_CONTOUR_VERTS: usize = 3;

/// Remove os vértices que o sweep produziu mas a FORMA não precisa, e reclassifica
/// o tipo de cada âncora sobrevivente:
///
/// 1. **Âncoras coincidentes** (o contorno costuma repetir o 1º ponto no fim, e um
///    corte no vértice pode duplicá-lo) — funde, preservando os handles de fora.
/// 2. **Âncoras colineares** entre dois segmentos RETOS — o sweep insere um vértice
///    onde os operandos se tocam, mesmo no meio de uma aresta reta que segue reta
///    (a união de 3 quadrados enfileirados devolvia 10 pontos para um retângulo).
/// 3. **`VertexKind`**: Smooth só quando os DOIS handles existem e são colineares;
///    caso contrário Corner (quina/cusp). Sem isto toda quina volta "suave".
///
/// Curvas nunca são tocadas: um vértice com qualquer handle não-degenerado é
/// preservado, então a forma resultante é exata.
fn prune_contour(verts: Vec<VecVertex>) -> Vec<VecVertex> {
    // 1. Âncoras coincidentes, incluindo o wrap final→inicial.
    let mut out: Vec<VecVertex> = Vec::with_capacity(verts.len());
    for v in verts {
        match out.last_mut() {
            Some(prev) if close_enough(prev.anchor, v.anchor) => prev.out_handle = v.out_handle,
            _ => out.push(v),
        }
    }
    if out.len() >= 2 && close_enough(out[0].anchor, out[out.len() - 1].anchor) {
        let last = out.pop().unwrap();
        out[0].in_handle = last.in_handle;
    }

    // 2. Colineares retos (cíclico; remover um pode expor o vizinho, então volta
    //    um passo — o laço termina porque a lista só encolhe).
    let mut i = 0;
    while i < out.len() && out.len() > MIN_CONTOUR_VERTS {
        let n = out.len();
        let (p, c, q) = (out[(i + n - 1) % n], out[i], out[(i + 1) % n]);
        if is_redundant(&p, &c, &q) {
            out.remove(i);
            // Não avança: o vértice que deslizou para `i` agora encara o mesmo `p`.
            // Recua um passo para reavaliar o anterior no wrap-around.
            i = i.saturating_sub(1);
        } else {
            i += 1;
        }
    }

    // 3. Tipo real de cada âncora.
    for v in &mut out {
        v.kind = classify(v);
    }
    out
}

/// `c` está no meio de uma aresta RETA `p → q` e não a dobra? Então é supérfluo.
fn is_redundant(p: &VecVertex, c: &VecVertex, q: &VecVertex) -> bool {
    if !is_line(p, c) || !is_line(c, q) {
        return false; // um dos lados é curvo — o vértice carrega forma
    }
    let d1 = [c.anchor[0] - p.anchor[0], c.anchor[1] - p.anchor[1]];
    let d2 = [q.anchor[0] - c.anchor[0], q.anchor[1] - c.anchor[1]];
    let (l1, l2) = (d1[0].hypot(d1[1]), d2[0].hypot(d2[1]));
    if l1 < COINCIDENT_EPS || l2 < COINCIDENT_EPS {
        return true; // aresta de comprimento zero
    }
    // Seno do ângulo entre as arestas (escala-livre) + mesmo sentido (não é bico).
    let cross = (d1[0] * d2[1] - d1[1] * d2[0]).abs() / (l1 * l2);
    let dot = d1[0] * d2[0] + d1[1] * d2[1];
    cross <= COLLINEAR_EPS && dot > 0.0
}

/// Smooth exige os dois handles presentes E colineares (tangente contínua);
/// qualquer outra coisa é uma quina.
fn classify(v: &VecVertex) -> VertexKind {
    let (i, o) = (
        [v.in_handle[0] - v.anchor[0], v.in_handle[1] - v.anchor[1]],
        [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]],
    );
    let (li, lo) = (i[0].hypot(i[1]), o[0].hypot(o[1]));
    if li < COINCIDENT_EPS || lo < COINCIDENT_EPS {
        return VertexKind::Corner;
    }
    // Colineares e OPOSTOS (in de um lado da âncora, out do outro).
    let cross = (i[0] * o[1] - i[1] * o[0]).abs() / (li * lo);
    let dot = i[0] * o[0] + i[1] * o[1];
    if cross <= COLLINEAR_EPS && dot < 0.0 {
        VertexKind::Smooth
    } else {
        VertexKind::Corner
    }
}

fn smooth(anchor: [f64; 2], in_h: [f64; 2], out_h: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor,
        in_handle: in_h,
        out_handle: out_h,
        kind: VertexKind::Smooth,
        // Sem raio: a booleana comeu a geometria já COZIDA, então o arredondamento já
        // está ASSADO nestes vértices. Carregar o raio adiante o aplicaria duas vezes.
        corner_radius: 0.0,
    }
}

fn close_enough(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::{Paint, Rgba8};

    fn square(cx: f64, cy: f64, r: f64) -> VecPath {
        let v = |x: f64, y: f64| VecVertex::corner([x, y]);
        VecPath {
            verts: vec![
                v(cx - r, cy - r),
                v(cx + r, cy - r),
                v(cx + r, cy + r),
                v(cx - r, cy + r),
            ],
            closed: true,
            ..VecPath::default()
        }
    }

    #[test]
    fn union_of_overlapping_squares_is_nonempty() {
        let a = square(0.0, 0.0, 2.0);
        let b = square(2.0, 2.0, 2.0);
        let r = apply(&a, &b, BoolOp::Union);
        assert!(!r.is_empty());
        assert!(r[0].closed);
        assert!(r[0].verts.len() >= 4);
    }

    #[test]
    fn intersect_overlapping_is_nonempty_disjoint_is_empty() {
        let a = square(0.0, 0.0, 2.0);
        let b = square(2.0, 2.0, 2.0);
        assert!(!apply(&a, &b, BoolOp::Intersect).is_empty());
        let far = square(100.0, 100.0, 2.0);
        assert!(apply(&a, &far, BoolOp::Intersect).is_empty());
    }

    #[test]
    fn subtract_overlapping_is_nonempty() {
        let a = square(0.0, 0.0, 3.0);
        let b = square(2.0, 2.0, 2.0);
        assert!(!apply(&a, &b, BoolOp::Subtract).is_empty());
    }

    #[test]
    fn exclude_overlapping_is_nonempty_identical_is_empty() {
        let a = square(0.0, 0.0, 3.0);
        let b = square(2.0, 2.0, 3.0);
        // XOR of two overlapping squares leaves the non-shared regions.
        assert!(!apply(&a, &b, BoolOp::Exclude).is_empty());
        // XOR of a region with itself is empty (everything is shared).
        assert!(apply(&a, &a, BoolOp::Exclude).is_empty());
    }

    /// O caso que motivou o compound path: um quadrado menor DENTRO de um maior,
    /// subtraído, tem de virar UM path com buraco — não dois paths sólidos.
    #[test]
    fn subtract_inner_from_outer_yields_one_compound_with_a_hole() {
        let outer = square(0.0, 0.0, 4.0);
        let inner = square(0.0, 0.0, 2.0);
        let r = apply(&outer, &inner, BoolOp::Subtract);
        assert_eq!(r.len(), 1, "a rosquinha é UM path, não dois");
        assert!(r[0].is_compound(), "o buraco vive em subpaths");
        assert_eq!(r[0].subpaths.len(), 1);
        assert_eq!(r[0].fill_rule, FillRule::EvenOdd);
        // Buraco de verdade: o centro está FORA da região.
        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(r[0].clone());
        assert!(
            !scene.path_contains_point(id, [0.0, 0.0]),
            "centro é vazado"
        );
        assert!(scene.path_contains_point(id, [3.0, 0.0]), "anel é sólido");
    }

    /// Union de 3 discos: o fold N-ário roda em uma chamada, e o estilo vem do topo.
    #[test]
    fn union_many_folds_and_takes_the_top_style() {
        let a = square(0.0, 0.0, 2.0);
        let mut b = square(2.0, 0.0, 2.0);
        let mut c = square(4.0, 0.0, 2.0);
        b.fill = Some(Paint::solid(Rgba8::new(1, 2, 3, 255)));
        c.fill = Some(Paint::solid(Rgba8::new(9, 8, 7, 255)));
        let r = apply_many(&[&a, &b, &c], BoolOp::Union);
        assert_eq!(r.len(), 1, "os três se tocam → uma região só");
        assert_eq!(r[0].fill, c.fill, "estilo do topo (Illustrator)");
    }

    /// `Subtract` N-ário preserva o de trás: `a − (b ∪ c)`.
    #[test]
    fn subtract_many_removes_every_path_above_the_base() {
        let base = square(0.0, 0.0, 6.0);
        let b = square(-4.0, 0.0, 2.5);
        let c = square(4.0, 0.0, 2.5);
        let r = apply_many(&[&base, &b, &c], BoolOp::Subtract);
        assert!(!r.is_empty());
        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(r[0].clone());
        // Os dois mordidos saíram; o miolo ficou.
        assert!(scene.path_contains_point(id, [0.0, 0.0]));
        assert!(!scene.path_contains_point(id, [-5.0, 0.0]));
        assert!(!scene.path_contains_point(id, [5.0, 0.0]));
    }
    /// A regressão: 3 quadrados enfileirados unem num RETÂNGULO. O sweep insere um
    /// vértice em cada aresta onde os operandos se tocam, e todos caem no meio de
    /// uma aresta reta que segue reta — 10 pontos para uma forma de 4.
    #[test]
    fn union_of_collinear_squares_prunes_down_to_the_rectangle() {
        let r = apply_many(
            &[
                &square(0.0, 0.0, 2.0),
                &square(2.0, 0.0, 2.0),
                &square(4.0, 0.0, 2.0),
            ],
            BoolOp::Union,
        );
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].verts.len(), 4, "um retângulo tem 4 pontos");
        // E é o retângulo certo: [-2,6] × [-2,2].
        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(r[0].clone());
        let (lo, hi) = scene.path_bbox(id).unwrap();
        assert_eq!((lo, hi), ([-2.0, -2.0], [6.0, 2.0]));
        assert!(scene.path_contains_point(id, [5.0, 0.0]));
        assert!(!scene.path_contains_point(id, [7.0, 0.0]));
    }

    /// Uma aresta reta tem de voltar reta: handles COLAPSADOS na âncora e vértice
    /// `Corner`. Antes, `to_bez` mandava a reta como cúbica degenerada, o sweep a
    /// cortava, e as metades voltavam com pontos de controle a 1/3 e 2/3 da aresta —
    /// cada quina virava um `Smooth` com dois handles pendurados no meio do nada.
    #[test]
    fn straight_edges_come_back_as_corners_with_collapsed_handles() {
        let r = apply(
            &square(0.0, 0.0, 2.0),
            &square(2.0, 2.0, 2.0),
            BoolOp::Union,
        );
        let p = &r[0];
        assert_eq!(p.verts.len(), 8, "união diagonal = octógono");
        for v in &p.verts {
            assert_eq!(v.kind, VertexKind::Corner, "quina, não Smooth");
            assert_eq!(v.in_handle, v.anchor, "handle colapsado");
            assert_eq!(v.out_handle, v.anchor);
        }
    }

    /// Podar não pode comer curva: um vértice com QUALQUER handle não-degenerado é
    /// preservado, e a forma continua exata.
    #[test]
    fn curved_contours_keep_every_control_point() {
        let circle = |cx: f64, cy: f64, rad: f64| ph2d_vec_scene::ellipse([cx, cy], rad, rad);
        let r = apply(
            &circle(0.0, 0.0, 4.0),
            &circle(0.0, 0.0, 2.0),
            BoolOp::Subtract,
        );
        assert_eq!(r.len(), 1);
        let p = &r[0];
        // Anel: 4 pontos por contorno, todos suaves (o círculo-Bézier canônico).
        assert_eq!((p.verts.len(), p.subpaths[0].verts.len()), (4, 4));
        assert!(p.verts_all().all(|v| v.kind == VertexKind::Smooth));
        assert!(
            p.verts_all()
                .all(|v| v.in_handle != v.anchor && v.out_handle != v.anchor)
        );

        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(p.clone());
        assert!(scene.path_contains_point(id, [3.0, 0.0]), "o anel é sólido");
        assert!(
            !scene.path_contains_point(id, [0.0, 0.0]),
            "o furo é vazado"
        );
        assert!(!scene.path_contains_point(id, [4.5, 0.0]), "fora do disco");
    }

    /// Mistura: quadrado menos círculo = contorno externo de quinas + furo curvo.
    /// Cada contorno é classificado por conta própria.
    #[test]
    fn a_mixed_result_keeps_corners_and_curves_apart() {
        let circle = ph2d_vec_scene::ellipse([0.0, 0.0], 2.0, 2.0);
        let r = apply(&square(0.0, 0.0, 4.0), &circle, BoolOp::Subtract);
        let p = &r[0];
        assert!(p.verts.iter().all(|v| v.kind == VertexKind::Corner));
        assert_eq!(p.verts.len(), 4);
        assert!(
            p.subpaths[0]
                .verts
                .iter()
                .all(|v| v.kind == VertexKind::Smooth)
        );
    }

    /// Um bico (a curva volta sobre si) NÃO é colinear-redundante: `dot > 0` exige
    /// que as arestas sigam no mesmo sentido, então a ponta sobrevive à poda.
    #[test]
    fn prune_keeps_a_spike_and_drops_a_zero_length_edge() {
        use ph2d_vec_scene::VecVertex as V;
        // Bico: (0,0) → (4,0) → volta a (2,0) → (2,4). O (4,0) é a ponta.
        let spike = prune_contour(vec![
            V::corner([0.0, 0.0]),
            V::corner([4.0, 0.0]),
            V::corner([2.0, 0.0]),
            V::corner([2.0, 4.0]),
        ]);
        assert_eq!(spike.len(), 4, "a ponta do bico é forma, não ruído");
        // Aresta de comprimento zero + colinear no meio: some.
        let flat = prune_contour(vec![
            V::corner([0.0, 0.0]),
            V::corner([2.0, 0.0]), // colinear
            V::corner([4.0, 0.0]),
            V::corner([4.0, 0.0]), // duplicado
            V::corner([4.0, 4.0]),
            V::corner([0.0, 4.0]),
        ]);
        assert_eq!(flat.len(), 4);
        assert_eq!(flat[0].anchor, [0.0, 0.0]);
        assert_eq!(flat[1].anchor, [4.0, 0.0]);
    }
}
