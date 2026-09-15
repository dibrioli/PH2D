//! **A DEFORMAÇÃO DO CANVAS sob o dab** — a arte pode estar dobrada por um esqueleto, e o pincel
//! tem de sair redondo **no ecrã**, não no espaço da textura.
//!
//! # O defeito, com foto (report do dono, 2026-09-14)
//!
//! *«o local é correto mas o pincel não considera o resultante das deformações do mesh … o pincel é
//! redondo mas pinta como se os polígonos não estivessem deformados»*. A pele desenha a imagem por
//! triângulos, e onde o leque comprime a arte um disco de textura chega ao ecrã como uma **lasca**.
//! O anel do cursor mostra um círculo; a tinta saía uma fatia fina.
//!
//! # A lei
//!
//! O artista pede um disco de raio `R` **no ecrã**. Se `W` é a deformação local (ecrã por textura,
//! adimensional — identidade em repouso), o que tem de ser pintado na TEXTURA é `W⁻¹` aplicado a
//! esse disco: uma **elipse**. E uma elipse é exactamente o que o dab já sabe ser — o par
//! *Flatten + Angle* do gizmo de Shape, mais o raio.
//!
//! ⇒ esta porta decompõe `W⁻¹ · E` (com `E` = a elipse que o artista autorou) nos três números que o
//! motor consome: **raio**, **achatamento** e **ângulo**. ⭐ **Nenhum tipo novo chega ao kernel**: o
//! `FootprintDeform` e o `radius_px` são os de sempre, e em repouso (`W = I`) a conta devolve o que
//! entrou, **ao bit**.
//!
//! # ⚠️ Sem transcendentais, e a razão é a mesma do resto do módulo (HR-5)
//!
//! Um `atan2` aqui devolveria um ângulo que podia arredondar para graus diferentes em plataformas
//! diferentes, e o `dab_angle_deg` é um **inteiro** que escolhe uma entrada da tabela cozida — um
//! grau de diferença é tinta diferente, e o hash de replay mede isso. A decomposição usa só
//! multiplicações e `sqrt` (exacta em IEEE), e o ângulo sai por **procura na própria tabela**
//! ([`crate::texture::rotate_by_degrees`]): o grau cujo vector está mais perto do eixo maior.

/// ⭐⭐⭐ **A DEFORMAÇÃO DA ARTE DEBAIXO DO DAB** — a matriz local **mais a curvatura**.
///
/// ⛔⛔ **A segunda metade existe porque a primeira tem TECTO, e ele foi medido** (report do dono,
/// 4.ª foto: *«quase bom»*). Um mapa linear não acompanha uma dobra, e o desvio da marca **não é
/// facetagem** — refinar a malha de `32` para `8 192` triângulos deixa-o onde estava. Ele é a
/// curvatura dentro do próprio dab, e cresce com o raio do pincel: `1,054` · `1,118` · `1,197` para
/// raios de `0,06` · `0,125` · `0,20`. Com os graus `2` e `3`: `1,008` · `1,005` · **`1,006`**.
/// Tabela e sondas em [`ph2d_render::sprite_mesh_warp`] (a porta que a mede).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasWarp {
    /// Ecrã por textura, adimensional — **identidade em repouso**. O que sempre viajou aqui.
    pub linear: [[f32; 2]; 2],
    /// Os graus `2` e `3` (`x²`, `x·y`, `y²`, `x³`, `x²·y`, `x·y²`, `y³`), com a entrada em
    /// **raios do footprint** e a saída na mesma escala de `linear`. ⛔ **Zero em repouso**, e é
    /// isso que mantém toda pincelada deste app byte a byte como era.
    pub curve: [[f32; 7]; 2],
}

impl CanvasWarp {
    /// A arte em repouso: sem deformação e sem dobra.
    #[must_use]
    pub const fn rest() -> Self {
        Self {
            linear: [[1.0, 0.0], [0.0, 1.0]],
            curve: [[0.0; 7]; 2],
        }
    }

    /// Só a matriz local, sem curvatura — o que as portas que **apontam** (o picking, as caixas)
    /// sabem responder.
    #[must_use]
    pub const fn linear(linear: [[f32; 2]; 2]) -> Self {
        Self {
            linear,
            curve: [[0.0; 7]; 2],
        }
    }

    /// Nenhuma dobra declarada ⇒ o caminho linear de sempre.
    #[must_use]
    pub fn is_straight(&self) -> bool {
        self.curve.iter().flatten().all(|c| *c == 0.0)
    }
}

