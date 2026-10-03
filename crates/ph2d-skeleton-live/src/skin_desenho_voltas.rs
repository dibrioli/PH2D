//! ⭐⭐ **As VOLTAS APERTADAS de um cozido viram NÓS antes do bake** (F50-h, report do dono de
//! 2026-10-03, foto: *«bloat +60 aparecem linhas bizarras»*).
//!
//! Um efeito forte desenha pontas que dão a volta DENTRO de um só segmento — o *Bloat* `+60` na
//! cápsula põe as alças de um lado a `~3,6` da âncora, e a cúbica vai a uma ponta, volta, e passa
//! por outra. O bake amostra cada segmento a passo fixo e ajusta pela tangente de Catmull-Rom, e
//! numa ponta a tangente de Catmull-Rom aponta de LADO: o ajuste sai em traços rectos
//! perpendiculares à ponta. Medido na barra a `60°`: o desenho afastava-se `0,92` do padrão-ouro
//! (o padrão-ouro dele `0,06`). ⇒ partir o segmento onde ele vira (de Casteljau, exacto: a curva
//! em repouso não muda um ponto) põe a ponta num NÓ, e o bake já preserva nós.

use ph2d_vec_scene::{VecPath, VecVertex};

/// ⭐ **A viragem máxima de um pedaço**, em graus — acima dela o pedaço é partido ao meio da
/// viragem. `90°` deixa cada pedaço com a tangente de Catmull-Rom dentro do mesmo quadrante.
pub const VIRAGEM_MAXIMA: f64 = 90.0;

/// A profundidade máxima da partição — `2⁶ = 64` pedaços por segmento no pior caso. ⚠️ Uma rede,
/// não um tecto de qualidade: uma cúbica vira no máximo `~360°` e a partição pára muito antes.
const PROFUNDIDADE: u32 = 6;

type Cub = [[f64; 2]; 4];

fn eval_d(c: &Cub, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let k = [3.0 * u * u, 6.0 * u * t, 3.0 * t * t];
    let d = |i: usize| [c[i + 1][0] - c[i][0], c[i + 1][1] - c[i][1]];
    let (a, b, e) = (d(0), d(1), d(2));
    [
        k[0] * a[0] + k[1] * b[0] + k[2] * e[0],
        k[0] * a[1] + k[1] * b[1] + k[2] * e[1],
    ]
}

/// `(viragem total em graus, t onde ela chega a metade)` — pela tangente amostrada; os pontos de
/// derivada nula (a própria cúspide) não votam.
fn viragem(c: &Cub) -> (f64, f64) {
    const N: usize = 64;
    let mut angulos: Vec<(f64, f64)> = Vec::with_capacity(N + 1);
    for i in 0..=N {
        #[expect(clippy::cast_precision_loss, reason = "N pequeno")]
        let t = i as f64 / N as f64;
        let d = eval_d(c, t);
        if d[0].hypot(d[1]) > 1e-12 {
            angulos.push((t, d[1].atan2(d[0])));
        }
    }
    let mut acum = vec![0.0];
    for w in angulos.windows(2) {
        let mut da = w[1].1 - w[0].1;
        while da > std::f64::consts::PI {
            da -= std::f64::consts::TAU;
        }
        while da < -std::f64::consts::PI {
            da += std::f64::consts::TAU;
        }
        acum.push(acum.last().copied().unwrap_or(0.0) + da.abs());
    }
    let total = acum.last().copied().unwrap_or(0.0);
    let meio = angulos
        .iter()
        .zip(&acum)
        .find(|(_, a)| **a >= total / 2.0)
        .map_or(0.5, |((t, _), _)| *t)
        .clamp(0.05, 0.95);
    (total.to_degrees(), meio)
}

fn parte(c: &Cub, t: f64) -> (Cub, Cub) {
    let l = |a: [f64; 2], b: [f64; 2]| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
    let (p01, p12, p23) = (l(c[0], c[1]), l(c[1], c[2]), l(c[2], c[3]));
    let (p012, p123) = (l(p01, p12), l(p12, p23));
    let m = l(p012, p123);
    ([c[0], p01, p012, m], [m, p123, p23, c[3]])
}

fn pedacos(c: Cub, fundo: u32, out: &mut Vec<Cub>) {
    let (v, t) = viragem(&c);
    if v <= VIRAGEM_MAXIMA || fundo >= PROFUNDIDADE {
        out.push(c);
        return;
    }
    let (a, b) = parte(&c, t);
    pedacos(a, fundo + 1, out);
    pedacos(b, fundo + 1, out);
}

