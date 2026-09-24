//! ⭐⭐⭐ **O RETRATO DA PEÇA** — a imagem com que a tela do Painter começa nos
//! modos que lêem a cor debaixo do pincel (borrar, esfregar, clonar, deformar,
//! o balde, a aquarela, os modos de mistura que não são o «over»).
//!
//! # Porque existe
//!
//! Na etapa 1 a tela começava TRANSPARENTE e a lei era a camada por cima da
//! peça ([`crate::tela_na_malha`]). Um borrão sobre uma tela transparente borra
//! o nada: o pincel lê a cor debaixo dele, e a cor debaixo dele estava na PEÇA.
//! ⇒ a tela começa com a peça desenhada nela, como o artista a vê, e a lei
//! passa a ser a DIFERENÇA entre o que a tela ficou e este retrato.
//!
//! # ⚠️ É um rasterizador de CPU, e não uma leitura da placa
//!
//! Correr aqui dá três coisas que a placa não dá: é **gateável sem adaptador**
//! (a lei e o retrato vivem na mesma crate e medem-se juntos), **não pede um
//! `wgpu::Device`** à costura, e o retrato sai no MESMO espaço de cor das
//! amostras (bytes sRGB = cor × 255, a convenção da tela do Painter), sem a
//! conversão do alvo de cor do renderizador pelo meio.
//!
//! # A lei, por píxel
//!
//! Uma face entra se estiver de FRENTE para o olho; cada triângulo dela
//! (um quad parte-se em `(0,1,2)` e `(0,2,3)`) cobre os centros de píxel
//! `(i + 0,5, j + 0,5)` com as coordenadas baricêntricas do ECRÃ, e a
//! profundidade é `1/w` interpolado — linear no ecrã, maior é mais perto. A cor
//! lê-se com as baricêntricas **corrigidas pela perspectiva**, pela porta que o
//! renderizador e o oráculo já partilham ([`ph2d_mesh_colors::Tinta::cor_tri`] ·
//! [`ph2d_mesh_colors::Tinta::cor_quad`]), ou da cor por vértice sem plano.
//! Fora da silhueta a tela fica TRANSPARENTE.
//!
//! ⚠️ **O retrato só precisa de ser CONSISTENTE, não exacto:** o que o pincel
//! não tocou anula-se na diferença por construção (os mesmos bytes dos dois
//! lados), logo um erro do retrato só aparece onde o pincel ARRASTA cor — e aí
//! ele arrasta a cor que o artista estava a ver.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

use crate::tela_na_malha::{Vista, de_frente};

/// ⭐ **O retrato de `mesh` visto por `vista`**, RGBA8 não pré-multiplicado,
/// `largura·altura·4` bytes — o formato da tela do Painter.
#[must_use]
pub fn semente(mesh: &Mesh, tinta: Option<&Tinta>, vista: &Vista) -> Vec<u8> {
    let (w, h) = vista.tamanho();
    let (wu, hu) = (w as usize, h as usize);
    let mut rgba = vec![0u8; wu * hu * 4];
    let mut perto = vec![0.0f32; wu * hu];
    let pos = mesh.positions();
    let cores = mesh.colors();
    let projectados: Vec<Option<([f32; 2], f32)>> =
        pos.iter().map(|&p| vista.ecra_e_inverso(p)).collect();
    for (fi, face) in mesh.faces().iter().enumerate() {
        let cantos = face.verts();
        if !de_frente(pos, cantos, vista.olho()) {
            continue;
        }
        let tris: &[[usize; 3]] = if cantos.len() == 3 {
            &[[0, 1, 2]]
        } else {
            &[[0, 1, 2], [0, 2, 3]]
        };
        for t in tris {
            let Some(ps) = t
                .iter()
                .map(|&c| projectados[cantos[c] as usize])
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let cor = |bar: [f32; 3]| -> [f32; 3] {
                if let Some(tinta) = tinta {
                    if cantos.len() == 3 {
                        return tinta.cor_tri(fi, cantos, bar);
                    }
                    // O uv de cada canto do quad, e o do ponto por baricêntricas.
                    const UV: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
                    let uv = [
                        bar[0] * UV[t[0]][0] + bar[1] * UV[t[1]][0] + bar[2] * UV[t[2]][0],
                        bar[0] * UV[t[0]][1] + bar[1] * UV[t[1]][1] + bar[2] * UV[t[2]][1],
                    ];
                    return tinta.cor_quad(fi, cantos, uv);
                }
                let mut o = [0.0f32; 3];
                for (k, &c) in t.iter().enumerate() {
                    let v = cantos[c] as usize;
                    let cv = cores.map_or(ph2d_mesh::DEFAULT_COLOR, |cs| cs[v]);
                    for e in 0..3 {
                        o[e] += cv[e] * bar[k];
                    }
                }
                o
            };
            rasteriza(&ps, (w, h), &mut perto, &mut rgba, cor);
        }
    }
    rgba
}

/// Um triângulo de ecrã `[(ponto, 1/w); 3]` para dentro da tela.
fn rasteriza(
    ps: &[([f32; 2], f32)],
    (w, h): (u32, u32),
    perto: &mut [f32],
    rgba: &mut [u8],
    cor: impl Fn([f32; 3]) -> [f32; 3],
) {
    let (a, b, c) = (ps[0].0, ps[1].0, ps[2].0);
    let area = aresta(a, b, c);
    if area.abs() <= f32::EPSILON {
        return;
    }
    let x0 = a[0].min(b[0]).min(c[0]).floor().max(0.0) as u32;
    let y0 = a[1].min(b[1]).min(c[1]).floor().max(0.0) as u32;
    let x1 = (a[0].max(b[0]).max(c[0]).ceil().max(0.0) as u32).min(w);
    let y1 = (a[1].max(b[1]).max(c[1]).ceil().max(0.0) as u32).min(h);
    for j in y0..y1 {
        for i in x0..x1 {
            let p = [i as f32 + 0.5, j as f32 + 0.5];
            let l = [
                aresta(b, c, p) / area,
                aresta(c, a, p) / area,
                aresta(a, b, p) / area,
            ];
            if l[0] < 0.0 || l[1] < 0.0 || l[2] < 0.0 {
                continue;
            }
            let inv = l[0] * ps[0].1 + l[1] * ps[1].1 + l[2] * ps[2].1;
            let o = j as usize * w as usize + i as usize;
            if inv <= perto[o] {
                continue;
            }
            perto[o] = inv;
            // As baricêntricas corrigidas pela perspectiva.
            let bar = [
                l[0] * ps[0].1 / inv,
                l[1] * ps[1].1 / inv,
                l[2] * ps[2].1 / inv,
            ];
            let c = cor(bar);
            let px = &mut rgba[o * 4..o * 4 + 4];
            for e in 0..3 {
                px[e] = (c[e].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
            px[3] = 255;
        }
    }
}

/// O dobro da área com sinal de `(a, b, p)`.
fn aresta(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}
