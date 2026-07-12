//! Do bitmap preenchido para a **geometria**: traçado de contorno (Moore) +
//! vetorização (RDP + suavização binomial).
//!
//! O resultado do balde é geometria, não pixels — é o que deixa o preenchimento ser
//! selecionado, movido, animado e tweenado como qualquer traço (`02 §6`). Os buracos
//! saem de graça: uma região com furo produz **vários** contornos fechados, e o
//! externo é o de maior área.
//!
//! **Divergência declarada do GP:** o pós-processo dele é "smooth 20 iterações +
//! decimação `2^simplify`", que deixa polilinhas densas e moles. A `04 §3` mandava
//! trocar por "RDP + fit Schneider". Ficamos com **RDP + binomial leve** e SEM o fit
//! de Bézier — porque o `FlipStroke` é uma **polilinha** (não tem handles): uma curva
//! de Bézier seria re-achatada em pontos no instante seguinte, e o único efeito seria
//! perder precisão no caminho de ida e volta. Se um dia o traço ganhar handles, o fit
//! entra aqui, num lugar só.

use crate::raster::{FILLED, Grid};
use ph2d_core::Vec2;

/// Tolerância do RDP, em pixels da grade (o `ε ≈ 1.25px` da `04 §3`): abaixo disso o
/// desvio é sub-pixel e ninguém vê.
pub const RDP_EPSILON_PX: f32 = 1.25;

/// **Traça todos os contornos** da região `FILLED` — o externo e os buracos.
///
/// Marching squares por células de 2×2: um contorno passa entre um pixel preenchido e
/// um vazio. Cada contorno sai como um anel fechado de pontos em coordenadas do
/// DOCUMENTO, em ordem consistente. Os buracos aparecem naturalmente como contornos
/// separados — não há caso especial nenhum.
#[must_use]
pub fn trace_contours(g: &Grid) -> Vec<Vec<Vec2>> {
    let filled = |x: i32, y: i32| -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < g.w
            && (y as usize) < g.h
            && g.at(x as usize, y as usize) & FILLED != 0
    };
    // Uma ARESTA de contorno vai entre dois cantos de pixel. Andamos pelos cantos:
    // o canto (x, y) fica no canto inferior-esquerdo do pixel (x, y).
    // Para cada canto, o "código" das 4 células ao redor decide para onde o contorno
    // segue (marching squares).
    let code = |x: i32, y: i32| -> u8 {
        (u8::from(filled(x - 1, y - 1)))
            | (u8::from(filled(x, y - 1)) << 1)
            | (u8::from(filled(x - 1, y)) << 2)
            | (u8::from(filled(x, y)) << 3)
    };

    let cw = g.w + 1; // cantos por linha
    let mut seen = vec![false; cw * (g.h + 1)];
    let mut out: Vec<Vec<Vec2>> = Vec::new();

    for sy in 0..=g.h as i32 {
        for sx in 0..=g.w as i32 {
            let c = code(sx, sy);
            // Só começa num canto que é fronteira (nem todo-vazio nem todo-cheio) e
            // que ainda não foi visitado.
            if c == 0 || c == 15 || seen[sy as usize * cw + sx as usize] {
                continue;
            }
            let mut ring: Vec<Vec2> = Vec::new();
            let (mut x, mut y) = (sx, sy);
            // Teto de segurança: um contorno não pode ser maior que o perímetro da
            // grade (nenhum canto é visitado duas vezes na mesma direção).
            let mut guard = 4 * (g.w + g.h) + 16;
            loop {
                seen[y as usize * cw + x as usize] = true;
                ring.push(corner_to_doc(g, x, y));
                // A regra: **ande com o preenchido à ESQUERDA**. Daí a tabela sai
                // sozinha, sem decorar — para cada direção, a célula à esquerda tem de
                // estar cheia e a da direita, vazia:
                //   leste  (+x): esquerda = NE, direita = SE
                //   oeste  (−x): esquerda = SW, direita = NW
                //   norte  (+y): esquerda = NW, direita = NE
                //   sul    (−y): esquerda = SE, direita = SW
                // (Os bits: 1=SW, 2=SE, 4=NW, 8=NE.) Os códigos 6 e 9 são as
                // diagonais AMBÍGUAS — duas escolhas válidas; fixar uma delas é o que
                // separa duas regiões que se tocam na quina em vez de fundi-las.
                let (dx, dy) = match code(x, y) {
                    1 | 3 | 11 => (-1, 0),  // oeste
                    2 | 10 | 14 => (0, -1), // sul
                    4 | 5 | 7 => (0, 1),    // norte
                    8 | 12 | 13 => (1, 0),  // leste
                    6 => (0, -1),           // ambíguo: sul (escolha fixa)
                    9 => (1, 0),            // ambíguo: leste (escolha fixa)
                    _ => break,
                };
                x += dx;
                y += dy;
                guard -= 1;
                if guard == 0 || (x == sx && y == sy) {
                    break;
                }
                if x < 0 || y < 0 || x > g.w as i32 || y > g.h as i32 {
                    break;
                }
            }
            if ring.len() >= 3 {
                out.push(ring);
            }
        }
    }
    out
}

