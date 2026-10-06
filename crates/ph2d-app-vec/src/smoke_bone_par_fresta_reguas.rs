//! ⭐⭐ **As RÉGUAS da costura** (A5-a) — filhas dos gates ([`super`]) para herdar o palco; o corte
//! é o tecto de LOC. Duas delas leem o que se VÊ: a que conhece a ordem das faces e a do vão fixo.

use super::*;

/// Os triângulos COSIDOS de `com` — os que usam um ponto que `sem` não tem (a costura entra
/// intercalada na malha, à profundidade do seu membro).
pub(super) fn cosidos<'a>(
    sem: &SpriteMesh,
    com: &'a SpriteMesh,
) -> impl Iterator<Item = &'a [u32; 3]> {
    let n = sem.local.len();
    com.tris
        .iter()
        .filter(move |t| t.iter().any(|&i| i as usize >= n))
}

/// `com` com a costura no FIM — a ordem de antes, o controlo da régua que conhece a ordem.
pub(super) fn no_fim(sem: &SpriteMesh, com: &SpriteMesh) -> SpriteMesh {
    let n = sem.local.len();
    let cosido = |t: &&[u32; 3]| t.iter().any(|&i| i as usize >= n);
    let mut tris: Vec<[u32; 3]> = com.tris.iter().filter(|t| !cosido(t)).copied().collect();
    tris.extend(com.tris.iter().filter(cosido).copied());
    SpriteMesh {
        tris,
        ..com.clone()
    }
}

/// A UV que o triângulo `t` de `m` pinta em `q`, se `q` estiver dentro dele.
fn uv_no_tri(m: &SpriteMesh, t: [u32; 3], q: [f64; 2]) -> Option<[f64; 2]> {
    let v = t.map(|i| m.local[i as usize].map(f64::from));
    if !dentro(q, v[0], v[1], v[2]) {
        return None;
    }
    let area =
        (v[1][0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
    if area == 0.0 {
        return None;
    }
    let b1 =
        ((q[0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[2][0] - v[0][0]) * (q[1] - v[0][1])) / area;
    let b2 =
        ((v[1][0] - v[0][0]) * (q[1] - v[0][1]) - (q[0] - v[0][0]) * (v[1][1] - v[0][1])) / area;
    let w = [1.0 - b1 - b2, b1, b2];
    Some([0, 1].map(|c| {
        (0..3)
            .map(|k| w[k] * f64::from(m.uv[t[k] as usize][c]))
            .sum()
    }))
}

/// A grelha de `1/4` px com origem `o` e `[w, a]` amostras: em cada uma, a UV do ÚLTIMO triângulo
/// (pela ordem) que ali tem TINTA (alfa `≥ 128`) — o que se vê.
fn pinta(m: &SpriteMesh, o: [f64; 2], [w, a]: [usize; 2]) -> Vec<Option<[f64; 2]>> {
    let h = 0.25 / f64::from(PPM);
    let alfa = alfa_da_arte();
    let mut buf = vec![None; w * a];
    #[expect(clippy::cast_precision_loss, reason = "amostras")]
    let fim = [o[0] + w as f64 * h, o[1] + a as f64 * h];
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "amostras"
    )]
    let ix = |x: f64, n: usize| ((x / h).max(0.0) as usize).min(n - 1);
    for t in &m.tris {
        let v = t.map(|i| m.local[i as usize].map(f64::from));
        let lo = v
            .iter()
            .fold([f64::MAX; 2], |c, q| [c[0].min(q[0]), c[1].min(q[1])]);
        let hi = v
            .iter()
            .fold([f64::MIN; 2], |c, q| [c[0].max(q[0]), c[1].max(q[1])]);
        if hi[0] < o[0] || lo[0] > fim[0] || hi[1] < o[1] || lo[1] > fim[1] {
            continue;
        }
        for j in ix(lo[1] - o[1], a)..=ix(hi[1] - o[1] + h, a) {
            for i in ix(lo[0] - o[0], w)..=ix(hi[0] - o[0] + h, w) {
                #[expect(clippy::cast_precision_loss, reason = "amostras")]
                let q = [o[0] + i as f64 * h, o[1] + j as f64 * h];
                let Some(uv) = uv_no_tri(m, *t, q) else {
                    continue;
                };
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "texel"
                )]
                let (x, y) = (
                    ((uv[0] * f64::from(IMG_W)) as u32).min(IMG_W - 1),
                    ((uv[1] * f64::from(IMG_H)) as u32).min(IMG_H - 1),
                );
                if alfa[(y * IMG_W + x) as usize] >= 128 {
                    buf[j * w + i] = Some(uv);
                }
            }
        }
    }
    buf
}