/// A elipse que o motor tem de pintar, nos números que ele já consome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarpedDab {
    /// Multiplicador do raio do dab (o semi-eixo MAIOR, em px de imagem). `1` em repouso.
    pub radius_scale: f32,
    /// O `dab_flatten` composto (`0` = redondo).
    pub flatten: f32,
    /// O `dab_angle_deg` composto — o eixo MAIOR da elipse.
    pub angle_deg: u16,
    /// ⭐ A curvatura que sobra depois de a elipse dar o que podia dar, já no referencial da pegada
    /// ([`super::canvas_warp_curve`]). Plana sempre que a arte não está dobrada.
    pub curve: crate::FootprintCurve,
}

impl WarpedDab {
    /// O no-op: raio inalterado, redondo, a zero graus.
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            radius_scale: 1.0,
            flatten: 0.0,
            angle_deg: 0,
            curve: crate::FootprintCurve::flat(),
        }
    }
}

/// A matriz `2×2` da elipse autorada — a que leva o disco unitário ao dab que o artista vê em
/// repouso. Mesma convenção do [`crate::footprint::FootprintDeform`]: semi-eixo `1` na direcção do
/// ângulo, `1 − flatten` na perpendicular.
fn authored_matrix(flatten: f32, angle_deg: u16) -> [[f32; 2]; 2] {
    let [c, s] = crate::texture::rotate_by_degrees(angle_deg);
    let m = 1.0 - flatten.clamp(0.0, crate::footprint::DAB_FLATTEN_MAX);
    // Colunas: o eixo maior (c, s) e o menor (−s, c)·m.
    [[c, -s * m], [s, c * m]]
}

/// ⭐⭐⭐ **A PORTA**: os três números que pintam um disco de raio `radius_px` **no ecrã** através de
/// uma deformação local `warp` (ecrã por textura), respeitando a elipse que o artista autorou.
///
/// `warp` é a identidade fora de uma arte deformada ⇒ devolve `(1, flatten, angle_deg)` **ao bit**,
/// e é isso que mantém toda pincelada deste app byte a byte como era.
///
/// ⛔ Uma `warp` degenerada (determinante ~0 — um triângulo colapsado) devolve a identidade: um dab
/// infinito não é a resposta a uma malha dobrada sobre si mesma.
#[must_use]
pub fn warped_dab(warp: CanvasWarp, flatten: f32, angle_deg: u16) -> WarpedDab {
    // ⭐⭐⭐ **O ATALHO DA IDENTIDADE é LOAD-BEARING, e o gate apanhou-o:** sem ele a decomposição
    // devolvia `radius_scale = 1,0000006` e `flatten = 0,39999998` sobre uma arte em REPOUSO —
    // números plausíveis e **outra tinta**, em toda pincelada do app. *«Byte a byte» não é uma
    // promessa que uma raiz quadrada cumpra: é um `if`.*
    if warp.linear == [[1.0, 0.0], [0.0, 1.0]] && warp.is_straight() {
        return WarpedDab {
            radius_scale: 1.0,
            flatten,
            angle_deg,
            curve: crate::FootprintCurve::flat(),
        };
    }
    let linear = warp.linear;
    let det = linear[0][0] * linear[1][1] - linear[0][1] * linear[1][0];
    if !det.is_finite() || det.abs() < 1e-6 {
        return WarpedDab {
            radius_scale: 1.0,
            flatten,
            angle_deg,
            curve: crate::FootprintCurve::flat(),
        };
    }
    // `W⁻¹` — o que a textura tem de ter para o ecrã ver o disco.
    let inv = [
        [linear[1][1] / det, -linear[0][1] / det],
        [-linear[1][0] / det, linear[0][0] / det],
    ];
    let e = authored_matrix(flatten, angle_deg);
    // `A = W⁻¹ · E`: do disco unitário para a elipse a pintar na textura.
    let a = [
        [
            inv[0][0] * e[0][0] + inv[0][1] * e[1][0],
            inv[0][0] * e[0][1] + inv[0][1] * e[1][1],
        ],
        [
            inv[1][0] * e[0][0] + inv[1][1] * e[1][0],
            inv[1][0] * e[0][1] + inv[1][1] * e[1][1],
        ],
    ];
    // Os eixos da elipse são os autovectores de `A·Aᵀ` (simétrica, semi-definida positiva) e os
    // semi-eixos as raízes dos autovalores — ⚠️ tudo com `sqrt`, nenhum transcendental.
    let p = a[0][0] * a[0][0] + a[0][1] * a[0][1];
    let q = a[0][0] * a[1][0] + a[0][1] * a[1][1];
    let r = a[1][0] * a[1][0] + a[1][1] * a[1][1];
    let tr = p + r;
    let disc = ((p - r) * (p - r) + 4.0 * q * q).max(0.0).sqrt();
    let (l1, l2) = (((tr + disc) * 0.5).max(0.0), ((tr - disc) * 0.5).max(0.0));
    let (s1, s2) = (l1.sqrt(), l2.sqrt());
    if s1 <= 0.0 {
        return WarpedDab::identity();
    }
    // O autovector do MAIOR: `(q, λ1 − p)`, ou o eixo X quando a matriz já é diagonal.
    let v = if q.abs() > 1e-12 {
        [q, l1 - p]
    } else if p >= r {
        [1.0, 0.0]
    } else {
        [0.0, 1.0]
    };
    let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let dir = if n > 0.0 {
        [v[0] / n, v[1] / n]
    } else {
        [1.0, 0.0]
    };
    let flatten_final = (1.0 - s2 / s1).clamp(0.0, crate::footprint::DAB_FLATTEN_MAX);
    let angulo = nearest_degree(dir);
    WarpedDab {
        radius_scale: s1,
        // ⚠️ A cerca é a do próprio footprint: uma lasca infinitamente fina não é pintável.
        flatten: flatten_final,
        angle_deg: angulo,
        curve: curvatura(warp.curve, e, linear, s1, flatten_final, angulo),
    }
}

