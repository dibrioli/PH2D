//! **AS DIREÇÕES PRINCIPAIS de curvatura, por FACE** — para onde a superfície
//! dobra mais, e quanto ela prefere essa direção às outras.
//!
//! Irmão do [`super::curvature`], e o corte é de espécie: lá mora um **escalar**
//! por vértice (quanto dobra, para a leitura de forma); aqui um **tensor** por
//! face (para que lado dobra). *São duas perguntas, e a segunda não se responde
//! com a primeira.*
//!
//! # Por que ela nasceu
//!
//! ⛔ **Porque a grade da retopologia não obedecia ao relevo, e a causa não era
//! afinação.** A energia do campo cruzado é
//! `Σ_e w_e (θ_f − θ_g + κ_e + (π/2)·p_e)²` — **só suavidade**. Não há um único
//! termo que puxe a cruz para a direção da curvatura, então o campo mais suave
//! sobre uma esfera com duas orelhas é o campo de uma esfera lisa: *ele não tem
//! como ver as orelhas.* Um alinhamento é um **termo**, não um ajuste.
//!
//! # O estimador
//!
//! Clean-room a partir de **Rusinkiewicz, *Estimating Curvatures and Their
//! Derivatives on Triangle Meshes*, 3DPVT 2004** — a segunda forma fundamental de
//! um triângulo lida das **diferenças de normal ao longo das três arestas**:
//!
//! ```text
//!     II · e_i  ≈  Δn_i           (i = 0,1,2, no plano tangente da face)
//! ```
//!
//! São seis equações para os três coeficientes de `II = [[a,b],[b,c]]`, resolvidas
//! por mínimos quadrados. Os autovetores de `II` são as direções principais.
//!
//! ⚠️ **Por FACE e não por vértice, e a escolha é do consumidor:** o campo cruzado
//! guarda um ângulo por **face** ([`ph2d_crossfield`]), então estimar por vértice
//! obrigaria a uma média que suaviza justamente a informação que se quer. *Quem
//! define a moldura define onde a direção mora.*

use crate::Mesh;

/// **A direção principal de uma face, e o quanto ela importa.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrincipalDir {
    /// A direção de curvatura **máxima**, no plano da face, normalizada.
    ///
    /// ⚠️ Ela é uma **direção sem sentido** (um eixo): `d` e `−d` dizem a mesma
    /// coisa, e o consumidor 4-RoSy trata as quatro rotações como equivalentes.
    pub dir: [f32; 3],
    /// ⭐ **A ANISOTROPIA, em `[0, 1]`** — `|k₁ − k₂| / (|k₁| + |k₂|)`.
    ///
    /// ⭐ **É a CONFIANÇA, e sem ela o alinhamento faz mal.** Numa esfera as duas
    /// curvaturas são iguais: não há direção preferida, e forçar uma põe uma
    /// costura onde a forma não pede nenhuma. *Um estimador que devolve sempre uma
    /// direção precisa de dizer quando ela não significa nada.*
    ///
    /// ⚠️ Normalizada de propósito: `|k₁ − k₂|` cru tem unidade `1/comprimento`,
    /// logo a mesma forma no dobro do tamanho daria metade do número — e o peso do
    /// alinhamento passaria a depender da escala da peça.
    pub anisotropy: f32,
}

impl Default for PrincipalDir {
    fn default() -> Self {
        Self {
            dir: [1.0, 0.0, 0.0],
            anisotropy: 0.0,
        }
    }
}

/// **AS DIREÇÕES PRINCIPAIS, uma por face.**
///
/// ⚠️ **Uma face degenerada devolve o [`PrincipalDir::default`]**, com anisotropia
/// zero — que é a resposta honesta: *não há direção ali*. Devolver a de um vizinho
/// seria inventar um dado que o consumidor não tem como distinguir de um medido.
#[must_use]
pub fn principal_dirs(mesh: &Mesh) -> Vec<PrincipalDir> {
    let pos = mesh.positions();
    let vn = mesh.normals();
    let fnormals = mesh.face_normals();
    mesh.faces()
        .iter()
        .enumerate()
        .map(|(f, face)| {
            let v = face.verts();
            if v.len() < 3 {
                return PrincipalDir::default();
            }
            let n = normalize(fnormals[f]);
            // A moldura da face: `u` ao longo da primeira aresta, `w = n × u`.
            let (p0, p1) = (pos[v[0] as usize], pos[v[1] as usize]);
            let u = normalize(sub(p1, p0));
            if norm(u) < 0.5 {
                return PrincipalDir::default();
            }
            let w = cross(n, u);

            // ⭐ As três arestas e as três diferenças de normal, na moldura — e o
            // ajuste vai pela PORTA ([`second_form`]), a mesma que a detecção de
            // feição usa sobre uma vizinhança de raio. ⚠️ A ordem de acumulação é a
            // mesma, então a saída é byte-idêntica à da versão em linha.
            let mut pairs = [([0.0f32; 2], [0.0f32; 2]); 3];
            for (k, slot) in pairs.iter_mut().enumerate() {
                let (a, b) = (v[k] as usize, v[(k + 1) % 3] as usize);
                let e = sub(pos[b], pos[a]);
                let dn = sub(vn[b], vn[a]);
                *slot = ([dot(e, u), dot(e, w)], [dot(dn, u), dot(dn, w)]);
            }
            let Some((k1, k2, [cu, cw])) = second_form(&pairs) else {
                return PrincipalDir::default();
            };
            let anisotropy = anisotropy_of(k1, k2);
            PrincipalDir {
                dir: normalize([
                    cu.mul_add(u[0], cw * w[0]),
                    cu.mul_add(u[1], cw * w[1]),
                    cu.mul_add(u[2], cw * w[2]),
                ]),
                anisotropy,
            }
        })
        .collect()
}

