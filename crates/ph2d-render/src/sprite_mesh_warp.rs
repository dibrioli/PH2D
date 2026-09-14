//! ⭐⭐⭐ **A DEFORMAÇÃO DEBAIXO DE UM DAB MEDE-SE AO TAMANHO DO DAB, nunca num ponto.**
//!
//! # O report que a pediu
//!
//! *«melhor. quase bom. Talvez artefato inevitável devido à natureza das deformações do mesh»*
//! (dono, 2026-09-14, terceira foto do mesmo pincel). As waves anteriores puseram a tinta no texel
//! certo (W10) e deram-lhe a forma que a deformação endireita (W11/W11b) — e o que sobrava era
//! real. ⛔ **Mas não era inevitável, e a medição diz porquê e quanto.**
//!
//! # O mecanismo
//!
//! Uma malha é **afim por triângulo**. Dentro de UM triângulo a deformação é constante e a elipse
//! que o pincel pinta volta ao ecrã como um disco **exacto**. Um dab que se estende por VÁRIOS
//! triângulos é corrigido pela deformação do triângulo debaixo do CENTRO, e as partes dele que
//! caem nos vizinhos recebem a correcção errada.
//!
//! ⇒ a grandeza certa não é a deformação **no ponto**, é a que a malha de facto faz **sobre o
//! disco que o dab ocupa**: o melhor afim (mínimos quadrados) do mapa da malha sobre esse disco.
//!
//! # ⭐ Porque ela degenera no de sempre
//!
//! Se todas as amostras do bordo caem no MESMO triângulo do centro, não há nada para ajustar — o
//! afim daquele triângulo já é exacto ali — e a porta devolve a resposta de hoje **sem tocar num
//! float**. É o que mantém byte a byte todo dab pequeno, que é o caso comum. *«Byte a byte» não é
//! uma promessa que uns mínimos quadrados cumpram: é um `if`* (a mesma lei que a
//! [`ph2d_painter_brush::canvas_warp`] pagou na W11).
//!
//! # A MEDIÇÃO (sonda sobre dois leques, 4 raios e 3 pontos — 24 células)
//!
//! Redondeza da marca no ecrã (`1,000` = disco perfeito), pior caso de cada coluna:
//!
//! | regime | sem correcção | facete (W11b) | **ao tamanho do dab** |
//! |---|---|---|---|
//! | dab DENTRO de um triângulo | `1,86`–`2,31` | `1,006` | `1,006` (o mesmo, ao bit) |
//! | dab sobre `~2` triângulos | `2,20` | `1,21` | **`1,11`** |
//! | dab sobre `~4` triângulos, leque forte | `1,19` | `1,38` | **`1,18`** |
//!
//! ⛔⛔ **A linha do fundo é a que obriga esta wave:** com um pincel GRANDE sobre uma malha grossa, a
//! correcção da W11b deixava a marca **menos redonda do que não corrigir nada** (`1,383` contra
//! `1,188`) — porque ela aplica ao dab inteiro a deformação de um pedaço dele. Medir ao tamanho do
//! dab tira esse caso; nas 24 células a resposta é **sempre melhor ou igual** à da facete.
//!
//! # O CUSTO, medido (`--release`, uma chamada por evento de ponteiro)
//!
//! | triângulos da malha | a deformação num PONTO | **ao tamanho do dab** |
//! |---|---|---|
//! | `128` (o `Fast`) | `0,1 µs` | `0,7 µs` |
//! | `2 048` | `0,8 µs` | `10,9 µs` |
//! | `7 688` (o tecto do `Smooth`) | `3,1 µs` | `40,7 µs` |
//!
//! ⚠️ **O recurso é a varredura dos triângulos**, e o pior caso é `0,24 %` de um quadro de 60 fps —
//! é por isso que ela pode ser UMA passagem e não `AMOSTRAS` delas, e é por isso que o rejeito por
//! caixa em UV existe. ⛔ Quem apontar (o picking, as caixas) passa `[0, 0]` e paga a coluna do
//! meio, que é o que se pagava antes desta wave.
//!
//! ⏳ **O que SOBRA depois disto** (`≈1,1`–`1,2` no pior regime) é inerente a **uma elipse por
//! dab**: sobre um footprint em que a deformação varia, nenhum afim único a descreve. Os dois
//! diminuidores são a malha mais fina (o `Smooth`) e o pincel menor — *é uma troca de resolução,
//! não uma parede*.