/// O canto `(x, y)` da grade em coordenadas do documento. Um canto fica na FRONTEIRA
/// entre pixels — daí o `to_doc(x,y) - meio pixel` (o `to_doc` devolve o CENTRO).
fn corner_to_doc(g: &Grid, x: i32, y: i32) -> Vec2 {
    Vec2::new(
        g.origin.x + x as f32 / g.scale,
        g.origin.y + y as f32 / g.scale,
    )
}

/// **A área (com sinal) de um anel** — o critério que separa o contorno EXTERNO (o de
/// maior |área|) dos buracos.
#[must_use]
pub fn signed_area(ring: &[Vec2]) -> f32 {
    let n = ring.len();
    let mut a = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        a += ring[i].x * ring[j].y - ring[j].x * ring[i].y;
    }
    a * 0.5
}

/// **Simplifica** um anel: Ramer-Douglas-Peucker com tolerância `eps` (unidades do
/// documento) + uma suavização binomial leve, que tira o serrilhado de um pixel que o
/// marching squares deixa nas diagonais.
///
/// **A ordem importa, e errá-la destrói a forma:** o alisamento vem ANTES do RDP.
/// Alisar DEPOIS opera sobre um anel já reduzido às quinas — e a média binomial de uma
/// quina de quadrado com os seus dois vizinhos (que agora estão a 20 unidades) puxa o
/// canto para o meio: um quadrado de área 400 vira um losango de área 26. Com o anel
/// DENSO (um ponto por pixel), a mesma média move cada ponto por uma fração de pixel —
/// que é exatamente o serrilhado que se quer tirar. (É a mesma lição do pré-filtro do
/// record: filtre o denso, ajuste o filtrado.)
#[must_use]
pub fn simplify_ring(ring: &[Vec2], eps: f32, smooth_passes: usize) -> Vec<Vec2> {
    if ring.len() < 4 {
        return ring.to_vec();
    }
    let ring = &smooth_closed(ring, smooth_passes);
    // O anel é FECHADO: o RDP clássico precisa de dois extremos fixos. Fixamos o ponto
    // mais distante do centroide (uma quina real, estável) e o oposto a ele — assim a
    // simplificação não depende de onde o traçado começou.
    let n = ring.len();
    let c = centroid(ring);
    let far = (0..n)
        .max_by(|&i, &j| dist2(ring[i], c).total_cmp(&dist2(ring[j], c)))
        .unwrap_or(0);
    let opp = (far + n / 2) % n;
    let (lo, hi) = (far.min(opp), far.max(opp));

    let mut keep = vec![false; n];
    keep[lo] = true;
    keep[hi] = true;
    rdp(ring, lo, hi, eps, &mut keep);
    // A 2ª metade do anel (de `hi` de volta a `lo`, dando a volta): reindexa.
    let wrapped: Vec<Vec2> = (hi..n).chain(0..=lo).map(|i| ring[i]).collect();
    let mut keep2 = vec![false; wrapped.len()];
    keep2[0] = true;
    let last = wrapped.len() - 1;
    keep2[last] = true;
    rdp(&wrapped, 0, last, eps, &mut keep2);
    for (k, &kept) in keep2.iter().enumerate() {
        if kept {
            keep[(hi + k) % n] = true;
        }
    }

    (0..n).filter(|&i| keep[i]).map(|i| ring[i]).collect()
}

