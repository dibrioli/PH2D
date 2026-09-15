//! ⭐⭐⭐ **O AJUSTE DE GRAU 3 DO MAPA DA MALHA** — irmão do [`super::sprite_mesh_warp`] por
//! RESPONSABILIDADE: ali mora *que amostras tirar da malha*, aqui *que polinómio elas determinam*.
//!
//! # Porque grau 3, medido (ver `sprite_mesh_warp_probe`)
//!
//! Na direcção que o motor de facto avalia (texel → ecrã), a redondeza da marca com um mapa de
//! grau `g`:
//!
//! | raio do dab | `g = 1` | `g = 2` | `g = 3` |
//! |---|---|---|---|
//! | `0,06` | `1,054` | `1,010` | `1,008` |
//! | `0,125` | `1,118` | `1,021` | `1,005` |
//! | `0,20` | `1,197` | `1,055` | **`1,006`** |
//!
//! ⛔ O grau `2` deixa `5,5 %` no pincel grande, que é onde o dono aponta. O `3` leva o pior caso a
//! `0,6 %`, abaixo do que a métrica distingue — e custa quatro monómios por eixo.
//!
//! ⚠️ **As amostras são `2` anéis × `12` ângulos, e os dois números são a lei, não decoração:** um
//! polinómio de grau `3` tem `9` incógnitas por eixo e as suas harmónicas angulares vão até `3`,
//! logo `12` ângulos resolvem-nas com folga (`8` seria o mínimo estrito) e `2` raios separam o que
//! é grau `2` do que é grau `3`. Com um anel só o sistema é **singular** por construção.

/// Quantos termos tem um polinómio de grau `3` em duas variáveis, sem o termo constante (a origem é
/// o centro do dab): `x`, `y`, `x²`, `x·y`, `y²`, `x³`, `x²·y`, `x·y²`, `y³`.
pub(crate) const TERMOS: usize = 9;

/// Os monómios de `(x, y)` na ordem de [`TERMOS`].
pub(crate) fn monomios(x: f64, y: f64) -> [f64; TERMOS] {
    let (xx, xy, yy) = (x * x, x * y, y * y);
    [x, y, xx, xy, yy, xx * x, xx * y, xy * y, yy * y]
}

/// ⭐ **A PORTA**: os coeficientes de grau `1..3` que melhor levam `entrada[k]` a `saida[k]`, por
/// mínimos quadrados em `f64`. `None` se o sistema for singular (amostras degeneradas).
///
/// ⚠️ **O `f64` não é conforto:** os monómios de grau `3` sobre um disco unitário dão uma matriz
/// normal com número de condição na casa dos milhares, e em `f32` o termo cúbico sai com o sinal
/// errado em parte do corpus — medido enquanto esta função foi escrita.
#[must_use]
pub(crate) fn ajusta(entrada: &[[f32; 2]], saida: &[[f32; 2]]) -> Option<[[f64; TERMOS]; 2]> {
    if entrada.len() != saida.len() || entrada.len() < TERMOS {
        return None;
    }
    let (mut ata, mut atb) = ([[0.0f64; TERMOS]; TERMOS], [[0.0f64; TERMOS]; 2]);
    for (p, q) in entrada.iter().zip(saida.iter()) {
        let b = monomios(f64::from(p[0]), f64::from(p[1]));
        for i in 0..TERMOS {
            for j in 0..TERMOS {
                ata[i][j] += b[i] * b[j];
            }
            atb[0][i] += b[i] * f64::from(q[0]);
            atb[1][i] += b[i] * f64::from(q[1]);
        }
    }
    // Gauss com pivô parcial sobre `TERMOS` incógnitas e DOIS lados direitos.
    let mut m = [[0.0f64; TERMOS + 2]; TERMOS];
    for i in 0..TERMOS {
        m[i][..TERMOS].copy_from_slice(&ata[i]);
        m[i][TERMOS] = atb[0][i];
        m[i][TERMOS + 1] = atb[1][i];
    }
    for c in 0..TERMOS {
        let piv = (c..TERMOS)
            .max_by(|&a, &b| m[a][c].abs().total_cmp(&m[b][c].abs()))
            .expect("ha' pelo menos uma linha");
        m.swap(c, piv);
        let pv = m[c][c];
        if !pv.is_finite() || pv.abs() < 1e-18 {
            return None;
        }
        for v in m[c].iter_mut() {
            *v /= pv;
        }
        // ⚠️ A linha do pivô é COPIADA (um array de `f64` é `Copy`): sem isso o empréstimo de
        // `m[r]` e `m[c]` ao mesmo tempo não passa, e o laço por índice que o contornava é o que o
        // clippy acusa.
        let pivo = m[c];
        for (r, linha) in m.iter_mut().enumerate() {
            if r != c {
                let f = linha[c];
                for (dst, src) in linha.iter_mut().zip(pivo.iter()) {
                    *dst -= f * src;
                }
            }
        }
    }
    let mut out = [[0.0f64; TERMOS]; 2];
    for i in 0..TERMOS {
        out[0][i] = m[i][TERMOS];
        out[1][i] = m[i][TERMOS + 1];
    }
    out.iter()
        .flatten()
        .all(|c| c.is_finite())
        .then_some(out)
}

/// O ajuste de grau `1` sobre as mesmas amostras — a **linha de base que shipa**, e a resposta
/// quando a curvatura não pode ser confiada.
///
/// ⚠️ Ela não é um caso particular da [`ajusta`]: quando a malha é grossa demais para resolver a
/// dobra, a parte linear do ajuste CÚBICO vem contaminada pelas oscilações dele, e o que se quer é
/// a recta que as amostras determinam — não a recta que sobra depois de um cúbico ter puxado.
#[must_use]
pub(crate) fn ajusta_linear(entrada: &[[f32; 2]], saida: &[[f32; 2]]) -> Option<[[f64; 2]; 2]> {
    if entrada.len() != saida.len() || entrada.len() < 2 {
        return None;
    }
    let (mut s, mut c) = ([[0.0f64; 2]; 2], [[0.0f64; 2]; 2]);
    for (p, q) in entrada.iter().zip(saida.iter()) {
        let pv = [f64::from(p[0]), f64::from(p[1])];
        let qv = [f64::from(q[0]), f64::from(q[1])];
        for i in 0..2 {
            for j in 0..2 {
                s[i][j] += pv[i] * pv[j];
                c[i][j] += qv[i] * pv[j];
            }
        }
    }
    let det = s[0][0] * s[1][1] - s[0][1] * s[1][0];
    if !det.is_finite() || det.abs() < 1e-20 {
        return None;
    }
    let inv = [
        [s[1][1] / det, -s[0][1] / det],
        [-s[1][0] / det, s[0][0] / det],
    ];
    Some([
        [
            c[0][0] * inv[0][0] + c[0][1] * inv[1][0],
            c[0][0] * inv[0][1] + c[0][1] * inv[1][1],
        ],
        [
            c[1][0] * inv[0][0] + c[1][1] * inv[1][0],
            c[1][0] * inv[0][1] + c[1][1] * inv[1][1],
        ],
    ])
}
