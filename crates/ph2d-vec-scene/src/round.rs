//! A família do CÍRCULO — módulo irmão de [`crate::shapes`].
//!
//! Elipse, **arco**, **pizza** (wedge), **rosquinha** (anel/donut) e **segmento** (a
//! corda) não são cinco formas: são pontos do mesmo espaço de três parâmetros — *onde o
//! corte começa* (`start`), *quanto ele abre* (`sweep`) e *quanto do miolo é furo*
//! (`inner`). É o modelo do Figma, e vale a pena porque o usuário chega na forma que
//! quer **arrastando um número**, em vez de procurar a primitiva certa num catálogo:
//!
//! | | `inner = 0` | `inner > 0` |
//! |---|---|---|
//! | **`sweep = 360`** | elipse | rosquinha |
//! | **`sweep < 360`** | pizza | anel parcial (arco grosso) |
//!
//! O **segmento** é o único que não sai daí (a corda fecha o arco em linha reta, não
//! pelo centro), então ele é uma forma própria.
//!
//! Toda a geometria é de vértices SUAVES com handles bézier exatos (`(4/3)·tan(α/4)·r`
//! por segmento de ≤90°) — o mesmo motor do `arc`, então o contorno é liso ao renderizar
//! e continua editável ponto a ponto depois de um "Convert to Curves".

use crate::{Contour, FillRule, VecPath, VecVertex, VertexKind};

/// Uma volta inteira, em graus.
pub const FULL_TURN: f64 = 360.0;