fn centroid(p: &[Vec2]) -> Vec2 {
    let n = p.len().max(1) as f32;
    let s = p.iter().fold(Vec2::new(0.0, 0.0), |a, b| a + *b);
    Vec2::new(s.x / n, s.y / n)
}

fn dist2(a: Vec2, b: Vec2) -> f32 {
    let d = b - a;
    d.x * d.x + d.y * d.y
}

/// RDP recursivo entre `lo` e `hi` (inclusive), marcando os pontos a manter.
fn rdp(p: &[Vec2], lo: usize, hi: usize, eps: f32, keep: &mut [bool]) {
    if hi <= lo + 1 {
        return;
    }
    let (a, b) = (p[lo], p[hi]);
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    let mut worst = (0.0f32, lo);
    for (i, item) in p.iter().enumerate().take(hi).skip(lo + 1) {
        let ap = *item - a;
        let d = if len2 < 1e-12 {
            (ap.x * ap.x + ap.y * ap.y).sqrt()
        } else {
            (ap.x * ab.y - ap.y * ab.x).abs() / len2.sqrt()
        };
        if d > worst.0 {
            worst = (d, i);
        }
    }
    if worst.0 > eps {
        keep[worst.1] = true;
        rdp(p, lo, worst.1, eps, keep);
        rdp(p, worst.1, hi, eps, keep);
    }
}