/// Um contorno com cada segmento partido onde vira mais que [`VIRAGEM_MAXIMA`].
fn contorno(verts: &[VecVertex], fechado: bool) -> Vec<VecVertex> {
    let n = verts.len();
    let segs = if fechado { n } else { n.saturating_sub(1) };
    if segs == 0 {
        return verts.to_vec();
    }
    let mut out: Vec<VecVertex> = Vec::with_capacity(n);
    for k in 0..segs {
        let (a, b) = (&verts[k], &verts[(k + 1) % n]);
        let mut ps = Vec::new();
        pedacos([a.anchor, a.out_handle, b.in_handle, b.anchor], 0, &mut ps);
        if k == 0 {
            out.push(*a);
        }
        if let Some(ultimo) = out.last_mut() {
            ultimo.out_handle = ps[0][1];
        }
        for (i, p) in ps.iter().enumerate() {
            let fim = i + 1 == ps.len();
            let mut v = if fim { *b } else { *a };
            v.anchor = p[3];
            v.in_handle = p[2];
            v.corner_radius = 0.0;
            if !fim {
                v.out_handle = ps[i + 1][1];
                v.kind = ph2d_vec_scene::VertexKind::Smooth;
            }
            out.push(v);
        }
    }
    if fechado {
        // O último nó empurrado é o 1.º outra vez: ele leva a alça de ENTRADA para o 1.º.
        if let Some(fecho) = out.pop() {
            out[0].in_handle = fecho.in_handle;
        }
    }
    out
}

/// ⭐⭐ **O cozido com as voltas apertadas em nós** — ver o cabeçalho. Exacto: a curva em repouso
/// é a mesma, ponto a ponto (de Casteljau).
#[must_use]
pub fn parte_nas_voltas(mut p: VecPath) -> VecPath {
    for c in 0..p.contour_count() {
        if let Some((verts, fechado)) = p.contour_mut(c) {
            *verts = contorno(verts, *fechado);
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐ **Partir não muda a curva, e nenhum pedaço vira mais que o tecto.**
    #[test]
    fn partir_nao_muda_a_curva_e_nenhum_pedaco_vira_demais() {
        let mut barra = ph2d_vec_scene::cook_tinted(
            ph2d_vec_scene::ShapeKind::RoundRect,
            [-2.25, -0.375],
            [2.25, 0.375],
            &[0.375],
            [0, 0, 0],
        );
        barra.effects = vec![ph2d_vec_scene::effect::FxEntry::new(
            ph2d_vec_scene::effect::PathEffect::Bloat(ph2d_vec_scene::fx_warp::BloatSpec {
                amount: 60.0,
            }),
        )];
        let cozido = barra.cooked().into_owned();
        let partido = parte_nas_voltas(cozido.clone());
        assert!(
            partido.verts.len() > cozido.verts.len(),
            "o CONTROLO: o Bloat +60 tem segmentos que viram mais que {VIRAGEM_MAXIMA}°"
        );
        let n = partido.verts.len();
        for k in 0..n {
            let (a, b) = (&partido.verts[k], &partido.verts[(k + 1) % n]);
            let (v, _) = viragem(&[a.anchor, a.out_handle, b.in_handle, b.anchor]);
            assert!(v <= VIRAGEM_MAXIMA + 1e-6, "o pedaço {k} vira {v:.1}°");
        }
        // A mesma curva: cada nó novo está SOBRE a original (de Casteljau é exacto).
        let m = cozido.verts.len();
        let original: Vec<[f64; 2]> = (0..m)
            .flat_map(|k| {
                let (a, b) = (&cozido.verts[k], &cozido.verts[(k + 1) % m]);
                let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
                (0..=512).map(move |i| {
                    let t = f64::from(i) / 512.0;
                    let u = 1.0 - t;
                    let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
                    [
                        w[0] * c[0][0] + w[1] * c[1][0] + w[2] * c[2][0] + w[3] * c[3][0],
                        w[0] * c[0][1] + w[1] * c[1][1] + w[2] * c[2][1] + w[3] * c[3][1],
                    ]
                })
            })
            .collect();
        for v in &partido.verts {
            let d = original
                .iter()
                .map(|q| (q[0] - v.anchor[0]).hypot(q[1] - v.anchor[1]))
                .fold(f64::INFINITY, f64::min);
            assert!(d < 1e-3, "o nó {:?} saiu da curva ({d})", v.anchor);
        }
    }
}