/// ⭐⭐⭐ **O NÚCLEO: a segunda forma fundamental, ajustada a pares `(aresta, salto de
/// normal)` numa moldura.**
///
/// ⚠️ **Ele existe como PORTA e não como conveniência.** A [`principal_dirs`] ajusta-a
/// sobre as **três arestas de um triângulo**; a detecção de feição
/// ([`crate::feature_dirs`]) ajusta-a sobre uma **vizinhança de raio `r`**. É a mesma
/// lei — `II·e ≈ Δn`, seis equações por três coeficientes — e escrevê-la duas vezes
/// seria tê-la a divergir no dia em que uma delas mudasse.
///
/// Devolve `(k₁, k₂, direcção de k₁ na moldura)`, ou `None` se o sistema for singular.
pub(crate) fn second_form(pairs: &[([f32; 2], [f32; 2])]) -> Option<(f32, f32, [f32; 2])> {
    let mut ata = [[0.0f64; 3]; 3];
    let mut atb = [0.0f64; 3];
    for &([eu, ew], [du, dw]) in pairs {
        // `II·(eu, ew) = (du, dw)` com `II = [[x0, x1], [x1, x2]]` dá duas linhas.
        for (row, rhs) in [([eu, ew, 0.0], du), ([0.0, eu, ew], dw)] {
            for i in 0..3 {
                for j in 0..3 {
                    ata[i][j] += f64::from(row[i]) * f64::from(row[j]);
                }
                atb[i] += f64::from(row[i]) * f64::from(rhs);
            }
        }
    }
    let x = solve3(ata, atb)?;
    #[allow(clippy::cast_possible_truncation)]
    let (a, b, c) = (x[0] as f32, x[1] as f32, x[2] as f32);
    let half = (a + c) * 0.5;
    let disc = (((a - c) * 0.5).powi(2) + b * b).max(0.0).sqrt();
    let (k1, k2) = (half + disc, half - disc);
    // O autovetor de `k1`, na moldura: `(b, k1 − a)` ou `(k1 − c, b)` —
    // ⚠️ o que tiver o maior módulo, senão o quase-isotrópico devolve zero.
    let (cu, cw) = if (k1 - a).abs() > (k1 - c).abs() {
        (b, k1 - a)
    } else {
        (k1 - c, b)
    };
    let len = cu.hypot(cw);
    (len >= 1.0e-12).then(|| (k1, k2, [cu / len, cw / len]))
}

/// ⭐ **A ANISOTROPIA relativa**, em `[0, 1]` — `|k₁ − k₂| / (|k₁| + |k₂|)`.
///
/// ⚠️ Porta única: [`principal_dirs`] e a detecção de feição leem a MESMA definição.
#[must_use]
pub(crate) fn anisotropy_of(k1: f32, k2: f32) -> f32 {
    let denom = k1.abs() + k2.abs();
    if denom > 1.0e-12 {
        ((k1 - k2).abs() / denom).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Gauss com pivô parcial sobre `3×3`; `None` quando o sistema é singular.
fn solve3(mut m: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    for i in 0..3 {
        let piv = (i..3).max_by(|&a, &c| m[a][i].abs().total_cmp(&m[c][i].abs()))?;
        if m[piv][i].abs() < 1.0e-14 {
            return None;
        }
        m.swap(i, piv);
        b.swap(i, piv);
        let (head, tail) = m.split_at_mut(i + 1);
        for (r, row) in tail.iter_mut().enumerate() {
            let f = row[i] / head[i][i];
            for c in i..3 {
                row[c] -= f * head[i][c];
            }
            b[i + 1 + r] -= f * b[i];
        }
    }
    let mut x = [0.0f64; 3];
    for i in (0..3).rev() {
        let mut s = b[i];
        for j in (i + 1)..3 {
            s -= m[i][j] * x[j];
        }
        x[i] = s / m[i][i];
    }
    Some(x)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = norm(a);
    if l < 1.0e-12 {
        [1.0, 0.0, 0.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}
