//! Arredondamento de QUINAS de um contorno de arestas retas — módulo irmão de
//! [`crate::shapes`] (que fica com as primitivas).
//!
//! É o mesmo arredondamento que o [`crate::rounded_rect`] faz à mão, generalizado para
//! um contorno qualquer e com **raio por-vértice**: é assim que o polígono ganha canto
//! redondo e a estrela ganha DOIS raios — um para as pontas (quinas convexas) e outro
//! para os vales (côncavas).
//!
//! Construção clássica (CAD): em cada quina, recua-se `t = r / tan(θ/2)` ao longo das
//! duas arestas (θ = ângulo interno) e liga-se os dois pontos por um arco. O vértice
//! único vira **dois** vértices de quina (e **três**, com um do meio, se a quina virar mais
//! de `90°` — ver [`circular_fillet`]), com o handle apontando para o vértice
//! original — a mesma forma do `rounded_rect` (arco de um lado, handle nulo do lado
//! reto), então as quinas seguem independentes e editáveis à mão depois.
//!
//! Robustez: `t` é clampado a **metade** de cada aresta adjacente, então nem o raio
//! pedido nem duas quinas vizinhas conseguem cruzar a aresta que compartilham (o
//! arredondamento satura, nunca inverte a forma). Raio ~0 ou quina degenerada
//! (colinear) devolvem o vértice **cru** — com todos os raios zerados a saída é
//! byte-idêntica ao contorno de entrada.

use crate::{VecPath, VecVertex, VertexKind};

/// Abaixo disto (unidades de mundo) um raio / aresta / recuo é tratado como zero.
const EPS: f64 = 1e-9;

/// O comprimento de handle (fração do raio) que faz uma cúbica seguir um quarto de
/// círculo. Compartilhado por quem monta arco à mão (round-rect, cilindro, "D"…).
pub const KAPPA: f64 = 0.552_284_75;

/// Arredonda as quinas de um contorno FECHADO de arestas retas: `pts[i]` recebe o raio
/// `radii[i]` (extra ignorado; faltante = 0). Devolve o path fechado, sem estilo.
///
/// `radii` menor que `pts` (ou todo zero) ⇒ os vértices saem crus, na ordem — a
/// identidade. Ver o módulo para a construção e os clamps.
#[must_use]
pub fn round_closed_corners(pts: &[[f64; 2]], radii: &[f64]) -> VecPath {
    round_closed_corners_smooth(pts, radii, 0.0)
}

