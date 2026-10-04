//! ⭐⭐⭐⭐ **A VIZINHANÇA DA PEÇA** — os efeitos de vizinhança (desfoque, nitidez,
//! brilho, sombras/realces) borram NA SUPERFÍCIE (`docs/3D/30` §14, a W6).
//!
//! O compositor do Painter pergunta «que vizinhos tem uma amostra» por uma porta
//! só (`ph2d_tool_painter::Neighbourhood`): no 2D é a grelha da imagem; na peça é
//! a retícula das amostras como GRAFO (`ph2d_mesh_colors::vizinhanca`) com o
//! laplaciano de cotangentes (`ph2d_mesh_colors::difusao`) — o raio em UNIDADES
//! DA PEÇA, sem costura nas arestas da malha. A dobra `1024 × H` é só a forma do
//! buffer: ninguém borra ao longo dela.
//!
//! ⚠️ O laplaciano depende das POSIÇÕES: esculpir muda-o. A [`Impressao`] da
//! geometria (vértices, faces, os níveis e os bits das posições) diz quando ele
//! tem de ser refeito — nunca a meio de um traço de cor, que não move nada.

use std::sync::Arc;

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;
use ph2d_mesh_colors::difusao::Difusao;
use ph2d_tool_painter::{AdjustWindow, Neighbourhood, SpatialUnits, gaussian_sigma};

/// ⭐⭐ **O degrau mais fino em que os efeitos de vizinhança acompanham ao vivo**
/// (`64x`). O recurso é o tráfego de memória da placa: o grau do polinómio
/// dobra e as amostras quadruplicam a cada degrau. Medido (`docs/3D/30` §14, a
/// esfera de fábrica, um passo do arrasto de um gaussiano):
///
/// | degrau | raio `0,6 %` da diagonal | `2,5 %` (o fim do curso) |
/// |---|---|---|
/// | `32x` | `25,5 ms` | `52,7 ms` |
/// | `64x` | `162 ms` | `340 ms` |
/// | `128x` (estimado: grau `×2`, amostras `×4`) | `~1,3 s` | `~2,7 s` |
///
/// ⇒ acima de `64x` um único passo leva segundos: a porta recusa com a frase.
pub(crate) const NIVEL_MAX_DA_VIZINHANCA: u8 = 6;

/// ⭐ **Um ajuste de `kind` serve num plano de `nivel`?** — os que leem os
/// vizinhos só até [`NIVEL_MAX_DA_VIZINHANCA`]; os outros em qualquer degrau.
pub(crate) fn recusa_do_degrau(
    kind: ph2d_tool_painter::AdjustmentKind,
    nivel: u8,
) -> Option<crate::pilha_da_peca::RecusaDaPilha> {
    (kind.reads_the_image_layout() && nivel > NIVEL_MAX_DA_VIZINHANCA)
        .then_some(crate::pilha_da_peca::RecusaDaPilha::DegrauAlto)
}

/// Onde cada amostra do plano cai na peça (as unidades dela).
pub(crate) fn posicoes(t: &Tinta, mesh: &Mesh) -> Vec<[f32; 3]> {
    use ph2d_mesh_colors::amostragem::{posicao_quad, posicao_tri};
    let mut out = vec![[0.0; 3]; t.amostras().len()];
    let pos = mesh.positions();
    for (f, face) in mesh.faces().iter().enumerate() {
        let c = face.verts();
        let l = t.lado_da_face(f);
        let p = |k: usize| pos[c[k] as usize];
        if c.len() == 3 {
            t.para_cada_amostra_tri(f, c, |i, ijk| {
                out[i as usize] = posicao_tri(p(0), p(1), p(2), l, ijk);
            });
        } else {
            t.para_cada_amostra_quad(f, c, |i, ij| {
                out[i as usize] = posicao_quad([p(0), p(1), p(2), p(3)], l, ij);
            });
        }
    }
    out
}

/// ⭐ **As unidades dos raios na peça** — as dela (o mundo), mostradas em % da
/// diagonal da caixa dela (`SpatialUnits::Surface`).
pub(crate) fn unidades(mesh: &Mesh) -> SpatialUnits {
    let b = mesh.bounds();
    let d = [0, 1, 2].map(|k| b.max[k] - b.min[k]);
    let size = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    SpatialUnits::Surface {
        size: if size.is_finite() && size > 0.0 {
            size
        } else {
            1.0
        },
    }
}

/// A impressão da geometria de que o laplaciano saiu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Impressao {
    amostras: usize,
    verts: usize,
    faces: usize,
    hash: u64,
}

impl Impressao {
    /// A impressão de um plano sobre uma malha (FNV-1a sobre os níveis por face
    /// e os bits das posições — `O(V + F)`, ordens abaixo do plano).
    pub(crate) fn de(tinta: &Tinta, mesh: &Mesh) -> Self {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mistura = |v: u32| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        for p in mesh.positions() {
            for c in p {
                mistura(c.to_bits());
            }
        }
        for &k in tinta.topologia().niveis() {
            mistura(u32::from(k));
        }
        Self {
            amostras: tinta.amostras().len(),
            verts: mesh.vert_count(),
            faces: mesh.faces().len(),
            hash: h,
        }
    }
}

/// ⭐⭐⭐ **A retícula da peça como vizinhança do compositor.**
#[derive(Debug)]
pub(crate) struct VizinhancaDaPeca {
    difusao: Difusao,
    janela: AdjustWindow,
    impressao: Impressao,
}

impl VizinhancaDaPeca {
    /// O laplaciano da retícula de `tinta` sobre `mesh`, na dobra `largura × altura`.
    pub(crate) fn nova(tinta: &Tinta, mesh: &Mesh, (largura, altura): (u32, u32)) -> Self {
        Self {
            difusao: Difusao::nova(tinta, |f| mesh.faces()[f].verts(), mesh.positions()),
            janela: AdjustWindow::full(largura, altura),
            impressao: Impressao::de(tinta, mesh),
        }
    }

    /// Serve este plano sobre esta malha?
    pub(crate) fn serve(&self, impressao: &Impressao) -> bool {
        self.impressao == *impressao
    }

    /// O laplaciano (a placa sobe-o).
    pub(crate) fn difusao(&self) -> &Difusao {
        &self.difusao
    }

    /// O nome desta geometria para a placa (a mesma impressão = o mesmo grafo).
    pub(crate) fn chave(&self) -> u64 {
        self.impressao.hash ^ (self.impressao.amostras as u64).rotate_left(32)
    }
}

impl Neighbourhood for VizinhancaDaPeca {
    fn window(&self) -> AdjustWindow {
        self.janela
    }
    fn blur4(&self, radius: f32, buf: &mut [[f32; 4]]) {
        self.difusao.desfoca(gaussian_sigma(radius), buf);
    }
    fn blur1(&self, radius: f32, buf: &mut [f32]) {
        self.difusao.desfoca(gaussian_sigma(radius), buf);
    }
    fn image_plane(&self) -> bool {
        false
    }
}

/// A vizinhança guardada na pilha — estado da SESSÃO: duas pilhas são iguais
/// com ou sem ela (`PartialEq`), e cloná-la partilha o laplaciano.
#[derive(Clone, Debug, Default)]
pub(crate) struct NaPilha(pub(crate) Option<Arc<VizinhancaDaPeca>>);

impl PartialEq for NaPilha {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(test)]
#[path = "vizinhanca_da_peca_tests.rs"]
pub(crate) mod tests;