/// ⭐⭐⭐ **A régua que CONHECE A ORDEM**: onde a costura toca (a caixa dos triângulos cosidos),
/// as amostras em que `sem` mostra tinta e `com` mostra um texel de OUTRA zona (UV a mais de `4`
/// texels) — a costura mudou à vista um pixel pintado.
pub(super) fn visiveis_sobre_tinta(sem: &SpriteMesh, com: &SpriteMesh) -> usize {
    let mut cx = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for t in cosidos(sem, com) {
        for &i in t {
            let q = com.local[i as usize].map(f64::from);
            cx = [
                cx[0].min(q[0]),
                cx[1].min(q[1]),
                cx[2].max(q[0]),
                cx[3].max(q[1]),
            ];
        }
    }
    if cx[0] > cx[2] {
        return 0;
    }
    let h = 0.25 / f64::from(PPM);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "amostras"
    )]
    let wa = [
        ((cx[2] - cx[0]) / h).ceil() as usize + 1,
        ((cx[3] - cx[1]) / h).ceil() as usize + 1,
    ];
    let o = [cx[0], cx[1]];
    let (s, c) = (pinta(sem, o, wa), pinta(com, o, wa));
    s.iter()
        .zip(&c)
        .filter(|(s, c)| match (s, c) {
            (Some(u), Some(v)) => {
                ((u[0] - v[0]) * f64::from(IMG_W)).hypot((u[1] - v[1]) * f64::from(IMG_H)) > 4.0
            }
            _ => false,
        })
        .count()
}

/// Dilatação por um disco de raio `r` amostras (fora da grelha vale `fora`).
fn dilata(b: &[bool], [w, a]: [usize; 2], r: i64, fora: bool) -> Vec<bool> {
    let disco: Vec<(i64, i64)> = (-r..=r)
        .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
        .filter(|(dx, dy)| dx * dx + dy * dy <= r * r)
        .collect();
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss, reason = "grelha")]
    (0..w * a)
        .map(|k| {
            let (i, j) = ((k % w) as i64, (k / w) as i64);
            disco.iter().any(|&(dx, dy)| {
                let (x, y) = (i + dx, j + dy);
                if x < 0 || y < 0 || x >= w as i64 || y >= a as i64 {
                    fora
                } else {
                    b[y as usize * w + x as usize]
                }
            })
        })
        .collect()
}

/// ⭐⭐⭐ **A régua do VÃO FIXO** numa janela: o vão é o da malha `sem` costura — o fecho
/// morfológico a `1 px` da tinta dela, menos a tinta — e para cada malha de `com` conta-se o que
/// dele fica sem tinta. Mais tinta nunca sobe a conta (a régua do fio na vertical subia quando um
/// vão largo fechava em parte).
pub(super) fn vao_aberto<const N: usize>(
    sem: &SpriteMesh,
    com: [&SpriteMesh; N],
    [x0, y0, x1, y1]: [f64; 4],
) -> [usize; N] {
    let h = 0.25 / f64::from(PPM);
    let (pad, r) = (8_usize, 4_i64);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "amostras"
    )]
    let (w0, a0) = (
        ((x1 - x0) / h + 1e-6).floor() as usize + 1,
        ((y1 - y0) / h + 1e-6).floor() as usize + 1,
    );
    let wa = [w0 + 2 * pad, a0 + 2 * pad];
    #[expect(clippy::cast_precision_loss, reason = "amostras")]
    let o = [x0 - pad as f64 * h, y0 - pad as f64 * h];
    let tinta =
        |m: &SpriteMesh| -> Vec<bool> { pinta(m, o, wa).iter().map(Option::is_some).collect() };
    let base = tinta(sem);
    let d = dilata(&base, wa, r, false);
    let neg: Vec<bool> = d.iter().map(|x| !x).collect();
    let fecho: Vec<bool> = dilata(&neg, wa, r, true).iter().map(|x| !x).collect();
    let na_janela =
        |k: usize| (pad..pad + w0).contains(&(k % wa[0])) && (pad..pad + a0).contains(&(k / wa[0]));
    com.map(|m| {
        let t = tinta(m);
        (0..wa[0] * wa[1])
            .filter(|&k| fecho[k] && !base[k] && !t[k] && na_janela(k))
            .count()
    })
}