/// Como [`round_closed_corners`], mas com **suavização de quina** (`smoothing ∈ [0, 1]`, o
/// *corner smoothing* do Figma): cada arco de círculo vira `asa + arco curto + asa`, e a
/// curvatura passa a subir em RAMPA a partir do lado reto em vez de saltar. A construção
/// (e por que a superelipse não serve) está em [`crate::smooth`].
///
/// **`smoothing <= 0` é a IDENTIDADE**: cai na quina de arco puro, byte-a-byte a de sempre —
/// é o que mantém polígono, estrela e round-rect sem suavização exatamente como estavam.
/// Uma quina de raio ~0 fica CRUA mesmo com suavização: não há arco para suavizar.
#[must_use]
pub fn round_closed_corners_smooth(pts: &[[f64; 2]], radii: &[f64], smoothing: f64) -> VecPath {
    let n = pts.len();
    let mut verts: Vec<VecVertex> = Vec::with_capacity(n * 2);
    for i in 0..n {
        let v = pts[i];
        let a = pts[(i + n - 1) % n];
        let b = pts[(i + 1) % n];
        let r = radii.get(i).copied().unwrap_or(0.0).max(0.0);
        // A quina SUAVE só existe com `smoothing > 0` E raio > 0 — `smooth_corner` devolve
        // `None` nos dois casos, e aí o caminho é o arco puro de sempre: a saída fica
        // byte-idêntica à de antes desta feature. **A identidade é sagrada.**
        if let Some(smooth) = crate::smooth::smooth_corner(a, v, b, r, smoothing) {
            verts.extend(smooth);
            continue;
        }
        match rounded_corner(a, v, b, r) {
            Some(quina) => verts.extend(quina),
            None => verts.push(VecVertex::corner(v)),
        }
    }
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// Os vértices que substituem a quina `v` (entre `a` e `b`) arredondada com raio `r`: o que
/// entra no arco e o que sai — e, numa quina que vira mais de `90°`, o do MEIO do arco
/// ([`circular_fillet`]). `None` quando não há o que arredondar (raio ~0, aresta degenerada,
/// ou quina colinear) — aí o chamador mantém o vértice cru.
fn rounded_corner(a: [f64; 2], v: [f64; 2], b: [f64; 2], r: f64) -> Option<Vec<VecVertex>> {
    if r <= EPS {
        return None;
    }
    // Versores das duas arestas, SAINDO da quina.
    let (ua, len_a) = unit(sub(a, v))?;
    let (ub, len_b) = unit(sub(b, v))?;
    // Ângulo interno da quina (o que as duas arestas abrem entre si), em [0, π].
    let theta = dot(ua, ub).clamp(-1.0, 1.0).acos();
    let half = theta * 0.5;
    if half <= EPS || (std::f64::consts::PI - theta) <= EPS {
        return None; // colinear (nada a arredondar) ou aresta dobrada sobre si
    }
    // Recuo pedido pelo raio, clampado a meia-aresta de cada lado: assim duas quinas
    // vizinhas nunca invadem uma a outra (o arredondamento SATURA, não inverte).
    let t = (r / half.tan()).min(len_a * 0.5).min(len_b * 0.5);
    if t <= EPS {
        return None;
    }
    // Raio efetivo depois do clamp (é ele que dita o arco, não o `r` pedido).
    let r_eff = t * half.tan();
    // Arco que a quina descreve = suplemento do ângulo interno.
    let alpha = std::f64::consts::PI - theta;
    let p_in = [v[0] + ua[0] * t, v[1] + ua[1] * t];
    let p_out = [v[0] + ub[0] * t, v[1] + ub[1] * t];
    // Handles apontam para a quina ORIGINAL (é o polo do arco); o outro lado fica nulo
    // (aresta reta) — mesma forma do `rounded_rect`, quinas independentes.
    let f = circular_fillet(p_in, [-ua[0], -ua[1]], p_out, ub, alpha, r_eff);
    let mut out = vec![VecVertex {
        anchor: p_in,
        in_handle: p_in,
        out_handle: f.out1,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }];
    out.extend(f.mid);
    out.push(VecVertex {
        anchor: p_out,
        in_handle: f.in2,
        out_handle: p_out,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    });
    Some(out)
}

/// Os alçapões de um filete CIRCULAR — e o vértice do meio quando ele varre mais de `90°`.
pub(crate) struct Fillet {
    /// O alçapão de SAÍDA do ponto onde o arco começa.
    pub out1: [f64; 2],
    /// O vértice do meio do arco, só quando ele varre mais de `90°`.
    pub mid: Option<VecVertex>,
    /// O alçapão de ENTRADA do ponto onde o arco acaba.
    pub in2: [f64; 2],
}

/// ⭐⭐ **UM ARCO DE CÍRCULO EM CÚBICAS DE ATÉ `90°` — a lei da casa, aplicada à quina** (2026-09-16).
///
/// O [`crate::shapes::arc`] já a escrevia (*«divide o arco em segmentos de ≤90° para o bézier
/// aproximar bem»*), e o `ellipse` e o `rounded_rect` cumprem-na por construção. **Os dois
/// arredondadores de quina eram os únicos escritores de arco da casa que a violavam**: uma quina que
/// vira mais de `90°` (a ponta de uma estrela, o lábio de um vaso, todo triângulo) saía numa cúbica
/// só. O erro radial intrínseco de uma cúbica cresce com a sexta potência da abertura:
///
/// | abertura | erro máximo / `r` |
/// |---:|---:|
/// |  60° | `2,39e-5` |
/// |  90° | `2,73e-4` |
/// | 127° (a quina de fora do lábio do vaso da cena 5) | `2,16e-3` |
/// | 150° | `5,97e-3` |
/// | 170° | `1,29e-2` |
///
/// ⇒ o artista pedia o raio `r` numa ponta e a curva desenhada afastava-se `1 %` dele; e o modelador
/// 3D, que reconhece arcos pela precisão de um quarto de círculo, não reconhecia esta — a quina ficava
/// partida em segmentos, com as faixas de luz do Bug #1 dos `docs/3DModeling/BUGS_3dmodeling.md`
/// sempre que o botão `Resolution` subia.
///
/// Partida em duas, cada metade varre `α/2 < 90°` e erra no máximo o que um quarto erra. ⚠️ **Até
/// `90°` a saída é BYTE-IDÊNTICA à de antes** (o mesmo alçapão `(4/3)·tan(α/4)·r`, a mesma conta), e
/// é isso que deixa o quadrado, o `rounded_rect` e todo polígono de cinco ou mais lados como estavam.
///
/// `t_in` é a direcção de MARCHA a chegar a `p_in` (aponta para a quina), `t_out` a de marcha a sair de
/// `p_out`, `alpha` o ângulo que a marcha vira, e `r` o raio. O vértice do meio fica na bissectriz, a
/// `r` do centro, com a tangente paralela à corda.
pub(crate) fn circular_fillet(
    p_in: [f64; 2],
    t_in: [f64; 2],
    p_out: [f64; 2],
    t_out: [f64; 2],
    alpha: f64,
    r: f64,
) -> Fillet {
    let quarter = std::f64::consts::FRAC_PI_2;
    let handles = |sweep: f64| (4.0 / 3.0) * (sweep * 0.25).tan() * r;
    if alpha <= quarter * (1.0 + 1e-12) {
        let h = handles(alpha);
        return Fillet {
            out1: [p_in[0] + t_in[0] * h, p_in[1] + t_in[1] * h],
            mid: None,
            in2: [p_out[0] - t_out[0] * h, p_out[1] - t_out[1] * h],
        };
    }
    let h = handles(alpha * 0.5);
    let chord = sub(p_out, p_in);
    let (e, _) = unit(chord).unwrap_or(([t_in[0], t_in[1]], 0.0));
    // A normal da corda que aponta para a QUINA — é para lá que o arco se afasta da corda.
    let mut w = [-e[1], e[0]];
    if dot(w, t_in) < 0.0 {
        w = [-w[0], -w[1]];
    }
    let sagitta = r * (1.0 - (alpha * 0.5).cos());
    let m = [
        (p_in[0] + p_out[0]) * 0.5 + w[0] * sagitta,
        (p_in[1] + p_out[1]) * 0.5 + w[1] * sagitta,
    ];
    Fillet {
        out1: [p_in[0] + t_in[0] * h, p_in[1] + t_in[1] * h],
        mid: Some(VecVertex {
            anchor: m,
            in_handle: [m[0] - e[0] * h, m[1] - e[1] * h],
            out_handle: [m[0] + e[0] * h, m[1] + e[1] * h],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        }),
        in2: [p_out[0] - t_out[0] * h, p_out[1] - t_out[1] * h],
    }
}

fn sub(p: [f64; 2], q: [f64; 2]) -> [f64; 2] {
    [p[0] - q[0], p[1] - q[1]]
}

fn dot(p: [f64; 2], q: [f64; 2]) -> f64 {
    p[0] * q[0] + p[1] * q[1]
}

/// Versor + comprimento; `None` se o vetor é degenerado.
fn unit(v: [f64; 2]) -> Option<([f64; 2], f64)> {
    let len = v[0].hypot(v[1]);
    (len > EPS).then(|| ([v[0] / len, v[1] / len], len))
}

#[cfg(test)]
#[path = "corner_split_tests.rs"]
mod corner_split_tests;

#[cfg(test)]
mod tests {
    use super::*;

    /// Raio zero = IDENTIDADE: o contorno sai exatamente como entrou (um vértice de
    /// quina por ponto). É o que mantém `regular_polygon`/`star` sem raio byte-idênticos.
    #[test]
    fn zero_radius_is_the_identity() {
        let sq = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let p = round_closed_corners(&sq, &[0.0; 4]);
        assert_eq!(p.verts.len(), 4, "nenhuma quina foi dividida");
        for (v, want) in p.verts.iter().zip(sq) {
            assert_eq!(v.anchor, want);
            assert_eq!(v.in_handle, want, "handle nulo (aresta reta)");
            assert_eq!(v.out_handle, want);
        }
    }

    /// Uma quina arredondada vira DOIS vértices, recuados `r` ao longo de cada aresta
    /// (num canto reto, `t = r`), com o handle no comprimento canônico do quarto de
    /// círculo — é o mesmo resultado que o `rounded_rect` produz à mão.
    #[test]
    fn a_right_angle_corner_matches_the_canonical_quarter_circle() {
        const KAPPA: f64 = 0.552_284_75;
        let sq = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let p = round_closed_corners(&sq, &[2.0; 4]);
        assert_eq!(p.verts.len(), 8, "4 quinas × 2 vértices");
        // A 1ª quina (0,0): entra vindo de (0,10) → recua em +Y; sai para (10,0) → +X.
        let v_in = &p.verts[0];
        let v_out = &p.verts[1];
        assert!((v_in.anchor[0] - 0.0).abs() < 1e-9 && (v_in.anchor[1] - 2.0).abs() < 1e-9);
        assert!((v_out.anchor[0] - 2.0).abs() < 1e-9 && (v_out.anchor[1] - 0.0).abs() < 1e-9);
        // Handle: aponta para a quina original, a KAPPA·r dela.
        let want_h = 2.0 * KAPPA;
        assert!(
            (v_in.out_handle[1] - (2.0 - want_h)).abs() < 1e-6,
            "handle do arco"
        );
        assert!((v_out.in_handle[0] - (2.0 - want_h)).abs() < 1e-6);
        assert_eq!(v_in.in_handle, v_in.anchor, "o lado reto tem handle nulo");
        assert_eq!(v_out.out_handle, v_out.anchor);
    }

    /// O raio SATURA em meia-aresta: pedir um raio absurdo não faz a forma inverter —
    /// os recuos param no meio de cada aresta (e as quinas vizinhas se encostam, no
    /// máximo). Sem esse clamp, uma estrela fina com raio grande vira um nó.
    #[test]
    fn an_absurd_radius_saturates_at_half_an_edge_instead_of_inverting() {
        let sq = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let p = round_closed_corners(&sq, &[1e6; 4]);
        assert_eq!(p.verts.len(), 8);
        // Todo vértice segue DENTRO do quadrado original (nada estourou a caixa).
        for v in &p.verts {
            assert!(
                (-1e-9..=10.0 + 1e-9).contains(&v.anchor[0])
                    && (-1e-9..=10.0 + 1e-9).contains(&v.anchor[1]),
                "âncora {:?} saiu da caixa — o raio inverteu a forma",
                v.anchor
            );
        }
        // O recuo saturou exatamente na metade da aresta (5.0), não além.
        assert!((p.verts[0].anchor[1] - 5.0).abs() < 1e-9);
    }

    /// A quina CÔNCAVA (o vale de uma estrela) arredonda igual à convexa — a
    /// construção é a mesma, o arco só curva para o outro lado. É o que permite
    /// arredondar as pontas e os vales com raios independentes.
    #[test]
    fn a_reflex_corner_rounds_too() {
        // "Seta": o vértice (5,5) é um vale côncavo entre duas pontas.
        let arrow = [
            [0.0, 0.0],
            [5.0, 5.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
        ];
        let sharp = round_closed_corners(&arrow, &[0.0; 5]);
        let mut radii = [0.0; 5];
        radii[1] = 1.0; // só o vale
        let rounded = round_closed_corners(&arrow, &radii);
        assert_eq!(sharp.verts.len(), 5);
        assert_eq!(rounded.verts.len(), 6, "só o vale virou dois vértices");
        // Os dois vértices do vale recuaram DE (5,5) em direção às pontas vizinhas.
        let (a, b) = (&rounded.verts[1], &rounded.verts[2]);
        assert!(a.anchor[1] < 5.0 && b.anchor[1] < 5.0, "recuou para dentro");
        assert!(a.anchor != b.anchor, "duas âncoras distintas");
    }
}