use crate::sprite_mesh::{SpriteMesh, barycentric, barycentric_raw, corners, uv_corners, warp_of};

/// Quantas amostras o bordo do footprint leva. ⚠️ **MEDIDO, não escolhido** (2026-09-14, a sonda
/// acima): `8` concorda com `32` a `±0,001` de redondeza nas **24** células, e `4` desvia até
/// `0,014`. O recurso é a contagem de testes ponto-em-triângulo por evento de ponteiro.
const AMOSTRAS: usize = 8;

/// O ponto LOCAL POSADO de `t` nos pesos `w`.
fn posado(mesh: &SpriteMesh, t: [usize; 3], w: [f32; 3]) -> [f32; 2] {
    let l = corners(mesh, t);
    [
        w[0] * l[0][0] + w[1] * l[1][0] + w[2] * l[2][0],
        w[0] * l[0][1] + w[1] * l[1][1] + w[2] * l[2][1],
    ]
}

/// ⭐⭐⭐ **A PORTA**: a deformação que a malha faz sobre o disco de raio `footprint_uv` (em UV de
/// REPOUSO, por eixo) à volta do ponto POSADO `p`. Ver o cabeçalho do módulo.
///
/// `footprint_uv` nulo ou não finito ⇒ a deformação da facete, que é a lei de [`warp_of`] e o que
/// as portas que não pintam (o picking, as caixas) querem.
#[must_use]
pub(crate) fn warp_over(
    mesh: &SpriteMesh,
    p: [f32; 2],
    size: [f32; 2],
    footprint_uv: [f32; 2],
) -> Option<[[f32; 2]; 2]> {
    if size[0] <= 0.0 || size[1] <= 0.0 {
        return None;
    }
    // ⚠️ **O triângulo é o do POSADO, e por isso a dobra obedece à mesma regra do desenho** (ganha o
    // desenhado por último) — é a razão de não se perguntar em UV quem contém o centro.
    let (t0, w0) = mesh
        .triangles()
        .rev()
        .find_map(|t| barycentric(p, corners(mesh, t)).map(|w| (t, w)))?;
    let facete = warp_of(mesh, t0, size)?;
    let (fx, fy) = (footprint_uv[0], footprint_uv[1]);
    if !(fx.is_finite() && fy.is_finite()) || fx <= 0.0 || fy <= 0.0 {
        return Some(facete);
    }
    let uv0 = uv_corners(mesh, t0);
    let centro = [
        w0[0] * uv0[0][0] + w0[1] * uv0[1][0] + w0[2] * uv0[2][0],
        w0[0] * uv0[0][1] + w0[1] * uv0[1][1] + w0[2] * uv0[2][1],
    ];
    // As amostras do bordo, em UV de repouso.
    let mut amostra_uv = [[0.0f32; 2]; AMOSTRAS];
    let mut amostra_duv = [[0.0f32; 2]; AMOSTRAS];
    for (k, (uv, duv)) in amostra_uv
        .iter_mut()
        .zip(amostra_duv.iter_mut())
        .enumerate()
    {
        let a = (k as f32) * std::f32::consts::TAU / (AMOSTRAS as f32);
        *duv = [fx * a.cos(), fy * a.sin()];
        *uv = [centro[0] + duv[0], centro[1] + duv[1]];
    }
    // ⭐ **UMA passagem sobre os triângulos** (o custo de um `uv_under`), com um rejeito por caixa
    // em UV à frente: sem ele seriam `AMOSTRAS` varreduras, e o `Smooth` refina a malha a milhares
    // de peças dentro do quadro.
    let (lo, hi) = (
        [centro[0] - fx, centro[1] - fy],
        [centro[0] + fx, centro[1] + fy],
    );
    let mut posicao = [[0.0f32; 2]; AMOSTRAS];
    let mut achou = [false; AMOSTRAS];
    let mut escapou = false;
    for t in mesh.triangles() {
        if t == t0 {
            continue;
        }
        let c = uv_corners(mesh, t);
        if c.iter().all(|q| q[0] < lo[0])
            || c.iter().all(|q| q[0] > hi[0])
            || c.iter().all(|q| q[1] < lo[1])
            || c.iter().all(|q| q[1] > hi[1])
        {
            continue;
        }
        for k in 0..AMOSTRAS {
            if achou[k] {
                continue;
            }
            if let Some(w) = barycentric(amostra_uv[k], c) {
                posicao[k] = posado(mesh, t, w);
                achou[k] = true;
                escapou = true;
            }
        }
    }
    // ⭐⭐⭐ **NADA escapou do triângulo do centro ⇒ a resposta de hoje, ao bit.** O afim dele é
    // exacto sobre o footprint inteiro, e uns mínimos quadrados sobre ele devolveriam o mesmo
    // número com o ruído de um `f32` por cima — que o atalho da identidade do pincel não perdoa.
    if !escapou {
        return Some(facete);
    }
    // ⚠️ **Uma amostra FORA da malha responde pelo afim do centro** — ela contribui exactamente o
    // que a facete já diz, logo a borda da arte comporta-se como hoje em vez de puxar o ajuste.
    for k in 0..AMOSTRAS {
        if !achou[k] {
            let Some(w) = barycentric_raw(amostra_uv[k], uv0) else {
                return Some(facete);
            };
            posicao[k] = posado(mesh, t0, w);
        }
    }
    let p0 = posado(mesh, t0, w0);
    // Mínimos quadrados: `M = (Σ q·pᵀ)·(Σ p·pᵀ)⁻¹`, com `p` na base de entrada da deformação
    // (`duv · size`) e `q` em coordenadas de ECRÃ (`y` para baixo — a mesma base das duas pontas,
    // que é o que a W11b pagou para aprender).
    let (mut s, mut c) = ([[0.0f32; 2]; 2], [[0.0f32; 2]; 2]);
    for k in 0..AMOSTRAS {
        let pv = [amostra_duv[k][0] * size[0], amostra_duv[k][1] * size[1]];
        let qv = [posicao[k][0] - p0[0], -(posicao[k][1] - p0[1])];
        for i in 0..2 {
            for j in 0..2 {
                s[i][j] += pv[i] * pv[j];
                c[i][j] += qv[i] * pv[j];
            }
        }
    }
    let det = s[0][0] * s[1][1] - s[0][1] * s[1][0];
    if !det.is_finite() || det.abs() < 1e-20 {
        return Some(facete);
    }
    let inv = [
        [s[1][1] / det, -s[0][1] / det],
        [-s[1][0] / det, s[0][0] / det],
    ];
    let m = [
        [
            c[0][0] * inv[0][0] + c[0][1] * inv[1][0],
            c[0][0] * inv[0][1] + c[0][1] * inv[1][1],
        ],
        [
            c[1][0] * inv[0][0] + c[1][1] * inv[1][0],
            c[1][0] * inv[0][1] + c[1][1] * inv[1][1],
        ],
    ];
    // ⛔ Um ajuste COLAPSADO (uma dobra que fecha o footprint sobre si mesmo) não é a resposta a
    // nada: ali a facete continua a ser o que se vê debaixo do cursor.
    let dm = m[0][0] * m[1][1] - m[0][1] * m[1][0];
    if !m.iter().flatten().all(|v| v.is_finite()) || dm.abs() < 1e-9 {
        return Some(facete);
    }
    Some(m)
}

#[cfg(test)]
#[path = "sprite_mesh_warp_tests.rs"]
mod tests;
