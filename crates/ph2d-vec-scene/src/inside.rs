//! **O ponto está dentro da forma?** — módulo irmão de [`crate::path_ops`] (teto de LOC).
//!
//! Amostra cada contorno FECHADO num polígono e aplica a [`FillRule`] do path (even-odd =
//! paridade de cruzamentos; non-zero = winding number). O buraco de um compound é um contorno a
//! mais, então ele sai `false` de graça. Transcendental-free.
//!
//! ⚠️ **O teste de cruzamento vive UMA vez** ([`crossing_counts`]) e tem **dois** consumidores: o
//! [`contains_point`] de uma forma (que soma sobre os contornos e só então aplica a regra de
//! preenchimento) e o [`point_in_polygon`] do LAÇO (um polígono, paridade). A parte sutil é a
//! regra semi-aberta `(a.y > p.y) != (b.y > p.y)`, que é o que impede um vértice exatamente na
//! altura do raio de contar duas vezes — uma segunda cópia dela é a forma clássica de o ponto
//! sobre um vértice cair para fora.

use crate::{FillRule, VecPath};

/// Amostras por segmento de cúbica ao achatar o contorno.
const CURVE_SAMPLES: usize = 16;

/// **Quantas vezes o raio para −x a partir de `p` cruza este polígono FECHADO**, e com que
/// enrolamento — `(crossings, winding)`.
///
/// O polígono é a lista de vértices; a aresta de fecho (último → primeiro) entra por construção
/// (o laço começa com `j = n - 1`). Menos de 3 pontos não delimitam área: devolve `(0, 0)`.
///
/// Transcendental-free, sem divisão por zero (o guard de altura garante `a[1] != b[1]`).
#[must_use]
fn crossing_counts(poly: &[[f64; 2]], p: [f64; 2]) -> (i32, i32) {
    let n = poly.len();
    if n < 3 {
        return (0, 0);
    }
    let (mut crossings, mut winding) = (0i32, 0i32);
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poly[j], poly[i]);
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let t = (p[1] - a[1]) / (b[1] - a[1]);
            let x = a[0] + t * (b[0] - a[0]);
            if p[0] < x {
                crossings += 1;
                winding += if b[1] > a[1] { 1 } else { -1 };
            }
        }
        j = i;
    }
    (crossings, winding)
}

/// **O ponto está dentro deste polígono?** — paridade de cruzamentos (even-odd).
///
/// É o predicado do **LAÇO de seleção**: o caminho que o artista desenhou é uma polilinha bruta
/// (sem regra de preenchimento autorada), e even-odd é o que "dentro do laço" significa — um laço
/// que se cruza deixa de fora o miolo que ele contornou duas vezes, que é o mesmo que o
/// Illustrator e o Blender fazem.
#[must_use]
pub fn point_in_polygon(poly: &[[f64; 2]], p: [f64; 2]) -> bool {
    crossing_counts(poly, p).0 % 2 != 0
}

