//! `ph2d-field-render` — **o traçador**: marcha raios contra o campo e devolve o que a superfície
//! de facto é ([ADR-0161] §2).
//!
//! # Por que a tela não passa pela malha
//!
//! Medido na W0 (`docs/3DModeling/01_resultados_spike.md` §1c): traçando o campo, a quina do cubo
//! sai como uma navalha e o filete sai liso; a **mesma** cena extraída em malha serrilha. A
//! geometria estava certa e o defeito era inteiramente da extração. Deixar a malha desenhar a tela
//! seria deixar **o caminho pior definir o teto do que se vê** — o que o [`CLAUDE.md §0`] proíbe.
//!
//! # O que sai daqui é GEOMETRIA, não cor
//!
//! [`trace`] devolve um G-buffer: máscara, **normal em espaço de vista** e profundidade. Nenhuma
//! decisão de cor mora nele. Quem quiser pixels passa um [`Matcap`] a [`shade`].
//!
//! ⚠️ **Matcap, e não o rig de lâmpadas** — e a distinção é da casa, não minha:
//! *"o rig é do DOCUMENTO (a mesma lâmpada acende a tinta ao lado), o matcap é do OLHO"*
//! (`ph2d-mesh-render::matcap`). Um viewport de modelagem lê **forma**, e forma se lê com a luz
//! presa à câmera. Inventar um segundo rig aqui seria exatamente o erro que a `ph2d-light` existe
//! para impedir.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [`CLAUDE.md §0`]: ../../../CLAUDE.md

use ph2d_field::FieldDoc;
use ph2d_field_eval::Engine;
use rayon::prelude::*;

/// `1/√2` — o passo seguro da marcha.
///
/// ⚠️ **Não é um fator de conforto: é o recíproco de uma constante MEDIDA.** A W0 mediu ‖∇f‖
/// chegando a **√2** no operador de arredondamento exato (`01_resultados_spike.md` §3), onde duas
/// superfícies se tocam quase tangentes. Avançar `d` num campo assim **atravessa** a superfície, e
/// o furo aparece como pixel de fundo no meio da peça.
const SAFE_STEP: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// Distância abaixo da qual o raio considerou-se na superfície.
const HIT_EPS: f32 = 2.0e-4;
/// Quanto um raio anda antes de desistir.
const T_MAX: f32 = 8.0;
const MAX_STEPS: usize = 400;
/// Passo da diferença central que devolve a normal.
const NORMAL_EPS: f32 = 1.0e-4;

/// A câmera em órbita. **Ortográfica** — ver a nota de [`Orbit::basis`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    /// Giro em torno de Y, em radianos.
    pub yaw: f32,
    /// Elevação, em radianos.
    pub pitch: f32,
    /// Quantas unidades de mundo cabem em meia altura de tela. Menor = mais perto.
    pub half_extent: f32,
    /// O ponto que fica no centro do quadro.
    pub target: [f32; 3],
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            // Três-quartos, ligeiramente por cima: o ângulo em que uma aresta viva e um filete se
            // distinguem sem ambiguidade (escolhido na W0, ao olhar as imagens).
            yaw: 0.72,
            pitch: 0.52,
            half_extent: 0.8,
            target: [0.0; 3],
        }
    }
}

impl Orbit {
    /// A base ortonormal da câmera: `(direita, cima, para-o-observador)`.
    ///
    /// ⚠️ **Projeção ortográfica**, e isso é uma escolha com data: é a que a W0 validou, e é a que
    /// o matcap pressupõe (`ph2d-mesh-render::matcap` amostra pela normal de vista, com a vista em
    /// `(0,0,1)`). Perspectiva é item ABERTO — ela muda o *feel* de um modelador e merece a sua
    /// própria comparação lado a lado, não uma troca silenciosa.
    ///
    /// ⚠️ O `sin`/`cos` aqui **não** fere o HR-5: a câmera é estado de VISTA — não entra no
    /// documento salvo, não entra no undo e não entra em hash de replay nenhum.
    #[must_use]
    pub fn basis(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let fwd = [cp * sy, sp, cp * cy]; // do alvo para o observador
        let right = [cy, 0.0, -sy];
        let up = [
            fwd[1] * right[2] - fwd[2] * right[1],
            fwd[2] * right[0] - fwd[0] * right[2],
            fwd[0] * right[1] - fwd[1] * right[0],
        ];
        (right, up, fwd)
    }
}

