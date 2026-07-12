//! **Setas em bloco** — módulo irmão de [`crate::shapes`].
//!
//! Cuidado com a palavra "seta": num editor vetorial ela é **duas coisas diferentes**.
//! Aqui estão as setas-FORMA — polígonos preenchíveis, que se escalam, giram e recebem
//! gradiente como qualquer outra forma. A outra é a *ponta de traço* (arrowhead), que é
//! propriedade do **stroke** e vale para qualquer path aberto; essa não mora aqui.
//!
//! Todas são desenhadas apontando para **+X** dentro da caixa do gesto (o gizmo gira).
//! Os parâmetros são **frações** da caixa, não medidas absolutas — assim a seta guarda a
//! proporção ao ser redimensionada, que é o que se espera de uma forma paramétrica.

use crate::{VecPath, VecVertex};

/// Constrói um contorno fechado de quinas a partir de pontos crus.
fn corners(pts: Vec<[f64; 2]>) -> VecPath {
    VecPath {
        verts: pts.into_iter().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// **Seta reta.** Haste retangular + cabeça triangular apontando para +X.
///
/// - `tail` = espessura da haste (fração da altura);
/// - `head_len` = comprimento da cabeça (fração da largura);
/// - `head_w` = largura da cabeça (fração da altura) — `1.0` = a cabeça ocupa a caixa toda.
#[must_use]
pub fn arrow_block(a: [f64; 2], b: [f64; 2], tail: f64, head_len: f64, head_w: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let ht = hh * tail.clamp(0.02, 1.0); // meia-espessura da haste
    let hwd = hh * head_w.clamp(0.05, 1.0); // meia-largura da cabeça
    // A cabeça nunca pode ser mais fina que a haste, senão o contorno inverte.
    let hwd = hwd.max(ht);
    let neck = cx + hw - 2.0 * hw * head_len.clamp(0.02, 1.0); // onde a cabeça começa
    let (x0, x1) = (cx - hw, cx + hw);
    corners(vec![
        [x0, cy - ht],
        [neck, cy - ht],
        [neck, cy - hwd],
        [x1, cy],
        [neck, cy + hwd],
        [neck, cy + ht],
        [x0, cy + ht],
    ])
}

/// **Seta dupla** (cabeça nas duas pontas) — a mesma haste, espelhada.
#[must_use]
pub fn arrow_double(a: [f64; 2], b: [f64; 2], tail: f64, head_len: f64, head_w: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let ht = hh * tail.clamp(0.02, 1.0);
    let hwd = (hh * head_w.clamp(0.05, 1.0)).max(ht);
    // Cada cabeça leva `head_len` da largura; juntas não podem passar da caixa.
    let hl = (2.0 * hw * head_len.clamp(0.02, 0.5)).min(hw);
    let (x0, x1) = (cx - hw, cx + hw);
    let (nl, nr) = (x0 + hl, x1 - hl);
    corners(vec![
        [nl, cy - ht],
        [nr, cy - ht],
        [nr, cy - hwd],
        [x1, cy],
        [nr, cy + hwd],
        [nr, cy + ht],
        [nl, cy + ht],
        [nl, cy + hwd],
        [x0, cy],
        [nl, cy - hwd],
    ])
}

/// **Seta em L** (cotovelo): entra pela esquerda na base, dobra e sai para CIMA. É a seta
/// de fluxo — a que liga uma caixa à de cima sem um conector de verdade.
#[must_use]
pub fn arrow_bent(a: [f64; 2], b: [f64; 2], tail: f64, head_len: f64, head_w: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let t = (2.0 * hh * tail.clamp(0.02, 0.9)).min(hh); // espessura (cheia) do corpo
    let hl = (2.0 * hh * head_len.clamp(0.02, 0.9)).min(hh); // comprimento da cabeça
    let hwd = (2.0 * hw * head_w.clamp(0.05, 1.0) * 0.5).max(t * 0.5); // meia-largura da cabeça
    let (x0, y0) = (cx - hw, cy - hh); // canto inferior-esquerdo
    let (x1, y1) = (cx + hw, cy + hh);
    // O eixo da haste vertical fica encostado à direita, com a cabeça cabendo na caixa.
    let sx = x1 - hwd;
    corners(vec![
        [x0, y0],
        [sx + t * 0.5, y0],
        [sx + t * 0.5, y1 - hl],
        [sx + hwd, y1 - hl],
        [sx, y1],
        [sx - hwd, y1 - hl],
        [sx - t * 0.5, y1 - hl],
        [sx - t * 0.5, y0 + t],
        [x0, y0 + t],
    ])
}

/// **Chevron** (seta-fita de processo): pentágono com um entalhe atrás, para encaixar no
/// próximo — o vocabulário de "etapa 1 → etapa 2" de qualquer diagrama.
#[must_use]
pub fn chevron(a: [f64; 2], b: [f64; 2], head_len: f64, notch: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let hl = 2.0 * hw * head_len.clamp(0.02, 0.9);
    let nk = 2.0 * hw * notch.clamp(0.0, 0.9);
    let (x0, x1) = (cx - hw, cx + hw);
    corners(vec![
        [x0, cy - hh],
        [x1 - hl, cy - hh],
        [x1, cy],
        [x1 - hl, cy + hh],
        [x0, cy + hh],
        [x0 + nk, cy], // o entalhe (0 = pentágono liso)
    ])
}

/// **Seta curva / circular**: um arco GROSSO com a cabeça na ponta. `sweep` grande dá a
/// seta de "refazer"; pequeno, a curva suave entre dois pontos.
///
/// - `tail` = espessura do arco (fração do raio);
/// - `head_len` = quanto do sweep a cabeça consome (fração);
/// - `head_w` = o quanto a cabeça extravasa a espessura (fração do raio).
#[must_use]
pub fn arrow_curved(
    a: [f64; 2],
    b: [f64; 2],
    sweep_deg: f64,
    tail: f64,
    head_len: f64,
    head_w: f64,
) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let sweep = sweep_deg.clamp(10.0, 350.0);
    let t = tail.clamp(0.05, 0.6); // espessura em fração do raio
    let head_sweep = sweep * head_len.clamp(0.05, 0.6);
    let body = sweep - head_sweep;
    // Raios do corpo (o arco grosso) + a cabeça, que extravasa a faixa dos dois lados.
    let ext = head_w.clamp(0.0, 0.5);
    let (mut r_in, mut r_out) = (1.0 - t, 1.0);
    let (mut h_in, mut h_out) = ((r_in - ext).max(0.02), r_out + ext);
    // **Tudo é normalizado pelo MAIOR raio** (o da cabeça): sem isto a cabeça estouraria
    // a caixa do gesto, a bbox mentiria e o gizmo ficaria desalinhado da forma. Quem
    // encosta na borda é a parte mais externa — as demais encolhem na proporção.
    let k = 1.0 / h_out;
    r_in *= k;
    r_out *= k;
    h_in *= k;
    h_out *= k;

    let pt = |r: f64, deg: f64| {
        let (s, c) = deg.to_radians().sin_cos();
        [cx + hw * r * c, cy + hh * r * s]
    };
    let n = ((body / 15.0).ceil() as usize).max(2);
    let mut pts: Vec<[f64; 2]> = Vec::with_capacity(2 * n + 4);
    // Borda externa do corpo, do início até o pescoço.
    for i in 0..=n {
        pts.push(pt(r_out, body * i as f64 / n as f64));
    }
    // A cabeça: base larga no pescoço → ponta no fim do sweep.
    pts.push(pt(h_out, body));
    pts.push(pt((h_in + h_out) * 0.5, sweep)); // a ponta
    pts.push(pt(h_in, body));
    // Borda interna, de volta.
    for i in (0..=n).rev() {
        pts.push(pt(r_in, body * i as f64 / n as f64));
    }
    corners(pts)
}

/// Centro + semi-eixos da caixa do gesto.
fn box_of(a: [f64; 2], b: [f64; 2]) -> (f64, f64, f64, f64) {
    (
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (b[0] - a[0]).abs() * 0.5,
        (b[1] - a[1]).abs() * 0.5,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [f64; 2] = [-2.0, -1.0];
    const B: [f64; 2] = [2.0, 1.0];

    fn xs(p: &VecPath) -> (f64, f64) {
        let lo = p.verts_all().map(|v| v.anchor[0]).fold(f64::MAX, f64::min);
        let hi = p.verts_all().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
        (lo, hi)
    }

    /// A seta APONTA: a ponta é o único vértice no extremo +X, e ela fica na linha do
    /// meio. É o que distingue uma seta de um retângulo com um entalhe.
    #[test]
    fn the_block_arrow_has_a_single_tip_at_the_right_edge() {
        let p = arrow_block(A, B, 0.4, 0.4, 1.0);
        let (_, hi) = xs(&p);
        let tips: Vec<&VecVertex> = p
            .verts
            .iter()
            .filter(|v| (v.anchor[0] - hi).abs() < 1e-9)
            .collect();
        assert_eq!(tips.len(), 1, "uma ponta so");
        assert!((tips[0].anchor[1]).abs() < 1e-9, "a ponta esta no eixo");
        assert!((hi - 2.0).abs() < 1e-9, "a ponta encosta na borda da caixa");
    }

    /// A cabeça é mais LARGA que a haste — senão não é uma seta, é uma barra. O clamp
    /// garante isso mesmo com um `head_w` menor que o `tail` (o usuário pode digitar).
    #[test]
    fn the_head_is_never_narrower_than_the_tail() {
        let p = arrow_block(A, B, 0.9, 0.4, 0.1); // cabeça pedida MENOR que a haste
        let ys: Vec<f64> = p.verts.iter().map(|v| v.anchor[1].abs()).collect();
        let widest = ys.iter().cloned().fold(0.0, f64::max);
        let tail = 1.0 * 0.9; // meia-espessura pedida
        assert!(
            widest >= tail - 1e-9,
            "o contorno inverteria: cabeca {widest} < haste {tail}"
        );
    }

    /// A seta dupla tem ponta nos DOIS extremos (e a caixa inteira entre elas).
    #[test]
    fn the_double_arrow_points_both_ways() {
        let p = arrow_double(A, B, 0.4, 0.3, 1.0);
        let (lo, hi) = xs(&p);
        assert!((lo + 2.0).abs() < 1e-9 && (hi - 2.0).abs() < 1e-9);
        let on_axis = p.verts.iter().filter(|v| v.anchor[1].abs() < 1e-9).count();
        assert_eq!(on_axis, 2, "as duas pontas ficam no eixo");
    }

    /// O chevron tem entalhe: com `notch = 0` é um pentágono; acima disso, o vértice de
    /// trás entra na caixa (é o encaixe do próximo chevron).
    #[test]
    fn the_chevron_notch_bites_into_the_back_edge() {
        let flat = chevron(A, B, 0.3, 0.0);
        let bitten = chevron(A, B, 0.3, 0.25);
        let back_flat = flat.verts[5].anchor[0];
        let back_bitten = bitten.verts[5].anchor[0];
        assert!(
            (back_flat + 2.0).abs() < 1e-9,
            "sem entalhe, a traseira e reta"
        );
        assert!(back_bitten > back_flat, "o entalhe entra na forma");
    }

    /// Todas as setas cabem na caixa do gesto — nenhuma extravasa (o que faria a bbox
    /// mentir e o gizmo desalinhar da forma).
    #[test]
    fn every_arrow_fits_inside_the_gesture_box() {
        let shapes = [
            arrow_block(A, B, 0.4, 0.4, 1.0),
            arrow_double(A, B, 0.4, 0.3, 1.0),
            arrow_bent(A, B, 0.3, 0.3, 0.6),
            chevron(A, B, 0.3, 0.2),
            arrow_curved(A, B, 270.0, 0.3, 0.25, 0.15),
        ];
        for (i, p) in shapes.iter().enumerate() {
            for v in p.verts_all() {
                assert!(
                    v.anchor[0] >= -2.0 - 1e-6
                        && v.anchor[0] <= 2.0 + 1e-6
                        && v.anchor[1] >= -1.0 - 1e-6
                        && v.anchor[1] <= 1.0 + 1e-6,
                    "forma {i}: ancora {:?} fora da caixa",
                    v.anchor
                );
            }
        }
    }
}
