//! ⭐ **A *Resize Box* MANTÉM a espessura do traço** (F13 B — o dono, 12/10: «faça como sugere»; o
//! *Scale Strokes & Effects* do Illustrator, desligado). A lei e o desvio medido: doc 17 §2
//! (`docs/UI_New_and_Simple/vetor_nova_ui/17_gizmos_do_illustrator.md`).
//!
//! A caixa que o artista arrasta é a VISUAL (o traço incluído). Com a caneta fixa, escalar a
//! geometria pela razão da caixa deixaria as bordas a `m·(r − 1)` do sítio (`m` = a margem do traço);
//! aqui a geometria escala o que FALTA à caixa visual — a margem fica — e o pivô passa para o ponto
//! que a deixa exacta nas duas bordas.

use ph2d_vec_scene::{PaintKind, VecPath, VecPathId, VecScene};

/// A menor razão de geometria (positiva: a caixa nunca vira do avesso) — a da moldura.
const MIN_RATIO: f64 = 1e-4;

/// Passes de acerto: a margem do traço só muda com a razão quando um bico em esquadria muda de
/// ângulo (escala desigual); no proporcional o 1.º passe é exacto. Medido (12/10, a estrela de
/// fábrica ×(2, 1), tolerância 1e-6 da caixa): a secante fecha em 5–6, 4 não chega; o ponto fixo
/// simples (sem secante) precisava de dezenas.
const PASSES: usize = 6;

/// Repõe em `p` as larguras e os efeitos de `start`: o traço, cada traço da pilha, o ladrilho da
/// estampa do traço, o deslocamento e a dilatação de cada camada.
pub fn keep_the_pen(p: &mut VecPath, start: &VecPath) {
    if let (Some(s), Some(s0)) = (p.stroke.as_mut(), start.stroke.as_ref()) {
        s.width = s0.width;
        if let (Some(t), Some(t0)) = (s.pattern_mut(), s0.pattern()) {
            t.size = t0.size;
        }
    }
    for (e, e0) in p.paints.iter_mut().zip(&start.paints) {
        if let (PaintKind::Stroke(s), PaintKind::Stroke(s0)) = (&mut e.kind, &e0.kind) {
            s.width = s0.width;
        }
        e.offset = e0.offset;
        e.dilate = e0.dilate;
    }
}

/// Redimensiona o caminho `id` (já reposto em `start`) para que a caixa VISUAL dele — sob a escala de
/// mundo `world_scale` — escale `s` em torno de `anchor` (local), com a caneta de `start`. Devolve a
/// razão da GEOMETRIA em cada eixo (a que a receita de uma forma viva segue).
pub fn resize_keeping_the_pen(
    scene: &mut VecScene,
    id: VecPathId,
    start: &VecPath,
    world_scale: [f64; 2],
    anchor: [f64; 2],
    s: [f64; 2],
) -> [f64; 2] {
    let boxes = |scene: &VecScene| {
        let p = scene.path(id)?;
        let geo = scene.path_curve_bbox(id)?;
        let vis = ph2d_vec_render::path_intrinsic_visual_bbox(p, world_scale).unwrap_or(geo);
        Some((geo, vis))
    };
    let Some(((glo, ghi), (vlo, vhi))) = boxes(scene) else {
        return [1.0; 2];
    };
    // A caixa visual pedida, e a margem do traço de cada lado (a de partida, depois a medida).
    let alvo = |i: usize| {
        (
            anchor[i] + (vlo[i] - anchor[i]) * s[i],
            anchor[i] + (vhi[i] - anchor[i]) * s[i],
        )
    };
    let mut margem = [[glo[0] - vlo[0], vhi[0] - ghi[0]], [glo[1] - vlo[1], vhi[1] - ghi[1]]];
    let largura = [ghi[0] - glo[0], ghi[1] - glo[1]];
    let tol = 1e-9 * (vhi[0] - vlo[0]).max(vhi[1] - vlo[1]).max(1.0);
    // O 1.º passe com a margem de partida; depois, a secante sobre o erro da largura visual.
    let mut g: [f64; 2] = std::array::from_fn(|i| {
        let (tlo, thi) = alvo(i);
        if largura[i] > 1e-12 {
            ((thi - margem[i][1] - tlo - margem[i][0]) / largura[i]).max(MIN_RATIO)
        } else {
            s[i]
        }
    });
    let mut antes: [Option<(f64, f64)>; 2] = [None; 2];
    for _ in 0..PASSES {
        // O ponto que a escala `g` deixa onde está, com o canto mínimo da curva a cair na margem.
        let pivot: [f64; 2] = std::array::from_fn(|i| {
            let lo = alvo(i).0 + margem[i][0];
            if (1.0 - g[i]).abs() > 1e-12 {
                (lo - g[i] * glo[i]) / (1.0 - g[i])
            } else {
                anchor[i]
            }
        });
        if let Some(p) = scene.path_mut(id) {
            *p = start.clone();
        }
        scene.scale_path(id, g[0], g[1], pivot);
        if let Some(p) = scene.path_mut(id) {
            keep_the_pen(p, start);
        }
        let Some(((nlo, nhi), (wlo, whi))) = boxes(scene) else {
            break;
        };
        let erro = (0..2)
            .map(|i| {
                let (tlo, thi) = alvo(i);
                (wlo[i] - tlo).abs().max((whi[i] - thi).abs())
            })
            .fold(0.0, f64::max);
        if erro < tol {
            break;
        }
        margem = [[nlo[0] - wlo[0], whi[0] - nhi[0]], [nlo[1] - wlo[1], whi[1] - nhi[1]]];
        for i in 0..2 {
            let (tlo, thi) = alvo(i);
            let f = (whi[i] - wlo[i]) - (thi - tlo);
            let proximo = match antes[i] {
                Some((gp, fp)) if (f - fp).abs() > 1e-15 => g[i] - f * (g[i] - gp) / (f - fp),
                _ if largura[i] > 1e-12 => g[i] - f / largura[i],
                _ => g[i],
            };
            antes[i] = Some((g[i], f));
            g[i] = proximo.max(MIN_RATIO);
        }
    }
    g
}

#[cfg(test)]
#[path = "vec_frame_pen_tests.rs"]
mod tests;