/// O que o traçado sabe da superfície, por pixel. **Sem cor.**
pub struct Gbuffer {
    pub width: u32,
    pub height: u32,
    /// O raio encontrou superfície?
    pub hit: Vec<bool>,
    /// Normal em **espaço de vista** (`z` para o observador), unitária. Lixo onde `!hit`.
    pub normal: Vec<[f32; 3]>,
}

impl Gbuffer {
    #[must_use]
    pub fn hits(&self) -> usize {
        self.hit.iter().filter(|h| **h).count()
    }
}

/// Marcha o campo do documento e devolve o G-buffer.
#[must_use]
pub fn trace(doc: &FieldDoc, cam: &Orbit, width: u32, height: u32) -> Gbuffer {
    trace_with_threads(doc, cam, width, height, true)
}

/// Igual a [`trace`], com o paralelismo sob controle — é o que o gate de byte-identidade dirige.
#[must_use]
pub fn trace_with_threads(
    doc: &FieldDoc,
    cam: &Orbit,
    width: u32,
    height: u32,
    parallel: bool,
) -> Gbuffer {
    let tree = ph2d_field_eval::compile(doc);
    let shape = Engine::from(tree);
    let (right, up, fwd) = cam.basis();
    let (w, h) = (width as usize, height as usize);
    let half = (w.min(h) as f32) * 0.5;

    let row = |y: usize| -> (Vec<bool>, Vec<[f32; 3]>) {
        // Um avaliador POR FATIA: a `fidget` precisa de estado mutável para avaliar, e partilhá-lo
        // entre threads exigiria trava. Criar o próprio é barato e mantém a escrita disjunta, que é
        // a condição do ADR-0109.
        let tape = shape.float_slice_tape(Default::default());
        let mut eval = Engine::new_float_slice_eval();
        let mut hit = vec![false; w];
        let mut normal = vec![[0.0f32; 3]; w];

        // Origem de cada raio da fatia, e a marcha em lockstep.
        let mut ox = vec![0.0f32; w];
        let mut oy = vec![0.0f32; w];
        let mut oz = vec![0.0f32; w];
        for (x, ((px, py), pz)) in ox
            .iter_mut()
            .zip(oy.iter_mut())
            .zip(oz.iter_mut())
            .enumerate()
        {
            let sx = (x as f32 + 0.5 - w as f32 * 0.5) / half * cam.half_extent;
            let sy = -(y as f32 + 0.5 - h as f32 * 0.5) / half * cam.half_extent;
            const START: f32 = 4.0;
            *px = cam.target[0] + right[0] * sx + up[0] * sy + fwd[0] * START;
            *py = cam.target[1] + right[1] * sx + up[1] * sy + fwd[1] * START;
            *pz = cam.target[2] + right[2] * sx + up[2] * sy + fwd[2] * START;
        }
        let dir = [-fwd[0], -fwd[1], -fwd[2]];

        let mut t = vec![0.0f32; w];
        let mut alive: Vec<u32> = (0..w as u32).collect();
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..MAX_STEPS {
            if alive.is_empty() {
                break;
            }
            xs.clear();
            ys.clear();
            zs.clear();
            for &i in &alive {
                let i = i as usize;
                xs.push(ox[i] + dir[0] * t[i]);
                ys.push(oy[i] + dir[1] * t[i]);
                zs.push(oz[i] + dir[2] * t[i]);
            }
            let Ok(out) = eval.eval(&tape, &xs, &ys, &zs) else {
                break;
            };
            let mut next = Vec::with_capacity(alive.len());
            for (k, &i) in alive.iter().enumerate() {
                let iu = i as usize;
                let d = out[k];
                if d < HIT_EPS {
                    hit[iu] = true;
                    continue;
                }
                t[iu] += d * SAFE_STEP;
                if t[iu] < T_MAX {
                    next.push(i);
                }
            }
            alive = next;
        }

        // Normais por diferença central, em lote, só nos pixels que acertaram.
        let idx: Vec<usize> = (0..w).filter(|i| hit[*i]).collect();
        if !idx.is_empty() {
            let mut gx = Vec::with_capacity(idx.len() * 6);
            let mut gy = Vec::with_capacity(idx.len() * 6);
            let mut gz = Vec::with_capacity(idx.len() * 6);
            for &i in &idx {
                let (px, py, pz) = (
                    ox[i] + dir[0] * t[i],
                    oy[i] + dir[1] * t[i],
                    oz[i] + dir[2] * t[i],
                );
                for (dx, dy, dz) in [
                    (NORMAL_EPS, 0.0, 0.0),
                    (-NORMAL_EPS, 0.0, 0.0),
                    (0.0, NORMAL_EPS, 0.0),
                    (0.0, -NORMAL_EPS, 0.0),
                    (0.0, 0.0, NORMAL_EPS),
                    (0.0, 0.0, -NORMAL_EPS),
                ] {
                    gx.push(px + dx);
                    gy.push(py + dy);
                    gz.push(pz + dz);
                }
            }
            if let Ok(g) = eval.eval(&tape, &gx, &gy, &gz) {
                for (k, &i) in idx.iter().enumerate() {
                    let b = k * 6;
                    let world = [g[b] - g[b + 1], g[b + 2] - g[b + 3], g[b + 4] - g[b + 5]];
                    let len =
                        (world[0] * world[0] + world[1] * world[1] + world[2] * world[2]).sqrt();
                    if len <= 0.0 {
                        hit[i] = false;
                        continue;
                    }
                    let n = [world[0] / len, world[1] / len, world[2] / len];
                    // Para o espaço de VISTA — é nele que o matcap vive.
                    normal[i] = [
                        n[0] * right[0] + n[1] * right[1] + n[2] * right[2],
                        n[0] * up[0] + n[1] * up[1] + n[2] * up[2],
                        n[0] * fwd[0] + n[1] * fwd[1] + n[2] * fwd[2],
                    ];
                }
            }
        }
        (hit, normal)
    };

    let rows: Vec<(Vec<bool>, Vec<[f32; 3]>)> = if parallel {
        (0..h).into_par_iter().map(row).collect()
    } else {
        (0..h).map(row).collect()
    };

    let mut hit = Vec::with_capacity(w * h);
    let mut normal = Vec::with_capacity(w * h);
    for (rh, rn) in rows {
        hit.extend(rh);
        normal.extend(rn);
    }
    Gbuffer {
        width,
        height,
        hit,
        normal,
    }
}