/// Os vértices de um arco elíptico de `sweep` graus a partir de `start` (0° = 3h,
/// anti-horário). `reverse` percorre o arco ao contrário — é o que faz o contorno
/// interno de um anel correr em sentido oposto ao externo, para o furo ser furo.
///
/// Segmenta em pedaços de ≤90° e usa o comprimento de handle exato de cada um, a
/// generalização do `KAPPA` (que é esse valor para 90°).
#[must_use]
pub(crate) fn arc_verts(
    center: [f64; 2],
    rx: f64,
    ry: f64,
    start_deg: f64,
    sweep_deg: f64,
    reverse: bool,
) -> Vec<VecVertex> {
    let (cx, cy) = (center[0], center[1]);
    let start = start_deg.to_radians();
    let total = sweep_deg.abs().min(FULL_TURN).to_radians();
    let n_seg = (total / std::f64::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let seg = total / n_seg as f64;
    let h = (4.0 / 3.0) * (seg / 4.0).tan();
    let mut verts = Vec::with_capacity(n_seg + 1);
    for i in 0..=n_seg {
        let step = if reverse { n_seg - i } else { i };
        let a = start + seg * step as f64;
        let (s, c) = a.sin_cos();
        let anchor = [cx + rx * c, cy + ry * s];
        // Tangente ao arco em `a` é (−sin, cos); o handle é ela × h × raio. Ao percorrer
        // ao contrário, a tangente inverte — senão os handles apontariam para trás e o
        // contorno interno do anel sairia com laços.
        let sign = if reverse { -1.0 } else { 1.0 };
        let (tx, ty) = (-rx * s * h * sign, ry * c * h * sign);
        verts.push(VecVertex::smooth(
            anchor,
            [anchor[0] - tx, anchor[1] - ty],
            [anchor[0] + tx, anchor[1] + ty],
        ));
    }
    verts
}

/// `sweep` normalizado: `≤ 0` (um save feito antes de a elipse ganhar o parâmetro) é uma
/// volta INTEIRA, não uma forma vazia. Sem isto, toda elipse já salva sumiria.
#[must_use]
fn full_if_unset(sweep_deg: f64) -> f64 {
    if sweep_deg <= 0.0 {
        FULL_TURN
    } else {
        sweep_deg.min(FULL_TURN)
    }
}

/// **A família inteira num construtor.** `sweep` graus a partir de `start`, com um furo
/// central de `inner` (fração do raio, `0` = sem furo):
///
/// - volta inteira, sem furo ⇒ **elipse**;
/// - volta inteira, com furo ⇒ **rosquinha** (dois contornos, furo por `EvenOdd`);
/// - parcial, sem furo ⇒ **pizza** (fecha pelo centro);
/// - parcial, com furo ⇒ **anel parcial** (fecha pelas duas bordas radiais).
#[must_use]
pub fn ellipse_sweep(
    center: [f64; 2],
    rx: f64,
    ry: f64,
    start_deg: f64,
    sweep_deg: f64,
    inner: f64,
) -> VecPath {
    let sweep = full_if_unset(sweep_deg);
    let inner = inner.clamp(0.0, 0.95);
    let full = sweep >= FULL_TURN - 1e-9;
    let (irx, iry) = (rx * inner, ry * inner);

    if full {
        let outer = arc_verts(center, rx, ry, start_deg, FULL_TURN, false);
        // A volta fecha sobre si: o último vértice coincide com o primeiro.
        let outer = outer[..outer.len() - 1].to_vec();
        if inner <= 0.0 {
            return VecPath {
                verts: outer,
                closed: true,
                ..VecPath::default()
            };
        }
        // ROSQUINHA: contorno interno em sentido OPOSTO, como subpath. `EvenOdd` faz o
        // miolo virar furo (com `NonZero` e os dois no mesmo sentido, o furo sumiria).
        let inner_v = arc_verts(center, irx, iry, start_deg, FULL_TURN, true);
        let inner_v = inner_v[..inner_v.len() - 1].to_vec();
        return VecPath {
            verts: outer,
            subpaths: vec![Contour {
                verts: inner_v,
                closed: true,
            }],
            fill_rule: FillRule::EvenOdd,
            closed: true,
            ..VecPath::default()
        };
    }

    // PARCIAL. Sem furo, fecha pelo CENTRO (pizza); com furo, pelo arco interno (o
    // contorno vai: arco externo → borda radial → arco interno de volta → borda radial).
    let mut outer_arc = arc_verts(center, rx, ry, start_deg, sweep, false);
    cap_arc_ends(&mut outer_arc);
    let mut verts = outer_arc;
    if inner <= 0.0 {
        verts.push(VecVertex::corner(center));
    } else {
        let mut inner_arc = arc_verts(center, irx, iry, start_deg, sweep, true);
        cap_arc_ends(&mut inner_arc);
        verts.extend(inner_arc);
    }
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// **Segmento circular** (a corda): o arco fechado por uma linha RETA entre as pontas —
/// não pelo centro, que é o que o distingue da pizza.
#[must_use]
pub fn ellipse_chord(
    center: [f64; 2],
    rx: f64,
    ry: f64,
    start_deg: f64,
    sweep_deg: f64,
) -> VecPath {
    let sweep = full_if_unset(sweep_deg);
    let mut verts = arc_verts(center, rx, ry, start_deg, sweep, false);
    cap_arc_ends(&mut verts);
    VecPath {
        verts,
        closed: true, // o fechamento É a corda
        ..VecPath::default()
    }
}

/// **Fecha as duas pontas de um arco contra uma aresta RETA** — o handle que aponta para
/// fora do arco é do ARCO, e não tem o que fazer numa reta.
///
/// ⚠️ **Isto é correção de GEOMETRIA, não uma preparação para o knob de quina.** O
/// `arc_verts` dá a cada vértice handles simétricos, *como se o arco continuasse*; num arco
/// PARCIAL o primeiro e o último vértice têm um lado que já não é arco — é a borda radial da
/// fatia, ou a corda. Deixar o handle ali faz a aresta **abaular**: medido
/// (`measure_corner_on_arcs::is_the_radial_edge_of_a_slice_actually_straight`), a aresta que
/// fecha uma pizza desviava **0,1865** da reta num raio 1 — quase **19% do raio** —, e a de um
/// anel parcial **0,2545**, numa aresta que a referência desenha reta. Depois: **0,0000**.
///
/// ⚠️ **A primeira medição dessa aresta usou o oráculo ERRADO e dizia `0,0960` sobre uma reta
/// perfeita.** Uma aresta reta é uma cúbica degenerada (`[a, a, b, b]`): ela percorre a mesma
/// reta com outra PARAMETRIZAÇÃO, então comparar o ponto em `t` com `lerp(a, b, t)` mede o
/// relógio e não a forma — a assinatura foi o número sair **igual** para 70° e 180°. O oráculo
/// é a distância PERPENDICULAR.
///
/// ⚠️ **E é o mesmo defeito que fazia a fatia não ter QUINA nenhuma.** Com o handle a sobrar,
/// a tangente atravessa a ponta do arco de forma CONTÍNUA, então
/// [`crate::corner_live::corner_at`] a lê como colinear e devolve `None`: uma pizza reportava
/// **1** quina (o bico do centro) e um anel parcial reportava **zero**, quando têm 3 e 4.
/// Um artista que pedisse raio de quina ali não recebia nada, e nada na tela dizia porquê.
fn cap_arc_ends(verts: &mut [VecVertex]) {
    if let Some(f) = verts.first_mut() {
        f.in_handle = f.anchor;
        f.kind = VertexKind::Corner;
    }
    if let Some(l) = verts.last_mut() {
        l.out_handle = l.anchor;
        l.kind = VertexKind::Corner;
    }
}

/// Arco **aberto** (um traço, sem interior) de `sweep` graus a partir de `start`.
#[must_use]
pub fn arc_open(center: [f64; 2], rx: f64, ry: f64, start_deg: f64, sweep_deg: f64) -> VecPath {
    VecPath {
        verts: arc_verts(center, rx, ry, start_deg, full_if_unset(sweep_deg), false),
        closed: false,
        ..VecPath::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in p.verts_all() {
            lo[0] = lo[0].min(v.anchor[0]);
            lo[1] = lo[1].min(v.anchor[1]);
            hi[0] = hi[0].max(v.anchor[0]);
            hi[1] = hi[1].max(v.anchor[1]);
        }
        (lo, hi)
    }

    /// Os quatro cantos do espaço de parâmetros dão as quatro formas — é a tabela do
    /// módulo, executável. Se um deles regredir, a "família num construtor" quebrou.
    #[test]
    fn the_parameter_space_yields_ellipse_donut_pie_and_partial_ring() {
        let c = [0.0, 0.0];
        // Elipse: um contorno, sem furo.
        let e = ellipse_sweep(c, 2.0, 1.0, 0.0, 360.0, 0.0);
        assert!(e.closed && !e.is_compound(), "elipse = 1 contorno fechado");

        // Rosquinha: DOIS contornos + furo por EvenOdd.
        let d = ellipse_sweep(c, 2.0, 1.0, 0.0, 360.0, 0.5);
        assert!(d.is_compound(), "rosquinha tem contorno interno");
        assert_eq!(d.fill_rule, FillRule::EvenOdd, "sem EvenOdd o furo some");
        // O contorno interno cabe DENTRO do externo (é o furo, não outra forma).
        let inner = &d.subpaths[0];
        for v in &inner.verts {
            assert!(
                v.anchor[0].abs() <= 1.0 + 1e-9 && v.anchor[1].abs() <= 0.5 + 1e-9,
                "furo em {:?} estourou o raio interno",
                v.anchor
            );
        }

        // Pizza: fecha pelo CENTRO (o centro é uma âncora do contorno).
        let p = ellipse_sweep(c, 1.0, 1.0, 0.0, 90.0, 0.0);
        assert!(p.closed && !p.is_compound());
        assert!(
            p.verts.iter().any(|v| v.anchor == c),
            "a pizza fecha pelo centro"
        );

        // Anel parcial: NÃO passa pelo centro (fecha pelo arco interno).
        let r = ellipse_sweep(c, 1.0, 1.0, 0.0, 90.0, 0.5);
        assert!(
            !r.verts.iter().any(|v| v.anchor == c),
            "o anel parcial nao toca o centro"
        );
        assert!(r.verts.len() > p.verts.len(), "tem os dois arcos");
    }

    /// **Retro-compat:** uma elipse salva ANTES de o parâmetro existir tem `sweep = 0`.
    /// Isso tem de valer uma volta inteira — senão toda elipse já salva sumiria da cena.
    #[test]
    fn an_unset_sweep_means_a_full_turn_not_an_empty_shape() {
        let e = ellipse_sweep([0.0, 0.0], 2.0, 1.0, 0.0, 0.0, 0.0);
        let (lo, hi) = bbox(&e);
        assert!(
            (hi[0] - lo[0] - 4.0).abs() < 1e-6,
            "a elipse tem a largura toda"
        );
        assert!(e.closed && !e.is_compound());
    }

    /// O `start` GIRA a forma: uma pizza de 90° começando em 90° ocupa outro quadrante.
    #[test]
    fn start_rotates_the_wedge() {
        let a = ellipse_sweep([0.0, 0.0], 1.0, 1.0, 0.0, 90.0, 0.0);
        let b = ellipse_sweep([0.0, 0.0], 1.0, 1.0, 90.0, 90.0, 0.0);
        let (lo_a, _) = bbox(&a);
        let (lo_b, _) = bbox(&b);
        assert!(lo_a[0] >= -1e-9, "a 1a fatia fica em x >= 0");
        assert!(lo_b[0] < -0.9, "a 2a fatia atravessou para x < 0");
    }

    /// O segmento fecha pela CORDA: mesmas âncoras do arco aberto, mas fechado — e sem o
    /// centro (é o que o separa da pizza).
    #[test]
    fn the_chord_closes_the_arc_without_passing_through_the_center() {
        let c = [0.0, 0.0];
        let s = ellipse_chord(c, 1.0, 1.0, 0.0, 120.0);
        let a = arc_open(c, 1.0, 1.0, 0.0, 120.0);
        assert!(s.closed && !a.closed);
        assert_eq!(s.verts.len(), a.verts.len(), "mesmas ancoras do arco");
        assert!(
            !s.verts.iter().any(|v| v.anchor == c),
            "nao passa pelo centro"
        );
    }
}

#[cfg(test)]
mod cap_tests {
    use super::*;
    use crate::corner_live::corner_at;

    /// O ponto de uma cúbica em `t` (de Casteljau, escrito aqui para o gate não depender de
    /// um privado de outro módulo).
    fn at(c: [[f64; 2]; 4], t: f64) -> [f64; 2] {
        let u = 1.0 - t;
        let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
        [
            c[0][0] * w[0] + c[1][0] * w[1] + c[2][0] * w[2] + c[3][0] * w[3],
            c[0][1] * w[0] + c[1][1] * w[1] + c[2][1] * w[2] + c[3][1] * w[3],
        ]
    }

    /// A pior distância **PERPENDICULAR** entre uma aresta e a reta que liga as suas pontas,
    /// olhando só as arestas que ligam dois raios DIFERENTES (as radiais / a corda).
    ///
    /// ⚠️ **Perpendicular, e não contra `lerp(a, b, t)`:** uma aresta reta é a cúbica
    /// degenerada `[a, a, b, b]`, que percorre a mesma reta com outra parametrização — o
    /// oráculo ingénuo reporta ~0,096 sobre geometria perfeita.
    fn worst_bow(p: &VecPath) -> f64 {
        let n = p.verts.len();
        let mut worst: f64 = 0.0;
        for i in 0..n {
            let (a, b) = (p.verts[i], p.verts[(i + 1) % n]);
            let ra = a.anchor[0].hypot(a.anchor[1]);
            let rb = b.anchor[0].hypot(b.anchor[1]);
            if (ra - rb).abs() < 1e-6 {
                continue; // pedaço de arco, não a aresta que fecha
            }
            let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
            let d = [b.anchor[0] - a.anchor[0], b.anchor[1] - a.anchor[1]];
            let len = d[0].hypot(d[1]);
            if len < 1e-12 {
                continue;
            }
            for k in 1..20 {
                let q = at(c, f64::from(k) / 20.0);
                let w = [q[0] - a.anchor[0], q[1] - a.anchor[1]];
                worst = worst.max((w[0] * d[1] - w[1] * d[0]).abs() / len);
            }
        }
        worst
    }

    /// **A ARESTA QUE FECHA UMA FATIA É RETA.**
    ///
    /// Medido antes da correcção: a pizza de 70° abaulava **0,1865** num raio 1 (19%), a de
    /// 180° **0,2450**, e o anel parcial **0,2545** — o handle do arco sobrava na ponta e
    /// curvava a borda radial. FALSIFICADO por remover o [`cap_arc_ends`].
    #[test]
    fn the_edge_that_closes_a_slice_is_straight() {
        let c = [0.0, 0.0];
        for (name, path) in [
            ("pizza 70", ellipse_sweep(c, 1.0, 1.0, 0.0, 70.0, 0.0)),
            ("pizza 180", ellipse_sweep(c, 1.0, 1.0, 0.0, 180.0, 0.0)),
            ("anel parcial", ellipse_sweep(c, 1.0, 1.0, 30.0, 220.0, 0.5)),
            ("corda", ellipse_chord(c, 1.0, 1.0, 0.0, 120.0)),
        ] {
            assert!(
                worst_bow(&path) < 1e-9,
                "{name}: a aresta que fecha desviou {:.4} da reta",
                worst_bow(&path)
            );
        }
    }

    /// **E AGORA A FATIA TEM QUINAS** — o motor de quinas vivas as vê, que é o que faz um raio
    /// de canto ter onde pousar.
    ///
    /// ⚠️ **A segunda metade é o CONTROLE, e é ela que impede a cura de ir longe demais:** a
    /// elipse inteira e a rosquinha não podem ganhar quina nenhuma — os vértices delas são de
    /// ARCO, e arredondá-los deformaria a curva.
    #[test]
    fn a_slice_has_corners_and_a_full_ellipse_has_none() {
        let c = [0.0, 0.0];
        let count = |p: &VecPath| {
            (0..p.verts.len())
                .filter(|&i| corner_at(&p.verts, p.closed, i).is_some())
                .count()
        };
        for (name, want, path) in [
            ("pizza", 3, ellipse_sweep(c, 1.0, 1.0, 0.0, 70.0, 0.0)),
            (
                "anel parcial",
                4,
                ellipse_sweep(c, 1.0, 1.0, 30.0, 220.0, 0.5),
            ),
            ("corda", 2, ellipse_chord(c, 1.0, 1.0, 0.0, 120.0)),
            ("elipse", 0, ellipse_sweep(c, 1.0, 1.0, 0.0, 360.0, 0.0)),
            ("rosquinha", 0, ellipse_sweep(c, 1.0, 1.0, 0.0, 360.0, 0.55)),
        ] {
            assert_eq!(count(&path), want, "{name}: quinas que o motor VÊ");
        }
    }
}