/// Quantos centros de triângulo COSIDOS caem em cima de tinta da malha sem costura — e de quantos.
pub(super) fn cosidos_sobre_tinta(p: &Palco) -> (usize, usize) {
    let (sem, com) = (desenhada(p, false, false), desenhada(p, true, false));
    let mut sobre = 0;
    let novos: Vec<[u32; 3]> = cosidos(&sem, &com).copied().collect();
    for t in &novos {
        let c = t.iter().fold([0.0, 0.0], |a, &i| {
            [
                a[0] + f64::from(com.local[i as usize][0]) / 3.0,
                a[1] + f64::from(com.local[i as usize][1]) / 3.0,
            ]
        });
        let h = 1e-4;
        if buracos(&sem, [c[0] - h, c[1] - h, c[0] + h, c[1] + h], 0.0) == 0
            && tinta_em(&sem, c)
            && !tinta_da_mesma_zona(&sem, c, t.map(|i| com.uv[i as usize]))
        {
            sobre += 1;
        }
    }
    (sobre, novos.len())
}

/// A tinta de `m` em `q` é da MESMA zona da imagem que o pedaço cosido (`uv` dele) — a menos de
/// `4` texels? Um remendo de `0,1` texel² sobre a tinta da própria beira, com a cor dela, não pinta
/// um membro por cima de outro (o que este gate guarda); MEDIDO na A5-a: `4` de `236` a `−144°`, onde
/// a borda recta encontra a tampa. Pela UV e não pela chave de osso (a régua não usa a lei).
pub(super) fn tinta_da_mesma_zona(m: &SpriteMesh, q: [f64; 2], uv: [[f32; 2]; 3]) -> bool {
    let p = |i: u32| {
        [
            f64::from(m.local[i as usize][0]),
            f64::from(m.local[i as usize][1]),
        ]
    };
    let meio = [0, 1].map(|k| (uv[0][k] + uv[1][k] + uv[2][k]) / 3.0);
    m.tris.iter().any(|t| {
        let (a, b, c) = (p(t[0]), p(t[1]), p(t[2]));
        if !dentro(q, a, b, c) {
            return false;
        }
        let v = [0, 1].map(|k| {
            (m.uv[t[0] as usize][k] + m.uv[t[1] as usize][k] + m.uv[t[2] as usize][k]) / 3.0
        });
        #[expect(clippy::cast_precision_loss, reason = "texels")]
        let d = ((v[0] - meio[0]) * IMG_W as f32).hypot((v[1] - meio[1]) * IMG_H as f32);
        d <= 4.0
    })
}

/// Há tinta da malha `m` no ponto `q`?
pub(super) fn tinta_em(m: &SpriteMesh, q: [f64; 2]) -> bool {
    let p = |i: u32| {
        [
            f64::from(m.local[i as usize][0]),
            f64::from(m.local[i as usize][1]),
        ]
    };
    let alfa = alfa_da_arte();
    m.tris.iter().any(|t| {
        let (a, b, c) = (p(t[0]), p(t[1]), p(t[2]));
        if !dentro(q, a, b, c) {
            return false;
        }
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if area == 0.0 {
            return false;
        }
        let b1 = ((q[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (q[1] - a[1])) / area;
        let b2 = ((b[0] - a[0]) * (q[1] - a[1]) - (q[0] - a[0]) * (b[1] - a[1])) / area;
        let w = [1.0 - b1 - b2, b1, b2];
        let (mut u, mut v) = (0.0, 0.0);
        for k in 0..3 {
            u += w[k] * f64::from(m.uv[t[k] as usize][0]);
            v += w[k] * f64::from(m.uv[t[k] as usize][1]);
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "texel"
        )]
        let (x, y) = (
            ((u * f64::from(IMG_W)) as u32).min(IMG_W - 1),
            ((v * f64::from(IMG_H)) as u32).min(IMG_H - 1),
        );
        alfa[(y * IMG_W + x) as usize] >= 128
    })
}