/// Os texels de um matcap, **em linear**, lado × lado, RGB.
///
/// ⚠️ Quem os fornece é o chamador — ver a nota do `Cargo.toml` sobre não arrastar o `wgpu` para
/// dentro de um traçador de CPU.
pub struct Matcap<'a> {
    pub side: u32,
    /// `side * side * 3` valores lineares.
    pub rgb_linear: &'a [f32],
}

/// Colore o G-buffer com um matcap e devolve RGBA8.
///
/// A lei de amostragem é a do matcap: `uv = n.xy * 0.5 + 0.5`, com `n` em espaço de vista. É por
/// isso que ela mora aqui, ao lado de quem produz essa normal — e não do outro lado do repositório,
/// onde a convenção teria de ser re-afirmada num comentário.
#[must_use]
pub fn shade(g: &Gbuffer, m: &Matcap<'_>, background: [u8; 4]) -> Vec<u8> {
    let side = m.side as usize;
    let mut out = vec![0u8; (g.width as usize) * (g.height as usize) * 4];
    for (i, px) in out.chunks_exact_mut(4).enumerate() {
        if !g.hit[i] || side == 0 {
            px.copy_from_slice(&background);
            continue;
        }
        let n = g.normal[i];
        let u = (n[0] * 0.5 + 0.5).clamp(0.0, 1.0);
        let v = (1.0 - (n[1] * 0.5 + 0.5)).clamp(0.0, 1.0);
        let tx = ((u * side as f32) as usize).min(side - 1);
        let ty = ((v * side as f32) as usize).min(side - 1);
        let t = (ty * side + tx) * 3;
        px[0] = ph2d_color::srgb::linear_to_srgb_byte(m.rgb_linear[t]);
        px[1] = ph2d_color::srgb::linear_to_srgb_byte(m.rgb_linear[t + 1]);
        px[2] = ph2d_color::srgb::linear_to_srgb_byte(m.rgb_linear[t + 2]);
        px[3] = 255;
    }
    out
}

#[cfg(test)]
mod tests;