/// A curvatura levada ao referencial da pegada — ver [`super::canvas_warp_curve`], que tem a conta
/// inteira e o porquê do `V`.
///
/// ⛔ **Sem dobra declarada a resposta é PLANA sem tocar num float** — o atalho que mantém byte a
/// byte toda pincelada sobre arte que não está dobrada.
fn curvatura(
    curve: [[f32; 7]; 2],
    e: [[f32; 2]; 2],
    linear: [[f32; 2]; 2],
    s1: f32,
    flatten: f32,
    angle_deg: u16,
) -> crate::FootprintCurve {
    use super::canvas_warp_curve as cc;
    if curve.iter().flatten().all(|c| *c == 0.0) {
        return crate::FootprintCurve::flat();
    }
    let Some(e_inv) = cc::inverte(e) else {
        return crate::FootprintCurve::flat();
    };
    // `G₁ = s₁·E⁻¹·L` — a parte linear do mapa exacto, no referencial do amostrador.
    let g1_raw = cc::produto(e_inv, linear);
    let g1 = [
        [g1_raw[0][0] * s1, g1_raw[0][1] * s1],
        [g1_raw[1][0] * s1, g1_raw[1][1] * s1],
    ];
    let rot = crate::texture::rotate_by_degrees(angle_deg);
    let inv_minor = 1.0 / (1.0 - flatten);
    if !inv_minor.is_finite() {
        return crate::FootprintCurve::flat();
    }
    let composta = cc::compoe(curve, e_inv, cc::v_transposta(g1, rot, inv_minor), s1, rot);
    // ⛔⛔ **A CERCA:** uma dobra violenta faz o mapa dobrar sobre si mesmo dentro do dab, e aí a
    // curvatura sai PIOR do que não corrigir nada — o erro que a W11b já pagou com a facete. A
    // pegada tem de continuar amostrável; se não estiver, a elipse é a resposta honesta.
    // Ver [`crate::FootprintDeform::is_sampleable`], que tem o número.
    if crate::FootprintDeform::new(flatten, angle_deg)
        .with_curve(composta)
        .is_sampleable()
    {
        composta
    } else {
        crate::FootprintCurve::flat()
    }
}

/// O grau inteiro cujo vector cozido está mais perto de `dir` — ⚠️ a procura EXISTE para não haver
/// `atan2` no caminho que escolhe tinta (ver o cabeçalho). `360` comparações, uma vez por traço.
fn nearest_degree(dir: [f32; 2]) -> u16 {
    let mut best = (f32::NEG_INFINITY, 0u16);
    for d in 0..360u16 {
        let [c, s] = crate::texture::rotate_by_degrees(d);
        let dot = c * dir[0] + s * dir[1];
        if dot > best.0 {
            best = (dot, d);
        }
    }
    best.1
}

#[cfg(test)]
#[path = "canvas_warp_tests.rs"]
mod tests;