/// Um ponto da cúbica em `t` (cópia local — o de `path_ops` é privado, e um módulo irmão não
/// justifica alargar a superfície dele).
fn cubic(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// O mesmo teste, para um [`VecPath`] **que não está na cena** — a ponta de seta, que é
/// construída sob demanda (`stroke_head`) e nunca vive no documento.
///
/// É o par natural do método acima: um hit-test que só soubesse perguntar por `id` não
/// conseguiria perguntar pela cabeça da seta — e a cabeça é justamente a parte gorda, a que o
/// olho mira e o mouse persegue.
#[must_use]
pub fn contains_point(path: &VecPath, p: [f64; 2]) -> bool {
    // A geometria COZIDA (quinas já arredondadas) — é a forma que está na tela, e é ela
    // que o dedo tem de pegar. Sem raio nenhum isto é a própria fonte, emprestada.
    let path = &*path.cooked();
    {
        let even_odd = path.fill_rule == FillRule::EvenOdd;
        let mut crossings = 0i32;
        let mut winding = 0i32;
        let mut any = false;
        for k in 0..path.contour_count() {
            let Some((verts, closed)) = path.contour(k) else {
                continue;
            };
            if !closed || verts.len() < 2 {
                continue;
            }
            any = true;
            let mut poly: Vec<[f64; 2]> = Vec::with_capacity(verts.len() * CURVE_SAMPLES);
            for i in 0..verts.len() {
                let a = &verts[i];
                let b = &verts[(i + 1) % verts.len()];
                for j in 0..CURVE_SAMPLES {
                    let t = j as f64 / CURVE_SAMPLES as f64;
                    poly.push(cubic(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
                }
            }
            let (c, w) = crossing_counts(&poly, p);
            crossings += c;
            winding += w;
        }
        if !any {
            return false;
        }
        if even_odd {
            crossings % 2 != 0
        } else {
            winding != 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{crossing_counts, point_in_polygon};

    /// O quadrado unitário, anti-horário.
    fn unit() -> Vec<[f64; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    /// **Dentro é dentro, fora é fora** — o caso trivial, que é o controle dos outros.
    #[test]
    fn the_inside_is_inside_and_the_outside_is_out() {
        assert!(point_in_polygon(&unit(), [0.5, 0.5]));
        for p in [[-0.5, 0.5], [1.5, 0.5], [0.5, -0.5], [0.5, 1.5]] {
            assert!(!point_in_polygon(&unit(), p), "{p:?} devia estar fora");
        }
    }

    /// **Um vértice na altura EXATA do raio conta UMA vez.**
    ///
    /// É a parte sutil do teste de cruzamento, e a razão de ele viver numa função só: a regra
    /// semi-aberta `(a.y > p.y) != (b.y > p.y)` faz cada vértice pertencer a exatamente uma das
    /// duas arestas que o tocam. Uma segunda cópia escrita com `>=` conta as duas, a paridade
    /// inverte, e o ponto ao lado de um vértice cai para FORA — o modo de falha clássico.
    ///
    /// Aqui o raio passa exatamente por `y = 0` e por `y = 1`, as duas alturas onde o quadrado tem
    /// vértices.
    #[test]
    fn a_vertex_at_the_ray_height_counts_once() {
        // `y = 0.0` é a altura de dois vértices; um ponto à direita deles está FORA.
        assert!(!point_in_polygon(&unit(), [2.0, 0.0]));
        assert!(!point_in_polygon(&unit(), [2.0, 1.0]));
        // E um losango, onde a altura do vértice é o MEIO da forma: à direita, fora; dentro,
        // dentro. Com `>=` no lugar do `>` este par inverte.
        let diamond = vec![[1.0, 0.0], [2.0, 1.0], [1.0, 2.0], [0.0, 1.0]];
        assert!(
            !point_in_polygon(&diamond, [3.0, 1.0]),
            "a' direita do losango"
        );
        assert!(
            point_in_polygon(&diamond, [1.0, 1.0]),
            "no centro do losango"
        );
    }

    /// **Menos de três pontos não delimitam área** — e a resposta é `(0, 0)`, não um pânico.
    #[test]
    fn a_degenerate_polygon_has_no_inside() {
        for poly in [vec![], vec![[0.0, 0.0]], vec![[0.0, 0.0], [1.0, 1.0]]] {
            assert_eq!(crossing_counts(&poly, [0.5, 0.5]), (0, 0));
            assert!(!point_in_polygon(&poly, [0.5, 0.5]));
        }
    }

    /// **O enrolamento tem SINAL** — é o que separa o even-odd do non-zero, e é a metade que o
    /// `contains_point` de um compound consome (um buraco desenhado ao contrário cancela).
    #[test]
    fn the_winding_carries_the_direction() {
        let ccw = unit();
        let cw: Vec<[f64; 2]> = unit().into_iter().rev().collect();
        let (c1, w1) = crossing_counts(&ccw, [0.5, 0.5]);
        let (c2, w2) = crossing_counts(&cw, [0.5, 0.5]);
        assert_eq!((c1, c2), (1, 1), "a paridade nao conhece o sentido");
        assert_eq!(
            w1, -w2,
            "o enrolamento conhece — e e' nisso que o non-zero se apoia"
        );
        assert_ne!(w1, 0);
    }
}