/// Suavização binomial `[1,2,1]` num anel FECHADO (os vizinhos dão a volta). Poucos
/// passes: o objetivo é tirar o degrau de um pixel, não derreter as quinas.
fn smooth_closed(p: &[Vec2], passes: usize) -> Vec<Vec2> {
    let n = p.len();
    if n < 4 || passes == 0 {
        return p.to_vec();
    }
    let mut cur = p.to_vec();
    for _ in 0..passes {
        let src = cur.clone();
        for i in 0..n {
            let a = src[(i + n - 1) % n];
            let b = src[i];
            let c = src[(i + 1) % n];
            cur[i] = Vec2::new(
                (a.x + 2.0 * b.x + c.x) * 0.25,
                (a.y + 2.0 * b.y + c.y) * 0.25,
            );
        }
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Preenche uma caixa e traça: sai UM contorno, com a área da região.
    #[test]
    fn a_filled_box_traces_one_ring() {
        let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0), 1.0, 0, 64);
        for (p, q) in [
            (Vec2::new(5.0, 5.0), Vec2::new(15.0, 5.0)),
            (Vec2::new(15.0, 5.0), Vec2::new(15.0, 15.0)),
            (Vec2::new(15.0, 15.0), Vec2::new(5.0, 15.0)),
            (Vec2::new(5.0, 15.0), Vec2::new(5.0, 5.0)),
        ] {
            g.stroke_capsule(p, q, 0.0);
        }
        let seed = g.pixel_of(Vec2::new(10.0, 10.0)).unwrap();
        assert!(g.flood(seed, 0));
        let rings = trace_contours(&g);
        assert_eq!(rings.len(), 1, "uma região = um contorno");
        let a = signed_area(&rings[0]).abs();
        assert!((50.0..=120.0).contains(&a), "área do miolo: {a}");
    }

    /// **Os buracos saem de graça:** uma caixa com uma ilha dentro produz DOIS
    /// contornos — o externo (maior) e o do furo.
    #[test]
    fn an_island_produces_a_second_ring_the_hole() {
        let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(30.0, 30.0), 1.0, 0, 64);
        let bx = |g: &mut Grid, a: f32, b: f32| {
            for (p, q) in [
                (Vec2::new(a, a), Vec2::new(b, a)),
                (Vec2::new(b, a), Vec2::new(b, b)),
                (Vec2::new(b, b), Vec2::new(a, b)),
                (Vec2::new(a, b), Vec2::new(a, a)),
            ] {
                g.stroke_capsule(p, q, 0.0);
            }
        };
        bx(&mut g, 5.0, 25.0); // externo
        bx(&mut g, 12.0, 18.0); // a ilha

        let seed = g.pixel_of(Vec2::new(8.0, 8.0)).unwrap(); // entre os dois
        assert!(g.flood(seed, 0));
        let rings = trace_contours(&g);
        assert!(rings.len() >= 2, "externo + furo, veio {}", rings.len());
        let mut areas: Vec<f32> = rings.iter().map(|r| signed_area(r).abs()).collect();
        areas.sort_by(f32::total_cmp);
        let (hole, outer) = (areas[areas.len() - 2], areas[areas.len() - 1]);
        assert!(
            outer > hole * 2.0,
            "o externo é bem maior: {outer} vs {hole}"
        );
    }

    /// **O bug que a ordem errada causa** (e que custou meia hora): alisar DEPOIS do
    /// RDP opera sobre as QUINAS — e a média binomial de um canto de quadrado com os
    /// vizinhos a 20 unidades puxa o canto para o meio. O quadrado de área 400 virava
    /// um losango de área 26. Alisando o anel DENSO primeiro, cada ponto anda uma
    /// fração de pixel (que é o serrilhado que se quer tirar) e a forma sobrevive.
    #[test]
    fn smoothing_a_dense_ring_preserves_the_shape() {
        // Um quadrado DENSO (um ponto por unidade), como o marching squares produz.
        let mut ring = Vec::new();
        for i in 0..20 {
            ring.push(Vec2::new(i as f32, 0.0));
        }
        for i in 0..20 {
            ring.push(Vec2::new(20.0, i as f32));
        }
        for i in 0..20 {
            ring.push(Vec2::new(20.0 - i as f32, 20.0));
        }
        for i in 0..20 {
            ring.push(Vec2::new(0.0, 20.0 - i as f32));
        }
        let out = simplify_ring(&ring, 1.25, 2);
        let a = signed_area(&out).abs();
        assert!(
            (a - 400.0).abs() < 40.0,
            "a área tem de sobreviver ao alisamento (400±10%), veio {a}"
        );
        assert!(
            out.len() <= 16,
            "e ainda assim simplifica: {} pontos",
            out.len()
        );
    }

    /// O RDP corta os pontos colineares (um retângulo traçado pixel a pixel vira
    /// meia dúzia de pontos), sem mover a forma.
    #[test]
    fn simplify_collapses_a_pixel_staircase_into_few_points() {
        // Um "retângulo" com 40 pontos, mas geometricamente com 4 quinas.
        let mut ring = Vec::new();
        for i in 0..10 {
            ring.push(Vec2::new(i as f32, 0.0));
        }
        for i in 0..10 {
            ring.push(Vec2::new(10.0, i as f32));
        }
        for i in 0..10 {
            ring.push(Vec2::new(10.0 - i as f32, 10.0));
        }
        for i in 0..10 {
            ring.push(Vec2::new(0.0, 10.0 - i as f32));
        }
        let out = simplify_ring(&ring, 1.25, 0);
        assert!(
            out.len() <= 8,
            "40 pontos colineares → poucas quinas, veio {}",
            out.len()
        );
        let a = signed_area(&out).abs();
        assert!((a - 100.0).abs() < 6.0, "a forma sobrevive: área {a}");
    }
}
